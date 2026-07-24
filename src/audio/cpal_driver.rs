//! Custom CPAL driver with lock-free decoder-to-callback playback.
//! EXPERIMENTAL/WIP: currently not functional, but in the future will support the missing audio playback features from rodio_driver

use async_trait::async_trait;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::traits::{Consumer, Observer, Producer, Split};
use ringbuf::{HeapCons, HeapProd, HeapRb};
use rodio::Source;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, mpsc};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tokio::sync::Notify;

use crate::audio::{AudioDriver, AudioDriverError, AudioEvent, AudioTelemetry, DriverCapabilities};
use crate::model::CueId;

const SAMPLE_BUFFER_SIZE: usize = 48_000;
const CONTROL_BUFFER_SIZE: usize = 32;

enum DecoderControl {
    SeekTo(f32),
    Stop,
}

struct PlaybackNode {
    cue_id: CueId,
    samples: HeapCons<f32>,
    controls: HeapProd<DecoderControl>,
    volume: f32,
    finished: Arc<AtomicBool>,
    frames: AtomicU64,
}

enum CpalCommand {
    SetDevice(String),
    Play(PlaybackNode),
    Seek { cue_id: CueId, position_sec: f32 },
    Stop(CueId),
}

enum CallbackCommand {
    Play(PlaybackNode),
    Seek { cue_id: CueId, position_sec: f32 },
    Stop(CueId),
}

pub struct CpalDriver {
    command_tx: Option<mpsc::Sender<CpalCommand>>,
    cpal_thread: Option<JoinHandle<()>>,
    device_name: Option<String>,
}

impl CpalDriver {
    pub fn new(
        telemetry: Arc<AudioTelemetry>,
        audio_event_tx: tokio::sync::mpsc::Sender<AudioEvent>,
    ) -> Self {
        let (event_tx, event_rx) = HeapRb::<AudioEvent>::new(256).split();
        let notify = Arc::new(Notify::new());
        tokio::spawn(event_forwarder(event_rx, notify.clone(), audio_event_tx));

        let (command_tx, command_rx) = mpsc::channel();
        let cpal_thread = thread::Builder::new()
            .name("articuelate-cpal".into())
            .spawn(move || cpal_thread(command_rx, event_tx, telemetry, notify))
            .expect("failed to spawn CPAL thread");

        Self {
            command_tx: Some(command_tx),
            cpal_thread: Some(cpal_thread),
            device_name: None,
        }
    }
}

#[async_trait(?Send)]
impl AudioDriver for CpalDriver {
    fn capabilities(&self) -> DriverCapabilities {
        DriverCapabilities {
            supports_matrix_routing: true,
            // The callback owns the ringbuffer producers and stream state; changing
            // devices requires constructing a fresh driver to preserve that ownership.
            supports_hot_swapping: false,
        }
    }

    fn output_devices(&self) -> Vec<String> {
        cpal::default_host()
            .output_devices()
            .into_iter()
            .flatten()
            .filter_map(|device| device.name().ok())
            .collect()
    }

    async fn set_device(&mut self, device_name: String) -> Result<(), AudioDriverError> {
        let command_tx = self
            .command_tx
            .as_ref()
            .ok_or_else(|| AudioDriverError::DeviceError("CPAL driver is shut down".into()))?;
        command_tx
            .send(CpalCommand::SetDevice(device_name.clone()))
            .map_err(|error| AudioDriverError::DeviceError(error.to_string()))?;
        self.device_name = Some(device_name);
        Ok(())
    }

    async fn play_cue(
        &mut self,
        cue_id: CueId,
        file_path: PathBuf,
        volume_db: f32,
        looping: bool,
        _matrix: Option<Vec<Vec<f32>>>,
    ) -> Result<(), AudioDriverError> {
        if self.device_name.is_none() {
            return Err(AudioDriverError::DeviceError(
                "No CPAL output device selected".into(),
            ));
        }
        let command_tx = self
            .command_tx
            .as_ref()
            .ok_or_else(|| AudioDriverError::PlaybackError("CPAL driver is shut down".into()))?
            .clone();
        let (sample_tx, sample_rx) = HeapRb::<f32>::new(SAMPLE_BUFFER_SIZE).split();
        let (control_tx, control_rx) = HeapRb::<DecoderControl>::new(CONTROL_BUFFER_SIZE).split();
        let finished = Arc::new(AtomicBool::new(false));
        let decoder_finished = finished.clone();
        tokio::task::spawn_blocking(move || {
            decode_file(file_path, sample_tx, control_rx, looping, decoder_finished)
        });

        command_tx
            .send(CpalCommand::Play(PlaybackNode {
                cue_id,
                samples: sample_rx,
                controls: control_tx,
                volume: 10.0_f32.powf(volume_db / 20.0),
                finished,
                frames: AtomicU64::new(0),
            }))
            .map_err(|error| AudioDriverError::PlaybackError(error.to_string()))
    }

    async fn seek_cue(&mut self, cue_id: CueId, position_sec: f32) -> Result<(), AudioDriverError> {
        self.send(CpalCommand::Seek {
            cue_id,
            position_sec,
        })
    }

    async fn stop_cue(&mut self, cue_id: CueId) -> Result<(), AudioDriverError> {
        self.send(CpalCommand::Stop(cue_id))
    }
}

impl CpalDriver {
    fn send(&self, command: CpalCommand) -> Result<(), AudioDriverError> {
        self.command_tx
            .as_ref()
            .ok_or_else(|| AudioDriverError::PlaybackError("CPAL driver is shut down".into()))?
            .send(command)
            .map_err(|error| AudioDriverError::PlaybackError(error.to_string()))
    }
}

impl Drop for CpalDriver {
    fn drop(&mut self) {
        self.command_tx.take();
        if let Some(thread) = self.cpal_thread.take() {
            let _ = thread.join();
        }
    }
}

fn cpal_thread(
    command_rx: mpsc::Receiver<CpalCommand>,
    event_tx: HeapProd<AudioEvent>,
    telemetry: Arc<AudioTelemetry>,
    notify: Arc<Notify>,
) {
    let host = cpal::default_host();
    let mut callback_tx: Option<mpsc::Sender<CallbackCommand>> = None;
    let mut event_tx = Some(event_tx);
    let mut stream: Option<cpal::Stream> = None;

    while let Ok(command) = command_rx.recv() {
        match command {
            CpalCommand::SetDevice(name) if callback_tx.is_none() => match build_stream(
                &host,
                &name,
                &mut callback_tx,
                event_tx
                    .take()
                    .expect("CPAL event producer already consumed"),
                telemetry.clone(),
                notify.clone(),
            ) {
                Ok(new_stream) => {
                    if let Err(error) = new_stream.play() {
                        eprintln!("failed to start audio output stream: {error}");
                    } else {
                        stream = Some(new_stream);
                    }
                }
                Err(error) => eprintln!("failed to build audio output stream: {error}"),
            },
            CpalCommand::SetDevice(_) => {
                eprintln!("CPAL device switching is only available before stream startup");
            }
            CpalCommand::Play(node) => {
                if let Some(tx) = &callback_tx {
                    let _ = tx.send(CallbackCommand::Play(node));
                }
            }
            CpalCommand::Seek {
                cue_id,
                position_sec,
            } => {
                if let Some(tx) = &callback_tx {
                    let _ = tx.send(CallbackCommand::Seek {
                        cue_id,
                        position_sec,
                    });
                }
            }
            CpalCommand::Stop(cue_id) => {
                if let Some(tx) = &callback_tx {
                    let _ = tx.send(CallbackCommand::Stop(cue_id));
                }
            }
        }
    }
    drop(callback_tx);
    drop(stream);
}

fn build_stream(
    host: &cpal::Host,
    device_name: &str,
    callback_tx: &mut Option<mpsc::Sender<CallbackCommand>>,
    mut events: HeapProd<AudioEvent>,
    telemetry: Arc<AudioTelemetry>,
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
    let (tx, rx) = mpsc::channel();
    *callback_tx = Some(tx);
    let error = |error| eprintln!("audio output stream error: {error}");

    let mut active = Vec::new();
    match supported.sample_format() {
        cpal::SampleFormat::F32 => device.build_output_stream(
            &config,
            move |data: &mut [f32], _| {
                fill_output(
                    data,
                    channels,
                    &rx,
                    &mut active,
                    &mut events,
                    &telemetry,
                    &notify,
                )
            },
            error,
            None,
        ),
        cpal::SampleFormat::I16 => device.build_output_stream(
            &config,
            move |data: &mut [i16], _| {
                fill_output_i16(
                    data,
                    channels,
                    &rx,
                    &mut active,
                    &mut events,
                    &telemetry,
                    &notify,
                )
            },
            error,
            None,
        ),
        cpal::SampleFormat::U16 => device.build_output_stream(
            &config,
            move |data: &mut [u16], _| {
                fill_output_u16(
                    data,
                    channels,
                    &rx,
                    &mut active,
                    &mut events,
                    &telemetry,
                    &notify,
                )
            },
            error,
            None,
        ),
        _ => Err(cpal::BuildStreamError::StreamConfigNotSupported),
    }
}

fn apply_commands(rx: &mpsc::Receiver<CallbackCommand>, active: &mut Vec<PlaybackNode>) {
    while let Ok(command) = rx.try_recv() {
        match command {
            CallbackCommand::Play(node) => active.push(node),
            CallbackCommand::Seek {
                cue_id,
                position_sec,
            } => {
                if let Some(node) = active.iter_mut().find(|node| node.cue_id == cue_id) {
                    while node.samples.try_pop().is_some() {}
                    let _ = node
                        .controls
                        .try_push(DecoderControl::SeekTo(position_sec.max(0.0)));
                }
            }
            CallbackCommand::Stop(cue_id) => active.retain_mut(|node| {
                if node.cue_id == cue_id {
                    let _ = node.controls.try_push(DecoderControl::Stop);
                    false
                } else {
                    true
                }
            }),
        }
    }
}

fn next_sample(active: &mut [PlaybackNode]) -> f32 {
    let mut mixed = 0.0;
    for node in active.iter_mut() {
        if let Some(sample) = node.samples.try_pop() {
            mixed += sample * node.volume;
            node.frames.fetch_add(1, Ordering::Relaxed);
        }
    }
    mixed.clamp(-1.0, 1.0)
}

fn finish_nodes(
    active: &mut Vec<PlaybackNode>,
    events: &mut HeapProd<AudioEvent>,
    notify: &Arc<Notify>,
) {
    active.retain(|node| {
        if node.finished.load(Ordering::Acquire) && node.samples.is_empty() {
            if events
                .try_push(AudioEvent::PlaybackFinished {
                    cue_id: node.cue_id,
                })
                .is_ok()
            {
                notify.notify_one();
            }
            false
        } else {
            true
        }
    });
}

fn fill_output(
    data: &mut [f32],
    channels: usize,
    rx: &mpsc::Receiver<CallbackCommand>,
    active: &mut Vec<PlaybackNode>,
    events: &mut HeapProd<AudioEvent>,
    telemetry: &Arc<AudioTelemetry>,
    notify: &Arc<Notify>,
) {
    fill_output_inner(
        data,
        channels,
        rx,
        active,
        events,
        telemetry,
        notify,
        |sample| sample,
    );
}
fn fill_output_i16(
    data: &mut [i16],
    channels: usize,
    rx: &mpsc::Receiver<CallbackCommand>,
    active: &mut Vec<PlaybackNode>,
    events: &mut HeapProd<AudioEvent>,
    telemetry: &Arc<AudioTelemetry>,
    notify: &Arc<Notify>,
) {
    fill_output_inner(
        data,
        channels,
        rx,
        active,
        events,
        telemetry,
        notify,
        |sample| (sample * i16::MAX as f32) as i16,
    );
}
fn fill_output_u16(
    data: &mut [u16],
    channels: usize,
    rx: &mpsc::Receiver<CallbackCommand>,
    active: &mut Vec<PlaybackNode>,
    events: &mut HeapProd<AudioEvent>,
    telemetry: &Arc<AudioTelemetry>,
    notify: &Arc<Notify>,
) {
    fill_output_inner(
        data,
        channels,
        rx,
        active,
        events,
        telemetry,
        notify,
        |sample| ((sample * i16::MAX as f32) + u16::MAX as f32 / 2.0) as u16,
    );
}

fn fill_output_inner<T: Copy, F: Fn(f32) -> T>(
    data: &mut [T],
    channels: usize,
    rx: &mpsc::Receiver<CallbackCommand>,
    active: &mut Vec<PlaybackNode>,
    events: &mut HeapProd<AudioEvent>,
    telemetry: &Arc<AudioTelemetry>,
    notify: &Arc<Notify>,
    convert: F,
) {
    apply_commands(rx, active);
    for frame in data.chunks_mut(channels) {
        let value = convert(next_sample(active));
        for sample in frame {
            *sample = value;
        }
    }
    for node in active.iter() {
        let frames = node.frames.load(Ordering::Relaxed);
        telemetry
            .metrics_for(node.cue_id)
            .update(frames as f64 / 48_000.0, 0.0, 0.0, 0.0);
    }
    finish_nodes(active, events, notify);
}

fn decode_file(
    path: PathBuf,
    mut samples: HeapProd<f32>,
    mut controls: HeapCons<DecoderControl>,
    looping: bool,
    finished: Arc<AtomicBool>,
) {
    let mut seek = None;
    loop {
        let Ok(file) = std::fs::File::open(&path) else {
            finished.store(true, Ordering::Release);
            return;
        };
        let Ok(decoder) = rodio::Decoder::new(BufReader::new(file)) else {
            finished.store(true, Ordering::Release);
            return;
        };
        let mut decoder: Box<dyn Iterator<Item = f32> + Send> = Box::new(
            decoder
                .skip_duration(seek.unwrap_or(Duration::ZERO))
                .map(|sample| sample as f32),
        );
        seek = None;
        loop {
            while let Some(control) = controls.try_pop() {
                match control {
                    DecoderControl::Stop => {
                        finished.store(true, Ordering::Release);
                        return;
                    }
                    DecoderControl::SeekTo(position) => {
                        seek = Some(Duration::from_secs_f32(position));
                    }
                }
            }
            if seek.is_some() {
                break;
            }
            match decoder.next() {
                Some(sample) => {
                    while samples.try_push(sample).is_err() {
                        thread::yield_now();
                        if matches!(controls.try_pop(), Some(DecoderControl::Stop)) {
                            finished.store(true, Ordering::Release);
                            return;
                        }
                    }
                }
                None if looping => break,
                None => {
                    finished.store(true, Ordering::Release);
                    return;
                }
            }
        }
    }
}

async fn event_forwarder(
    mut events: HeapCons<AudioEvent>,
    notify: Arc<Notify>,
    tx: tokio::sync::mpsc::Sender<AudioEvent>,
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
