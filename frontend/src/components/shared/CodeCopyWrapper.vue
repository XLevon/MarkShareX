<template>
  <div ref="containerRef" class="code-copy-wrapper" v-html="html"></div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, nextTick } from 'vue'

const props = withDefaults(defineProps<{ html: string; copyEnabled?: boolean }>(), {
  copyEnabled: true,
})
const emit = defineEmits<{ 'copy-restricted': [] }>()
const containerRef = ref<HTMLElement>()

const COPY_ICON = `<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>`
const CHECK_ICON = `<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>`

function enhanceCodeBlocks() {
  if (!containerRef.value) return

  const blocks = containerRef.value.querySelectorAll('pre code')
  blocks.forEach((block) => {
    const pre = block.parentElement
    if (!pre) return
    const existingButton = pre.querySelector<HTMLButtonElement>('.code-copy-btn')
    if (existingButton) {
      existingButton.title = props.copyEnabled ? '复制代码' : '登录后复制'
      return
    }

    const btn = document.createElement('button')
    btn.className = 'code-copy-btn'
    btn.innerHTML = COPY_ICON
    btn.title = props.copyEnabled ? '复制代码' : '登录后复制'
    btn.onclick = async (e: Event) => {
      e.stopPropagation()
      if (!props.copyEnabled) {
        emit('copy-restricted')
        return
      }
      try {
        await navigator.clipboard.writeText(block.textContent || '')
        btn.innerHTML = CHECK_ICON
      } catch {
        const textarea = document.createElement('textarea')
        textarea.value = block.textContent || ''
        textarea.style.cssText = 'position:fixed;opacity:0'
        document.body.appendChild(textarea)
        textarea.select()
        document.execCommand('copy')
        document.body.removeChild(textarea)
        btn.innerHTML = CHECK_ICON
      }
      setTimeout(() => { btn.innerHTML = COPY_ICON }, 2000)
    }
    pre.style.position = 'relative'
    pre.appendChild(btn)
  })
}

onMounted(() => enhanceCodeBlocks())
watch(() => props.html, () => nextTick(() => enhanceCodeBlocks()))
watch(() => props.copyEnabled, () => nextTick(() => enhanceCodeBlocks()))
</script>

<style>
.code-copy-wrapper pre {
  position: relative;
}
.code-copy-btn {
  position: absolute;
  top: 8px;
  right: 8px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: 1px solid var(--color-border, #e5e7eb);
  border-radius: 4px;
  background: var(--color-bg, #ffffff);
  color: var(--color-text-muted, #9ca3af);
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.2s, color 0.2s;
  z-index: 1;
  padding: 0;
}
pre:hover .code-copy-btn {
  opacity: 1;
}
.code-copy-btn:hover {
  color: var(--color-primary, #6366f1);
  border-color: var(--color-primary, #6366f1);
}
</style>
