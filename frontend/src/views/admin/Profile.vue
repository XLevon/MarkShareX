<template>
  <!-- Page mode: standalone page with title -->
  <div v-if="mode === 'page'" class="profile-page">
    <h1 class="page-title">个人信息</h1>

    <div v-if="loading" class="loading-state"><div class="spinner"></div></div>

    <div v-else-if="profile" class="profile-card">
      <!-- Header -->
      <div class="profile-header">
        <div class="avatar">{{ (profile.display_name || profile.username)[0] }}</div>
        <div class="profile-meta">
          <h2>{{ profile.display_name || profile.username }}</h2>
          <span class="role-tag">{{ roleName }}</span>
        </div>
      </div>
      <!-- Form -->
      <form @submit.prevent="handleSave" class="profile-form">
        <!-- 用户名 -->
        <div class="form-row">
          <label class="form-label">用户名</label>
          <input type="text" :value="profile.username" disabled class="form-input disabled" />
        </div>
        <!-- 角色 -->
        <div class="form-row">
          <label class="form-label">角色</label>
          <input type="text" :value="roleName" disabled class="form-input disabled" />
        </div>
        <!-- 昵称 -->
        <div class="form-row">
          <label class="form-label">昵称</label>
          <input v-model="form.display_name" type="text" class="form-input" placeholder="显示名称" />
        </div>
        <!-- 邮箱 -->
        <div class="form-row">
          <label class="form-label">邮箱</label>
          <input :value="profile.email" type="email" disabled class="form-input disabled" />
        </div>
        <!-- 抬头 -->
        <div class="form-row">
          <label class="form-label">抬头</label>
          <input v-model="form.title" type="text" class="form-input" placeholder="全栈开发者 / 站长" />
        </div>
        <!-- 简介 -->
        <div class="form-row">
          <label class="form-label">简介</label>
          <textarea v-model="form.bio" class="form-textarea" rows="8" placeholder="写一段自我介绍..." />
        </div>
        <div v-if="error" class="error-msg">{{ error }}</div>
        <div v-if="success" class="success-msg">{{ success }}</div>
        <div class="form-actions">
          <button type="submit" class="btn-save" :disabled="saving">{{ saving ? '保存中...' : '保存修改' }}</button>
        </div>
      </form>
    </div>
  </div>

  <!-- Modal mode: compact form used inside parent modal wrapper -->
  <div v-else>
    <div v-if="loading" class="flex justify-center py-10">
      <div class="spinner"></div>
    </div>

    <form v-else @submit.prevent="handleSave" class="profile-form-modal">
      <div class="form-row">
        <label class="form-label">用户名</label>
        <input type="text" :value="profile?.username" disabled class="form-input disabled" />
      </div>
      <div class="form-row">
        <label class="form-label">角色</label>
        <input type="text" :value="roleName" disabled class="form-input disabled" />
      </div>
      <div class="form-row">
        <label class="form-label">昵称</label>
        <input v-model="form.display_name" type="text" class="form-input" placeholder="显示名称" />
      </div>
      <div class="form-row">
        <label class="form-label">邮箱</label>
        <input :value="profile?.email" type="email" disabled class="form-input disabled" />
      </div>
      <div class="form-row">
        <label class="form-label">抬头</label>
        <input v-model="form.title" type="text" class="form-input" placeholder="全栈开发者 / 站长" />
      </div>
      <div class="form-row">
        <label class="form-label">简介</label>
        <textarea v-model="form.bio" class="form-textarea" rows="3" placeholder="写一段自我介绍..." />
      </div>
      <div v-if="error" class="error-msg">{{ error }}</div>
      <div v-if="success" class="success-msg">{{ success }}</div>
      <div class="form-actions-modal">
        <button type="button" class="btn-cancel" @click="emit('close')" :style="{ color: 'var(--color-text-secondary)' }">取消</button>
        <button type="submit" class="btn-save" :disabled="saving">{{ saving ? '保存中...' : '保存' }}</button>
      </div>
    </form>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, computed } from 'vue'
import { fetchProfile, updateProfile } from '@/api/admin'

const props = withDefaults(defineProps<{ mode?: 'page' | 'modal' }>(), { mode: 'page' })
const emit = defineEmits<{ close: []; saved: [] }>()

const loading = ref(true)
const saving = ref(false)
const error = ref('')
const success = ref('')
const profile = ref<any>(null)
const form = reactive({ display_name: '', bio: '', title: '' })

const roleMap: Record<string, string> = { admin: '管理员', sub_admin: '子管理员', author: '作者', visitor: '访客' }
const roleName = computed(() => roleMap[profile.value?.role] || profile.value?.role || '')

async function load() {
  loading.value = true; error.value = ''
  try {
    const { data } = await fetchProfile()
    profile.value = data.data
    form.display_name = data.data.display_name || ''
    form.bio = data.data.bio || ''
    form.title = data.data.title || ''
  } catch { error.value = '加载失败' }
  finally { loading.value = false }
}

async function handleSave() {
  saving.value = true; error.value = ''; success.value = ''
  try {
    const payload: Record<string, string> = {}
    if (form.display_name !== (profile.value?.display_name || '')) payload.display_name = form.display_name
    if (form.bio !== (profile.value?.bio || '')) payload.bio = form.bio
    if (form.title !== (profile.value?.title || '')) payload.title = form.title
    if (Object.keys(payload).length === 0) { success.value = '没有需要保存的修改'; return }
    const { data } = await updateProfile(payload)
    profile.value = data.data
    success.value = '保存成功'
    emit('saved')
  } catch (e: any) { error.value = e?.response?.data?.error || '保存失败' }
  finally { saving.value = false }
}

onMounted(load)
</script>

<style scoped>
.profile-page { max-width: 680px; animation: fadeIn 0.3s ease; }
@keyframes fadeIn { from { opacity: 0; transform: translateY(8px); } to { opacity: 1; transform: translateY(0); } }
.page-title { font-size: 28px; font-weight: 700; color: var(--input-color); margin-bottom: 24px; }
.loading-state { display: flex; justify-content: center; padding: 60px 0; }
.spinner { width: 32px; height: 32px; border: 3px solid rgba(79,70,229,0.15); border-top-color: #4f46e5; border-radius: 50%; animation: spin 0.6s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
.profile-card { background: var(--card-bg); border: 1px solid rgba(255,255,255,0.06); border-radius: 14px; padding: 32px; }
.profile-header { display: flex; align-items: center; gap: 16px; margin-bottom: 28px; padding-bottom: 20px; border-bottom: 1px solid rgba(255,255,255,0.06); }
.avatar { width: 56px; height: 56px; border-radius: 50%; background: #4f46e5; color: #fff; display: flex; align-items: center; justify-content: center; font-size: 22px; font-weight: 700; flex-shrink: 0; }
.profile-meta h2 { margin: 0 0 4px; font-size: 18px; color: var(--input-color); }
.role-tag { font-size: 12px; color: #818cf8; background: rgba(79,70,229,0.1); padding: 2px 8px; border-radius: 6px; }

/* ── Horizontal form rows ── */
.profile-form { display: flex; flex-direction: column; gap: 14px; }
.profile-form-modal { display: flex; flex-direction: column; gap: 10px; }
.form-row { display: flex; align-items: flex-start; gap: 12px; }
.form-label { width: 56px; flex-shrink: 0; text-align: right; font-size: 13px; color: var(--text-secondary); font-weight: 500; line-height: 34px; }
.form-input, .form-textarea { flex: 1; padding: 8px 12px; border-radius: 8px; border: 1px solid rgba(255,255,255,0.08); background: var(--input-bg); color: var(--input-color); font-size: 14px; outline: none; transition: border-color 0.15s; box-sizing: border-box; min-width: 0; }
.form-input:focus, .form-textarea:focus { border-color: rgba(79,70,229,0.4); }
.form-input.disabled { opacity: 0.5; cursor: not-allowed; }
.form-textarea { resize: vertical; min-height: 200px; font-family: inherit; }
.form-input::placeholder, .form-textarea::placeholder { color: #4b5563; }
.error-msg { background: rgba(239,68,68,0.08); color: #f87171; padding: 9px 12px; border-radius: 8px; font-size: 13px; }
.success-msg { background: rgba(16,185,129,0.08); color: #34d399; padding: 9px 12px; border-radius: 8px; font-size: 13px; }
.form-actions { display: flex; justify-content: flex-end; padding-top: 4px; }
.form-actions-modal { display: flex; justify-content: flex-end; padding-top: 4px; gap: 8px; }
.btn-cancel { padding: 8px 18px; border-radius: 8px; border: 1px solid rgba(255,255,255,0.15); background: transparent; cursor: pointer; font-size: 13px; color: var(--color-text-secondary); transition: all 0.15s; }
.btn-save { padding: 8px 20px; border-radius: 8px; border: none; background: #4f46e5; color: #fff; font-size: 13px; font-weight: 500; cursor: pointer; transition: background 0.15s; }
.btn-save:hover { background: #4338ca; }
.btn-save:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
