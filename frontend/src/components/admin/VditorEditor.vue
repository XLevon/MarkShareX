<template>
  <div class="editor-wrapper">
    <div ref="editorRef" class="editor-container"></div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch, nextTick } from 'vue'
import Vditor from 'vditor'
import 'vditor/dist/index.css'
import { uploadFile, batchUpload, calculateFileMd5 } from '@/api/files'
import { useDarkMode } from '@/composables/useDarkMode'

const props = defineProps<{
  modelValue: string
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
  (e: 'openLibrary'): void
  (e: 'fileUploaded', fileId: number): void
}>()

const editorRef = ref<HTMLElement>()
let vditor: Vditor | null = null
let isInternalChange = false

// 暴露 vditor 实例和插入方法给父组件
defineExpose({
  insertImage(url: string, alt: string) {
    const md = `\n![${alt}](${url})\n`
    vditor?.insertValue(md)
  },
})

// 处理粘贴的图片
async function handlePaste(e: ClipboardEvent) {
  const items = e.clipboardData?.items
  if (!items || items.length === 0) return

  const imageFiles: File[] = []
  for (const item of items) {
    if (item.type.startsWith('image/')) {
      const file = item.getAsFile()
      if (file) {
        imageFiles.push(file)
      }
    }
  }

  if (imageFiles.length > 0) {
    e.preventDefault()
    
    vditor?.tip(`📤 正在上传 ${imageFiles.length} 张图片...`, 2000)
    
    try {
      const { data: resp } = await batchUpload(imageFiles)
      const results = resp.data
      
      for (const result of results) {
        if (result.success && result.url) {
          const md = `\n![${result.original_name}](${result.url})\n`
          vditor?.insertValue(md)
        } else if (!result.success) {
          vditor?.tip(`❌ ${result.original_name} 上传失败: ${result.error || '未知错误'}`, 3000)
        }
      }
      
      if (results.every(r => r.success)) {
        vditor?.tip(`✅ 成功上传 ${results.length} 张图片`, 2000)
      }
    } catch (err: any) {
      const msg = err?.response?.data?.message || err?.message || '批量上传失败'
      vditor?.tip(`❌ ${msg}`, 3000)
      console.error('Batch upload failed:', err)
    }
  }
}

onMounted(() => {
  if (!editorRef.value) return
  const { isDark } = useDarkMode()
  initVditor(isDark.value)

  // 监听主题切换，实时更新编辑器主题
  watch(isDark, (dark) => {
    if (!vditor) return
    // 切换 Vditor 的 dark class
    const el = vditor.vditor?.element
    if (el) {
      if (dark) {
        el.classList.add('vditor--dark')
      } else {
        el.classList.remove('vditor--dark')
      }
    }
    // 更新编辑器内联样式中的 CSS 变量，使编辑区背景色跟随主题
    updateEditorBg()
  })

  // 添加粘贴事件监听
  editorRef.value.addEventListener('paste', handlePaste)

  // 首次加载时同步背景色
  nextTick(() => updateEditorBg())
})

// 给所有外链图片加 referrerpolicy，绕过防盗链
function fixExternalImageReferrer() {
  const container = editorRef.value
  if (!container) return
  container.querySelectorAll('img').forEach((img) => {
    const src = img.getAttribute('src') || ''
    if (src.startsWith('http')) {
      img.setAttribute('referrerpolicy', 'no-referrer')
    }
  })
}

function initVditor(dark: boolean) {
  if (!editorRef.value) return
  const vdTheme = dark ? 'dark' : 'classic'
  vditor = new Vditor(editorRef.value, {
    height: '100%',
    mode: 'ir',
    theme: vdTheme,
    icon: 'ant',
    link: {
      // Vditor 源码: if (options.link.click) { click(el) } else { window.open(url) }
      // 提供空回调 → 阻止 window.open，但不影响其他功能
      click: () => {},
    },
    cache: { enable: false },
    after: () => {
      // 初始渲染后修复外链图片 referrerpolicy
      fixExternalImageReferrer()
      // 用 MutationObserver 持续监控后续渲染的图片
      const observer = new MutationObserver(() => fixExternalImageReferrer())
      observer.observe(editorRef.value!, { childList: true, subtree: true })
    },
    value: props.modelValue || '',
    input: (value: string) => {
      isInternalChange = true
      emit('update:modelValue', value)
      isInternalChange = false
    },
    upload: {
      accept: 'image/*',
      handler: async (files: File[]) => {
        for (const file of files) {
          try {
            const { data: resp } = await uploadFile(file)
            const url = resp.data.url
            const md = `\n![${file.name}](${url})\n`
            vditor?.insertValue(md)
            vditor?.tip(`✅ ${file.name} 上传成功`, 2000)
            if (resp.data.id) emit('fileUploaded', resp.data.id)
          } catch (e: any) {
            const msg = e?.response?.data?.message || e?.message || '上传失败'
            vditor?.tip(`❌ ${msg}`, 3000)
            console.error('Upload failed:', e)
          }
        }
        return null
      },
    },
    toolbar: [
      'undo', 'redo', '|',
      'headings', 'bold', 'italic', 'strike', '|',
      'line', 'quote', 'list', 'ordered-list', 'check', '|',
      'code', 'inline-code', 'link', 'upload', 'table', '|',
      {
        name: 'library',
        tip: '从资源库选择图片',
        icon: '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="1" y="1" width="10" height="10" rx="1.5"/><rect x="13" y="1" width="10" height="10" rx="1.5"/><rect x="1" y="13" width="10" height="10" rx="1.5"/><polyline points="5 17 8 14 11 17"/><circle cx="8.5" cy="6" r="1"/></svg>',
        click: () => emit('openLibrary'),
      },
      '|',
      'fullscreen',
    ],
    toolbarConfig: {
      pin: false,
    },
    preview: {
      theme: { current: vdTheme },
    },
  })
}

// 更新编辑器背景色以跟随主题 CSS 变量
function updateEditorBg() {
  const wrapper = editorRef.value?.querySelector('.vditor-ir') as HTMLElement | null
  if (wrapper) {
    wrapper.style.backgroundColor = getComputedStyle(document.documentElement).getPropertyValue('--color-bg').trim()
  }
  // 同时更新预览区背景
  const preview = editorRef.value?.querySelector('.vditor-preview') as HTMLElement | null
  if (preview) {
    preview.style.backgroundColor = getComputedStyle(document.documentElement).getPropertyValue('--color-bg').trim()
  }
}

watch(() => props.modelValue, (newVal) => {
  if (isInternalChange) return
  if (vditor && newVal !== undefined) {
    const current = vditor.getValue()
    if (current !== newVal) {
      vditor.setValue(newVal || '')
    }
  }
})

onBeforeUnmount(() => {
  // 移除粘贴事件监听
  editorRef.value?.removeEventListener('paste', handlePaste)
  vditor?.destroy()
  vditor = null
})
</script>

<style>
/* === Fullscreen: below nav bar, hide edit top bar === */
.vditor--fullscreen {
  top: 56px !important;
  left: 0 !important;
  width: 100vw !important;
  height: calc(100vh - 56px) !important;
  z-index: 50 !important;
}
body:has(.vditor--fullscreen) .editor-topbar {
  display: none;
}
body:has(.vditor--fullscreen) .expand-preview-btn {
  display: none;
}
/* Disable link click behavior in editor content area ONLY */
/* Vditor IR mode renders links as <span class="vditor-ir__link">, not <a> */
.vditor-ir .vditor-ir__link,
.vditor-ir span[data-type="link"] {
  pointer-events: none !important;
  cursor: text !important;
}
</style>

<style scoped>
.editor-wrapper {
  height: 100%;
  display: flex;
  flex-direction: column;
  position: relative;
}

.editor-container {
  flex: 1;
  overflow: hidden;
}
</style>
