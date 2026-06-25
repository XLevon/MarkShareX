import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import './styles/main.css'

// 深色模式：默认启用 .dark class，确保深色背景上文字清晰可见
// 同时监听系统 prefers-color-scheme 变化，自动同步
function applyDarkMode() {
  const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
  // 始终启用深色模式（用户系统为浅色时也启用，因为页面的默认设计即为深色主题）
  document.documentElement.classList.add('dark')
}
applyDarkMode()

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.mount('#app')
