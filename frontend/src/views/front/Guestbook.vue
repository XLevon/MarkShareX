<template>
  <div class="max-w-4xl mx-auto px-4 py-12">
    <!-- Page Header -->
    <div class="mb-10 flex items-end justify-between">
      <div>
        <h1 class="text-3xl font-bold mb-2" :style="{ color: 'var(--color-text)' }">留言板</h1>
        <p class="text-sm" :style="{ color: 'var(--color-text-muted)' }">留下你的足迹，分享你的想法</p>
      </div>
      <button
        @click="openForm"
        class="px-5 py-2.5 rounded-xl text-sm font-medium text-white transition-all hover:opacity-90 active:scale-95"
        :style="{ backgroundColor: 'var(--color-primary)' }"
      >写留言</button>
    </div>

    <!-- 搜索 -->
    <div class="mb-6">
      <input
        v-model="search"
        type="text"
        placeholder="搜索留言..."
        class="w-full max-w-xs px-4 py-2 rounded-lg border text-sm outline-none"
        :style="{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border)', color: 'var(--color-text)' }"
        @keyup.enter="searchEntries"
      />
    </div>

    <!-- 留言列表 -->
    <div v-if="loading" class="text-center py-8" :style="{ color: 'var(--color-text-muted)' }">加载中...</div>

    <div v-else-if="entries.length === 0" class="text-center py-8" :style="{ color: 'var(--color-text-muted)' }">
      暂无留言，来坐沙发吧 🛋️
    </div>

    <div v-else class="space-y-4">
      <div
        v-for="entry in entries"
        :key="entry.id"
        class="rounded-xl border p-4"
        :style="{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border)' }"
      >
        <div class="flex items-center justify-between mb-2">
          <span class="text-sm font-medium" :style="{ color: 'var(--color-text)' }">{{ entry.nickname }}</span>
          <span class="text-xs" :style="{ color: 'var(--color-text-muted)' }">{{ dayjs(entry.created_at).format('YYYY-MM-DD HH:mm') }}</span>
        </div>
        <p class="text-sm leading-relaxed" :style="{ color: 'var(--color-text-secondary)' }">{{ entry.content }}</p>
        <!-- 站长回复 -->
        <div v-if="entry.reply" class="mt-3 pl-4 border-l-2 rounded" :style="{ borderColor: 'var(--color-primary)' }">
          <div class="flex items-center gap-2 mb-1">
            <span class="text-xs font-medium" :style="{ color: 'var(--color-primary)' }">站长回复</span>
          </div>
          <p class="text-sm" :style="{ color: 'var(--color-text-secondary)' }">{{ entry.reply }}</p>
        </div>
      </div>
    </div>

    <!-- 分页 -->
    <div v-if="totalPages > 1" class="flex justify-center gap-3 mt-8">
      <button :disabled="page <= 1" @click="page--; loadEntries()" class="px-4 py-1.5 rounded-lg border text-sm" :style="{ borderColor: 'var(--color-border)', color: 'var(--color-text)' }">上一页</button>
      <span class="text-sm self-center" :style="{ color: 'var(--color-text-muted)' }">{{ page }} / {{ totalPages }}</span>
      <button :disabled="page >= totalPages" @click="page++; loadEntries()" class="px-4 py-1.5 rounded-lg border text-sm" :style="{ borderColor: 'var(--color-border)', color: 'var(--color-text)' }">下一页</button>
    </div>

    <!-- 留言弹窗（仅创建表单，不显示列表） -->
    <div v-if="showForm" class="fixed inset-0 z-50 flex items-center justify-center" style="background: rgba(0,0,0,0.5);">
      <div class="rounded-2xl p-6 w-full max-w-md mx-4 shadow-2xl" :style="{ backgroundColor: 'var(--color-bg-card)' }">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-lg font-bold" :style="{ color: 'var(--color-text)' }">💬 写留言</h3>
          <button @click="showForm = false" class="w-8 h-8 rounded-full flex items-center justify-center hover:opacity-70" :style="{ color: 'var(--color-text-muted)' }">✕</button>
        </div>
        <div class="space-y-4">
          <!-- 昵称 -->
          <div>
            <input
              v-model="formNickname"
              type="text"
              placeholder="你的昵称"
              maxlength="30"
              :disabled="!!authStore.user"
              class="w-full px-4 py-2.5 rounded-lg border text-sm outline-none transition-colors disabled:opacity-60 disabled:cursor-not-allowed"
              :style="{ backgroundColor: 'var(--color-bg)', borderColor: 'var(--color-border)', color: 'var(--color-text)' }"
            />
            <span v-if="authStore.user" class="text-xs mt-1 inline-block" :style="{ color: 'var(--color-text-muted)' }">已登录，使用账号昵称</span>
          </div>
          <!-- 邮箱 -->
          <div>
            <input
              v-model="formEmail"
              type="email"
              placeholder="你的邮箱"
              maxlength="100"
              class="w-full px-4 py-2.5 rounded-lg border text-sm outline-none transition-colors"
              :class="formEmail && !formEmailValid ? 'border-red-400' : ''"
              :style="{ backgroundColor: 'var(--color-bg)', borderColor: formEmail && !formEmailValid ? '#f87171' : 'var(--color-border)', color: 'var(--color-text)' }"
              @blur="formEmailTouched = true"
            />
            <span v-if="formEmailTouched && formEmail && !formEmailValid" class="text-xs mt-1 inline-block" style="color: #f87171;">请输入有效的邮箱地址</span>
          </div>
          <textarea
            v-model="formContent"
            placeholder="写下你的留言或建议..."
            rows="6"
            maxlength="500"
            class="w-full px-4 py-2.5 rounded-lg border text-sm outline-none transition-colors resize-none"
            :style="{ backgroundColor: 'var(--color-bg)', borderColor: 'var(--color-border)', color: 'var(--color-text)' }"
          ></textarea>
          <div class="flex items-center justify-between">
            <span class="text-xs" :style="{ color: 'var(--color-text-muted)' }">{{ formContent.length }}/500</span>
            <button
              @click="submitForm"
              :disabled="!formCanSubmit || submitting"
              class="px-5 py-2 rounded-lg text-white text-sm font-medium transition-all disabled:opacity-40"
              :style="{ backgroundColor: 'var(--color-primary)' }"
            >
              {{ submitting ? '提交中...' : '提交留言' }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { fetchGuestbook, createGuestbook, type GuestbookEntry } from '@/api/guestbook'
import { useAuthStore } from '@/stores/auth'
import dayjs from 'dayjs'

const GUESTBOOK_CACHE_KEY = 'marksharex_guestbook'

const route = useRoute()

const authStore = useAuthStore()
const entries = ref<GuestbookEntry[]>([])
const search = ref('')
const submitting = ref(false)
const loading = ref(true)
const page = ref(1)
const totalPages = ref(1)
const pageSize = 15

// ── 弹窗表单 ──
const showForm = ref(false)
const formNickname = ref('')
const formEmail = ref('')
const formContent = ref('')
const formEmailTouched = ref(false)
const formEmailValid = computed(() => /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(formEmail.value))
const formCanSubmit = computed(() => formNickname.value.trim() && formEmailValid.value && formContent.value.trim())

// 打开弹窗时填充缓存/登录信息
function openForm() {
  if (authStore.user) {
    formNickname.value = authStore.user.display_name || authStore.user.username || ''
    formEmail.value = authStore.user.email || ''
  } else {
    try {
      const raw = localStorage.getItem(GUESTBOOK_CACHE_KEY)
      if (raw) {
        const cached = JSON.parse(raw)
        formNickname.value = cached.nickname || ''
        formEmail.value = cached.email || ''
      }
    } catch {}
  }
  formContent.value = ''
  formEmailTouched.value = false
  showForm.value = true
}

// 从缓存恢复
function loadCache() {
  try {
    const raw = localStorage.getItem(GUESTBOOK_CACHE_KEY)
    if (raw) {
      const cached = JSON.parse(raw)
      formNickname.value = cached.nickname || ''
      formEmail.value = cached.email || ''
    }
  } catch {}
}

function saveCache() {
  if (!authStore.user) {
    try {
      localStorage.setItem(GUESTBOOK_CACHE_KEY, JSON.stringify({
        nickname: formNickname.value.trim(),
        email: formEmail.value.trim(),
      }))
    } catch {}
  }
}

onMounted(() => {
  loadCache()
  loadEntries()
  if (route.query.write === '1') {
    openForm()
  }
})

async function loadEntries() {
  loading.value = true
  try {
    const params: any = { page: page.value, page_size: pageSize }
    if (search.value.trim()) params.search = search.value.trim()
    const resp = await fetchGuestbook(params)
    entries.value = resp.data.data
    totalPages.value = Math.max(1, Math.ceil(resp.data.pagination.total / pageSize))
  } catch { entries.value = [] }
  finally { loading.value = false }
}

function searchEntries() {
  page.value = 1
  loadEntries()
}

async function submitForm() {
  if (!formCanSubmit.value) return
  submitting.value = true
  try {
    await createGuestbook({
      nickname: formNickname.value.trim(),
      email: formEmail.value.trim(),
      content: formContent.value.trim(),
    })
    saveCache()
    showForm.value = false
    page.value = 1
    loadEntries()
  } catch { /* ignore */ }
  finally { submitting.value = false }
}
</script>
