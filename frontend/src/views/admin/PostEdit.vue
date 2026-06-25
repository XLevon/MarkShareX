<template>
  <div class="editor-page" :style="{ backgroundColor: 'var(--color-bg)' }">
    <!-- 顶部栏：返回 + 标题 + 分类 + 操作 -->
    <div class="editor-topbar" :style="{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border)' }">
      <div class="topbar-left">
        <button @click="router.back()" class="topbar-back" :style="{ color: 'var(--color-text-secondary)' }">
          ← 返回
        </button>
        <input
          v-model="form.title"
          type="text"
          placeholder="输入文章标题..."
          class="topbar-title"
          :style="{
            backgroundColor: 'var(--color-bg)',
            borderColor: titleError ? '#f87171' : 'var(--color-border)',
            color: 'var(--color-text)'
          }"
          @input="titleError = false"
        />

        <!-- 分类选择 -->
        <div class="topbar-category">
          <n-select
            v-model:value="form.category_id"
            :options="categoryOptions"
            placeholder="选择分类"
            clearable
            size="small"
            :style="{ minWidth: '120px' }"
            :consistent-menu-width="false"
          />
          <span v-if="categoryError" class="field-error">必选</span>
        </div>
      </div>

      <div class="topbar-right">
        <!-- 设置按钮 -->
        <button @click="openSettings" class="settings-btn" :style="{ borderColor: 'var(--color-border)', color: 'var(--color-text-secondary)' }" title="文章设置">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/></svg>
        </button>

        <!-- 字数统计 -->
        <span class="word-count" :style="{ color: 'var(--color-text-muted)' }">{{ wordCount }} 字</span>

        <!-- 状态标签 -->
        <span class="status-badge" :style="form.status === 'published' ? { backgroundColor: 'rgba(74,222,128,0.15)', color: '#4ade80' } : { backgroundColor: 'rgba(251,191,36,0.15)', color: '#fbbf24' }">
          {{ form.status === 'published' ? '已发布' : '草稿' }}
        </span>
        <span v-if="dirty" class="dirty-dot" title="未保存">⬤</span>

        <!-- 操作按钮 -->
        <button @click="handleSave('draft')" :disabled="saving" class="btn-draft" :style="{ borderColor: 'var(--color-border)', color: 'var(--color-text)' }">
          保存草稿
        </button>
        <button @click="handleSave('published')" :disabled="saving" class="btn-publish" :style="{ backgroundColor: 'var(--color-primary)' }">
          {{ saving ? '保存中...' : '发布文章' }}
        </button>
      </div>
    </div>

    <!-- 双栏：编辑器 + 预览 -->
    <div class="editor-body" :style="{ backgroundColor: 'var(--color-bg)' }">
      <!-- 左：编辑器 -->
      <div v-if="!previewFullscreen" class="editor-pane" :class="{ 'preview-hidden': previewCollapsed }">
        <div v-if="!editorReady" class="editor-loading" :style="{ color: 'var(--color-text-muted)' }">加载文章内容...</div>
        <VditorEditor v-else ref="editorRef" v-model="form.content" @update:modelValue="markDirty" @openLibrary="openImageLibrary" @file-uploaded="onEditorFileUploaded" />
      </div>

      <!-- 拖拽分割条 -->
      <div
        v-if="!previewCollapsed && !previewFullscreen"
        class="splitter"
        :class="{ dragging: isDragging }"
        @mousedown="onSplitterMouseDown"
      ></div>

      <!-- 右：预览 -->
      <div
        v-if="!previewCollapsed"
        ref="previewPaneRef"
        class="preview-pane"
        :class="{ 'preview-fullscreen': previewFullscreen }"
        :style="{ width: previewFullscreen ? '100%' : previewWidth + 'px', borderColor: 'var(--color-border)', backgroundColor: 'var(--color-bg)' }"
        @click="handlePreviewClick"
      >
        <div class="preview-header" :style="{ borderColor: 'var(--color-border)', backgroundColor: 'var(--color-bg-card)' }">
          <span class="preview-title" :style="{ color: 'var(--color-text)' }">预览</span>
          <div class="preview-header-actions">
            <button @click="previewFullscreen = !previewFullscreen" class="preview-action-btn" :style="{ color: 'var(--color-text-muted)' }">
              {{ previewFullscreen ? '退出全屏' : '全屏' }}
            </button>
            <button @click="previewCollapsed = true" class="preview-action-btn" :style="{ color: 'var(--color-text-muted)' }">收起</button>
          </div>
        </div>
        <CodeCopyWrapper class="preview-content markdown-body" :style="{ color: 'var(--color-text)' }" :html="renderedPreview" />
      </div>

      <!-- 展开预览按钮 -->
      <button
        v-if="previewCollapsed"
        @click="previewCollapsed = false"
        class="expand-preview-btn"
        :style="{ borderColor: 'var(--color-border)', color: 'var(--color-text-muted)', backgroundColor: 'var(--color-bg-card)' }"
      >预览</button>
    </div>

    <!-- 设置弹窗 -->
    <teleport to="body">
      <PostSettingsModal
        :visible="showSettings"
        :cover-image="form.cover_image"
        :summary="form.summary"
        :tags="tagNames"
        :article-type="form.article_type"
        :article-status="form.article_status"
        @confirm="confirmSettings"
        @cancel="cancelSettings"
      />
    </teleport>

    <!-- 统一图片选择器 -->
    <ImageSelector
      :visible="showImageSelector"
      :title="selectorTitle"
      @close="showImageSelector = false"
      @select="onImageSelected"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted, watch } from 'vue'
import { useRouter, useRoute, onBeforeRouteLeave } from 'vue-router'
import { useMessage } from 'naive-ui'
import VditorEditor from '@/components/admin/VditorEditor.vue'
import ImageSelector from '@/components/shared/ImageSelector.vue'
import PostSettingsModal from '@/components/shared/PostSettingsModal.vue'
import { fetchPost, createPost, updatePost } from '@/api/posts'
import { fetchAdminCategories } from '@/api/categories'
import { batchDeleteFiles } from '@/api/files'
import { fetchNetworkResources } from '@/api/admin'
import type { Category } from '@/api/index'
import { marked } from 'marked'
// marked 15.x 移除了 headerIds 选项，用自定义 renderer 生成 heading ID
// 规则：空格→-、删标点、保留字母/数字/中文/连字符、合并连续-
const headingRenderer = new marked.Renderer()
headingRenderer.heading = function ({ tokens, depth }: { tokens: any[]; depth: number }) {
  const text = this.parser.parseInline(tokens)
  const id = text
    .replace(/<[^>]*>/g, '')            // 去除 HTML 标签
    .toLowerCase()
    .replace(/\s+/g, '-')               // 空格→-
    .replace(/[^\w\u4e00-\u9fff-]/g, '') // 删标点符号
    .replace(/-+/g, '-')                // 合并连续-
    .replace(/^-+|-+$/g, '')            // 去除首尾-
  return `<h${depth} id="${id}">${text}</h${depth}>\n`
}
import CodeCopyWrapper from '@/components/shared/CodeCopyWrapper.vue'

const router = useRouter()
const route = useRoute()
const message = useMessage()

const loading = ref(false)
const saving = ref(false)
const dirty = ref(false)
// 本次编辑过程中上传的文件 ID（取消时批量删除）
const uploadedFileIds = ref<number[]>([])
const previewCollapsed = ref(false)
const previewFullscreen = ref(false)
const previewPaneRef = ref<HTMLElement>()
const previewWidth = ref(420)
const isDragging = ref(false)
const showSettings = ref(false)
function openSettings() {
  showSettings.value = true
}
function confirmSettings(values: { coverImage: string; summary: string; tags: string[]; articleType: string; articleStatus: string }) {
  form.cover_image = values.coverImage
  form.summary = values.summary
  tagNames.value = values.tags
  form.article_type = values.articleType
  form.article_status = values.articleStatus
  markDirty()
  showSettings.value = false
}
function cancelSettings() {
  showSettings.value = false
}
// ── 统一图片选择器 ──
const showImageSelector = ref(false)
const selectorMode = ref<'cover' | 'insert'>('cover')
const selectorTitle = computed(() => selectorMode.value === 'insert' ? '从资源库选择图片插入' : '设置封面图片')

function openImageSelector(mode: 'cover' | 'insert') {
  selectorMode.value = mode
  showImageSelector.value = true
}

function onImageSelected(value: string) {
  // 归一化：自托管绝对 URL → 相对路径 /uploads/xxx
  let normalized = value.startsWith('http') && value.includes('/uploads/')
    ? value.replace(/https?:\/\/[^/]+\/uploads\//, '/uploads/')
    : value
  // 插入编辑器时：nr:{id} → 解析为实际 URL 以便预览
  if (selectorMode.value === 'insert' && normalized.startsWith('nr:')) {
    const id = Number(normalized.slice(3))
    const resolvedUrl = networkUrlCache.get(id)
    if (resolvedUrl) normalized = resolvedUrl
  }
  if (selectorMode.value === 'cover') {
    form.cover_image = normalized
    markDirty()
  } else if (selectorMode.value === 'insert') {
    editorRef.value?.insertImage(normalized, '')
    markDirty()
  }
  showImageSelector.value = false
}

// === 编辑器插图库 ===
const editorRef = ref<InstanceType<typeof VditorEditor>>()
function openImageLibrary() {
  openImageSelector('insert')
}

// 编辑器内粘贴/拖拽上传的图片，记录文件 ID
function onEditorFileUploaded(fileId: number) {
  uploadedFileIds.value.push(fileId)
}

const titleError = ref(false)
const categoryError = ref(false)
const categories = ref<Category[]>([])
const tagNames = ref<string[]>([])

const isEdit = computed(() => !!route.params.id)
const editorReady = ref(!isEdit.value)  // 新建立即渲染，编辑等数据加载完

const form = reactive({
  title: '',
  content: '',
  summary: '',
  status: 'draft' as string,
  category_id: null as number | null,
  cover_image: '',
  article_type: 'ai_organized' as string,
  article_status: 'latest' as string,
})

const categoryOptions = computed(() => {
  const opts: { label: string; value: number | null }[] = []
  const cats = categories.value || []
  const topLevel = cats.filter(c => !c.parent_id)
  const children = cats.filter(c => c.parent_id)
  
  topLevel.sort((a, b) => a.sort_order - b.sort_order).forEach(c => {
    opts.push({ label: c.name, value: c.id })
    children.filter(ch => ch.parent_id === c.id)
      .sort((a, b) => a.sort_order - b.sort_order)
      .forEach(ch => {
        opts.push({ label: `  ⤷ ${ch.name}`, value: ch.id })
      })
  })
  // 孤儿子分类（parent 已被删除或不在列表中）
  const seenIds = new Set(opts.map(o => o.value))
  children.filter(ch => !seenIds.has(ch.id)).forEach(ch => {
    opts.push({ label: `  ⤷ ${ch.name}`, value: ch.id })
  })
  return opts
})

const wordCount = computed(() => {
  if (!form.content) return 0
  return form.content.replace(/[#*`\[\]()>\\-_\\s]/g, '').length
})

// 封面图片展示URL：文件名→动态拼接，完整URL→直接使用
const networkUrlCache = reactive(new Map<number, string>())

const renderedPreview = computed(() => {
  if (!form.content) return '<p style="color: var(--color-text-muted);opacity:0.5">开始写作...</p>'
  return marked.parse(form.content, { renderer: headingRenderer })
})

// 预览区内锚点跳转：拦截 # 链接点击，手动滚动 .preview-content 容器
function handlePreviewClick(e: MouseEvent) {
  const target = e.target as HTMLElement
  const anchor = target.closest('a[href^="#"]') as HTMLAnchorElement | null
  if (!anchor) return
  const href = anchor.getAttribute('href')
  if (!href || href === '#') return
  const id = href.slice(1)
  const container = previewPaneRef.value?.querySelector('.preview-content') as HTMLElement | null
  if (!container) return
  const heading = container.querySelector(`#${CSS.escape(id)}`) as HTMLElement | null
  if (!heading) return
  e.preventDefault()
  // 给当前历史条目打标记（返回时恢复到此位置）
  history.replaceState({ previewScrollTop: container.scrollTop }, '')
  // 推入新条目记录跳转目标
  history.pushState({ previewScrollTop: container.scrollTop, anchorId: id }, '', `#${id}`)
  const offset = heading.getBoundingClientRect().top - container.getBoundingClientRect().top + container.scrollTop - 16
  container.scrollTo({ top: offset, behavior: 'smooth' })
}

// 监听浏览器后退，恢复预览区滚动位置
function onPreviewPopstate(e: PopStateEvent) {
  const container = previewPaneRef.value?.querySelector('.preview-content') as HTMLElement | null
  if (container && e.state?.previewScrollTop !== undefined) {
    container.scrollTo({ top: e.state.previewScrollTop, behavior: 'smooth' })
  }
}
onMounted(() => window.addEventListener('popstate', onPreviewPopstate))
onUnmounted(() => window.removeEventListener('popstate', onPreviewPopstate))

// === 预览面板拖拽 ===
function onSplitterMouseDown(e: MouseEvent) {
  isDragging.value = true
  const startX = e.clientX
  const startWidth = previewWidth.value

  const onMove = (ev: MouseEvent) => {
    const delta = startX - ev.clientX
    previewWidth.value = Math.max(200, Math.min(800, startWidth + delta))
  }
  const onUp = () => {
    isDragging.value = false
    document.removeEventListener('mousemove', onMove)
    document.removeEventListener('mouseup', onUp)
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
  }

  document.addEventListener('mousemove', onMove)
  document.addEventListener('mouseup', onUp)
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
}

async function loadData() {
  loading.value = true
  try {
    const { data: catResp } = await fetchAdminCategories()
    categories.value = catResp?.data ?? []
    if (isEdit.value) {
      const id = Number(route.params.id)
      console.log('[PostEdit] loading post id:', id)
      const { data: resp } = await fetchPost(id)
      console.log('[PostEdit] fetchPost response:', resp)
      const post = resp.data
      if (!post) {
        console.error('[PostEdit] fetchPost returned empty data')
        message.error('文章数据为空')
        return
      }
      console.log('[PostEdit] post content length:', post.content?.length || 0, 'title:', post.title)
      form.title = post.title
      form.content = (post.content || '').replace(/\(\.\/uploads\//g, '(/uploads/')
      form.summary = post.summary || ''
      form.status = post.status
      form.category_id = post.category_id
      form.cover_image = post.cover_image || ''
      form.article_type = post.article_type || 'ai_organized'
      form.article_status = post.article_status || 'latest'
      tagNames.value = (post.tags || []).map((t: any) => typeof t === 'string' ? t : t.name).filter(Boolean)
    }
  } catch (e: any) {
    console.error('[PostEdit] loadData failed:', e)
    message.error('加载文章失败: ' + (e?.response?.data?.error || e?.message || '未知错误'))
  } finally {
    loading.value = false
    editorReady.value = true
  }
}

async function handleSave(status: string) {
  // Validate
  titleError.value = !form.title.trim()
  categoryError.value = !form.category_id

  if (titleError.value) {
    message.warning('请输入文章标题')
    return
  }
  if (categoryError.value) {
    message.warning('请选择文章分类')
    return
  }

  saving.value = true
  try {
    const payload = {
      title: form.title,
      content: form.content.replace(/\(\/uploads\//g, '(./uploads/'),
      summary: form.summary,
      status,
      category_id: form.category_id,
      tags: tagNames.value,
      cover_image: form.cover_image,
      article_type: form.article_type,
      article_status: form.article_status,
    }
    if (isEdit.value) {
      await updatePost(Number(route.params.id), payload)
      message.success('文章已更新')
    } else {
      await createPost(payload)
      message.success(status === 'published' ? '文章已发布' : '草稿已保存')
    }
    // 保存成功 → 图片已关联到文章内容，清除追踪
    uploadedFileIds.value = []
    dirty.value = false
    router.push(`/admin/posts?mtab=posts&status=${status}#p${isEdit.value ? route.params.id : ''}`)
  } catch (e: any) {
    message.error(e.response?.data?.error || '保存失败')
  } finally {
    saving.value = false
  }
}

function markDirty() { dirty.value = true }
async function cleanupUploads() {
  if (uploadedFileIds.value.length > 0) {
    try {
      await batchDeleteFiles(uploadedFileIds.value)
      uploadedFileIds.value = []
    } catch {
      // 静默失败：用户离开时不阻塞
    }
  }
}

// 路由离开时：若未保存，清理本次上传的图片
onBeforeRouteLeave(async (_to, _from, next) => {
  if (!dirty.value && uploadedFileIds.value.length === 0) {
    next()
    return
  }
  await cleanupUploads()
  next()
})

onMounted(async () => {
  await loadData()
  // 预热网络资源缓存（用于 nr:{id} → URL 解析）
  fetchNetworkResources({ page_size: 500, source_type: 'image' }).then(({ data: resp }) => {
    (resp.data || []).forEach((nr: any) => {
      networkUrlCache.set(nr.id, nr.url)
    })
  }).catch(() => {})
})

// 同组件路由切换（posts/new ↔ posts/:id）时重新加载数据
watch(() => route.params.id, () => {
  form.title = ''
  form.content = ''
  form.summary = ''
  form.status = 'draft'
  form.category_id = null
  form.cover_image = ''
  form.article_type = 'ai_organized'
  form.article_status = 'latest'
  tagNames.value = []
  editorReady.value = false
  loadData()
})
</script>

<style scoped>
.editor-page {
  height: calc(100vh - 56px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* ===== 顶部栏 ===== */
.editor-topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 8px 16px;
  border-bottom: 1px solid;
  flex-shrink: 0;
}
.topbar-left {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: 1;
  min-width: 0;
}
.topbar-back {
  flex-shrink: 0;
  padding: 4px 8px;
  border: none;
  background: none;
  cursor: pointer;
  font-size: 13px;
  border-radius: 6px;
}
.topbar-back:hover {
  background: rgba(255,255,255,0.06);
}
.topbar-title {
  flex: 1;
  min-width: 200px;
  height: 34px;
  padding: 0 12px;
  border: 1px solid;
  border-radius: 8px;
  font-size: 15px;
  font-weight: 600;
  outline: none;
  transition: border-color 0.15s;
}
.topbar-title::placeholder {
  font-weight: 400;
  opacity: 0.5;
}
.topbar-title:focus {
  border-color: var(--color-primary) !important;
}

.topbar-category {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 4px;
}
.field-error {
  font-size: 11px;
  color: #f87171;
  white-space: nowrap;
}

/* 右侧操作区 */
.topbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}
.word-count {
  font-size: 11px;
  white-space: nowrap;
}
.status-badge {
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 500;
  white-space: nowrap;
}
.dirty-dot {
  font-size: 8px;
  color: #fbbf24;
}

.settings-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid;
  border-radius: 8px;
  background: none;
  cursor: pointer;
  transition: all 0.15s;
}
.settings-btn:hover {
  background: rgba(255,255,255,0.06);
}

.btn-draft {
  padding: 6px 14px;
  border: 1px solid;
  border-radius: 8px;
  background: none;
  font-size: 13px;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.15s;
}
.btn-draft:hover {
  background: rgba(255,255,255,0.06);
}
.btn-draft:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-publish {
  padding: 6px 18px;
  border: none;
  border-radius: 8px;
  color: #fff;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  white-space: nowrap;
  transition: opacity 0.15s;
}
.btn-publish:hover {
  opacity: 0.85;
}
.btn-publish:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* ===== 双栏主体 ===== */
.editor-body {
  flex: 1;
  display: flex;
  overflow: hidden;
  position: relative;
}
.editor-pane {
  flex: 1;
  overflow: hidden;
}
.editor-pane.preview-hidden {
  flex: 1;
}

.editor-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  font-size: 14px;
}

.preview-pane {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 拖拽分割条 */
.splitter {
  width: 5px;
  flex-shrink: 0;
  cursor: col-resize;
  background: transparent;
  transition: background 0.15s;
  position: relative;
}
.splitter:hover,
.splitter.dragging {
  background: #4f46e5;
}
.preview-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  border-bottom: 1px solid;
  flex-shrink: 0;
}
.preview-title {
  font-size: 13px;
  font-weight: 500;
}
.preview-header-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}
.preview-action-btn {
  border: none;
  background: none;
  font-size: 12px;
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 4px;
  transition: all 0.15s;
}
.preview-action-btn:hover {
  background: rgba(255,255,255,0.08);
}
.preview-content {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

/* 预览全屏 */
.preview-fullscreen {
  flex: 1 !important;
  width: 100% !important;
  flex-shrink: 1 !important;
}

.expand-preview-btn {
  position: absolute;
  right: 12px;
  top: 12px;
  padding: 6px 14px;
  border: 1px solid;
  border-radius: 8px;
  font-size: 12px;
  cursor: pointer;
  z-index: 10;
  transition: all 0.15s;
}
.expand-preview-btn:hover {
  border-color: #4f46e5;
  color: #4f46e5;
}

/* ===== Library picker ===== */
.picker-modal {
  width: 640px;
  max-height: 80vh;
  border: 1px solid;
  border-radius: 14px;
  overflow: hidden;
  box-shadow: 0 20px 60px rgba(0,0,0,0.4);
  display: flex;
  flex-direction: column;
}
.picker-tabs {
  display: flex;
  gap: 0;
  border-bottom: 1px solid;
  padding: 0 18px;
}
.picker-tab {
  padding: 8px 16px;
  font-size: 13px;
  border: none;
  border-bottom: 2px solid;
  background: none;
  cursor: pointer;
  transition: all 0.15s;
}
.picker-tab:hover {
  color: #4f46e5;
}
.picker-tab.active {
  font-weight: 600;
}
.picker-body {
  padding: 16px;
  overflow-y: auto;
  flex: 1;
  max-height: 55vh;
}
.picker-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 10px;
}
.picker-item {
  border: 2px solid rgba(255,255,255,0.05);
  border-radius: 8px;
  overflow: hidden;
  cursor: pointer;
  transition: all 0.15s;
  background: rgba(0,0,0,0.2);
}
.picker-item:hover {
  border-color: rgba(79,70,229,0.3);
}
.picker-item.selected {
  border-color: #4f46e5;
}
.picker-item img {
  width: 100%;
  aspect-ratio: 1;
  object-fit: cover;
  display: block;
}
.picker-name {
  display: block;
  padding: 6px 8px;
  font-size: 11px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.picker-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding: 12px 16px;
  border-top: 1px solid;
}

/* Markdown 预览样式 */
.markdown-body { font-size: 14px; line-height: 1.7; }
.markdown-body :deep(h1) { font-size: 1.6em; margin: 0.5em 0; }
.markdown-body :deep(h2) { font-size: 1.3em; margin: 0.5em 0; }
.markdown-body :deep(p) { margin: 0.5em 0; }
.markdown-body :deep(code) { background: rgba(255,255,255,0.06); padding: 2px 6px; border-radius: 4px; font-size: 0.9em; }
.markdown-body :deep(pre) { background: var(--color-bg-card); padding: 12px; border-radius: 8px; overflow-x: auto; }
.markdown-body :deep(blockquote) { border-left: 3px solid var(--color-primary); padding-left: 12px; opacity: 0.8; }
.markdown-body :deep(table) { border-collapse: collapse; width: 100%; }
.markdown-body :deep(th), .markdown-body :deep(td) { border: 1px solid var(--color-border); padding: 6px 10px; text-align: left; }
.markdown-body :deep(a) { color: var(--color-primary); }
</style>

<style>
/* === Vditor 全屏时隐藏编辑器顶栏 + 预览面板 === */
body:has(.vditor--fullscreen) [class*="editor-topbar"],
body:has(.vditor--fullscreen) [class*="preview-pane"],
body:has(.vditor--fullscreen) [class*="expand-preview"] {
  display: none !important;
}
/* 编辑器全宽 */
body:has(.vditor--fullscreen) [class*="editor-pane"] {
  flex: 1 !important;
}
/* 确保预览区内 offsetTop 相对容器计算 */
.preview-content {
  position: relative;
}
</style>
