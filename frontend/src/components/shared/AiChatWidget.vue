<template>
  <div class="ai-chat-widget">
    <!-- 悬浮按钮 -->
    <button
      v-if="!open"
      class="ai-chat-fab"
      @click="open = true"
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
        <div v-if="open" class="ai-chat-panel">
          <div class="ai-chat-header">
            <div class="ai-chat-title-bar">
              <button class="ai-chat-back" v-if="showSessions" @click="showSessions = false">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 18l-6-6 6-6"/></svg>
              </button>
              <span v-if="showSessions">会话列表</span>
              <span v-else>🤖 AI 助手</span>
              <span class="session-name" v-if="!showSessions && sessionId && isAdmin">#{{ sessionId }}</span>
            </div>
            <div class="ai-chat-header-actions">
              <template v-if="isAdmin">
                <button class="ai-chat-btn-icon" @click="toggleSessions" title="会话列表">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01"/></svg>
                </button>
                <button class="ai-chat-btn-icon" @click="newSession" title="新会话">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 5v14M5 12h14"/></svg>
                </button>
              </template>
              <button class="ai-chat-close" @click="open = false">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
              </button>
            </div>
          </div>

          <!-- 会话列表（仅管理后台） -->
          <div v-if="showSessions && isAdmin" class="ai-chat-sessions">
            <div v-if="sessions.length === 0" class="ai-chat-empty">暂无会话</div>
            <div
              v-for="s in sessions" :key="s.id"
              :class="['ai-session-item', { active: s.id === sessionId }]"
              @click="switchSession(s.id)"
            >
              <div class="ai-session-title">{{ s.title }}</div>
              <div class="ai-session-meta">{{ s.msg_count }} 条消息</div>
              <button class="ai-session-del" @click.stop="deleteSessionHandler(s.id)" title="删除">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
              </button>
            </div>
          </div>

          <!-- 消息区域 -->
          <div v-else class="ai-chat-body">
            <div class="ai-chat-messages" ref="msgContainer">
              <div v-if="messages.length === 0" class="ai-chat-empty">
                <div>我是 AI 助手，可以帮你：</div>
                <div class="ai-chat-hints">
                  <div class="ai-chat-hint" @click="sendHint('网站有哪些内容？')">📚 站内导航</div>
                  <div class="ai-chat-hint" @click="sendHint('今天有什么热门资讯？')">📰 今日资讯</div>
                  <div class="ai-chat-hint" @click="sendHint('如何使用 MarkShareX？')">❓ 使用帮助</div>
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
                v-model="input"
                @keydown.enter="send"
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
import { ref, nextTick, watch, onMounted, computed } from 'vue'
import { sendChatMessage, fetchSessions, getSession, deleteSession, type ChatMessage, type ChatSession } from '@/api/ai'
import { marked } from 'marked'

const props = withDefaults(defineProps<{ mode?: 'front' | 'admin' }>(), { mode: 'admin' })
const isAdmin = computed(() => props.mode === 'admin')

const open = ref(false)
const input = ref('')
const loading = ref(false)
const messages = ref<ChatMessage[]>([])
const msgContainer = ref<HTMLElement | null>(null)
const sessionId = ref<number | null>(null)
const sessions = ref<ChatSession[]>([])
const showSessions = ref(false)

async function loadSessions() {
  if (!isAdmin.value) return
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
  return marked.parse(content)
}

async function sendHint(text: string) {
  input.value = text
  await send()
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
      // 前台模式不传 session_id，每次都是新会话
      session_id: isAdmin.value ? (sessionId.value ?? undefined) : undefined,
    })
    const data = resp.data.data
    messages.value.push({ role: 'assistant', content: data.reply })
    // 后台模式更新 session ID
    if (isAdmin.value && !sessionId.value) {
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
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 4px 16px rgba(99, 102, 241, 0.4);
  transition: transform 0.2s, box-shadow 0.2s;
}
.ai-chat-fab:hover { transform: scale(1.08); box-shadow: 0 6px 24px rgba(99, 102, 241, 0.55); }

.ai-chat-panel {
  position: fixed;
  bottom: 24px;
  right: 24px;
  z-index: 1000;
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
