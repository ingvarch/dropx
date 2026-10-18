import assert from 'node:assert/strict';
import { test } from 'node:test';
import {
  fileRowExists,
  listR2ObjectSizes,
  pathOf,
  renameExpiredFile,
  request,
  runScheduledCleanup,
  setExpiresAt,
  upload,
  uploadOk,
} from './helpers.mjs';

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

test('Max-Downloads deletes the file after the last allowed download', async () => {
  const file = await uploadOk('hello', { headers: { 'Max-Downloads': '1' } });

  const first = await request(`/${file.id}`);
  assert.equal(first.status, 200);
  assert.equal(await first.text(), 'hello');

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

test('a rejected upload leaves no object in R2', async () => {
  const uniqueSize = 7919;

  const response = await upload('x'.repeat(uniqueSize), { headers: { 'Max-Age-Days': '999' } });

  assert.equal(response.status, 400);
  assert.ok(!(await listR2ObjectSizes()).includes(uniqueSize), 'object left in R2');
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
