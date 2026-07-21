import { describe, expect, it } from 'vitest'
import { canContinueAiSession, createLatestRequestGate } from '@/utils/aiSessionAccess'

describe('canContinueAiSession', () => {
  it('allows a new session and the authenticated owner session', () => {
    expect(canContinueAiSession(null, null, null)).toBe(true)
    expect(canContinueAiSession(11, 7, 7)).toBe(true)
  })

  it('keeps foreign sessions and unknown identities read-only', () => {
    expect(canContinueAiSession(11, 7, 8)).toBe(false)
    expect(canContinueAiSession(11, 7, null)).toBe(false)
    expect(canContinueAiSession(11, null, 7)).toBe(false)
  })
})

describe('createLatestRequestGate', () => {
  it('allows only the newest request to commit state', () => {
    const gate = createLatestRequestGate()
    const first = gate.begin()
    const second = gate.begin()

    expect(gate.isLatest(first)).toBe(false)
    expect(gate.isLatest(second)).toBe(true)
  })

  it('invalidates pending requests when the selected session is cleared', () => {
    const gate = createLatestRequestGate()
    const pending = gate.begin()
    gate.invalidate()

    expect(gate.isLatest(pending)).toBe(false)
  })
})
