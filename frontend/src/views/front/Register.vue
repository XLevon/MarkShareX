<template>
  <!-- Fullscreen modal overlay -->
  <Teleport to="body">
    <div class="auth-overlay">
      <div class="auth-modal">
        <button class="auth-close" @click="closeModal" :style="{ color: 'var(--color-text-muted)' }">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
        </button>

        <!-- 注册成功提示 -->
        <template v-if="registered">
          <div class="text-center py-4">
            <div class="text-5xl mb-4">🎉</div>
            <h2 class="text-xl font-bold mb-3" :style="{ color: 'var(--color-text)' }">注册成功</h2>
            <p class="text-sm mb-6 leading-relaxed" :style="{ color: 'var(--color-text-secondary)' }">
              新注册访客 <strong :style="{ color: 'var(--color-primary)' }">{{ registeredDisplayName }}</strong>，需要申请成为作者才能分享文章。
            </p>
            <div class="flex flex-col gap-3 items-center">
              <button
                @click="goApply"
                class="w-full px-6 py-2.5 rounded-xl text-sm font-medium transition-colors border-0 cursor-pointer"
                :style="{ backgroundColor: 'var(--color-primary)', color: '#fff' }"
              >申请成为作者</button>
              <button
                @click="closeModal"
                class="text-sm border-0 bg-transparent cursor-pointer"
                :style="{ color: 'var(--color-text-muted)' }"
              >稍后再说</button>
            </div>
          </div>
        </template>

        <!-- 注册表单 -->
        <template v-else>
          <div class="text-center mb-6">
            <h2 class="text-2xl font-bold" :style="{ color: 'var(--color-text)' }">创建账号</h2>
            <p class="text-sm mt-1" :style="{ color: 'var(--color-text-muted)' }">请填写以下信息完成注册</p>
          </div>

          <div v-if="error" class="mb-4 px-3 py-2 rounded-lg text-sm bg-red-500/10 text-red-400 border border-red-500/20">{{ error }}</div>

          <form @submit.prevent="handleRegister" class="space-y-4">
            <div>
              <label class="block text-sm font-medium mb-1.5" :style="{ color: 'var(--color-text-secondary)' }">用户名 <span :style="{ color: '#f87171' }">*</span></label>
              <input v-model="form.username" type="text" placeholder="设置用户名" required class="w-full px-4 py-2.5 rounded-xl border outline-none text-sm transition-colors" :style="inputStyle"
                @focus="(e: any) => e.target.style.borderColor = 'var(--color-primary)'" @blur="(e: any) => e.target.style.borderColor = 'var(--color-border)'" />
            </div>

            <div>
              <label class="block text-sm font-medium mb-1.5" :style="{ color: 'var(--color-text-secondary)' }">邮箱地址 <span :style="{ color: '#f87171' }">*</span></label>
              <input v-model="form.email" type="email" placeholder="your@email.com" required class="w-full px-4 py-2.5 rounded-xl border outline-none text-sm transition-colors" :style="inputStyle"
                @focus="(e: any) => e.target.style.borderColor = 'var(--color-primary)'" @blur="(e: any) => e.target.style.borderColor = 'var(--color-border)'" />
            </div>

            <div>
              <label class="block text-sm font-medium mb-1.5" :style="{ color: 'var(--color-text-secondary)' }">密码 <span :style="{ color: '#f87171' }">*</span></label>
              <div class="relative">
                <input v-model="form.password" :type="showPassword ? 'text' : 'password'" placeholder="设置密码（8-20位，含字母和数字）" required minlength="8"
                  class="w-full px-4 py-2.5 pr-10 rounded-xl border outline-none text-sm transition-colors" :style="inputStyle"
                  @focus="(e: any) => e.target.style.borderColor = 'var(--color-primary)'" @blur="(e: any) => e.target.style.borderColor = 'var(--color-border)'" />
                <button type="button" class="absolute right-3 top-1/2 -translate-y-1/2 text-lg" :style="{ color: 'var(--color-text-muted)' }" @click="showPassword = !showPassword">{{ showPassword ? '🙈' : '👁' }}</button>
              </div>
            </div>

            <div>
              <label class="block text-sm font-medium mb-1.5" :style="{ color: 'var(--color-text-secondary)' }">确认密码 <span :style="{ color: '#f87171' }">*</span></label>
              <div class="relative">
                <input v-model="form.confirmPassword" :type="showConfirm ? 'text' : 'password'" placeholder="再次输入密码" required
                  class="w-full px-4 py-2.5 pr-10 rounded-xl border outline-none text-sm transition-colors" :style="inputStyle"
                  @focus="(e: any) => e.target.style.borderColor = 'var(--color-primary)'" @blur="(e: any) => e.target.style.borderColor = 'var(--color-border)'" />
                <button type="button" class="absolute right-3 top-1/2 -translate-y-1/2 text-lg" :style="{ color: 'var(--color-text-muted)' }" @click="showConfirm = !showConfirm">{{ showConfirm ? '🙈' : '👁' }}</button>
              </div>
            </div>

            <label class="flex items-start gap-2 cursor-pointer">
              <input type="checkbox" v-model="agreed" class="mt-0.5" />
              <span class="text-xs" :style="{ color: 'var(--color-text-muted)' }">我已阅读并同意 <a href="#" class="underline" :style="{ color: 'var(--color-primary)' }">服务条款</a> 和 <a href="#" class="underline" :style="{ color: 'var(--color-primary)' }">隐私政策</a></span>
            </label>

            <button type="submit" :disabled="loading || !agreed" class="w-full py-3 rounded-xl text-white font-medium transition-all disabled:opacity-50" :style="{ backgroundColor: 'var(--color-primary)' }">
              {{ loading ? '注册中...' : '创建账号' }}
            </button>
          </form>
        </template>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { register } from '@/api/auth'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()

const loading = ref(false)
const error = ref('')
const showPassword = ref(false)
const showConfirm = ref(false)
const agreed = ref(false)
const registered = ref(false)
const registeredDisplayName = ref('')

const form = reactive({ username: '', email: '', password: '', confirmPassword: '' })

const inputStyle = {
  backgroundColor: 'var(--color-bg)',
  borderColor: 'var(--color-border)',
  color: 'var(--color-text)',
}

function closeModal() {
  const redirect = route.query.redirect as string
  // Don't follow auth-protected redirects — user is cancelling
  if (redirect && !redirect.startsWith('/admin') && redirect !== route.path) {
    router.replace(redirect)
  } else {
    router.replace('/')
  }
}

function goApply() {
  router.replace('/apply')
}

async function handleRegister() {
  error.value = ''
  if (!agreed.value) { error.value = '请先同意服务条款和隐私政策'; return }
  if (form.password !== form.confirmPassword) { error.value = '两次输入的密码不一致'; return }
  if (form.password.length < 8) { error.value = '密码至少需要8位'; return }

  loading.value = true
  try {
    const resp = await register(form.username, form.email, form.password, form.username)
    const user = resp.data.data.user
    authStore.setTokens(resp.data.data.access_token, resp.data.data.refresh_token, user)

    if (user.role === 'visitor') {
      registered.value = true
      registeredDisplayName.value = user.display_name || user.username
    } else {
      router.replace('/admin/posts/new')
    }
  } catch (e: any) {
    error.value = e.response?.data?.error || '注册失败，请稍后重试'
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
