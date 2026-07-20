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
.pagination-controls {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}
.page-arrow,
.page-number {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  border: 1px solid var(--card-border-color, var(--color-border));
  border-radius: 6px;
  background: var(--card-bg, var(--color-bg-card));
  color: var(--input-color, var(--color-text));
  cursor: pointer;
  transition: color 0.15s, border-color 0.15s, background-color 0.15s;
}
.page-arrow {
  font-size: 24px;
  line-height: 1;
}
.page-number {
  font-size: 12px;
}
.page-number.active {
  color: #fff;
  border-color: var(--color-primary);
  background: var(--color-primary);
  cursor: default;
}
.page-arrow:hover:not(:disabled),
.page-number:hover:not(.active) {
  color: var(--color-primary);
  border-color: var(--color-primary);
}
.page-arrow:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}
.page-ellipsis {
  display: inline-flex;
  width: 20px;
  height: 28px;
  align-items: center;
  justify-content: center;
  color: var(--color-text-muted);
  font-size: 12px;
}
.page-size-select {
  height: 28px;
  margin-left: 2px;
  padding: 0 24px 0 8px;
  border: 1px solid var(--card-border-color, var(--color-border));
  border-radius: 6px;
  background: var(--card-bg, var(--color-bg-card));
  color: var(--input-color, var(--color-text));
  font-size: 12px;
  cursor: pointer;
  outline: none;
}
.page-size-select:focus {
  border-color: var(--color-primary);
}

@media (max-width: 640px) {
  .pagination-wrap {
    justify-content: flex-end;
    gap: 4px;
  }
  .page-arrow,
  .page-number {
    width: 27px;
    height: 27px;
  }
  .page-ellipsis {
    width: 14px;
  }
  .page-size-select {
    height: 27px;
    padding-left: 6px;
  }
}
</style>
