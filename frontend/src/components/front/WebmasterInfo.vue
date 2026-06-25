<template>
  <div class="webmaster-info" v-if="admin.bio">
    <!-- Avatar + Name Row -->
    <div class="flex items-center gap-3 mb-3">
      <div
        class="w-10 h-10 rounded-full overflow-hidden flex-shrink-0 border-2 flex items-center justify-center text-white font-bold text-sm"
        :style="{ borderColor: 'var(--color-primary)', backgroundColor: 'var(--color-primary)' }"
      >
        <img
          v-if="admin.avatar_url"
          :src="admin.avatar_url"
          class="w-full h-full object-cover"
          :alt="admin.display_name || '站长'"
        />
        <span v-else>{{ avatarFallback }}</span>
      </div>
      <div>
        <div class="text-base font-semibold" :style="{ color: 'var(--color-text)' }">
          {{ admin.display_name || '站长' }}
          <a
            v-if="admin.email"
            :href="'mailto:' + admin.email"
            class="text-sm font-normal ml-1 opacity-60 hover:opacity-100 transition-opacity"
            :style="{ color: 'var(--color-text-secondary)' }"
          >{{ admin.email }}</a>
        </div>
        <div class="text-sm font-bold mt-0.5" :style="{ color: 'var(--color-text-muted)' }">
          {{ admin.title || '站点管理员' }}
        </div>
      </div>
    </div>

    <!-- Bio -->
    <p class="text-xs leading-relaxed" :style="{ color: 'var(--color-text-secondary)' }">
      {{ admin.bio }}
    </p>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { SiteManagerInfo } from '@/api/index'

const props = defineProps<{
  admin: SiteManagerInfo
}>()

const avatarFallback = computed(() => {
  const name = props.admin.display_name || '站'
  return name.charAt(0)
})
</script>
