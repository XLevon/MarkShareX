<template>
  <div class="import-page">
    <div class="page-header">
      <h1 class="page-title">📦 批导</h1>
      <p class="page-desc">拖拽或选择文件、文件夹或ZIP包，批量导入/导出 Markdown 文档和相关图片</p>
    </div>

    <!-- 操作选项卡 -->
    <div class="tabs">
      <button 
        class="tab-btn" 
        :class="{ active: activeTab === 'import' }"
        @click="activeTab = 'import'"
      >
        导入
      </button>
      <button 
        class="tab-btn" 
        :class="{ active: activeTab === 'export' }"
        @click="activeTab = 'export'"
      >
        导出
      </button>
    </div>

    <!-- 导入面板 -->
    <div v-if="activeTab === 'import'" class="import-panel">
      <!-- 导入方式选择 -->
      <div class="import-modes">
        <button 
          class="mode-btn" 
          :class="{ active: importMode === 'files' }"
          @click="importMode = 'files'"
        >
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="17 8 12 3 7 8"/>
            <line x1="12" y1="3" x2="12" y2="15"/>
          </svg>
          <span>文件/文件夹</span>
        </button>
        <button 
          class="mode-btn" 
          :class="{ active: importMode === 'zip' }"
          @click="importMode = 'zip'"
        >
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="14" y="2" width="10" height="20" rx="2" ry="2"/>
            <path d="M4 16v2a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2v-2"/>
            <path d="M14 2v4m0 4v10"/>
            <path d="M4 16H2v-2a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v2h-2"/>
            <path d="M6 10h12"/>
            <path d="M6 6h12"/>
          </svg>
          <span>ZIP 压缩包</span>
        </button>
      </div>

      <!-- 文件上传区域 -->
      <div
        class="upload-zone"
        :class="{ dragging: isDragging, 'has-files': pendingItems.length > 0 }"
        @dragenter.prevent="isDragging = true"
        @dragleave.prevent="onDragLeave"
        @dragover.prevent
        @drop.prevent="onDrop"
        @click.stop
      >
        <input
          ref="fileInput"
          type="file"
          multiple
          :accept="importMode === 'zip' ? '.zip' : '.md,.markdown,.txt,image/*'"
          class="hidden-input"
          @change="onFileChange"
        />
        <input
          v-if="importMode === 'files'"
          ref="folderInput"
          type="file"
          webkitdirectory
          class="hidden-input"
          @change="onFileChange"
        />
        <div v-if="pendingItems.length === 0" class="upload-prompt">
          <svg v-if="importMode === 'files'" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="17 8 12 3 7 8"/>
            <line x1="12" y1="3" x2="12" y2="15"/>
          </svg>
          <svg v-else width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="14" y="2" width="10" height="20" rx="2" ry="2"/>
            <path d="M4 16v2a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2v-2"/>
            <path d="M14 2v4m0 4v10"/>
            <path d="M4 16H2v-2a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v2h-2"/>
            <path d="M6 10h12"/>
            <path d="M6 6h12"/>
          </svg>
          <p class="upload-title">
            <template v-if="importMode === 'zip'">
              拖拽 ZIP 压缩包到此处，或<span class="upload-link" @click.stop="triggerUpload">点击上传</span>
            </template>
            <template v-else>
              拖拽文件或文件夹到此处，或点击<span class="upload-link" @click.stop="triggerUpload">上传文件</span>或<span class="upload-link" @click.stop="triggerFolderUpload">文件夹</span>
            </template>
          </p>
          <p class="upload-sub">
            {{ importMode === 'zip' ? '支持 .zip 格式，可包含多个 MD 文件和图片' : '支持 .md / .markdown / 图片文件' }}
          </p>
        </div>
        <div v-else class="file-list">
          <div v-for="(item, idx) in pendingItems" :key="idx" class="file-row">
            <div class="file-icon" :class="{ 'zip-icon': item.type === 'zip' }">
              <svg v-if="item.type === 'md'" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                <polyline points="14 2 14 8 20 8"/>
                <line x1="16" y1="13" x2="8" y2="13"/>
                <line x1="16" y1="17" x2="8" y2="17"/>
              </svg>
              <svg v-else-if="item.type === 'zip'" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="14" y="2" width="10" height="20" rx="2" ry="2"/>
                <path d="M4 16v2a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2v-2"/>
                <path d="M14 2v4m0 4v10"/>
                <path d="M4 16H2v-2a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v2h-2"/>
              </svg>
              <svg v-else width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="3" y="3" width="18" height="18" rx="2"/>
                <circle cx="8.5" cy="8.5" r="1.5"/>
                <polyline points="21 15 16 10 5 21"/>
              </svg>
            </div>
            <div class="file-detail">
              <span class="file-name">{{ item.name }}</span>
              <span class="file-size">{{ formatSize(item.size) }}</span>
            </div>
            <button class="file-remove" @click.stop="removeItem(idx)">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="18" y1="6" x2="6" y2="18"/>
                <line x1="6" y1="6" x2="18" y2="18"/>
              </svg>
            </button>
          </div>
        </div>
      </div>

      <!-- 导入选项 -->
      <div v-if="pendingItems.length > 0" class="import-options">
        <div class="option-group">
          <label class="option-label">默认状态</label>
          <div class="radio-group">
            <label class="radio-item" :class="{ active: defaultStatus === 'draft' }">
              <input type="radio" v-model="defaultStatus" value="draft" />
              <span>草稿</span>
            </label>
            <label class="radio-item" :class="{ active: defaultStatus === 'published' }">
              <input type="radio" v-model="defaultStatus" value="published" />
              <span>已发布</span>
            </label>
          </div>
        </div>
        <button
          class="btn-import"
          :disabled="importing"
          @click="startImport"
        >
          <svg v-if="!importing" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="17 8 12 3 7 8"/>
            <line x1="12" y1="3" x2="12" y2="15"/>
          </svg>
          <span>{{ importing ? '导入中...' : `开始导入` }}</span>
        </button>
      </div>

      <!-- 导入结果 -->
      <div v-if="importResult" class="import-results" :class="{ success: importResult.success, fail: !importResult.success }">
        <div class="result-header">
          <svg v-if="importResult.success" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="20 6 9 17 4 12"/>
          </svg>
          <svg v-else width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <line x1="12" y1="8" x2="12" y2="12"/>
            <line x1="12" y1="16" x2="12.01" y2="16"/>
          </svg>
          <span class="result-title">{{ importResult.success ? '✅ 导入成功' : '❌ 导入失败' }}</span>
        </div>
        <div class="result-content">
          <p>{{ importResult.message }}</p>
          <div v-if="importResult.errors && importResult.errors.length > 0" class="error-list">
            <h4>失败详情:</h4>
            <ul>
              <li v-for="(e, i) in importResult.errors" :key="i">{{ e }}</li>
            </ul>
          </div>
        </div>
      </div>
    </div>

    <!-- 导出面板 -->
    <div v-if="activeTab === 'export'" class="export-panel">
      <div class="export-options">
        <div class="option-group">
          <label class="option-label">导出范围</label>
          <div class="radio-group">
            <label class="radio-item" :class="{ active: exportScope === 'all' }">
              <input type="radio" v-model="exportScope" value="all" />
              <span>全部文章</span>
            </label>
            <label class="radio-item" :class="{ active: exportScope === 'selected' }">
              <input type="radio" v-model="exportScope" value="selected" />
              <span>选择文章</span>
            </label>
          </div>
        </div>
        <div class="option-group">
          <label class="option-label">
            <input type="checkbox" v-model="exportDrafts" />
            <span>包含草稿</span>
          </label>
        </div>
        <div class="option-group" style="margin-left: auto;">
          <button
            class="btn-export"
            :disabled="exporting"
            @click="startExport"
          >
            <svg v-if="!exporting" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
              <polyline points="7 10 12 15 17 10"/>
              <line x1="12" y1="15" x2="12" y2="3"/>
            </svg>
            <span>{{ exporting ? '导出中...' : '开始导出' }}</span>
          </button>
        </div>
      </div>

      <!-- 文章列表 -->
      <div v-if="exportScope === 'selected'" class="posts-list-container">
        <div class="posts-list-header">
          <h3>文章列表</h3>
          <button 
            class="btn-import btn-sm"
            :disabled="exporting"
            @click="selectAllPosts"
          >
            全选
          </button>
          <button
            class="btn-import btn-sm"
            :disabled="exporting || selectedPostIds.length === 0"
            @click="clearSelection"
          >
            清空
          </button>
          <div class="filter-controls">
            <input 
              type="text" 
              v-model="postFilter" 
              placeholder="搜索文章标题..."
              class="post-id-input"
            />
            <select v-model="statusFilter" class="post-id-input" style="width: 120px;">
              <option value="all">全部状态</option>
              <option value="published">已发布</option>
              <option value="draft">草稿</option>
            </select>
          </div>
        </div>
        
        <div class="posts-list" v-if="!loadingPosts">
          <div 
            v-for="post in filteredPosts" 
            :key="post.id"
            class="post-item"
            :class="{ selected: selectedPostIds.includes(post.id.toString()) }"
            @click="togglePostSelection(post.id.toString())"
          >
            <div class="post-checkbox">
              <input 
                type="checkbox" 
                :checked="selectedPostIds.includes(post.id.toString())"
                @click.stop="togglePostSelection(post.id.toString())"
              />
            </div>
            <div class="post-info">
              <div class="post-title">{{ post.title }}</div>
              <div class="post-meta">
                <span class="post-date">{{ formatDate(post.created_at) }}</span>
                <span class="post-status" :class="post.status">{{ post.status === 'published' ? '已发布' : '草稿' }}</span>
                <span class="post-author">{{ post.author_name || '未知' }}</span>
              </div>
            </div>
          </div>
          <div v-if="filteredPosts.length === 0" class="empty-state">
            没有找到符合条件的文章
          </div>
        </div>
        <div v-else class="loading-state">
          加载文章列表中...
        </div>
      </div>

      <!-- 导出说明 -->
      <div class="export-guide">
        <h3>📝 导出说明</h3>
        <ul>
          <li>每篇文章会生成独立的文件夹，命名格式：<code>作者_日期_标题</code></li>
          <li>文章内容会自动添加标准的 Front Matter 元数据</li>
          <li>文章中的图片会保存在 <code>uploads/</code> 子目录</li>
          <li>导出的 ZIP 包可直接再次导入，实现批量恢复功能</li>
        </ul>
      </div>
    </div>

    <!-- Front Matter 格式说明 -->
    <div class="fm-guide">
      <h3 class="guide-title">📋 Front Matter 格式说明</h3>
      <p class="guide-desc">导入时会自动解析 Markdown 文件开头的 YAML 元数据：</p>
      <div class="fm-example">
        <pre><code>---
title: 文章标题
date: 2024-01-15 10:30:00
status: published
draft: false
category: 技术
tags:
  - Rust
  - Web 开发
slug: my-post-slug
summary: 文章摘要描述
author_id: 1
---

# 正文内容从这里开始

Markdown 正文...</code></pre>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { importFromFiles, parseZipAndImport, exportPosts, type ImportResult } from '@/api/importExport'
import { fetchAdminPosts } from '@/api/posts'

const fileInput = ref<HTMLInputElement>()
const folderInput = ref<HTMLInputElement>()
const activeTab = ref<'import' | 'export'>('import')
const importMode = ref<'files' | 'zip'>('files')
const exportScope = ref<'all' | 'selected'>('all')
const exportDrafts = ref(true)
const selectedPostIds = ref<string[]>([])
const defaultStatus = ref<'draft' | 'published'>('draft')
const isDragging = ref(false)
const importing = ref(false)
const exporting = ref(false)
const pendingItems = ref<{ name: string; size: number; type: string; file: File }[]>([])
const importResult = ref<ImportResult | null>(null)

// 导出文章列表相关
const posts = ref<any[]>([])
const loadingPosts = ref(false)
const postFilter = ref('')
const statusFilter = ref('all')

interface PendingItem {
  name: string
  size: number
  type: string
  file: File
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(2) + ' MB'
}

function formatDate(dateString: string): string {
  const date = new Date(dateString)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

function triggerUpload(): void {
  fileInput.value?.click()
}

function triggerFolderUpload(): void {
  folderInput.value?.click()
}

async function onFileChange(event: Event): Promise<void> {
  const target = event.target as HTMLInputElement
  const files = target.files
  if (!files || files.length === 0) return
  
  await processFiles(Array.from(files))
  target.value = ''
}

async function onDrop(event: DragEvent): Promise<void> {
  isDragging.value = false
  const files = event.dataTransfer?.files
  if (!files || files.length === 0) return
  
  await processFiles(Array.from(files))
}

function onDragLeave(event: DragEvent): void {
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect()
  const x = event.clientX
  const y = event.clientY
  if (x <= rect.left || x >= rect.right || y <= rect.top || y >= rect.bottom) {
    isDragging.value = false
  }
}

async function processFiles(files: File[]): Promise<void> {
  for (const file of files) {
    const ext = file.name.split('.').pop()?.toLowerCase()
    let type = 'other'
    
    if (ext === 'md' || ext === 'markdown') type = 'md'
    else if (ext === 'zip') type = 'zip'
    else if (file.type.startsWith('image/')) type = 'image'
    
    if (type !== 'other') {
      pendingItems.value.push({
        name: file.name,
        size: file.size,
        type,
        file
      })
    }
  }
}

function removeItem(index: number): void {
  pendingItems.value.splice(index, 1)
}

async function startImport(): Promise<void> {
  if (pendingItems.value.length === 0) return
  
  importing.value = true
  importResult.value = null
  
  try {
    const files = pendingItems.value.map(item => item.file)
    const zipFiles = files.filter(f => f.name.endsWith('.zip'))
    const regularFiles = files.filter(f => !f.name.endsWith('.zip'))
    
    let result: ImportResult
    
    if (zipFiles.length > 0) {
      result = await parseZipAndImport(zipFiles[0])
    } else {
      result = await importFromFiles(regularFiles)
    }
    
    importResult.value = result
  } catch (error: any) {
    importResult.value = {
      success: false,
      message: error.message || '导入失败',
      imported_count: 0,
      skipped_count: 0,
      errors: [error.message || '未知错误']
    }
  } finally {
    importing.value = false
    pendingItems.value = []
  }
}

async function startExport(): Promise<void> {
  if (exportScope.value === 'selected' && selectedPostIds.value.length === 0) {
    alert('没有选中文章，请先选择需要导出的文章')
    return
  }
  
  exporting.value = true
  
  try {
    let postIds: number[] | undefined
    
    if (exportScope.value === 'selected' && selectedPostIds.value.length > 0) {
      postIds = selectedPostIds.value
        .map(id => parseInt(id))
        .filter(id => !isNaN(id))
    }
    
    await exportPosts(postIds)
  } catch (error: any) {
    console.error('Export failed:', error)
    alert('导出失败: ' + (error.message || '未知错误'))
  } finally {
    exporting.value = false
  }
}

// 文章列表相关
async function loadPosts(): Promise<void> {
  loadingPosts.value = true
  try {
    const { data } = await fetchAdminPosts({ page: 1, page_size: 100, status: exportDrafts.value ? undefined : 'published' })
    posts.value = data.data.map((p: any) => ({
      ...p,
      author_name: p.author?.username || '未知'
    }))
  } catch (error) {
    console.error('Load posts failed:', error)
  } finally {
    loadingPosts.value = false
  }
}

const filteredPosts = computed(() => {
  let filtered = posts.value
  
  // 状态筛选
  if (statusFilter.value !== 'all') {
    filtered = filtered.filter(p => p.status === statusFilter.value)
  }
  
  // 标题搜索
  if (postFilter.value) {
    const filter = postFilter.value.toLowerCase()
    filtered = filtered.filter(p => p.title.toLowerCase().includes(filter))
  }
  
  return filtered
})

function togglePostSelection(postId: string): void {
  const index = selectedPostIds.value.indexOf(postId)
  if (index > -1) {
    selectedPostIds.value.splice(index, 1)
  } else {
    selectedPostIds.value.push(postId)
  }
}

function selectAllPosts(): void {
  selectedPostIds.value = filteredPosts.value.map(p => p.id.toString())
}

function clearSelection(): void {
  selectedPostIds.value = []
}

// 监听导出范围变化，加载文章列表
watch(exportScope, (newScope) => {
  if (newScope === 'selected') {
    loadPosts()
  }
})

// 监听包含草稿选项变化
watch(exportDrafts, () => {
  if (exportScope.value === 'selected') {
    loadPosts()
  }
})

onMounted(() => {
  if (exportScope.value === 'selected') {
    loadPosts()
  }
})
</script>

<style scoped>
.import-page {
  min-height: 80vh;
  color: var(--input-color);
}

.tabs {
  display: flex;
  gap: 8px;
  margin-bottom: 20px;
}

.tab-btn {
  padding: 10px 20px;
  border: 1px solid var(--card-border-color);
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.05);
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s;
}

.tab-btn:hover {
  background: var(--card-border-color);
  color: var(--input-color);
}

.tab-btn.active {
  background: rgba(79, 70, 229, 0.12);
  color: var(--input-color);
  border-color: rgba(79, 70, 229, 0.3);
}

.import-modes {
  display: flex;
  gap: 12px;
  margin-bottom: 20px;
}

.mode-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 24px;
  border: 2px solid var(--card-border-color);
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.05);
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s;
}

.mode-btn:hover {
  border-color: rgba(79, 70, 229, 0.5);
  color: var(--input-color);
}

.mode-btn.active {
  border-color: rgba(79, 70, 229, 0.5);
  background: rgba(79, 70, 229, 0.12);
  color: var(--input-color);
}

.upload-zone {
  border: 2px dashed var(--card-border-color);
  border-radius: 12px;
  padding: 40px;
  text-align: center;
  cursor: pointer;
  transition: all 0.2s;
  margin-bottom: 20px;
  background: rgba(255, 255, 255, 0.03);
}

.upload-zone:hover,
.upload-zone.dragging {
  border-color: rgba(79, 70, 229, 0.5);
  background: rgba(79, 70, 229, 0.05);
}

.upload-zone.has-files {
  border-style: solid;
  background: rgba(255, 255, 255, 0.05);
}

.upload-prompt {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  color: var(--text-secondary);
}

.upload-prompt svg {
  margin-bottom: 16px;
  color: var(--text-dim);
  opacity: 0.7;
  flex-shrink: 0;
}

.upload-title {
  font-size: 16px;
  font-weight: 500;
  margin-bottom: 8px;
  color: var(--input-color);
}

.upload-link { color: #4f46e5; font-weight: 500; }

.upload-sub {
  font-size: 14px;
  color: var(--text-secondary);
}

.file-list {
  text-align: left;
}

.file-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 8px;
  margin-bottom: 8px;
  border: 1px solid var(--card-border-color);
}

.file-icon {
  color: #4f46e5;
  opacity: 0.8;
}

.file-icon.zip-icon {
  color: #f59e0b;
}

.file-detail {
  flex: 1;
}

.file-name {
  display: block;
  font-size: 14px;
  font-weight: 500;
  color: var(--input-color);
}

.file-size {
  font-size: 12px;
  color: var(--text-secondary);
}

.file-remove {
  padding: 6px;
  border: none;
  background: rgba(220, 38, 38, 0.1);
  color: #fecaca;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
}

.file-remove:hover {
  background: rgba(220, 38, 38, 0.2);
  color: #fee2e2;
}

.hidden-input {
  display: none;
}

.import-options,
.export-options {
  display: flex;
  align-items: center;
  gap: 20px;
  padding: 16px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 12px;
  border: 1px solid var(--card-border-color);
  margin-bottom: 20px;
}

.option-group {
  display: flex;
  align-items: center;
  gap: 12px;
}

.option-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--input-color);
}

.radio-group {
  display: flex;
  gap: 16px;
}

.radio-item {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  font-size: 14px;
  color: var(--text-secondary);
  padding: 6px 12px;
  border-radius: 6px;
  transition: all 0.2s;
}

.radio-item input {
  display: none;
}

.radio-item.active {
  background: rgba(79, 70, 229, 0.12);
  color: var(--input-color);
}

.btn-import,
.btn-export {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 24px;
  background: #4f46e5;
  color: white;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s;
  margin-left: auto;
}

.btn-import:hover:not(:disabled),
.btn-export:hover:not(:disabled) {
  background: #4338ca;
  transform: translateY(-1px);
}

.btn-import:disabled,
.btn-export:disabled {
  background: #3730a3;
  cursor: not-allowed;
  opacity: 0.6;
}

.btn-sm {
  padding: 6px 14px !important;
  font-size: 12px !important;
  margin-left: 0 !important;
  border-radius: 6px !important;
}

.import-results {
  padding: 20px;
  border-radius: 12px;
  margin-bottom: 20px;
  border: 1px solid var(--card-border-color);
  background: rgba(255, 255, 255, 0.03);
}

.import-results.success {
  background: rgba(16, 185, 129, 0.1);
  border-color: rgba(16, 185, 129, 0.3);
  color: #a7f3d0;
}

.import-results.fail {
  background: rgba(220, 38, 38, 0.1);
  border-color: rgba(220, 38, 38, 0.3);
  color: #fecaca;
}

.result-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.result-title {
  font-size: 18px;
  font-weight: 600;
}

.result-content {
  font-size: 14px;
}

.error-list {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--card-border-color);
}

.error-list h4 {
  margin-bottom: 8px;
  color: #fecaca;
}

.error-list ul {
  margin: 0;
  padding-left: 20px;
}

.error-list li {
  margin-bottom: 4px;
}

.post-id-input {
  padding: 8px 12px;
  border: 1px solid var(--card-border-color);
  border-radius: 6px;
  font-size: 14px;
  width: 200px;
  background: rgba(255, 255, 255, 0.05);
  color: var(--input-color);
}

.post-id-input:focus {
  outline: none;
  border-color: rgba(79, 70, 229, 0.5);
  background: rgba(255, 255, 255, 0.08);
}

.export-guide {
  padding: 16px;
  background: rgba(245, 158, 11, 0.1);
  border-radius: 12px;
  border: 1px solid rgba(245, 158, 11, 0.25);
  margin-top: 20px;
  color: #92400e;
}

.export-guide h3 {
  margin: 0 0 12px 0;
  font-size: 16px;
  color: #92400e;
}

.export-guide ul {
  margin: 0;
  padding-left: 20px;
}

.export-guide li {
  margin-bottom: 8px;
  font-size: 14px;
  color: #92400e;
}

.export-guide code {
  background: rgba(245, 158, 11, 0.2);
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 13px;
  color: #b45309;
}

/* 暗色主题下恢复亮色文字 */
.dark .export-guide { color: #fcd34d; }
.dark .export-guide h3 { color: #fcd34d; }
.dark .export-guide li { color: #fcd34d; }
.dark .export-guide code { color: #fef3c7; }

.fm-guide {
  padding: 20px;
  background: var(--card-bg);
  border-radius: 12px;
  border: 1px solid var(--card-border-color);
  margin-top: 20px;
}

.guide-title {
  margin: 0 0 12px 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--input-color);
}

.guide-desc {
  margin: 0 0 16px 0;
  font-size: 14px;
  color: var(--input-color);
}

.fm-example {
  background: var(--card-bg);
  padding: 16px;
  border-radius: 8px;
  overflow-x: auto;
  margin-bottom: 16px;
  border: 1px solid var(--card-border-color);
}

.fm-example pre {
  margin: 0;
  font-size: 13px;
  line-height: 1.6;
  color: var(--input-color);
}

.fm-example code {
  font-family: 'Fira Code', 'Monaco', monospace;
}

.page-header {
  margin-bottom: 24px;
}

.page-title {
  margin: 0 0 8px 0;
  font-size: 28px;
  font-weight: 700;
  color: var(--input-color);
}

.page-desc {
  margin: 0;
  font-size: 14px;
  color: var(--text-secondary);
}

.post-id-input::placeholder {
  color: var(--text-dim);
}

/* 文章列表样式 */
.posts-list-container {
  margin: 20px 0;
  border: 1px solid var(--card-border-color);
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.03);
  overflow: hidden;
}

.posts-list-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 20px;
  border-bottom: 1px solid var(--card-border-color);
  background: rgba(255, 255, 255, 0.05);
}

.posts-list-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--input-color);
}

.filter-controls {
  display: flex;
  gap: 12px;
  align-items: center;
  margin-left: auto;
}

.posts-list {
  max-height: 400px;
  overflow-y: auto;
}

.post-item {
  display: flex;
  align-items: center;
  padding: 12px 20px;
  border-bottom: 1px solid var(--card-border-color);
  cursor: pointer;
  transition: all 0.2s;
}

.post-item:hover {
  background: rgba(255, 255, 255, 0.05);
}

.post-item.selected {
  background: rgba(79, 70, 229, 0.12);
  border-left: 4px solid #4f46e5;
}

.post-checkbox {
  margin-right: 12px;
  flex-shrink: 0;
}

.post-checkbox input {
  width: 16px;
  height: 16px;
  accent-color: #4f46e5;
}

.post-info {
  flex: 1;
}

.post-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--input-color);
  margin-bottom: 4px;
}

.post-meta {
  display: flex;
  gap: 16px;
  font-size: 12px;
  color: var(--text-secondary);
}

.post-status {
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 500;
}

.post-status.published {
  background: rgba(16, 185, 129, 0.2);
  color: #a7f3d0;
}

.post-status.draft {
  background: rgba(245, 158, 11, 0.2);
  color: #fcd34d;
}

.empty-state,
.loading-state {
  padding: 40px;
  text-align: center;
  color: var(--text-secondary);
  font-size: 14px;
}

/* 滚动条样式 */
.posts-list::-webkit-scrollbar {
  width: 6px;
}

.posts-list::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.05);
}

.posts-list::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 3px;
}

.posts-list::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.3);
}
</style>
