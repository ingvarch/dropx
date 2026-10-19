import assert from 'node:assert/strict';
import { test } from 'bun:test';
import { ADMIN_TOKEN, adminRequest, createApiKey, createApiKeyResponse, request, upload, uploadOk } from './helpers.mjs';

test('admin routes need the admin token', async () => {
  const response = await request('/admin/stats');

  assert.equal(response.status, 401);
});

test('admin routes reject a wrong admin token', async () => {
  const lastCharChanged = ADMIN_TOKEN.slice(0, -1) + (ADMIN_TOKEN.endsWith('x') ? 'y' : 'x');
  const prefix = ADMIN_TOKEN.slice(0, -1);

  for (const token of [lastCharChanged, prefix]) {
    const response = await request('/admin/stats', { headers: { 'X-Admin-Token': token } });

    assert.equal(response.status, 401);
    assert.equal((await response.json()).error, 'Unauthorized');
  }
});

test('stats count stored files and active API keys', async () => {
  await uploadOk('hello');
  await createApiKey('standard');

  const response = await adminRequest('/admin/stats');

  assert.equal(response.status, 200);
  const stats = await response.json();
  assert.ok(stats.total_files >= 1);
  assert.ok(stats.api_keys_count >= 1);
});

for (const [tier, limits] of [['standard', { hourly: 100, daily: 500 }], ['premium', { hourly: 1000, daily: 5000 }]]) {
  test(`creating a ${tier} API key returns the key and its limits`, async () => {
    const created = await createApiKeyResponse(tier);

    assert.match(created.key, /^tfr_[0-9a-f]{32}$/);
    assert.equal(created.tier, tier);
    assert.deepEqual(created.limits, limits);
  });
}

test('an API key raises the limits to its tier', async () => {
  const key = await createApiKey('standard');

  const file = await uploadOk('hello', { headers: { 'X-API-Key': key, 'Max-Age-Days': '7' } });

  assert.equal(file.expires_in, '7d');
});

test('a revoked API key is rejected', async () => {
  const key = await createApiKey('standard');
  assert.equal((await upload('hello', { headers: { 'X-API-Key': key } })).status, 200);

  const revoked = await adminRequest(`/admin/api-keys/${key}`, { method: 'DELETE' });
  assert.equal(revoked.status, 200);

  assert.equal((await upload('hello', { headers: { 'X-API-Key': key } })).status, 401);
});

test('revoking an unknown API key returns 404', async () => {
  const response = await adminRequest('/admin/api-keys/tfr_does_not_exist', { method: 'DELETE' });

  assert.equal(response.status, 404);
  assert.equal((await response.json()).error, 'NotFound');
});

test('revoking the same API key twice returns 200 both times', async () => {
  const key = await createApiKey('standard');

  for (let attempt = 0; attempt < 2; attempt++) {
    const response = await adminRequest(`/admin/api-keys/${key}`, { method: 'DELETE' });
    assert.equal(response.status, 200);
  }
});
