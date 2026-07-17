import { toValue, watchEffect, type MaybeRefOrGetter } from 'vue'
import { useSettingsStore } from '@/stores/settings'
import { buildDocumentTitle } from '@/utils/documentTitle'

/**
 * Keep the settled browser title in sync with reactive page data and site settings.
 * Passing undefined leaves the server-rendered title untouched until dynamic data loads.
 */
export function useDocumentTitle(pageTitle: MaybeRefOrGetter<string | null | undefined>) {
  const settingsStore = useSettingsStore()

  watchEffect(() => {
    const page = toValue(pageTitle)
    if (page === undefined) return
    document.title = buildDocumentTitle(page, settingsStore.settings.site_title)
  })
}
