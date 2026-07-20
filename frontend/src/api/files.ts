import api from './index'
import type { FileInfo, PaginatedData } from './index'
export type { FileInfo }

export interface Md5CheckResult {
  md5: string
  exists: boolean
  url?: string
  file_id?: number
}

export interface BatchUploadResult {
  original_name: string
  success: boolean
  url?: string
  file_id?: number
  error?: string
}

export function uploadFile(file: File, params?: { rename?: string; overwrite?: boolean }) {
  const formData = new FormData()
  formData.append('file', file)
  return api.post<{ data: FileInfo }>('/files/upload', formData, {
    params,
    timeout: 0,  // 上传不限时，避免大文件超时
  })
}

export function fetchFiles(params?: { page?: number; page_size?: number }) {
  return api.get<PaginatedData<FileInfo>>('/files', { params })
}

export function deleteFile(id: number) {
  return api.delete(`/files/${id}`)
}

export function batchDeleteFiles(ids: number[]) {
  return api.delete('/files/batch', { data: { ids } })
}

// 新增：获取未被引用的文件列表
export function fetchUnreferencedFiles() {
  return api.get<{ data: FileInfo[] }>('/files/unreferenced')
}

// 新增：检查多个 MD5 是否已存在
export function checkMd5Exists(md5List: string[]) {
  return api.post<{ data: Md5CheckResult[] }>('/files/check-md5', { md5_list: md5List })
}

// 新增：批量上传文件
export function batchUpload(files: File[]) {
  const formData = new FormData()
  files.forEach(file => formData.append('files', file))
  return api.post<{ data: BatchUploadResult[] }>('/files/batch', formData)
}

// 新增：计算文件的 MD5 哈希值
export async function calculateFileMd5(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = (e) => {
      const arrayBuffer = e.target?.result as ArrayBuffer
      const hash = md5ArrayBuffer(arrayBuffer)
      resolve(hash)
    }
    reader.onerror = reject
    reader.readAsArrayBuffer(file)
  })
}

function md5ArrayBuffer(arrayBuffer: ArrayBuffer): string {
  const k = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476]
  const s = [7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22,
             5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20,
             4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
             6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21]
  
  const bytes = new Uint8Array(arrayBuffer)
  const originalBitLength = BigInt(bytes.length) * BigInt(8)
  
  let paddingLength = (56 - (bytes.length % 64) + 64) % 64
  if (paddingLength === 0) paddingLength = 64
  
  const paddedBytes = new Uint8Array(bytes.length + paddingLength + 8)
  paddedBytes.set(bytes)
  paddedBytes[bytes.length] = 0x80
  
  const bitLengthBytes = new Uint8Array(8)
  for (let i = 0; i < 8; i++) {
    bitLengthBytes[7 - i] = Number((originalBitLength >> BigInt(i * 8)) & BigInt(0xff))
  }
  paddedBytes.set(bitLengthBytes, paddedBytes.length - 8)
  
  const words = new Uint32Array(Math.ceil(paddedBytes.length / 4))
  for (let i = 0; i < words.length; i++) {
    for (let j = 0; j < 4; j++) {
      words[i] |= (paddedBytes[i * 4 + j] << (j * 8)) >>> 0
    }
  }
  
  let a = k[0], b = k[1], c = k[2], d = k[3]
  
  for (let i = 0; i < words.length; i += 16) {
    const AA = a, BB = b, CC = c, DD = d
    for (let j = 0; j < 64; j++) {
      let f: number, g: number
      if (j < 16) {
        f = (b & c) | (~b & d)
        g = j
      } else if (j < 32) {
        f = (d & b) | (~d & c)
        g = (5 * j + 1) % 16
      } else if (j < 48) {
        f = b ^ c ^ d
        g = (3 * j + 5) % 16
      } else {
        f = c ^ (b | ~d)
        g = (7 * j) % 16
      }
      const temp = d
      d = c
      c = b
      b = (b + leftRotate((a + f + words[i + g] + k[j % 4]) >>> 0, s[j])) >>> 0
      a = temp
    }
    a = (a + AA) >>> 0
    b = (b + BB) >>> 0
    c = (c + CC) >>> 0
    d = (d + DD) >>> 0
  }
  
  function leftRotate(n: number, s: number): number {
    return ((n << s) | (n >>> (32 - s))) >>> 0
  }
  
  const h = [a, b, c, d]
  return h.map(x => x.toString(16).padStart(8, '0')).join('')
}
