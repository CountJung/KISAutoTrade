#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const START_MARKER = "<!-- project-map:generated:start -->";
const END_MARKER = "<!-- project-map:generated:end -->";
const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const REPOSITORY_ROOT = resolve(SCRIPT_DIR, "..");
const PROJECT_MAP_PATH = resolve(REPOSITORY_ROOT, "docs/project-map.md");
const VALID_MODES = new Set(["--check", "--write"]);
const RUNTIME_PREFIXES = [".serena/logs/", "graphify-out/"];

function fail(message) {
  console.error(`project-map: ${message}`);
  process.exit(1);
}

function parseMode(argv) {
  if (argv.length !== 1 || !VALID_MODES.has(argv[0])) {
    fail("usage: node scripts/project-map.mjs (--check|--write)");
  }

  return argv[0];
}

function compareNames(left, right) {
  if (left < right) return -1;
  if (left > right) return 1;
  return 0;
}

function listRepositoryFiles() {
  const trackedOutput = execFileSync(
    "git",
    ["ls-files", "-z", "--cached"],
    {
      cwd: REPOSITORY_ROOT,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "inherit"],
    },
  );
  const untrackedOutput = execFileSync(
    "git",
    ["ls-files", "-z", "--others", "--exclude-standard"],
    {
      cwd: REPOSITORY_ROOT,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "inherit"],
    },
  );
  const deletedOutput = execFileSync(
    "git",
    ["ls-files", "-z", "--deleted"],
    {
      cwd: REPOSITORY_ROOT,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "inherit"],
    },
  );
  const deletedFiles = new Set(deletedOutput.split("\0").filter(Boolean));

  const trackedFiles = trackedOutput
    .split("\0")
    .filter(Boolean)
    .filter((path) => !deletedFiles.has(path))
    .filter((path) => !RUNTIME_PREFIXES.some((prefix) => path.startsWith(prefix)));
  const untrackedFiles = untrackedOutput
    .split("\0")
    .filter(Boolean)
    .filter((path) => !RUNTIME_PREFIXES.some((prefix) => path.startsWith(prefix)));

  return [...new Set([...trackedFiles, ...untrackedFiles])]
    .map((path) => path.replaceAll("\\", "/"))
    .sort(compareNames);
}

function buildTree(paths) {
  const root = new Map();

  for (const path of paths) {
    if (path.includes("\n") || path.startsWith("/") || path.split("/").includes("..")) {
      fail(`unsupported repository path: ${JSON.stringify(path)}`);
    }

    let node = root;
    for (const segment of path.split("/")) {
      if (!node.has(segment)) {
        node.set(segment, new Map());
      }
      node = node.get(segment);
    }
  }

  return root;
}

function renderChildren(node, prefix = "") {
  const entries = [...node.entries()].sort(([leftName, leftChildren], [rightName, rightChildren]) => {
    const leftIsDirectory = leftChildren.size > 0;
    const rightIsDirectory = rightChildren.size > 0;
    if (leftIsDirectory !== rightIsDirectory) {
      return leftIsDirectory ? -1 : 1;
    }
    return compareNames(leftName, rightName);
  });

  return entries.flatMap(([name, children], index) => {
    const isLast = index === entries.length - 1;
    const isDirectory = children.size > 0;
    const line = `${prefix}${isLast ? "└── " : "├── "}${name}${isDirectory ? "/" : ""}`;

    if (!isDirectory) {
      return [line];
    }

    const childPrefix = `${prefix}${isLast ? "    " : "│   "}`;
    return [line, ...renderChildren(children, childPrefix)];
  });
}

function generateInventory() {
  const files = listRepositoryFiles();
  const tree = buildTree(files);
  const lines = ["KISAutoTrade/", ...renderChildren(tree)];

  return [
    START_MARKER,
    "<!-- 이 블록은 scripts/project-map.mjs가 생성합니다. 직접 편집하지 마세요. -->",
    "```text",
    ...lines,
    "```",
    END_MARKER,
  ].join("\n");
}

function replaceGeneratedInventory(document, inventory) {
  const start = document.indexOf(START_MARKER);
  const end = document.indexOf(END_MARKER);

  if (start === -1 || end === -1 || end < start) {
    fail(`generated markers are missing or invalid in ${PROJECT_MAP_PATH}`);
  }

  if (document.indexOf(START_MARKER, start + START_MARKER.length) !== -1) {
    fail(`duplicate start marker in ${PROJECT_MAP_PATH}`);
  }
  if (document.indexOf(END_MARKER, end + END_MARKER.length) !== -1) {
    fail(`duplicate end marker in ${PROJECT_MAP_PATH}`);
  }

  return `${document.slice(0, start)}${inventory}${document.slice(end + END_MARKER.length)}`;
}

const mode = parseMode(process.argv.slice(2));
const currentDocument = readFileSync(PROJECT_MAP_PATH, "utf8");
const nextDocument = replaceGeneratedInventory(currentDocument, generateInventory());

if (mode === "--check") {
  if (currentDocument !== nextDocument) {
    fail("docs/project-map.md is stale; run `npm run project-map:update` and review the result");
  }
  console.log("project-map: docs/project-map.md is up to date");
} else {
  writeFileSync(PROJECT_MAP_PATH, nextDocument);
  console.log("project-map: updated docs/project-map.md");
}
