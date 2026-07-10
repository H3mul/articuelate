import { Group, TextInput, Button, Tooltip } from '@mantine/core';
import {
  IconPlayerPlayFilled,
  IconPlayerPauseFilled,
  IconArrowBackUp,
  IconAlertTriangle,
  IconSearch,
} from '@tabler/icons-react';

import { back, go, panicStop, pause } from '../ipc';
import { refreshFromBackend } from '../store';
import { setSearch, useAppData } from '../store';

export default function Toolbar() {
  const { search } = useAppData();

  const handleGo = async () => {
    await go();
    await refreshFromBackend();
  };

  const handlePanic = async () => {
    await panicStop();
    await refreshFromBackend();
  };

  return (
    <Group h="100%" px="sm" gap="sm" wrap="nowrap">
      <Tooltip label="Pause">
        <Button
          variant="default"
          size="md"
          onClick={() => pause()}
          leftSection={<IconPlayerPauseFilled size={18} />}
        >
          Pause
        </Button>
      </Tooltip>

      <Tooltip label="GO — fire next cue">
        <Button
          color="green"
          size="md"
          miw={90}
          onClick={handleGo}
          leftSection={<IconPlayerPlayFilled size={20} />}
        >
          GO
        </Button>
      </Tooltip>

      <Tooltip label="Back">
        <Button
          variant="default"
          size="md"
          onClick={() => back()}
          leftSection={<IconArrowBackUp size={18} />}
        >
          Back
        </Button>
      </Tooltip>

      <TextInput
        placeholder="Search cues…"
        leftSection={<IconSearch size={16} />}
        value={search}
        onChange={(e) => setSearch(e.currentTarget.value)}
        style={{ flex: 1, minWidth: 160 }}
      />

      <Tooltip label="Panic — stop all">
        <Button
          color="red"
          size="md"
          miw={90}
          onClick={handlePanic}
          leftSection={<IconAlertTriangle size={18} />}
        >
          PANIC
        </Button>
      </Tooltip>
    </Group>
  );
}
