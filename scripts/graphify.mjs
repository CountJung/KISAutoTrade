#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, extname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const REPOSITORY_ROOT = resolve(SCRIPT_DIR, "..");
const SNAPSHOT_PATH = resolve(REPOSITORY_ROOT, "graphify-out/source-hashes.json");
const VALID_MODES = new Set(["--check", "--update", "--refresh"]);
const SOURCE_PREFIXES = ["scripts/", "src/", "src-tauri/src/", "tests/"];
const SOURCE_EXTENSIONS = new Set([
  ".bash",
  ".cjs",
  ".css",
  ".html",
  ".js",
  ".json",
  ".jsx",
  ".mjs",
  ".rs",
  ".scss",
  ".sh",
  ".ts",
  ".tsx",
]);
const ROOT_CONFIGS = new Set([
  "Cargo.toml",
  "package.json",
  "playwright.config.ts",
  "src-tauri/Cargo.toml",
  "src-tauri/build.rs",
  "src-tauri/tauri.conf.json",
  "tsconfig.json",
  "tsconfig.node.json",
  "vite.config.ts",
]);

function fail(message) {
  console.error(`graphify-check: ${message}`);
  process.exit(1);
}

function parseMode(argv) {
  if (argv.length !== 1 || !VALID_MODES.has(argv[0])) {
    fail("usage: node scripts/graphify.mjs (--check|--update|--refresh)");
  }
  return argv[0];
}

function listRepositoryFiles() {
  const output = execFileSync(
    "git",
    ["ls-files", "-z", "--cached", "--others", "--exclude-standard"],
    {
      cwd: REPOSITORY_ROOT,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "inherit"],
    },
  );

  return [...new Set(output.split("\0").filter(Boolean))]
    .filter((path) => {
      if (ROOT_CONFIGS.has(path)) return true;
      if (!SOURCE_PREFIXES.some((prefix) => path.startsWith(prefix))) return false;
      return SOURCE_EXTENSIONS.has(extname(path).toLowerCase());
    })
    .filter((path) => existsSync(resolve(REPOSITORY_ROOT, path)))
    .sort();
}

function buildSnapshot() {
  return Object.fromEntries(
    listRepositoryFiles().map((path) => {
      const content = readFileSync(resolve(REPOSITORY_ROOT, path));
      return [path, createHash("sha256").update(content).digest("hex")];
    }),
  );
}

function writeSnapshot(snapshot) {
  writeFileSync(SNAPSHOT_PATH, `${JSON.stringify(snapshot, null, 2)}\n`);
}

function runGraphify(force) {
  execFileSync("graphify", ["update", ".", ...(force ? ["--force"] : [])], {
    cwd: REPOSITORY_ROOT,
    stdio: "inherit",
  });
}

const mode = parseMode(process.argv.slice(2));

if (mode === "--update" || mode === "--refresh") {
  const before = buildSnapshot();
  runGraphify(mode === "--refresh");
  const after = buildSnapshot();
  if (JSON.stringify(before) !== JSON.stringify(after)) {
    fail("source files changed while Graphify was running; rerun the command on a stable working tree");
  }
  writeSnapshot(after);
  console.log("graphify-check: source snapshot updated");
} else {
  if (!existsSync(SNAPSHOT_PATH)) {
    fail("source snapshot is missing; run `npm run graphify:refresh`");
  }
  const expected = JSON.parse(readFileSync(SNAPSHOT_PATH, "utf8"));
  const current = buildSnapshot();
  const expectedText = JSON.stringify(expected);
  const currentText = JSON.stringify(current);
  if (expectedText !== currentText) {
    fail("code graph is stale; run `npm run graphify:refresh` and review the generated artifacts");
  }
  console.log("graphify-check: graph source snapshot is up to date");
}
