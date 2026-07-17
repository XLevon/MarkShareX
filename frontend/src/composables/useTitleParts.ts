import { computed } from 'vue'

export interface TitlePart {
  text: string
  color: string
}

/** Match the server-side browser title rule: remove all ASCII hyphens. */
export function displaySiteTitle(title: string | null | undefined): string {
  const displayed = (title || '').replaceAll('-', '').trim()
  return displayed || 'MarkShareX'
}

/**
 * Split a title by "-" and assign colors to each part.
 * - Part 1: fixed blue (--color-primary)
 * - Part 2: theme-dependent text color
 * - Part 3: accent warm color
 * - Part 4+: cycle text/accent
 */
export function useTitleParts(title: () => string, isDark: () => boolean) {
  return computed<TitlePart[]>(() => {
    const t = title() || ''
    // 提取 _ 前面的部分作为 Logo 文字，去掉连字符后颜色化显示
    const logoPart = t.split('_')[0]
    const parts = logoPart.split('-').filter(s => s.length > 0)
    if (parts.length === 0) return [{ text: t || 'MarkShareX', color: 'var(--color-primary)' }]
    if (parts.length === 1) return [{ text: parts[0], color: 'var(--color-primary)' }]

    const textColor = isDark() ? '#ffffff' : '#1a1a2e'
    const accentColor = '#f59e0b'

    return parts.map((text, i) => {
      if (i === 0) return { text, color: 'var(--color-primary)' }
      if (i === 1) return { text, color: textColor }
      if (i === 2) return { text, color: accentColor }
      // i >= 3: cycle between textColor and accentColor
      return { text, color: (i % 2 === 1) ? textColor : accentColor }
    })
  })
}
