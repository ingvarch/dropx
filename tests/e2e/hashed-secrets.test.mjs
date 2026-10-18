import assert from 'node:assert/strict';
import { test } from 'node:test';
import { createApiKey, d1Raw, ipUserId, newClientIp, pathOf, sha256Hex, uploadOk } from './helpers.mjs';

const IP_HASH_HEX_LENGTH = 32;

async function storedColumn(sql, param) {
  const { results } = await d1Raw(sql, [param]);
  assert.equal(results.rows.length, 1, `expected one row for ${sql}`);
  return results.rows[0][0];
}

const deleteTokenOf = (file) => pathOf(file.delete_url).split('/')[2];

test('the stored delete token is the SHA-256 of the token in delete_url', async () => {
  const file = await uploadOk('hello');
  const token = deleteTokenOf(file);

  const stored = await storedColumn('SELECT delete_token FROM files WHERE id = ?1', file.id);

  assert.notEqual(stored, token);
  assert.equal(stored, sha256Hex(token));
});

test('the stored API key is the SHA-256 of the key from the create response', async () => {
  const key = await createApiKey('standard');

  const stored = await storedColumn('SELECT key FROM api_keys WHERE key = ?1', sha256Hex(key));
  assert.equal(stored, sha256Hex(key));

  const { results } = await d1Raw('SELECT key FROM api_keys WHERE key = ?1', [key]);
  assert.equal(results.rows.length, 0, 'the plain key must not be stored');
});

test('uploaded_by of a keyed upload is the key hash', async () => {
  const key = await createApiKey('standard');
  const file = await uploadOk('hello', { headers: { 'X-API-Key': key } });

  const uploadedBy = await storedColumn('SELECT uploaded_by FROM files WHERE id = ?1', file.id);

  assert.equal(uploadedBy, sha256Hex(key));
});

test('uploaded_by of an anonymous upload is the HMAC of the client IP', async () => {
  const ip = newClientIp();
  const file = await uploadOk('hello', { ip });

  const uploadedBy = await storedColumn('SELECT uploaded_by FROM files WHERE id = ?1', file.id);

  assert.equal(uploadedBy, ipUserId(ip));
  assert.notEqual(uploadedBy, sha256Hex(ip).slice(0, IP_HASH_HEX_LENGTH));
});
