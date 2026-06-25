<template>
  <div class="min-h-[70vh] flex items-center justify-center px-4" :style="{ backgroundColor: 'var(--color-bg)' }">
    <div class="text-center max-w-md">
      <h1 class="text-8xl font-extrabold mb-4 tracking-tighter" :style="{ color: 'var(--color-primary)' }">404</h1>
      <h2 class="text-2xl font-bold mb-3" :style="{ color: 'var(--color-text)' }">页面未找到</h2>
      <p class="mb-8 leading-relaxed" :style="{ color: 'var(--color-text-secondary)' }">
        抱歉，您访问的页面不存在或已被移除。请检查链接是否正确，或返回首页继续浏览。
      </p>
      <div class="flex justify-center gap-3">
        <button
          @click="$router.back()"
          class="px-5 py-2.5 rounded-xl text-sm font-medium transition-colors border"
          :style="{
            color: 'var(--color-text)',
            borderColor: 'var(--color-border)',
            backgroundColor: 'var(--color-bg-card)',
          }"
        >
          ← 返回上一页
        </button>
        <router-link
          to="/"
          class="px-5 py-2.5 rounded-xl text-sm font-medium text-white transition-colors no-underline"
          :style="{ backgroundColor: 'var(--color-primary)' }"
        >
          返回首页
        </router-link>
      </div>
    </div>
  </div>
</template>
