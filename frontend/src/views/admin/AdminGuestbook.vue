<template>
  <div class="admin-guestbook-page">
    <div class="page-header">
      <h2 class="page-title">💬 留言板</h2>
    </div>

    <div v-if="loading" class="loading-state"><div class="spinner"></div><span>加载中...</span></div>

    <div v-else-if="entries.length === 0" class="empty-state">暂无留言</div>

    <div v-else class="entries-list">
      <div
        v-for="entry in entries"
        :key="entry.id"
        class="entry-card"
        :style="{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border)' }"
      >
        <div class="entry-header">
          <div class="entry-user-info">
            <span class="entry-nickname" :style="{ color: 'var(--color-text)' }">
              {{ entry.nickname }}
              <template v-if="entry.username">（<span :style="{ color: 'var(--color-primary)' }">{{ entry.username }}</span>）</template>
            </span>
            <span v-if="entry.email" class="entry-email" :style="{ color: 'var(--color-text-muted)' }">{{ entry.email }}</span>
          </div>
          <div class="entry-meta">
            <span v-if="entry.is_replied" class="replied-badge">已回复</span>
            <span class="entry-time" :style="{ color: 'var(--color-text-muted)' }">{{ dayjs(entry.created_at).format('YYYY-MM-DD HH:mm') }}</span>
          </div>
        </div>
        <p class="entry-content" :style="{ color: 'var(--color-text-secondary)' }">{{ entry.content }}</p>
        <div v-if="entry.reply" class="entry-reply">
          <span class="reply-label">回复：</span>{{ entry.reply }}
          <span class="reply-time" :style="{ color: 'var(--color-text-muted)' }">{{ dayjs(entry.updated_at).format('YYYY-MM-DD HH:mm') }}</span>
        </div>
        <div class="entry-actions">
          <button class="action-btn" @click="openReply(entry.id)">回复</button>
          <button class="action-btn danger" @click="confirmDelete(entry)">删除</button>
        </div>
      </div>
    </div>

    <div v-if="totalPages > 1" class="pagination">
      <button :disabled="page <= 1" @click="page--; loadEntries()">上一页</button>
      <span>{{ page }} / {{ totalPages }}</span>
      <button :disabled="page >= totalPages" @click="page++; loadEntries()">下一页</button>
    </div>

    <!-- 回复弹窗 -->
    <div v-if="replyTargetId" class="modal-overlay" @click.self="replyTargetId = null">
      <div class="modal-box">
        <h3>回复留言</h3>
        <textarea
          v-model="replyText"
          rows="3"
          placeholder="输入回复内容..."
          class="w-full px-3 py-2 rounded-lg border text-sm outline-none resize-none mt-3"
          :style="{ backgroundColor: 'var(--color-bg)', borderColor: 'var(--color-border)', color: 'var(--color-text)' }"
        ></textarea>
        <div class="modal-actions mt-4">
          <button class="btn-secondary" @click="replyTargetId = null">取消</button>
          <button class="btn-primary" @click="doReply" :disabled="!replyText.trim()">回复</button>
        </div>
      </div>
    </div>

    <!-- 删除确认 -->
    <div v-if="deleteTarget" class="modal-overlay" @click.self="deleteTarget = null">
      <div class="modal-box">
        <h3>确认删除</h3>
        <p>确定要删除这条留言吗？</p>
        <div class="modal-actions">
          <button class="btn-secondary" @click="deleteTarget = null">取消</button>
          <button class="btn-danger" @click="doDelete">删除</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { fetchGuestbook, replyGuestbook, deleteGuestbook, type GuestbookEntry } from '@/api/guestbook'
import dayjs from 'dayjs'

const entries = ref<GuestbookEntry[]>([])
const loading = ref(true)
const page = ref(1)
const totalPages = ref(1)
const pageSize = 15

const replyTargetId = ref<number | null>(null)
const replyText = ref('')
const deleteTarget = ref<GuestbookEntry | null>(null)

async function loadEntries() {
  loading.value = true
  try {
    const resp = await fetchGuestbook({ page: page.value, page_size: pageSize })
    entries.value = resp.data.data
    totalPages.value = Math.max(1, Math.ceil(resp.data.pagination.total / pageSize))
  } catch { entries.value = [] }
  finally { loading.value = false }
}

function openReply(id: number) {
  replyTargetId.value = id
  replyText.value = ''
}

async function doReply() {
  if (!replyTargetId.value || !replyText.value.trim()) return
  try {
    await replyGuestbook(replyTargetId.value, replyText.value.trim())
    replyTargetId.value = null
    loadEntries()
  } catch { /* ignore */ }
}

function confirmDelete(entry: GuestbookEntry) {
  deleteTarget.value = entry
}

async function doDelete() {
  if (!deleteTarget.value) return
  try {
    await deleteGuestbook(deleteTarget.value.id)
    deleteTarget.value = null
    loadEntries()
  } catch { /* ignore */ }
}

onMounted(() => loadEntries())
</script>

<style scoped>
.admin-guestbook-page { padding: 0 0 24px; }
.page-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 24px; }
.page-title { font-size: 28px; font-weight: 700; color: var(--input-color); margin: 0; }
.entries-list { display: flex; flex-direction: column; gap: 12px; }
.entry-card { border: 1px solid; border-radius: 12px; padding: 16px; }
.entry-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 8px; }
.entry-user-info { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
.entry-nickname { font-size: 15px; font-weight: 600; }
.entry-email { font-size: 12px; }
.entry-meta { display: flex; align-items: center; gap: 8px; }
.replied-badge { font-size: 11px; padding: 1px 8px; border-radius: 10px; background: rgba(16,185,129,0.12); color: #34d399; }
.entry-time { font-size: 12px; }
.entry-content { font-size: 14px; line-height: 1.6; margin-bottom: 8px; }
.entry-reply { font-size: 13px; padding: 8px 12px; border-radius: 8px; background: rgba(79,70,229,0.06); color: var(--color-text-secondary); margin-bottom: 8px; display: flex; align-items: baseline; gap: 12px; }
.reply-time { font-size: 11px; flex-shrink: 0; }
.reply-label { font-weight: 600; color: var(--color-primary); }
.entry-actions { display: flex; gap: 8px; }
.action-btn { padding: 4px 14px; border-radius: 6px; font-size: 12px; border: none; cursor: pointer; background: var(--color-bg-hover); color: var(--color-text); }
.action-btn.danger { color: #ef4444; }
.action-btn.danger:hover { background: rgba(239,68,68,0.1); }
.action-btn:hover { background: var(--color-primary-bg); }

.modal-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 100; }
.modal-box { background: var(--color-bg-card); border-radius: 16px; padding: 24px; min-width: 380px; max-width: 500px; box-shadow: 0 20px 60px rgba(0,0,0,0.3); }
.modal-actions { display: flex; gap: 10px; justify-content: flex-end; }
.btn-primary, .btn-secondary, .btn-danger { padding: 8px 20px; border-radius: 8px; font-size: 14px; border: none; cursor: pointer; }
.btn-primary { background: var(--color-primary); color: #fff; }
.btn-secondary { background: var(--color-bg-hover); color: var(--color-text); }
.btn-danger { background: rgba(239,68,68,0.15); color: #ef4444; }
.loading-state, .empty-state { text-align: center; padding: 60px 0; color: var(--color-text-muted); }
.pagination { display: flex; justify-content: center; gap: 12px; margin-top: 24px; align-items: center; }
.pagination button { padding: 6px 16px; border-radius: 8px; border: 1px solid var(--color-border); background: var(--color-bg-card); color: var(--color-text); cursor: pointer; }
.pagination button:disabled { opacity: 0.4; cursor: default; }
</style>
