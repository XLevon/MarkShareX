<template>
  <div class="ai-chat-widget">
    <!-- 悬浮按钮 -->
    <button
      v-if="!open"
      class="ai-chat-fab"
      :style="fabDragTransform"
      @mousedown="(e: MouseEvent) => startFabDrag(e)"
      @touchstart="(e: TouchEvent) => startFabDrag(e)"
      @click="fabDragged ? (fabDragged = false) : (open = true)"
      title="AI 助手"
    >
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
        <path d="M8 9h8M8 13h6"/>
      </svg>
    </button>

    <!-- 展开面板 -->
    <Teleport to="body">
      <Transition name="chat-slide">
        <div v-if="open" class="ai-chat-panel" :style="panelDragTransform">
          <div class="ai-chat-header" @mousedown="startPanelDrag" @touchstart="startPanelDrag">
            <div class="ai-chat-title-bar">
              <button class="ai-chat-back" v-if="showSessions" @click="showSessions = false">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 18l-6-6 6-6"/></svg>
              </button>
              <span v-if="showSessions">会话列表</span>
              <span v-else>🤖 AI 助手</span>
              <span class="session-name" v-if="!showSessions && sessionId">#{{ sessionId }}</span>
            </div>
            <div class="ai-chat-header-actions">
              <button class="ai-chat-btn-icon" @click="toggleSessions" title="会话列表">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01"/></svg>
              </button>
              <button class="ai-chat-btn-icon" @click="newSession" title="新会话">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 5v14M5 12h14"/></svg>
              </button>
              <button class="ai-chat-close" @click="open = false">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
              </button>
            </div>
          </div>

          <!-- 会话列表 -->
          <div v-if="showSessions" class="ai-chat-sessions">
            <div v-if="sessions.length === 0" class="ai-chat-empty">暂无会话</div>
            <div
              v-for="s in sessions" :key="s.id"
              :class="['ai-session-item', { active: s.id === sessionId }]"
              @click="switchSession(s.id)"
            >
              <div class="ai-session-title">
                <template v-if="s.user_display_name">【{{ s.user_display_name }}】</template>{{ s.title }}（{{ s.msg_count }}条）
              </div>
              <div class="ai-session-meta">{{ formatDate(s.created_at) }}</div>
              <button class="ai-session-del" @click.stop="deleteSessionHandler(s.id)" title="删除">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
              </button>
            </div>
          </div>

          <!-- 消息区域 -->
          <div v-else class="ai-chat-body">
            <div class="ai-chat-messages" ref="msgContainer" @click="handleMsgClick">
              <div v-if="messages.length === 0" class="ai-chat-empty">
                <div>我是 AI 助手，可以帮你：</div>
                <div class="ai-chat-hints">
                  <div class="ai-chat-hint" @click="sendHint('系统怎么使用？')">❓ 使用帮助</div>
                  <div class="ai-chat-hint" @click="sendHint('搜索站内资源')">📚 站内搜索</div>
                </div>
              </div>
              <div v-for="(msg, i) in messages" :key="i" :class="['ai-chat-msg', msg.role]">
                <div class="ai-chat-msg-content" v-html="renderMsg(msg.content)"></div>
              </div>
              <div v-if="loading" class="ai-chat-msg assistant">
                <div class="ai-chat-typing">思考中...</div>
              </div>
            </div>
            <div class="ai-chat-input">
              <input
                ref="inputField"
                v-model="input"
                @keydown.enter="onInputEnter"
                placeholder="输入消息..."
                :disabled="loading"
                class="ai-chat-input-field"
              />
              <button @click="send" :disabled="loading || !input.trim()" class="ai-chat-send">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M2 21l21-9L2 3v7l15 2-15 2v7z"/></svg>
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, watch, onMounted, onUnmounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { sendChatMessage, fetchSessions, getSession, deleteSession, type ChatMessage, type ChatSession } from '@/api/ai'
import { marked } from 'marked'
import { renderMarkdown } from '@/utils/renderMarkdown'
import { useAuthStore } from '@/stores/auth'

function formatDate(iso: string) {
  const d = new Date(iso)
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}${pad(d.getMonth() + 1)}${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`
}

const props = withDefaults(defineProps<{ mode?: 'front' | 'admin' }>(), { mode: 'admin' })
const isAdmin = computed(() => props.mode === 'admin')
const router = useRouter()
const authStore = useAuthStore()

const open = ref(false)
const input = ref('')
const loading = ref(false)
const messages = ref<ChatMessage[]>([])
const msgContainer = ref<HTMLElement | null>(null)
const inputField = ref<HTMLInputElement | null>(null)
const sessionId = ref<number | null>(null)
const sessions = ref<ChatSession[]>([])
const showSessions = ref(false)

// ── Drag state ──
const fabDrag = ref({ x: 0, y: 0 })
const panelDrag = ref({ x: 0, y: 0 })
const fabDragged = ref(false)
const panelDragging = ref<{ sx: number; sy: number; ox: number; oy: number } | null>(null)
const fabDragging = ref<{ sx: number; sy: number; ox: number; oy: number } | null>(null)

const fabDragTransform = computed(() =>
  fabDrag.value.x || fabDrag.value.y
    ? `transform: translate(${fabDrag.value.x}px, ${fabDrag.value.y}px)`
    : ''
)
const panelDragTransform = computed(() =>
  panelDrag.value.x || panelDrag.value.y
    ? `transform: translate(${panelDrag.value.x}px, ${panelDrag.value.y}px)`
    : ''
)

function startFabDrag(e: MouseEvent | TouchEvent) {
  const pt = 'touches' in e ? e.touches[0] : e
  fabDragging.value = { sx: pt.clientX, sy: pt.clientY, ox: fabDrag.value.x, oy: fabDrag.value.y }
  fabDragged.value = false
  document.addEventListener('mousemove', onFabDrag)
  document.addEventListener('mouseup', stopFabDrag)
  document.addEventListener('touchmove', onFabDrag, { passive: false })
  document.addEventListener('touchend', stopFabDrag)
}
function onFabDrag(e: MouseEvent | TouchEvent) {
  if (!fabDragging.value) return
  e.preventDefault()
  const pt = 'touches' in e ? e.touches[0] : e
  const dx = pt.clientX - fabDragging.value.sx
  const dy = pt.clientY - fabDragging.value.sy
  if (Math.abs(dx) > 3 || Math.abs(dy) > 3) fabDragged.value = true
  fabDrag.value = { x: fabDragging.value.ox + dx, y: fabDragging.value.oy + dy }
}
function stopFabDrag() {
  fabDragging.value = null
  document.removeEventListener('mousemove', onFabDrag)
  document.removeEventListener('mouseup', stopFabDrag)
  document.removeEventListener('touchmove', onFabDrag)
  document.removeEventListener('touchend', stopFabDrag)
}

function startPanelDrag(e: MouseEvent | TouchEvent) {
  const target = e.target as HTMLElement
  if (target.closest('button')) return  // 不拦截按钮点击
  const pt = 'touches' in e ? e.touches[0] : e
  panelDragging.value = { sx: pt.clientX, sy: pt.clientY, ox: panelDrag.value.x, oy: panelDrag.value.y }
  document.addEventListener('mousemove', onPanelDrag)
  document.addEventListener('mouseup', stopPanelDrag)
  document.addEventListener('touchmove', onPanelDrag, { passive: false })
  document.addEventListener('touchend', stopPanelDrag)
}
function onPanelDrag(e: MouseEvent | TouchEvent) {
  if (!panelDragging.value) return
  e.preventDefault()
  const pt = 'touches' in e ? e.touches[0] : e
  const dx = pt.clientX - panelDragging.value.sx
  const dy = pt.clientY - panelDragging.value.sy
  panelDrag.value = { x: panelDragging.value.ox + dx, y: panelDragging.value.oy + dy }
}
function stopPanelDrag() {
  panelDragging.value = null
  document.removeEventListener('mousemove', onPanelDrag)
  document.removeEventListener('mouseup', stopPanelDrag)
  document.removeEventListener('touchmove', onPanelDrag)
  document.removeEventListener('touchend', stopPanelDrag)
}

// Reset panel position when open changes
watch(open, (val) => {
  if (!val) panelDrag.value = { x: 0, y: 0 }
  // Lock body scroll when panel is open
  document.body.style.overflow = val ? 'hidden' : ''
})

// Ensure scroll lock is released on unmount
onUnmounted(() => {
  document.body.style.overflow = ''
})

async function loadSessions() {
  try {
    const r = await fetchSessions()
    sessions.value = r.data.data || []
  } catch {}
}

async function switchSession(sid: number) {
  sessionId.value = sid
  showSessions.value = false
  messages.value = []
  loading.value = true
  try {
    const resp = await getSession(sid)
    const detail = resp.data.data
    messages.value = detail.messages.map(m => ({
      role: m.role,
      content: m.content,
    }))
    await nextTick()
    scrollBottom()
  } catch {
    messages.value = []
  } finally {
    loading.value = false
  }
}

async function deleteSessionHandler(sid: number) {
  if (!confirm('确定删除此会话？')) return
  try {
    await deleteSession(sid)
    if (sessionId.value === sid) {
      sessionId.value = null
      messages.value = []
    }
    loadSessions()
  } catch {}
}

function toggleSessions() {
  showSessions.value = !showSessions.value
  if (showSessions.value) loadSessions()
}

async function newSession() {
  sessionId.value = null
  messages.value = []
  showSessions.value = false
}

function renderMsg(content: string) {
  return renderMarkdown(content)
}

/** 拦截消息中的链接点击：内部链接用 router.push，外部链接正常跳转 */
function handleMsgClick(e: MouseEvent) {
  const target = e.target as HTMLElement
  const link = target.closest('a')
  if (!link) return
  const href = link.getAttribute('href')
  if (!href) return

  // 锚点 / 新标签 → 浏览器默认行为
  if (href.startsWith('#') || link.getAttribute('target') === '_blank') {
    return
  }

  // 完整 URL → 判断是否本站
  if (href.startsWith('http://') || href.startsWith('https://')) {
    try {
      const url = new URL(href)
      if (url.host === window.location.host) {
        // 本站完整 URL → 提取路径用 router.push
        e.preventDefault()
        router.push(url.pathname + url.search)
        return
      }
    } catch {}
    // 外部链接 → 浏览器默认行为
    return
  }

  // 内部相对路径 → 路由跳转
  e.preventDefault()
  router.push(href)
}

async function sendHint(text: string) {
  input.value = text
  await send()
}

/** Enter 发送，过滤 IME 组合输入中的 Enter */
function onInputEnter(e: KeyboardEvent) {
  if (e.isComposing) return  // IME 组合中，不发送
  send()
}

async function send() {
  const text = input.value.trim()
  if (!text || loading.value) return
  input.value = ''
  messages.value.push({ role: 'user', content: text })
  loading.value = true
  await nextTick()
  scrollBottom()

  try {
    const resp = await sendChatMessage({
      message: text,
      history: [],
      session_id: sessionId.value ?? undefined,
      in_admin: isAdmin.value,
    })
    const data = resp.data.data
    messages.value.push({ role: 'assistant', content: data.reply })
    // 更新 session ID
    if (!sessionId.value) {
      sessionId.value = data.session_id
      loadSessions()
    }
  } catch (err: any) {
    const status = err?.response?.status
    if (status === 401) {
      messages.value.push({ role: 'assistant', content: '请先<a href="/login">登录</a>后使用 AI 助手。' })
    } else {
      const msg = err?.response?.data?.error || err?.message || '未知错误'
      messages.value.push({ role: 'assistant', content: `抱歉，请求失败：${msg}` })
    }
  } finally {
    loading.value = false
    await nextTick()
    scrollBottom()
    inputField.value?.focus()
  }
}

function scrollBottom() {
  if (msgContainer.value) {
    msgContainer.value.scrollTop = msgContainer.value.scrollHeight
  }
}

watch(open, async (val) => {
  if (val) {
    await nextTick()
    scrollBottom()
  }
})

// 登出/登录时重置会话状态
watch(() => authStore.token, (newToken) => {
  if (!newToken) {
    // 登出：清空会话和消息
    sessionId.value = null
    messages.value = []
    sessions.value = []
  } else {
    // 登录：重新加载会话列表
    loadSessions()
  }
})

// 页面跳转后保持输入框焦点
watch(() => router.currentRoute.value.path, () => {
  nextTick(() => {
    if (open.value) inputField.value?.focus()
  })
})

onMounted(() => {
  loadSessions()
})
</script>

<style scoped>
.ai-chat-fab {
  position: fixed;
  bottom: 24px;
  right: 24px;
  z-index: 999;
  width: 52px;
  height: 52px;
  border-radius: 50%;
  border: none;
  background: var(--color-primary, #6366f1);
  color: #fff;
  cursor: grab;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 4px 16px rgba(99, 102, 241, 0.4);
}
.ai-chat-fab:hover { box-shadow: 0 6px 24px rgba(99, 102, 241, 0.55); }

.ai-chat-panel {
  position: fixed;
  bottom: 24px;
  right: 24px;
  z-index: 1150;
  width: 420px;
  max-width: calc(100vw - 32px);
  height: 580px;
  max-height: calc(100vh - 100px);
  border-radius: 16px;
  display: flex;
  flex-direction: column;
  box-shadow: 0 12px 48px rgba(0, 0, 0, 0.2);
  overflow: hidden;
  background: var(--color-bg-card, #fff);
  border: 1px solid var(--color-border, #e5e7eb);
}
.ai-chat-fab:hover { box-shadow: 0 6px 24px rgba(99, 102, 241, 0.55); }

.ai-chat-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  font-weight: 600;
  font-size: 15px;
  border-bottom: 1px solid var(--color-border, #e5e7eb);
  background: var(--color-bg-card, #fff);
  color: var(--color-text, #1f2937);
  cursor: grab;
  user-select: none;
}
.ai-chat-title-bar { display: flex; align-items: center; gap: 8px; }
.session-name { font-weight: 400; font-size: 12px; color: var(--color-text-muted, #9ca3af); }
.ai-chat-header-actions { display: flex; align-items: center; gap: 4px; }

.ai-chat-back, .ai-chat-btn-icon {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--color-text-muted, #9ca3af);
  padding: 4px;
  border-radius: 6px;
  display: flex;
}
.ai-chat-btn-icon:hover, .ai-chat-back:hover {
  background: var(--color-bg-hover, #f3f4f6);
  color: var(--color-text, #1f2937);
}
.ai-chat-close {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--color-text-muted, #9ca3af);
  padding: 4px;
  border-radius: 6px;
  display: flex;
}
.ai-chat-close:hover { background: #fee2e2; color: #ef4444; }

.ai-chat-body { flex: 1; display: flex; flex-direction: column; overflow: hidden; }

.ai-chat-messages {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.ai-chat-empty {
  text-align: center;
  color: var(--color-text-muted, #9ca3af);
  font-size: 14px;
  margin-top: 20px;
  line-height: 1.6;
}
.ai-chat-hints { display: flex; gap: 8px; justify-content: center; margin-top: 16px; flex-wrap: wrap; }
.ai-chat-hint {
  padding: 8px 14px;
  border-radius: 20px;
  border: 1px solid var(--color-border, #e5e7eb);
  cursor: pointer;
  font-size: 13px;
  transition: all 0.15s;
}
.ai-chat-hint:hover { background: var(--color-primary, #6366f1); color: #fff; border-color: var(--color-primary, #6366f1); }

.ai-chat-msg { max-width: 85%; padding: 10px 14px; border-radius: 14px; font-size: 14px; line-height: 1.6; word-break: break-word; }
.ai-chat-msg.user { align-self: flex-end; background: var(--color-primary, #6366f1); color: #fff; border-bottom-right-radius: 6px; }
.ai-chat-msg.assistant, .ai-chat-msg.system { align-self: flex-start; background: var(--color-bg-hover, #f3f4f6); color: var(--color-text, #1f2937); border-bottom-left-radius: 6px; }
.ai-chat-typing { color: var(--color-text-muted, #9ca3af); font-style: italic; font-size: 13px; }

/* 消息内超链接样式 */
.ai-chat-msg-content :deep(a) {
  color: #6366f1;
  font-weight: 600;
  text-decoration: underline;
  text-underline-offset: 3px;
  padding: 2px 4px;
  border-radius: 4px;
  transition: all 0.15s;
}
.ai-chat-msg-content :deep(a:hover) {
  background: #eef2ff;
  color: #4f46e5;
}
/* 用户消息气泡内的链接反色 */
.ai-chat-msg.user .ai-chat-msg-content :deep(a) {
  color: #c7d2fe;
  text-decoration-color: #818cf8;
}
.ai-chat-msg.user .ai-chat-msg-content :deep(a:hover) {
  background: rgba(255,255,255,0.15);
  color: #fff;
}

.ai-chat-sessions { flex: 1; overflow-y: auto; padding: 8px; }
.ai-session-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: 10px;
  cursor: pointer;
  transition: background 0.15s;
}
.ai-session-item:hover, .ai-session-item.active { background: var(--color-bg-hover, #f3f4f6); }
.ai-session-title { flex: 1; font-size: 14px; font-weight: 500; color: var(--color-text, #1f2937); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.ai-session-meta { font-size: 12px; color: var(--color-text-muted, #9ca3af); white-space: nowrap; }
.ai-session-del { background: none; border: none; cursor: pointer; color: var(--color-text-muted, #9ca3af); padding: 2px; border-radius: 4px; display: flex; opacity: 0; transition: opacity 0.15s; }
.ai-session-item:hover .ai-session-del { opacity: 1; }
.ai-session-del:hover { background: #fee2e2; color: #ef4444; }

.ai-chat-input {
  display: flex;
  gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid var(--color-border, #e5e7eb);
}
.ai-chat-input-field {
  flex: 1;
  padding: 10px 14px;
  border-radius: 10px;
  border: 1px solid var(--color-border, #d1d5db);
  font-size: 14px;
  outline: none;
  background: var(--color-bg, #fff);
  color: var(--color-text, #1f2937);
}
.ai-chat-input-field:focus { border-color: var(--color-primary, #6366f1); }
.ai-chat-send {
  width: 40px; height: 40px; border-radius: 10px; border: none;
  background: var(--color-primary, #6366f1); color: #fff;
  cursor: pointer; display: flex; align-items: center; justify-content: center; flex-shrink: 0;
}
.ai-chat-send:disabled { opacity: 0.5; cursor: not-allowed; }

.chat-slide-enter-active { transition: all 0.25s ease-out; }
.chat-slide-leave-active { transition: all 0.2s ease-in; }
.chat-slide-enter-from { opacity: 0; transform: translateY(20px) scale(0.95); }
.chat-slide-leave-to { opacity: 0; transform: translateY(10px) scale(0.98); }
</style>
