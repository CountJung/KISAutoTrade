#!/usr/bin/env node

import { execFileSync, spawnSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const appName = 'kis-auto-trade';
const semverPattern = /^\d+\.\d+\.\d+$/;

const options = {
  allowDirty: false,
  dryRun: false,
  noGit: false,
  noPush: false,
  remote: 'origin',
};

const positional = [];
const args = process.argv.slice(2);

for (let index = 0; index < args.length; index += 1) {
  const arg = args[index];

  if (arg === '--allow-dirty') {
    options.allowDirty = true;
  } else if (arg === '--dry-run') {
    options.dryRun = true;
  } else if (arg === '--help' || arg === '-h') {
    printUsage();
    process.exit(0);
  } else if (arg === '--no-git') {
    options.noGit = true;
  } else if (arg === '--no-push') {
    options.noPush = true;
  } else if (arg === '--remote') {
    const remote = args[index + 1];
    if (!remote) {
      fail('--remote requires a remote name.');
    }
    options.remote = remote;
    index += 1;
  } else if (arg.startsWith('--remote=')) {
    options.remote = arg.slice('--remote='.length);
  } else if (arg.startsWith('-')) {
    fail(`Unknown option: ${arg}`);
  } else {
    positional.push(arg);
  }
}

const rawVersion = positional[0];

if (!rawVersion) {
  printUsage();
  fail('Missing version.');
}

const version = rawVersion.startsWith('v') ? rawVersion.slice(1) : rawVersion;
const tagName = `v${version}`;

if (!semverPattern.test(version)) {
  fail(`Version must use x.y.z format so it matches the release workflow tag filter. Received: ${rawVersion}`);
}

process.chdir(rootDir);

const versionFiles = [
  'package.json',
  'package-lock.json',
  'src-tauri/tauri.conf.json',
  'src-tauri/Cargo.toml',
  'Cargo.lock',
  'src-tauri/Cargo.lock',
].filter((filePath) => existsSync(filePath));

if (!options.noGit && !options.dryRun) {
  assertGitClean();
  assertTagAvailable(tagName);
}

const changedFiles = syncVersionFiles(version);

if (options.dryRun) {
  if (changedFiles.length === 0) {
    console.log(`[dry-run] All version files already use ${version}.`);
  } else {
    console.log(`[dry-run] Would update ${changedFiles.join(', ')} to ${version}.`);
  }

  if (!options.noGit) {
    console.log(`[dry-run] Would commit version files with message: chore: release ${tagName}`);
    console.log(`[dry-run] Would create annotated tag: ${tagName}`);
    if (!options.noPush) {
      console.log(`[dry-run] Would push current branch and ${tagName} to ${options.remote}.`);
    }
  }

  process.exit(0);
}

if (options.noGit) {
  console.log(changedFiles.length === 0
    ? `All version files already use ${version}.`
    : `Updated ${changedFiles.join(', ')} to ${version}.`);
  process.exit(0);
}

git(['add', ...versionFiles]);

if (hasStagedChanges()) {
  git(['commit', '-m', `chore: release ${tagName}`]);
} else {
  console.log(`No version file changes to commit; tagging current HEAD as ${tagName}.`);
}

git(['tag', '-a', tagName, '-m', `KISAutoTrade ${tagName}`]);

if (options.noPush) {
  console.log(`Created ${tagName}. Skipped push because --no-push was set.`);
  process.exit(0);
}

const branchName = currentBranchName();
git(['push', options.remote, branchName]);
git(['push', options.remote, tagName]);

console.log(`Released ${tagName}. GitHub Actions will build from the pushed tag.`);

function syncVersionFiles(nextVersion) {
  const changed = [];

  updateTextFile('package.json', (content) => validateJsonAfterReplace(
    replaceFirst(content, /(^  "version": ")[^"]+(")/m, `$1${nextVersion}$2`, 'package.json version'),
    'package.json',
  ), changed);

  updateTextFile('package-lock.json', (content) => {
    let nextContent = replaceFirst(
      content,
      /(^  "version": ")[^"]+(")/m,
      `$1${nextVersion}$2`,
      'package-lock.json top-level version',
    );
    nextContent = replaceFirst(
      nextContent,
      /(^      "version": ")[^"]+(")/m,
      `$1${nextVersion}$2`,
      'package-lock.json root package version',
    );

    return validateJsonAfterReplace(nextContent, 'package-lock.json');
  }, changed);

  updateTextFile('src-tauri/tauri.conf.json', (content) => validateJsonAfterReplace(
    replaceFirst(content, /(^  "version": ")[^"]+(")/m, `$1${nextVersion}$2`, 'src-tauri/tauri.conf.json version'),
    'src-tauri/tauri.conf.json',
  ), changed);

  updateTextFile('src-tauri/Cargo.toml', (content) => replaceFirst(
    content,
    /(\[package\][\s\S]*?^version\s*=\s*")[^"]+(")/m,
    `$1${nextVersion}$2`,
    'src-tauri/Cargo.toml package version',
  ), changed);

  for (const lockFile of ['Cargo.lock', 'src-tauri/Cargo.lock']) {
    updateTextFile(lockFile, (content) => replaceFirst(
      content,
      new RegExp(`(\\[\\[package\\]\\]\\r?\\nname = "${escapeRegExp(appName)}"\\r?\\nversion = ")[^"]+(")`),
      `$1${nextVersion}$2`,
      `${lockFile} ${appName} package version`,
      { optional: true },
    ), changed);
  }

  return changed;
}

function updateTextFile(filePath, updater, changed) {
  if (!existsSync(filePath)) {
    return;
  }

  const before = readFileSync(filePath, 'utf8');
  const after = updater(before);

  if (after !== before) {
    changed.push(filePath);
    if (!options.dryRun) {
      writeFileSync(filePath, after);
    }
  }
}

function validateJsonAfterReplace(content, filePath) {
  try {
    JSON.parse(content);
  } catch (error) {
    fail(`Updated JSON is invalid for ${filePath}: ${error.message}`);
  }

  return content;
}

function replaceFirst(content, pattern, replacement, label, optionsForReplace = {}) {
  if (!pattern.test(content)) {
    if (optionsForReplace.optional) {
      return content;
    }

    fail(`Could not find ${label}.`);
  }

  return content.replace(pattern, replacement);
}

function assertGitClean() {
  if (options.allowDirty) {
    return;
  }

  const status = gitCapture(['status', '--porcelain']);
  if (status.trim().length > 0) {
    fail([
      'Working tree is not clean. Commit or stash existing changes before running a release.',
      'Use --allow-dirty only if you intentionally want to release while unrelated changes remain unstaged.',
      '',
      status.trim(),
    ].join('\n'));
  }
}

function assertTagAvailable(nextTagName) {
  const localTag = spawnSync('git', ['rev-parse', '--verify', `refs/tags/${nextTagName}`], {
    cwd: rootDir,
    encoding: 'utf8',
  });

  if (localTag.status === 0) {
    fail(`Local tag already exists: ${nextTagName}`);
  }

  const remoteTag = spawnSync('git', ['ls-remote', '--exit-code', '--tags', options.remote, `refs/tags/${nextTagName}`], {
    cwd: rootDir,
    encoding: 'utf8',
  });

  if (remoteTag.status === 0) {
    fail(`Remote tag already exists on ${options.remote}: ${nextTagName}`);
  }

  if (remoteTag.status !== 2) {
    fail(`Could not check remote tag on ${options.remote}: ${nextTagName}`);
  }
}

function hasStagedChanges() {
  const diff = spawnSync('git', ['diff', '--cached', '--quiet'], {
    cwd: rootDir,
    encoding: 'utf8',
  });

  return diff.status === 1;
}

function currentBranchName() {
  const branch = gitCapture(['branch', '--show-current']).trim();
  if (!branch) {
    fail('Cannot push branch from a detached HEAD.');
  }

  return branch;
}

function git(argsToRun) {
  execFileSync('git', argsToRun, {
    cwd: rootDir,
    stdio: 'inherit',
  });
}

function gitCapture(argsToRun) {
  return execFileSync('git', argsToRun, {
    cwd: rootDir,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  });
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function printUsage() {
  console.log([
    'Usage:',
    '  npm run release -- 0.2.0',
    '  npm run release:dry -- 0.2.0',
    '  npm run version:set -- 0.2.0',
    '',
    'Options:',
    '  --no-push       Commit and tag locally, but do not push.',
    '  --no-git        Only update version files.',
    '  --dry-run       Print planned changes without writing files or running git changes.',
    '  --remote <name> Push to a remote other than origin.',
    '  --allow-dirty   Allow release when the working tree already has changes.',
  ].join('\n'));
}

function fail(message) {
  console.error(message);
  process.exit(1);
}
