import { beforeEach, describe, expect, it } from 'vitest'
import {
  clearAuthSession,
  readAuthSession,
  writeAuthSession,
} from '@/utils/authStorage'

const persistent = {
  accessToken: 'persistent-access',
  refreshToken: 'persistent-refresh',
  user: { id: 1, username: 'persistent-user', role: 'admin' },
}
const transient = {
  accessToken: 'session-access',
  refreshToken: 'session-refresh',
  user: { id: 2, username: 'session-user', role: 'author' },
}

function writeRaw(storage: Storage, user: string | null) {
  storage.setItem('marksharex_token', 'raw-access')
  storage.setItem('marksharex_refresh_token', 'raw-refresh')
  if (user !== null) storage.setItem('marksharex_user', user)
}

describe('auth storage isolation', () => {
  beforeEach(() => {
    localStorage.clear()
    sessionStorage.clear()
  })

  it('writes a remembered login only to localStorage', () => {
    writeAuthSession(persistent, true)
    expect(readAuthSession()).toEqual({ ...persistent, storage: localStorage })
    expect(sessionStorage.length).toBe(0)
  })

  it('writes a non-remembered login only to sessionStorage', () => {
    writeAuthSession(transient, false)
    expect(readAuthSession()).toEqual({ ...transient, storage: sessionStorage })
    expect(localStorage.length).toBe(0)
  })

  it('clears the other storage before switching persistence mode', () => {
    writeAuthSession(persistent, true)
    writeAuthSession(transient, false)
    expect(readAuthSession()).toEqual({ ...transient, storage: sessionStorage })
    expect(localStorage.length).toBe(0)

    writeAuthSession(persistent, true)
    expect(readAuthSession()).toEqual({ ...persistent, storage: localStorage })
    expect(sessionStorage.length).toBe(0)
  })

  it('fails closed when both stores contain otherwise complete sessions', () => {
    writeRaw(localStorage, JSON.stringify(persistent.user))
    sessionStorage.setItem('marksharex_token', 'other-access')
    sessionStorage.setItem('marksharex_refresh_token', 'other-refresh')
    sessionStorage.setItem('marksharex_user', JSON.stringify(transient.user))

    expect(readAuthSession()).toBeNull()
    expect(localStorage.length).toBe(0)
    expect(sessionStorage.length).toBe(0)
  })

  it.each([
    ['missing user', null],
    ['JSON null', 'null'],
    ['string user', '"user"'],
    ['array user', '[]'],
    ['object without id', '{"username":"missing-id","role":"author"}'],
    ['non-numeric id', '{"id":"1","role":"author"}'],
    ['object without role', '{"id":1,"username":"missing-role"}'],
    ['empty role', '{"id":1,"role":""}'],
    ['invalid JSON', '{'],
  ])('fails closed and clears a tuple with %s', (_label, rawUser) => {
    writeRaw(localStorage, rawUser)
    expect(readAuthSession()).toBeNull()
    expect(localStorage.length).toBe(0)
    expect(sessionStorage.length).toBe(0)
  })

  it('fails closed and clears orphaned tuple fields', () => {
    localStorage.setItem('marksharex_token', 'orphan-access')
    expect(readAuthSession()).toBeNull()
    expect(localStorage.length).toBe(0)
  })

  it('rejects malformed writes without retaining either storage', () => {
    expect(() => writeAuthSession({
      accessToken: 'access',
      refreshToken: 'refresh',
      user: { id: 1, role: '' },
    }, true)).toThrow('Invalid authentication session')
    expect(localStorage.length).toBe(0)
    expect(sessionStorage.length).toBe(0)
  })

  it('logout clears the complete authentication tuple from both stores', () => {
    writeAuthSession(persistent, true)
    sessionStorage.setItem('marksharex_user', '{"id":99,"role":"author"}')
    clearAuthSession()
    expect(readAuthSession()).toBeNull()
    expect(localStorage.length).toBe(0)
    expect(sessionStorage.length).toBe(0)
  })
})
