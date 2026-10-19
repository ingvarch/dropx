import assert from 'node:assert/strict';
import { describe, test } from 'bun:test';
import { nextVersion, readVersion, writeVersion } from './release.mjs';

const CARGO_TOML = `[package]
name = "dropx"
version = "0.1.0"
edition = "2021"

[dependencies]
uuid = { version = "1.0", features = ["v4"] }
version = "9.9.9"
`;

describe('readVersion', () => {
  test('reads the version from the package section', () => {
    assert.equal(readVersion(CARGO_TOML), '0.1.0');
  });

  test('ignores a version key in other sections', () => {
    assert.throws(() => readVersion('[dependencies]\nversion = "1.2.3"\n'), /version/);
  });
});

describe('nextVersion', () => {
  test('patch bumps the last part', () => {
    assert.equal(nextVersion('1.2.3', 'patch'), '1.2.4');
  });

  test('minor resets patch', () => {
    assert.equal(nextVersion('1.2.3', 'minor'), '1.3.0');
  });

  test('major resets minor and patch', () => {
    assert.equal(nextVersion('1.2.3', 'major'), '2.0.0');
  });

  test('rejects a version that is not X.Y.Z', () => {
    for (const version of ['1.2', '1.2.3.4', '1.2.3-rc.1', 'v1.2.3', '']) {
      assert.throws(() => nextVersion(version, 'patch'), /X\.Y\.Z/);
    }
  });

  test('rejects an unknown bump', () => {
    assert.throws(() => nextVersion('1.2.3', 'huge'), /bump/);
  });
});

describe('writeVersion', () => {
  test('changes only the package version', () => {
    const updated = writeVersion(CARGO_TOML, '0.2.0');
    assert.equal(updated, CARGO_TOML.replace('version = "0.1.0"', 'version = "0.2.0"'));
    assert.equal(readVersion(updated), '0.2.0');
  });

  test('fails without a package version', () => {
    assert.throws(() => writeVersion('[dependencies]\nversion = "1.2.3"\n', '0.2.0'), /version/);
  });
});
