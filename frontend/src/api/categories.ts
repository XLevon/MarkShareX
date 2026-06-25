import api from './index'
import type { Category } from './index'

export function fetchCategories() {
  return api.get<{ data: Category[] }>('/categories')
}

export function fetchAdminCategories() {
  return api.get<{ data: Category[] }>('/admin/categories')
}

export function createCategory(data: { name: string; description?: string; image_url?: string | null; image_filename?: string | null }) {
  return api.post<{ data: Category }>('/categories', data)
}

export function updateCategory(id: number, data: { name?: string; description?: string; image_url?: string | null; image_filename?: string | null; parent_id?: number | null; is_visible?: boolean }) {
  return api.put<{ data: Category }>(`/categories/${id}`, data)
}

export function deleteCategory(id: number) {
  return api.delete(`/categories/${id}`)
}

export function reorderCategories(ids: number[]) {
  return api.put('/admin/categories/reorder', { ids })
}
