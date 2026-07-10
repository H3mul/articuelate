import { useEffect } from "react";

import { AppShell, MantineProvider } from "@mantine/core";

import StatusBar from "./components/StatusBar";
import Toolbar from "./components/Toolbar";
import { refreshFromBackend } from "./store";
import { theme } from "./theme";
import Workspace from "./layout/Workspace";

export default function App() {
  useEffect(() => {
    refreshFromBackend();
  }, []);

  return (
    <MantineProvider theme={theme} defaultColorScheme="dark">
      <AppShell
        header={{ height: 54 }}
        footer={{ height: 26 }}
        padding={0}
        styles={{
          root: {
            display: "flex",
            flexDirection: "column",
            height: "100vh",
            overflow: "hidden",
          },
          header: {
            position: "static",
            flex: "0 0 auto",
          },
          footer: {
            position: "static",
            flex: "0 0 auto",
          },
          main: {
            flex: "1 1 auto",
            minHeight: 0,
            padding: 0,
            overflow: "hidden",
          },
        }}
      >
        <AppShell.Header>
          <Toolbar />
        </AppShell.Header>
        <AppShell.Main>
          <Workspace />
        </AppShell.Main>
        <AppShell.Footer>
          <StatusBar />
        </AppShell.Footer>
      </AppShell>
    </MantineProvider>
  );
}
