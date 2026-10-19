import assert from 'node:assert/strict';
import { test } from 'bun:test';
import { adminRequest, createApiKey, ipUserId, newClientIp, upload } from './helpers.mjs';

const ANON_HOURLY_LIMIT = 10;

async function uploadTimes(count, ip) {
  for (let i = 0; i < count; i++) {
    const response = await upload(`n${i}`, { ip });
    assert.equal(response.status, 200, `upload ${i + 1}: ${await response.text()}`);
  }
}

async function reachLimit(ip) {
  await uploadTimes(ANON_HOURLY_LIMIT, ip);
  assert.equal((await upload('over', { ip })).status, 429);
}

test('limits are counted per client', async () => {
  await reachLimit(newClientIp());

  assert.equal((await upload('other', { ip: newClientIp() })).status, 200);
});

test('admin reset clears the counters of one client', async () => {
  const ip = newClientIp();
  await reachLimit(ip);

  const reset = await adminRequest(`/admin/rate-limit/reset/${ipUserId(ip)}`, { method: 'POST' });

  assert.equal(reset.status, 200);
  assert.equal((await upload('again', { ip })).status, 200);
});

test('anonymous client is limited to 10 uploads per hour and told when to retry', async () => {
  const ip = newClientIp();
  await uploadTimes(ANON_HOURLY_LIMIT, ip);

  const limited = await upload('over', { ip });

  assert.equal(limited.status, 429);
  const retryAfter = limited.headers.get('Retry-After');
  assert.match(retryAfter, /^\d+$/);
  assert.ok(Number(retryAfter) >= 1 && Number(retryAfter) <= 3600, `Retry-After: ${retryAfter}`);
  assert.deepEqual(await limited.json(), { error: 'Rate limit exceeded', message: 'Hourly rate limit exceeded' });
});

test('uploads rejected with 400 do not use the rate limit', async () => {
  const ip = newClientIp();
  for (let i = 0; i < ANON_HOURLY_LIMIT; i++) {
    const response = await upload(`n${i}`, { ip, headers: { 'Max-Age-Days': '999' } });
    assert.equal(response.status, 400, `rejected upload ${i + 1}`);
  }

  await uploadTimes(ANON_HOURLY_LIMIT, ip);
});

test('an API key raises the hourly limit to its tier', async () => {
  const key = await createApiKey('standard');
  const ip = newClientIp();

  for (let i = 0; i <= ANON_HOURLY_LIMIT; i++) {
    const response = await upload(`n${i}`, { ip, headers: { 'X-API-Key': key } });
    assert.equal(response.status, 200, `upload ${i + 1}: ${await response.text()}`);
  }
});
