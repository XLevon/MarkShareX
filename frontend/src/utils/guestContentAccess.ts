export function canCopyArticleContent(
  isAuthenticated: boolean,
  guestCopySetting?: string,
): boolean {
  return isAuthenticated || guestCopySetting !== 'false'
}
