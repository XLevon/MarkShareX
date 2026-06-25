<template>
  <div
    class="rounded-xl border p-5 transition-theme"
    :style="{
      backgroundColor: 'var(--color-bg-card)',
      borderColor: 'var(--color-border)',
      boxShadow: 'var(--shadow-card)',
    }"
  >
    <slot v-if="$slots.title" name="title" />
    <h3 v-else-if="title" class="text-sm font-bold uppercase tracking-wider mb-4" :style="{ color: 'var(--color-text-muted)' }">
      {{ title }}
    </h3>
    <slot />
  </div>
</template>

<script setup lang="ts">
defineProps<{ title?: string }>()
</script>
