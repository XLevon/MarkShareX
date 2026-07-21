export function canContinueAiSession(
  sessionId: number | null,
  sessionOwnerId: number | null,
  currentUserId: number | null,
): boolean {
  if (sessionId === null) return true
  return sessionOwnerId !== null && sessionOwnerId === currentUserId
}

export interface LatestRequestGate {
  begin(): number
  isLatest(requestId: number): boolean
  invalidate(): void
}

export function createLatestRequestGate(): LatestRequestGate {
  let latestRequestId = 0

  return {
    begin() {
      latestRequestId += 1
      return latestRequestId
    },
    isLatest(requestId) {
      return requestId === latestRequestId
    },
    invalidate() {
      latestRequestId += 1
    },
  }
}
