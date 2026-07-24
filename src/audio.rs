//! Lock-free audio subsystem boundary.

use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::{HeapCons, HeapProd, HeapRb};
use tokio::sync::{mpsc, oneshot};

use crate::model::CueId;

pub const SAMPLE_BUFFER_SIZE: usize = 48_000;
pub const TELEMETRY_BUFFER_SIZE: usize = 256;
const COMMAND_BUFFER_SIZE: usize = 128;

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

/// The public, execution-engine-facing event shape is retained for callers.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum AudioEvent {
    PlaybackFinished {
        cue_id: CueId,
    },
    DeviceLost {
        device_name: Option<String>,
        error_message: String,
    },
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
    control: HeapProd<DecoderControl>,
}

/// Events emitted directly by the CPAL callback through the event ringbuffer.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum AudioEngineEvent {
    DeviceLost {
        device_name: Option<String>,
        error_message: String,
    },
    PlaybackFinished(CueId),
}

enum CpalCommand {
    AddNode(PlaybackNode),
    RemoveNode(CueId),
    Seek {
        cue_id: CueId,
        position_sec: f32,
    },
    Shutdown {
        return_tx: oneshot::Sender<CpalThreadContext>,
    },
}

/// State handed between CPAL streams. It is never shared: one CPAL callback
/// owns it at a time, and shutdown transfers it through the oneshot.
struct CpalThreadContext {
    nodes: Vec<PlaybackNode>,
    command_rx: HeapCons<CpalCommand>,
    telemetry_tx: HeapProd<AudioTelemetry>,
    #[allow(dead_code)]
    event_tx: HeapProd<AudioEngineEvent>,
}

pub struct AudioEngine {
    host: Arc<cpal::Host>,
    runtime: Option<tokio::runtime::Runtime>,

    command_tx: mpsc::Sender<DSPCommand>,
    shutdown_tx: Option<oneshot::Sender<oneshot::Sender<()>>>,
    audio_event_rx: Option<mpsc::Receiver<AudioEvent>>,
    telemetry_cons: Option<HeapCons<AudioTelemetry>>,
}

impl AudioEngine {
    pub fn new() -> Self {
        let host = Arc::new(cpal::default_host());
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build audio tokio runtime");

        let (command_tx, command_rx) = mpsc::channel(64);
        let (audio_event_tx, audio_event_rx) = mpsc::channel(64);
        let (event_prod, event_cons) = HeapRb::<AudioEngineEvent>::new(256).split();
        let (telemetry_prod, telemetry_cons) =
            HeapRb::<AudioTelemetry>::new(TELEMETRY_BUFFER_SIZE).split();
        let (cpal_prod, cpal_rx) = HeapRb::<CpalCommand>::new(COMMAND_BUFFER_SIZE).split();
        let context = CpalThreadContext {
            nodes: Vec::new(),
            command_rx: cpal_rx,
            telemetry_tx: telemetry_prod,
            event_tx: event_prod,
        };
        let device_name = host
            .output_devices()
            .ok()
            .and_then(|mut devices| devices.next().and_then(|device| device.name().ok()));
        let cpal_thread = spawn_cpal_thread(host.clone(), context, device_name);

        let (shutdown_tx, shutdown_rx) = oneshot::channel::<oneshot::Sender<()>>();
        runtime.spawn(command_router(
            host.clone(),
            command_rx,
            cpal_prod,
            cpal_thread,
            shutdown_rx,
        ));
        runtime.spawn(event_forwarder(event_cons, audio_event_tx));

        Self {
            host,
            runtime: Some(runtime),

            command_tx,
            shutdown_tx: Some(shutdown_tx),
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
        if let (Some(runtime), Some(shutdown_tx)) = (self.runtime.as_ref(), self.shutdown_tx.take())
        {
            let _ = runtime.block_on(async {
                let (done_tx, done_rx) = oneshot::channel();
                if shutdown_tx.send(done_tx).is_ok() {
                    let _ = done_rx.await;
                }
            });
        }
        self.runtime.take();
    }
}

async fn command_router(
    host: Arc<cpal::Host>,
    mut rx: mpsc::Receiver<DSPCommand>,
    mut cpal_tx: HeapProd<CpalCommand>,
    mut cpal_thread: JoinHandle<()>,
    mut shutdown_rx: oneshot::Receiver<oneshot::Sender<()>>,
) {
    loop {
        let command = tokio::select! {
            command = rx.recv() => command,
            done_tx = &mut shutdown_rx => {
                if let Ok(done_tx) = done_tx {
                    shutdown_cpal(&mut cpal_tx, &mut cpal_thread).await;
                    let _ = done_tx.send(());
                }
                break;
            }
        };
        let Some(command) = command else {
            shutdown_cpal(&mut cpal_tx, &mut cpal_thread).await;
            break;
        };
        match command {
            DSPCommand::SetAudioDevice { device_name } => {
                let (return_tx, return_rx) = oneshot::channel();
                if cpal_tx
                    .try_push(CpalCommand::Shutdown { return_tx })
                    .is_err()
                {
                    continue;
                }
                let Ok(context) = return_rx.await else {
                    continue;
                };
                let old_thread = std::mem::replace(&mut cpal_thread, thread::spawn(|| {}));
                let _ = old_thread.join();
                cpal_thread = spawn_cpal_thread(host.clone(), context, Some(device_name));
            }
            DSPCommand::Play {
                cue_id, file_path, ..
            } => {
                let (sample_prod, sample_cons) = HeapRb::<f32>::new(SAMPLE_BUFFER_SIZE).split();
                let (control_prod, control_cons) = HeapRb::<DecoderControl>::new(32).split();
                if cpal_tx
                    .try_push(CpalCommand::AddNode(PlaybackNode {
                        cue_id,
                        samples: sample_cons,
                        control: control_prod,
                    }))
                    .is_ok()
                {
                    tokio::task::spawn_blocking(move || {
                        decode_file(file_path, sample_prod, control_cons)
                    });
                }
            }
            DSPCommand::Seek {
                cue_id,
                position_sec,
            } => {
                let _ = cpal_tx.try_push(CpalCommand::Seek {
                    cue_id,
                    position_sec,
                });
            }
            DSPCommand::Stop { cue_id } | DSPCommand::Pause { cue_id } => {
                let _ = cpal_tx.try_push(CpalCommand::RemoveNode(cue_id));
            }
        }
    }
}

async fn shutdown_cpal(cpal_tx: &mut HeapProd<CpalCommand>, cpal_thread: &mut JoinHandle<()>) {
    let (return_tx, return_rx) = oneshot::channel();
    if cpal_tx
        .try_push(CpalCommand::Shutdown { return_tx })
        .is_ok()
    {
        let _ = return_rx.await;
    }
    let old_thread = std::mem::replace(cpal_thread, thread::spawn(|| {}));
    let _ = old_thread.join();
}

fn decode_file(path: PathBuf, mut samples: HeapProd<f32>, mut controls: HeapCons<DecoderControl>) {
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
        while samples.try_push(sample as f32 / i16::MAX as f32).is_err() {
            thread::yield_now();
        }
    }
}

async fn event_forwarder(mut events: HeapCons<AudioEngineEvent>, tx: mpsc::Sender<AudioEvent>) {
    loop {
        tokio::time::sleep(std::time::Duration::from_millis(16)).await;
        while let Some(event) = events.try_pop() {
            let event = match event {
                AudioEngineEvent::PlaybackFinished(cue_id) => {
                    AudioEvent::PlaybackFinished { cue_id }
                }
                AudioEngineEvent::DeviceLost {
                    device_name,
                    error_message,
                } => AudioEvent::DeviceLost {
                    device_name,
                    error_message,
                },
            };
            if tx.send(event).await.is_err() {
                return;
            }
        }
    }
}

fn spawn_cpal_thread(
    host: Arc<cpal::Host>,
    context: CpalThreadContext,
    device_name: Option<String>,
) -> JoinHandle<()> {
    thread::Builder::new()
        .name("articuelate-cpal".into())
        .spawn(move || {
            let (wake_tx, wake_rx) = std::sync::mpsc::channel::<()>();
            let mut stream =
                build_output_stream(&host, device_name.as_deref(), context, wake_tx.clone());
            if let Some(ref stream) = stream {
                let _ = stream.play();
            }
            // The callback owns the context. Shutdown wakes this thread after the
            // context has been transferred to the caller.
            let _ = wake_rx.recv();
            drop(stream.take());
        })
        .expect("failed to spawn CPAL thread")
}

fn build_output_stream(
    host: &cpal::Host,
    device_name: Option<&str>,
    context: CpalThreadContext,
    wake_tx: std::sync::mpsc::Sender<()>,
) -> Option<cpal::Stream> {
    let device = host.output_devices().ok()?.find(|device| {
        device_name.map_or(true, |name| {
            device.name().map(|n| n == name).unwrap_or(false)
        })
    })?;
    let supported = device.default_output_config().ok()?;
    let config = supported.config();
    let channels = config.channels as usize;
    let mut context = Some(context);
    match supported.sample_format() {
        cpal::SampleFormat::F32 => device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _| fill_output(data, channels, &mut context, &wake_tx),
                move |error| eprintln!("audio output stream error: {error}"),
                None,
            )
            .ok(),
        cpal::SampleFormat::I16 => device
            .build_output_stream(
                &config,
                move |data: &mut [i16], _| fill_output_i16(data, channels, &mut context, &wake_tx),
                move |error| eprintln!("audio output stream error: {error}"),
                None,
            )
            .ok(),
        cpal::SampleFormat::U16 => device
            .build_output_stream(
                &config,
                move |data: &mut [u16], _| fill_output_u16(data, channels, &mut context, &wake_tx),
                move |error| eprintln!("audio output stream error: {error}"),
                None,
            )
            .ok(),
        _ => None,
    }
}

fn process_commands(
    context: &mut Option<CpalThreadContext>,
    wake_tx: &std::sync::mpsc::Sender<()>,
) {
    loop {
        let Some(command) = context
            .as_mut()
            .and_then(|context| context.command_rx.try_pop())
        else {
            break;
        };
        match command {
            CpalCommand::AddNode(node) => {
                if let Some(context) = context.as_mut() {
                    if let Some(index) = context
                        .nodes
                        .iter()
                        .position(|old| old.cue_id == node.cue_id)
                    {
                        let mut old = context.nodes.swap_remove(index);
                        let _ = old.control.try_push(DecoderControl::Stop);
                    }
                    context.nodes.push(node);
                }
            }
            CpalCommand::RemoveNode(cue_id) => {
                if let Some(context) = context.as_mut() {
                    if let Some(index) = context.nodes.iter().position(|node| node.cue_id == cue_id)
                    {
                        let mut node = context.nodes.swap_remove(index);
                        let _ = node.control.try_push(DecoderControl::Stop);
                    }
                }
            }
            CpalCommand::Seek {
                cue_id,
                position_sec,
            } => {
                if let Some(context) = context.as_mut() {
                    if let Some(node) = context.nodes.iter_mut().find(|node| node.cue_id == cue_id)
                    {
                        let _ = node.control.try_push(DecoderControl::SeekTo(position_sec));
                    }
                }
            }
            CpalCommand::Shutdown { return_tx } => {
                if let Some(returned) = context.take() {
                    let _ = return_tx.send(returned);
                }
                let _ = wake_tx.send(());
                break;
            }
        }
    }
}

fn next_sample(nodes: &mut [PlaybackNode]) -> f32 {
    nodes
        .iter_mut()
        .find_map(|node| node.samples.try_pop())
        .unwrap_or(0.0)
}

fn fill_output(
    data: &mut [f32],
    channels: usize,
    context: &mut Option<CpalThreadContext>,
    wake_tx: &std::sync::mpsc::Sender<()>,
) {
    process_commands(context, wake_tx);
    if let Some(context) = context.as_mut() {
        for frame in data.chunks_mut(channels) {
            frame.fill(next_sample(&mut context.nodes));
        }
        publish_telemetry(context);
    } else {
        data.fill(0.0);
    }
}

fn fill_output_i16(
    data: &mut [i16],
    channels: usize,
    context: &mut Option<CpalThreadContext>,
    wake_tx: &std::sync::mpsc::Sender<()>,
) {
    process_commands(context, wake_tx);
    if let Some(context) = context.as_mut() {
        for frame in data.chunks_mut(channels) {
            frame.fill((next_sample(&mut context.nodes) * i16::MAX as f32) as i16);
        }
        publish_telemetry(context);
    } else {
        data.fill(0);
    }
}

fn fill_output_u16(
    data: &mut [u16],
    channels: usize,
    context: &mut Option<CpalThreadContext>,
    wake_tx: &std::sync::mpsc::Sender<()>,
) {
    process_commands(context, wake_tx);
    if let Some(context) = context.as_mut() {
        for frame in data.chunks_mut(channels) {
            frame.fill(
                (next_sample(&mut context.nodes).clamp(-1.0, 1.0) * i16::MAX as f32
                    + u16::MAX as f32 / 2.0) as u16,
            );
        }
        publish_telemetry(context);
    } else {
        data.fill(u16::MAX / 2);
    }
}

fn publish_telemetry(context: &mut CpalThreadContext) {
    for node in &context.nodes {
        let _ = context
            .telemetry_tx
            .try_push(AudioTelemetry::PlaybackState {
                cue_id: node.cue_id,
                current_time_sec: 0.0,
                total_duration_sec: 0.0,
                left_peak: 0.0,
                right_peak: 0.0,
            });
    }
}
