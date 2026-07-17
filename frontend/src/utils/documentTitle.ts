const META_TITLE_MAX_WIDTH = 60

const STATIC_ROUTE_TITLES: Record<string, string> = {
  home: '',
  'knowledge-base': '知识库',
  tags: '标签',
  types: '文章类型',
  statuses: '文章状态',
  categories: '分类',
  authors: '作者',
  search: '搜索结果',
  login: '登录',
  register: '注册',
  apply: '申请成为作者',
  changelog: '更新日志',
  pinned: '推荐文章',
  guestbook: '留言板',
  'not-found': '页面未找到',
}

export function displaySiteTitle(title: string | null | undefined): string {
  const displayed = (title || '').replaceAll('-', '').trim()
  return displayed || 'MarkShareX_用AI学AI'
}

function titleCharacterWidth(character: string): number {
  const codePoint = character.codePointAt(0) || 0
  if (
    (codePoint >= 0x0300 && codePoint <= 0x036f)
    || (codePoint >= 0x1ab0 && codePoint <= 0x1aff)
    || (codePoint >= 0x1dc0 && codePoint <= 0x1dff)
    || (codePoint >= 0x20d0 && codePoint <= 0x20ff)
  ) return 0

  if (
    (codePoint >= 0x1100 && codePoint <= 0x115f)
    || (codePoint >= 0x2329 && codePoint <= 0x232a)
    || (codePoint >= 0x2e80 && codePoint <= 0xa4cf)
    || (codePoint >= 0xac00 && codePoint <= 0xd7a3)
    || (codePoint >= 0xf900 && codePoint <= 0xfaff)
    || (codePoint >= 0xfe10 && codePoint <= 0xfe19)
    || (codePoint >= 0xfe30 && codePoint <= 0xfe6f)
    || (codePoint >= 0xff00 && codePoint <= 0xff60)
    || (codePoint >= 0xffe0 && codePoint <= 0xffe6)
    || (codePoint >= 0x1f300 && codePoint <= 0x1faff)
    || (codePoint >= 0x20000 && codePoint <= 0x3fffd)
  ) return 2

  return 1
}

export function titleDisplayWidth(value: string): number {
  return Array.from(value).reduce((width, character) => width + titleCharacterWidth(character), 0)
}

function compactWhitespace(value: string): string {
  return value.trim().split(/\s+/).filter(Boolean).join(' ')
}

function truncateTitle(value: string): string {
  const compact = compactWhitespace(value)
  if (titleDisplayWidth(compact) <= META_TITLE_MAX_WIDTH) return compact

  const contentLimit = META_TITLE_MAX_WIDTH - titleCharacterWidth('…')
  let result = ''
  let width = 0
  for (const character of compact) {
    const characterWidth = titleCharacterWidth(character)
    if (width + characterWidth > contentLimit) break
    result += character
    width += characterWidth
  }
  return `${result.trimEnd()}…`
}

export function buildDocumentTitle(
  pageTitle: string | null | undefined,
  rawSiteTitle: string | null | undefined,
): string {
  const page = compactWhitespace(pageTitle || '')
  const site = displaySiteTitle(rawSiteTitle)
  if (!page) return truncateTitle(site)

  const combined = `${page} - ${site}`
  return titleDisplayWidth(combined) <= META_TITLE_MAX_WIDTH
    ? combined
    : truncateTitle(page)
}

export function staticRoutePageTitle(routeName: string | null | undefined): string | undefined {
  if (!routeName) return undefined
  return STATIC_ROUTE_TITLES[routeName]
}
