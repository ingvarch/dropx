import assert from 'node:assert/strict';
import { randomBytes } from 'node:crypto';
import { test } from 'bun:test';
import {
  BASE_URL,
  createApiKey,
  d1Raw,
  fileRowExists,
  listR2ObjectSizes,
  newClientIp,
  pathOf,
  renameExpiredFile,
  request,
  runScheduledCleanup,
  setExpiresAt,
  sha256Hex,
  upload,
  uploadOk,
} from './helpers.mjs';

const ANON_MAX_FILE_SIZE = 25 * 1024 * 1024;

test('upload returns the file id, size, expiry and links', async () => {
  const file = await uploadOk('hello', { filename: 'hello.txt' });

  assert.match(file.id, /^[1-9A-HJ-NP-Za-km-z]+$/);
  assert.equal(file.filename, 'hello.txt');
  assert.equal(file.size, 5);
  assert.equal(file.expires_in, '3d');
  assert.equal(pathOf(file.url), `/${file.id}`);
  assert.match(pathOf(file.delete_url), new RegExp(`^/${file.id}/\\w+$`));
});

test('download returns the uploaded bytes with metadata headers', async () => {
  const body = new Uint8Array([0, 1, 2, 250, 255]);
  const file = await uploadOk(body, { filename: 'data.bin' });

  const response = await request(`/${file.id}`);

  assert.equal(response.status, 200);
  assert.deepEqual(new Uint8Array(await response.arrayBuffer()), body);
  assert.equal(response.headers.get('Content-Disposition'), 'attachment; filename="data.bin"');
  assert.equal(response.headers.get('X-Download-Count'), '0');
  assert.equal(response.headers.get('X-Expires-At'), String(file.expires_at));
});

test('info returns metadata and counts downloads', async () => {
  const file = await uploadOk('hello');
  await request(`/${file.id}`);

  const response = await request(`/${file.id}/info`);

  assert.equal(response.status, 200);
  const info = await response.json();
  assert.equal(info.id, file.id);
  assert.equal(info.size, 5);
  assert.equal(info.download_count, 1);
});

test('unknown file id returns 404', async () => {
  const response = await request('/doesnotexist');

  assert.equal(response.status, 404);
  assert.equal((await response.json()).error, 'File not found');
});

test('delete with the delete token removes the file', async () => {
  const file = await uploadOk('hello');

  const deleted = await request(pathOf(file.delete_url), { method: 'DELETE' });

  assert.equal(deleted.status, 200);
  assert.equal((await deleted.json()).success, true);
  assert.equal((await request(`/${file.id}`)).status, 404);
});

test('delete with a wrong token keeps the file', async () => {
  const file = await uploadOk('hello');

  const deleted = await request(`/${file.id}/wrongtoken`, { method: 'DELETE' });

  assert.equal(deleted.status, 404);
  assert.equal((await deleted.json()).error, 'Unauthorized');
  assert.equal((await request(`/${file.id}`)).status, 200);
});

test('Max-Downloads stops serving the file after the last allowed download', async () => {
  const file = await uploadOk('hello', { headers: { 'Max-Downloads': '1' } });

  const first = await request(`/${file.id}`);
  assert.equal(first.status, 200);
  assert.equal(await first.text(), 'hello');

  assert.equal((await request(`/${file.id}`)).status, 404);
});

test('Max-Downloads keeps the file until the Nth download', async () => {
  const file = await uploadOk('hello', { headers: { 'Max-Downloads': '2' } });

  assert.equal((await request(`/${file.id}`)).status, 200);
  const info = await request(`/${file.id}/info`);
  assert.equal(info.status, 200);
  assert.equal((await info.json()).download_count, 1);

  assert.equal((await request(`/${file.id}`)).status, 200);
  assert.equal((await request(`/${file.id}`)).status, 404);
});

// data.bin is sent as an attachment by /:id, so each route shows that it sets its own disposition.
for (const [route, disposition] of [['get', 'attachment'], ['inline', 'inline']]) {
  test(`/${route}/:id serves the last allowed download and then returns 404`, async () => {
    const file = await uploadOk('hello', { filename: 'data.bin', headers: { 'Max-Downloads': '1' } });

    const first = await request(`/${route}/${file.id}`);
    assert.equal(first.status, 200);
    assert.equal(await first.text(), 'hello');
    assert.equal(first.headers.get('Content-Disposition'), `${disposition}; filename="data.bin"`);

    assert.equal((await request(`/${route}/${file.id}`)).status, 404);
  });
}

const PNG_BYTES = Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00]);
const HTML_BODY = '<html><script>alert(1)</script></html>';
const SVG_BODY = '<svg xmlns="http://www.w3.org/2000/svg"><script>alert(1)</script></svg>';
const XML_BODY = '<?xml version="1.0"?><root/>';

async function assertDownload(path, { type, disposition, body }) {
  const response = await request(path);
  assert.equal(response.status, 200);
  assert.equal(response.headers.get('Content-Type'), type);
  assert.equal(response.headers.get('Content-Disposition'), disposition);
  assert.equal(response.headers.get('X-Content-Type-Options'), 'nosniff');
  assert.deepEqual(Buffer.from(await response.arrayBuffer()), Buffer.from(body));
}

test('an HTML upload is never shown as a page', async () => {
  const file = await uploadOk(HTML_BODY, { filename: 'login.html' });

  await assertDownload(`/${file.id}`, { type: 'text/html', disposition: 'attachment; filename="login.html"', body: HTML_BODY });
  await assertDownload(`/inline/${file.id}`, { type: 'text/plain; charset=utf-8', disposition: 'inline; filename="login.html"', body: HTML_BODY });
  await assertDownload(`/get/${file.id}`, { type: 'text/html', disposition: 'attachment; filename="login.html"', body: HTML_BODY });
});

test('an SVG upload is never shown as an image page', async () => {
  const file = await uploadOk(SVG_BODY, { filename: 'logo.svg' });

  await assertDownload(`/${file.id}`, { type: 'image/svg+xml', disposition: 'attachment; filename="logo.svg"', body: SVG_BODY });
  await assertDownload(`/inline/${file.id}`, { type: 'text/plain; charset=utf-8', disposition: 'inline; filename="logo.svg"', body: SVG_BODY });
});

test('content that starts with <?xml is shown as text on /inline/:id', async () => {
  const file = await uploadOk(XML_BODY, { filename: 'data.bin' });

  await assertDownload(`/inline/${file.id}`, { type: 'text/plain; charset=utf-8', disposition: 'inline; filename="data.bin"', body: XML_BODY });
  await assertDownload(`/${file.id}`, { type: 'application/xml', disposition: 'attachment; filename="data.bin"', body: XML_BODY });
});

test('passive files keep their type and inline display on /:id', async () => {
  const text = await uploadOk('hello', { filename: 'hello.txt' });
  const png = await uploadOk(PNG_BYTES, { filename: 'pic.png' });

  await assertDownload(`/${text.id}`, { type: 'text/plain', disposition: 'inline; filename="hello.txt"', body: 'hello' });
  await assertDownload(`/inline/${text.id}`, { type: 'text/plain', disposition: 'inline; filename="hello.txt"', body: 'hello' });
  await assertDownload(`/${png.id}`, { type: 'image/png', disposition: 'inline; filename="pic.png"', body: PNG_BYTES });
  await assertDownload(`/get/${png.id}`, { type: 'image/png', disposition: 'attachment; filename="pic.png"', body: PNG_BYTES });
});

test('Max-Age-Days shortens the retention', async () => {
  const file = await uploadOk('hello', { headers: { 'Max-Age-Days': '1' } });

  assert.equal(file.expires_in, '1d');
});

test('Max-Age-Days above the tier default is rejected', async () => {
  const response = await upload('hello', { headers: { 'Max-Age-Days': '4' } });

  assert.equal(response.status, 400);
  assert.equal((await response.json()).error, 'Invalid retention period');
});

test('landing page is HTML for browsers and text for curl', async () => {
  const html = await request('/', { headers: { Accept: 'text/html' } });
  assert.equal(html.status, 200);
  assert.match(html.headers.get('Content-Type'), /^text\/html/);

  const text = await request('/', { headers: { Accept: '*/*' } });
  assert.match(text.headers.get('Content-Type'), /^text\/plain/);
});

test('HEAD / returns 200', async () => {
  const response = await request('/', { method: 'HEAD' });

  assert.equal(response.status, 200);
});

test('a rejected upload leaves no object in R2', async () => {
  const uniqueSize = 7919;

  const response = await upload('x'.repeat(uniqueSize), { headers: { 'Max-Age-Days': '999' } });

  assert.equal(response.status, 400);
  assert.ok(!(await listR2ObjectSizes()).includes(uniqueSize), 'object left in R2');
});

test('a failed D1 insert removes the object that was just written', async () => {
  const uniqueSize = 7907;
  await d1Raw(
    "CREATE TRIGGER fail_insert BEFORE INSERT ON files WHEN NEW.original_filename = 'fail-insert.bin' BEGIN SELECT RAISE(ABORT, 'forced insert failure'); END",
    [],
  );
  try {
    const response = await upload('x'.repeat(uniqueSize), { filename: 'fail-insert.bin' });

    assert.equal(response.status, 500);
    assert.ok(!(await listR2ObjectSizes()).includes(uniqueSize), 'object left in R2');
  } finally {
    await d1Raw('DROP TRIGGER fail_insert', []);
  }
});

const READ_ROUTES = [
  ['download', (id) => `/${id}`],
  ['/get', (id) => `/get/${id}`],
  ['/inline', (id) => `/inline/${id}`],
  ['info', (id) => `/${id}/info`],
];

for (const [name, pathFor] of READ_ROUTES) {
  test(`${name} returns 404 once the file has expired`, async () => {
    const file = await uploadOk('hello');
    await setExpiresAt(file.id, Math.floor(Date.now() / 1000) - 1);

    const response = await request(pathFor(file.id));

    assert.equal(response.status, 404);
    assert.equal((await response.json()).error, 'File not found');
  });
}

test('delete with the delete token still removes an expired file and its object', async () => {
  const body = 'x'.repeat(1000 + Math.floor(Math.random() * 100000));
  const file = await uploadOk(body);
  await setExpiresAt(file.id, Math.floor(Date.now() / 1000) - 1);

  const deleted = await request(pathOf(file.delete_url), { method: 'DELETE' });

  assert.equal(deleted.status, 200);
  assert.equal((await deleted.json()).success, true);
  assert.ok(!(await listR2ObjectSizes()).includes(body.length), 'object left in R2');
});

test('the cleanup job removes the row and the object of an expired file and keeps the others', async () => {
  const expiredBody = 'x'.repeat(2000 + Math.floor(Math.random() * 100000));
  const activeBody = 'y'.repeat(expiredBody.length + 1);
  const expired = await uploadOk(expiredBody);
  const active = await uploadOk(activeBody);
  await setExpiresAt(expired.id, Math.floor(Date.now() / 1000) - 1);

  await runScheduledCleanup();

  assert.equal(await fileRowExists(expired.id), false, 'row left in D1');
  const sizes = await listR2ObjectSizes();
  assert.ok(!sizes.includes(expiredBody.length), 'object left in R2');
  assert.ok(sizes.includes(activeBody.length), 'active object removed from R2');
  const download = await request(`/${active.id}`);
  assert.equal(download.status, 200);
  assert.equal(await download.text(), activeBody);
});

test('the cleanup job keeps the row of a file whose R2 delete failed', async () => {
  const failing = await uploadOk('z'.repeat(3000 + Math.floor(Math.random() * 100000)));
  const normal = await uploadOk('w'.repeat(500 + Math.floor(Math.random() * 1000)));
  const rejectedKey = 'k'.repeat(1100);
  const past = Math.floor(Date.now() / 1000) - 1;
  await renameExpiredFile(failing.id, rejectedKey, past);
  await setExpiresAt(normal.id, past);

  await runScheduledCleanup();

  assert.equal(await fileRowExists(normal.id), false, 'row of deleted object left in D1');
  assert.equal(await fileRowExists(rejectedKey), true, 'row of failed R2 delete removed');
});

// The row stays until the cleanup job removes it.
test('a file whose download limit is used up is not served again', async () => {
  const file = await uploadOk('once only', { headers: { 'Max-Downloads': '1' } });
  await d1Raw('UPDATE files SET download_count = 1 WHERE id = ?1', [file.id]);

  for (const path of [`/${file.id}`, `/get/${file.id}`, `/inline/${file.id}`, `/${file.id}/info`]) {
    assert.equal((await request(path)).status, 404, path);
  }
});

test('delete keeps the row of a file whose R2 delete failed', async () => {
  const file = await uploadOk('r'.repeat(500 + Math.floor(Math.random() * 1000)));
  const token = pathOf(file.delete_url).split('/')[2];
  const rejectedKey = 'd'.repeat(1100);
  await renameExpiredFile(file.id, rejectedKey, Math.floor(Date.now() / 1000) + 3600);

  const deleted = await request(`/${rejectedKey}/${token}`, { method: 'DELETE' });

  assert.equal(deleted.status, 500);
  assert.equal(await fileRowExists(rejectedKey), true, 'row removed although its R2 delete failed');
});

test('expires_at is the retention period after the upload', async () => {
  const before = Math.floor(Date.now() / 1000);
  const file = await uploadOk('hello');
  const after = Math.floor(Date.now() / 1000);

  const threeDays = 3 * 24 * 60 * 60;
  assert.ok(file.expires_at >= before + threeDays && file.expires_at <= after + threeDays, `expires_at: ${file.expires_at}`);
});

test('upload without a file name is stored as "file"', async () => {
  const response = await request('/', { method: 'POST', body: 'hello' });

  assert.equal(response.status, 200);
  assert.equal((await response.json()).filename, 'file');
});

test('upload above the anonymous size limit returns 413 and stores nothing', async () => {
  const limit = ANON_MAX_FILE_SIZE;
  const response = await upload(Buffer.alloc(limit + 1));

  assert.equal(response.status, 413);
  const body = await response.json();
  assert.equal(body.error, 'File too large');
  assert.equal(body.max_size, limit);
  assert.ok(!(await listR2ObjectSizes()).includes(limit + 1), 'object left in R2');
});

test('an API key raises the size limit to its tier', async () => {
  const key = await createApiKey('standard');
  // One byte more than the 413 test uploads, so that test can look for its own object size in R2.
  const size = ANON_MAX_FILE_SIZE + 2;

  const file = await uploadOk(Buffer.alloc(size), { headers: { 'X-API-Key': key } });

  assert.equal(file.size, size);
});

test('an API key with an unknown tier fails the upload and stores nothing', async () => {
  const key = 'unknown-tier-key';
  const uniqueSize = 7901;
  await d1Raw(
    "INSERT INTO api_keys (key, user_email, created_at, is_active, tier) VALUES (?1, 'e2e@example.com', 0, 1, 'Premium')",
    [sha256Hex(key)],
  );
  try {
    const response = await upload('x'.repeat(uniqueSize), { filename: 'unknown-tier.bin', headers: { 'X-API-Key': key } });

    assert.equal(response.status, 500);
    const { results } = await d1Raw('SELECT id FROM files WHERE original_filename = ?1', ['unknown-tier.bin']);
    assert.equal(results.rows.length, 0, 'row left in D1');
    assert.ok(!(await listR2ObjectSizes()).includes(uniqueSize), 'object left in R2');
  } finally {
    await d1Raw('DELETE FROM api_keys WHERE key = ?1', [sha256Hex(key)]);
  }
});

test('upload at the anonymous size limit succeeds', async () => {
  const limit = ANON_MAX_FILE_SIZE;

  const file = await uploadOk(Buffer.alloc(limit));

  assert.equal(file.size, limit);
});

test('upload links start with the Worker origin and work as returned', async () => {
  const response = await upload('origin check');
  const file = await response.json();

  assert.equal(file.url, `${BASE_URL}/${file.id}`);
  assert.ok(file.delete_url.startsWith(`${BASE_URL}/${file.id}/`), file.delete_url);
  assert.equal(response.headers.get('X-Url-Delete'), file.delete_url);

  const downloaded = await request(file.url);
  assert.equal(await downloaded.text(), 'origin check');

  const deleted = await request(file.delete_url, { method: 'DELETE' });
  assert.equal(deleted.status, 200);
});

function uniqueBody(char, base) {
  return char.repeat(base + Math.floor(Math.random() * 100000));
}

test('the last allowed download leaves the object in R2 until the cleanup job runs', async () => {
  const body = uniqueBody('m', 4000);
  const file = await uploadOk(body, { headers: { 'Max-Downloads': '1' } });

  const first = await request(`/${file.id}`);
  assert.equal(first.status, 200);
  assert.equal(await first.text(), body);
  assert.equal((await request(`/${file.id}`)).status, 404);
  assert.equal(await fileRowExists(file.id), true, 'row removed by the download');
  assert.ok((await listR2ObjectSizes()).includes(body.length), 'object removed by the download');

  await runScheduledCleanup();

  assert.equal(await fileRowExists(file.id), false, 'row left in D1');
  assert.ok(!(await listR2ObjectSizes()).includes(body.length), 'object left in R2');
});

test('the cleanup job keeps files below their download limit and files without a limit', async () => {
  const limitedBody = uniqueBody('n', 5000);
  const unlimitedBody = 'o'.repeat(limitedBody.length + 1);
  const limited = await uploadOk(limitedBody, { headers: { 'Max-Downloads': '2' } });
  const unlimited = await uploadOk(unlimitedBody);
  assert.equal((await request(`/${limited.id}`)).status, 200);

  await runScheduledCleanup();

  assert.equal(await fileRowExists(limited.id), true, 'row below its limit removed');
  assert.equal(await fileRowExists(unlimited.id), true, 'row without a limit removed');
  const sizes = await listR2ObjectSizes();
  assert.ok(sizes.includes(limitedBody.length), 'object below its limit removed');
  assert.ok(sizes.includes(unlimitedBody.length), 'object without a limit removed');
});

test('a large download returns the uploaded bytes and the Content-Length', async () => {
  const size = 20 * 1024 * 1024;
  const body = new Uint8Array(size);
  for (let offset = 0; offset < size; offset += 65536) {
    crypto.getRandomValues(body.subarray(offset, offset + 65536));
  }
  const file = await uploadOk(body, { filename: 'large.bin' });

  const response = await request(`/${file.id}`);

  assert.equal(response.status, 200);
  assert.equal(response.headers.get('Content-Length'), String(size));
  const received = new Uint8Array(await response.arrayBuffer());
  assert.equal(received.length, size);
  assert.equal(sha256Hex(received), sha256Hex(body));
});

const CHUNK_SIZE = 1024 * 1024;

// A body without Content-Length: bun sends a ReadableStream as chunked.
function chunkedBody(buffer) {
  let offset = 0;
  return new ReadableStream({
    pull(controller) {
      if (offset >= buffer.length) return controller.close();
      controller.enqueue(buffer.subarray(offset, offset + CHUNK_SIZE));
      offset += CHUNK_SIZE;
    },
  });
}

function uploadChunked(buffer, options = {}) {
  return upload(chunkedBody(buffer), { duplex: 'half', ...options });
}

test('a chunked upload of a small file is stored and downloads as the same bytes', async () => {
  const bytes = randomBytes(300_000);

  const file = await (async () => {
    const response = await uploadChunked(bytes);
    assert.equal(response.status, 200, await response.clone().text());
    return response.json();
  })();

  assert.equal(file.size, bytes.length);
  const downloaded = Buffer.from(await (await request(file.url)).arrayBuffer());
  assert.equal(sha256Hex(downloaded), sha256Hex(bytes));
});

test('a chunked anonymous upload over the size limit returns 413 and stores nothing', async () => {
  const size = ANON_MAX_FILE_SIZE + 3;

  const response = await uploadChunked(Buffer.alloc(size));

  assert.equal(response.status, 413);
  assert.equal((await response.json()).error, 'File too large');
  assert.ok(!(await listR2ObjectSizes()).includes(size), 'object left in R2');
});

test('a chunked upload over 25 MB with an API key returns 411 and stores nothing', async () => {
  const key = await createApiKey('standard');
  const size = ANON_MAX_FILE_SIZE + 5;

  const response = await uploadChunked(Buffer.alloc(size), { headers: { 'X-API-Key': key } });

  assert.equal(response.status, 411);
  const body = await response.json();
  assert.equal(body.error, 'Length required');
  assert.match(body.message, /Content-Length/);
  assert.ok(!(await listR2ObjectSizes()).includes(size), 'object left in R2');
});

test('a known-length upload of 30 MB with an API key is stored and downloads intact', async () => {
  const key = await createApiKey('standard');
  const bytes = randomBytes(30 * 1024 * 1024);

  const file = await uploadOk(bytes, { headers: { 'X-API-Key': key } });

  assert.equal(file.size, bytes.length);
  const downloaded = Buffer.from(await (await request(file.url)).arrayBuffer());
  assert.equal(sha256Hex(downloaded), sha256Hex(bytes));
});

test('a known-length upload over the size limit does not count against the rate limit', async () => {
  const ip = newClientIp();

  const rejected = await upload(Buffer.alloc(ANON_MAX_FILE_SIZE + 7), { ip });
  assert.equal(rejected.status, 413);

  for (let i = 0; i < 10; i++) {
    await uploadOk(`small ${i}`, { ip });
  }
});

test('the type is detected from the stored content for a file without an extension', async () => {
  const pdf = Buffer.concat([Buffer.from('%PDF-1.7\n'), randomBytes(2000)]);

  const file = await uploadOk(pdf, { filename: 'noext' });

  assert.equal(file.mime_type, 'application/pdf');
});

for (const [name, send] of [
  ['with Content-Length 0', () => upload(Buffer.alloc(0), { filename: 'empty.txt' })],
  ['without a body', () => upload(undefined, { filename: 'empty.txt' })],
  ['chunked', () => uploadChunked(Buffer.alloc(0), { filename: 'empty.txt' })],
]) {
  test(`an empty upload ${name} is stored as a 0-byte file typed by its extension`, async () => {
    const response = await send();
    assert.equal(response.status, 200, await response.clone().text());
    const file = await response.json();

    assert.equal(file.size, 0);
    assert.equal(file.mime_type, 'text/plain');
    const downloaded = await request(file.url);
    assert.equal(downloaded.status, 200);
    assert.equal((await downloaded.arrayBuffer()).byteLength, 0);
  });
}

test('a premium upload over 100 MB, the Cloudflare Pro plan body limit, returns 413', async () => {
  const cloudflareProLimit = 100 * 1024 * 1024;
  const key = await createApiKey('premium');

  const response = await upload(Buffer.alloc(cloudflareProLimit + 1), { headers: { 'X-API-Key': key } });

  assert.equal(response.status, 413);
  assert.equal((await response.json()).max_size, cloudflareProLimit);
});
