import { defineConfig, loadEnv } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'
import AutoImport from 'unplugin-auto-import/vite'
import Components from 'unplugin-vue-components/vite'
import { NaiveUiResolver } from 'unplugin-vue-components/resolvers'
import { resolve } from 'path'
import { readFileSync } from 'fs'

export default defineConfig(({ mode }) => {
  const projectRoot = resolve(__dirname, '..')

  // 三级配置优先级：Shell 环境变量 > .env 文件 > config.toml
  const env = loadEnv(mode, projectRoot, 'MARKSHAREX_')

  // 从 config.toml 读取 [server] host 和 port（最低优先级）
  let tomlHost = 'localhost'
  let tomlPort = '5023'
  try {
    const toml = readFileSync(resolve(projectRoot, 'config.toml'), 'utf-8')
    const hm = toml.match(/\[server\][\s\S]*?^host\s*=\s*"([^"]+)"/m)
    const pm = toml.match(/\[server\][\s\S]*?^port\s*=\s*(\d+)/m)
    if (hm) tomlHost = hm[1]
    if (pm) tomlPort = pm[1]
  } catch {}

  // 0.0.0.0 是绑定地址，proxy 应连接 localhost
  const host = (process.env.MARKSHAREX_SERVER_HOST || env.MARKSHAREX_SERVER_HOST || tomlHost)
    .replace('0.0.0.0', 'localhost')
  const port = process.env.MARKSHAREX_SERVER_PORT || env.MARKSHAREX_SERVER_PORT || tomlPort
  const backendUrl = `http://${host}:${port}`

  return {
    plugins: [
      vue(),
      tailwindcss(),
      AutoImport({
        imports: ['vue', 'vue-router', 'pinia'],
        dts: 'src/auto-imports.d.ts',
      }),
      Components({
        resolvers: [NaiveUiResolver()],
        dts: 'src/components.d.ts',
      }),
    ],
    resolve: {
      alias: {
        '@': resolve(__dirname, 'src'),
      },
    },
    server: {
      port: 5173,
      proxy: {
        '/api': {
          target: backendUrl,
          changeOrigin: true,
        },
        '/uploads': {
          target: backendUrl,
          changeOrigin: true,
        },
        '/scalar': {
          target: backendUrl,
          changeOrigin: true,
        },
      },
    },
    build: {
      outDir: '../static/frontend',
      emptyOutDir: true,
    },
  }
})
