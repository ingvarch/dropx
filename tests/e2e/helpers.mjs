import assert from 'node:assert/strict';
import { createHash, randomInt } from 'node:crypto';

export const BASE_URL = process.env.DROPX_URL;
export const ADMIN_TOKEN = process.env.DROPX_ADMIN_TOKEN;
assert.ok(BASE_URL && ADMIN_TOKEN, 'start the e2e tests with `npm run test:e2e`');

const IP_HASH_HEX_LENGTH = 32;

// A random client IP gives each test its own rate limit counters.
export function newClientIp() {
  return `10.${randomInt(256)}.${randomInt(256)}.${randomInt(1, 255)}`;
}

// Same as hash_identifier in src/utils/auth.rs: the rate limiter's id for an anonymous client.
export function ipUserId(ip) {
  return createHash('sha256').update(ip).digest('hex').slice(0, IP_HASH_HEX_LENGTH);
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

export async function createApiKey(tier) {
  const response = await adminRequest('/admin/api-keys', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email: 'e2e@example.com', tier }),
  });
  assert.equal(response.status, 201, await response.clone().text());
  return (await response.json()).key;
}

// Upload responses use the public https host; tests send the path to the local Worker.
export function pathOf(url) {
  return new URL(url).pathname;
}
