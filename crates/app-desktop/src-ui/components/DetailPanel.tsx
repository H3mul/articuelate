import type { ReactNode } from 'react';

import {
  Box,
  Divider,
  NumberInput,
  Paper,
  Select,
  Slider,
  Stack,
  Text,
  Textarea,
  TextInput,
  Title,
} from '@mantine/core';

import { useAppData } from '../store';

export default function DetailPanel() {
  const { cues, selectedIndex } = useAppData();
  const cue = cues[selectedIndex];

  if (!cue) {
    return (
      <div className="gl-panel-root">
        <Box p="md">
          <Text c="dimmed" size="sm">
            Nothing selected — adjust the global show defaults.
          </Text>
        </Box>
      </div>
    );
  }

  return (
    <div className="gl-panel-root">
      <Box px="md" py={6} style={{ borderBottom: '1px solid var(--mantine-color-dark-4)' }}>
        <Text size="xs" tt="uppercase" fw={700} c="dimmed">
          Detail · Cue {cue.number.toFixed(1)}
        </Text>
      </Box>

      <Stack className="gl-panel-body" p="md" gap="sm">
        <Title order={6}>{cue.name}</Title>

        <Select
          label="Follow mode"
          data={['Manual', 'AutoContinue', 'AutoFollow']}
          value={cue.follow_mode}
          onChange={() => undefined}
          allowDeselect={false}
        />

        <GroupRow
          left={<NumberInput label="Pre-wait (s)" value={cue.pre_wait_secs} onChange={() => undefined} />}
          right={<NumberInput label="Post-wait (s)" value={cue.post_wait_secs} onChange={() => undefined} />}
        />

        <Textarea label="Designer notes" value={cue.notes} autosize minRows={2} onChange={() => undefined} />

        <Divider label="Tasks" labelPosition="left" />

        {cue.tasks.length === 0 && (
          <Text c="dimmed" size="sm">
            No explicit tasks on this cue.
          </Text>
        )}

        {cue.tasks.map((task, i) => (
          <Paper key={i} withBorder p="sm" radius="sm">
            <Stack gap="xs">
              <TextInput label="Target" value={task.target_name} readOnly />
              <Select
                label="Property"
                data={['Volume', 'Play', 'Stop']}
                value={task.property}
                onChange={() => undefined}
                allowDeselect={false}
              />
              <GroupRow
                left={
                  <NumberInput
                    label="Target value"
                    value={task.target_value}
                    onChange={() => undefined}
                  />
                }
                right={
                  <NumberInput
                    label="Duration (s)"
                    value={task.duration_secs}
                    onChange={() => undefined}
                  />
                }
              />
              <Box>
                <Text size="xs" c="dimmed" mb={4}>
                  Volume · {task.target_value.toFixed(1)} dB
                </Text>
                <Slider value={task.target_value} min={-60} max={0} step={0.5} onChange={() => undefined} />
              </Box>
              <TextInput label="Output" value={task.output.name} readOnly />
            </Stack>
          </Paper>
        ))}
      </Stack>
    </div>
  );
}

function GroupRow({ left, right }: { left: ReactNode; right: ReactNode }) {
  return (
    <Box style={{ display: 'flex', gap: 12 }}>
      <Box style={{ flex: 1 }}>{left}</Box>
      <Box style={{ flex: 1 }}>{right}</Box>
    </Box>
  );
}
