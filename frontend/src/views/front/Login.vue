<template>
  <!-- Fullscreen modal overlay -->
  <Teleport to="body">
    <div class="auth-overlay">
      <div class="auth-modal">
        <!-- Close button -->
        <button class="auth-close" @click="closeModal" :style="{ color: 'var(--color-text-muted)' }">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
        </button>

        <div class="text-center mb-6">
          <h2 class="text-2xl font-bold" :style="{ color: 'var(--color-text)' }">欢迎回来</h2>
          <p class="text-sm mt-1" :style="{ color: 'var(--color-text-muted)' }">登录你的账号</p>
        </div>

        <!-- Error -->
        <div v-if="error" class="mb-4 px-3 py-2 rounded-lg text-sm bg-red-500/10 text-red-400 border border-red-500/20">
          {{ error }}
        </div>

        <form @submit.prevent="handleLogin" class="space-y-4">
          <div>
            <label class="block text-sm font-medium mb-1.5" :style="{ color: 'var(--color-text-secondary)' }">邮箱地址</label>
            <input
              v-model="form.email"
              type="email"
              placeholder="name@example.com"
              required
              class="w-full px-4 py-2.5 rounded-xl border outline-none text-sm transition-colors"
              :style="inputStyle"
              @focus="(e: any) => e.target.style.borderColor = 'var(--color-primary)'"
              @blur="(e: any) => e.target.style.borderColor = 'var(--color-border)'"
            />
          </div>

          <div>
            <label class="block text-sm font-medium mb-1.5" :style="{ color: 'var(--color-text-secondary)' }">密码</label>
            <div class="relative">
              <input
                v-model="form.password"
                :type="showPassword ? 'text' : 'password'"
                placeholder="请输入密码"
                required
                class="w-full px-4 py-2.5 pr-10 rounded-xl border outline-none text-sm transition-colors"
                :style="inputStyle"
                @focus="(e: any) => e.target.style.borderColor = 'var(--color-primary)'"
                @blur="(e: any) => e.target.style.borderColor = 'var(--color-border)'"
              />
              <button type="button" class="absolute right-3 top-1/2 -translate-y-1/2 text-lg" :style="{ color: 'var(--color-text-muted)' }" @click="showPassword = !showPassword">
                {{ showPassword ? '🙈' : '👁' }}
              </button>
            </div>
          </div>

          <div class="flex items-center justify-between">
            <label class="flex items-center gap-2 cursor-pointer text-sm" :style="{ color: 'var(--color-text-muted)' }">
              <input type="checkbox" v-model="rememberMe" /> 记住我
            </label>
            <a href="#" class="text-sm" :style="{ color: 'var(--color-primary)' }" @click.prevent>忘记密码？</a>
          </div>

          <button
            type="submit"
            :disabled="loading"
            class="w-full py-3 rounded-xl text-white font-medium transition-all disabled:opacity-50"
            :style="{ backgroundColor: 'var(--color-primary)' }"
          >{{ loading ? '登录中...' : '登录' }}</button>
        </form>

        <div class="text-center mt-6 space-y-2">
          <p class="text-sm" :style="{ color: 'var(--color-text-muted)' }">
            还没有账号？
            <a href="#" class="font-medium" :style="{ color: 'var(--color-primary)' }" @click.prevent="goRegister">立即注册</a>
          </p>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()

const loading = ref(false)
const error = ref('')
const showPassword = ref(false)
const rememberMe = ref(false)

const form = reactive({ email: '', password: '' })

const inputStyle = {
  backgroundColor: 'var(--color-bg)',
  borderColor: 'var(--color-border)',
  color: 'var(--color-text)',
}

function closeModal() {
  const redirect = route.query.redirect as string
  // Don't follow auth-protected redirects — user is cancelling login
  if (redirect && !redirect.startsWith('/admin') && redirect !== route.path) {
    router.replace(redirect)
  } else {
    router.replace('/')
  }
}

function goRegister() {
  const redirect = route.query.redirect as string
  if (redirect) {
    router.replace(`/register?redirect=${encodeURIComponent(redirect)}`)
  } else {
    router.replace('/register')
  }
}

async function handleLogin() {
  error.value = ''
  loading.value = true
  try {
    await authStore.login(form.email, form.password, rememberMe.value)
    const redirect = (route.query.redirect as string) || '/'
    // Visitors can't access admin pages — redirect to apply instead
    if (authStore.user?.role === 'visitor' && redirect.startsWith('/admin')) {
      router.replace('/apply')
    } else {
      router.replace(redirect)
    }
  } catch (e: any) {
    error.value = e.response?.data?.error || '登录失败，请检查邮箱和密码'
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
  overflow-y: auto;
}
/* 移动端：顶部对齐，避免居中时内容被截断 */
@media (max-width: 480px) {
  .auth-overlay {
    align-items: flex-start;
    padding-top: 40px;
    padding-bottom: 40px;
  }
}
.auth-modal {
  position: relative;
  width: 100%;
  max-width: 400px;
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: 16px;
  padding: 32px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4);
  max-height: none;
}
/* 移动端：减小内边距 */
@media (max-width: 480px) {
  .auth-modal {
    padding: 20px 16px;
    border-radius: 12px;
  }
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
