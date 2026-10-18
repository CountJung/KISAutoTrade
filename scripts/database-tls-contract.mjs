#!/usr/bin/env node
// Disposable localhost-only PostgreSQL 16 contract. Never reads app credentials.
import { spawn } from 'node:child_process'
import { randomUUID } from 'node:crypto'
import { mkdir, writeFile, chmod, rm } from 'node:fs/promises'
import { dirname, resolve, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { createServer } from 'node:net'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const runId = randomUUID()
const directory = join(root, '.cache', 'database-tls-contract', runId)
const label = 'com.kisautotrade.database-tls-fixture'
const image = 'postgres:16'
const containers = []
const results = []
const username = 'kisat_fixture'
const password = `synthetic-${runId}`

// Do not inherit libpq defaults, pgpass files, previous contract endpoints or
// client certificates. Only this child environment trusts this disposable CA.
const environment = { ...process.env }
for (const key of Object.keys(environment)) {
  if (key.startsWith('PG') || key.startsWith('KISAT_PG_')) delete environment[key]
}
delete environment.NO_COLOR
environment.PGPASSFILE = join(directory, 'empty.pgpass')
environment.PGSSLROOTCERT = join(directory, 'root.crt')
environment.KISAT_PG_TLS_FIXTURE = runId

const interruption = new AbortController()
for (const signal of ['SIGINT', 'SIGTERM']) {
  process.on(signal, () => interruption.abort(new Error(`Database fixture interrupted by ${signal}`)))
}

async function terminate(child) {
  if (!child.pid) return
  if (process.platform === 'win32') {
    // taskkill includes the test binary spawned by Cargo; killing Cargo alone
    // leaves inherited stdout pipes and the database test process alive.
    await new Promise(accept => {
      const killer = spawn('taskkill', ['/PID', String(child.pid), '/T', '/F'], { stdio: 'ignore', timeout: 10_000 })
      killer.once('error', accept)
      killer.once('close', accept)
    })
  } else {
    try { process.kill(-child.pid, 'SIGKILL') } catch (error) { if (error.code !== 'ESRCH') throw error }
  }
}

async function command(binary, args, { env = environment, timeout = 60_000, allowFailure = false, stream = false, cleanup = false } = {}) {
  if (!cleanup) interruption.signal.throwIfAborted()
  const child = spawn(binary, args, { cwd: root, env, detached: process.platform !== 'win32', stdio: ['ignore', 'pipe', 'pipe'] })
  let output = ''
  const collect = chunk => {
    const text = chunk.toString()
    output = (output + text).slice(-1_048_576)
    if (stream) process.stdout.write(text)
  }
  child.stdout.on('data', collect)
  child.stderr.on('data', collect)
  return await new Promise((accept, reject) => {
    let settled = false
    const release = () => {
      clearTimeout(timer)
      interruption.signal.removeEventListener('abort', abort)
    }
    const cancel = error => {
      if (settled) return
      settled = true
      release()
      terminate(child).catch(killError => { error.cause = killError }).finally(() => {
        child.stdout.destroy()
        child.stderr.destroy()
        reject(error)
      })
    }
    const abort = () => cancel(interruption.signal.reason)
    const timer = setTimeout(() => cancel(new Error(`${binary} deadline exceeded: ${output}`)), timeout)
    if (!cleanup) interruption.signal.addEventListener('abort', abort, { once: true })
    child.once('error', error => {
      if (settled) return
      settled = true
      release()
      reject(error)
    })
    child.once('close', code => {
      if (settled) return
      settled = true
      release()
      if (code !== 0 && !allowFailure) reject(new Error(`${binary} failed: ${output}`))
      else accept({ code, output })
    })
    if (!cleanup && interruption.signal.aborted) abort()
  })
}

async function certificates() {
  await writeFile(join(directory, 'root.cnf'), `[req]
prompt=no
distinguished_name=dn
x509_extensions=ca
[dn]
CN=KISAutoTrade disposable contract CA
[ca]
basicConstraints=critical,CA:TRUE
keyUsage=critical,keyCertSign,cRLSign
subjectKeyIdentifier=hash
`)
  await writeFile(join(directory, 'server.ext'), `basicConstraints=critical,CA:FALSE
keyUsage=critical,digitalSignature,keyEncipherment
extendedKeyUsage=serverAuth
subjectAltName=DNS:localhost
`)
  for (const ca of ['root', 'untrusted']) {
    await command('openssl', ['req', '-x509', '-newkey', 'rsa:2048', '-nodes', '-days', '2',
      '-config', join(directory, 'root.cnf'), '-keyout', join(directory, `${ca}.key`), '-out', join(directory, `${ca}.crt`)])
  }
  await command('openssl', ['req', '-new', '-newkey', 'rsa:2048', '-nodes', '-subj', '/CN=localhost',
    '-keyout', join(directory, 'server.key'), '-out', join(directory, 'server.csr')])
  await command('openssl', ['x509', '-req', '-days', '2', '-in', join(directory, 'server.csr'),
    '-CA', join(directory, 'root.crt'), '-CAkey', join(directory, 'root.key'),
    '-CAserial', join(directory, 'root.srl'), '-CAcreateserial',
    '-extfile', join(directory, 'server.ext'), '-out', join(directory, 'server.crt')])
  for (const key of ['root.key', 'untrusted.key', 'server.key']) await chmod(join(directory, key), 0o600)
}

async function startServer(kind) {
  const name = `kisat-pg-tls-${runId}-${kind}`
  const reservation = createServer()
  await new Promise((accept, reject) => { reservation.once('error', reject); reservation.listen(0, '127.0.0.1', accept) })
  const requestedPort = reservation.address().port
  await new Promise((accept, reject) => reservation.close(error => error ? reject(error) : accept()))
  const args = ['run', '--detach', '--name', name, '--label', `${label}=${runId}`,
    '--publish', `127.0.0.1:${requestedPort}:5432`, '--env', `POSTGRES_USER=${username}`, '--env', `POSTGRES_PASSWORD=${password}`,
    '--env', 'POSTGRES_DB=postgres', '--env', 'POSTGRES_INITDB_ARGS=--auth-host=scram-sha-256']
  if (kind === 'verified') {
    args.push('--mount', `type=bind,source=${directory},target=/fixture,readonly`, '--entrypoint', 'sh', image, '-c',
      'mkdir -p /tmp/kisat-tls && cp /fixture/server.crt /fixture/server.key /tmp/kisat-tls/ && chmod 0600 /tmp/kisat-tls/server.key && chown -R postgres:postgres /tmp/kisat-tls && exec /usr/local/bin/docker-entrypoint.sh postgres -c ssl=on -c ssl_cert_file=/tmp/kisat-tls/server.crt -c ssl_key_file=/tmp/kisat-tls/server.key')
  } else args.push(image, 'postgres', '-c', 'ssl=off')
  containers.push(name)
  await command('docker', args)
  const port = await command('docker', ['port', name, '5432/tcp'])
  const match = /^127\.0\.0\.1:(\d+)\s*$/.exec(port.output)
  if (!match || Number(match[1]) !== requestedPort) throw new Error(`Fixture is not bound exclusively to the requested loopback port: ${port.output}`)
  const deadline = Date.now() + 30_000
  while (Date.now() < deadline) {
    const ready = await command('docker', ['exec', name, 'pg_isready', '-h', '127.0.0.1', '-U', username, '-d', 'postgres'], { allowFailure: true, timeout: Math.min(5_000, Math.max(1, deadline - Date.now())) })
    if (ready.code === 0) return { name, port: Number(match[1]) }
    await new Promise(accept => setTimeout(accept, 250))
  }
  const logs = await command('docker', ['logs', '--tail', '30', name], { allowFailure: true })
  throw new Error(`PostgreSQL fixture did not become ready: ${logs.output}`)
}

async function tests(filter, env, marker) {
  console.log(`Running ${filter}`)
  const started = Date.now()
  const result = await command('cargo', ['test', '--workspace', '--lib', '--locked', filter, '--', ...(marker ? ['--exact'] : []), '--nocapture'],
    { env, timeout: 900_000, stream: true })
  if (marker && !result.output.includes(marker)) throw new Error(`Contract did not execute: ${marker}`)
  if (!marker && !/test result: ok\. 3 passed; 0 failed/.test(result.output)) throw new Error('The three PostgreSQL roundtrip/authentication/confirmation contracts did not execute')
  results.push({ filter, exitCode: result.code, durationMs: Date.now() - started })
}

await mkdir(directory, { recursive: true, mode: 0o700 })
await writeFile(environment.PGPASSFILE, '', { mode: 0o600 })
let failure
try {
  const context = await command('docker', ['context', 'inspect', '--format', '{{.Endpoints.docker.Host}}'])
  const endpoint = environment.DOCKER_CONTEXT ? context.output.trim() : environment.DOCKER_HOST || context.output.trim()
  if (!/^(unix:\/\/|npipe:\/\/)/.test(endpoint)) throw new Error('Disposable database fixtures require a local Docker socket context')
  await command('docker', ['info', '--format', '{{.ServerVersion}}'])
  const present = await command('docker', ['image', 'inspect', image], { allowFailure: true })
  if (present.code !== 0) await command('docker', ['pull', image], { timeout: 300_000, stream: true })
  await certificates()
  const tls = await startServer('verified')
  const plain = await startServer('plaintext')
  const fixtureEnv = { ...environment, KISAT_PG_HOST: 'localhost', KISAT_PG_PORT: String(tls.port),
    KISAT_PG_USER: username, KISAT_PG_PASSWORD: password, KISAT_PG_TLS_CONTAINER: tls.name,
    KISAT_PG_PLAIN_PORT: String(plain.port) }
  await tests('storage::database_contract_tests::postgresql_', fixtureEnv)
  await tests('storage::database_contract_tests::tls_restart::verified_tls_and_restart_contract', fixtureEnv,
    'TLS_REQUIRE_AND_RESTART_CONTRACT_PASSED')
  await tests('storage::database_contract_tests::tls_restart::rejects_untrusted_ca_contract',
    { ...fixtureEnv, PGSSLROOTCERT: join(directory, 'untrusted.crt'), KISAT_PG_TLS_UNTRUSTED: '1' },
    'TLS_UNTRUSTED_CA_CONTRACT_PASSED')
} catch (error) {
  failure = error
} finally {
  for (const name of containers.reverse()) {
    try {
      const owned = await command('docker', ['inspect', '--format', `{{ index .Config.Labels "${label}" }}`, name], { cleanup: true })
      if (owned.output.trim() !== runId) throw new Error('Refusing cleanup of a container not owned by this fixture')
      await command('docker', ['rm', '--force', '--volumes', name], { cleanup: true })
    } catch (error) { failure ??= error }
  }
  for (const key of ['root.key', 'untrusted.key', 'server.key']) await rm(join(directory, key), { force: true })
  await writeFile(join(directory, 'result.json'), JSON.stringify({ runId, image, tests: results, passed: !failure }, null, 2) + '\n')
}
if (failure) { console.error(failure.message); process.exitCode = 1 }
else console.log(`PostgreSQL TLS/restart contract PASS: ${directory}`)
