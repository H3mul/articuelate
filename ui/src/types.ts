export type FollowMode = 'Manual' | 'AutoContinue' | 'AutoFollow';
export type CueStatus = 'Ready' | 'Playing' | 'Paused' | 'Complete';
export type FadeCurve = 'Linear' | 'Logarithmic' | 'Exponential';

export interface OutputTarget {
  name: string;
}

export interface Task {
  target_name: string;
  property: string;
  target_value: number;
  duration_secs: number;
  curve: FadeCurve;
  output: OutputTarget;
}

export interface Cue {
  number: number;
  name: string;
  tasks: Task[];
  status: CueStatus;
  follow_mode: FollowMode;
  pre_wait_secs: number;
  post_wait_secs: number;
  notes: string;
  indented: boolean;
  audio_file_name: string | null;
}

export interface ActivePlayback {
  cue_number: number;
  label: string;
  volume_db: number;
  progress: number;
}

export interface StatusInfo {
  connected: boolean;
  device_name: string;
  cpu_usage: number;
  dsp_usage: number;
}
