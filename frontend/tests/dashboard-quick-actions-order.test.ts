import test from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'

const dashboardSource = readFileSync(new URL('../src/views/admin/Dashboard.vue', import.meta.url), 'utf8')

test('dashboard quick actions place system settings last', () => {
  const quickActions = dashboardSource.match(/<div class="quick-actions">([\s\S]*?)<\/div>\s*\n\s*<!-- 存储使用量 -->/)?.[1]
  assert.ok(quickActions, 'quick actions block should exist')

  const labels = [...quickActions.matchAll(/<span class="quick-label">([^<]+)<\/span>/g)]
    .map((match) => match[1])

  assert.deepEqual(labels, ['写文章', '管理文章', '分类管理', '标签管理', '资讯管理', '系统设置'])
  assert.equal(labels.at(-1), '系统设置')
  assert.doesNotMatch(quickActions, />基础设置</)
})
