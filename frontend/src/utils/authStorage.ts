export const AUTH_STORAGE_KEYS = [
  'marksharex_token',
  'marksharex_refresh_token',
  'marksharex_user',
] as const

export interface AuthUser {
  id: number
  role: string
  [key: string]: unknown
}

export interface AuthSession<TUser extends { id: number; role: string } = AuthUser> {
  accessToken: string
  refreshToken: string
  user: TUser
}

export interface StoredAuthSession<TUser extends { id: number; role: string } = AuthUser>
  extends AuthSession<TUser> {
  storage: Storage
}

export function isAuthUser(value: unknown): value is AuthUser {
  return value !== null
    && typeof value === 'object'
    && !Array.isArray(value)
    && Number.isSafeInteger((value as { id?: unknown }).id)
    && (value as { id: number }).id > 0
    && typeof (value as { role?: unknown }).role === 'string'
    && (value as { role: string }).role.length > 0
}

function hasAuthData(storage: Storage): boolean {
  return AUTH_STORAGE_KEYS.some(key => storage.getItem(key) !== null)
}

function readCandidate<TUser extends { id: number; role: string }>(storage: Storage): StoredAuthSession<TUser> | null {
  const accessToken = storage.getItem('marksharex_token')
  const refreshToken = storage.getItem('marksharex_refresh_token')
  const rawUser = storage.getItem('marksharex_user')
  if (!accessToken || !refreshToken || rawUser === null) return null

  try {
    const user: unknown = JSON.parse(rawUser)
    if (!isAuthUser(user)) return null
    return { accessToken, refreshToken, user: user as TUser, storage }
  } catch {
    return null
  }
}

export function clearAuthSession(): void {
  for (const key of AUTH_STORAGE_KEYS) {
    localStorage.removeItem(key)
    sessionStorage.removeItem(key)
  }
}

export function readAuthSession<TUser extends { id: number; role: string } = AuthUser>(): StoredAuthSession<TUser> | null {
  const localHasData = hasAuthData(localStorage)
  const sessionHasData = hasAuthData(sessionStorage)
  if (!localHasData && !sessionHasData) return null

  const local = localHasData ? readCandidate<TUser>(localStorage) : null
  const session = sessionHasData ? readCandidate<TUser>(sessionStorage) : null

  // Authentication is one indivisible access/refresh/user tuple. Conflicting,
  // partial, or malformed state is cleared instead of guessing an identity.
  if ((localHasData && !local) || (sessionHasData && !session) || (local && session)) {
    clearAuthSession()
    return null
  }
  return local ?? session
}

export function writeAuthSession<TUser extends { id: number; role: string }>(
  session: AuthSession<TUser>,
  rememberMe: boolean,
): Storage {
  clearAuthSession()
  if (!session.accessToken || !session.refreshToken || !isAuthUser(session.user)) {
    throw new Error('Invalid authentication session')
  }

  const storage = rememberMe ? localStorage : sessionStorage
  storage.setItem('marksharex_token', session.accessToken)
  storage.setItem('marksharex_refresh_token', session.refreshToken)
  storage.setItem('marksharex_user', JSON.stringify(session.user))
  return storage
}
