import test from 'node:test'
import assert from 'node:assert/strict'
import { displaySiteTitle } from '../src/composables/useTitleParts.ts'

test('admin browser title follows the front-end site title display rule', () => {
  assert.equal(displaySiteTitle('Mark-Share-X_用AI学AI'), 'MarkShareX_用AI学AI')
  assert.equal(displaySiteTitle('Mark---Share-X'), 'MarkShareX')
})

test('admin browser title has the same fallback as the site title', () => {
  assert.equal(displaySiteTitle(''), 'MarkShareX')
  assert.equal(displaySiteTitle('----'), 'MarkShareX')
})
