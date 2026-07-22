import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import api from '@/api'
import router from '@/router'
import { useAuthStore } from '@/stores/auth'
import { writeAuthSession } from '@/utils/authStorage'

const defaultAdapter = api.defaults.adapter

function authorizationHeader(headers: unknown): string | undefined {
  if (!headers || typeof headers !== 'object') return undefined
  return (headers as { Authorization?: string }).Authorization
}

describe('authentication integration parity', () => {
  let store: ReturnType<typeof useAuthStore> | undefined

  beforeEach(async () => {
    Object.defineProperty(window, 'matchMedia', {
      configurable: true,
      value: vi.fn().mockReturnValue({
        matches: false,
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
      }),
    })
    Object.defineProperty(window, 'scrollTo', {
      configurable: true,
      value: vi.fn(),
    })
    localStorage.clear()
    sessionStorage.clear()
    setActivePinia(createPinia())
    api.defaults.adapter = defaultAdapter
    await router.replace('/')
  })

  afterEach(() => {
    store?.$dispose()
    store = undefined
    api.defaults.adapter = defaultAdapter
  })

  it('accepts one valid tuple consistently in Pinia, Axios, and the router', async () => {
    const user = {
      id: 9,
      username: 'integrated',
      email: 'integrated@example.com',
      role: 'author',
    }
    writeAuthSession(
      { accessToken: 'integrated-access', refreshToken: 'integrated-refresh', user },
      false,
    )
    store = useAuthStore()
    let header: string | undefined
    api.defaults.adapter = async config => {
      header = authorizationHeader(config.headers)
      return { status: 200, statusText: 'OK', data: {}, headers: {}, config }
    }

    await api.get('/integration-auth')
    await router.push('/admin/dashboard')

    expect(store.isAuthenticated).toBe(true)
    expect(store.user).toEqual(user)
    expect(header).toBe('Bearer integrated-access')
    expect(router.currentRoute.value.name).toBe('admin-dashboard')
  })

  it('rejects a malformed tuple consistently in Pinia, Axios, and the router', async () => {
    localStorage.setItem('marksharex_token', 'malformed-access')
    localStorage.setItem('marksharex_refresh_token', 'malformed-refresh')
    localStorage.setItem('marksharex_user', '{"id":9,"username":"no-role"}')
    store = useAuthStore()
    let header: string | undefined
    api.defaults.adapter = async config => {
      header = authorizationHeader(config.headers)
      return { status: 200, statusText: 'OK', data: {}, headers: {}, config }
    }

    await api.get('/integration-anonymous')
    await router.push('/admin/dashboard')

    expect(store.isAuthenticated).toBe(false)
    expect(store.user).toBeNull()
    expect(header).toBeUndefined()
    expect(router.currentRoute.value.name).toBe('login')
    expect(localStorage.length).toBe(0)
    expect(sessionStorage.length).toBe(0)
  })

  it('keeps Pinia, Axios, and router roles aligned after a cross-tab identity replacement', async () => {
    const admin = {
      id: 10,
      username: 'admin-tab',
      email: 'admin-tab@example.com',
      role: 'admin',
    }
    const visitor = {
      id: 11,
      username: 'visitor-tab',
      email: 'visitor-tab@example.com',
      role: 'visitor',
    }
    writeAuthSession(
      { accessToken: 'admin-tab-access', refreshToken: 'admin-tab-refresh', user: admin },
      true,
    )
    store = useAuthStore()

    writeAuthSession(
      { accessToken: 'visitor-tab-access', refreshToken: 'visitor-tab-refresh', user: visitor },
      true,
    )
    window.dispatchEvent(new StorageEvent('storage', {
      key: 'marksharex_token',
      storageArea: localStorage,
    }))

    let header: string | undefined
    api.defaults.adapter = async config => {
      header = authorizationHeader(config.headers)
      return { status: 200, statusText: 'OK', data: {}, headers: {}, config }
    }
    await api.get('/integration-cross-tab')
    await router.push('/admin/ai')

    expect(store.token).toBe('visitor-tab-access')
    expect(store.user).toEqual(visitor)
    expect(header).toBe('Bearer visitor-tab-access')
    expect(router.currentRoute.value.name).toBe('home')
  })
})
