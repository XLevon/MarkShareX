<template>
  <ul class="flex flex-col gap-1">
    <li v-for="(link, i) in links" :key="i">
      <a
        :href="link.url"
        target="_blank"
        rel="noopener noreferrer"
        class="flex items-center gap-2 px-2 py-1.5 rounded-lg text-sm transition-colors no-underline hover:bg-[var(--color-bg-hover)]"
        :style="{ color: 'var(--color-text-secondary)' }"
      >
        <span class="w-1.5 h-1.5 rounded-full flex-shrink-0" style="background: #4f46e5"></span>
        {{ link.name }}
      </a>
    </li>
    <li v-if="links.length === 0" class="text-xs" :style="{ color: 'var(--color-text-muted)' }">
      暂无链接
    </li>
  </ul>
</template>

<script setup lang="ts">
import { computed } from 'vue'

export interface FriendLink {
  name: string
  url: string
}

const props = defineProps<{
  data: string
}>()

const links = computed<FriendLink[]>(() => {
  if (!props.data) return []
  try {
    const parsed = JSON.parse(props.data)
    if (Array.isArray(parsed)) return parsed.filter(l => l.name && l.url)
    return []
  } catch {
    return []
  }
})
</script>
