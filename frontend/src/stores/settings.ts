import { defineStore } from 'pinia'
import { ref, reactive, computed } from 'vue'
import { fetchSettings as apiFetchSettings, updateSettings as apiUpdateSettings } from '@/api/settings'
import { fetchNetworkResources } from '@/api/admin'
import type { Settings } from '@/api/index'

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<Settings>({
    site_title: 'MarkShareX',
    site_subtitle: '',
    site_description: 'A Markdown Blog',
    site_logo: '',
    friend_links: '[]',
    comment_moderation: 'false',
    sidebar_collapse: 'false',
    guestbook_enabled: 'true',
    guest_copy_enabled: 'true',
    batch_load_size: '5',
    scroll_load_size: '3',
    'site-manager': '',
  })
  const loaded = ref(false)

  // 网络资源缓存：id → URL，用于 nr:{id} 引用解析
  const networkUrlCache = reactive(new Map<number, string>())

  // 解析后的 logo URL：处理 nr:{id}、相对路径、完整 URL
  const resolvedLogoUrl = computed(() => {
    const logo = settings.value.site_logo
    if (!logo) return ''
    if (logo.startsWith('http')) return logo
    if (logo.startsWith('nr:')) {
      const id = Number(logo.slice(3))
      return networkUrlCache.get(id) || ''
    }
    return logo.startsWith('/') ? logo : `/${logo}`
  })

  async function fetchSettings() {
    try {
      const { data: resp } = await apiFetchSettings()
      settings.value = resp.data.settings || (resp.data as any)
      loaded.value = true
    } catch {
      // use defaults
    }
    // 预热网络资源缓存
    try {
      const { data: resp } = await fetchNetworkResources({ page_size: 500, source_type: 'image' })
      ;(resp.data || []).forEach((nr: any) => {
        networkUrlCache.set(nr.id, nr.url)
      })
    } catch { /* 网络资源不可用时忽略 */ }
  }

  async function updateSettings(data: Partial<Settings>) {
    const { data: resp } = await apiUpdateSettings(data as any)
    settings.value = (resp.data as any).settings || resp.data
  }

  return { settings, loaded, resolvedLogoUrl, networkUrlCache, fetchSettings, updateSettings }
})
