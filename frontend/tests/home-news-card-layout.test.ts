import test from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'

const homeSource = readFileSync(new URL('../src/views/front/Home.vue', import.meta.url), 'utf8')

test('home news card stacks date above topic in a non-growing metadata column', () => {
  assert.match(
    homeSource,
    /class="news-card-meta [^"]*flex-col[^"]*shrink-0[^"]*"/,
    'news metadata should be a vertical, non-growing column so it consumes only its widest line',
  )

  const metaBlock = homeSource.match(/<div class="news-card-meta [^"]*">([\s\S]*?)<\/div>/)?.[1]
  assert.ok(metaBlock, 'news metadata block should exist')
  assert.ok(
    metaBlock.indexOf('news-card-date') < metaBlock.indexOf('news-card-topic'),
    'date should be rendered before the topic',
  )
})

test('home news card title remains the flexible region beside metadata', () => {
  assert.match(
    homeSource,
    /class="news-card-title flex-1 min-w-0"/,
    'the title region should receive all remaining horizontal space',
  )
})
