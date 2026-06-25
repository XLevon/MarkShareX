<template>
  <Teleport to="body">
    <div class="auth-overlay" @click.self="closeModal">
      <div class="auth-modal">
        <button class="auth-close" @click="closeModal" :style="{ color: 'var(--color-text-muted)' }">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
        </button>

        <!-- 提交成功 -->
        <template v-if="submitted">
          <div class="text-center py-4">
            <div class="text-5xl mb-4">📩</div>
            <h2 class="text-xl font-bold mb-3" :style="{ color: 'var(--color-text)' }">申请已提交</h2>
            <p class="text-sm mb-6 leading-relaxed" :style="{ color: 'var(--color-text-secondary)' }">
              你的作者申请已提交，管理员审核通过后即可开始分享文章。
            </p>
            <button
              @click="closeModal"
              class="inline-block px-6 py-2.5 rounded-xl text-sm font-medium transition-colors border-0 cursor-pointer"
              :style="{ backgroundColor: 'var(--color-primary)', color: '#fff' }"
            >返回首页</button>
          </div>
        </template>

        <!-- 申请表单 -->
        <template v-else>
          <div class="text-center mb-6">
            <h2 class="text-2xl font-bold" :style="{ color: 'var(--color-text)' }">申请成为作者</h2>
            <p class="text-sm mt-1" :style="{ color: 'var(--color-text-muted)' }">填写以下信息，管理员审核后即可开始分享文章</p>
          </div>

          <div v-if="error" class="mb-4 px-3 py-2 rounded-lg text-sm bg-red-500/10 text-red-400 border border-red-500/20">{{ error }}</div>

          <form @submit.prevent="handleSubmit" class="space-y-4">
            <div>
              <label class="block text-sm font-medium mb-1.5" :style="{ color: 'var(--color-text-secondary)' }">申请理由 <span :style="{ color: '#f87171' }">*</span></label>
              <textarea
                v-model="form.reason"
                placeholder="请说明你为什么想成为作者，例如：想分享技术文章、记录学习笔记等"
                rows="3"
                required
                class="w-full px-4 py-2.5 rounded-xl border outline-none text-sm transition-colors resize-none"
                :style="inputStyle"
                @focus="(e: any) => e.target.style.borderColor = 'var(--color-primary)'"
                @blur="(e: any) => e.target.style.borderColor = 'var(--color-border)'"
              ></textarea>
            </div>

            <div>
              <label class="block text-sm font-medium mb-1.5" :style="{ color: 'var(--color-text-secondary)' }">分享内容说明 <span :style="{ color: '#f87171' }">*</span></label>
              <textarea
                v-model="form.content"
                placeholder="请简单描述你将分享的内容方向，例如：前端开发、Python教程、AI工具推荐等"
                rows="3"
                required
                class="w-full px-4 py-2.5 rounded-xl border outline-none text-sm transition-colors resize-none"
                :style="inputStyle"
                @focus="(e: any) => e.target.style.borderColor = 'var(--color-primary)'"
                @blur="(e: any) => e.target.style.borderColor = 'var(--color-border)'"
              ></textarea>
            </div>

            <div class="flex gap-3 pt-2">
              <button
                type="button"
                @click="closeModal"
                class="flex-1 py-3 rounded-xl text-sm font-medium transition-colors border-0 cursor-pointer"
                :style="{ backgroundColor: 'var(--color-bg-hover)', color: 'var(--color-text-secondary)' }"
              >取消</button>
              <button
                type="submit"
                :disabled="loading"
                class="flex-1 py-3 rounded-xl text-white font-medium transition-all disabled:opacity-50"
                :style="{ backgroundColor: 'var(--color-primary)' }"
              >{{ loading ? '提交中...' : '确认申请' }}</button>
            </div>
          </form>
        </template>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { submitApplication } from '@/api/admin'

const router = useRouter()
const route = useRoute()

const loading = ref(false)
const error = ref('')
const submitted = ref(false)

const form = reactive({ reason: '', content: '' })

const inputStyle = {
  backgroundColor: 'var(--color-bg)',
  borderColor: 'var(--color-border)',
  color: 'var(--color-text)',
}

function closeModal() {
  const redirect = route.query.redirect as string
  if (redirect && !redirect.startsWith('/admin') && redirect !== route.path) {
    router.replace(redirect)
  } else {
    router.replace('/')
  }
}

async function handleSubmit() {
  error.value = ''
  if (!form.reason.trim()) { error.value = '请填写申请理由'; return }
  if (!form.content.trim()) { error.value = '请填写分享内容说明'; return }

  loading.value = true
  try {
    await submitApplication(form.reason.trim(), form.content.trim())
    submitted.value = true
  } catch (e: any) {
    error.value = e?.response?.data?.message || e?.response?.data?.error || '提交失败，请稍后重试'
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.auth-overlay {
  position: fixed;
  inset: 0;
  z-index: 9998;
  background: rgba(0, 0, 0, 0.55);
  backdrop-filter: blur(6px);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
}
.auth-modal {
  position: relative;
  width: 100%;
  max-width: 440px;
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: 16px;
  padding: 32px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4);
  max-height: 90vh;
  overflow-y: auto;
}
.auth-close {
  position: absolute;
  top: 12px;
  right: 12px;
  border: none;
  background: transparent;
  cursor: pointer;
  padding: 4px;
  border-radius: 6px;
  transition: background 0.15s;
}
.auth-close:hover {
  background: var(--color-bg-hover);
}
</style>
