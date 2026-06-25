import JSZip from 'jszip'
import { saveAs } from 'file-saver'
import api from './index'

export interface ExportRequest {
  post_ids?: number[]
  include_drafts?: boolean
}

export interface ImportItem {
  filename: string
  content: string
  images: Array<{
    data: string
    name: string
  }>
}

export interface ImportResult {
  success: boolean
  message: string
  imported_count: number
  skipped_count: number
  errors: string[]
}

export async function exportPosts(postIds?: number[]): Promise<void> {
  const req: ExportRequest = {
    post_ids: postIds,
    include_drafts: true
  }
  
  const response = await api.post('/export/posts', req, {
    responseType: 'blob',
    headers: { 'Content-Type': 'application/json' }
  })
  
  const blob = new Blob([response.data], { type: 'application/zip' })
  const contentDisposition = response.headers['content-disposition'] || response.headers['Content-Disposition']
  let filename = 'marksharex_export.zip'
  
  if (contentDisposition) {
    const match = contentDisposition.match(/filename[^;=\n]*=((['"]).*?\2|[^;\n]*)/)
    if (match != null && match[1]) {
      filename = match[1].replace(/['"]/g, '')
    }
  }
  
  saveAs(blob, filename)
}

export async function importPosts(items: ImportItem[]): Promise<ImportResult> {
  const response = await api.post('/import/posts', { items })
  return response.data.data
}

export async function parseZipAndImport(zipFile: File): Promise<ImportResult> {
  const zip = await JSZip.loadAsync(zipFile)
  const items: ImportItem[] = []
  
  for (const [path, file] of Object.entries(zip.files)) {
    if (file.dir) continue
    
    const relativePath = path
    const ext = relativePath.split('.').pop()?.toLowerCase()
    
    if (ext === 'md') {
      let content = await file.async('string')
      const dirName = relativePath.substring(0, relativePath.lastIndexOf('/'))
      
      const images: Array<{ data: string; name: string }> = []
      
      // 提取Markdown文件中引用的图片
      const referencedImages = extractReferencedImages(content)
      
      for (const [imgPath, imgFile] of Object.entries(zip.files)) {
        if (imgFile.dir) continue
        if (imgPath.startsWith(dirName + '/uploads/') && !imgPath.endsWith('.md')) {
          const imgData = await imgFile.async('base64')
          const imgExt = imgPath.split('.').pop()?.toLowerCase() || 'png'
          const imgName = imgPath.split('/').pop() || `image_${Date.now()}.${imgExt}`
          
          // 只添加被引用的图片，使用完整引用路径作为名称以正确替换
          const matchedRef = referencedImages.find(ref => ref.includes(imgName))
          if (matchedRef) {
            let imageData = `data:image/${imgExt};base64,${imgData}`
            // 特殊处理SVG
            if (imgExt === 'svg') {
              imageData = `data:image/svg+xml;base64,${imgData}`
            }
            images.push({
              data: imageData,
              name: matchedRef  // 使用完整引用路径，如 ./uploads/xxx.svg
            })
          }
        }
      }
      
      items.push({
        filename: relativePath.split('/').pop() || 'untitled.md',
        content,
        images
      })
    }
  }
  
  if (items.length === 0) {
    return {
      success: false,
      message: '未找到Markdown文件',
      imported_count: 0,
      skipped_count: 0,
      errors: ['ZIP包中没有发现.md文件']
    }
  }
  
  return importPosts(items)
}

export async function importFromFiles(files: File[]): Promise<ImportResult> {
  const mdFiles = files.filter(f => f.name.endsWith('.md') || f.name.endsWith('.markdown'))
  const imageFiles = files.filter(f => f.type.startsWith('image/'))
  
  if (mdFiles.length === 0) {
    return {
      success: false,
      message: '未找到Markdown文件',
      imported_count: 0,
      skipped_count: 0,
      errors: ['请至少选择一个.md文件']
    }
  }
  
  const items: ImportItem[] = []
  
  for (const mdFile of mdFiles) {
    let content = await mdFile.text()
    const images: Array<{ data: string; name: string }> = []
    
    // 提取Markdown文件中引用的图片
    const referencedImages = extractReferencedImages(content)
    
    // 只添加被引用的图片，使用完整引用路径作为名称以正确替换
    for (const imgFile of imageFiles) {
      const imgFilename = imgFile.name
      const matchedRef = referencedImages.find(ref => ref.includes(imgFilename))
      if (matchedRef) {
        const imgData = await imgFile.arrayBuffer()
        const base64 = btoa(String.fromCharCode(...new Uint8Array(imgData)))
        const ext = imgFile.name.split('.').pop()?.toLowerCase() || 'png'
        let imageData = `data:image/${ext};base64,${base64}`
        // 特殊处理SVG
        if (ext === 'svg') {
          imageData = `data:image/svg+xml;base64,${base64}`
        }
        images.push({
          data: imageData,
          name: matchedRef  // 使用完整引用路径
        })
      }
    }
    
    items.push({
      filename: mdFile.name,
      content,
      images
    })
  }
  
  return importPosts(items)
}

// 提取Markdown文件中引用的图片
function extractReferencedImages(content: string): string[] {
  const regex = /!\[.*?\]\(([^)]+)\)/g
  const references: string[] = []
  let match
  
  while ((match = regex.exec(content)) !== null) {
    const url = match[1]
    // 只处理相对路径的图片
    if (!url.startsWith('http://') && !url.startsWith('https://')) {
      references.push(url)
    }
  }
  
  return references
}
