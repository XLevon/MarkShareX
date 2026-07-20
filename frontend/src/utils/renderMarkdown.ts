// Shared safe Markdown renderer — DOMPurify + marked.
// Every call site that renders user- or LLM-generated Markdown into
// HTML via v-html MUST use this function.  Never call marked.parse()
// directly in templates or components.
import { marked, type Renderer } from 'marked'
import DOMPurify from 'dompurify'

export function renderMarkdown(markdown: string, renderer?: Renderer): string {
    const options = renderer ? { renderer } : {}
    const raw = marked.parse(markdown, { ...options, async: false }) as string
    return DOMPurify.sanitize(raw)
}
