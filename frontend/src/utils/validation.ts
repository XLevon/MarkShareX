/**
 * 校验 IP 地址格式（支持 IPv4 / IPv6 / CIDR）
 * 返回值：空字符串表示有效，否则返回错误提示
 */
export function validateIp(input: string): string {
  const s = input.trim()
  if (!s) return '请输入IP地址'

  // 分离 CIDR 后缀
  const slashIdx = s.lastIndexOf('/')
  let ipPart = s
  let cidrPart = ''
  if (slashIdx > 0) {
    ipPart = s.substring(0, slashIdx)
    cidrPart = s.substring(slashIdx + 1)
  }

  // 判断 IPv4 vs IPv6
  if (ipPart.includes(':')) {
    // ── IPv6 ──
    if (!isValidIPv6(ipPart)) return `无效的 IPv6 地址：${ipPart}`
    if (cidrPart) {
      const n = Number(cidrPart)
      if (!Number.isInteger(n) || n < 0 || n > 128) return 'CIDR 前缀必须在 0-128 之间'
    }
  } else if (ipPart.includes('.')) {
    // ── IPv4 ──
    if (!isValidIPv4(ipPart)) return `无效的 IPv4 地址：${ipPart}`
    if (cidrPart) {
      const n = Number(cidrPart)
      if (!Number.isInteger(n) || n < 0 || n > 32) return 'CIDR 前缀必须在 0-32 之间'
    }
  } else {
    return `无法识别的 IP 格式：${s}`
  }

  return '' // 有效
}

function isValidIPv4(ip: string): boolean {
  const re = /^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/
  return re.test(ip)
}

function isValidIPv6(ip: string): boolean {
  // 简化的 IPv6 校验：允许 :: 缩写，每段 1-4 位十六进制
  if (ip === '::') return true
  if (ip === '::1') return true

  // 展开 :: 缩写
  let expanded = ip
  if (ip.includes('::')) {
    const parts = ip.split('::')
    if (parts.length > 2) return false
    const left = parts[0] ? parts[0].split(':') : []
    const right = parts[1] ? parts[1].split(':') : []
    if (left.length + right.length >= 8) return false
    const missing = 8 - left.length - right.length
    const middle = Array(missing).fill('0')
    expanded = [...left, ...middle, ...right].join(':')
  }

  const segments = expanded.split(':')
  if (segments.length !== 8) return false

  const hexRe = /^[0-9a-fA-F]{1,4}$/
  return segments.every(s => hexRe.test(s))
}
