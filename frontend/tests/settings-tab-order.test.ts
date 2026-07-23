import test from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'

const settingsSource = readFileSync(new URL('../src/views/admin/Settings.vue', import.meta.url), 'utf8')

test('system settings tabs are ordered as site, IP access, then version maintenance', () => {
  const tabOrder = [...settingsSource.matchAll(/<n-tab-pane\s+name="[^"]+"\s+tab="([^"]+)"/g)]
    .map((match) => match[1])

  assert.deepEqual(tabOrder, ['站点设置', 'IP访问设置', '版本维护'])
})
