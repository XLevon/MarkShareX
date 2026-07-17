import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import './styles/main.css'

// dark class 由 index.html 的 inline <script> 在解析 HTML 前立即添加，消除 FOUC

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.mount('#app')
