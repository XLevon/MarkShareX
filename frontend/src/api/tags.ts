import api from './index'
import type { Tag } from './index'

export function fetchTags() {
  return api.get<{ data: Tag[] }>('/tags')
}

export function createTag(name: string) {
  return api.post<{ data: Tag }>('/tags', { name })
}

export function deleteTag(id: number) {
  return api.delete(`/tags/${id}`)
}
