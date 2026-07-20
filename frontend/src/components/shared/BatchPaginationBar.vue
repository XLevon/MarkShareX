<template>
  <div class="pagination-wrap">
    <div class="batch-actions-inline" v-if="checkedCount > 0">
      <n-button size="small" type="success" @click="$emit('publish')">发布 {{ checkedCount }}</n-button>
      <n-button size="small" type="warning" @click="$emit('unpublish')">撤回 {{ checkedCount }}</n-button>
      <n-button v-if="isAdmin" size="small" type="error" @click="$emit('delete')">删除 {{ checkedCount }}</n-button>
    </div>
    <div class="pagination-controls">
      <button class="page-arrow" :disabled="page === 1" @click="$emit('goToPage', page - 1)">‹</button>
      <template v-for="(p, index) in visiblePages" :key="`${p}-${index}`">
        <span v-if="p === -1" class="page-ellipsis">…</span>
        <button v-else class="page-number" :class="{ active: p === page }" @click="$emit('goToPage', p)">{{ p }}</button>
      </template>
      <button class="page-arrow" :disabled="page >= totalPages" @click="$emit('goToPage', page + 1)">›</button>
      <select class="page-size-select" :value="pageSize" @change="$emit('pageSizeChange', Number(($event.target as HTMLSelectElement).value))">
        <option v-for="size in pageSizeOptions" :key="size" :value="size">{{ size }} 条/页</option>
      </select>
    </div>
  </div>
</template>

<script setup lang="ts">
import { NButton } from 'naive-ui'

defineProps<{
  checkedCount: number
  isAdmin: boolean
  page: number
  totalPages: number
  pageSize: number
  visiblePages: number[]
  pageSizeOptions: number[]
}>()

defineEmits<{
  publish: []
  unpublish: []
  delete: []
  goToPage: [page: number]
  pageSizeChange: [size: number]
}>()
</script>

<style scoped>
.pagination-wrap {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 6px;
  max-width: 100%;
}
.batch-actions-inline {
  display: flex;
  gap: 6px;
}
</style>
