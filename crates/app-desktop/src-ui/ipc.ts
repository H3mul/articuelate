import type { ActivePlayback, Cue, StatusInfo } from './types';

// Detect whether we are running inside the Tauri webview. When false we fall
// back to static mock data so the layout can be iterated on with a plain
// `vite` dev server, without compiling the Rust backend.
const isTauri =
  typeof window !== 'undefined' &&
  ('__TAURI_INTERNALS__' in window || '__TAURI__' in window);

const MOCK_CUES: Cue[] = [
  {
    number: 1.0,
    name: 'Storm Intro',
    tasks: [
      {
        target_name: 'BGM',
        property: 'Volume',
        target_value: -24.0,
        duration_secs: 3.0,
        curve: 'Linear',
        output: { name: 'Main L/R' },
      },
      {
        target_name: 'Player',
        property: 'Play',
        target_value: 0.0,
        duration_secs: 0.0,
        curve: 'Linear',
        output: { name: 'Main L/R' },
      },
    ],
    status: 'Ready',
    follow_mode: 'Manual',
    pre_wait_secs: 0.0,
    post_wait_secs: 0.0,
    notes: '',
    indented: false,
    audio_file_name: 'Wind_Loop.wav',
  },
  {
    number: 2.0,
    name: 'Thunder Strike',
    tasks: [],
    status: 'Ready',
    follow_mode: 'Manual',
    pre_wait_secs: 0.0,
    post_wait_secs: 0.0,
    notes: '',
    indented: true,
    audio_file_name: 'Thunder.wav',
  },
  {
    number: 3.0,
    name: 'Storm Outro',
    tasks: [],
    status: 'Ready',
    follow_mode: 'AutoFollow',
    pre_wait_secs: 0.0,
    post_wait_secs: 0.0,
    notes: '',
    indented: true,
    audio_file_name: null,
  },
];

const MOCK_STATUS: StatusInfo = {
  connected: true,
  device_name: 'USB Audio Device',
  cpu_usage: 4,
  dsp_usage: 12,
};

const MOCK_PLAYBACKS: ActivePlayback[] = [
  { cue_number: 1, label: 'Wind_Loop.wav', volume_db: -12, progress: 0.65 },
  { cue_number: 1, label: 'Rain_Heavy.wav', volume_db: -8, progress: 0.42 },
];

// Lazily import the Tauri core API only when running inside Tauri so the
// module can be bundled for the plain-Vite fallback too.
async function invoke<T>(cmd: string): Promise<T> {
  const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
  return tauriInvoke<T>(cmd);
}

export async function getCues(): Promise<Cue[]> {
  if (!isTauri) return MOCK_CUES;
  return invoke<Cue[]>('get_cues');
}

export async function getStatus(): Promise<StatusInfo> {
  if (!isTauri) return MOCK_STATUS;
  return invoke<StatusInfo>('get_status');
}

export async function getActivePlaybacks(): Promise<ActivePlayback[]> {
  if (!isTauri) return MOCK_PLAYBACKS;
  return invoke<ActivePlayback[]>('get_active_playbacks');
}

export async function go(): Promise<void> {
  if (isTauri) await invoke<void>('go');
}

export async function back(): Promise<void> {
  if (isTauri) await invoke<void>('back');
}

export async function pause(): Promise<void> {
  if (isTauri) await invoke<void>('pause');
}

export async function panicStop(): Promise<void> {
  if (isTauri) await invoke<void>('panic_stop');
}
