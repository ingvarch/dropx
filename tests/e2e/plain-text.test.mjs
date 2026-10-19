import assert from 'node:assert/strict';
import { test } from 'bun:test';
import { chunkedBody, createApiKey, newClientIp, pathOf, request, upload } from './helpers.mjs';

const ANON_MAX_FILE_SIZE = 25 * 1024 * 1024;
const ANON_HOURLY_LIMIT = 10;
const TEXT_TYPE = 'text/plain; charset=utf-8';

// Bun's fetch sends `Accept: */*` when none is set, so this is also what a client without Accept sends.
function plainUpload(body, { headers, ...options } = {}) {
  return upload(body, { filename: 'plain.txt', ...options, headers: { Accept: '*/*', ...headers } });
}

function assertTextError(response, status, message) {
  assert.equal(response.status, status);
  assert.equal(response.headers.get('Content-Type'), TEXT_TYPE);
  assert.match(response.headers.get('Vary'), /Accept/i);
  return response.text().then((body) => assert.equal(body, `Error: ${message}\n`));
}

for (const [name, headers] of [['Accept: */*', { Accept: '*/*' }], ['Accept: text/plain', { Accept: 'text/plain' }]]) {
  test(`upload with ${name} answers the download link as plain text`, async () => {
    const response = await plainUpload('plain bytes', { headers });

    assert.equal(response.status, 200);
    assert.equal(response.headers.get('Content-Type'), TEXT_TYPE);
    assert.match(response.headers.get('Vary'), /Accept/i);
    const body = await response.text();
    assert.match(body, /^[^\s]+\n$/);
    const url = body.slice(0, -1);
    assert.equal((await (await request(pathOf(url))).text()), 'plain bytes');
    const deleteUrl = response.headers.get('X-Url-Delete');
    assert.ok(deleteUrl, 'X-Url-Delete missing');
    assert.equal((await request(pathOf(deleteUrl), { method: 'DELETE' })).status, 200);
    assert.equal((await request(pathOf(url))).status, 404);
  });
}

test('upload with Accept: text/html answers plain text', async () => {
  const response = await plainUpload('x', { headers: { Accept: 'text/html' } });

  assert.equal(response.status, 200);
  assert.equal(response.headers.get('Content-Type'), TEXT_TYPE);
  assert.match(await response.text(), /^[^\s]+\n$/);
});

test('upload with Accept: application/json answers the JSON and Vary: Accept', async () => {
  const response = await upload('json bytes');

  assert.equal(response.status, 200);
  assert.match(response.headers.get('Content-Type'), /^application\/json/);
  assert.match(response.headers.get('Vary'), /Accept/i);
  assert.ok(response.headers.get('X-Url-Delete'));
  const body = await response.json();
  assert.equal(body.size, 10);
  assert.equal(response.headers.get('X-Url-Delete'), body.delete_url);
});

test('upload with a mixed Accept that lists JSON answers JSON', async () => {
  const response = await upload('x', { headers: { Accept: 'application/json, */*;q=0.5' } });

  assert.equal((await response.json()).size, 1);
});

test('plain text 413 from Content-Length', async () => {
  const response = await plainUpload(Buffer.alloc(ANON_MAX_FILE_SIZE + 1));

  await assertTextError(response, 413, 'Maximum file size is 25 MB');
});

test('plain text 400 for Max-Age-Days 0', async () => {
  const response = await plainUpload('x', { headers: { 'Max-Age-Days': '0' } });

  await assertTextError(response, 400, 'Retention period must be at least 1 day');
});

test('plain text 401 for an unknown API key', async () => {
  const response = await plainUpload('x', { headers: { 'X-API-Key': 'unknown-key' } });

  await assertTextError(response, 401, 'The provided API key is invalid or inactive');
});

test('plain text 411 for a chunked body over 25 MB with an API key', async () => {
  const key = await createApiKey('standard');

  const response = await plainUpload(chunkedBody(Buffer.alloc(ANON_MAX_FILE_SIZE + 5)), { duplex: 'half', headers: { 'X-API-Key': key } });

  await assertTextError(response, 411, 'Send a Content-Length header for files over 25 MB');
});

test('plain text 413 while buffering a chunked anonymous body', async () => {
  const response = await plainUpload(chunkedBody(Buffer.alloc(ANON_MAX_FILE_SIZE + 3)), { duplex: 'half' });

  await assertTextError(response, 413, 'Maximum file size is 25 MB');
});

test('plain text 429 keeps Retry-After', async () => {
  const ip = newClientIp();
  for (let i = 0; i < ANON_HOURLY_LIMIT; i++) {
    assert.equal((await plainUpload(`n${i}`, { ip })).status, 200);
  }

  const limited = await plainUpload('over', { ip });

  await assertTextError(limited, 429, 'Hourly rate limit exceeded');
  assert.match(limited.headers.get('Retry-After'), /^\d+$/);
});

test('JSON errors keep today\'s bodies and send Vary: Accept', async () => {
  const tooLarge = await upload(Buffer.alloc(ANON_MAX_FILE_SIZE + 1));
  assert.equal(tooLarge.status, 413);
  assert.match(tooLarge.headers.get('Vary'), /Accept/i);
  assert.deepEqual(await tooLarge.json(), {
    error: 'File too large',
    message: 'Maximum file size is 25 MB',
    max_size: ANON_MAX_FILE_SIZE,
  });

  const retention = await upload('x', { headers: { 'Max-Age-Days': '0' } });
  assert.equal(retention.status, 400);
  assert.match(retention.headers.get('Vary'), /Accept/i);
  assert.deepEqual(await retention.json(), {
    error: 'Invalid retention period',
    message: 'Retention period must be at least 1 day',
  });

  const unauthorized = await upload('x', { headers: { 'X-API-Key': 'unknown-key' } });
  assert.equal(unauthorized.status, 401);
  assert.match(unauthorized.headers.get('Vary'), /Accept/i);
  assert.deepEqual(await unauthorized.json(), {
    error: 'Invalid API key',
    message: 'The provided API key is invalid or inactive',
  });
});
