<template>
  <div class="password-page">
    <h1 class="page-title">修改密码</h1>

    <div class="password-card">
      <form @submit.prevent="handleSave" class="password-form">
        <div class="form-group">
          <label>旧密码</label>
          <input v-model="form.old_password" type="password" class="form-input" placeholder="输入当前密码" required />
        </div>
        <div class="form-group">
          <label>新密码</label>
          <input v-model="form.new_password" type="password" class="form-input" placeholder="至少 8 位" required minlength="8" />
        </div>
        <div class="form-group">
          <label>确认新密码</label>
          <input v-model="form.confirm_password" type="password" class="form-input" placeholder="再次输入新密码" required minlength="8" />
        </div>
        <div v-if="error" class="error-msg">{{ error }}</div>
        <div v-if="success" class="success-msg">{{ success }}</div>
        <div class="form-actions">
          <button type="submit" class="btn-save" :disabled="saving">
            {{ saving ? '保存中...' : '修改密码' }}
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { changePassword } from '@/api/admin'

const saving = ref(false)
const error = ref('')
const success = ref('')

const form = reactive({
  old_password: '',
  new_password: '',
  confirm_password: '',
})

async function handleSave() {
  error.value = ''
  success.value = ''

  if (form.new_password !== form.confirm_password) {
    error.value = '两次输入的新密码不一致'
    return
  }

  saving.value = true
  try {
    await changePassword({
      old_password: form.old_password,
      new_password: form.new_password,
      confirm_password: form.confirm_password,
    })
    success.value = '密码修改成功'
    form.old_password = ''
    form.new_password = ''
    form.confirm_password = ''
  } catch (e: any) {
    error.value = e?.response?.data?.error || '修改失败'
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
.password-page {
  max-width: 480px;
  animation: fadeIn 0.3s ease;
}
@keyframes fadeIn {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
}
.page-title { font-size: 28px; font-weight: 700; color: var(--input-color); margin-bottom: 24px; }

.password-card {
  background: var(--card-bg);
  border: 1px solid rgba(255,255,255,0.06);
  border-radius: 14px;
  padding: 32px;
}

.password-form { display: flex; flex-direction: column; gap: 18px; }
.form-group { display: flex; flex-direction: column; gap: 6px; }
.form-group label {
  font-size: 13px; color: var(--text-secondary); font-weight: 500;
}
.form-input {
  padding: 10px 14px; border-radius: 8px;
  border: 1px solid rgba(255,255,255,0.08);
  background: var(--input-bg); color: var(--input-color);
  font-size: 14px; outline: none; transition: border-color 0.15s;
}
.form-input:focus { border-color: rgba(79,70,229,0.4); }
.form-input::placeholder { color: #4b5563; }

.error-msg {
  background: rgba(239,68,68,0.08); color: #f87171;
  padding: 10px 14px; border-radius: 8px; font-size: 13px;
}
.success-msg {
  background: rgba(16,185,129,0.08); color: #34d399;
  padding: 10px 14px; border-radius: 8px; font-size: 13px;
}

.form-actions { display: flex; justify-content: flex-end; padding-top: 8px; }
.btn-save {
  padding: 10px 24px; border-radius: 8px; border: none;
  background: #4f46e5; color: #fff; font-size: 14px; font-weight: 500;
  cursor: pointer; transition: background 0.15s;
}
.btn-save:hover { background: #4338ca; }
.btn-save:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
