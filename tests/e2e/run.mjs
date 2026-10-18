// Starts the Worker with `wrangler dev` on fresh local state, runs the e2e tests against it, then stops it.
import { spawn } from 'node:child_process';
import { randomBytes } from 'node:crypto';
import { mkdtempSync, rmSync } from 'node:fs';
import { createServer } from 'node:net';
import { constants, tmpdir } from 'node:os';
import { join } from 'node:path';
import { stripVTControlCharacters } from 'node:util';

const ROOT = join(import.meta.dirname, '..', '..');
const WRANGLER = join(ROOT, 'node_modules', '.bin', 'wrangler');
const DATABASE = 'dropx-db';
const HOST = '127.0.0.1';
// The first run compiles the Worker from scratch.
const READY_TIMEOUT_MS = 10 * 60 * 1000;
const STOP_TIMEOUT_MS = 10 * 1000;
const TEST_TIMEOUT_MS = 30 * 1000;
const TEST_FILES = 'tests/e2e/**/*.test.mjs';
const STOP_SIGNALS = ['SIGINT', 'SIGTERM', 'SIGHUP'];

function freePort() {
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.unref();
    server.on('error', reject);
    server.listen(0, HOST, () => {
      const { port } = server.address();
      server.close(() => resolve(port));
    });
  });
}

// stdin is closed, so wrangler takes the default answer instead of waiting for a prompt.
function run(command, args, env = process.env) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, { cwd: ROOT, env, stdio: ['ignore', 'inherit', 'inherit'] });
    child.on('error', reject);
    child.on('close', (code, signal) => resolve(code ?? (signal ? 1 : 0)));
  });
}

function startWorker(stateDir, port, adminToken) {
  const args = [
    'dev',
    '--ip', HOST,
    '--port', String(port),
    '--persist-to', stateDir,
    '--var', `ADMIN_TOKEN:${adminToken}`,
    '--show-interactive-dev-session=false',
  ];
  // Own process group, so stopping it also stops workerd.
  const child = spawn(WRANGLER, args, { cwd: ROOT, detached: true, stdio: ['ignore', 'pipe', 'pipe'] });
  const closed = new Promise((resolve) => child.on('close', resolve));
  let output = '';

  const killGroup = (signal) => {
    try {
      process.kill(-child.pid, signal);
    } catch (error) {
      if (error.code !== 'ESRCH') throw error;
    }
  };

  const stop = async () => {
    killGroup('SIGTERM');
    const timer = setTimeout(() => killGroup('SIGKILL'), STOP_TIMEOUT_MS);
    await closed;
    clearTimeout(timer);
  };

  const ready = new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error('wrangler dev did not become ready in time')), READY_TIMEOUT_MS);
    const onData = (chunk) => {
      output += chunk;
      if (stripVTControlCharacters(output).includes('Ready on')) {
        clearTimeout(timer);
        resolve(`http://${HOST}:${port}`);
      }
    };
    child.stdout.on('data', onData);
    child.stderr.on('data', onData);
    closed.then((code) => {
      clearTimeout(timer);
      reject(new Error(`wrangler dev exited with code ${code} before it was ready`));
    });
  });

  return { ready, stop, killGroup, output: () => output };
}

async function main() {
  const stateDir = mkdtempSync(join(tmpdir(), 'dropx-e2e-'));
  const adminToken = randomBytes(16).toString('hex');
  let worker;

  const cleanUp = () => rmSync(stateDir, { recursive: true, force: true, maxRetries: 3 });
  // A signal to this process does not reach the detached wrangler group.
  for (const signal of STOP_SIGNALS) {
    process.once(signal, () => {
      worker?.killGroup('SIGKILL');
      cleanUp();
      process.exit(128 + constants.signals[signal]);
    });
  }
  process.once('exit', () => worker?.killGroup('SIGKILL'));

  try {
    const migrated = await run(WRANGLER, ['d1', 'migrations', 'apply', DATABASE, '--local', '--persist-to', stateDir]);
    if (migrated !== 0) {
      return migrated;
    }

    worker = startWorker(stateDir, await freePort(), adminToken);
    const url = await worker.ready;

    const env = { ...process.env, DROPX_URL: url, DROPX_ADMIN_TOKEN: adminToken };
    const testArgs = ['--test', '--test-reporter=spec', `--test-timeout=${TEST_TIMEOUT_MS}`, TEST_FILES];
    const code = await run(process.execPath, testArgs, env);
    if (code !== 0) {
      console.error('\n--- wrangler dev output ---\n' + worker.output());
    }
    return code;
  } catch (error) {
    console.error(error.message);
    if (worker) {
      console.error('\n--- wrangler dev output ---\n' + worker.output());
    }
    return 1;
  } finally {
    await worker?.stop();
    cleanUp();
  }
}

process.exitCode = await main();
