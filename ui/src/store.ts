import { useSyncExternalStore } from 'react';

import { getActivePlaybacks, getCues, getStatus } from './ipc';
import type { ActivePlayback, Cue, StatusInfo } from './types';

export interface AppData {
  cues: Cue[];
  selectedIndex: number;
  search: string;
  status: StatusInfo | null;
  playbacks: ActivePlayback[];
}

let state: AppData = {
  cues: [],
  selectedIndex: 0,
  search: '',
  status: null,
  playbacks: [],
};

const listeners = new Set<() => void>();

function emit() {
  listeners.forEach((l) => l());
}

function subscribe(listener: () => void) {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

function getSnapshot() {
  return state;
}

export function setState(patch: Partial<AppData>) {
  state = { ...state, ...patch };
  emit();
}

export function selectCue(index: number) {
  setState({ selectedIndex: index });
}

export function setSearch(value: string) {
  setState({ search: value });
}

export function useAppData(): AppData {
  return useSyncExternalStore(subscribe, getSnapshot);
}

/// Pull a fresh snapshot of cues/status/playbacks from the backend.
export async function refreshFromBackend() {
  const [cues, status, playbacks] = await Promise.all([
    getCues(),
    getStatus(),
    getActivePlaybacks(),
  ]);
  setState({ cues, status, playbacks });
}
