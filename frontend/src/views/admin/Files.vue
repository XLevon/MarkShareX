<template>
  <div class="files-page">
    <div class="page-header">
      <h1 class="page-title">📁 资源库</h1>
    </div>

    <!-- Tab 切换 -->
    <div class="resource-tabs">
      <button :class="['tab-btn', { active: activeTab === 'local' }]" @click="activeTab = 'local'">📁 本地资源</button>
      <button :class="['tab-btn', { active: activeTab === 'network' }]" @click="activeTab = 'network'">🌐 网络资源</button>
    </div>

    <!-- 本地资源 -->
    <template v-if="activeTab === 'local'">

    <!-- 存储使用量 -->
    <div class="storage-card">
      <div class="storage-header">
        <span class="storage-title">存储使用量</span>
        <span class="storage-usage">已用 {{ formatSize(storageUsed) }} / 总计 {{ formatSize(storageTotal) }}</span>
      </div>
      <div class="storage-bar">
        <div class="storage-fill" :style="{ width: storagePct + '%' }"></div>
      </div>
      <div class="storage-detail">
        <span>{{ fileCount }} 个文件</span>
        <span>{{ storagePct }}% 已使用</span>
      </div>
    </div>

    <!-- 上传区域 -->
    <div
      class="upload-zone"
      :class="{ dragging: isDragging }"
      @dragenter.prevent="isDragging = true"
      @dragleave.prevent="onDragLeave"
      @dragover.prevent
      @drop.prevent="onDrop"
      @click="triggerUpload"
    >
      <input
        ref="fileInput"
        type="file"
        multiple
        :accept="acceptTypes"
        class="hidden-input"
        @change="onFileChange"
      />
      <div class="upload-content">
        <svg class="upload-icon" width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
          <polyline points="17 8 12 3 7 8"/>
          <line x1="12" y1="3" x2="12" y2="15"/>
        </svg>
        <p class="upload-text">拖拽文件到此处，或<span class="upload-link">点击上传</span></p>
        <p class="upload-hint">支持图片 (jpg/png/gif/webp/svg)、视频、音频、文档、压缩包等</p>
      </div>
    </div>

    <!-- 上传进度 -->
    <div v-if="uploading" class="upload-progress">
      <div class="progress-bar">
        <div class="progress-fill" :style="{ width: uploadProgress + '%' }"></div>
      </div>
      <span class="progress-text">上传中... {{ uploadProgress }}%</span>
    </div>

    <!-- 类型筛选标签 -->
    <div class="filter-tabs">
      <template v-for="tab in fileCategories" :key="tab.key">
      <button
        class="filter-tab"
        :class="{ active: activeFilter === tab.key, warning: tab.warning }"
        @click="switchFilter(tab.key)"
      >
        {{ tab.label }}
        <span class="filter-count" v-if="tab.key !== 'all'">
          {{ activeFilter.value === 'unreferenced' && tab.key === 'unreferenced' ? files.length : getTypeCount(tab.key) }}
        </span>
      </button>
      <span v-if="tab.key === 'unreferenced' && activeFilter === 'unreferenced' && files.length > 0" class="unref-actions">
        <label class="inline-check">
          <input type="checkbox" :checked="allSelected" :indeterminate="someSelected && !allSelected" @change="toggleSelectAll" />
          全选
        </label>
        <button
          v-if="selectedFiles.size > 0"
          class="inline-delete-btn"
          @click="confirmBatchDelete"
        >
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/></svg>
          删除({{ selectedFiles.size }})
        </button>
      </span>
      </template>
    </div>

    <!-- 搜索框 -->
    <div class="file-search">
      <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
      <input
        v-model="searchQuery"
        type="text"
        class="search-input"
        placeholder="搜索文件名..."
      />
      <button v-if="searchQuery" class="search-clear" @click="searchQuery = ''">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
      </button>
    </div>

    <!-- 文件网格 -->
    <div v-if="loading" class="loading-state">
      <div class="spinner"></div>
      <span>加载中...</span>
    </div>

    <div v-else-if="files.length === 0" class="empty-state">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>
      <p>资源库中还没有文件</p>
    </div>

    <div v-else class="file-grid">
      <div
        v-for="file in filteredFiles"
        :key="file.id"
        class="file-card"
        :class="{ selected: selectedFiles.has(file.id) }"
        @click="activeFilter === 'unreferenced' ? toggleFile(file) : previewFile(file)"
      >
        <div class="file-check" v-if="activeFilter === 'unreferenced'">
          <input type="checkbox" :checked="selectedFiles.has(file.id)" @click.stop="toggleFile(file)" />
        </div>
        <div class="file-thumb">
          <template v-if="isImage(file.mime_type) && !imgErrors.has(file.id)">
            <img :src="file.url" :alt="file.filename" loading="lazy" @error="imgErrors.add(file.id)" />
          </template>
          <div v-else class="file-type-icon">
            <svg v-if="isVideo(file.mime_type)" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><polygon points="23 7 16 12 23 17 23 7"/><rect x="1" y="5" width="15" height="14" rx="2"/></svg>
            <svg v-else-if="isAudio(file.mime_type)" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>
            <svg v-else-if="isPdf(file.mime_type)" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
            <svg v-else-if="isArchive(file.mime_type)" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>
            <svg v-else-if="isDocument(file.mime_type)" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>
            <svg v-else width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/><polyline points="13 2 13 9 20 9"/></svg>
          </div>
        </div>
        <div class="file-info">
          <span class="file-name" :title="file.filename">{{ file.filename }}</span>
          <span class="file-size">{{ formatSize(file.size) }}</span>
        </div>
        <div class="file-actions">
          <button class="file-btn copy" @click.stop="copyUrl(file.url)" title="复制链接">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
          </button>
          <button v-if="activeFilter === 'unreferenced'" class="file-btn delete" @click.stop="confirmDelete(file)" title="删除">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/></svg>
          </button>
        </div>
      </div>
    </div>

    <!-- 分页 -->
    <div v-if="activeFilter === 'all' && totalPages > 1" class="pagination">
      <button :disabled="currentPage <= 1" @click="currentPage--; loadFiles()">上一页</button>
      <span>{{ currentPage }} / {{ totalPages }}</span>
      <button :disabled="currentPage >= totalPages" @click="currentPage++; loadFiles()">下一页</button>
    </div>

    <!-- 预览弹窗 -->
    <div v-if="previewFileData" class="modal-overlay" @click.self="closePreview">
      <!-- 关闭按钮：固定在视口右上角，与右侧切换按钮垂直对齐 -->
      <button class="preview-close" @click="closePreview">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
      </button>
      <!-- 左右切换按钮：固定在视口两侧 -->
      <button v-if="isImage(previewFileData.mime_type) && canGoPrev" class="preview-nav preview-prev" @click.stop="goPrev">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>
      </button>
      <button v-if="isImage(previewFileData.mime_type) && canGoNext" class="preview-nav preview-next" @click.stop="goNext">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 18 15 12 9 6"/></svg>
      </button>
      <!-- 位置指示 -->
      <div v-if="isImage(previewFileData.mime_type) && imageFiles.length > 1" class="preview-counter">{{ previewIndex + 1 }} / {{ imageFiles.length }}</div>

      <div class="preview-box">
        <img v-if="isImage(previewFileData.mime_type)" :src="previewFileData.url" :alt="previewFileData.filename" class="preview-image" :style="previewFileData.mime_type === 'image/svg+xml' ? { width: 'min(70vw, 1000px)', height: 'auto' } : { maxWidth: '90vw', maxHeight: '85vh' }" />
        <div v-else class="preview-generic">
          <svg v-if="isVideo(previewFileData.mime_type)" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><polygon points="23 7 16 12 23 17 23 7"/><rect x="1" y="5" width="15" height="14" rx="2"/></svg>
          <svg v-else-if="isAudio(previewFileData.mime_type)" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>
          <svg v-else-if="isArchive(previewFileData.mime_type)" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>
          <svg v-else-if="isDocument(previewFileData.mime_type)" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg>
          <svg v-else width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/><polyline points="13 2 13 9 20 9"/></svg>
          <p>{{ previewFileData.filename }}</p>
          <span>{{ formatSize(previewFileData.size) }}</span>
        </div>
      </div>
    </div>

    <!-- 删除确认 -->
    <div v-if="deleteTarget" class="modal-overlay" @click.self="deleteTarget = null">
      <div class="modal-box">
        <h3>确认删除</h3>
        <p>确定要删除文件「{{ deleteTarget.filename }}」吗？</p>
        <div class="modal-actions">
          <button class="btn-secondary" @click="deleteTarget = null">取消</button>
          <button class="btn-danger" @click="handleDelete">删除</button>
        </div>
      </div>
    </div>

    <!-- 重名冲突弹窗 -->
    <div v-if="conflictState" class="modal-overlay">
      <div class="modal-box">
        <h3>文件名冲突</h3>
        <p>文件「{{ conflictState.originalName }}」已存在。</p>
        <div class="conflict-options">
          <label class="conflict-option">
            <input type="radio" v-model="conflictAction" value="rename" /> 使用建议名称
            <code class="conflict-suggestion">{{ conflictState.suggestion }}</code>
          </label>
          <label class="conflict-option">
            <input type="radio" v-model="conflictAction" value="overwrite" /> 覆盖已有文件
          </label>
          <label class="conflict-option">
            <input type="radio" v-model="conflictAction" value="custom" /> 自定义名称
            <input
              v-if="conflictAction === 'custom'"
              v-model="conflictCustomName"
              type="text"
              class="conflict-custom-input"
              placeholder="输入新文件名..."
            />
          </label>
        </div>
        <div class="modal-actions">
          <button class="btn-secondary" @click="conflictState = null">跳过</button>
          <button class="btn-primary" @click="resolveConflict">确认</button>
        </div>
      </div>
    </div>
    <!-- 批量删除确认 -->
    <div v-if="batchDeleteTarget" class="modal-overlay" @click.self="batchDeleteTarget = false">
      <div class="modal-box">
        <h3>批量删除确认</h3>
        <p>确定要删除选中的 {{ selectedFiles.size }} 个文件吗？此操作不可撤销。</p>
        <div class="modal-actions">
          <button class="btn-secondary" @click="batchDeleteTarget = false">取消</button>
          <button class="btn-danger" @click="handleBatchDelete">删除</button>
        </div>
      </div>
    </div>
    </template>

    <!-- 网络资源 -->
    <div v-if="activeTab === 'network'" class="network-tab-content">
      <NetworkResources />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted, watch } from 'vue'
import { fetchFiles, uploadFile, deleteFile, fetchUnreferencedFiles, batchDeleteFiles } from '@/api/files'
import type { FileInfo } from '@/api/index'
import NetworkResources from '@/views/admin/NetworkResources.vue'

const activeTab = ref<'local' | 'network'>('local')
const loading = ref(false)
const files = ref<FileInfo[]>([])
const fileInput = ref<HTMLInputElement>()
const isDragging = ref(false)
const uploading = ref(false)
const uploadProgress = ref(0)
const currentPage = ref(1)
const totalFiles = ref(0)
const storageUsed = ref(0)
const storageTotal = ref(1024 * 1024 * 1024) // 1GB
const totalPages = ref(1)
const previewFileData = ref<FileInfo | null>(null)
const deleteTarget = ref<FileInfo | null>(null)
// 存储未被引用文件的数量（用于标签计数）
const unreferencedCount = ref(0)
// 批量选择
const selectedFiles = ref(new Set<number>())
const batchDeleteTarget = ref(false)

const allSelected = computed(() => files.value.length > 0 && selectedFiles.value.size === files.value.length)
const someSelected = computed(() => selectedFiles.value.size > 0 && selectedFiles.value.size < files.value.length)

function toggleFile(file: FileInfo) {
  const s = new Set(selectedFiles.value)
  if (s.has(file.id)) s.delete(file.id)
  else s.add(file.id)
  selectedFiles.value = s
}
function toggleSelectAll() {
  if (allSelected.value) {
    selectedFiles.value = new Set()
  } else {
    selectedFiles.value = new Set(files.value.map(f => f.id))
  }
}
function confirmBatchDelete() {
  batchDeleteTarget.value = true
}
async function handleBatchDelete() {
  try {
    await batchDeleteFiles([...selectedFiles.value])
    selectedFiles.value = new Set()
    batchDeleteTarget.value = false
    loadFiles()
  } catch { /* ignore */ }
}

// === 重名冲突 ===
interface ConflictState {
  originalName: string
  suggestion: string
  file: File
  resolve: (action: 'skip' | 'rename' | 'overwrite', customName?: string) => void
}
const conflictState = ref<ConflictState | null>(null)
const conflictAction = ref('rename')
const conflictCustomName = ref('')

function resolveConflict() {
  if (!conflictState.value) return
  const action = conflictAction.value
  const customName = action === 'custom' ? conflictCustomName.value : undefined
  const name = action === 'rename' ? conflictState.value.suggestion : customName
  conflictState.value.resolve(action as any, name)
  conflictState.value = null
}

const acceptTypes = 'image/*,video/*,audio/*,.pdf,.doc,.docx,.xls,.xlsx,.zip,.tar,.gz,.md,.txt'

// 追踪加载失败的图片（DB 有记录但磁盘文件缺失），回退显示图标
const imgErrors = reactive(new Set<number>())

const fileCount = ref(0)

const isImage = (mime: string) => mime?.startsWith('image/')
const isVideo = (mime: string) => mime?.startsWith('video/')
const isAudio = (mime: string) => mime?.startsWith('audio/')
const isPdf = (mime: string) => mime === 'application/pdf'
const isArchive = (mime: string) => {
  if (!mime) return false
  const m = mime.toLowerCase()
  return m.includes('zip') || m.includes('tar') || m.includes('gzip') ||
    m.includes('rar') || m.includes('7z') || m.includes('compress') ||
    m === 'application/x-bzip2' || m === 'application/x-xz'
}
const isDocument = (mime: string) => {
  if (!mime || isPdf(mime) || isArchive(mime)) return false
  return mime.startsWith('text/') ||
    mime.includes('document') || mime.includes('spreadsheet') ||
    mime.includes('presentation') || mime.includes('msword') ||
    mime.includes('openxmlformats')
}

// === 文件类型筛选 ===
const activeFilter = ref('all')
const searchQuery = ref('')
const fileCategories = [
  { key: 'all', label: '全部' },
  { key: 'image', label: '图片' },
  { key: 'video', label: '视频' },
  { key: 'audio', label: '音频' },
  { key: 'document', label: '文档' },
  { key: 'archive', label: '压缩包' },
  { key: 'unreferenced', label: '未被引用', warning: true },
]

function getFileType(mime: string): string {
  if (!mime) return 'other'
  if (mime.startsWith('image/')) return 'image'
  if (mime.startsWith('video/')) return 'video'
  if (mime.startsWith('audio/')) return 'audio'
  if (mime.startsWith('text/') || mime === 'application/pdf' ||
      mime.includes('document') || mime.includes('spreadsheet') ||
      mime.includes('presentation') || mime.includes('msword') ||
      mime.includes('openxmlformats'))
    return 'document'
  if (mime.includes('zip') || mime.includes('tar') || mime.includes('gzip') ||
      mime.includes('rar') || mime.includes('7z') || mime.includes('compress'))
    return 'archive'
  return 'other'
}

// 所有已加载文件的总列表（用于计算各类型数量）
const allFiles = ref<FileInfo[]>([])

const filteredFiles = computed(() => {
  let result = files.value
  // 类型筛选
  if (activeFilter.value !== 'all' && activeFilter.value !== 'unreferenced') {
    result = result.filter(f => getFileType(f.mime_type) === activeFilter.value)
  }
  // 名称搜索
  if (searchQuery.value.trim()) {
    const q = searchQuery.value.trim().toLowerCase()
    result = result.filter(f => f.filename.toLowerCase().includes(q))
  }
  return result
})

function getTypeCount(typeKey: string): number {
  // 特殊处理：未被引用的文件计数
  if (typeKey === 'unreferenced') {
    return unreferencedCount.value
  }
  return allFiles.value.filter(f => getFileType(f.mime_type) === typeKey).length
}

function switchFilter(key: string) {
  activeFilter.value = key
  currentPage.value = 1
  selectedFiles.value = new Set()
  loadFiles()
}

const storagePct = ref(0)

function formatSize(bytes: number) {
  if (!bytes) return '0 B'
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
  return (bytes / (1024 * 1024 * 1024)).toFixed(1) + ' GB'
}

async function loadFiles() {
  loading.value = true
  try {
    // 特殊处理：加载未被引用的文件
    if (activeFilter.value === 'unreferenced') {
      const { data: resp } = await fetchUnreferencedFiles()
      files.value = resp.data
      totalFiles.value = resp.data.length
      totalPages.value = 1
      unreferencedCount.value = resp.data.length
      return
    }

    const isFiltering = activeFilter.value !== 'all'
    const pageSize = isFiltering ? 200 : 24
    const { data: resp } = await fetchFiles({ page: 1, page_size: pageSize })
    files.value = resp.data
    totalFiles.value = resp.pagination.total
    totalPages.value = Math.max(1, Math.ceil(resp.pagination.total / 24))

    // Save all for type counting
    if (isFiltering) {
      allFiles.value = resp.data
    } else {
      // Load all for counting (one-time)
      const allResp = await fetchFiles({ page: 1, page_size: Math.min(resp.pagination.total, 500) })
      allFiles.value = allResp.data.data
      storageUsed.value = allFiles.value.reduce((s: number, f: any) => s + (f.size || 0), 0)
      fileCount.value = allFiles.value.length
      storagePct.value = Math.round((storageUsed.value / storageTotal.value) * 100)
      
      // 异步获取未被引用文件数量（不阻塞主流程）
      fetchUnreferencedFiles().then(({ data }) => {
        unreferencedCount.value = data.data.length
      }).catch(() => {
        unreferencedCount.value = 0
      })
    }
  } catch { /* ignore */ }
  finally {
    loading.value = false
  }
}

function triggerUpload() {
  fileInput.value?.click()
}

function onDragLeave(e: DragEvent) {
  // Only set dragging false when actually leaving the zone
  if ((e.currentTarget as HTMLElement)?.contains(e.relatedTarget as Node)) return
  isDragging.value = false
}

function onDrop(e: DragEvent) {
  isDragging.value = false
  const dt = e.dataTransfer
  if (!dt?.files.length) return
  handleFiles(dt.files)
}

function onFileChange(e: Event) {
  const target = e.target as HTMLInputElement
  if (target.files?.length) handleFiles(target.files)
  target.value = ''
}

async function handleFiles(fileList: FileList) {
  uploading.value = true
  uploadProgress.value = 0
  const files = Array.from(fileList)
  let completed = 0

  for (const file of files) {
    try {
      await uploadWithConflictHandling(file)
      completed++
      uploadProgress.value = Math.round((completed / files.length) * 100)
    } catch (e: any) {
      // Non-conflict errors: just skip
      if (e?.response?.status !== 409) {
        console.error('Upload failed:', e)
      }
    }
  }

  uploading.value = false
  loadFiles()
}

async function uploadWithConflictHandling(file: File): Promise<void> {
  try {
    await uploadFile(file)
  } catch (e: any) {
    if (e?.response?.status === 409) {
      const data = e.response.data
      const suggestion = data?.suggestion || file.name.replace(/(\.\w+)$/, '_1$1')
      return new Promise((resolve, reject) => {
        conflictState.value = {
          originalName: file.name,
          suggestion,
          file,
          resolve: async (action, name) => {
            if (action === 'skip') { resolve(); return }
            try {
              if (action === 'overwrite') {
                await uploadFile(file, { overwrite: true })
              } else if (name) {
                await uploadFile(file, { rename: name })
              }
              resolve()
            } catch (err) {
              reject(err)
            }
          },
        }
        conflictAction.value = 'rename'
        conflictCustomName.value = ''
      })
    }
    throw e
  }
}

function previewFile(file: FileInfo) {
  previewFileData.value = file
}

// ── 图片预览左右切换 ──
const imageFiles = computed(() =>
  files.value.filter(f => isImage(f.mime_type))
)
const previewIndex = computed(() =>
  previewFileData.value ? imageFiles.value.findIndex(f => f.id === previewFileData.value!.id) : -1
)
const canGoPrev = computed(() => previewIndex.value > 0)
const canGoNext = computed(() => previewIndex.value < imageFiles.value.length - 1)

function goPrev() {
  if (canGoPrev.value) previewFileData.value = imageFiles.value[previewIndex.value - 1]
}
function goNext() {
  if (canGoNext.value) previewFileData.value = imageFiles.value[previewIndex.value + 1]
}
function closePreview() {
  previewFileData.value = null
}

// ── 键盘导航：全局监听，打开预览时注册，关闭时移除 ──
watch(previewFileData, (val) => {
  if (val) document.addEventListener('keydown', onPreviewKey)
  else document.removeEventListener('keydown', onPreviewKey)
})
onUnmounted(() => document.removeEventListener('keydown', onPreviewKey))
function onPreviewKey(e: KeyboardEvent) {
  if (e.key === 'ArrowLeft') goPrev()
  else if (e.key === 'ArrowRight') goNext()
  else if (e.key === 'Escape') closePreview()
}

function copyUrl(url: string) {
  if (navigator.clipboard && window.isSecureContext) {
    navigator.clipboard.writeText(url).catch(() => {})
  } else {
    const ta = document.createElement('textarea')
    ta.value = url
    ta.style.position = 'fixed'
    ta.style.opacity = '0'
    document.body.appendChild(ta)
    ta.select()
    document.execCommand('copy')
    document.body.removeChild(ta)
  }
}

function confirmDelete(file: FileInfo) {
  deleteTarget.value = file
}

async function handleDelete() {
  if (!deleteTarget.value) return
  try {
    await deleteFile(deleteTarget.value.id)
    deleteTarget.value = null
    loadFiles()
  } catch { /* ignore */ }
}

onMounted(loadFiles)
</script>

<style scoped>
.files-page {
  animation: fadeIn 0.3s ease;
  min-width: 0;
  max-width: 100%;
}
@keyframes fadeIn {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
}

.page-header {
  margin-bottom: 20px;
}
.page-title {
  font-size: 28px;
  font-weight: 700;
  color: var(--input-color);
}

/* ===== Tabs ===== */
.resource-tabs {
  display: flex;
  gap: 0;
  margin-bottom: 20px;
  border-bottom: 2px solid rgba(255, 255, 255, 0.06);
}
.tab-btn {
  padding: 10px 20px;
  font-size: 14px;
  border: none;
  background: transparent;
  color: var(--color-text-muted, #999);
  cursor: pointer;
  border-bottom: 2px solid transparent;
  margin-bottom: -2px;
  transition: color 0.2s, border-color 0.2s;
}
.tab-btn:hover {
  color: var(--color-text, #333);
}
.tab-btn.active {
  color: var(--color-primary, #4f46e5);
  border-bottom-color: var(--color-primary, #4f46e5);
  font-weight: 600;
}
.network-tab-content {
  animation: fadeIn 0.2s ease;
  min-width: 0;
  max-width: 100%;
}

/* ===== Storage ===== */
.storage-card {
  background: var(--card-bg);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 12px;
  padding: 20px;
  margin-bottom: 20px;
}
.storage-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}
.storage-title { font-size: 14px; font-weight: 600; color: var(--text-primary); }
.storage-usage { font-size: 13px; color: var(--text-dim); }
.storage-bar {
  height: 6px;
  background: var(--storage-bar-bg);
  border-radius: 3px;
  overflow: hidden;
  margin-bottom: 8px;
}
.storage-fill {
  height: 100%;
  background: linear-gradient(90deg, #4f46e5, #818cf8);
  border-radius: 3px;
  transition: width 0.5s ease;
}
.storage-detail {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  color: #4b5563;
}

/* ===== Upload ===== */
.upload-zone {
  border: 2px dashed var(--card-border-color, rgba(255, 255, 255, 0.12));
  border-radius: 14px;
  padding: 40px;
  text-align: center;
  cursor: pointer;
  transition: all 0.2s;
  margin-bottom: 20px;
}
.upload-zone:hover,
.upload-zone.dragging {
  border-color: rgba(79, 70, 229, 0.4);
  background: rgba(79, 70, 229, 0.03);
}
.upload-content {
  display: flex;
  flex-direction: column;
  align-items: center;
}
.upload-icon {
  color: var(--text-dim);
  margin-bottom: 12px;
}
.upload-text {
  font-size: 15px;
  color: var(--text-secondary);
  margin: 0 0 6px;
}
.upload-link { color: #4f46e5; font-weight: 500; }
.upload-hint {
  font-size: 12px;
  color: #4b5563;
  margin: 0;
}
.hidden-input { display: none; }

/* Upload progress */
.upload-progress {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
  padding: 12px 16px;
  background: var(--card-bg);
  border-radius: 10px;
  border: 1px solid rgba(79, 70, 229, 0.2);
}
.progress-bar {
  flex: 1;
  height: 6px;
  background: var(--storage-bar-bg);
  border-radius: 3px;
  overflow: hidden;
}
.progress-fill {
  height: 100%;
  background: #4f46e5;
  border-radius: 3px;
  transition: width 0.3s;
}
.progress-text { font-size: 13px; color: #818cf8; white-space: nowrap; }

/* ===== Filter Tabs ===== */
.filter-tabs {
  display: flex;
  gap: 6px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}
.filter-tab {
  padding: 6px 14px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  background: transparent;
  color: var(--text-dim);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s;
  display: flex;
  align-items: center;
  gap: 5px;
  white-space: nowrap;
}
.filter-tab:hover {
  border-color: rgba(79, 70, 229, 0.25);
  color: var(--text-secondary);
}
.filter-tab.active {
  background: rgba(79, 70, 229, 0.12);
  border-color: #4f46e5;
  color: #818cf8;
  font-weight: 500;
}
.filter-tab.warning {
  color: #f59e0b;
}
.filter-tab.warning.active {
  background: rgba(245, 158, 11, 0.12);
  border-color: #f59e0b;
  color: #fbbf24;
}
.filter-count {
  font-size: 11px;
  color: inherit;
  opacity: 0.7;
}
.filter-tab.active .filter-count {
  opacity: 1;
}

/* ===== Search ===== */
.file-search {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 16px;
  padding: 8px 14px;
  background: var(--card-bg);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 10px;
  transition: border-color 0.2s;
}
.file-search:focus-within {
  border-color: rgba(79, 70, 229, 0.35);
}
.search-icon {
  color: var(--text-dim);
  flex-shrink: 0;
}
.search-input {
  flex: 1;
  border: none;
  background: transparent;
  color: var(--text-primary);
  font-size: 13px;
  outline: none;
}
.search-input::placeholder {
  color: var(--text-dim);
}
.search-clear {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  border: none;
  background: rgba(255, 255, 255, 0.06);
  color: var(--text-dim);
  cursor: pointer;
  flex-shrink: 0;
  transition: background 0.15s;
}
.search-clear:hover {
  background: rgba(239, 68, 68, 0.2);
  color: #f87171;
}

/* ===== Batch actions inline in tab ===== */
.unref-actions {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}
.inline-check {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  font-size: 12px;
  color: inherit;
  cursor: pointer;
  opacity: 0.85;
}
.inline-check input[type="checkbox"] {
  appearance: none;
  -webkit-appearance: none;
  width: 14px;
  height: 14px;
  border: 1.5px solid #ef4444;
  border-radius: 3px;
  background: transparent;
  cursor: pointer;
  margin: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all 0.15s;
}
.inline-check input[type="checkbox"]:checked {
  background: #ef4444;
  border-color: #ef4444;
}
.inline-check input[type="checkbox"]:checked::after {
  content: '';
  display: block;
  width: 4px;
  height: 7px;
  border: solid white;
  border-width: 0 1.5px 1.5px 0;
  transform: rotate(45deg);
  margin-top: -1px;
}
.inline-check input[type="checkbox"]:indeterminate {
  background: #ef4444;
  border-color: #ef4444;
}
.inline-check input[type="checkbox"]:indeterminate::after {
  content: '';
  display: block;
  width: 7px;
  height: 1.5px;
  background: white;
  border-radius: 1px;
}
.inline-delete-btn {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding: 2px 8px;
  border-radius: 5px;
  border: 1px solid rgba(239, 68, 68, 0.25);
  background: rgba(239, 68, 68, 0.08);
  color: #f87171;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}
.inline-delete-btn:hover {
  background: rgba(239, 68, 68, 0.18);
}

/* ===== Loading ===== */
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 60px 0;
  color: var(--text-dim);
  font-size: 14px;
}
.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid rgba(79, 70, 229, 0.15);
  border-top-color: #4f46e5;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding: 80px 0;
  color: #4b5563;
}

/* ===== File Grid ===== */
.file-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 12px;
}
.file-card {
  background: var(--card-bg);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 10px;
  overflow: hidden;
  cursor: pointer;
  transition: all 0.2s;
  position: relative;
}
.file-card:hover {
  border-color: rgba(79, 70, 229, 0.25);
  transform: translateY(-2px);
}
.file-card.selected {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 1px var(--color-primary-light);
}
.file-check {
  position: absolute;
  top: 8px;
  left: 8px;
  z-index: 10;
}
.file-check input[type="checkbox"] {
  appearance: none;
  -webkit-appearance: none;
  width: 18px;
  height: 18px;
  border: 2px solid #ef4444;
  border-radius: 4px;
  background: transparent;
  cursor: pointer;
  margin: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all 0.15s;
}
.file-check input[type="checkbox"]:checked {
  background: #ef4444;
  border-color: #ef4444;
}
.file-check input[type="checkbox"]:checked::after {
  content: '';
  display: block;
  width: 5px;
  height: 9px;
  border: solid white;
  border-width: 0 2px 2px 0;
  transform: rotate(45deg);
  margin-top: -2px;
}
.file-thumb {
  aspect-ratio: 1;
  background: var(--filter-bg);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}
.file-thumb img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.file-type-icon {
  color: var(--text-dim);
}
.file-info {
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.file-name {
  font-size: 12px;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.file-size { font-size: 11px; color: var(--text-dim); }
.file-actions {
  position: absolute;
  top: 8px;
  right: 8px;
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.15s;
}
.file-card:hover .file-actions { opacity: 1; }
.file-btn {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  border: 1px solid var(--card-border-color);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.15s;
}
.file-btn.copy { background: rgba(0, 0, 0, 0.5); color: var(--text-primary); }
.file-btn.copy:hover { background: rgba(79, 70, 229, 0.3); color: #fff; }
.file-btn.delete { background: rgba(0, 0, 0, 0.5); color: var(--text-primary); }
.file-btn.delete:hover { background: rgba(239, 68, 68, 0.3); color: #f87171; }

/* ===== Pagination ===== */
.pagination {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  margin-top: 24px;
}
.pagination button {
  padding: 8px 16px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: var(--card-bg);
  color: var(--text-primary);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s;
}
.pagination button:hover:not(:disabled) {
  border-color: rgba(79, 70, 229, 0.3);
  color: var(--input-color);
}
.pagination button:disabled { opacity: 0.4; cursor: not-allowed; }
.pagination span { font-size: 13px; color: var(--text-dim); }

/* ===== Preview Modal ===== */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 300;
  padding: 20px;
}
.preview-box {
  position: relative;
  max-width: 90vw;
  max-height: 90vh;
  background: var(--modal-bg);
  border-radius: 14px;
  overflow: hidden;
}
.preview-close {
  position: fixed;
  top: 24px;
  right: 24px;
  width: 40px;
  height: 40px;
  border-radius: 50%;
  border: none;
  background: rgba(0, 0, 0, 0.45);
  color: #fff;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 310;
  transition: background 0.15s;
}
.preview-close:hover { background: rgba(239, 68, 68, 0.7); }

/* 左右切换按钮 — 固定在视口两侧，不随图片大小变化 */
.preview-nav {
  position: fixed;
  top: 50%;
  transform: translateY(-50%);
  width: 48px;
  height: 48px;
  border-radius: 50%;
  border: none;
  background: rgba(0, 0, 0, 0.45);
  color: #fff;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 310;
  transition: background 0.15s;
}
.preview-nav:hover { background: rgba(249, 115, 22, 0.55); }
.preview-prev { left: 24px; }
.preview-next { right: 24px; }

/* 位置计数器 — 固定在视口底部中央 */
.preview-counter {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(0, 0, 0, 0.55);
  color: #fff;
  padding: 6px 18px;
  border-radius: 14px;
  font-size: 14px;
  z-index: 310;
}
.preview-image {
  display: block;
  object-fit: contain;
}
.preview-generic {
  padding: 60px 80px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  color: var(--text-secondary);
}
.preview-generic p { font-size: 16px; color: var(--text-primary); margin: 0; }

/* Modal for delete */
.modal-box {
  background: var(--modal-bg);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 14px;
  padding: 28px;
  max-width: 400px;
  width: 90%;
}
.modal-box h3 { margin: 0 0 12px; font-size: 17px; color: var(--input-color); }
.modal-box p { font-size: 14px; color: var(--text-secondary); line-height: 1.6; margin: 0 0 20px; }
.modal-actions { display: flex; justify-content: flex-end; gap: 10px; }
.btn-secondary {
  padding: 8px 18px; border-radius: 8px; border: 1px solid rgba(255,255,255,0.1);
  background: transparent; color: var(--text-primary); font-size: 13px; cursor: pointer;
}
.btn-secondary:hover { background: rgba(255,255,255,0.04); }
.btn-danger {
  padding: 8px 18px; border-radius: 8px; border: none;
  background: #dc2626; color: #fff; font-size: 13px; cursor: pointer;
}
.btn-danger:hover { background: #b91c1c; }

/* Conflict dialog */
.conflict-options {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-bottom: 20px;
}
.conflict-option {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--text-primary);
  cursor: pointer;
  flex-wrap: wrap;
}
.conflict-option input[type="radio"] {
  accent-color: #4f46e5;
  margin: 0;
}
.conflict-suggestion {
  background: rgba(79, 70, 229, 0.12);
  color: #818cf8;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-family: monospace;
}
.conflict-custom-input {
  padding: 4px 10px;
  border: 1px solid rgba(255,255,255,0.1);
  border-radius: 6px;
  background: #121218;
  color: var(--text-primary);
  font-size: 12px;
  outline: none;
  min-width: 180px;
}
.conflict-custom-input:focus {
  border-color: #4f46e5;
}
.btn-primary {
  padding: 8px 18px; border-radius: 8px; border: none;
  background: #4f46e5; color: #fff; font-size: 13px; cursor: pointer;
}
.btn-primary:hover { background: #4338ca; }

@media (max-width: 768px) {
  .file-grid { grid-template-columns: repeat(auto-fill, minmax(140px, 1fr)); }
  .upload-zone { padding: 24px; }
}
</style>
