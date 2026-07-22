import { defineStore } from 'pinia'
import { computed, onScopeDispose, ref } from 'vue'
import { login as apiLogin } from '@/api/auth'
import {
  AUTH_STORAGE_KEYS,
  clearAuthSession,
  readAuthSession,
  writeAuthSession,
} from '@/utils/authStorage'

interface UserInfo {
  id: number
  username: string
  email: string
  display_name?: string
  role: string
  avatar_url?: string
}

export const useAuthStore = defineStore('auth', () => {
  const stored = readAuthSession<UserInfo>()
  const token = ref(stored?.accessToken ?? '')
  const refreshTokenVal = ref(stored?.refreshToken ?? '')
  const user = ref<UserInfo | null>(stored?.user ?? null)
  let authGeneration = 0

  const isAuthenticated = computed(() => !!token.value)

  function applyTokens(
    accessToken: string,
    refreshToken: string,
    userInfo: UserInfo,
    rememberMe: boolean,
  ) {
    writeAuthSession({ accessToken, refreshToken, user: userInfo }, rememberMe)
    token.value = accessToken
    refreshTokenVal.value = refreshToken
    user.value = userInfo
  }

  function setTokens(
    accessToken: string,
    refreshToken: string,
    userInfo: UserInfo,
    rememberMe = false,
  ) {
    authGeneration += 1
    applyTokens(accessToken, refreshToken, userInfo, rememberMe)
  }

  function synchronizeFromStorage() {
    const current = readAuthSession<UserInfo>()
    token.value = current?.accessToken ?? ''
    refreshTokenVal.value = current?.refreshToken ?? ''
    user.value = current?.user ?? null
  }

  async function login(username: string, password: string, rememberMe = false) {
    const generation = ++authGeneration
    const { data: resp } = await apiLogin(username, password, rememberMe)
    if (generation !== authGeneration) throw new Error('Stale login response')

    const d = resp.data
    if (!d.user) throw new Error('Login response is missing user data')
    applyTokens(d.access_token, d.refresh_token, d.user, rememberMe)
  }

  function logout() {
    authGeneration += 1
    token.value = ''
    refreshTokenVal.value = ''
    user.value = null
    clearAuthSession()
    if (typeof document !== 'undefined') {
      document.cookie = 'scalar_token=; path=/scalar; max-age=0'
    }
  }

  if (typeof window !== 'undefined') {
    const onRefreshed = () => synchronizeFromStorage()
    const onExpired = () => synchronizeFromStorage()
    const onStorage = (event: StorageEvent) => {
      if (event.key !== null && !AUTH_STORAGE_KEYS.includes(event.key as typeof AUTH_STORAGE_KEYS[number])) {
        return
      }
      authGeneration += 1
      synchronizeFromStorage()
    }

    window.addEventListener('auth:refreshed', onRefreshed)
    window.addEventListener('auth:expired', onExpired)
    window.addEventListener('storage', onStorage)
    onScopeDispose(() => {
      window.removeEventListener('auth:refreshed', onRefreshed)
      window.removeEventListener('auth:expired', onExpired)
      window.removeEventListener('storage', onStorage)
    })
  }

  return { token, refreshTokenVal, user, isAuthenticated, login, logout, setTokens }
})
