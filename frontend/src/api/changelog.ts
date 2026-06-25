import api from './index'

export interface ChangelogEntry {
  id: number
  version: string
  content: string
  created_at: string
  updated_at: string
}

// Admin CRUD
export function fetchChangelogs(params?: { page?: number; page_size?: number }) {
  return api.get<{ data: ChangelogEntry[]; pagination: any }>('/changelogs', { params })
}

export function createChangelog(data: { version?: string; content: string }) {
  return api.post<{ data: ChangelogEntry }>('/changelogs', data)
}

export function updateChangelog(id: number, data: { version?: string; content?: string }) {
  return api.put<{ data: ChangelogEntry }>(`/changelogs/${id}`, data)
}

export function deleteChangelog(id: number) {
  return api.delete(`/changelogs/${id}`)
}

// Public
export function fetchLatestVersion() {
  return api.get<{ data: ChangelogEntry | null }>('/changelogs/latest')
}

export function fetchPublicChangelogs() {
  return api.get<{ data: ChangelogEntry[] }>('/changelogs/public')
}
