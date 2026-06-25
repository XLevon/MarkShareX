<template>
  <div
    class="post-card"
    :class="mode"
    @click="handleClick"
  >
    <div class="card-cover">
      <img v-if="coverSrc" :key="coverSrc" :src="coverSrc" :alt="post.title" referrerpolicy="no-referrer" />
      <div v-else class="cover-placeholder">
        <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
          <polyline points="14 2 14 8 20 8"/>
        </svg>
      </div>
    </div>

    <div class="card-body">
      <!-- Admin: header WITHOUT status badge (moved to bottom) -->
      <div v-if="mode === 'admin'" class="card-header">
        <h3 class="card-title">{{ post.title || '无标题' }}</h3>
      </div>

      <!-- Frontend: title only -->
      <h3 v-else class="card-title" :style="{ color: 'var(--color-text)' }">
        {{ post.title || '无标题' }}
      </h3>

      <!-- Excerpt -->
      <p class="card-excerpt" :style="mode === 'front' ? { color: 'var(--color-text-secondary)' } : {}">
        {{ excerpt }}
      </p>

      <!-- Bottom row: author, date, stats, category, tags -->
      <div class="card-bottom-row">
        <span class="card-meta">{{ authorText }}</span>
        <span class="card-meta-sep">·</span>
        <span class="card-meta">{{ dateText }}</span>

        <!-- Front mode: category & tags on left side -->
        <template v-if="mode === 'front'">
          <span v-if="categoryText" class="card-meta-sep">·</span>
          <span v-if="categoryText" class="card-category">{{ categoryText }}</span>
          <span v-if="post.tags && post.tags.length" class="card-meta-sep">·</span>
          <span v-if="post.tags && post.tags.length" class="card-tags-inline">
            <span v-for="tag in post.tags.slice(0, maxTags)" :key="tag" class="card-tag">{{ tag }}</span>
          </span>
          <!-- Spacer: pushes stats to right -->
          <span class="bottom-spacer"></span>
        </template>

        <!-- Stats: admin mode gets preceding separator, front mode does not -->
        <span v-if="mode === 'admin'" class="card-meta-sep">·</span>
        <span v-if="showViewCount" class="stat-item">
          <svg :width="iconSize" :height="iconSize" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>
          {{ post.view_count || 0 }}
        </span>
        <span v-if="showLikeCount" class="stat-item">
          <svg :width="iconSize" :height="iconSize" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/></svg>
          {{ post.like_count || 0 }}
        </span>
        <span v-if="showCommentCount" class="stat-item">
          <svg :width="iconSize" :height="iconSize" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
          {{ post.comment_count || 0 }}
        </span>

        <!-- Admin mode: category & tags after stats -->
        <template v-if="mode === 'admin'">
          <span v-if="categoryText" class="card-meta-sep">·</span>
          <span v-if="categoryText" class="card-category">{{ categoryText }}</span>
          <span v-if="post.tags && post.tags.length" class="card-meta-sep">·</span>
          <span v-if="post.tags && post.tags.length" class="card-tags-inline">
            <span v-for="tag in post.tags.slice(0, maxTags)" :key="tag" class="card-tag">{{ tag }}</span>
          </span>
        </template>
      </div>
    </div>

    <!-- Admin: hover action buttons -->
    <div v-if="mode === 'admin'" class="card-actions">
      <slot name="actions" :post="post" />
    </div>

    <!-- Admin: status badge — right edge, vertically aligned with bottom row -->
    <span v-if="mode === 'admin'" class="card-status-right" :class="post.status">
      {{ post.status === 'published' ? '已发布' : '草稿' }}
    </span>

    <!-- Edge badges: brush-stroke highlights -->
    <div v-if="hasBadges || post.is_pinned" class="edge-badges">
      <div v-if="post.is_pinned" class="brush-badge brush-pinned">
        <svg class="brush-stroke" viewBox="0 0 90 28" aria-hidden="true">
          <path d="M5,2 C14,1 24,3 36,5 C50,7 64,2 78,2 C84,3 86,7 86,11 C84,21 70,25 56,24 C42,23 28,26 16,25 C8,24 3,20 3,14 C3,7 3,3 5,2 Z" />
        </svg>
        <span class="brush-text">📌 置顶</span>
      </div>
      <div v-if="typeBadge" class="brush-badge" :class="'brush-type-' + typeBadge.key">
        <svg class="brush-stroke" viewBox="0 0 90 28" aria-hidden="true">
          <path d="M5,2 C14,1 24,3 36,5 C50,7 64,2 78,2 C84,3 86,7 86,11 C84,21 70,25 56,24 C42,23 28,26 16,25 C8,24 3,20 3,14 C3,7 3,3 5,2 Z" />
        </svg>
        <span class="brush-text">{{ typeBadge.label }}</span>
      </div>
      <div v-if="statusBadge" class="brush-badge" :class="'brush-status-' + statusBadge.key">
        <svg class="brush-stroke" viewBox="0 0 90 28" aria-hidden="true">
          <path d="M5,2 C14,1 24,3 36,5 C50,7 64,2 78,2 C84,3 86,7 86,11 C84,21 70,25 56,24 C42,23 28,26 16,25 C8,24 3,20 3,14 C3,7 3,3 5,2 Z" />
        </svg>
        <span class="brush-text">{{ statusBadge.label }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import type { Post } from '@/api/index'
import dayjs from 'dayjs'

const props = withDefaults(defineProps<{
  post: Post
  mode: 'front' | 'admin'
  maxTags?: number
}>(), {
  maxTags: 4,
})

const emit = defineEmits<{
  preview: [post: Post]
}>()

const router = useRouter()

const excerpt = computed(() => {
  const p = props.post
  return p.summary || (p.content || '').replace(/[#*`>_\[\]!()\-]/g, '').slice(0, 120) || '暂无摘要'
})

const coverSrc = computed(() => {
  return props.post.cover_image || props.post.category_cover_image || null
})

const iconSize = computed(() => props.mode === 'admin' ? 14 : 13)
const showViewCount = computed(() => props.mode === 'front' || props.post.view_count !== undefined)
const showLikeCount = computed(() => props.mode === 'front' || props.post.like_count !== undefined)
const showCommentCount = computed(() => props.mode === 'front' || props.post.comment_count !== undefined)

const authorText = computed(() => {
  return props.post.author_name || props.post.author || '未知作者'
})

const dateText = computed(() => {
  const date = props.post.published_at || props.post.created_at
  return date ? dayjs(date).format('YYYY-MM-DD') : ''
})

// ── Article type & status badges ──
const typeBadge = computed(() => {
  const t = (props.post as any).article_type_name
  return t ? { key: (props.post as any).article_type || '', label: t } : null
})
const statusBadge = computed(() => {
  const s = (props.post as any).article_status_name
  return s ? { key: (props.post as any).article_status || '', label: s } : null
})
const hasBadges = computed(() => !!(typeBadge.value || statusBadge.value))

const categoryText = computed(() => {
  const cat = props.post as any
  return cat.category_name || cat.category?.name || ''
})

function handleClick() {
  if (props.mode === 'admin') {
    emit('preview', props.post)
  } else {
    router.push(`/post/${props.post.slug}`)
  }
}
</script>

<style scoped>
/* ===== Base card ===== */
.post-card {
  display: flex;
  border: 1px solid;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s;
  position: relative;
}
/* Clip cover image to card border-radius */
.post-card .card-cover {
  border-radius: 12px 0 0 12px;
  overflow: hidden;
}
@media (max-width: 640px) {
  .post-card .card-cover {
    border-radius: 12px 12px 0 0;
  }
}

/* ===== Frontend mode ===== */
.post-card.front {
  flex-direction: row;
  border-color: #4f46e5;
  background: var(--color-bg-card);
  box-shadow: var(--shadow-card);
}
.post-card.front:hover {
  border-color: #f59e0b;
  box-shadow: var(--shadow-card-hover);
  transform: translateX(2px);
}
.post-card.front .card-cover {
  width: 160px;
  height: 130px;
  flex-shrink: 0;
  background: var(--color-bg-secondary);
}
.post-card.front .cover-placeholder {
  color: var(--color-text-muted);
  opacity: 0.4;
}
.post-card.front .card-body {
  padding: 16px 20px;
  gap: 8px;
}
.post-card.front .card-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text);
}
.post-card.front .card-excerpt {
  font-size: 13px;
  line-height: 1.55;
  color: var(--color-text-secondary);
}
.post-card.front .stat-item { color: var(--color-text-muted); }
.post-card.front .card-meta {
  font-size: 12px;
  color: var(--color-text-muted);
  white-space: nowrap;
}

/* ===== Admin mode ===== */
.post-card.admin {
  flex-direction: row;
  border-color: #4f46e5;
  background: var(--card-bg, #16161d);
}
.post-card.admin:hover {
  border-color: #f59e0b;
}
.post-card.admin .card-cover {
  width: 160px;
  height: 130px;
  flex-shrink: 0;
  background: var(--input-bg, #0f0f16);
}
.post-card.admin .cover-placeholder {
  color: var(--text-dim);
  opacity: 0.6;
}
.post-card.admin .card-body {
  padding: 16px 20px;
  gap: 8px;
}
.post-card.admin .card-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary, #e0e0e0);
}
.post-card.admin .card-excerpt {
  font-size: 13px;
  color: var(--text-secondary, #6b7280);
}
.post-card.admin .stat-item { color: var(--text-dim, #4b5563); }
.post-card.admin .card-meta {
  font-size: 12px;
  color: var(--text-dim, #4b5563);
}

/* ===== Brush-stroke highlight badges ===== */
.edge-badges {
  position: absolute;
  right: 0;
  top: 8px;
  display: flex;
  flex-direction: column;
  gap: 3px;
  z-index: 5;
  pointer-events: none;
}

.brush-badge {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 2px 10px;
  transform: translateX(25%) rotate(-6deg);
}

/* Brush stroke SVG — sits behind text, not stretched */
.brush-stroke {
  position: absolute;
  bottom: -2px;
  left: 0;
  width: 100%;
  height: 100%;
  overflow: visible;
  pointer-events: none;
}
.brush-stroke path {
  fill: var(--brush-color);
  opacity: 0.15;
  filter: url(#brush-edge);
}

/* Text on top of brush stroke */
.brush-text {
  position: relative;
  z-index: 1;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.2px;
  white-space: nowrap;
  color: var(--color-text);
  text-shadow: 0 0 4px var(--color-bg);
}

/* ===== Brush colors — type ===== */
.brush-pinned                   { --brush-color: #f59e0b; }
.brush-type-original            { --brush-color: #3b82f6; }
.brush-type-ai_organized        { --brush-color: #a855f7; }
.brush-type-knowledge_summary   { --brush-color: #22c55e; }
.brush-type-reprint_translation { --brush-color: #fb923c; }
.brush-type-opinion_essay       { --brush-color: #ec4899; }

/* ===== Brush colors — status ===== */
.brush-status-latest             { --brush-color: #22c55e; }
.brush-status-partially_outdated { --brush-color: #eab308; }
.brush-status-outdated           { --brush-color: #ef4444; }
.brush-status-continuously_updated { --brush-color: #3b82f6; }
.brush-status-classic_archive    { --brush-color: #8b5cf6; }
.brush-status-experimental       { --brush-color: #06b6d4; }

/* hover: intensify */
.post-card:hover .brush-stroke path {
  opacity: 0.75;
  filter: url(#brush-edge) brightness(1.1);
}

/* ===== Shared sub-elements ===== */
.card-cover {
  display: flex;
  align-items: center;
  justify-content: center;
}
.card-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.cover-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}
.card-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}
.card-header {
  display: flex;
  align-items: center;
  gap: 12px;
}
.card-title {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin: 0;
}
.card-excerpt {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  margin: 0;
}
.card-tags {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}
.card-tag {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 6px;
  background: rgba(129, 140, 248, 0.08);
  color: #818cf8;
  white-space: nowrap;
}
.card-bottom-row {
  margin-top: auto;
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 6px;
  font-size: 12px;
  color: var(--color-text-muted, #6b7280);
}
.bottom-spacer {
  flex: 1;
}
.card-meta-sep {
  color: var(--color-text-muted, #9ca3af);
  opacity: 0.4;
  font-size: 11px;
}
.card-meta {
  white-space: nowrap;
}
.card-category {
  white-space: nowrap;
  color: #f59e0b;
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 4px;
  background: rgba(245, 158, 11, 0.06);
}
.card-tags-inline {
  display: inline-flex;
  gap: 4px;
}
.card-stat-sep {
  color: var(--color-text-muted, #9ca3af);
  opacity: 0.3;
}
.stat-item {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
}

/* ===== Admin extras ===== */
.card-status {
  padding: 1px 8px;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 500;
  flex-shrink: 0;
  line-height: 1.4;
}
.card-status.published { background: rgba(16, 185, 129, 0.12); color: #34d399; }
.card-status.draft { background: rgba(107, 114, 128, 0.12); color: #9ca3af; }

/* Status badge — right side, same baseline as bottom-row text */
.card-status-right {
  position: absolute;
  right: 12px;
  bottom: 16px;
  padding: 1px 8px;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 500;
  line-height: 1.4;
}
.card-status-right.published { background: rgba(16, 185, 129, 0.12); color: #34d399; }
.card-status-right.draft { background: rgba(107, 114, 128, 0.12); color: #9ca3af; }

.card-actions {
  position: absolute;
  right: 72px;
  bottom: 8px;
  display: flex;
  gap: 6px;
  opacity: 0;
  transition: opacity 0.15s;
  z-index: 3;
}
.post-card.admin:hover .card-actions {
  opacity: 1;
}

/* ===== Responsive ===== */
@media (max-width: 640px) {
  .post-card.front,
  .post-card.admin {
    flex-direction: column;
  }
  .post-card.front .card-cover,
  .post-card.admin .card-cover {
    width: 100%;
    height: 180px;
  }
}
</style>
