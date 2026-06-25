import api from './index'
import type { Settings } from './index'

export function fetchSettings() {
  return api.get<{ data: Settings }>('/settings')
}

export function updateSettings(data: Partial<Settings>) {
  return api.put<{ data: Settings }>('/settings', data)
}
