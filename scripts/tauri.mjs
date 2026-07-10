// Cross-platform Tauri CLI launcher.
//
// Tauri's CLI only looks for `tauri.conf.json` in the current directory and its
// subfolders, so it must run from the repo root (where the config lives) rather
// than from `ui/`. This script resolves the locally-installed `@tauri-apps/cli`
// and executes it with the repo root as the working directory.
import { spawnSync } from 'node:child_process';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const require = createRequire(resolve(root, 'ui/package.json'));

const candidates = [
  '@tauri-apps/cli/tauri.js',
  '@tauri-apps/cli/bin/tauri.js',
];

let cliPath;
for (const c of candidates) {
  try {
    cliPath = require.resolve(c, { paths: [resolve(root, 'ui')] });
    break;
  } catch {
    // try next candidate
  }
}

if (!cliPath) {
  console.error(
    'Could not resolve @tauri-apps/cli. Run `npm --prefix ui install` first.',
  );
  process.exit(1);
}

const result = spawnSync(
  process.execPath,
  [cliPath, ...process.argv.slice(2)],
  { cwd: root, stdio: 'inherit' },
);

process.exit(result.status ?? 1);
