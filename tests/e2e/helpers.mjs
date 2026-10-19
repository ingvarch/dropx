import assert from 'node:assert/strict';
import { createHash, createHmac, randomInt } from 'node:crypto';

export const BASE_URL = process.env.DROPX_URL;
export const ADMIN_TOKEN = process.env.DROPX_ADMIN_TOKEN;
const IP_HASH_KEY = process.env.DROPX_IP_HASH_KEY;
assert.ok(BASE_URL && ADMIN_TOKEN && IP_HASH_KEY, 'start the e2e tests with `bun run test:e2e`');

const IP_HASH_HEX_LENGTH = 32;

// A random client IP gives each test its own rate limit counters.
export function newClientIp() {
  return `10.${randomInt(256)}.${randomInt(256)}.${randomInt(1, 255)}`;
}

// A random IPv6 /64 network: `address(n)` gives distinct addresses inside it, `prefix` is the network as the Worker names it.
export function newIpv6Network() {
  // Non-zero groups, so the prefix has no zero run that the Worker would shorten to `::`.
  const hextet = () => randomInt(1, 0x10000).toString(16);
  const prefix = `2001:db8:${hextet()}:${hextet()}`;
  return { prefix: `${prefix}::/64`, address: (n) => `${prefix}::${n.toString(16)}` };
}

export function sha256Hex(value) {
  return createHash('sha256').update(value).digest('hex');
}

// Same as hash_ip in src/utils/auth.rs: the rate limiter's id for an anonymous client.
export function ipUserId(ip) {
  return createHmac('sha256', IP_HASH_KEY).update(ip).digest('hex').slice(0, IP_HASH_HEX_LENGTH);
}

export function request(path, { ip = newClientIp(), headers = {}, ...init } = {}) {
  return fetch(new URL(path, BASE_URL), { ...init, headers: { 'CF-Connecting-IP': ip, ...headers } });
}

export function upload(body, { filename = 'file.txt', ...options } = {}) {
  return request(`/${filename}`, { method: 'PUT', body, ...options });
}

export async function uploadOk(body, options) {
  const response = await upload(body, options);
  assert.equal(response.status, 200, await response.clone().text());
  return response.json();
}

export function adminRequest(path, { headers = {}, ...init } = {}) {
  return request(path, { ...init, headers: { 'X-Admin-Token': ADMIN_TOKEN, ...headers } });
}

export async function createApiKeyResponse(tier) {
  const response = await adminRequest('/admin/api-keys', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email: 'e2e@example.com', tier }),
  });
  assert.equal(response.status, 201, await response.clone().text());
  return response.json();
}

export async function createApiKey(tier) {
  return (await createApiKeyResponse(tier)).key;
}

export function pathOf(url) {
  return new URL(url).pathname;
}

const EXPLORER_API = '/cdn-cgi/local/explorer/api/r2/buckets';

async function explorerJson(path) {
  const response = await fetch(new URL(path, BASE_URL));
  assert.equal(response.status, 200, await response.clone().text());
  return (await response.json());
}

// Sizes of all objects in the local R2 bucket, read through the wrangler dev Local Explorer API.
export async function listR2ObjectSizes() {
  const { result } = await explorerJson(EXPLORER_API);
  assert.equal(result.buckets.length, 1, 'expected one R2 bucket');
  const objectsPath = `${EXPLORER_API}/${result.buckets[0].name}/objects`;

  const sizes = [];
  let cursor;
  do {
    const query = cursor ? `?cursor=${encodeURIComponent(cursor)}` : '';
    const page = await explorerJson(objectsPath + query);
    sizes.push(...page.result.map((object) => object.size));
    cursor = String(page.result_info.is_truncated) === 'true' ? page.result_info.cursor : undefined;
  } while (cursor);
  return sizes;
}

const D1_EXPLORER_API = '/cdn-cgi/local/explorer/api/d1/database';

export async function d1Raw(sql, params) {
  const { result } = await explorerJson(D1_EXPLORER_API);
  assert.equal(result.length, 1, 'expected one D1 database');
  const response = await fetch(new URL(`${D1_EXPLORER_API}/${result[0].uuid}/raw`, BASE_URL), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ sql, params }),
  });
  assert.equal(response.status, 200, await response.clone().text());
  const [statement] = (await response.json()).result;
  return statement;
}

// Sets expires_at of a file in the local D1 through the Local Explorer API. The API accepts string params only.
export async function setExpiresAt(fileId, expiresAt) {
  const { meta } = await d1Raw('UPDATE files SET expires_at = ?1 WHERE id = ?2', [String(expiresAt), fileId]);
  assert.equal(meta.changes, 1, 'expected one updated row');
}

// Renames a file row and expires it. Keys over 1024 bytes make the local R2 reject the delete.
export async function renameExpiredFile(fileId, newId, expiresAt) {
  const { meta } = await d1Raw('UPDATE files SET id = ?1, expires_at = ?2 WHERE id = ?3', [newId, String(expiresAt), fileId]);
  assert.equal(meta.changes, 1, 'expected one updated row');
}

export async function fileRowExists(fileId) {
  const { results } = await d1Raw('SELECT id FROM files WHERE id = ?1', [fileId]);
  return results.rows.length > 0;
}

// Runs the cron handler once; wrangler dev serves it at this path.
export async function runScheduledCleanup() {
  const response = await fetch(new URL('/cdn-cgi/local/scheduled', BASE_URL));
  assert.equal(response.status, 200, await response.clone().text());
}
