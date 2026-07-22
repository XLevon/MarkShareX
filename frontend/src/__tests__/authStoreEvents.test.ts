import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { useAuthStore } from '@/stores/auth'
import { clearAuthSession, readAuthSession, writeAuthSession } from '@/utils/authStorage'

const authApi = vi.hoisted(() => ({ login: vi.fn() }))
vi.mock('@/api/auth', () => ({ login: authApi.login }))

const firstUser = {
  id: 1,
  username: 'first',
  email: 'first@example.com',
  role: 'author',
}
const secondUser = {
  id: 2,
  username: 'second',
  email: 'second@example.com',
  role: 'admin',
}

function loginResponse(
  accessToken: string,
  refreshToken: string,
  user: typeof firstUser | typeof secondUser,
) {
  return {
    data: {
      data: {
        access_token: accessToken,
        refresh_token: refreshToken,
        user,
      },
    },
  }
}

function storageEvent(key: string | null = 'marksharex_token') {
  window.dispatchEvent(new StorageEvent('storage', { key, storageArea: localStorage }))
}

describe('auth store session events and operation races', () => {
  const stores: Array<ReturnType<typeof useAuthStore>> = []

  function createStore() {
    const store = useAuthStore()
    stores.push(store)
    return store
  }

  beforeEach(() => {
    localStorage.clear()
    sessionStorage.clear()
    authApi.login.mockReset()
    setActivePinia(createPinia())
  })

  afterEach(() => {
    while (stores.length) stores.pop()?.$dispose()
  })

  it('hydrates only from a complete valid tuple', () => {
    localStorage.setItem('marksharex_token', 'partial-access')
    localStorage.setItem('marksharex_refresh_token', 'partial-refresh')

    const store = createStore()

    expect(store.isAuthenticated).toBe(false)
    expect(store.user).toBeNull()
    expect(localStorage.length).toBe(0)
  })

  it('synchronizes Pinia from the authoritative tuple after refresh', () => {
    writeAuthSession(
      { accessToken: 'first-access', refreshToken: 'first-refresh', user: firstUser },
      true,
    )
    const store = createStore()
    writeAuthSession(
      { accessToken: 'second-access', refreshToken: 'second-refresh', user: secondUser },
      false,
    )

    window.dispatchEvent(new CustomEvent('auth:refreshed'))

    expect(store.token).toBe('second-access')
    expect(store.refreshTokenVal).toBe('second-refresh')
    expect(store.user).toEqual(secondUser)
    expect(store.isAuthenticated).toBe(true)
  })

  it('clears Pinia authentication when the current session expires', () => {
    writeAuthSession(
      { accessToken: 'expired-access', refreshToken: 'expired-refresh', user: firstUser },
      true,
    )
    const store = createStore()
    clearAuthSession()

    window.dispatchEvent(new CustomEvent('auth:expired'))

    expect(store.token).toBe('')
    expect(store.refreshTokenVal).toBe('')
    expect(store.user).toBeNull()
    expect(store.isAuthenticated).toBe(false)
  })

  it('synchronizes remembered identity replacement and logout from cross-tab storage events', () => {
    writeAuthSession(
      { accessToken: 'tab-first-access', refreshToken: 'tab-first-refresh', user: firstUser },
      true,
    )
    const store = createStore()

    writeAuthSession(
      { accessToken: 'tab-second-access', refreshToken: 'tab-second-refresh', user: secondUser },
      true,
    )
    storageEvent()
    expect(store.token).toBe('tab-second-access')
    expect(store.user).toEqual(secondUser)

    clearAuthSession()
    storageEvent(null)
    expect(store.isAuthenticated).toBe(false)
    expect(store.user).toBeNull()
  })

  it('does not let a login response completing after logout restore authentication', async () => {
    let resolveLogin!: (value: unknown) => void
    authApi.login.mockImplementation(() => new Promise(resolve => { resolveLogin = resolve }))
    const store = createStore()

    const pending = store.login('first', 'secret', true)
    store.logout()
    resolveLogin(loginResponse('late-access', 'late-refresh', firstUser))

    await expect(pending).rejects.toThrow('Stale login response')
    expect(store.isAuthenticated).toBe(false)
    expect(readAuthSession()).toBeNull()
  })

  it('keeps the newest overlapping login when responses resolve out of order', async () => {
    let resolveFirst!: (value: unknown) => void
    let resolveSecond!: (value: unknown) => void
    authApi.login.mockImplementation((username: string) => new Promise(resolve => {
      if (username === 'first') resolveFirst = resolve
      else resolveSecond = resolve
    }))
    const store = createStore()

    const first = store.login('first', 'secret', true)
    const second = store.login('second', 'secret', false)
    resolveSecond(loginResponse('second-access', 'second-refresh', secondUser))
    await second
    resolveFirst(loginResponse('first-access', 'first-refresh', firstUser))

    await expect(first).rejects.toThrow('Stale login response')
    expect(store.token).toBe('second-access')
    expect(store.user).toEqual(secondUser)
    expect(readAuthSession()).toMatchObject({
      accessToken: 'second-access',
      refreshToken: 'second-refresh',
      user: secondUser,
      storage: sessionStorage,
    })
  })

  it('invalidates a pending local login when another tab replaces the session', async () => {
    let resolveLogin!: (value: unknown) => void
    authApi.login.mockImplementation(() => new Promise(resolve => { resolveLogin = resolve }))
    const store = createStore()

    const pending = store.login('first', 'secret', true)
    writeAuthSession(
      { accessToken: 'other-tab-access', refreshToken: 'other-tab-refresh', user: secondUser },
      true,
    )
    storageEvent()
    resolveLogin(loginResponse('stale-local-access', 'stale-local-refresh', firstUser))

    await expect(pending).rejects.toThrow('Stale login response')
    expect(store.token).toBe('other-tab-access')
    expect(store.user).toEqual(secondUser)
  })

  it('removes window event listeners when a Pinia store is disposed', () => {
    writeAuthSession(
      { accessToken: 'disposed-access', refreshToken: 'disposed-refresh', user: firstUser },
      true,
    )
    const store = createStore()
    store.$dispose()

    writeAuthSession(
      { accessToken: 'replacement-access', refreshToken: 'replacement-refresh', user: secondUser },
      true,
    )
    storageEvent()

    expect(store.token).toBe('disposed-access')
    expect(store.user).toEqual(firstUser)
  })
})
