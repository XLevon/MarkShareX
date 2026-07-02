<template>
  <!-- Global SVG filter for brush-stroke badges (defined once to avoid FOUC) -->
  <svg style="position:absolute;width:0;height:0;pointer-events:none" aria-hidden="true">
    <filter id="brush-edge" x="-30%" y="-30%" width="160%" height="160%">
      <feTurbulence type="fractalNoise" baseFrequency="0.08" numOctaves="3" result="n"/>
      <feDisplacementMap in="SourceGraphic" in2="n" scale="1.5" xChannelSelector="R" yChannelSelector="G"/>
    </filter>
  </svg>
  <n-config-provider :theme="naiveTheme" :locale="zhCN" :date-locale="dateZhCN">
    <n-message-provider>
      <n-dialog-provider>
        <router-view />
        <AiChatWidget :mode="chatMode" />
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { darkTheme, zhCN, dateZhCN } from 'naive-ui'
import { useDarkMode } from '@/composables/useDarkMode'
import AiChatWidget from '@/components/shared/AiChatWidget.vue'

const route = useRoute()
const router = useRouter()
const { isDark, initDarkMode } = useDarkMode()

const naiveTheme = computed(() => isDark.value ? darkTheme : null)

const chatMode = computed(() => route.path.startsWith('/admin') ? 'admin' as const : 'front' as const)

// ── 首次加载时后台检查系统是否已初始化，未初始化则跳转到 setup ──
import { fetchSetupStatus } from '@/api/setup'

onMounted(async () => {
  initDarkMode()

  // 仅首次访问时检查（localStorage 缓存避免重复）
  if (!localStorage.getItem('marksharex_initialized')) {
    try {
      const { data: resp } = await fetchSetupStatus()
      if (resp.data.initialized) {
        localStorage.setItem('marksharex_initialized', '1')
      } else {
        router.replace({ name: 'admin-setup' })
      }
    } catch {
      // API 异常，假定已初始化
    }
  }
})
</script>
