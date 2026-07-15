<template>
  <header
    class="z-[1180] border-b backdrop-blur-md"
    :class="{ sticky: sticky, 'top-0': sticky, relative: !sticky }"
    :style="headerStyle"
  >
    <div class="max-w-7xl mx-auto px-6">
      <div class="flex items-center justify-between h-16 gap-4">
        <!-- Left: Logo + Nav -->
        <div class="flex items-center gap-1 sm:gap-4 flex-shrink-0" :style="{ minWidth: isMobile ? 'auto' : '240px' }">
          <router-link
            to="/"
            class="flex items-center font-bold no-underline flex-shrink-0 gap-1"
            style="font-size: 22px; letter-spacing: -0.5px"
          >
            <img v-if="logoImage" :src="logoImage" class="h-6 w-auto object-contain" alt="Logo" />
            <span class="flex"><span v-for="(part, i) in logoParts" :key="i" :style="{ color: part.color }">{{ part.text }}</span></span>
          </router-link>
          <slot name="nav-left" />
        </div>

        <!-- Center -->
        <slot name="nav-center" />

        <!-- Right: DarkMode + User -->
        <div class="flex items-center gap-1 sm:gap-2 flex-shrink-0">
          <!-- Dark Mode Toggle -->
          <button
            @click="$emit('toggle-dark')"
            class="w-10 h-10 flex items-center justify-center rounded-[10px] transition-all duration-150 border-0 cursor-pointer"
            :style="{ color: isDark ? '#9ca3af' : '#4b5563', background: 'transparent' }"
            :title="isDark ? '切换到浅色模式' : '切换到深色模式'"
          >
            <!-- Sun icon -->
            <svg v-if="isDark" width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/><line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/></svg>
            <!-- Moon icon -->
            <svg v-else width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg>
          </button>

          <!-- User Area -->
          <template v-if="isLoggedIn">
            <div class="relative">
              <button
                @click="showUserMenu = !showUserMenu"
                class="flex items-center gap-2 py-0 px-2.5 rounded-[10px] text-sm transition-all duration-150 border-0 cursor-pointer hover:bg-white/5"
                style="color: #9ca3af; background: transparent"
              >
                <span class="w-8 h-8 rounded-full flex items-center justify-center text-white flex-shrink-0 relative" style="background: #4f46e5; font-size: 14px; font-weight: 600">
                  {{ userInitial }}
                  <span v-if="badgeCount > 0" class="avatar-badge">{{ badgeCount > 99 ? '99+' : badgeCount }}</span>
                </span>
                <span style="font-size: 14px" :style="{ color: isDark ? '#d1d5db' : '#1f2937' }">{{ displayUser }}</span>
                <svg class="transition-transform duration-150" :class="{ 'rotate-180': showUserMenu }" width="16" height="16" viewBox="0 0 16 16"><path d="M4 6l4 4 4-4" stroke="currentColor" stroke-width="1.5" fill="none"/></svg>
              </button>
              <div v-if="showUserMenu" class="absolute right-0 top-full mt-1.5 rounded-lg overflow-hidden z-[1400]" :style="dropdownStyle" @click="showUserMenu = false">
                <slot name="dropdown-items" />
              </div>
            </div>
          </template>
          <template v-else>
            <slot name="user-else" />
          </template>
        </div>
      </div>
    </div>

  </header>

  <!-- Backdrop to close dropdown (outside header for proper click handling) -->
  <div v-if="showUserMenu" class="fixed inset-0 z-40" @click="showUserMenu = false"></div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useTitleParts } from '@/composables/useTitleParts'

const props = withDefaults(defineProps<{
  logoText: string
  logoImage: string
  isDark: boolean
  isLoggedIn: boolean
  displayUser: string
  userInitial: string
  headerStyle: Record<string, string>
  badgeCount?: number
  sticky?: boolean
}>(), {
  sticky: true,
})

const isMobile = ref(window.innerWidth < 640)
if (typeof window !== 'undefined') {
  window.addEventListener('resize', () => {
    isMobile.value = window.innerWidth < 640
  })
}

const logoParts = useTitleParts(
  () => props.logoText,
  () => props.isDark
)

defineEmits<{
  'toggle-dark': []
}>()

const showUserMenu = ref(false)

const dropdownStyle = computed(() => ({
  background: props.isDark ? '#1e1e2a' : '#ffffff',
  border: props.isDark ? '1px solid rgba(255,255,255,0.08)' : '1px solid #e5e7eb',
  boxShadow: props.isDark ? '0 12px 40px rgba(0,0,0,0.4)' : '0 4px 24px rgba(0,0,0,0.08)',
  minWidth: '150px',
  zIndex: 1400,
}))
</script>

<style scoped>
.avatar-badge {
  position: absolute;
  top: -4px;
  right: -4px;
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  border-radius: 9px;
  background: #ef4444;
  color: #fff;
  font-size: 11px;
  font-weight: 700;
  line-height: 18px;
  text-align: center;
  border: 2px solid var(--admin-nav-color, #9ca3af);
}
</style>
