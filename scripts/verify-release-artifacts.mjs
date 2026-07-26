#!/usr/bin/env node

import { createHash } from 'node:crypto';
import { readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs';
import path from 'node:path';

const args = process.argv.slice(2);
const valueAfter = (flag, fallback) => {
  const index = args.indexOf(flag);
  return index >= 0 ? args[index + 1] : fallback;
};
const artifactDir = path.resolve(valueAfter('--dir', 'release-assets'));
const checksumPath = path.resolve(valueAfter('--output', path.join(artifactDir, 'SHA256SUMS')));
const manifestPath = path.resolve(valueAfter('--manifest', path.join(artifactDir, 'release-artifacts.json')));
const requireSignatures = args.includes('--require-signatures');
const artifactPattern = /\.(dmg|msi|exe|app\.tar\.gz|appimage|deb|rpm)$/i;

const names = readdirSync(artifactDir).sort();
const artifacts = names.filter((name) => artifactPattern.test(name));
const signatures = names.filter((name) => name.endsWith('.sig'));
if (artifacts.length === 0) throw new Error(`No supported release artifacts found in ${artifactDir}`);

const records = artifacts.map((name) => {
  const fullPath = path.join(artifactDir, name);
  const size = statSync(fullPath).size;
  if (size === 0) throw new Error(`Artifact is empty: ${name}`);
  return {
    name,
    size,
    sha256: createHash('sha256').update(readFileSync(fullPath)).digest('hex'),
    signature: signatures.includes(`${name}.sig`) ? `${name}.sig` : null,
  };
});

for (const signature of signatures) {
  const target = signature.slice(0, -4);
  if (!artifacts.includes(target)) throw new Error(`Detached signature has no matching artifact: ${signature}`);
  if (statSync(path.join(artifactDir, signature)).size === 0) throw new Error(`Signature is empty: ${signature}`);
}
if (requireSignatures && records.some((record) => !record.signature)) {
  throw new Error('One or more artifacts are missing detached .sig files');
}

writeFileSync(checksumPath, `${records.map((record) => `${record.sha256}  ${record.name}`).join('\n')}\n`);
writeFileSync(manifestPath, `${JSON.stringify({ schemaVersion: 1, artifacts: records }, null, 2)}\n`);
console.log(`Verified ${records.length} non-empty artifact(s); ${signatures.length} detached signature(s)`);
console.log(`Wrote ${checksumPath} and ${manifestPath}`);