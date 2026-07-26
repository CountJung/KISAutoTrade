#!/usr/bin/env node

import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const readJson = (file) => JSON.parse(readFileSync(path.join(rootDir, file), 'utf8'));
const fail = (message) => {
  console.error(`Lockfile verification failed: ${message}`);
  process.exitCode = 1;
};

const cargoLockfiles = ['Cargo.lock', 'src-tauri/Cargo.lock'];

for (const file of ['package.json', 'package-lock.json', 'Cargo.toml', ...cargoLockfiles]) {
  if (!existsSync(path.join(rootDir, file))) fail(`missing ${file}`);
}
if (process.exitCode) process.exit();

const pkg = readJson('package.json');
const npmLock = readJson('package-lock.json');
const lockedRoot = npmLock.packages?.[''];

if (npmLock.lockfileVersion !== 3) fail(`package-lock.json lockfileVersion must be 3, found ${npmLock.lockfileVersion}`);
if (npmLock.name !== pkg.name || lockedRoot?.name !== pkg.name) fail('npm package name does not match package-lock.json');
if (npmLock.version !== pkg.version || lockedRoot?.version !== pkg.version) fail('npm package version does not match package-lock.json');
if (!lockedRoot) fail('package-lock.json is missing the root package entry');

for (const section of ['dependencies', 'devDependencies']) {
  const declared = pkg[section] ?? {};
  const locked = lockedRoot?.[section] ?? {};
  const declaredNames = Object.keys(declared).sort();
  const lockedNames = Object.keys(locked).sort();
  if (JSON.stringify(declaredNames) !== JSON.stringify(lockedNames)) {
    fail(`package-lock.json root ${section} names do not match package.json`);
    continue;
  }
  for (const name of declaredNames) {
    if (locked[name] !== declared[name]) {
      fail(`package-lock.json root ${section}.${name} is ${locked[name]}, expected ${declared[name]}`);
    }
  }
}

const escapedName = pkg.name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
for (const lockfile of cargoLockfiles) {
  const cargoLock = readFileSync(path.join(rootDir, lockfile), 'utf8');
  if (!/^version = [34]$/m.test(cargoLock)) fail(`${lockfile} must use Cargo lockfile format 3 or 4`);
  if (!/\[\[package\]\]/.test(cargoLock)) fail(`${lockfile} contains no packages`);
}

const workspaceLock = readFileSync(path.join(rootDir, 'Cargo.lock'), 'utf8');
const appEntry = new RegExp(`\\[\\[package\\]\\]\\r?\\nname = "${escapedName}"\\r?\\nversion = "([^"]+)"`).exec(workspaceLock);
if (!appEntry) fail(`Cargo.lock is missing package ${pkg.name}`);
if (appEntry?.[1] !== pkg.version) {
  fail(`Cargo.lock app version ${appEntry?.[1]} does not match package.json ${pkg.version}`);
}

if (!process.exitCode) {
  console.log(`Lockfiles OK: npm v${npmLock.lockfileVersion}, ${cargoLockfiles.join(' + ')}, app ${pkg.version}`);
}