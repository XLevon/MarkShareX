import { describe, it, expect } from 'vitest'
import { renderMarkdown } from '@/utils/renderMarkdown'

describe('renderMarkdown (SEC-07 XSS sanitization)', () => {
  it('renders plain markdown as HTML', () => {
    const html = renderMarkdown('**bold** and *italic*')
    expect(html).toContain('<strong>bold</strong>')
    expect(html).toContain('<em>italic</em>')
  })

  it('strips script tags', () => {
    const html = renderMarkdown('<script>alert("xss")</script>')
    expect(html).not.toContain('<script>')
    expect(html).not.toContain('alert')
  })

  it('strips onerror handlers', () => {
    const html = renderMarkdown('<img src=x onerror="alert(1)">')
    expect(html).not.toContain('onerror')
  })

  it('strips javascript: URLs', () => {
    const html = renderMarkdown('[click](javascript:alert(1))')
    expect(html).not.toContain('javascript:')
  })

  it('strips iframe elements', () => {
    const html = renderMarkdown('<iframe src="https://evil.com"></iframe>')
    expect(html).not.toContain('<iframe')
  })

  it('preserves safe markdown features', () => {
    const html = renderMarkdown(`# Title

| col1 | col2 |
|------|------|
| a    | b    |

\`\`\`js
const x = 1
\`\`\`

[link](https://example.com)`)
    expect(html).toContain('Title')
    expect(html).toContain('<table')
    expect(html).toContain('<code')
    expect(html).toContain('href="https://example.com"')
  })

  it('strips SVG event handlers', () => {
    const html = renderMarkdown('<svg onload="alert(1)"></svg>')
    expect(html).not.toContain('onload')
  })

  it('handles empty input', () => {
    const html = renderMarkdown('')
    expect(html).toBeDefined()
  })
})
