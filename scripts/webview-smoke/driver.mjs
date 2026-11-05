#!/usr/bin/env node
// Build-only. This driver never starts the generated executable or any app.
import { mkdtemp, mkdir, readFile, writeFile, readdir, lstat, copyFile, chmod, realpath } from 'node:fs/promises'
import { createHash, randomUUID } from 'node:crypto'
import { fileURLToPath } from 'node:url'
import path from 'node:path'
import { spawn } from 'node:child_process'

const repo = fileURLToPath(new URL('../../', import.meta.url))
const source = fileURLToPath(new URL('./', import.meta.url))
const args = process.argv.slice(2)
const reuse = args.filter(arg => arg.startsWith('--reuse-identifier='))
if (reuse.length > 1 || args.some(arg => !['--build', '--prepare'].includes(arg) && !arg.startsWith('--reuse-identifier='))) throw new Error('Only --prepare, --build and --reuse-identifier=org.example.kis-ui-smoke.r<32hex> are supported; there is no launch command')
const reusedIdentity = reuse[0]?.slice('--reuse-identifier='.length)
if (reusedIdentity && !/^org\.example\.kis-ui-smoke\.r[a-f0-9]{32}$/.test(reusedIdentity)) throw new Error('Only a synthetic smoke identifier can be reused')
if (process.platform !== 'darwin') throw new Error('This independent harness targets the current macOS host only')
const root = await realpath(await mkdtemp('/tmp/kis-webview-smoke-8-'))
const dist = await realpath(path.join(repo, 'dist'))
if (dist !== path.join(await realpath(repo), 'dist')) throw new Error('dist must be the real repository dist directory')
const identity = reusedIdentity ?? `org.example.kis-ui-smoke.r${randomUUID().replaceAll('-', '')}`
const files = []
let total = 0
const digest = bytes => createHash('sha256').update(bytes).digest('hex')
await mkdir(path.join(root, 'src'))
await mkdir(path.join(root, 'assets'))
await mkdir(path.join(root, 'output'))
await mkdir(path.join(root, 'rules'))
async function snapshot(directory, prefix = '') {
  for (const entry of (await readdir(directory)).sort()) {
    const absolute = path.join(directory, entry)
    const relative = prefix + entry
    const stat = await lstat(absolute)
    if (stat.isSymbolicLink()) throw new Error('Asset symlinks are not allowed')
    if (stat.isDirectory()) { await snapshot(absolute, `${relative}/`); continue }
    if (!stat.isFile() || stat.size > 10 * 1024 * 1024) throw new Error('Invalid or oversized asset')
    if (files.length >= 256 || (total += stat.size) > 32 * 1024 * 1024) throw new Error('Asset snapshot exceeds bounds')
    const bytes = await readFile(absolute)
    const destination = path.join(root, 'assets', relative)
    await mkdir(path.dirname(destination), { recursive: true })
    await writeFile(destination, bytes, { mode: 0o400 })
    files.push({ path: `/${relative}`, size: bytes.length, sha256: digest(bytes), destination })
  }
}
await snapshot(dist)
for (const file of files) {
  const current = await readFile(path.join(dist, file.path.slice(1)))
  if (digest(current) !== file.sha256) throw new Error('dist changed during snapshot; wait for its build to finish')
}
const index = files.find(file => file.path === '/index.html')
if (!index) throw new Error('dist/index.html is required')
const html = await readFile(index.destination, 'utf8')
const hashes = [...html.matchAll(/<script\b([^>]*)>([\s\S]*?)<\/script>/gi)]
  .filter(match => !/\bsrc\s*=/.test(match[1]) && match[2].trim())
  .map(match => `'sha256-${createHash('sha256').update(match[2]).digest('base64')}'`)
const csp = `default-src 'none'; script-src 'self' kis-smoke: ${hashes.join(' ')}; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self' data:; connect-src 'none'; worker-src 'none'; child-src 'none'; frame-src 'none'; object-src 'none'; media-src 'none'; base-uri 'none'; form-action 'none'`
const mime = name => ({ '.html': 'text/html; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.css': 'text/css; charset=utf-8', '.svg': 'image/svg+xml', '.png': 'image/png', '.ico': 'image/x-icon', '.woff': 'font/woff', '.woff2': 'font/woff2' }[path.extname(name)] ?? 'application/octet-stream')
const generated = [
  `pub const ROOT: &str = ${JSON.stringify(root)};`,
  `pub const IDENTITY: &str = ${JSON.stringify(identity)};`,
  `pub const CSP: &str = ${JSON.stringify(csp)};`,
  'pub static ASSETS: &[(&str, &[u8], &str)] = &[',
  ...files.map(file => `(${JSON.stringify(file.path)}, include_bytes!(${JSON.stringify(file.destination)}), ${JSON.stringify(mime(file.path))}),`),
  '];',
].join('\n')
await writeFile(path.join(root, 'src/generated.rs'), generated)
for (const name of ['host.rs', 'native.rs', 'bootstrap.js', 'fixtures.json']) await copyFile(path.join(source, name), path.join(root, 'src', name === 'host.rs' ? 'main.rs' : name))
await writeFile(path.join(root, 'Cargo.toml'), `[package]
name = "kis-webview-smoke-8"
version = "0.1.0"
edition = "2021"
[workspace]
[dependencies]
wry = { version = "=0.57.1", default-features = false, features = ["os-webview"] }
tao = { version = "=0.37.2", default-features = false, features = ["rwh_06"] }
serde_json = "=1.0.154"
objc2 = "=0.6.5"
block2 = "=0.6.2"
objc2-foundation = { version = "=0.3.2", default-features = false, features = ["std", "NSString", "NSURL", "NSError", "NSData", "NSDictionary"] }
objc2-web-kit = { version = "=0.3.2", default-features = false, features = ["std", "block2", "objc2-app-kit", "WKWebView", "WKWebViewConfiguration", "WKWebsiteDataStore", "WKUserContentController", "WKContentRuleList", "WKContentRuleListStore", "WKSnapshotConfiguration"] }
objc2-app-kit = { version = "=0.3.2", default-features = false, features = ["std", "NSImage", "NSImageRep", "NSBitmapImageRep", "NSGraphics"] }
`)
// Public lockfile only; Cargo prunes unrelated production packages offline.
await copyFile(path.join(repo, 'Cargo.lock'), path.join(root, 'Cargo.lock'))
const packageJson = JSON.parse(await readFile(path.join(repo, 'package.json'), 'utf8'))
const manifest = { identity, reusedIdentifier: Boolean(reusedIdentity), root, sourceDist: dist, stage: 'build-only; not launched', nodeVersion: process.version, versions: packageJson.dependencies, devVersions: packageJson.devDependencies, csp, files: files.map(({ destination, ...file }) => file), sourceHashes: {} }
for (const name of ['driver.mjs', 'host.rs', 'native.rs', 'bootstrap.js', 'fixtures.json']) manifest.sourceHashes[name] = digest(await readFile(path.join(source, name)))
await writeFile(path.join(root, 'snapshot.json'), JSON.stringify(manifest, null, 2))
console.log(JSON.stringify({ root, identity, assetCount: files.length, totalBytes: total, stage: 'prepared; no app launched' }))
async function cargo(arguments_, log) {
  const child = spawn('cargo', arguments_, { cwd: root, env: { ...process.env, CARGO_NET_OFFLINE: 'true', CARGO_TARGET_DIR: path.join(root, 'target') }, stdio: ['ignore', 'pipe', 'pipe'] })
  const chunks = []
  const diagnostics = []
  let bytes = 0
  const timer = setTimeout(() => child.kill('SIGTERM'), 300_000)
  for (const [stream, target] of [[child.stdout, chunks], [child.stderr, diagnostics]]) stream.on('data', chunk => {
    bytes += chunk.length
    if (bytes > 8 * 1024 * 1024) child.kill('SIGTERM')
    else target.push(chunk)
  })
  const code = await new Promise((resolve, reject) => { child.once('error', reject); child.once('exit', resolve) })
  clearTimeout(timer)
  const output = Buffer.concat(chunks).toString()
  await writeFile(path.join(root, log), output)
  await writeFile(path.join(root, `${log}.stderr`), Buffer.concat(diagnostics))
  if (code !== 0) throw new Error(`Cargo ${arguments_[0]} failed (${code}); ${path.join(root, log)}`)
  return output
}
const metadata = JSON.parse(await cargo(['metadata', '--offline', '--format-version', '1'], 'metadata.json'))
const forbidden = /^(tauri($|-)|kis-auto-trade$|sqlx($|-)|keyring$|reqwest$)/
for (const item of metadata.packages) {
  if (forbidden.test(item.name)) throw new Error(`Forbidden production dependency: ${item.name}`)
  if (item.source === null && !item.manifest_path.startsWith(`${root}/`)) throw new Error(`Unexpected path dependency: ${item.manifest_path}`)
}
await writeFile(path.join(root, 'isolation.json'), JSON.stringify({ workspaceRoot: metadata.workspace_root, members: metadata.workspace_members, productionDependencies: [], packageCount: metadata.packages.length, packages: metadata.packages.map(item => ({ name: item.name, version: item.version, source: item.source })) }, null, 2))
if (args.includes('--build')) {
  await cargo(['build', '--offline', '--locked'], 'build.log')
  const app = path.join(root, 'KIS UI Smoke 8.app')
  await mkdir(path.join(app, 'Contents/MacOS'), { recursive: true })
  await copyFile(path.join(root, 'target/debug/kis-webview-smoke-8'), path.join(app, 'Contents/MacOS/kis-webview-smoke-8'))
  await chmod(path.join(app, 'Contents/MacOS/kis-webview-smoke-8'), 0o700)
  await writeFile(path.join(app, 'Contents/Info.plist'), `<?xml version="1.0" encoding="UTF-8"?><!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd"><plist version="1.0"><dict><key>CFBundleIdentifier</key><string>${identity}</string><key>CFBundleName</key><string>KIS UI Smoke 8</string><key>CFBundleDisplayName</key><string>Offline UI Smoke 8</string><key>CFBundleExecutable</key><string>kis-webview-smoke-8</string><key>CFBundlePackageType</key><string>APPL</string><key>CFBundleVersion</key><string>1</string><key>NSHighResolutionCapable</key><true/></dict></plist>`)
  await writeFile(path.join(root, 'build-result.json'), JSON.stringify({
    app, identity, executableSha256: digest(await readFile(path.join(app, 'Contents/MacOS/kis-webview-smoke-8'))),
    infoPlistSha256: digest(await readFile(path.join(app, 'Contents/Info.plist'))),
    snapshotManifestSha256: digest(await readFile(path.join(root, 'snapshot.json'))),
    launched: false,
  }, null, 2))
  console.log(JSON.stringify({ app, output: path.join(root, 'output'), stage: 'built only; root audit and explicit CUA launch remain' }))
}
