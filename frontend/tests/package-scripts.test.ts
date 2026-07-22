import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import test from 'node:test'

interface PackageManifest {
  scripts?: Record<string, string>
}

const manifest = JSON.parse(
  await readFile(new URL('../package.json', import.meta.url), 'utf8'),
) as PackageManifest
const scripts = manifest.scripts ?? {}

test('production build runs the zero-error TypeScript gate before Vite', () => {
  assert.match(
    scripts.build ?? '',
    /^npm run type-check\s*&&\s*vite build$/,
    'build must stop before Vite when vue-tsc reports any error',
  )
  assert.equal(scripts['type-check'], 'vue-tsc --noEmit')
})

test('the default test gate includes both Node and non-watch Vitest suites', () => {
  assert.match(scripts.test ?? '', /npm run test:node/)
  assert.match(scripts.test ?? '', /npm run test:vitest/)
  assert.equal(scripts['test:vitest'], 'vitest run')
  assert.doesNotMatch(scripts.test ?? '', /watch/)
})
