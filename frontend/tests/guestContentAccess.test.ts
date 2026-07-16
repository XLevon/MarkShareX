import test from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'

import { canCopyArticleContent } from '../src/utils/guestContentAccess.ts'

test('article copy access defaults open and only restricts anonymous visitors when explicitly disabled', () => {
  const cases = [
    { authenticated: false, setting: undefined, expected: true },
    { authenticated: false, setting: 'true', expected: true },
    { authenticated: false, setting: 'false', expected: false },
    { authenticated: true, setting: 'false', expected: true },
  ]

  for (const { authenticated, setting, expected } of cases) {
    assert.equal(canCopyArticleContent(authenticated, setting), expected)
  }
})

test('PostDetail delegates every code-copy control to CodeCopyWrapper', () => {
  const source = readFileSync(
    new URL('../src/views/front/PostDetail.vue', import.meta.url),
    'utf8',
  )

  assert.doesNotMatch(source, /function\s+enhanceCodeBlocks\s*\(/)
  assert.doesNotMatch(source, /enhanceCodeBlocks\s*\(\)/)
  assert.doesNotMatch(source, /onclick=.*clipboard\.writeText/)
})
