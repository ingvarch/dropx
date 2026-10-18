import assert from 'node:assert/strict';
import { test } from 'node:test';
import { adminRequest, ipUserId, newClientIp, upload } from './helpers.mjs';

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

test('anonymous client is limited to 10 uploads per hour', async () => {
  const ip = newClientIp();
  await uploadTimes(ANON_HOURLY_LIMIT, ip);

  const limited = await upload('over', { ip });

  assert.equal(limited.status, 429);
});

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
