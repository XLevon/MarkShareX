<template>
  <teleport to="body">
    <div v-if="visible" class="settings-overlay">
      <div class="settings-modal" :style="{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border)' }">
        <div class="settings-header" :style="{ borderColor: 'var(--color-border)' }">
          <span :style="{ color: 'var(--color-text)' }">文章设置</span>
          <button @click="handleCancel" class="settings-close" :style="{ color: 'var(--color-text-muted)' }">✕</button>
        </div>
        <div class="settings-body">
          <!-- 文章类型 -->
          <div class="setting-item">
            <label class="setting-label" :style="{ color: 'var(--color-text-muted)' }">文章类型</label>
            <n-select
              v-model:value="localArticleType"
              :options="articleTypeOptions"
              size="small"
              placeholder="选择创作方式"
            />
          </div>

          <!-- 文章状态 -->
          <div class="setting-item">
            <label class="setting-label" :style="{ color: 'var(--color-text-muted)' }">文章状态</label>
            <n-select
              v-model:value="localArticleStatus"
              :options="articleStatusOptions"
              size="small"
              placeholder="选择内容时效"
            />
          </div>

          <!-- 标签 -->
          <div class="setting-item">
            <label class="setting-label" :style="{ color: 'var(--color-text-muted)' }">标签</label>
            <div v-if="highFreqTags.length" class="freq-tags">
              <span class="freq-label">常用：</span>
              <button
                v-for="t in highFreqTags"
                :key="t.name"
                class="freq-chip"
                :class="{ used: localTags.includes(t.name) }"
                :disabled="localTags.includes(t.name)"
                @click="addTag(t.name)"
              >{{ t.name }}</button>
            </div>
            <n-dynamic-tags :value="localTags.filter(Boolean)" @update:value="(vals: string[]) => localTags = vals" size="small" />
          </div>

          <!-- 摘要 -->
          <div class="setting-item">
            <label class="setting-label" :style="{ color: 'var(--color-text-muted)' }">摘要</label>
            <textarea
              v-model="localSummary"
              placeholder="文章摘要（可选）..."
              rows="3"
              class="summary-input"
              :style="{ backgroundColor: 'var(--color-bg)', borderColor: 'var(--color-border)', color: 'var(--color-text)' }"
            ></textarea>
          </div>

          <!-- 封面图片 -->
          <div class="setting-item">
            <label class="setting-label" :style="{ color: 'var(--color-text-muted)' }">封面图片</label>
            <div v-if="localCoverImage" class="cover-preview">
              <img :src="coverPreviewUrl" class="cover-img" />
              <button @click="localCoverImage = ''" class="cover-remove" :style="{ color: '#fff' }">✕ 移除</button>
            </div>
            <button @click="showImageSelector = true" class="cover-action-btn" :style="{ borderColor: 'var(--color-border)', color: 'var(--color-text-muted)', flex: 1 }">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>
              设置图片
            </button>
          </div>
        </div>
        <div class="settings-footer" :style="{ borderColor: 'var(--color-border)' }">
          <button @click="handleCancel" class="btn-cancel" :style="{ borderColor: 'var(--color-border)', color: 'var(--color-text)' }">取消</button>
          <button @click="handleConfirm" class="btn-confirm" :style="{ backgroundColor: 'var(--color-primary)' }">确定</button>
        </div>
      </div>
    </div>
  </teleport>

  <!-- 统一图片选择器 -->
  <ImageSelector
    :visible="showImageSelector"
    title="设置封面图片"
    :no-teleport="true"
    @close="showImageSelector = false"
    @select="onImageSelected"
  />
</template>

<script setup lang="ts">
import { ref, watch, computed, onMounted } from 'vue'
import { NSelect, NDynamicTags } from 'naive-ui'
import ImageSelector from '@/components/shared/ImageSelector.vue'
import { fetchTags } from '@/api/tags'
import { fetchNetworkResources, fetchArticleTypes, fetchArticleStatuses } from '@/api/admin'
import type { NetworkResource } from '@/api/admin'

const props = defineProps<{
  visible: boolean
  coverImage: string
  summary: string
  tags: string[]
  articleType: string
  articleStatus: string
}>()

const emit = defineEmits<{
  (e: 'update:visible', v: boolean): void
  (e: 'confirm', v: { coverImage: string; summary: string; tags: string[]; articleType: string; articleStatus: string }): void
  (e: 'cancel'): void
}>()

// ── 本地副本（快照）──
const localCoverImage = ref('')
const localSummary = ref('')
const localTags = ref<string[]>([])
const localArticleType = ref('')
const localArticleStatus = ref('')
const snapshot = ref({ coverImage: '', summary: '', tags: [] as string[], articleType: '', articleStatus: '' })

watch(() => props.visible, (v) => {
  if (v) {
    localCoverImage.value = props.coverImage
    localSummary.value = props.summary
    localTags.value = [...props.tags]
    localArticleType.value = props.articleType
    localArticleStatus.value = props.articleStatus
    snapshot.value = {
      coverImage: props.coverImage,
      summary: props.summary,
      tags: [...props.tags],
      articleType: props.articleType,
      articleStatus: props.articleStatus,
    }
  }
})

function handleConfirm() {
  emit('confirm', {
    coverImage: localCoverImage.value,
    summary: localSummary.value,
    tags: localTags.value,
    articleType: localArticleType.value,
    articleStatus: localArticleStatus.value,
  })
}

function handleCancel() {
  emit('cancel')
}

// ── 封面预览 URL ──
const networkUrlCache = ref(new Map<number, string>())
const coverPreviewUrl = computed(() => {
  const img = localCoverImage.value
  if (!img) return ''
  if (img.startsWith('nr:')) {
    return networkUrlCache.value.get(Number(img.slice(3))) || ''
  }
  if (img.startsWith('http://') || img.startsWith('https://')) return img
  if (img.startsWith('/uploads/') || img.startsWith('./uploads/')) return img.replace('./uploads/', '/uploads/')
  return `/uploads/${img}`
})

// ── 图片选择 ──
const showImageSelector = ref(false)

function onImageSelected(value: string) {
  let normalized = value.startsWith('http') && value.includes('/uploads/')
    ? value.replace(/https?:\/\/[^/]+\/uploads\//, '/uploads/')
    : value
  localCoverImage.value = normalized
  if (normalized.startsWith('nr:')) {
    const id = Number(normalized.slice(3))
    if (!networkUrlCache.value.has(id)) {
      fetchNetworkResources({ page_size: 200, source_type: 'image' }).then(({ data: resp }) => {
        (resp.data || []).forEach((nr: NetworkResource) => {
          networkUrlCache.value.set(nr.id, nr.url)
        })
      }).catch(() => {})
    }
  }
  showImageSelector.value = false
}

// ── 标签 ──
function addTag(name: string) {
  if (!localTags.value.includes(name)) {
    localTags.value = [...localTags.value, name]
  }
}

// ── 高频标签 ──
const highFreqTags = ref<{ name: string; post_count: number }[]>([])

// ── 类型/状态选项 ──
const rawTypes = ref<{ code: string; display_name: string }[]>([])
const rawStatuses = ref<{ code: string; display_name: string }[]>([])
const articleTypeOptions = computed(() => rawTypes.value.map(t => ({ label: t.display_name, value: t.code })))
const articleStatusOptions = computed(() => rawStatuses.value.map(s => ({ label: s.display_name, value: s.code })))

onMounted(async () => {
  try {
    const tagResp = await fetchTags()
    // API 返回 { data: { data: Tag[] } }（axios.data = ApiResponse { data: [...] }）
    const raw = tagResp?.data
    const tags: any[] = Array.isArray(raw) ? raw : (raw?.data ?? [])
    highFreqTags.value = tags
      .filter((t: any) => (t.post_count || 0) > 0)
      .sort((a: any, b: any) => (b.post_count || 0) - (a.post_count || 0))
      .slice(0, 12)
  } catch (e) {
    console.error('PostSettingsModal: fetchTags failed', e)
  }
  fetchArticleTypes().then(res => {
    rawTypes.value = ((res as any).data?.data ?? []) as any
  }).catch(() => {})
  fetchArticleStatuses().then(res => {
    rawStatuses.value = ((res as any).data?.data ?? []) as any
  }).catch(() => {})
})
</script>

<style scoped>
.settings-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.5);
  z-index: 999;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 80px;
}
.settings-modal {
  width: 420px;
  max-height: 80vh;
  border: 1px solid;
  border-radius: 14px;
  overflow: hidden;
  box-shadow: 0 20px 60px rgba(0,0,0,0.4);
}
.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  border-bottom: 1px solid;
  font-weight: 600;
  font-size: 15px;
}
.settings-close {
  border: none;
  background: none;
  font-size: 18px;
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 6px;
}
.settings-close:hover {
  background: rgba(255,255,255,0.08);
}
.settings-body {
  padding: 18px;
  display: flex;
  flex-direction: column;
  gap: 18px;
  max-height: 60vh;
  overflow-y: auto;
}
.settings-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 10px;
  padding: 12px 18px;
  border-top: 1px solid;
}
.btn-cancel,
.btn-confirm {
  padding: 6px 20px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: opacity 0.15s;
}
.btn-cancel {
  background: transparent;
  border: 1px solid;
}
.btn-confirm {
  border: none;
  color: #fff;
}
.btn-cancel:hover,
.btn-confirm:hover {
  opacity: 0.85;
}
.setting-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.setting-label {
  font-size: 12px;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.cover-preview {
  position: relative;
  display: inline-block;
}
.cover-img {
  width: 100%;
  max-height: 160px;
  object-fit: cover;
  border-radius: 8px;
}
.cover-remove {
  position: absolute;
  top: 6px;
  right: 6px;
  border: none;
  background: rgba(0,0,0,0.7);
  font-size: 11px;
  padding: 3px 8px;
  border-radius: 6px;
  cursor: pointer;
}
.cover-action-btn {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 8px 12px;
  border: 1px dashed;
  border-radius: 8px;
  cursor: pointer;
  font-size: 12px;
  background: none;
  transition: all 0.15s;
  justify-content: center;
  width: 100%;
}
.cover-action-btn:hover {
  border-color: #4f46e5 !important;
  color: #4f46e5 !important;
}
.freq-tags {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
  margin-top: 8px;
}
.freq-label {
  font-size: 11px;
  color: var(--color-text-muted);
  flex-shrink: 0;
}
.freq-chip {
  padding: 2px 10px;
  border-radius: 12px;
  border: 1px solid var(--color-border);
  background: transparent;
  color: var(--color-text);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}
.freq-chip:hover:not(:disabled) {
  border-color: #4f46e5;
  color: #4f46e5;
}
.freq-chip.used,
.freq-chip:disabled {
  opacity: 0.35;
  cursor: not-allowed;
  border-color: transparent;
}
.summary-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid;
  border-radius: 8px;
  font-size: 13px;
  resize: vertical;
}
</style>
