import axios from 'axios'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import api from '@/api'
import { readAuthSession, writeAuthSession } from '@/utils/authStorage'

const defaultAdapter = api.defaults.adapter

function user(id: number, role = 'author') {
  return { id, username: `user-${id}`, role }
}

function authorizationHeader(headers: unknown): string | undefined {
  if (!headers || typeof headers !== 'object') return undefined
  return (headers as { Authorization?: string }).Authorization
}

function unauthorized(config: unknown) {
  return {
    config,
    response: { status: 401, data: {}, headers: {}, config },
  }
}

function refreshResponse(accessToken: string, refreshToken: string, id = 1) {
  return {
    data: {
      data: {
        access_token: accessToken,
        refresh_token: refreshToken,
        user: user(id),
      },
    },
  }
}

describe('API authentication refresh', () => {
  beforeEach(() => {
    localStorage.clear()
    sessionStorage.clear()
    vi.restoreAllMocks()
    api.defaults.adapter = defaultAdapter
  })

  afterEach(() => {
    api.defaults.adapter = defaultAdapter
    vi.restoreAllMocks()
  })

  it('shares one refresh across concurrent 401s and keeps the original storage', async () => {
    writeAuthSession(
      { accessToken: 'expired-access', refreshToken: 'session-refresh', user: user(7) },
      false,
    )
    const refresh = vi.spyOn(axios, 'post').mockResolvedValue(
      refreshResponse('renewed-access', 'renewed-refresh', 7),
    )
    api.defaults.adapter = async config => {
      if (authorizationHeader(config.headers) !== 'Bearer renewed-access') {
        return Promise.reject(unauthorized(config))
      }
      return { status: 200, statusText: 'OK', data: { ok: true }, headers: {}, config }
    }

    const [first, second] = await Promise.all([api.get('/protected-a'), api.get('/protected-b')])

    expect(first.data).toEqual({ ok: true })
    expect(second.data).toEqual({ ok: true })
    expect(refresh).toHaveBeenCalledTimes(1)
    expect(readAuthSession()).toEqual({
      accessToken: 'renewed-access',
      refreshToken: 'renewed-refresh',
      user: user(7),
      storage: sessionStorage,
    })
    expect(localStorage.length).toBe(0)
  })

  it('replays a staggered old-token 401 without starting a second refresh', async () => {
    writeAuthSession(
      { accessToken: 'staggered-old-access', refreshToken: 'staggered-old-refresh', user: user(3) },
      true,
    )
    const refresh = vi.spyOn(axios, 'post').mockResolvedValue(
      refreshResponse('staggered-new-access', 'staggered-new-refresh', 3),
    )
    let rejectDelayed!: () => void
    let delayedInitialSeen = false
    api.defaults.adapter = async config => {
      const auth = authorizationHeader(config.headers)
      if (config.url === '/staggered-b' && auth === 'Bearer staggered-old-access' && !delayedInitialSeen) {
        delayedInitialSeen = true
        return new Promise((_, reject) => {
          rejectDelayed = () => reject(unauthorized(config))
        })
      }
      if (auth === 'Bearer staggered-old-access') return Promise.reject(unauthorized(config))
      return { status: 200, statusText: 'OK', data: { auth }, headers: {}, config }
    }

    const first = api.get('/staggered-a')
    const second = api.get('/staggered-b')
    await expect(first).resolves.toMatchObject({ data: { auth: 'Bearer staggered-new-access' } })
    await vi.waitFor(() => expect(rejectDelayed).toBeTypeOf('function'))
    rejectDelayed()
    await expect(second).resolves.toMatchObject({ data: { auth: 'Bearer staggered-new-access' } })
    expect(refresh).toHaveBeenCalledTimes(1)
  })

  it('never refreshes or replays an old request under a newer login', async () => {
    writeAuthSession(
      { accessToken: 'identity-a-access', refreshToken: 'identity-a-refresh', user: user(1) },
      true,
    )
    const refresh = vi.spyOn(axios, 'post').mockResolvedValue(
      refreshResponse('should-not-exist', 'should-not-exist', 1),
    )
    const seenHeaders: Array<string | undefined> = []
    let rejectOld401!: () => void
    api.defaults.adapter = async config => {
      seenHeaders.push(authorizationHeader(config.headers))
      return new Promise((_, reject) => {
        rejectOld401 = () => reject(unauthorized(config))
      })
    }

    const oldMutation = api.post('/dangerous-mutation', { enabled: true })
    await vi.waitFor(() => expect(rejectOld401).toBeTypeOf('function'))
    writeAuthSession(
      { accessToken: 'identity-b-access', refreshToken: 'identity-b-refresh', user: user(2, 'admin') },
      false,
    )
    rejectOld401()

    await expect(oldMutation).rejects.toBeDefined()
    expect(refresh).not.toHaveBeenCalled()
    expect(seenHeaders).toEqual(['Bearer identity-a-access'])
    expect(readAuthSession()).toMatchObject({
      accessToken: 'identity-b-access',
      refreshToken: 'identity-b-refresh',
      user: user(2, 'admin'),
    })
  })

  it('does not join an old in-flight refresh or clear the newer session', async () => {
    writeAuthSession(
      { accessToken: 'join-a-access', refreshToken: 'join-a-refresh', user: user(1) },
      true,
    )
    let rejectRefresh!: (reason?: unknown) => void
    const refresh = vi.spyOn(axios, 'post').mockImplementation(
      () => new Promise((_, reject) => { rejectRefresh = reject }) as ReturnType<typeof axios.post>,
    )
    let rejectDelayed!: () => void
    api.defaults.adapter = async config => {
      if (config.url === '/start-old-refresh') return Promise.reject(unauthorized(config))
      return new Promise((_, reject) => {
        rejectDelayed = () => reject(unauthorized(config))
      })
    }

    const refreshing = api.get('/start-old-refresh').catch(error => error)
    await vi.waitFor(() => expect(rejectRefresh).toBeTypeOf('function'))
    const delayed = api.post('/old-delayed-mutation').catch(error => error)
    await vi.waitFor(() => expect(rejectDelayed).toBeTypeOf('function'))
    writeAuthSession(
      { accessToken: 'join-b-access', refreshToken: 'join-b-refresh', user: user(2, 'admin') },
      false,
    )
    rejectDelayed()
    await delayed
    rejectRefresh(new Error('old refresh failed'))
    await refreshing

    expect(refresh).toHaveBeenCalledTimes(1)
    expect(readAuthSession()).toMatchObject({
      accessToken: 'join-b-access',
      refreshToken: 'join-b-refresh',
      user: user(2, 'admin'),
    })
  })

  it('does not clear a newer login when the old refresh later fails', async () => {
    writeAuthSession(
      { accessToken: 'failure-old-access', refreshToken: 'failure-old-refresh', user: user(1) },
      true,
    )
    let rejectRefresh!: (reason?: unknown) => void
    vi.spyOn(axios, 'post').mockImplementation(
      () => new Promise((_, reject) => { rejectRefresh = reject }) as ReturnType<typeof axios.post>,
    )
    api.defaults.adapter = async config => Promise.reject(unauthorized(config))

    const oldRequest = api.get('/old-session-failure').catch(error => error)
    await vi.waitFor(() => expect(rejectRefresh).toBeTypeOf('function'))
    writeAuthSession(
      { accessToken: 'failure-new-access', refreshToken: 'failure-new-refresh', user: user(2) },
      false,
    )
    rejectRefresh(new Error('old refresh failed'))
    await oldRequest

    expect(readAuthSession()).toMatchObject({
      accessToken: 'failure-new-access',
      refreshToken: 'failure-new-refresh',
      user: user(2),
    })
  })

  it('does not let a late refresh overwrite a newer login', async () => {
    writeAuthSession(
      { accessToken: 'success-old-access', refreshToken: 'success-old-refresh', user: user(1) },
      true,
    )
    let resolveRefresh!: (value: unknown) => void
    vi.spyOn(axios, 'post').mockImplementation(
      () => new Promise(resolve => { resolveRefresh = resolve }) as ReturnType<typeof axios.post>,
    )
    api.defaults.adapter = async config => Promise.reject(unauthorized(config))

    const oldRequest = api.get('/old-session').catch(error => error)
    await vi.waitFor(() => expect(resolveRefresh).toBeTypeOf('function'))
    writeAuthSession(
      { accessToken: 'success-new-access', refreshToken: 'success-new-refresh', user: user(2) },
      false,
    )
    resolveRefresh(refreshResponse('late-old-access', 'late-old-refresh', 1))
    await oldRequest

    expect(readAuthSession()).toMatchObject({
      accessToken: 'success-new-access',
      refreshToken: 'success-new-refresh',
      user: user(2),
    })
  })

  it('does not clear a newer login when the retried request later returns 401', async () => {
    writeAuthSession(
      { accessToken: 'retry-old-access', refreshToken: 'retry-old-refresh', user: user(1) },
      true,
    )
    vi.spyOn(axios, 'post').mockResolvedValue(
      refreshResponse('retry-refreshed-access', 'retry-refreshed-refresh', 1),
    )
    let rejectRetry!: () => void
    api.defaults.adapter = async config => {
      if (authorizationHeader(config.headers) === 'Bearer retry-old-access') {
        return Promise.reject(unauthorized(config))
      }
      return new Promise((_, reject) => {
        rejectRetry = () => reject(unauthorized(config))
      })
    }

    const request = api.post('/retry-then-switch').catch(error => error)
    await vi.waitFor(() => expect(rejectRetry).toBeTypeOf('function'))
    writeAuthSession(
      { accessToken: 'retry-new-login', refreshToken: 'retry-new-refresh', user: user(2, 'admin') },
      false,
    )
    rejectRetry()
    await request

    expect(readAuthSession()).toMatchObject({
      accessToken: 'retry-new-login',
      refreshToken: 'retry-new-refresh',
      user: user(2, 'admin'),
    })
  })

  it('does not refresh or replay a late 401 after logout', async () => {
    writeAuthSession(
      { accessToken: 'logout-access', refreshToken: 'logout-refresh', user: user(1) },
      true,
    )
    const refresh = vi.spyOn(axios, 'post')
    let reject401!: () => void
    api.defaults.adapter = async config => new Promise((_, reject) => {
      reject401 = () => reject(unauthorized(config))
    })

    const request = api.put('/logout-race', { value: true })
    await vi.waitFor(() => expect(reject401).toBeTypeOf('function'))
    localStorage.clear()
    sessionStorage.clear()
    reject401()

    await expect(request).rejects.toBeDefined()
    expect(refresh).not.toHaveBeenCalled()
    expect(readAuthSession()).toBeNull()
  })

  it('keeps refresh locks isolated when a new session receives its own 401', async () => {
    writeAuthSession(
      { accessToken: 'lock-a-access', refreshToken: 'lock-a-refresh', user: user(1) },
      true,
    )
    let rejectARefresh!: (reason?: unknown) => void
    const refresh = vi.spyOn(axios, 'post').mockImplementation((_url, body) => {
      const refreshToken = (body as { refresh_token: string }).refresh_token
      if (refreshToken === 'lock-a-refresh') {
        return new Promise((_, reject) => { rejectARefresh = reject }) as ReturnType<typeof axios.post>
      }
      return Promise.resolve(refreshResponse('lock-b-new-access', 'lock-b-new-refresh', 2)) as ReturnType<typeof axios.post>
    })
    api.defaults.adapter = async config => {
      const auth = authorizationHeader(config.headers)
      if (auth === 'Bearer lock-b-new-access') {
        return { status: 200, statusText: 'OK', data: { identity: 'b' }, headers: {}, config }
      }
      return Promise.reject(unauthorized(config))
    }

    const requestA = api.get('/session-a').catch(error => error)
    await vi.waitFor(() => expect(rejectARefresh).toBeTypeOf('function'))
    writeAuthSession(
      { accessToken: 'lock-b-access', refreshToken: 'lock-b-refresh', user: user(2, 'admin') },
      false,
    )
    const responseB = await api.get('/session-b')
    rejectARefresh(new Error('session A refresh failed'))
    await requestA

    expect(responseB.data).toEqual({ identity: 'b' })
    expect(refresh).toHaveBeenCalledTimes(2)
    expect(readAuthSession()).toMatchObject({
      accessToken: 'lock-b-new-access',
      refreshToken: 'lock-b-new-refresh',
      user: user(2),
    })
  })

  it('does not apply a late successful refresh after logout', async () => {
    writeAuthSession(
      { accessToken: 'late-logout-access', refreshToken: 'late-logout-refresh', user: user(1) },
      true,
    )
    let resolveRefresh!: (value: unknown) => void
    vi.spyOn(axios, 'post').mockImplementation(
      () => new Promise(resolve => { resolveRefresh = resolve }) as ReturnType<typeof axios.post>,
    )
    api.defaults.adapter = async config => Promise.reject(unauthorized(config))

    const request = api.get('/refresh-then-logout').catch(error => error)
    await vi.waitFor(() => expect(resolveRefresh).toBeTypeOf('function'))
    localStorage.clear()
    sessionStorage.clear()
    resolveRefresh(refreshResponse('must-not-apply', 'must-not-apply-refresh', 1))
    await request

    expect(readAuthSession()).toBeNull()
  })

  it.each(['post', 'put', 'delete'] as const)(
    'rejects an identity-conflicting refresh without replaying the originating %s mutation',
    async (method) => {
      writeAuthSession(
        { accessToken: `identity-refresh-${method}`, refreshToken: `identity-refresh-token-${method}`, user: user(1) },
        true,
      )
      vi.spyOn(axios, 'post').mockResolvedValue(
        refreshResponse(`wrong-user-access-${method}`, `wrong-user-refresh-${method}`, 2),
      )
      const seenHeaders: Array<string | undefined> = []
      api.defaults.adapter = async config => {
        seenHeaders.push(authorizationHeader(config.headers))
        return Promise.reject(unauthorized(config))
      }

      const request = method === 'delete'
        ? api.delete(`/identity-conflict-${method}`)
        : api[method](`/identity-conflict-${method}`, { dangerous: true })

      await expect(request).rejects.toBeDefined()
      expect(seenHeaders).toEqual([`Bearer identity-refresh-${method}`])
      expect(readAuthSession()).toBeNull()
    },
  )

  it('fails closed on a malformed refresh response and notifies expiration', async () => {
    writeAuthSession(
      { accessToken: 'malformed-access', refreshToken: 'malformed-refresh', user: user(1) },
      true,
    )
    vi.spyOn(axios, 'post').mockResolvedValue({ data: { data: { access_token: 'only-access' } } })
    api.defaults.adapter = async config => Promise.reject(unauthorized(config))
    const expired = vi.fn()
    window.addEventListener('auth:expired', expired, { once: true })

    await expect(api.get('/malformed-refresh')).rejects.toBeDefined()
    expect(readAuthSession()).toBeNull()
    expect(expired).toHaveBeenCalledTimes(1)
  })
})
