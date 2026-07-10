import { useEffect } from 'react';

import { Box, Paper, ScrollArea, Text } from '@mantine/core';

import { selectCue, useAppData } from '../store';
import type { Cue, CueStatus, FollowMode } from '../types';

function statusIcon(status: CueStatus): string {
  switch (status) {
    case 'Playing':
      return '▶';
    case 'Paused':
      return '⏸';
    case 'Complete':
      return '✓';
    default:
      return '○';
  }
}

function followText(mode: FollowMode): string {
  switch (mode) {
    case 'AutoContinue':
      return ' (Auto-Continue)';
    case 'AutoFollow':
      return ' (Auto-Follow)';
    default:
      return '';
  }
}

function CueRow({
  cue,
  index,
  selected,
  onSelect,
}: {
  cue: Cue;
  index: number;
  selected: boolean;
  onSelect: (i: number) => void;
}) {
  return (
    <Paper
      withBorder={false}
      radius={0}
      py={6}
      px={cue.indented ? 'xl' : 'md'}
      style={{
        cursor: 'pointer',
        borderLeft: selected ? '3px solid var(--mantine-color-blue-5)' : '3px solid transparent',
        background: selected ? 'var(--mantine-color-dark-5)' : 'transparent',
        display: 'flex',
        alignItems: 'center',
        gap: 8,
      }}
      onClick={() => onSelect(index)}
    >
      <Text span c="dimmed" w={500}>
        {statusIcon(cue.status)}
      </Text>
      <Text span fw={600}>
        {cue.number.toFixed(1)}
      </Text>
      <Text span style={{ flex: 1 }} truncate>
        {cue.name}
        <Text span c="dimmed" size="xs">
          {followText(cue.follow_mode)}
        </Text>
      </Text>
    </Paper>
  );
}

export default function CueList() {
  const { cues, selectedIndex, search } = useAppData();

  const filtered = cues
    .map((cue, index) => ({ cue, index }))
    .filter(({ cue }) => {
      const q = search.trim().toLowerCase();
      if (!q) return true;
      return (
        cue.name.toLowerCase().includes(q) ||
        (cue.audio_file_name?.toLowerCase().includes(q) ?? false)
      );
    });

  // Keyboard navigation: Up/Down to move selection, Enter to "fire".
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement;
      if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') return;
      if (filtered.length === 0) return;
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        const next = Math.min(filtered.length - 1, selectedIndex + 1);
        selectCue(filtered[next].index);
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        const prev = Math.max(0, selectedIndex - 1);
        selectCue(filtered[prev].index);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [filtered, selectedIndex]);

  return (
    <div className="gl-panel-root">
      <Box px="md" py={4} style={{ borderBottom: '1px solid var(--mantine-color-dark-4)' }}>
        <Text size="xs" tt="uppercase" fw={700} c="dimmed">
          Cuelist · {filtered.length}/{cues.length}
        </Text>
      </Box>
      <ScrollArea className="gl-panel-body">
        {filtered.map(({ cue, index }) => (
          <CueRow
            key={index}
            cue={cue}
            index={index}
            selected={index === selectedIndex}
            onSelect={selectCue}
          />
        ))}
      </ScrollArea>
    </div>
  );
}
