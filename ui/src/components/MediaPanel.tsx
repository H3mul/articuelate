import { useEffect, useState } from 'react';

import { Box, Paper, Progress, Slider, Stack, Text } from '@mantine/core';

import { useAppData } from '../store';
import type { ActivePlayback } from '../types';

export default function MediaPanel() {
  const { playbacks } = useAppData();
  // Local copy so we can animate the meters smoothly (the backend only
  // updates at IPC cadence). Resync whenever the backend snapshot changes.
  const [items, setItems] = useState<ActivePlayback[]>(playbacks);

  useEffect(() => {
    setItems(playbacks);
  }, [playbacks]);

  useEffect(() => {
    const id = setInterval(() => {
      setItems((prev) =>
        prev.map((p) => ({ ...p, progress: (p.progress + 0.01) % 1 })),
      );
    }, 50);
    return () => clearInterval(id);
  }, []);

  return (
    <div className="gl-panel-root">
      <Box px="md" py={6} style={{ borderBottom: '1px solid var(--mantine-color-dark-4)' }}>
        <Text size="xs" tt="uppercase" fw={700} c="dimmed">
          Active Media · {items.length} layers
        </Text>
      </Box>

      <Stack className="gl-panel-body" p="md" gap="md">
        {items.length === 0 && (
          <Text c="dimmed" size="sm">
            No active playback. Press GO to fire a cue.
          </Text>
        )}

        {items.map((pb, i) => (
          <Paper key={i} withBorder p="sm" radius="sm">
            <Stack gap={4}>
              <Text size="sm" fw={600}>
                Cue {pb.cue_number.toFixed(1)} · {pb.label}
              </Text>
              <Progress value={pb.progress * 100} size="md" radius="xs" />
              <Text size="xs" c="dimmed">
                {pb.volume_db.toFixed(0)} dB
              </Text>
              <Slider
                value={-pb.volume_db}
                min={0}
                max={60}
                step={1}
                onChange={() => undefined}
                label={(v) => `-${v} dB`}
              />
            </Stack>
          </Paper>
        ))}
      </Stack>
    </div>
  );
}
