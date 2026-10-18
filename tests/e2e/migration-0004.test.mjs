// Applies the migrations to its own fresh local D1. It does not use the running Worker.
import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { after, before, test } from 'node:test';
import { promisify } from 'node:util';

const run = promisify(execFile);
const ROOT = join(import.meta.dirname, '..', '..');
const WRANGLER = join(ROOT, 'node_modules', '.bin', 'wrangler');
const DATABASE = 'dropx-db';
const MIGRATION_TIMEOUT_MS = 2 * 60 * 1000;
const LEGACY_FILES = ['0001_initial.sql', '0002_add_delete_token.sql', '0003_add_max_downloads.sql'];
const MIGRATION = '0004_hash_stored_secrets.sql';

const PLAIN_DELETE_TOKEN = 'plainDeleteTokenForMigrationTest';
const PLAIN_API_KEY = 'tfr_plainkeyformigrationtest0000000';
const PLAIN_UPLOADED_BY = 'a1b2c3d4e5f60718293a4b5c6d7e8f90';
// Rows written by the new version before 0004 runs already hold SHA-256 hex.
const HASHED_DELETE_TOKEN = 'b'.repeat(64);
const HASHED_API_KEY = 'c'.repeat(64);
const HASHED_UPLOADER = 'e'.repeat(64);

let stateDir;

async function execute(...args) {
  const { stdout } = await run(WRANGLER, ['d1', 'execute', DATABASE, '--local', '--persist-to', stateDir, '--json', ...args], { cwd: ROOT });
  return JSON.parse(stdout);
}

async function select(sql) {
  const [{ results }] = await execute('--command', sql);
  return results;
}

before(async () => {
  stateDir = mkdtempSync(join(tmpdir(), 'dropx-migration-'));
  const legacy = LEGACY_FILES.map((name) => readFileSync(join(ROOT, 'migrations', name), 'utf8')).join('\n');
  const legacyFile = join(stateDir, 'legacy.sql');
  writeFileSync(legacyFile, legacy);
  await execute('--file', legacyFile);

  await execute('--command', `INSERT INTO files (id, original_filename, size, mime_type, created_at, expires_at, download_count, uploaded_by, delete_token, max_downloads)
    VALUES ('legacyfile', 'a.txt', 5, 'text/plain', 1000, 2000, 3, '${PLAIN_UPLOADED_BY}', '${PLAIN_DELETE_TOKEN}', 2)`);
  await execute('--command', `INSERT INTO api_keys (key, user_email, created_at, is_active, tier)
    VALUES ('${PLAIN_API_KEY}', 'legacy@example.com', 1000, 1, 'premium')`);
  await execute('--command', `INSERT INTO files (id, original_filename, size, mime_type, created_at, expires_at, download_count, uploaded_by, delete_token)
    VALUES ('hashedfile', 'b.txt', 5, 'text/plain', 1000, 2000, 0, '${'d'.repeat(32)}', '${HASHED_DELETE_TOKEN}')`);
  await execute('--command', `INSERT INTO api_keys (key, user_email, created_at, is_active, tier)
    VALUES ('${HASHED_API_KEY}', 'new@example.com', 1000, 1, 'standard')`);
  // Keyed uploads: the old version stored the plain key, the new one stores its hash.
  await execute('--command', `INSERT INTO files (id, original_filename, size, mime_type, created_at, expires_at, download_count, uploaded_by, delete_token)
    VALUES ('legacykeyed', 'c.txt', 5, 'text/plain', 1000, 2000, 0, '${PLAIN_API_KEY}', '${PLAIN_DELETE_TOKEN}x')`);
  await execute('--command', `INSERT INTO files (id, original_filename, size, mime_type, created_at, expires_at, download_count, uploaded_by, delete_token)
    VALUES ('hashedkeyed', 'd.txt', 5, 'text/plain', 1000, 2000, 0, '${HASHED_UPLOADER}', '${'f'.repeat(64)}')`);

  await execute('--file', join(ROOT, 'migrations', MIGRATION));
}, { timeout: MIGRATION_TIMEOUT_MS });

after(() => rmSync(stateDir, { recursive: true, force: true, maxRetries: 3 }));

test('0004 replaces the plain delete token and the uploader id', async () => {
  const [file] = await select("SELECT delete_token, uploaded_by FROM files WHERE id = 'legacyfile'");

  assert.match(file.delete_token, /^[0-9a-f]{64}$/);
  assert.match(file.uploaded_by, /^[0-9a-f]{32}$/);
  assert.notEqual(file.delete_token, PLAIN_DELETE_TOKEN);
  assert.notEqual(file.uploaded_by, PLAIN_UPLOADED_BY);
});

test('0004 keeps the other fields of a file row', async () => {
  const [file] = await select("SELECT * FROM files WHERE id = 'legacyfile'");

  assert.deepEqual(
    { id: file.id, name: file.original_filename, size: file.size, mime: file.mime_type, created: file.created_at, expires: file.expires_at, downloads: file.download_count, max: file.max_downloads },
    { id: 'legacyfile', name: 'a.txt', size: 5, mime: 'text/plain', created: 1000, expires: 2000, downloads: 3, max: 2 },
  );
});

test('0004 keeps delete tokens and API keys already stored as hashes', async () => {
  const [file] = await select("SELECT delete_token FROM files WHERE id = 'hashedfile'");
  const [key] = await select("SELECT key, is_active FROM api_keys WHERE user_email = 'new@example.com'");

  assert.equal(file.delete_token, HASHED_DELETE_TOKEN);
  assert.deepEqual({ key: key.key, active: key.is_active }, { key: HASHED_API_KEY, active: 1 });
});

test('0004 keeps the uploader id of a keyed upload already stored as a hash', async () => {
  const [file] = await select("SELECT uploaded_by FROM files WHERE id = 'hashedkeyed'");

  assert.equal(file.uploaded_by, HASHED_UPLOADER);
});

test('0004 replaces the plain API key and deactivates it', async () => {
  const keys = await select("SELECT * FROM api_keys WHERE user_email = 'legacy@example.com'");

  assert.equal(keys.length, 1);
  assert.match(keys[0].key, /^[0-9a-f]{64}$/);
  assert.notEqual(keys[0].key, PLAIN_API_KEY);
  assert.equal(keys[0].is_active, 0);
  assert.deepEqual(
    { email: keys[0].user_email, created: keys[0].created_at, tier: keys[0].tier },
    { email: 'legacy@example.com', created: 1000, tier: 'premium' },
  );
});

test('no plain value is left anywhere in the database', async () => {
  const files = JSON.stringify(await select('SELECT * FROM files'));
  const keys = JSON.stringify(await select('SELECT * FROM api_keys'));

  for (const plain of [PLAIN_DELETE_TOKEN, PLAIN_API_KEY, PLAIN_UPLOADED_BY]) {
    assert.ok(!files.includes(plain) && !keys.includes(plain), `${plain} is still stored`);
  }
});
