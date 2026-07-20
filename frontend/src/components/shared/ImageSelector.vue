<template>
  <teleport to="body" :disabled="noTeleport">
    <div v-if="visible" class="settings-overlay" @click.self="$emit('close')">
      <div class="picker-modal" :style="{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border)' }">
        <!-- Header -->
        <div class="settings-header" :style="{ borderColor: 'var(--color-border)' }">
          <span :style="{ color: 'var(--color-text)' }">{{ title }}</span>
          <button @click="$emit('close')" class="settings-close" :style="{ color: 'var(--color-text-muted)' }">✕</button>
        </div>

        <!-- Tabs -->
        <div class="picker-tabs" :style="{ borderColor: 'var(--color-border)' }">
          <button
            v-for="tab in tabs"
            :key="tab.key"
            class="picker-tab"
            :class="{ active: activeTab === tab.key }"
            :style="activeTab === tab.key ? { color: 'var(--color-primary)', borderColor: 'var(--color-primary)' } : { color: 'var(--color-text-muted)', borderColor: 'transparent' }"
            @click="switchTab(tab.key)"
          >{{ tab.label }}</button>
        </div>

        <!-- Tab: 资源库 -->
        <div v-if="activeTab === 'library'" class="picker-body">
          <!-- Sub-tabs -->
          <div class="lib-subtabs-row" :style="{ borderColor: 'var(--color-border)' }">
            <div class="lib-subtabs" :style="{ borderColor: 'var(--color-border)' }">
              <button
                class="lib-subtab"
                :class="{ active: libSubTab === 'local' }"
                :style="libSubTab === 'local' ? { color: 'var(--color-primary)', borderColor: 'var(--color-primary)' } : { color: 'var(--color-text-muted)', borderColor: 'transparent' }"
                @click="libSubTab = 'local'"
              >本地资源</button>
              <button
                class="lib-subtab"
                :class="{ active: libSubTab === 'network' }"
                :style="libSubTab === 'network' ? { color: 'var(--color-primary)', borderColor: 'var(--color-primary)' } : { color: 'var(--color-text-muted)', borderColor: 'transparent' }"
                @click="libSubTab = 'network'; loadNetworkResources()"
              >网络资源</button>
            </div>
            <div class="lib-search">
              <svg class="search-icon-sm" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
              <input
                v-model="libSearch"
                type="text"
                :placeholder="libSubTab === 'local' ? '搜索本地资源...' : '搜索网络资源...'"
                class="lib-search-input"
                @input="libSubTab === 'network' ? loadNetworkResources() : null"
              />
            </div>
          </div>
          <div v-if="loading" class="loading-state">加载中...</div>
          <div v-else-if="libSubTab === 'local' && filteredLocalImages.length === 0" class="empty-state">{{ libSearch ? '没有匹配的资源' : '资源库中还没有图片，请先上传' }}</div>
          <div v-else-if="libSubTab === 'network' && filteredNetworkImages.length === 0" class="empty-state">{{ libSearch ? '没有匹配的资源' : '还没有网络资源' }}</div>
          <div v-else class="picker-grid">
            <div
              v-for="item in (libSubTab === 'local' ? filteredLocalImages : filteredNetworkImages)"
              :key="(libSubTab === 'local' ? 'loc-' : 'nr-') + item.id"
              class="picker-item"
              :class="{ selected: (libSubTab === 'local' ? selectedLocalId === item.id : selectedNetworkId === item.id) }"
              @click="selectLibItem(libSubTab === 'local' ? 'local' : 'network', item)"
            >
              <img :src="item.url" :alt="(item as any).label || (item as any).original_name || ''" loading="lazy" :referrerpolicy="libSubTab === 'network' ? 'no-referrer' : undefined" />
              <span class="picker-name">{{ (item as any).label || (item as any).original_name || item.url }}</span>
            </div>
          </div>
        </div>

        <!-- Tab: 上传图片 -->
        <div v-if="activeTab === 'upload'" class="picker-body" style="padding:20px">
          <n-upload
            :max="1"
            accept="image/*"
            :custom-request="(handleUpload as any)"
            :show-file-list="false"
            style="width:100%"
          >
            <button class="upload-btn-inline" :style="{ borderColor: 'var(--color-border)', color: 'var(--color-text-muted)' }">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
              <span>点击上传图片</span>
              <span class="upload-hint">支持 JPG/PNG/WebP/GIF</span>
            </button>
          </n-upload>
          <div v-if="uploadPreview" class="upload-preview" style="margin-top:12px;text-align:center">
            <img :src="uploadPreview" style="max-width:200px;max-height:120px;border-radius:8px;object-fit:cover" />
            <p style="font-size:12px;color:var(--color-text-muted);margin-top:4px">上传成功，点击确认选择</p>
          </div>
        </div>

        <!-- Tab: 网络图片 -->
        <div v-if="activeTab === 'url'" class="picker-body" style="padding:20px">
          <div class="url-input-group">
            <input
              v-model="urlInput"
              type="text"
              placeholder="输入图片 URL（https://...）"
              class="summary-input"
<!-- @ts-ignore -->              style="width:100%"
              @keyup.enter="confirmUrl"
            />
            <div v-if="urlPreview" style="margin-top:12px;text-align:center">
              <img :src="urlPreview" style="max-width:200px;max-height:120px;border-radius:8px;object-fit:cover" referrerpolicy="no-referrer" />
            </div>
          </div>
        </div>

        <!-- Footer -->
        <div class="picker-footer" :style="{ borderColor: 'var(--color-border)' }">
          <button @click="$emit('close')" class="cover-action-btn" :style="{ borderColor: 'var(--color-border)', color: 'var(--color-text-muted)' }">取消</button>
          <button
            @click="confirmSelection"
            :disabled="!canConfirm"
            class="btn-publish"
            :style="{ backgroundColor: canConfirm ? 'var(--color-primary)' : 'rgba(255,255,255,0.08)', padding:'6px 16px', fontSize:'13px', opacity: canConfirm ? 1 : 0.4 }"
          >确认选择</button>
        </div>
      </div>
    </div>
  </teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useMessage } from 'naive-ui'
import { fetchFiles, uploadFile as uploadFileApi } from '@/api/files'
import { fetchNetworkResources } from '@/api/admin'
import type { FileInfo } from '@/api/files'
import type { NetworkResource } from '@/api/admin'

const props = withDefaults(defineProps<{
  visible: boolean
  title?: string
  noTeleport?: boolean
}>(), {
  title: '设置图片',
  noTeleport: false,
})

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'select', value: string): void  // emit: filename (local upload), nr:{id} (network), or full URL
}>()

const message = useMessage()

const tabs = [
  { key: 'library', label: '资源库' },
  { key: 'upload', label: '上传图片' },
  { key: 'url', label: '网络图片' },
] as const
const activeTab = ref<'upload' | 'library' | 'url'>('library')

// ── Upload ──
const uploadPreview = ref('')
async function handleUpload({ file, onFinish, onError }: { file: { file: File }; onFinish: (args: any) => void; onError: () => void }) {
  try {
    const { data: resp } = await uploadFileApi(file.file)
    const filename = resp.data.filename
    uploadPreview.value = `${window.location.origin}/uploads/${filename}`
    onFinish({ url: uploadPreview.value })
  } catch (e: any) {
    onError()
    message.error('上传失败')
  }
}

// ── Library Picker ──
const libSubTab = ref<'local' | 'network'>('local')
const loading = ref(false)
const libSearch = ref('')
const localImages = ref<FileInfo[]>([])
const networkImages = ref<NetworkResource[]>([])
const selectedLocalId = ref<number | null>(null)
const selectedNetworkId = ref<number | null>(null)

const filteredLocalImages = computed(() => {
  if (!libSearch.value.trim()) return localImages.value
  const q = libSearch.value.toLowerCase()
  return localImages.value.filter(f =>
    ((f as any).original_name || f.filename || '').toLowerCase().includes(q)
  )
})

const filteredNetworkImages = computed(() => {
  if (!libSearch.value.trim()) return networkImages.value
  const q = libSearch.value.toLowerCase()
  return networkImages.value.filter(f =>
    (f.label || f.url || '').toLowerCase().includes(q)
  )
})

async function loadLocalImages() {
  loading.value = true
  try {
    const { data: resp } = await fetchFiles({ page_size: 200 })
    const all = resp.data as FileInfo[]
    localImages.value = all.filter(f => f.mime_type?.startsWith('image/'))
  } catch {
    localImages.value = []
  } finally {
    loading.value = false
  }
}

async function loadNetworkResources() {
  loading.value = true
  try {
    const { data: resp } = await fetchNetworkResources({ page_size: 200, source_type: 'image' })
    networkImages.value = (resp.data || []) as NetworkResource[]
  } catch {
    networkImages.value = []
  } finally {
    loading.value = false
  }
}

function selectLibItem(type: 'local' | 'network', item: any) {
  if (type === 'local') {
    selectedLocalId.value = (selectedLocalId.value === item.id) ? null : item.id
    selectedNetworkId.value = null
  } else {
    selectedNetworkId.value = (selectedNetworkId.value === item.id) ? null : item.id
    selectedLocalId.value = null
  }
}

// ── URL Input ──
const urlInput = ref('')
const urlPreview = ref('')

// Watch URL input for live preview
watch(urlInput, (val) => {
  if (val && (val.startsWith('http://') || val.startsWith('https://'))) {
    urlPreview.value = val
  } else {
    urlPreview.value = ''
  }
})

// ── Confirm ──
const canConfirm = computed(() => {
  if (activeTab.value === 'upload') return !!uploadPreview.value
  if (activeTab.value === 'library') {
    return libSubTab.value === 'local' ? selectedLocalId.value !== null : selectedNetworkId.value !== null
  }
  if (activeTab.value === 'url') return urlInput.value.trim().length > 0
  return false
})

async function confirmSelection() {
  if (activeTab.value === 'upload') {
    emit('select', uploadPreview.value)
  } else if (activeTab.value === 'library') {
    if (libSubTab.value === 'local' && selectedLocalId.value) {
      const img = localImages.value.find(i => i.id === selectedLocalId.value)
      if (img) emit('select', `${window.location.origin}/uploads/${img.filename}`)
    } else if (libSubTab.value === 'network' && selectedNetworkId.value) {
      const img = networkImages.value.find(i => i.id === selectedNetworkId.value)
      if (img) emit('select', 'nr:' + img.id)
    }
  } else if (activeTab.value === 'url') {
    const url = urlInput.value.trim()
    if (!url) return
    emit('select', url)
  }
  resetState()
  emit('close')
}

function switchTab(key: 'upload' | 'library' | 'url') {
  activeTab.value = key
  if (key === 'library' && localImages.value.length === 0) {
    loadLocalImages()
  }
  libSearch.value = ''
}

function resetState() {
  uploadPreview.value = ''
  selectedLocalId.value = null
  selectedNetworkId.value = null
  urlInput.value = ''
  urlPreview.value = ''
  activeTab.value = 'library'
  libSubTab.value = 'local'
}

// Reset on open + block Vditor from hijacking paste inside this overlay
let pasteBlocker: ((e: ClipboardEvent) => void) | null = null
watch(() => props.visible, (v) => {
  if (v) {
    resetState()
    loadLocalImages()
    // Vditor intercepts paste at document level. Capture-phase
    // handler runs before Vditor and stops propagation when focus
    // is inside this overlay, so clipboard works in our inputs.
    pasteBlocker = (e: ClipboardEvent) => {
      const overlay = document.querySelector('.settings-overlay')
      if (overlay && document.activeElement && overlay.contains(document.activeElement)) {
        e.stopImmediatePropagation()
      }
    }
    document.addEventListener('paste', pasteBlocker, true)
  } else {
    if (pasteBlocker) {
      document.removeEventListener('paste', pasteBlocker, true)
      pasteBlocker = null
    }
  }
})
</script>

<style scoped>
/* Reuse existing styles from PostEdit.vue */
.settings-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.5);
  z-index: 9999;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 80px;
}
.picker-modal {
  width: 600px;
  max-height: 80vh;
  border: 1px solid;
  border-radius: 16px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid;
  font-size: 15px;
  font-weight: 600;
}
.settings-close {
  border: none;
  background: none;
  cursor: pointer;
  font-size: 18px;
  padding: 4px 8px;
  border-radius: 4px;
  transition: background 0.15s;
}
.settings-close:hover { background: rgba(255,255,255,0.06); }

.picker-tabs {
  display: flex;
  gap: 0;
  border-bottom: 1px solid;
  padding: 0 20px;
}
.picker-tab {
  padding: 10px 18px;
  border: none;
  border-bottom: 2px solid;
  background: none;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
}
.picker-tab:hover { opacity: 0.8; }

.lib-subtabs {
  display: flex;
  gap: 8px;
  padding: 0;
  border: none;
}
.lib-subtabs-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px 0;
  border-bottom: 1px solid;
  margin-bottom: 8px;
  flex-wrap: wrap;
  gap: 8px;
}
.lib-search {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  background: rgba(255,255,255,0.03);
}
.lib-search:focus-within {
  border-color: var(--color-primary);
}
.search-icon-sm {
  color: var(--color-text-muted);
  flex-shrink: 0;
}
.lib-search-input {
  border: none;
  background: none;
  outline: none;
  font-size: 12px;
  color: var(--color-text);
  width: 120px;
}
.lib-search-input::placeholder {
  color: var(--color-text-muted);
}
.lib-subtab {
  padding: 6px 14px;
  border: none;
  border-bottom: 2px solid;
  background: none;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}

.picker-body {
  flex: 1;
  overflow-y: auto;
  min-height: 120px;
  max-height: 420px;
}
.picker-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
  padding: 12px 16px;
}
.picker-item {
  cursor: pointer;
  border-radius: 8px;
  overflow: hidden;
  border: 2px solid transparent;
  transition: all 0.15s;
  background: rgba(255,255,255,0.03);
}
.picker-item:hover { border-color: rgba(255,255,255,0.15); }
.picker-item.selected { border-color: var(--color-primary); }
.picker-item img {
  width: 100%;
  height: 80px;
  object-fit: cover;
}
.picker-name {
  display: block;
  padding: 4px 6px;
  font-size: 11px;
  color: var(--color-text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.picker-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 20px;
  border-top: 1px solid;
}

.cover-action-btn {
  padding: 6px 16px;
  border: 1px solid;
  border-radius: 8px;
  background: none;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s;
}
.cover-action-btn:hover { background: rgba(255,255,255,0.04); }

.btn-publish {
  border: none;
  border-radius: 8px;
  color: #fff;
  font-weight: 500;
  cursor: pointer;
  transition: opacity 0.15s;
}
.btn-publish:hover:not(:disabled) { opacity: 0.85; }
.btn-publish:disabled { cursor: not-allowed; }

.upload-area {
  width: 100%;
  padding: 32px;
  border: 2px dashed;
  border-radius: 12px;
  background: none;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  transition: all 0.15s;
}
.upload-area:hover {
  border-color: var(--color-primary) !important;
  background: rgba(255,255,255,0.02);
}

.upload-btn-inline {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 10px 14px;
  border: 1px solid;
  border-radius: 8px;
  background: none;
  cursor: pointer;
  font-size: 13px;
  transition: all 0.15s;
  width: 100%;
}
.upload-btn-inline:hover {
  border-color: var(--color-primary) !important;
  background: rgba(255,255,255,0.03);
}
.upload-hint {
  font-size: 11px;
  opacity: 0.5;
}

.url-input-group {
  display: flex;
  flex-direction: column;
}
.summary-input {
  padding: 10px 14px;
  border: 1px solid;
  border-radius: 8px;
  font-size: 13px;
  outline: none;
  transition: border-color 0.15s;
}
.summary-input:focus { border-color: var(--color-primary) !important; }

.loading-state, .empty-state {
  padding: 40px;
  text-align: center;
  color: var(--color-text-muted);
  font-size: 13px;
}
</style>
