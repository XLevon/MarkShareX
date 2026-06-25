<template>
  <div class="min-h-screen flex flex-col transition-theme" :style="{ backgroundColor: 'var(--color-bg)', color: 'var(--color-text)' }">
    <!-- Navigation Bar -->
    <NavBar
      :logo-text="settingsStore.settings.site_title || 'MarkShareX'"
      :logo-image="settingsStore.resolvedLogoUrl"
      :is-dark="isDark"
      :is-logged-in="isLoggedIn"
      :display-user="displayUser"
      :user-initial="userInitial"
      :header-style="navStyle"
      :badge-count="avatarBadge"
      @toggle-dark="toggleDarkMode"
    >
      <template #nav-left>
        <nav class="hidden sm:flex items-center gap-1">
          <router-link
            to="/"
            class="px-3 py-2 rounded-lg text-sm font-medium transition-colors no-underline"
            :class="isHomeActive ? '' : 'hover:bg-[var(--color-bg-hover)]'"
            :style="isHomeActive ? { color: 'var(--color-primary)', backgroundColor: 'var(--color-primary-bg)' } : { color: 'var(--color-text-secondary)' }"
          >首页</router-link>
          <router-link
            to="/knowledge-base"
            class="px-3 py-2 rounded-lg text-sm font-medium transition-colors no-underline"
            :class="isKBactive ? '' : 'hover:bg-[var(--color-bg-hover)]'"
            :style="isKBactive ? { color: 'var(--color-primary)', backgroundColor: 'var(--color-primary-bg)' } : { color: 'var(--color-text-secondary)' }"
          >知识库</router-link>
          <router-link
            to="/pinned"
            class="px-3 py-2 rounded-lg text-sm font-medium transition-colors no-underline"
            :class="isPinnedActive ? '' : 'hover:bg-[var(--color-bg-hover)]'"
            :style="isPinnedActive ? { color: 'var(--color-primary)', backgroundColor: 'var(--color-primary-bg)' } : { color: 'var(--color-text-secondary)' }"
          >推荐</router-link>
          <router-link
            to="/categories"
            class="px-3 py-2 rounded-lg text-sm font-medium transition-colors no-underline"
            :class="isCategoryActive ? '' : 'hover:bg-[var(--color-bg-hover)]'"
            :style="isCategoryActive ? { color: 'var(--color-primary)', backgroundColor: 'var(--color-primary-bg)' } : { color: 'var(--color-text-secondary)' }"
          >分类</router-link>
          <router-link
            to="/tags"
            class="px-3 py-2 rounded-lg text-sm font-medium transition-colors no-underline"
            :class="isTagActive ? '' : 'hover:bg-[var(--color-bg-hover)]'"
            :style="isTagActive ? { color: 'var(--color-primary)', backgroundColor: 'var(--color-primary-bg)' } : { color: 'var(--color-text-secondary)' }"
          >标签</router-link>
          <router-link
            to="/types"
            class="px-3 py-2 rounded-lg text-sm font-medium transition-colors no-underline"
            :class="isTypeActive ? '' : 'hover:bg-[var(--color-bg-hover)]'"
            :style="isTypeActive ? { color: 'var(--color-primary)', backgroundColor: 'var(--color-primary-bg)' } : { color: 'var(--color-text-secondary)' }"
          >类型</router-link>
          <router-link
            to="/statuses"
            class="px-3 py-2 rounded-lg text-sm font-medium transition-colors no-underline"
            :class="isStatusActive ? '' : 'hover:bg-[var(--color-bg-hover)]'"
            :style="isStatusActive ? { color: 'var(--color-primary)', backgroundColor: 'var(--color-primary-bg)' } : { color: 'var(--color-text-secondary)' }"
          >状态</router-link>
          <router-link
            to="/authors"
            class="px-3 py-2 rounded-lg text-sm font-medium transition-colors no-underline"
            :class="isAuthorActive ? '' : 'hover:bg-[var(--color-bg-hover)]'"
            :style="isAuthorActive ? { color: 'var(--color-primary)', backgroundColor: 'var(--color-primary-bg)' } : { color: 'var(--color-text-secondary)' }"
          >作者</router-link>
          <router-link
            v-if="guestbookEnabled"
            to="/guestbook"
            class="px-3 py-2 rounded-lg text-sm font-medium transition-colors no-underline"
            :class="isGuestbookActive ? '' : 'hover:bg-[var(--color-bg-hover)]'"
            :style="isGuestbookActive ? { color: 'var(--color-primary)', backgroundColor: 'var(--color-primary-bg)' } : { color: 'var(--color-text-secondary)' }"
          >留言板</router-link>
        </nav>
      </template>
      <template #nav-center>
        <div v-if="navSearchVisible" class="hidden md:flex items-center relative flex-1 min-w-0">
          <input v-model="searchQuery" type="text" placeholder="搜索文章、标签或作者..." class="w-full px-3 py-2 pl-9 text-sm rounded-lg border outline-none transition-colors"
            :style="{ backgroundColor: 'var(--color-bg-secondary)', borderColor: 'var(--color-border)', color: 'var(--color-text)' }"
            @keyup.enter="doSearch"
            @focus="$event.target.style.borderColor = 'var(--color-primary)'"
            @blur="$event.target.style.borderColor = 'var(--color-border)'"
          />
          <svg class="absolute left-2.5 top-1/2 -translate-y-1/2 w-4 h-4" :style="{ color: 'var(--color-text-muted)' }" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
          </svg>
        </div>
      </template>
      <template #dropdown-items>
        <button class="flex items-center w-full text-left px-4 py-2.5 text-sm border-0 cursor-pointer transition-colors" :style="{ color: isDark ? '#d1d5db' : '#1f2937', background: 'transparent' }" @click="showProfile = true">
          <svg class="mr-2 opacity-50 flex-shrink-0" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
          个人信息
        </button>
        <button class="flex items-center w-full text-left px-4 py-2.5 text-sm border-0 cursor-pointer transition-colors" :style="{ color: isDark ? '#d1d5db' : '#1f2937', background: 'transparent' }" @click="openPassword()">
          <svg class="mr-2 opacity-50 flex-shrink-0" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
          修改密码
        </button>
        <button v-if="canAccessAdmin" class="flex items-center w-full text-left px-4 py-2.5 text-sm border-0 cursor-pointer transition-colors" :style="{ color: isDark ? '#d1d5db' : '#1f2937', background: 'transparent' }" @click="goAdmin()">
          <svg class="mr-2 opacity-50 flex-shrink-0" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/></svg>
          管理后台
        </button>
        <button v-if="!canAccessAdmin && !hasApplied" class="flex items-center w-full text-left px-4 py-2.5 text-sm border-0 cursor-pointer transition-colors" :style="{ color: isDark ? '#d1d5db' : '#1f2937', background: 'transparent' }" @click="router.replace(`/apply?redirect=${encodeURIComponent(route.fullPath)}`)">
          <svg class="mr-2 opacity-50 flex-shrink-0" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="12" y1="18" x2="12" y2="12"/><line x1="9" y1="15" x2="15" y2="15"/></svg>
          申请分享
        </button>
        <button v-if="!canAccessAdmin && hasApplied" class="flex items-center w-full text-left px-4 py-2.5 text-sm border-0 transition-colors opacity-40" style="color: #9ca3af; background: transparent; cursor: not-allowed" disabled>
          <svg class="mr-2 opacity-50 flex-shrink-0" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="12" y1="18" x2="12" y2="12"/><line x1="9" y1="15" x2="15" y2="15"/></svg>
          已申请，等待审核
        </button>
        <button class="flex items-center w-full text-left px-4 py-2.5 text-sm border-0 cursor-pointer transition-colors" :style="{ color: isDark ? '#f87171' : '#dc2626', background: 'transparent' }" @click="handleLogout()">
          <svg class="mr-2 opacity-70 flex-shrink-0" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" y1="12" x2="9" y2="12"/></svg>
          退出登录
        </button>
      </template>
      <template #user-else>
        <a href="#" @click.prevent="router.replace(`/login?redirect=${encodeURIComponent(route.fullPath)}`)" class="inline-flex items-center px-3 py-2 text-sm font-medium rounded-lg transition-colors no-underline" :style="{ color: 'var(--color-text)', border: '1px solid var(--color-border)' }">登录</a>
        <a href="#" @click.prevent="router.replace(`/register?redirect=${encodeURIComponent(route.fullPath)}`)" class="inline-flex items-center px-4 py-2 text-sm font-medium rounded-lg transition-colors no-underline ml-2" :style="{ backgroundColor: '#4f46e5', color: '#ffffff' }">注册</a>
      </template>
    </NavBar>

    <!-- Main Content -->
    <main class="flex-1">
      <RouterView />
    </main>

    <!-- Footer -->
    <footer
      class="border-t py-8 mt-16 transition-theme"
      :style="{ borderColor: 'var(--color-border)' }"
    >
      <div class="max-w-7xl mx-auto px-4 text-center text-sm" :style="{ color: 'var(--color-text-muted)' }">
        <p>{{ settingsStore.settings.site_subtitle || '' }}</p>
        <p class="mt-2">&copy; {{ new Date().getFullYear() }} <a href="https://www.xlevon.cn" target="_blank" rel="noopener noreferrer" class="hover:underline" :style="{ color: 'var(--color-primary)' }">MarkShareX</a> ・ Created by <a href="mailto:408251965@qq.com" class="hover:underline" :style="{ color: 'var(--color-primary)' }">XLevon</a> ・ Built with Vibe<span v-if="appVersion"> ・ <router-link to="/changelog" class="hover:underline" :style="{ color: 'var(--color-primary)' }">v{{ appVersion }}</router-link></span></p>
      </div>
    </footer>

    <!-- ======== 个人信息弹窗 ======== -->
    <div v-if="showProfile" class="fixed inset-0 z-50 flex items-center justify-center" :style="{ background: modalTheme.overlayBg }">
      <div class="rounded-2xl p-7 w-full" :style="{ maxWidth: '440px', background: modalTheme.boxBg, border: modalTheme.boxBorder }">
        <h3 class="m-0 mb-5 text-lg" :style="{ color: modalTheme.titleColor }">个人信息</h3>
        <ProfileView mode="modal" @close="showProfile = false" @saved="showProfile = false" />
      </div>
    </div>

    <!-- ======== 修改密码弹窗 ======== -->
    <div v-if="showPassword" class="fixed inset-0 z-50 flex items-center justify-center" :style="{ background: modalTheme.overlayBg }">
      <div class="rounded-2xl p-7 w-full" :style="{ maxWidth: '420px', background: modalTheme.boxBg, border: modalTheme.boxBorder }">
        <h3 class="m-0 mb-5 text-lg" :style="{ color: modalTheme.titleColor }">修改密码</h3>
        <form @submit.prevent="handlePasswordSave" class="flex flex-col gap-4">
          <div class="flex flex-col gap-1.5">
            <label class="text-xs font-medium" :style="{ color: modalTheme.labelColor }">旧密码</label>
            <input v-model="pwForm.old_password" type="password" required class="px-3.5 py-2.5 rounded-lg text-sm outline-none transition-colors" placeholder="输入当前密码" :style="{ border: modalTheme.inputBorder, background: modalTheme.inputBg, color: modalTheme.inputColor }" />
          </div>
          <div class="flex flex-col gap-1.5">
            <label class="text-xs font-medium" :style="{ color: modalTheme.labelColor }">新密码</label>
            <input v-model="pwForm.new_password" type="password" required minlength="8" class="px-3.5 py-2.5 rounded-lg text-sm outline-none transition-colors" placeholder="至少 8 位" :style="{ border: modalTheme.inputBorder, background: modalTheme.inputBg, color: modalTheme.inputColor }" />
          </div>
          <div class="flex flex-col gap-1.5">
            <label class="text-xs font-medium" :style="{ color: modalTheme.labelColor }">确认新密码</label>
            <input v-model="pwForm.confirm_password" type="password" required minlength="8" class="px-3.5 py-2.5 rounded-lg text-sm outline-none transition-colors" placeholder="再次输入新密码" :style="{ border: modalTheme.inputBorder, background: modalTheme.inputBg, color: modalTheme.inputColor }" />
          </div>
          <div v-if="pwError" class="px-3.5 py-2.5 rounded-lg text-xs" :style="{ background: modalTheme.errorBg, color: modalTheme.errorColor }">{{ pwError }}</div>
          <div v-if="pwOk" class="px-3.5 py-2.5 rounded-lg text-xs" :style="{ background: modalTheme.okBg, color: modalTheme.okColor }">{{ pwOk }}</div>
          <div class="flex justify-end gap-2.5 pt-1">
            <button type="button" class="px-4 py-2 rounded-lg text-xs border cursor-pointer transition-colors" :style="{ borderColor: modalTheme.btnSecBorder, color: modalTheme.btnSecColor, background: 'transparent' }" @click="showPassword = false">取消</button>
            <button type="submit" :disabled="pwSaving" class="px-5 py-2 rounded-lg text-xs font-medium border-0 cursor-pointer transition-colors text-white" style="background: #4f46e5" :style="pwSaving ? {opacity: 0.5, cursor: 'not-allowed'} : {}">{{ pwSaving ? '保存中...' : '修改密码' }}</button>
          </div>
        </form>
      </div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useSettingsStore } from '@/stores/settings'
import { useDarkMode } from '@/composables/useDarkMode'
import { useAuthStore } from '@/stores/auth'
import { changePassword, getApplicationStatus, getPendingCount, getCommentPendingCount } from '@/api/admin'
import { fetchAdminPosts } from '@/api/posts'
import NavBar from '@/components/shared/NavBar.vue'
import ProfileView from '@/views/admin/Profile.vue'
import { navSearchVisible } from '@/composables/useSearchVisibility'
import { fetchLatestVersion } from '@/api/changelog'

const route = useRoute()
const router = useRouter()
const settingsStore = useSettingsStore()
const authStore = useAuthStore()
const { isDark, toggleDarkMode } = useDarkMode()

const modalTheme = computed(() => ({
  overlayBg: 'rgba(0,0,0,0.6)',
  boxBg: isDark.value ? '#1e1e2a' : '#ffffff',
  boxBorder: isDark.value ? '1px solid rgba(255,255,255,0.08)' : '1px solid #e5e7eb',
  titleColor: isDark.value ? '#f0f0f0' : '#1a1a2e',
  labelColor: isDark.value ? '#9ca3af' : '#6b7280',
  inputBg: isDark.value ? '#0f0f13' : '#ffffff',
  inputColor: isDark.value ? '#e0e0e0' : '#1a1a2e',
  inputBorder: isDark.value ? '1px solid rgba(255,255,255,0.08)' : '1px solid #d1d5db',
  errorBg: isDark.value ? 'rgba(239,68,68,0.08)' : 'rgba(239,68,68,0.06)',
  errorColor: isDark.value ? '#f87171' : '#dc2626',
  okBg: isDark.value ? 'rgba(16,185,129,0.08)' : 'rgba(16,185,129,0.06)',
  okColor: isDark.value ? '#34d399' : '#059669',
  btnSecColor: isDark.value ? '#9ca3af' : '#6b7280',
  btnSecBorder: isDark.value ? '1px solid rgba(255,255,255,0.08)' : '1px solid #d1d5db',
  spinnerBorder: isDark.value ? '3px solid rgba(79,70,229,0.15)' : '3px solid rgba(79,70,229,0.12)',
}))

const appVersion = ref('...')
const changelogVersion = ref('')
const searchQuery = ref('')
const avatarBadge = ref(0)

async function loadAvatarBadge() {
  const u = authStore.user
  if (!u) return
  const role = u.role || ''
  let total = 0
  if (role === 'admin' || role === 'sub_admin') {
    try {
      const [appResp, commentResp, draftResp] = await Promise.all([
        getPendingCount(),
        getCommentPendingCount(),
        fetchAdminPosts({ page: 1, page_size: 1, status: 'draft' }),
      ])
      total = (appResp.data.data || 0) + (commentResp.data.data || 0) + (draftResp.data.pagination.total || 0)
    } catch { /* ignore */ }
  } else if (role === 'author') {
    try {
      const { data: resp } = await getCommentPendingCount({ scope: 'mine' })
      total = resp.data || 0
    } catch { /* ignore */ }
  }
  avatarBadge.value = total
}

onMounted(async () => {
  try {
    const res = await fetch('/api/v1/version')
    const data = await res.json()
    appVersion.value = data.version
  } catch { appVersion.value = '' }
  try {
    const { data } = await fetchLatestVersion()
    changelogVersion.value = data.data?.version || ''
  } catch { /* ignore */ }
  if (isLoggedIn.value) {
    loadAvatarBadge()
  }
})

const isLoggedIn = computed(() => authStore.isAuthenticated)
const isAdmin = computed(() => authStore.user?.role === 'admin')
const canAccessAdmin = computed(() => {
  const u = authStore.user
  if (!u) return false
  return ['admin', 'sub_admin', 'author'].includes(u.role)
})
const hasApplied = ref(false)

// Check application status on auth state change
watch(() => authStore.isAuthenticated, async (val) => {
  if (val && !canAccessAdmin.value) {
    try {
      const { data } = await getApplicationStatus()
      hasApplied.value = data.data?.status === 'pending'
    } catch { hasApplied.value = false }
  } else {
    hasApplied.value = false
  }
  // Reload avatar badge
  if (val) {
    loadAvatarBadge()
  } else {
    avatarBadge.value = 0
  }
}, { immediate: true })

// NavBar header style
const navStyle = computed(() => ({
  backgroundColor: isDark.value ? 'rgba(15, 23, 42, 0.85)' : 'rgba(255, 255, 255, 0.85)',
  borderColor: 'var(--color-border)',
  boxShadow: 'var(--shadow-nav)',
}))

// User display info — reactive from auth store
const displayUser = computed(() => {
  const u = authStore.user
  if (u) return u.display_name || u.username || '用户'
  return '用户'
})
const userInitial = computed(() => (displayUser.value || '用')[0])

const isHomeActive = computed(() => route.path === '/')
const isKBactive = computed(() => route.path === '/knowledge-base')
const isPinnedActive = computed(() => route.path === '/pinned')
const isCategoryActive = computed(() => route.path === '/categories' || route.path.startsWith('/category/'))
const isTagActive = computed(() => route.path === '/tags' || route.path.startsWith('/tag/'))
const isTypeActive = computed(() => route.path === '/types' || route.path.startsWith('/type/'))
const isStatusActive = computed(() => route.path === '/statuses' || route.path.startsWith('/status/'))
const isAuthorActive = computed(() => route.path === '/authors' || route.path.startsWith('/author/'))
const isGuestbookActive = computed(() => route.path === '/guestbook')
const guestbookEnabled = computed(() => settingsStore.settings.guestbook_enabled !== 'false')

function doSearch() {
  const q = searchQuery.value.trim()
  if (q) {
    router.push({ path: '/search', query: { q } })
  } else {
    router.push({ path: '/search', query: {} })
  }
}

function goAdmin() {
  router.push('/admin')
}

function handleLogout() {
  authStore.logout()
  // 前台下不跳转，停留在原地
}

// ── Profile modal ──
const showProfile = ref(false)

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

function handlePasswordSave() {
  pwError.value = ''
  pwOk.value = ''
  if (pwForm.new_password !== pwForm.confirm_password) {
    pwError.value = '两次输入的新密码不一致'
    return
  }
  pwSaving.value = true
  changePassword({
    old_password: pwForm.old_password,
    new_password: pwForm.new_password,
    confirm_password: pwForm.confirm_password,
  }).then(() => {
    pwOk.value = '密码修改成功，即将关闭...'
    setTimeout(() => { showPassword.value = false }, 2000)
  }).catch((e: any) => {
    pwError.value = e?.response?.data?.error || '修改失败'
  }).finally(() => {
    pwSaving.value = false
  })
}

// ── Admin redirect ──
if (!settingsStore.loaded) {
  settingsStore.fetchSettings()
}
</script>
