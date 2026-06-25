<template>
  <div class="max-w-3xl mx-auto px-4 py-12">
    <div class="flex items-center justify-between mb-8">
      <div>
        <h1 class="text-3xl font-bold mb-1" :style="{ color: 'var(--color-text)' }">📋 版本更新说明</h1>
        <p :style="{ color: 'var(--color-text-muted)' }">MarkShareX 的版本迭代记录</p>
      </div>
      <button class="back-btn" @click="$router.back()">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="15 18 9 12 15 6"/></svg>
        返回
      </button>
    </div>

    <div v-if="loading" class="text-center py-12" :style="{ color: 'var(--color-text-muted)' }">
      <div class="w-8 h-8 rounded-full animate-spin mx-auto mb-4" style="border: 3px solid var(--color-border); border-top-color: #4f46e5;"></div>
      加载中...
    </div>

    <div v-else-if="error" class="text-center py-12" :style="{ color: 'var(--color-text-muted)' }">
      {{ error }}
    </div>

    <div v-else-if="changelogs.length === 0" class="text-center py-12" :style="{ color: 'var(--color-text-muted)' }">
      暂无版本记录
    </div>

    <div v-else class="space-y-8">
      <div
        v-for="entry in changelogs"
        :key="entry.id"
        class="p-6 rounded-xl transition-theme border"
        :style="{ background: 'var(--color-card-bg)', borderColor: 'var(--color-border)' }"
      >
        <div class="flex items-center gap-3 mb-4">
          <span
            class="inline-flex items-center px-3 py-1 rounded-full text-sm font-semibold text-white"
            style="background: #4f46e5;"
          >
            {{ entry.version }}
          </span>
          <span class="text-sm" :style="{ color: 'var(--color-text-muted)' }">
            {{ formatDate(entry.created_at) }}
          </span>
        </div>
        <div
          class="prose max-w-none"
          :style="{ color: 'var(--color-text)' }"
          v-html="renderMarkdown(entry.content)"
        ></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { fetchPublicChangelogs, type ChangelogEntry } from '@/api/changelog'

const changelogs = ref<ChangelogEntry[]>([])
const loading = ref(true)
const error = ref('')

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  })
}

function renderMarkdown(md: string) {
  return md
    .replace(/### (.+)/g, '<h3 style="font-size:1.1em;font-weight:600;margin:1em 0 0.5em">$1</h3>')
    .replace(/## (.+)/g, '<h2 style="font-size:1.2em;font-weight:700;margin:1.2em 0 0.5em">$1</h2>')
    .replace(/# (.+)/g, '<h1 style="font-size:1.4em;font-weight:700;margin:1.4em 0 0.5em">$1</h1>')
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    .replace(/\*(.+?)\*/g, '<em>$1</em>')
    .replace(/`(.+?)`/g, '<code style="background:var(--color-border);padding:0.15em 0.4em;border-radius:3px;font-size:0.9em">$1</code>')
    .replace(/^- (.+)/gm, '<li style="margin-left:1.5em">$1</li>')
    .replace(/\n/g, '<br/>')
}

onMounted(async () => {
  try {
    const { data } = await fetchPublicChangelogs()
    changelogs.value = data.data || []
  } catch {
    error.value = '加载失败'
  } finally {
    loading.value = false
  }
})
</script>

<style scoped>
.back-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 20px;
  border-radius: 10px;
  border: 1px solid var(--color-border);
  background: var(--color-bg-card);
  color: var(--color-text-secondary);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}
.back-btn:hover {
  background: var(--color-border);
}
</style>
