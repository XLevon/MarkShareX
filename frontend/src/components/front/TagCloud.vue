<template>
  <div class="tag-cloud">
    <router-link
      v-for="tag in styledTags"
      :key="tag.id"
      :to="`/tag/${tag.slug}`"
      class="tag-cloud-item"
      :style="tag.style"
      @mouseenter="onHover(tag, true)"
      @mouseleave="onHover(tag, false)"
    >
      {{ tag.name }}
    </router-link>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Tag {
  id: number
  name: string
  slug: string
  post_count: number
}

const props = defineProps<{
  tags: Tag[]
}>()

// Max tags to display (prevent overcrowding)
const MAX_TAGS = 25

// Color palette — indigo + accent
const colors = [
  '#818cf8', '#6366f1', '#4f46e5', '#a78bfa',
  '#8b5cf6', '#3b82f6', '#06b6d4', '#10b981',
  '#f59e0b', '#f97316', '#ec4899', '#14b8a6',
]

const styledTags = computed(() => {
  const sorted = [...props.tags]
    .sort((a, b) => (b.post_count || 0) - (a.post_count || 0))
    .slice(0, MAX_TAGS)

  if (sorted.length === 0) return []

  const maxCount = Math.max(...sorted.map(t => t.post_count || 1), 1)
  const minCount = Math.min(...sorted.map(t => t.post_count || 1), maxCount)

  // Shuffle slightly for organic look (but keep big tags bigger)
  return sorted.map((tag, i) => {
    const count = tag.post_count || 1
    // Font size: 0.7rem (min) to 2.2rem (max), logarithmic scaling
    const ratio = maxCount === minCount ? 0.5 : (count - minCount) / (maxCount - minCount)
    const fontSize = 0.7 + ratio * 1.5

    // Font weight: 400 (thin) to 800 (extra bold)
    const fontWeight = ratio > 0.6 ? 700 : ratio > 0.3 ? 600 : 400

    // Opacity: 0.55 (light) to 1.0 (bold)
    const opacity = 0.5 + ratio * 0.5

    // Random rotation for organic feel (-3 to 3 degrees)
    const rotate = Math.sin(i * 1.7) * 2

    // Random padding variation
    const paddingX = 6 + Math.sin(i * 0.7) * 4
    const paddingY = 2 + Math.sin(i * 1.3) * 2

    return {
      ...tag,
      style: {
        color: colors[i % colors.length],
        fontSize: `${fontSize.toFixed(2)}rem`,
        fontWeight,
        opacity: opacity.toFixed(2),
        transform: `rotate(${rotate.toFixed(1)}deg)`,
        padding: `${paddingY.toFixed(0)}px ${paddingX.toFixed(0)}px`,
        lineHeight: 1.4,
      },
    }
  })
})

function onHover(tag: any, enter: boolean) {
  // No programmatic manipulation needed — CSS :hover handles it
}
</script>

<style scoped>
.tag-cloud {
  display: flex;
  flex-wrap: wrap;
  gap: 4px 10px;
  align-items: center;
  justify-content: center;
  padding: 12px 4px 16px;
  line-height: 1.6;
}

.tag-cloud-item {
  display: inline-block;
  cursor: pointer;
  text-decoration: none;
  transition: transform 0.2s ease, opacity 0.2s ease, filter 0.2s ease;
  white-space: nowrap;
  border-radius: 6px;
}

.tag-cloud-item:hover {
  transform: scale(1.15) !important;
  opacity: 1 !important;
  filter: brightness(1.3);
  text-decoration: underline;
  text-underline-offset: 3px;
}
</style>
