import assert from 'node:assert/strict';
import { test } from 'node:test';
import { adminRequest, createApiKey, request, upload, uploadOk } from './helpers.mjs';

test('admin routes need the admin token', async () => {
  const response = await request('/admin/stats');

  assert.equal(response.status, 401);
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
