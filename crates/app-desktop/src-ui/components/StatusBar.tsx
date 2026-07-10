import { Group, Text } from '@mantine/core';

import { useAppData } from '../store';

export default function StatusBar() {
  const { status } = useAppData();

  const connected = status?.connected ?? false;
  const device = status?.device_name ?? '—';

  return (
    <Group
      h="100%"
      px="sm"
      justify="space-between"
      gap="xs"
      style={{ borderTop: '1px solid var(--mantine-color-dark-4)' }}
    >
      <Text size="xs" c={connected ? 'teal' : 'red'}>
        {connected ? 'Connected' : 'Disconnected'} ({device})
      </Text>
      <Group gap="xs">
        <Text size="xs" c="dimmed">
          CPU: {status ? status.cpu_usage.toFixed(0) : '–'}%
        </Text>
        <Text size="xs" c="dimmed">
          DSP: {status ? status.dsp_usage.toFixed(0) : '–'}%
        </Text>
      </Group>
    </Group>
  );
}
