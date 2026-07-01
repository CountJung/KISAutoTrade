import fs from 'node:fs'
import path from 'node:path'

const root = process.cwd()
const srcRoot = path.join(root, 'src')
const layers = ['shared', 'entities', 'features', 'widgets', 'pages', 'app']
const layerRank = new Map(layers.map((layer, index) => [layer, index]))
const importPattern = /\b(?:import|export)\s+(?:[^'"]*\s+from\s+)?['"]([^'"]+)['"]/g

function walk(dir) {
  if (!fs.existsSync(dir)) return []
  return fs.readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const fullPath = path.join(dir, entry.name)
    if (entry.isDirectory()) return walk(fullPath)
    return /\.(ts|tsx)$/.test(entry.name) ? [fullPath] : []
  })
}

function layerOf(filePath) {
  const rel = path.relative(srcRoot, filePath).split(path.sep)
  return layerRank.has(rel[0]) ? rel[0] : null
}

function resolveImport(fromFile, specifier) {
  if (specifier.startsWith('@/')) return path.join(srcRoot, specifier.slice(2))
  if (specifier.startsWith('.')) return path.resolve(path.dirname(fromFile), specifier)
  return null
}

const violations = []
for (const file of walk(srcRoot)) {
  const fromLayer = layerOf(file)
  if (!fromLayer) continue
  const source = fs.readFileSync(file, 'utf8')
  for (const match of source.matchAll(importPattern)) {
    const resolved = resolveImport(file, match[1])
    if (!resolved || !resolved.startsWith(srcRoot)) continue
    const toLayer = layerOf(resolved)
    if (!toLayer) continue
    if (layerRank.get(fromLayer) < layerRank.get(toLayer)) {
      violations.push(
        `${path.relative(root, file)} imports ${match[1]} (${fromLayer} -> ${toLayer})`,
      )
    }
  }
}

if (violations.length > 0) {
  console.error('FSD import boundary violations:')
  for (const violation of violations) console.error(`- ${violation}`)
  process.exit(1)
}

console.log('FSD import boundaries OK')

