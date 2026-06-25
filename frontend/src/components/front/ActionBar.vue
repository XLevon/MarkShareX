<template>
  <div class="pt-6 border-t" :style="{ borderColor: 'var(--color-border)' }">
    <div class="flex items-center gap-4">
      <!-- Left: Prev / Next -->
      <div class="flex items-center gap-3 flex-1 min-w-0">
        <router-link
          v-if="prev"
          :to="`/post/${prev.slug}`"
          class="pn-link flex items-center gap-1.5 text-xs no-underline min-w-0"
          :style="{ color: 'var(--color-text-muted)' }"
        >
          <svg class="w-4 h-4 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"/></svg>
          <span class="text-xs flex-shrink-0">上一篇：</span>
          <span class="truncate">{{ prev.title }}</span>
        </router-link>
        <span v-if="prev && next" class="text-xs" :style="{ color: 'var(--color-text-muted)' }">|</span>
        <router-link
          v-if="next"
          :to="`/post/${next.slug}`"
          class="pn-link flex items-center gap-1.5 text-xs no-underline min-w-0"
          :style="{ color: 'var(--color-text-muted)' }"
        >
          <span class="text-xs flex-shrink-0">下一篇：</span>
          <span class="truncate">{{ next.title }}</span>
          <svg class="w-4 h-4 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/></svg>
        </router-link>
      </div>

      <!-- Right: Views + Like + Share -->
      <div class="flex items-center gap-2 flex-shrink-0">
        <!-- Views -->
        <span class="inline-flex items-center gap-1 text-xs" :style="{ color: 'var(--color-text-muted)' }">
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"/></svg>
          {{ views }}
        </span>
        <!-- Like -->
        <button
          v-if="isLoggedIn"
          @click="$emit('toggleLike')"
          class="inline-flex items-center gap-1 px-2 py-1 rounded text-xs transition-colors border-0 cursor-pointer"
          :style="liked
            ? { backgroundColor: 'rgba(239,68,68,0.12)', color: '#ef4444' }
            : { backgroundColor: 'var(--color-bg-hover)', color: 'var(--color-text-muted)' }"
          :disabled="likeLoading"
        >
          <svg class="w-3.5 h-3.5" :fill="liked ? 'currentColor' : 'none'" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z"/></svg>
          {{ likeCount }}
        </button>
        <span v-else class="inline-flex items-center gap-1 text-xs" :style="{ color: 'var(--color-text-muted)' }">
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z"/></svg>
          {{ likeCount }}
        </span>
        <!-- Share -->
        <div class="share-wrapper">
          <button
            @click="showShare = !showShare"
            class="inline-flex items-center gap-1 px-2 py-1 rounded text-xs transition-colors border-0 cursor-pointer"
            :style="{ backgroundColor: 'var(--color-bg-hover)', color: 'var(--color-text-muted)' }"
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.368 2.684 3 3 0 00-5.368-2.684z"/></svg>
            分享
          </button>
          <div v-if="showShare" :class="['share-dropdown', dropdownUp ? 'share-dropdown-up' : '']">
            <button @click="doShareWechat" class="share-item">💬 微信分享</button>
            <button @click="doCopyLink" class="share-item">{{ copySuccess ? '✅ 已复制' : '📋 复制链接' }}</button>
          </div>
        </div>
      </div>
    </div>

    <!-- Click-outside overlay -->
    <div v-if="showShare" class="fixed inset-0 z-10" @click="showShare = false"></div>

    <!-- QR Code Modal (Desktop only) -->
    <Teleport to="body">
      <div v-if="showQrModal" class="qr-overlay" @click.self="showQrModal = false">
        <div class="qr-modal">
          <h3 class="qr-title">微信扫码分享</h3>
          <p class="qr-hint">打开微信「扫一扫」扫描二维码，即可将文章分享到朋友圈或发送给朋友</p>
          <canvas ref="qrCanvas" class="qr-canvas"></canvas>
          <div class="qr-info">
            <p class="qr-post-title">{{ shareTitle }}</p>
          </div>
          <button class="qr-close-btn" @click="showQrModal = false">关闭</button>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from 'vue'
import QRCode from 'qrcode'

const props = defineProps<{
  prev: { id: number; title: string; slug: string } | null
  next: { id: number; title: string; slug: string } | null
  views: number
  likeCount: number
  liked: boolean
  likeLoading: boolean
  isLoggedIn: boolean
  shareTitle: string
  shareText: string
  dropdownUp?: boolean
}>()

defineEmits<{
  toggleLike: []
}>()

const showShare = ref(false)
const copySuccess = ref(false)
const showQrModal = ref(false)
const qrCanvas = ref<HTMLCanvasElement | null>(null)

// Detect mobile: touch support + narrow screen + mobile user-agent
const isMobile = ref(checkMobile())

function checkMobile(): boolean {
  if (typeof navigator === 'undefined') return false
  const ua = navigator.userAgent || ''
  // Check mobile user-agent OR touch device with small screen
  if (/Android|iPhone|iPad|iPod|webOS/i.test(ua)) return true
  if (navigator.maxTouchPoints > 0 && window.innerWidth < 768) return true
  return false
}

async function doShareWechat() {
  showShare.value = false
  if (isMobile.value) {
    // Mobile: use native share sheet (includes WeChat)
    if (navigator.share) {
      try {
        await navigator.share({
          title: props.shareTitle || '分享文章',
          text: props.shareText || props.shareTitle || '',
          url: window.location.href,
        })
      } catch { /* user cancelled */ }
    } else {
      doCopyLink()
    }
  } else {
    // Desktop: show QR code for scanning
    showQrModal.value = true
    await nextTick()
    if (qrCanvas.value) {
      QRCode.toCanvas(qrCanvas.value, window.location.href, {
        width: 220,
        margin: 1,
        color: { dark: '#1f2937', light: '#ffffff' },
      })
    }
  }
}

async function doCopyLink() {
  showShare.value = false
  try {
    await navigator.clipboard.writeText(window.location.href)
    copySuccess.value = true
    setTimeout(() => { copySuccess.value = false }, 2000)
  } catch {
    const input = document.createElement('input')
    input.value = window.location.href
    document.body.appendChild(input)
    input.select()
    document.execCommand('copy')
    document.body.removeChild(input)
    copySuccess.value = true
    setTimeout(() => { copySuccess.value = false }, 2000)
  }
}
</script>

<style scoped>
.pn-link {
  transition: color 0.15s ease, border-color 0.15s ease;
  border-bottom: 1px solid transparent;
}
.pn-link:hover {
  color: var(--color-primary) !important;
  border-bottom-color: var(--color-primary);
}

.share-wrapper {
  position: relative;
}
.share-dropdown {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 6px;
  min-width: 160px;
  background: #1a1a24;
  border: 1px solid rgba(255,255,255,0.08);
  border-radius: 10px;
  box-shadow: 0 8px 24px rgba(0,0,0,0.4);
  padding: 6px;
  z-index: 20;
}
.share-dropdown-up {
  top: auto;
  bottom: 100%;
  margin-top: 0;
  margin-bottom: 6px;
}
.share-item {
  display: block;
  width: 100%;
  padding: 8px 12px;
  border: none;
  background: transparent;
  color: #d1d5db;
  font-size: 13px;
  text-align: left;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s;
}
.share-item:hover {
  background: rgba(255,255,255,0.06);
  color: #f0f0f0;
}

/* QR Modal */
.qr-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  backdrop-filter: blur(4px);
}
.qr-modal {
  background: #ffffff;
  border-radius: 16px;
  padding: 32px;
  max-width: 360px;
  width: 90%;
  text-align: center;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}
.qr-title {
  margin: 0 0 8px;
  font-size: 18px;
  font-weight: 700;
  color: #1f2937;
}
.qr-hint {
  margin: 0 0 20px;
  font-size: 13px;
  color: #6b7280;
  line-height: 1.5;
}
.qr-canvas {
  display: block;
  margin: 0 auto 16px;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
}
.qr-info {
  margin-bottom: 20px;
  padding: 10px;
  background: #f9fafb;
  border-radius: 8px;
}
.qr-post-title {
  margin: 0;
  font-size: 13px;
  color: #374151;
  line-height: 1.4;
  word-break: break-word;
}
.qr-close-btn {
  display: inline-block;
  padding: 8px 24px;
  border: none;
  background: #4f46e5;
  color: #fff;
  font-size: 14px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.15s;
}
.qr-close-btn:hover {
  background: #4338ca;
}
</style>
