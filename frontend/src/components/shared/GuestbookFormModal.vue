<template>
  <div v-if="visible" class="fixed inset-0 z-50 flex items-center justify-center" style="background: rgba(0,0,0,0.5);">
    <div class="rounded-2xl p-6 w-full max-w-md mx-4 shadow-2xl" :style="{ backgroundColor: 'var(--color-bg-card)' }">
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-lg font-bold" :style="{ color: 'var(--color-text)' }">💬 写留言</h3>
        <button @click="$emit('close')" class="w-8 h-8 rounded-full flex items-center justify-center hover:opacity-70" :style="{ color: 'var(--color-text-muted)' }">✕</button>
      </div>
      <div class="space-y-4">
        <!-- 昵称 -->
        <div>
          <input
            v-model="nickname"
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
            v-model="email"
            type="email"
            placeholder="你的邮箱"
            maxlength="100"
            class="w-full px-4 py-2.5 rounded-lg border text-sm outline-none transition-colors"
            :class="email && !emailValid ? 'border-red-400' : ''"
            :style="{ backgroundColor: 'var(--color-bg)', borderColor: email && !emailValid ? '#f87171' : 'var(--color-border)', color: 'var(--color-text)' }"
            @blur="emailTouched = true"
          />
          <span v-if="emailTouched && email && !emailValid" class="text-xs mt-1 inline-block" style="color: #f87171;">请输入有效的邮箱地址</span>
        </div>
        <textarea
          v-model="content"
          placeholder="写下你的留言或建议..."
          rows="6"
          maxlength="500"
          class="w-full px-4 py-2.5 rounded-lg border text-sm outline-none transition-colors resize-none"
          :style="{ backgroundColor: 'var(--color-bg)', borderColor: 'var(--color-border)', color: 'var(--color-text)' }"
        ></textarea>
        <div class="flex items-center justify-between">
          <span class="text-xs" :style="{ color: 'var(--color-text-muted)' }">{{ content.length }}/500</span>
          <button
            @click="submit"
            :disabled="!canSubmit || submitting"
            class="px-5 py-2 rounded-lg text-white text-sm font-medium transition-all disabled:opacity-40"
            :style="{ backgroundColor: 'var(--color-primary)' }"
          >
            {{ submitting ? '提交中...' : '提交留言' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { createGuestbook } from '@/api/guestbook'
import { useAuthStore } from '@/stores/auth'

const GUESTBOOK_CACHE_KEY = 'marksharex_guestbook'

const props = defineProps<{ visible: boolean }>()
const emit = defineEmits<{ close: [] }>()

const authStore = useAuthStore()
const nickname = ref('')
const email = ref('')
const content = ref('')
const submitting = ref(false)
const emailTouched = ref(false)
const emailValid = computed(() => /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email.value))
const canSubmit = computed(() => nickname.value.trim() && emailValid.value && content.value.trim())

function initForm() {
  if (authStore.user) {
    nickname.value = authStore.user.display_name || authStore.user.username || ''
    email.value = authStore.user.email || ''
  } else {
    try {
      const raw = localStorage.getItem(GUESTBOOK_CACHE_KEY)
      if (raw) {
        const cached = JSON.parse(raw)
        nickname.value = cached.nickname || ''
        email.value = cached.email || ''
      } else {
        nickname.value = ''
        email.value = ''
      }
    } catch {
      nickname.value = ''
      email.value = ''
    }
  }
  content.value = ''
  emailTouched.value = false
}

function saveCache() {
  if (!authStore.user) {
    try {
      localStorage.setItem(GUESTBOOK_CACHE_KEY, JSON.stringify({
        nickname: nickname.value.trim(),
        email: email.value.trim(),
      }))
    } catch {}
  }
}

watch(() => props.visible, (v) => {
  if (v) initForm()
})

async function submit() {
  if (!canSubmit.value) return
  submitting.value = true
  try {
    await createGuestbook({
      nickname: nickname.value.trim(),
      email: email.value.trim(),
      content: content.value.trim(),
    })
    saveCache()
    emit('close')
  } catch { /* ignore */ }
  finally { submitting.value = false }
}
</script>
