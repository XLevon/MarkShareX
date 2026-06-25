import api from './index'

export function fetchSetupStatus() {
  return api.get<{ data: { initialized: boolean } }>('/setup/status')
}

export function setupSystem(data: {
  username: string
  display_name: string
  email: string
  password: string
  bio?: string
}) {
  return api.post('/setup', data)
}
