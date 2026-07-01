<template>
  <div class="admin-layout" :style="{ 
    background: isDark ? '#0f0f13' : '#f9fafb', 
    color: isDark ? '#e0e0e0' : '#1a1a2e',
    '--admin-nav-color': isDark ? '#9ca3af' : '#4b5563',
    '--admin-nav-hover-color': isDark ? '#e0e0e0' : '#1f2937',
    '--admin-nav-active-color': isDark ? '#e0e0e0' : '#1f2937',
    '--admin-nav-active-bg': isDark ? 'rgba(79,70,229,0.12)' : 'rgba(79,70,229,0.08)',
    '--admin-nav-icon-opacity': isDark ? '0.7' : '0.55',
    '--card-bg': isDark ? '#16161d' : '#ffffff',
    '--card-border': isDark ? '1px solid rgba(255,255,255,0.06)' : '1px solid #e5e7eb',
    '--card-border-color': isDark ? 'rgba(255,255,255,0.12)' : '#d1d5db',
    '--modal-bg': isDark ? '#1e1e2a' : '#ffffff',
    '--input-bg': isDark ? '#0f0f13' : '#ffffff',
    '--input-color': isDark ? '#e0e0e0' : '#1a1a2e',
    '--filter-bg': isDark ? '#16161d' : '#f3f4f6',
    '--chart-bg': isDark ? '#1e1e2a' : '#f3f4f6',
    '--storage-bar-bg': isDark ? 'rgba(255,255,255,0.06)' : '#d1d5db',
    '--text-primary': isDark ? '#e0e0e0' : '#1a1a2e',
    '--text-secondary': isDark ? '#9ca3af' : '#4b5563',
    '--text-dim': isDark ? '#9ca3af' : '#6b7280',
  }">
    <!-- 顶部导航栏 + 移动端菜单（sticky 包裹） -->
    <div class="sticky top-0 z-50">
      <NavBar
        :logo-text="settingsStore.settings.site_title || 'MarkShareX'"
        :logo-image="settingsStore.resolvedLogoUrl"
        :is-dark="isDark"
        :is-logged-in="true"
        :display-user="displayUserName"
        :user-initial="userInitial"
        :header-style="adminNavStyle"
        :sticky="false"
        @toggle-dark="toggleDark"
      >
        <template #nav-left>
          <!-- 汉堡菜单按钮（仅手机端） -->
          <button
            class="lg:hidden w-10 h-10 flex items-center justify-center rounded-lg border-0 cursor-pointer transition-colors hover:bg-white/5"
            :style="{ color: isDark ? '#9ca3af' : '#4b5563' }"
            @click="mobileMenuOpen = !mobileMenuOpen"
          >
            <svg v-if="!mobileMenuOpen" width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="3" y1="6" x2="21" y2="6"/><line x1="3" y1="12" x2="21" y2="12"/><line x1="3" y1="18" x2="21" y2="18"/></svg>
            <svg v-else width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
          </button>
          <!-- 桌面端导航栏 -->
          <nav class="!hidden lg:!flex admin-nav-links">
            <router-link
              v-for="item in navItems"
              :key="item.key"
              :to="item.to"
              class="admin-nav-link flex-shrink-0"
              :class="{ active: isActive(item.key) }"
            >
              <span class="nav-icon" v-html="item.icon"></span>
              <span>{{ item.label }}</span>
              <span v-if="item.badge" class="nav-badge">{{ item.badge }}</span>
            </router-link>
          </nav>
        </template>
      <template #dropdown-items>
        <button class="dd-btn" :style="{ color: modalTheme.ddBtnColor }" @mouseenter="($event.target as HTMLElement).style.background = modalTheme.ddBtnHoverBg" @mouseleave="($event.target as HTMLElement).style.background = 'transparent'" @click="showProfile = true">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
          个人信息
        </button>
        <button class="dd-btn" :style="{ color: modalTheme.ddBtnColor }" @mouseenter="($event.target as HTMLElement).style.background = modalTheme.ddBtnHoverBg" @mouseleave="($event.target as HTMLElement).style.background = 'transparent'" @click="openPassword()">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
          修改密码
        </button>
        <button v-if="currentUserRole === 'admin'" class="dd-btn" :style="{ color: modalTheme.ddBtnColor }" @mouseenter="($event.target as HTMLElement).style.background = modalTheme.ddBtnHoverBg" @mouseleave="($event.target as HTMLElement).style.background = 'transparent'" @click="openApiKey()">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4"/></svg>
          API Key
        </button>
        <button v-if="currentUserRole === 'admin'" class="dd-btn" :style="{ color: modalTheme.ddBtnColor }" @mouseenter="($event.target as HTMLElement).style.background = modalTheme.ddBtnHoverBg" @mouseleave="($event.target as HTMLElement).style.background = 'transparent'" @click="openApiDocs()">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg>
          API 文档
        </button>
        <button class="dd-btn dd-danger" :style="{ color: modalTheme.ddDangerColor }" @mouseenter="($event.target as HTMLElement).style.background = modalTheme.ddDangerHoverBg" @mouseleave="($event.target as HTMLElement).style.background = 'transparent'" @click="handleLogout()">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" y1="12" x2="9" y2="12"/></svg>
          退出登录
        </button>
      </template>
    </NavBar>
    </div>

    <!-- 移动端导航悬浮层 -->
    <template v-if="mobileMenuOpen">
      <!-- 半透明遮罩 -->
      <div class="fixed inset-0 z-40" style="background: rgba(0,0,0,0.4)" @click="mobileMenuOpen = false"></div>
      <!-- 菜单面板（全部导航项平铺） -->
      <div
        class="fixed top-0 left-0 right-0 z-50 shadow-2xl"
        :style="{ borderColor: isDark ? 'rgba(255,255,255,0.06)' : '#e5e7eb', backgroundColor: isDark ? '#16161d' : '#ffffff', marginTop: '64px' }"
      >
        <nav class="grid grid-cols-3 gap-2 p-4">
          <router-link
            v-for="item in navItems"
            :key="item.key"
            :to="item.to"
            class="relative flex flex-col items-center justify-center gap-1.5 py-3 px-1 rounded-xl text-xs font-medium no-underline transition-colors"
            :style="isActive(item.key)
              ? { color: '#e0e0e0', backgroundColor: 'rgba(79,70,229,0.15)' }
              : { color: isDark ? '#9ca3af' : '#4b5563' }"
            @click="mobileMenuOpen = false"
          >
            <span class="relative flex items-center justify-center w-9 h-9 rounded-lg" :style="isActive(item.key) ? { backgroundColor: 'rgba(79,70,229,0.2)' } : { backgroundColor: isDark ? 'rgba(255,255,255,0.04)' : 'rgba(0,0,0,0.03)' }">
              <span v-html="item.icon" :style="isActive(item.key) ? { opacity: 1 } : { opacity: 0.65 }"></span>
              <span v-if="item.badge" class="absolute -top-1.5 -right-1.5 min-w-[16px] h-4 px-1 rounded-full bg-red-500 text-white text-[10px] font-bold leading-4 text-center">{{ item.badge }}</span>
            </span>
            <span class="leading-tight">{{ item.label }}</span>
          </router-link>
        </nav>
      </div>
    </template>

    <!-- 内容区域 -->
    <main class="admin-main">
      <router-view v-slot="{ Component }">
        <keep-alive :include="['PostList']">
          <component :is="Component" />
        </keep-alive>
      </router-view>
    </main>

    <!-- ==================== 个人信息弹窗 ==================== -->
    <div v-if="showProfile" class="modal-overlay" :style="{ background: modalTheme.overlayBg }">
      <div class="modal-box modal-profile" :style="{ background: modalTheme.boxBg, border: modalTheme.boxBorder, boxShadow: modalTheme.boxShadow }">
        <h3 :style="{ color: modalTheme.titleColor }">个人信息</h3>
        <ProfileView mode="modal" @close="showProfile = false" @saved="onProfileSaved" />
      </div>
    </div>

    <!-- ==================== 修改密码弹窗 ==================== -->
    <div v-if="showPassword" class="modal-overlay" :style="{ background: modalTheme.overlayBg }">
      <div class="modal-box modal-password" :style="{ background: modalTheme.boxBg, border: modalTheme.boxBorder, boxShadow: modalTheme.boxShadow }">
        <h3 :style="{ color: modalTheme.titleColor }">修改密码</h3>
        <form @submit.prevent="handlePasswordSave" class="modal-form">
          <div class="form-row">
            <label :style="{ color: modalTheme.labelColor }">旧密码</label>
            <input v-model="pwForm.old_password" type="password" class="form-input" placeholder="输入当前密码" required :style="{ background: modalTheme.inputBg, color: modalTheme.inputColor, border: modalTheme.inputBorder }" @focus="($event.target as HTMLInputElement).style.borderColor = modalTheme.inputFocusBorder" @blur="($event.target as HTMLInputElement).style.borderColor = modalTheme.inputBorder" />
          </div>
          <div class="form-row">
            <label :style="{ color: modalTheme.labelColor }">新密码</label>
            <input v-model="pwForm.new_password" type="password" class="form-input" placeholder="至少 8 位" required minlength="8" :style="{ background: modalTheme.inputBg, color: modalTheme.inputColor, border: modalTheme.inputBorder }" @focus="($event.target as HTMLInputElement).style.borderColor = modalTheme.inputFocusBorder" @blur="($event.target as HTMLInputElement).style.borderColor = modalTheme.inputBorder" />
          </div>
          <div class="form-row">
            <label :style="{ color: modalTheme.labelColor }">确认新密码</label>
            <input v-model="pwForm.confirm_password" type="password" class="form-input" placeholder="再次输入新密码" required minlength="8" :style="{ background: modalTheme.inputBg, color: modalTheme.inputColor, border: modalTheme.inputBorder }" @focus="($event.target as HTMLInputElement).style.borderColor = modalTheme.inputFocusBorder" @blur="($event.target as HTMLInputElement).style.borderColor = modalTheme.inputBorder" />
          </div>
          <div v-if="pwError" class="form-error" :style="{ background: modalTheme.errorBg, color: modalTheme.errorColor }">{{ pwError }}</div>
          <div v-if="pwOk" class="form-ok" :style="{ background: modalTheme.okBg, color: modalTheme.okColor }">{{ pwOk }}</div>
          <div class="modal-actions">
            <button type="button" class="btn-secondary" :style="{ border: modalTheme.btnSecBorder, color: modalTheme.btnSecColor }" @mouseenter="($event.target as HTMLElement).style.color = modalTheme.btnSecHoverColor; ($event.target as HTMLElement).style.borderColor = modalTheme.btnSecHoverBorder" @mouseleave="($event.target as HTMLElement).style.color = modalTheme.btnSecColor; ($event.target as HTMLElement).style.borderColor = modalTheme.btnSecBorder" @click="showPassword = false">取消</button>
            <button type="submit" class="btn-primary" :disabled="pwSaving">{{ pwSaving ? '保存中...' : '修改密码' }}</button>
          </div>
        </form>
      </div>
    </div>

    <!-- ==================== API Key 弹窗 ==================== -->
    <div v-if="showApiKey" class="modal-overlay" :style="{ background: modalTheme.overlayBg }" @click.self="showApiKey = false">
      <div class="modal-box modal-apikey" :style="{ background: modalTheme.boxBg, border: modalTheme.boxBorder, boxShadow: modalTheme.boxShadow }">
        <h3 :style="{ color: modalTheme.titleColor }">API Key</h3>
        <p class="modal-desc" :style="{ color: modalTheme.labelColor }">用于 AI 工具通过 <code :style="{ background: modalTheme.inputBg, padding: '2px 6px', borderRadius: '4px' }">X-API-Key</code> 头访问 API，权限等同你的账号。</p>
        <div v-if="apiKeyLoading" class="modal-loading"><div class="spinner" :style="{ border: modalTheme.spinnerBorder, borderTopColor: modalTheme.spinnerTop }"></div></div>
        <div v-else class="modal-form">
          <div v-if="apiKeyError" class="form-error" :style="{ background: modalTheme.errorBg, color: modalTheme.errorColor }">{{ apiKeyError }}</div>
          <div v-if="apiKeyOk" class="form-ok" :style="{ background: modalTheme.okBg, color: modalTheme.okColor }">{{ apiKeyOk }}</div>

          <div v-if="apiKeyValue">
            <span :style="{ color: modalTheme.labelColor, fontSize: '13px' }">当前 Key</span>
            <div class="apikey-display" :style="{ background: modalTheme.inputBg, border: modalTheme.inputBorder, borderRadius: '8px', padding: '12px 16px', marginBottom: '16px', marginTop: '8px' }">
              <code class="apikey-value" :style="{ color: modalTheme.titleColor, fontSize: '13px', wordBreak: 'break-all', fontFamily: 'monospace' }">{{ apiKeyValue }}</code>
            </div>
          </div>
          <div v-else class="apikey-empty" :style="{ color: modalTheme.labelColor, textAlign: 'center', padding: '24px', fontSize: '14px' }">
            暂未生成 API Key
          </div>

          <div v-if="apiKeyOk" class="form-ok" :style="{ background: modalTheme.okBg, color: modalTheme.okColor, marginTop: '12px' }">{{ apiKeyOk }}</div>

          <div class="modal-actions">
            <button v-if="apiKeyValue" type="button" class="btn-copy mr-auto" @click="copyApiKey()" :style="{ fontSize: '12px', padding: '2px 10px', border: modalTheme.btnSecBorder, borderRadius: '4px', color: modalTheme.btnSecColor, background: 'transparent', cursor: 'pointer' }">{{ copied ? '已复制 ✓' : '复制' }}</button>
            <button type="button" class="btn-secondary" :style="{ border: modalTheme.btnSecBorder, color: modalTheme.btnSecColor }" @mouseenter="($event.target as HTMLElement).style.color = modalTheme.btnSecHoverColor; ($event.target as HTMLElement).style.borderColor = modalTheme.btnSecHoverBorder" @mouseleave="($event.target as HTMLElement).style.color = modalTheme.btnSecColor; ($event.target as HTMLElement).style.borderColor = modalTheme.btnSecBorder" @click="showApiKey = false">关闭</button>
            <button type="button" class="btn-primary" :disabled="apiKeyRegenerating" @click="handleRegenerateApiKey()">{{ apiKeyRegenerating ? '生成中...' : (apiKeyValue ? '重新生成' : '生成 Key') }}</button>
          </div>
        </div>
      </div>
    </div>
  </div>

  <AiChatWidget />
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useSettingsStore } from '@/stores/settings'
import { changePassword, getPendingCount, getCommentPendingCount, fetchApiKey, regenerateApiKey } from '@/api/admin'
import { fetchAdminPosts } from '@/api/posts'
import NavBar from '@/components/shared/NavBar.vue'
import ProfileView from '@/views/admin/Profile.vue'

import { useDarkMode } from '@/composables/useDarkMode'
import AiChatWidget from '@/components/shared/AiChatWidget.vue'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()
const settingsStore = useSettingsStore()
const { isDark, toggleDarkMode, initDarkMode } = useDarkMode()

const adminNavStyle = computed(() => ({
  backgroundColor: isDark.value ? '#16161d' : '#ffffff',
  borderColor: isDark.value ? 'rgba(255, 255, 255, 0.06)' : '#e5e7eb',
  boxShadow: isDark.value ? 'none' : '0 1px 3px rgba(0,0,0,0.06)',
}))

const modalTheme = computed(() => ({
  // Overlay
  overlayBg: isDark.value ? 'rgba(0,0,0,0.6)' : 'rgba(0,0,0,0.25)',
  // Box
  boxBg: isDark.value ? '#1e1e2a' : '#ffffff',
  boxBorder: isDark.value ? '1px solid rgba(255,255,255,0.08)' : '1px solid #e5e7eb',
  boxShadow: isDark.value ? '0 12px 40px rgba(0,0,0,0.4)' : '0 4px 24px rgba(0,0,0,0.08)',
  // Title
  titleColor: isDark.value ? '#f0f0f0' : '#1a1a2e',
  // Form
  inputBg: isDark.value ? '#0f0f13' : '#ffffff',
  inputColor: isDark.value ? '#e0e0e0' : '#1a1a2e',
  inputBorder: isDark.value ? '1px solid rgba(255,255,255,0.08)' : '1px solid #d1d5db',
  inputFocusBorder: isDark.value ? 'rgba(79,70,229,0.4)' : '#4f46e5',
  labelColor: isDark.value ? '#9ca3af' : '#6b7280',
  placeholderColor: isDark.value ? '#4b5563' : '#9ca3af',
  // DD Buttons
  ddBtnColor: isDark.value ? '#d1d5db' : '#1f2937',
  ddBtnHoverBg: isDark.value ? 'rgba(255,255,255,0.05)' : 'rgba(0,0,0,0.04)',
  ddDangerColor: isDark.value ? '#f87171' : '#dc2626',
  ddDangerHoverBg: isDark.value ? 'rgba(248,113,113,0.1)' : 'rgba(220,38,38,0.06)',
  // Secondary btn
  btnSecColor: isDark.value ? '#9ca3af' : '#6b7280',
  btnSecHoverColor: isDark.value ? '#e0e0e0' : '#374151',
  btnSecHoverBorder: isDark.value ? 'rgba(255,255,255,0.15)' : '#9ca3af',
  btnSecBorder: isDark.value ? '1px solid rgba(255,255,255,0.08)' : '1px solid #d1d5db',
  // Error/OK
  errorBg: isDark.value ? 'rgba(239,68,68,0.08)' : 'rgba(239,68,68,0.06)',
  errorColor: isDark.value ? '#f87171' : '#dc2626',
  okBg: isDark.value ? 'rgba(16,185,129,0.08)' : 'rgba(16,185,129,0.06)',
  okColor: isDark.value ? '#34d399' : '#059669',
  // Spinner
  spinnerBorder: isDark.value ? '3px solid rgba(79,70,229,0.15)' : '3px solid rgba(79,70,229,0.12)',
  spinnerTop: '#4f46e5',
}))

// Display name: from auth store (reacts to login/logout across both storages)
const displayUserName = computed(() => {
  const u = authStore.user
  if (u) return u.display_name || u.username || '用户'
  // Fallback to storage in case store hasn't hydrated
  try {
    const stored = localStorage.getItem('marksharex_user') || sessionStorage.getItem('marksharex_user')
    if (stored) {
      const parsed = JSON.parse(stored)
      return parsed.display_name || parsed.username || '用户'
    }
  } catch {}
  return '用户'
})

const username = computed(() => displayUserName.value)
const userInitial = computed(() => (username.value || '管')[0])

function isActive(key: string) {
  if (key === 'dashboard') return route.name === 'admin-dashboard'
  if (key === 'posts') return String(route.name).startsWith('admin-post')
  if (key === 'files') return route.name === 'admin-files'
  if (key === 'import') return route.name === 'admin-import'
  if (key === 'comments') return route.name === 'admin-analytics-comments'
  if (key === 'users') return route.name === 'admin-users'
  if (key === 'settings') return route.name === 'admin-settings'
  if (key === 'guestbook') return route.name === 'admin-guestbook'
  return false
}

function toggleDark() {
  toggleDarkMode()
}

function handleLogout() {
  authStore.logout()
  router.push('/')
}

// ── Profile modal ──
const mobileMenuOpen = ref(false)
const showProfile = ref(false)
function onProfileSaved() {
  // Refresh auth store so top-right display_name syncs
  const stored = localStorage.getItem('marksharex_user') || sessionStorage.getItem('marksharex_user')
  if (stored) {
    try {
      const u = JSON.parse(stored)
      const target = localStorage.getItem('marksharex_user') ? localStorage : sessionStorage
      target.setItem('marksharex_user', JSON.stringify(u))
    } catch {}
  }
}

// ── Password modal ──
const showPassword = ref(false)
const pwSaving = ref(false)
const pwError = ref('')
const pwOk = ref('')
const pwForm = reactive({ old_password: '', new_password: '', confirm_password: '' })

function openPassword() {
  showPassword.value = true
  pwError.value = ''
  pwOk.value = ''
  pwForm.old_password = ''
  pwForm.new_password = ''
  pwForm.confirm_password = ''
}

async function handlePasswordSave() {
  pwError.value = ''
  pwOk.value = ''

  if (pwForm.new_password !== pwForm.confirm_password) {
    pwError.value = '两次输入的新密码不一致'
    return
  }

  pwSaving.value = true
  try {
    await changePassword({
      old_password: pwForm.old_password,
      new_password: pwForm.new_password,
      confirm_password: pwForm.confirm_password,
    })
    pwOk.value = '密码修改成功，即将关闭...'
    setTimeout(() => { showPassword.value = false }, 2000)
  } catch (e: any) {
    pwError.value = e?.response?.data?.error || '修改失败'
  } finally {
    pwSaving.value = false
  }
}

// ── API Key modal ──
const showApiKey = ref(false)
const apiKeyLoading = ref(false)
const apiKeyRegenerating = ref(false)
const apiKeyError = ref('')
const apiKeyOk = ref('')
const apiKeyValue = ref('')
const copied = ref(false)

async function openApiKey() {
  showApiKey.value = true
  apiKeyError.value = ''
  apiKeyOk.value = ''
  apiKeyLoading.value = true
  copied.value = false
  try {
    const { data } = await fetchApiKey()
    apiKeyValue.value = data.data.api_key || ''
  } catch {
    apiKeyError.value = '获取失败'
  } finally {
    apiKeyLoading.value = false
  }
}

function openApiDocs() {
  // Set short-lived cookie (60s) scoped to /scalar, then navigate
  document.cookie = `scalar_token=${authStore.token}; path=/scalar; max-age=60; SameSite=Strict`
  window.open('/scalar', '_blank')
}

async function handleRegenerateApiKey() {
  apiKeyRegenerating.value = true
  apiKeyError.value = ''
  apiKeyOk.value = ''
  copied.value = false
  try {
    const { data } = await regenerateApiKey()
    apiKeyValue.value = data.data.api_key || ''
    apiKeyOk.value = '新的 API Key 已生成，请妥善保存！'
  } catch (e: any) {
    apiKeyError.value = e?.response?.data?.error || '生成失败'
  } finally {
    apiKeyRegenerating.value = false
  }
}

function copyApiKey() {
  if (!apiKeyValue.value) return
  // Try modern API first, fallback to execCommand for non-HTTPS
  if (navigator.clipboard && window.isSecureContext) {
    navigator.clipboard.writeText(apiKeyValue.value).then(() => {
      copied.value = true
      setTimeout(() => { copied.value = false }, 2000)
    }).catch(() => {
      apiKeyError.value = '复制失败，请手动复制'
    })
  } else {
    // Fallback for HTTP environments
    const ta = document.createElement('textarea')
    ta.value = apiKeyValue.value
    ta.style.cssText = 'position:fixed;opacity:0;'
    document.body.appendChild(ta)
    ta.select()
    try {
      document.execCommand('copy')
      copied.value = true
      setTimeout(() => { copied.value = false }, 2000)
    } catch {
      apiKeyError.value = '复制失败，请手动复制'
    }
    document.body.removeChild(ta)
  }
}

// SVG icons
const dashboardIcon = '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/></svg>'
const postsIcon = '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg>'
const filesIcon = '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>'
const importIcon = '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>'
const usersIcon = '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>'
const settingsIcon = '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/></svg>'
const guestbookIcon = '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"/></svg>'
const newsIcon = '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 22h16a2 2 0 0 0 2-2V4a2 2 0 0 0-2-2H8a2 2 0 0 0-2 2v16a2 2 0 0 1-2 2zm0 0a2 2 0 0 1-2-2v-9c0-1.1.9-2 2-2h2"/><path d="M18 14h-8"/><path d="M15 18h-5"/><path d="M10 6h8v4h-8z"/></svg>'
const aiIcon = '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2a4 4 0 0 1 4 4v1h2a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V9a2 2 0 0 1 2-2h2V6a4 4 0 0 1 4-4z"/><circle cx="9" cy="13" r="1"/><circle cx="15" cy="13" r="1"/><path d="M9 17c.85.63 1.89 1 3 1s2.15-.37 3-1"/></svg>'
const changelogIcon = '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><path d="M12 18v-6"/><path d="M9 15h6"/></svg>'
const commentsIcon = '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>'

// Role filtering for nav
const currentUserRole = ref('author')
const adminPendingCount = ref(0)
const pendingCommentCount = ref(0)
const draftCount = ref(0)

function loadCurrentUserRole() {
  try {
    const u = authStore.user
    if (u) {
      currentUserRole.value = u.role || 'author'
      return
    }
    const stored = localStorage.getItem('marksharex_user') || sessionStorage.getItem('marksharex_user')
    if (stored) {
      const user = JSON.parse(stored)
      currentUserRole.value = user.role || 'author'
    }
  } catch {}
}
loadCurrentUserRole()

async function loadAdminPendingCount() {
  // Only admin/sub_admin see application badge
  const role = currentUserRole.value
  if (role !== 'admin' && role !== 'sub_admin') {
    adminPendingCount.value = 0
    return
  }
  try {
    const { data: resp } = await getPendingCount()
    adminPendingCount.value = resp.data
  } catch { /* ignore */ }
}

async function loadCommentPendingCount() {
  try {
    const role = currentUserRole.value
    const params = (role !== 'admin' && role !== 'sub_admin') ? { scope: 'mine' } : undefined
    const { data: resp } = await getCommentPendingCount(params)
    pendingCommentCount.value = resp.data
  } catch { /* ignore */ }
}

async function loadDraftCount() {
  try {
    const { data: resp } = await fetchAdminPosts({ page: 1, page_size: 1, status: 'draft' })
    draftCount.value = resp.pagination.total
  } catch { /* ignore */ }
}

// Avatar badge: sum of all pending items + draft count
const avatarBadgeCount = computed(() => {
  return adminPendingCount.value + pendingCommentCount.value + draftCount.value
})

const allNavItems = [
  { label: '仪表盘', key: 'dashboard', to: '/admin/dashboard', icon: dashboardIcon, roles: ['admin', 'sub_admin', 'author'] },
  { label: '知识库', key: 'posts', to: '/admin/posts', icon: postsIcon, roles: ['admin', 'sub_admin', 'author'] },
  { label: '资源库', key: 'files', to: '/admin/files', icon: filesIcon, roles: ['admin', 'sub_admin', 'author'] },
  { label: '批导', key: 'import', to: '/admin/import', icon: importIcon, roles: ['admin', 'sub_admin', 'author'] },
  { label: '用户', key: 'users', to: '/admin/users', icon: usersIcon, roles: ['admin', 'sub_admin'] },
  { label: '设置', key: 'settings', to: '/admin/settings', icon: settingsIcon, roles: ['admin'] },
  { label: '留言板', key: 'guestbook', to: '/admin/guestbook', icon: guestbookIcon, roles: ['admin', 'sub_admin'] },
  { label: '资讯', key: 'news', to: '/admin/news', icon: newsIcon, roles: ['admin', 'sub_admin'] },
  { label: 'AI', key: 'ai', to: '/admin/ai', icon: aiIcon, roles: ['admin', 'sub_admin'] },
]

const navItems = computed(() =>
  allNavItems
    .filter(item => item.roles.includes(currentUserRole.value))
    .map(item => {
      let badge = ''
      // Suppress badge when already on that page
      if (item.key === 'users' && adminPendingCount.value > 0 && !isActive('users')) {
        badge = String(adminPendingCount.value)
      } else if (item.key === 'dashboard' && pendingCommentCount.value > 0 && !isActive('dashboard')) {
        badge = String(pendingCommentCount.value)
      }
      return { ...item, badge }
    })
)

// SVG icons

onMounted(() => {
  initDarkMode()
  loadAdminPendingCount()
  loadCommentPendingCount()
  loadDraftCount()
})

if (!settingsStore.loaded) {
  settingsStore.fetchSettings()
}
</script>

<style scoped>
.admin-layout {
  min-height: 100vh;
  background: var(--input-bg);
  color: var(--input-color);
}

/* ===== Admin Nav Links ===== */
.admin-nav-links {
  display: flex;
  align-items: center;
  gap: 4px;
}
.admin-nav-link {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 16px;
  border-radius: 8px;
  color: var(--admin-nav-color);
  text-decoration: none;
  font-size: 15px;
  font-weight: 500;
  transition: all 0.15s;
}
.admin-nav-link:hover {
  color: var(--admin-nav-hover-color);
  background: rgba(255, 255, 255, 0.04);
}
.admin-nav-link.active {
  color: var(--admin-nav-active-color);
  background: var(--admin-nav-active-bg);
}
.admin-nav-link.active .nav-icon {
  opacity: 1;
  color: #4f46e5;
}
.nav-icon {
  display: flex;
  align-items: center;
  opacity: var(--admin-nav-icon-opacity);
}
.nav-badge {
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 10px;
  background: #ef4444;
  color: #fff;
  margin-left: 2px;
}

/* ===== Dropdown Buttons (in NavBar slot) ===== */
.dd-btn {
  display: flex;
  align-items: center;
  width: 100%;
  padding: 10px 16px;
  border: none;
  background: transparent;
  color: #d1d5db;
  font-size: 14px;
  cursor: pointer;
  transition: background 0.1s;
  text-align: left;
}
.dd-btn svg {
  margin-right: 8px;
  flex-shrink: 0;
  opacity: 0.55;
}
.dd-btn:hover {
  background: rgba(255, 255, 255, 0.05);
}
.dd-disabled {
  opacity: 0.5;
  cursor: not-allowed;
  pointer-events: none;
}
.dd-danger {
  color: #f87171;
}
.dd-danger:hover {
  background: rgba(248, 113, 113, 0.1);
}

/* ===== 主内容区 ===== */
.admin-main {
  max-width: 1280px;
  margin: 0 auto;
  padding: 28px 24px 60px;
}

/* ===== 弹窗 ===== */
.modal-overlay {
  position: fixed; inset: 0; background: rgba(0,0,0,0.6);
  display: flex; align-items: center; justify-content: center; z-index: 300;
}
.modal-box {
  background: var(--modal-bg); border: 1px solid rgba(255,255,255,0.08);
  border-radius: 14px; padding: 28px; max-width: 440px; width: 90%;
}
.modal-box h3 { margin: 0 0 20px; font-size: 17px; color: #f0f0f0; }
.modal-loading { display: flex; justify-content: center; padding: 40px 0; }

.modal-form { display: flex; flex-direction: column; gap: 16px; }
.form-row { display: flex; flex-direction: column; gap: 6px; }
.form-row label { font-size: 13px; color: #9ca3af; font-weight: 500; }
.form-input, .form-textarea {
  padding: 9px 14px; border-radius: 8px;
  border: 1px solid rgba(255,255,255,0.08); background: var(--input-bg);
  color: var(--input-color); font-size: 14px; outline: none;
  transition: border-color 0.15s;
}
.form-input:focus, .form-textarea:focus { border-color: rgba(79,70,229,0.4); }
.form-input:disabled { opacity: 0.5; cursor: not-allowed; }
.form-textarea { resize: vertical; min-height: 70px; font-family: inherit; }
.form-input::placeholder, .form-textarea::placeholder { color: #4b5563; }

.form-error { background: rgba(239,68,68,0.08); color: #f87171; padding: 10px 14px; border-radius: 8px; font-size: 13px; }
.form-ok { background: rgba(16,185,129,0.08); color: #34d399; padding: 10px 14px; border-radius: 8px; font-size: 13px; }

.modal-actions { display: flex; justify-content: flex-end; gap: 10px; padding-top: 4px; }
.btn-secondary {
  padding: 9px 18px; border-radius: 8px; border: 1px solid rgba(255,255,255,0.08);
  background: transparent; color: #9ca3af; font-size: 13px; cursor: pointer;
}
.btn-secondary:hover { color: var(--input-color); border-color: rgba(255,255,255,0.15); }
.btn-primary {
  padding: 9px 20px; border-radius: 8px; border: none;
  background: #4f46e5; color: #fff; font-size: 13px; font-weight: 500; cursor: pointer;
}
.btn-primary:hover { background: #4338ca; }
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }

.spinner {
  width: 28px; height: 28px; border: 3px solid rgba(79,70,229,0.15);
  border-top-color: #4f46e5; border-radius: 50%; animation: spin 0.6s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }
</style>
