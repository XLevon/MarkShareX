<template>
  <div class="network-resources-page">
    <!-- 搜索 -->
    <div class="search-bar">
      <input v-model="searchText" type="text" class="search-input" placeholder="搜索 URL 或标签..." @keydown.enter="search()" />
      <button class="btn-search" @click="search()">搜索</button>
      <button v-if="searchText" class="btn-clear" @click="clearSearch()">清除</button>
      <button class="btn-add" @click="openAdd()">+ 添加资源</button>
    </div>

    <!-- 列表 -->
    <div v-if="loading" class="loading">加载中...</div>
    <div v-else class="table-scroll" aria-label="网络资源列表，可左右滑动查看全部列">
      <table class="data-table">
        <thead>
          <tr>
            <th style="width:50px">ID</th>
            <th style="width:80px">缩略图</th>
            <th style="width:200px">URL</th>
            <th style="width:200px">标签</th>
            <th style="width:80px">类型</th>
            <th style="width:130px">创建时间</th>
            <th style="width:190px">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="item in items" :key="item.id">
            <td>{{ item.id }}</td>
            <td>
              <a v-if="item.source_type === 'image'" :href="item.url" target="_blank">
                <img :src="item.url" class="thumb-img" loading="lazy" referrerpolicy="no-referrer" @error="($event.target as HTMLImageElement).style.display='none'" />
              </a>
              <span v-else class="thumb-placeholder">—</span>
            </td>
            <td class="url-cell">{{ item.url }}</td>
            <td>{{ item.label || '-' }}</td>
            <td><span class="type-badge">{{ item.source_type }}</span></td>
            <td>{{ item.created_at }}</td>
            <td class="actions">
              <button class="btn-action" @click="openRefs(item)">引用</button>
              <button class="btn-action" @click="openEdit(item)">编辑</button>
              <button v-if="!item.referenced" class="btn-action btn-danger" @click="confirmDelete(item)">删除</button>
            </td>
          </tr>
          <tr v-if="items.length === 0">
            <td colspan="7" class="empty-row">暂无网络资源</td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 分页 -->
    <div v-if="pagination && pagination.pages > 1" class="pagination">
      <button :disabled="page <= 1" @click="goPage(page - 1)">上一页</button>
      <span>第 {{ page }} / {{ pagination.pages }} 页</span>
      <button :disabled="page >= pagination.pages" @click="goPage(page + 1)">下一页</button>
    </div>

    <!-- 添加/编辑弹窗 -->
    <div v-if="showModal" class="modal-overlay">
      <div class="modal-box">
        <h3>{{ editing ? '编辑资源' : '添加网络资源' }}</h3>
        <form @submit.prevent="save()">
          <div class="form-row">
            <label>URL</label>
            <input v-model="form.url" type="url" class="form-input" placeholder="https://example.com/image.jpg" required />
          </div>
          <div class="form-row">
            <label>标签（可选）</label>
            <input v-model="form.label" type="text" class="form-input" placeholder="描述性标签" />
          </div>
          <div class="form-row">
            <label>类型</label>
            <select v-model="form.source_type" class="form-input">
              <option value="image">图片</option>
              <option value="video">视频</option>
              <option value="other">其他</option>
            </select>
          </div>
          <div v-if="error" class="form-error">{{ error }}</div>
          <div class="modal-actions">
            <button type="button" class="btn-cancel" @click="showModal = false">取消</button>
            <button type="submit" class="btn-save" :disabled="saving">{{ saving ? '保存中...' : '保存' }}</button>
          </div>
        </form>
      </div>
    </div>

    <!-- 引用清单弹窗 -->
    <div v-if="showRefs" class="modal-overlay" @click.self="showRefs = false">
      <div class="refs-modal">
        <!-- 资源预览 -->
        <div class="refs-preview">
          <img v-if="refTarget?.source_type === 'image'" :src="refTarget!.url" class="refs-thumb" />
          <div class="refs-info">
            <div class="refs-label">{{ refTarget?.label || '未命名资源' }}</div>
            <div class="refs-url">{{ refTarget?.url }}</div>
          </div>
        </div>

        <!-- Loading -->
        <div v-if="refsLoading" class="refs-loading">
          <div class="spinner-sm"></div>
          <span>查询引用中...</span>
        </div>

        <!-- 无引用 -->
        <div v-else-if="refs.length === 0" class="refs-empty">
          <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/></svg>
          <p>未被任何分类或文章引用</p>
          <span>该资源可安全删除</span>
        </div>

        <!-- 引用列表 -->
        <template v-else>
          <div class="refs-summary">
            <span class="refs-count-cat" v-if="catRefs.length">{{ catRefs.length }} 个分类</span>
            <span class="refs-dot" v-if="catRefs.length && postRefs.length">·</span>
            <span class="refs-count-post" v-if="postRefs.length">{{ postRefs.length }} 篇文章</span>
          </div>
          <div class="refs-list">
            <div v-for="ref in refs" :key="ref.target_type + ref.target_id" class="refs-item">
              <div class="refs-item-icon" :class="ref.target_type === 'category' ? 'icon-cat' : 'icon-post'">
                {{ ref.target_type === 'category' ? '📁' : '📄' }}
              </div>
              <div class="refs-item-body">
                <div class="refs-item-name">
                  {{ ref.target_name }}
                  <span class="refs-item-type">{{ ref.target_type === 'category' ? '分类' : '文章' }}</span>
                </div>
                <div class="refs-item-slug">/{{ ref.target_slug }}</div>
                <div v-if="ref.target_description" class="refs-item-desc">{{ ref.target_description }}</div>
              </div>
            </div>
          </div>
        </template>

        <div class="refs-footer">
          <button class="btn-cancel" @click="showRefs = false">关闭</button>
        </div>
      </div>
    </div>

    <!-- 删除确认 -->
    <div v-if="showDelete" class="modal-overlay" @click.self="showDelete = false">
      <div class="modal-box modal-small">
        <h3>确认删除</h3>
        <p>确定要删除这个网络资源吗？<br /><code>{{ deleteTarget?.url }}</code></p>
        <div v-if="error" class="form-error">{{ error }}</div>
        <div class="modal-actions">
          <button class="btn-cancel" @click="showDelete = false">取消</button>
          <button class="btn-save btn-danger" @click="doDelete()">确认删除</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, computed } from 'vue'
import { fetchNetworkResources, createNetworkResource, updateNetworkResource, deleteNetworkResource, fetchNetworkResourceReferences, type NetworkResource, type ReferenceItem } from '@/api/admin'
import { useSettingsStore } from '@/stores/settings'

const items = ref<NetworkResource[]>([])
const loading = ref(true)
const page = ref(1)
const pageSize = 20
const settingsStore = useSettingsStore()
const pagination = ref<{ total: number; pages: number; page: number; page_size: number } | null>(null)
const searchText = ref('')

const showModal = ref(false)
const editing = ref(false)
const saving = ref(false)
const error = ref('')
const form = reactive({ url: '', label: '', source_type: 'image' })
const editId = ref(0)

const showDelete = ref(false)
const deleteTarget = ref<NetworkResource | null>(null)

// 引用清单
const showRefs = ref(false)
const refsLoading = ref(false)
const refs = ref<ReferenceItem[]>([])
const refTarget = ref<NetworkResource | null>(null)

const catRefs = computed(() => refs.value.filter(r => r.target_type === 'category'))
const postRefs = computed(() => refs.value.filter(r => r.target_type === 'post'))

async function load() {
  loading.value = true
  try {
    const { data } = await fetchNetworkResources({
      page: page.value,
      page_size: pageSize,
      search: searchText.value || undefined,
    })
    items.value = data.data
    pagination.value = data.pagination || null
  } catch {
    error.value = '加载失败'
  } finally {
    loading.value = false
  }
}

function search() { page.value = 1; load() }
function clearSearch() { searchText.value = ''; page.value = 1; load() }
function goPage(p: number) { page.value = p; load() }

function openAdd() {
  editing.value = false
  editId.value = 0
  form.url = ''
  form.label = ''
  form.source_type = 'image'
  error.value = ''
  showModal.value = true
}

function openEdit(item: NetworkResource) {
  editing.value = true
  editId.value = item.id
  form.url = item.url
  form.label = item.label || ''
  form.source_type = item.source_type
  error.value = ''
  showModal.value = true
}

async function openRefs(item: NetworkResource) {
  refTarget.value = item
  refs.value = []
  refsLoading.value = true
  showRefs.value = true
  try {
    const { data } = await fetchNetworkResourceReferences(item.id)
    refs.value = data.data
  } catch {
    error.value = '加载引用失败'
  } finally {
    refsLoading.value = false
  }
}

async function save() {
  saving.value = true
  error.value = ''
  try {
    if (editing.value) {
      await updateNetworkResource(editId.value, { url: form.url, label: form.label })
      // 立即更新导航栏 Logo 缓存（可能引用了该网络资源）
      settingsStore.networkUrlCache.set(editId.value, form.url)
    } else {
      await createNetworkResource({ url: form.url, label: form.label, source_type: form.source_type })
    }
    showModal.value = false
    load()
  } catch (e: any) {
    error.value = e?.response?.data?.error || '保存失败'
  } finally {
    saving.value = false
  }
}

function confirmDelete(item: NetworkResource) {
  deleteTarget.value = item
  error.value = ''
  showDelete.value = true
}

async function doDelete() {
  if (!deleteTarget.value) return
  try {
    await deleteNetworkResource(deleteTarget.value.id)
    showDelete.value = false
    deleteTarget.value = null
    load()
  } catch (e: any) {
    error.value = e?.response?.data?.error || '删除失败'
  }
}

onMounted(load)
</script>

<style scoped>
.network-resources-page {
  width: 100%;
  max-width: 100%;
  min-width: 0;
  padding: 24px;
  box-sizing: border-box;
}

.btn-add { padding: 8px 20px; background: #4f46e5; color: #fff; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; }
.btn-add:hover { background: #4338ca; }

/* 搜索栏 — 使用 AdminLayout 变量 */
.search-bar { display: flex; gap: 8px; max-width: 100%; margin-bottom: 20px; }
.search-input {
  flex: 1; min-width: 0; padding: 9px 14px;
  border: 1px solid var(--card-border-color);
  border-radius: 10px;
  background: var(--card-bg);
  color: var(--text-primary);
  font-size: 14px; outline: none;
  transition: border-color 0.2s;
}
.search-input::placeholder { color: var(--text-dim); }
.search-input:focus { border-color: rgba(79, 70, 229, 0.4); }
.btn-search { padding: 8px 16px; background: #4f46e5; color: #fff; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; }
.btn-search:hover { background: #4338ca; }
.btn-clear { padding: 8px 16px; background: transparent; border: 1px solid var(--card-border-color); border-radius: 8px; cursor: pointer; font-size: 14px; color: var(--text-secondary); }
.btn-clear:hover { color: var(--text-primary); }

.loading { text-align: center; padding: 40px; color: var(--text-dim); }

/* 缩略图 */
.thumb-img { width: 60px; height: 45px; object-fit: cover; border-radius: 4px; border: 1px solid var(--card-border-color); cursor: pointer; transition: opacity 0.15s; }
.thumb-img:hover { opacity: 0.8; }
.thumb-placeholder { color: var(--text-dim); font-size: 13px; }

.table-scroll {
  width: 100%;
  max-width: 100%;
  min-width: 0;
  overflow-x: auto;
  overscroll-behavior-x: contain;
  -webkit-overflow-scrolling: touch;
}
.data-table { width: 930px; min-width: 930px; border-collapse: collapse; table-layout: fixed; }
.data-table th { text-align: left; padding: 10px 12px; border-bottom: 2px solid var(--card-border-color); font-size: 13px; color: var(--text-dim); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.data-table td { padding: 10px 12px; border-bottom: 1px solid rgba(128,128,128,0.1); font-size: 14px; color: var(--text-primary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.url-cell { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.type-badge { display: inline-block; padding: 2px 8px; border-radius: 10px; font-size: 12px; background: rgba(79,70,229,0.12); color: #818cf8; }
.type-cat { background: rgba(245,158,11,0.12); color: #f59e0b; }
.type-post { background: rgba(59,130,246,0.12); color: #60a5fa; }
.actions { white-space: nowrap; }
.btn-action { padding: 4px 10px; border: 1px solid var(--card-border-color); border-radius: 4px; background: transparent; cursor: pointer; font-size: 13px; margin-right: 4px; color: var(--text-secondary); }
.btn-action:hover { background: var(--filter-bg); color: var(--text-primary); }
.btn-danger { color: #f87171; border-color: rgba(248,113,113,0.3); }
.btn-danger:hover { background: rgba(239,68,68,0.1); }
.empty-row { text-align: center; padding: 40px; color: var(--text-dim); }

/* 引用清单弹窗 — 卡片式 */
.refs-modal {
  background: var(--modal-bg);
  border: 1px solid rgba(255,255,255,0.08);
  border-radius: 16px;
  width: 90%;
  max-width: 580px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.refs-preview {
  display: flex;
  gap: 14px;
  padding: 20px 24px;
  border-bottom: 1px solid rgba(128,128,128,0.1);
  background: var(--input-bg);
}
.refs-thumb {
  width: 80px; height: 56px;
  object-fit: cover; border-radius: 8px;
  border: 1px solid var(--card-border-color);
  flex-shrink: 0;
}
.refs-info { overflow: hidden; display: flex; flex-direction: column; justify-content: center; gap: 4px; }
.refs-label { font-size: 15px; font-weight: 600; color: var(--text-primary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.refs-url { font-size: 12px; color: var(--text-dim); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

.refs-loading { display: flex; align-items: center; justify-content: center; gap: 10px; padding: 40px 0; color: var(--text-dim); font-size: 14px; }
.spinner-sm { width: 20px; height: 20px; border: 2px solid rgba(79,70,229,0.15); border-top-color: #4f46e5; border-radius: 50%; animation: spin 0.6s linear infinite; flex-shrink: 0; }
@keyframes spin { to { transform: rotate(360deg); } }

.refs-empty { display: flex; flex-direction: column; align-items: center; gap: 8px; padding: 48px 0; color: var(--text-dim); }
.refs-empty svg { color: #34d399; }
.refs-empty p { font-size: 14px; margin: 0; color: var(--text-secondary); }
.refs-empty span { font-size: 12px; }

.refs-summary { display: flex; align-items: center; gap: 6px; padding: 14px 24px 0; font-size: 13px; }
.refs-count-cat { color: #f59e0b; }
.refs-count-post { color: #60a5fa; }
.refs-dot { color: var(--text-dim); }

.refs-list { overflow-y: auto; padding: 10px 24px 16px; display: flex; flex-direction: column; gap: 8px; max-height: 340px; }
.refs-item { display: flex; gap: 12px; padding: 12px 14px; border-radius: 10px; background: var(--input-bg); border: 1px solid rgba(128,128,128,0.06); transition: border-color 0.15s; }
.refs-item:hover { border-color: rgba(79,70,229,0.2); }
.refs-item-icon { width: 36px; height: 36px; border-radius: 8px; display: flex; align-items: center; justify-content: center; font-size: 16px; flex-shrink: 0; }
.icon-cat { background: rgba(245,158,11,0.1); }
.icon-post { background: rgba(59,130,246,0.1); }
.refs-item-body { flex: 1; min-width: 0; }
.refs-item-name { font-size: 14px; font-weight: 500; color: var(--text-primary); display: flex; align-items: center; gap: 8px; }
.refs-item-type { font-size: 11px; padding: 1px 7px; border-radius: 8px; background: rgba(128,128,128,0.08); color: var(--text-dim); font-weight: 400; }
.refs-item-slug { font-size: 12px; color: var(--text-dim); margin-top: 2px; }
.refs-item-desc { font-size: 12px; color: var(--text-secondary); margin-top: 4px; line-height: 1.4; }

.refs-footer { padding: 14px 24px; border-top: 1px solid rgba(128,128,128,0.1); display: flex; justify-content: flex-end; }

.pagination { display: flex; justify-content: center; align-items: center; gap: 16px; margin-top: 20px; font-size: 14px; color: var(--text-dim); }
.pagination button { padding: 6px 14px; border: 1px solid var(--card-border-color); border-radius: 6px; background: var(--card-bg); cursor: pointer; color: var(--text-secondary); }
.pagination button:disabled { opacity: 0.4; cursor: not-allowed; }

.modal-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.6); display: flex; justify-content: center; align-items: center; z-index: 100; }
.modal-box { background: var(--modal-bg); border: 1px solid rgba(255,255,255,0.08); border-radius: 12px; padding: 24px; width: 90%; max-width: 480px; color: var(--text-primary); }
.modal-small { max-width: 360px; }
.modal-box h3 { margin: 0 0 16px; font-size: 1.1rem; color: var(--text-primary); }
.modal-box code { word-break: break-all; font-size: 0.85rem; opacity: 0.8; }
.form-row { margin-bottom: 14px; }
.form-row label { display: block; margin-bottom: 4px; font-size: 13px; color: var(--text-dim); }
.form-input { width: 100%; padding: 8px 12px; border: 1px solid var(--card-border-color); border-radius: 8px; background: var(--input-bg); color: var(--input-color); font-size: 14px; outline: none; box-sizing: border-box; }
.form-input:focus { border-color: rgba(79,70,229,0.4); }
.form-error { padding: 8px 12px; background: rgba(239,68,68,0.08); color: #f87171; border-radius: 6px; font-size: 13px; margin-bottom: 12px; }
.modal-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 16px; }
.btn-cancel { padding: 8px 16px; border: 1px solid var(--card-border-color); border-radius: 8px; background: transparent; cursor: pointer; font-size: 14px; color: var(--text-secondary); }
.btn-cancel:hover { color: var(--text-primary); }
.btn-save { padding: 8px 20px; background: #4f46e5; color: #fff; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; }
.btn-save:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-save.btn-danger { background: #dc2626; }

@media (max-width: 640px) {
  .network-resources-page {
    padding: 12px 0 0;
  }
  .search-bar {
    flex-wrap: wrap;
  }
  .search-input {
    flex: 1 0 100%;
  }
  .btn-search,
  .btn-clear,
  .btn-add {
    flex: 1;
    min-width: 0;
    padding-left: 8px;
    padding-right: 8px;
    white-space: nowrap;
  }
}
</style>
