import { spawnSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { createInterface } from 'node:readline';

const PACKAGE_VERSION = /(\[package\][^[]*?^version\s*=\s*")([^"]*)(")/m;
const SEMVER = /^(\d+)\.(\d+)\.(\d+)$/;
const BUMPS = { p: 'patch', patch: 'patch', m: 'minor', minor: 'minor', M: 'major', major: 'major' };

export function readVersion(cargoToml) {
  const match = PACKAGE_VERSION.exec(cargoToml);
  if (!match) throw new Error('no version in the [package] section of Cargo.toml');
  return match[2];
}

export function nextVersion(version, bump) {
  const match = SEMVER.exec(version);
  if (!match) throw new Error(`version "${version}" is not X.Y.Z`);
  const [major, minor, patch] = match.slice(1).map(Number);
  switch (bump) {
    case 'patch': return `${major}.${minor}.${patch + 1}`;
    case 'minor': return `${major}.${minor + 1}.0`;
    case 'major': return `${major + 1}.0.0`;
    default: throw new Error(`unknown bump "${bump}"`);
  }
}

export function writeVersion(cargoToml, version) {
  readVersion(cargoToml);
  return cargoToml.replace(PACKAGE_VERSION, `$1${version}$3`);
}

function run(command, args) {
  const { status, stdout, stderr } = spawnSync(command, args, { encoding: 'utf8' });
  if (status !== 0) throw new Error(`${command} ${args.join(' ')} failed:\n${stderr || stdout}`);
  return stdout.trim();
}

const git = (...args) => run('git', args);

function checkRepository() {
  if (git('status', '--porcelain')) throw new Error('the working tree has uncommitted changes');
  const branch = git('branch', '--show-current');
  if (branch !== 'main') throw new Error(`the current branch is "${branch}", not main`);
  git('fetch', 'origin');
  if (git('rev-parse', 'HEAD') !== git('rev-parse', 'origin/main')) {
    throw new Error('main differs from origin/main: pull or push first');
  }
}

function checkTagIsFree(tag) {
  if (git('ls-remote', '--tags', 'origin', `refs/tags/${tag}`)) {
    throw new Error(`tag ${tag} already exists on origin`);
  }
  if (spawnSync('git', ['rev-parse', '-q', '--verify', `refs/tags/${tag}`]).status === 0) {
    throw new Error(`tag ${tag} already exists locally`);
  }
}

function updateLockfile() {
  try {
    run('cargo', ['update', '--workspace', '--offline']);
  } catch {
    run('cargo', ['update', '--workspace']);
  }
  const [added, removed] = git('diff', '--numstat', 'Cargo.lock').split('\t');
  if (added !== '1' || removed !== '1') {
    throw new Error('Cargo.lock changed more than the dropx entry; check `git diff Cargo.lock`');
  }
}

async function main() {
  const lines = createInterface({ input: process.stdin })[Symbol.asyncIterator]();
  const ask = async (question) => {
    process.stdout.write(question);
    const { value } = await lines.next();
    return (value ?? '').trim();
  };

  checkRepository();
  const current = readVersion(readFileSync('Cargo.toml', 'utf8'));
  console.log(`Current version: ${current}`);
  const menu = ['patch (p)', 'minor (m)', 'major (M)']
    .map((label) => `${label} -> ${nextVersion(current, label.split(' ')[0])}`)
    .join('\n  ');
  const bump = BUMPS[await ask(`Bump:\n  ${menu}\n> `)];
  if (!bump) throw new Error('no bump chosen, nothing changed');

  const version = nextVersion(current, bump);
  const tag = `v${version}`;
  checkTagIsFree(tag);

  writeFileSync('Cargo.toml', writeVersion(readFileSync('Cargo.toml', 'utf8'), version));
  updateLockfile();
  git('add', 'Cargo.toml', 'Cargo.lock');
  git('commit', '-m', `chore(build): release ${tag}`, '--', 'Cargo.toml', 'Cargo.lock');
  git('tag', '-a', tag, '-m', tag);
  console.log(`Committed and tagged ${tag}.`);

  const pushCommand = `git push --atomic origin main ${tag}`;
  if ((await ask('Push to origin? [y/N] ')).toLowerCase() === 'y') {
    git('push', '--atomic', 'origin', 'main', tag);
    console.log(`Pushed ${tag}.`);
  } else {
    console.log(`Not pushed. To push:\n  ${pushCommand}\nTo undo:\n  git tag -d ${tag}\n  git reset --hard HEAD~1`);
  }
  process.exit(0);
}

if (import.meta.main) {
  main().catch((error) => {
    console.error(error.message);
    process.exit(1);
  });
}
