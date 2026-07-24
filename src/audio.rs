//! Autonomous audio subsystem boundary.
//!
//! The public surface is deliberately small: the execution engine sends high-level
//! commands, while the UI receives a lock-free telemetry consumer. Runtime, decoder
//! tasks, and the CPAL-owned thread remain implementation details here.

use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::{HeapCons, HeapProd, HeapRb};
use tokio::sync::{Notify, mpsc};

use crate::model::CueId;

pub const SAMPLE_BUFFER_SIZE: usize = 48_000;
pub const TELEMETRY_BUFFER_SIZE: usize = 256;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum DSPCommand {
    SetAudioDevice {
        device_name: String,
    },
    Play {
        cue_id: CueId,
        file_path: PathBuf,
        volume_db: f32,
        looping: bool,
    },
    Seek {
        cue_id: CueId,
        position_sec: f32,
    },
    Pause {
        cue_id: CueId,
    },
    Stop {
        cue_id: CueId,
    },
}

#[derive(Clone, Debug)]
pub enum AudioEvent {
    PlaybackFinished { cue_id: CueId },
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum AudioTelemetry {
    PlaybackState {
        cue_id: CueId,
        current_time_sec: f32,
        total_duration_sec: f32,
        left_peak: f32,
        right_peak: f32,
    },
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum DecoderControl {
    SeekTo(f32),
    Stop,
}

#[allow(dead_code)]
struct DecoderBuffers {
    samples: HeapCons<f32>,
    control: HeapProd<DecoderControl>,
}

struct PlaybackNode {
    cue_id: CueId,
    samples: HeapCons<f32>,
}

type SharedEvents = Arc<Mutex<HeapProd<AudioEvent>>>;
type SharedTelemetry = Arc<Mutex<HeapProd<AudioTelemetry>>>;

enum CpalCommand {
    Play {
        cue_id: CueId,
        buffers: DecoderBuffers,
    },
    SetAudioDevice(String),
}

/// Owner of every resource belonging to the audio domain.
pub struct AudioEngine {
    host: Arc<cpal::Host>,
    runtime: Option<tokio::runtime::Runtime>,
    cpal_thread: Option<JoinHandle<()>>,
    cpal_tx: Option<std::sync::mpsc::Sender<CpalCommand>>,
    command_tx: mpsc::Sender<DSPCommand>,
    audio_event_rx: Option<mpsc::Receiver<AudioEvent>>,
    telemetry_cons: Option<HeapCons<AudioTelemetry>>,
}

impl AudioEngine {
    /// Starts the private async runtime, router, event forwarder, and CPAL thread.
    pub fn new() -> Self {
        let host = Arc::new(cpal::default_host());
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build audio tokio runtime");

        let (command_tx, command_rx) = mpsc::channel(64);
        let (audio_event_tx, audio_event_rx) = mpsc::channel(64);
        let (event_prod, event_cons) = HeapRb::<AudioEvent>::new(256).split();
        let (telemetry_prod, telemetry_cons) =
            HeapRb::<AudioTelemetry>::new(TELEMETRY_BUFFER_SIZE).split();
        let notify = Arc::new(Notify::new());
        let (cpal_tx, cpal_rx) = std::sync::mpsc::channel();

        let cpal_thread = spawn_cpal_thread(
            host.clone(),
            cpal_rx,
            event_prod,
            telemetry_prod,
            notify.clone(),
        );
        runtime.spawn(command_router(command_rx, cpal_tx.clone()));
        runtime.spawn(event_forwarder(event_cons, notify, audio_event_tx));

        Self {
            host,
            runtime: Some(runtime),
            cpal_thread: Some(cpal_thread),
            cpal_tx: Some(cpal_tx),
            command_tx,
            audio_event_rx: Some(audio_event_rx),
            telemetry_cons: Some(telemetry_cons),
        }
    }

    pub fn command_sender(&self) -> mpsc::Sender<DSPCommand> {
        self.command_tx.clone()
    }

    #[allow(dead_code)]
    pub async fn send_command(
        &self,
        command: DSPCommand,
    ) -> Result<(), mpsc::error::SendError<DSPCommand>> {
        self.command_tx.send(command).await
    }

    pub fn take_audio_events(&mut self) -> mpsc::Receiver<AudioEvent> {
        self.audio_event_rx
            .take()
            .expect("audio events already taken")
    }

    pub fn take_telemetry(&mut self) -> HeapCons<AudioTelemetry> {
        self.telemetry_cons
            .take()
            .expect("audio telemetry already taken")
    }

    /// Returns the output device names discovered during subsystem startup.
    pub fn output_devices(&self) -> Vec<String> {
        self.host
            .output_devices()
            .into_iter()
            .flatten()
            .filter_map(|device| device.name().ok())
            .collect()
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        // Dropping the runtime stops the router/forwarder before joining the
        // CPAL thread, so shutdown cannot wait on a task that owns its sender.
        self.runtime.take();
        self.cpal_tx.take();
        if let Some(thread) = self.cpal_thread.take() {
            let _ = thread.join();
        }
    }
}

async fn command_router(
    mut rx: mpsc::Receiver<DSPCommand>,
    cpal_tx: std::sync::mpsc::Sender<CpalCommand>,
) {
    while let Some(command) = rx.recv().await {
        match command {
            DSPCommand::SetAudioDevice { device_name } => {
                let _ = cpal_tx.send(CpalCommand::SetAudioDevice(device_name));
            }
            DSPCommand::Play {
                cue_id, file_path, ..
            } => {
                let (sample_prod, sample_cons) = HeapRb::<f32>::new(SAMPLE_BUFFER_SIZE).split();
                let (control_prod, control_cons) = HeapRb::<DecoderControl>::new(32).split();
                let _ = cpal_tx.send(CpalCommand::Play {
                    cue_id,
                    buffers: DecoderBuffers {
                        samples: sample_cons,
                        control: control_prod,
                    },
                });
                tokio::task::spawn_blocking(move || {
                    decode_file(file_path, sample_prod, control_cons)
                });
            }
            DSPCommand::Stop { .. } => {
                // Seek/stop are consumed by the CPAL node in the full mixer.
                // Their typed boundary is established now so callers do not
                // need to know how decoder controls are transported.
            }
            _ => {}
        }
    }
}

fn decode_file(
    path: PathBuf,
    mut samples: HeapProd<f32>,
    mut controls: ringbuf::HeapCons<DecoderControl>,
) {
    let Ok(file) = std::fs::File::open(path) else {
        return;
    };
    let Ok(decoder) = rodio::Decoder::new(BufReader::new(file)) else {
        return;
    };
    for sample in decoder {
        while let Some(control) = controls.try_pop() {
            if matches!(control, DecoderControl::Stop) {
                return;
            }
        }
        if samples.try_push(sample as f32 / i16::MAX as f32).is_err() {
            std::thread::yield_now();
        }
    }
}

async fn event_forwarder(
    mut events: HeapCons<AudioEvent>,
    notify: Arc<Notify>,
    tx: mpsc::Sender<AudioEvent>,
) {
    loop {
        notify.notified().await;
        while let Some(event) = events.try_pop() {
            if tx.send(event).await.is_err() {
                return;
            }
        }
    }
}

fn spawn_cpal_thread(
    host: Arc<cpal::Host>,
    rx: std::sync::mpsc::Receiver<CpalCommand>,
    events: HeapProd<AudioEvent>,
    telemetry: HeapProd<AudioTelemetry>,
    notify: Arc<Notify>,
) -> JoinHandle<()> {
    thread::Builder::new()
        .name("articuelate-cpal".into())
        .spawn(move || {
            let active = Arc::new(Mutex::new(Vec::<PlaybackNode>::new()));
            let events = Arc::new(Mutex::new(events));
            let telemetry = Arc::new(Mutex::new(telemetry));
            let mut stream: Option<cpal::Stream> = None;

            loop {
                let command = match rx.recv_timeout(Duration::from_millis(16)) {
                    Ok(command) => command,
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                };
                match command {
                    CpalCommand::Play { cue_id, buffers } => {
                        if let Ok(mut active) = active.lock() {
                            active.push(PlaybackNode {
                                cue_id,
                                samples: buffers.samples,
                            });
                        }
                    }
                    CpalCommand::SetAudioDevice(name) => {
                        if let Ok(new_stream) = build_output_stream(
                            &host,
                            &name,
                            active.clone(),
                            events.clone(),
                            telemetry.clone(),
                            notify.clone(),
                        ) {
                            if let Err(error) = new_stream.play() {
                                eprintln!("failed to start audio output stream: {error}");
                            } else {
                                stream = Some(new_stream);
                            }
                        }
                    }
                }
            }
            drop(stream);
            if let Ok(mut events) = events.lock() {
                if let Ok(active) = active.lock() {
                    for node in active.iter() {
                        if events
                            .try_push(AudioEvent::PlaybackFinished {
                                cue_id: node.cue_id,
                            })
                            .is_ok()
                        {
                            notify.notify_one();
                        }
                    }
                }
            }
        })
        .expect("failed to spawn CPAL thread")
}

fn build_output_stream(
    host: &cpal::Host,
    device_name: &str,
    active: Arc<Mutex<Vec<PlaybackNode>>>,
    events: SharedEvents,
    telemetry: SharedTelemetry,
    notify: Arc<Notify>,
) -> Result<cpal::Stream, cpal::BuildStreamError> {
    let device = host
        .output_devices()
        .map_err(|_| cpal::BuildStreamError::DeviceNotAvailable)?
        .find(|device| {
            device
                .name()
                .map(|name| name == device_name)
                .unwrap_or(false)
        })
        .ok_or(cpal::BuildStreamError::DeviceNotAvailable)?;
    let supported = device
        .default_output_config()
        .map_err(|_| cpal::BuildStreamError::DeviceNotAvailable)?;
    let config = supported.config();
    let channels = config.channels as usize;
    let error = |error| eprintln!("audio output stream error: {error}");

    match supported.sample_format() {
        cpal::SampleFormat::F32 => device.build_output_stream(
            &config,
            move |data: &mut [f32], _| {
                fill_output(data, channels, &active, &events, &telemetry, &notify)
            },
            error,
            None,
        ),
        cpal::SampleFormat::I16 => device.build_output_stream(
            &config,
            move |data: &mut [i16], _| {
                fill_output_i16(data, channels, &active, &events, &telemetry, &notify)
            },
            error,
            None,
        ),
        cpal::SampleFormat::U16 => device.build_output_stream(
            &config,
            move |data: &mut [u16], _| {
                fill_output_u16(data, channels, &active, &events, &telemetry, &notify)
            },
            error,
            None,
        ),
        _ => Err(cpal::BuildStreamError::StreamConfigNotSupported),
    }
}

fn next_sample(active: &mut [PlaybackNode]) -> f32 {
    active
        .iter_mut()
        .find_map(|node| node.samples.try_pop())
        .unwrap_or(0.0)
}

fn fill_output(
    data: &mut [f32],
    channels: usize,
    active: &Arc<Mutex<Vec<PlaybackNode>>>,
    events: &SharedEvents,
    telemetry: &SharedTelemetry,
    notify: &Arc<Notify>,
) {
    let Ok(mut active) = active.lock() else {
        data.fill(0.0);
        return;
    };
    for frame in data.chunks_mut(channels) {
        let sample = next_sample(&mut active);
        frame.fill(sample);
    }
    publish_telemetry(&active, telemetry);
    let _ = (events, notify);
}

fn fill_output_i16(
    data: &mut [i16],
    channels: usize,
    active: &Arc<Mutex<Vec<PlaybackNode>>>,
    events: &SharedEvents,
    telemetry: &SharedTelemetry,
    notify: &Arc<Notify>,
) {
    let Ok(mut active) = active.lock() else {
        data.fill(0);
        return;
    };
    for frame in data.chunks_mut(channels) {
        frame.fill((next_sample(&mut active) * i16::MAX as f32) as i16);
    }
    publish_telemetry(&active, telemetry);
    let _ = (events, notify);
}

fn fill_output_u16(
    data: &mut [u16],
    channels: usize,
    active: &Arc<Mutex<Vec<PlaybackNode>>>,
    events: &SharedEvents,
    telemetry: &SharedTelemetry,
    notify: &Arc<Notify>,
) {
    let Ok(mut active) = active.lock() else {
        data.fill(u16::MAX / 2);
        return;
    };
    for frame in data.chunks_mut(channels) {
        frame.fill(
            (next_sample(&mut active).clamp(-1.0, 1.0) * i16::MAX as f32 + u16::MAX as f32 / 2.0)
                as u16,
        );
    }
    publish_telemetry(&active, telemetry);
    let _ = (events, notify);
}

fn publish_telemetry(active: &[PlaybackNode], telemetry: &SharedTelemetry) {
    if let Ok(mut telemetry) = telemetry.lock() {
        for node in active {
            let _ = telemetry.try_push(AudioTelemetry::PlaybackState {
                cue_id: node.cue_id,
                current_time_sec: 0.0,
                total_duration_sec: 0.0,
                left_peak: 0.0,
                right_peak: 0.0,
            });
        }
    }
}
