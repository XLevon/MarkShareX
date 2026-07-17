<template>
  <div v-cloak>
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
        <AiChatWidget v-if="showAiChat" :mode="chatMode" />
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { darkTheme, zhCN, dateZhCN } from 'naive-ui'
import { useDarkMode } from '@/composables/useDarkMode'
import AiChatWidget from '@/components/shared/AiChatWidget.vue'
import { fetchDefaultAgent } from '@/api/ai'
import { useDocumentTitle } from '@/composables/useDocumentTitle'
import { staticRoutePageTitle } from '@/utils/documentTitle'

const route = useRoute()
const router = useRouter()
const { isDark, initDarkMode } = useDarkMode()
const staticPageTitle = computed(() => staticRoutePageTitle(route.name?.toString()))
useDocumentTitle(staticPageTitle)

const naiveTheme = computed(() => isDark.value ? darkTheme : null)

const chatMode = computed(() => route.path.startsWith('/admin') ? 'admin' as const : 'front' as const)

// 仅在存在默认 Agent 配置时才显示 AI 对话框
const showAiChat = ref(false)

async function refreshDefaultAgent() {
  try {
    const { data: agentResp } = await fetchDefaultAgent()
    showAiChat.value = agentResp.data.has_default
  } catch { /* 接口不可用时默认不显示 */ }
}

// ── 首次加载时后台检查系统是否已初始化，未初始化则跳转到 setup ──
import { fetchSetupStatus } from '@/api/setup'

onMounted(async () => {
  initDarkMode()

  // 检查是否有默认 Agent 配置
  await refreshDefaultAgent()

  // 监听管理后台的默认 Agent 切换事件，即时刷新
  window.addEventListener('marksharex:default-agent-changed', refreshDefaultAgent)

  // 首页加载后异步检查，若系统未初始化则跳转管理员初始化页面
  try {
    const { data: resp } = await fetchSetupStatus()
    if (!resp.data.initialized) {
      router.replace({ name: 'admin-setup' })
    }
  } catch {
    // API 异常不阻塞，放行
  }
})
</script>
