import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { login as apiLogin } from '@/api/auth'

interface UserInfo {
  id: number
  username: string
  email: string
  display_name?: string
  role: string
  avatar_url?: string
}

export const useAuthStore = defineStore('auth', () => {
  function readToken(key: string): string {
    return localStorage.getItem(key) || sessionStorage.getItem(key) || ''
  }

  const token = ref(readToken('marksharex_token'))
  const refreshTokenVal = ref(readToken('marksharex_refresh_token'))
  const user = ref<UserInfo | null>(null)

  try {
    const saved = readToken('marksharex_user')
    if (saved) user.value = JSON.parse(saved)
  } catch {}

  const isAuthenticated = computed(() => !!token.value)

  function setTokens(accessToken: string, refreshToken: string, userInfo?: UserInfo, rememberMe = false) {
    const storage = rememberMe ? localStorage : sessionStorage
    token.value = accessToken
    refreshTokenVal.value = refreshToken
    if (userInfo) {
      user.value = userInfo
      storage.setItem('marksharex_user', JSON.stringify(userInfo))
    }
    storage.setItem('marksharex_token', accessToken)
    storage.setItem('marksharex_refresh_token', refreshToken)
  }

  async function login(username: string, password: string, rememberMe = false) {
    const { data: resp } = await apiLogin(username, password, rememberMe)
    const d = resp.data
    setTokens(d.access_token, d.refresh_token, d.user, rememberMe)
  }

  function logout() {
    token.value = ''
    refreshTokenVal.value = ''
    user.value = null
    for (const key of ['marksharex_token', 'marksharex_refresh_token', 'marksharex_user']) {
      localStorage.removeItem(key)
      sessionStorage.removeItem(key)
    }
    document.cookie = 'scalar_token=; path=/scalar; max-age=0'
  }

  // 监听 interceptor 刷新成功事件，同步响应式 ref
  if (typeof window !== 'undefined') {
    window.addEventListener('auth:refreshed', ((e: CustomEvent) => {
      const { token: newToken, refreshToken: newRefresh, user: newUser } = e.detail
      token.value = newToken
      refreshTokenVal.value = newRefresh
      if (newUser) user.value = newUser
    }) as EventListener)
  }

  return { token, refreshTokenVal, user, isAuthenticated, login, logout, setTokens }
})
