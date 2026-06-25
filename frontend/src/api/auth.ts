import api from './index'

export function register(username: string, email: string, password: string, display_name?: string, rememberMe?: boolean) {
  return api.post('/auth/register', { username, email, password, display_name, remember_me: rememberMe })
}

export function login(username: string, password: string, rememberMe?: boolean) {
  return api.post('/auth/login', { username, password, remember_me: rememberMe })
}

export function refreshToken(refresh_token: string) {
  return api.post('/auth/refresh', { refresh_token })
}
