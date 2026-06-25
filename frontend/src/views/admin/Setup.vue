<template>
  <div class="setup-page min-h-screen flex items-center justify-center bg-[#0f0f1a]">
    <div class="w-full max-w-lg p-8 bg-[#1e1e2e] rounded-xl shadow-2xl">
      <h1 class="text-2xl font-bold text-center text-[#e94560] mb-2">系统初始化</h1>
      <p class="text-gray-400 text-center mb-8">首次使用，请创建管理员账户</p>
      <n-form ref="formRef" :model="form" :rules="rules" @submit.prevent="handleSetup">
        <n-form-item path="username" label="用户名">
          <n-input v-model:value="form.username" placeholder="管理员登录用户名" />
        </n-form-item>
        <n-form-item path="display_name" label="显示名称">
          <n-input v-model:value="form.display_name" placeholder="显示在网站上的名称" />
        </n-form-item>
        <n-form-item path="email" label="邮箱">
          <n-input v-model:value="form.email" placeholder="管理员邮箱" />
        </n-form-item>
        <n-form-item path="password" label="密码">
          <n-input v-model:value="form.password" type="password" placeholder="请输入密码" show-password-on="click" />
        </n-form-item>
        <n-form-item path="password_confirm" label="确认密码">
          <n-input v-model:value="form.password_confirm" type="password" placeholder="再次输入密码" show-password-on="click" />
        </n-form-item>
        <n-form-item path="bio" label="简介">
          <n-input v-model:value="form.bio" type="textarea" placeholder="介绍一下你自己（选填）" :rows="3" />
        </n-form-item>

        <n-button type="primary" block :loading="loading" attr-type="submit">
          完成初始化
        </n-button>
      </n-form>
      <div v-if="error" class="mt-4 text-red-400 text-center text-sm">{{ error }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { fetchSetupStatus, setupSystem } from '@/api/setup'

const router = useRouter()
const loading = ref(false)
const error = ref('')

const form = reactive({
  username: '',
  display_name: '',
  email: '',
  password: '',
  password_confirm: '',
  bio: '',
})

const rules = {
  username: { required: true, message: '请输入用户名', trigger: 'blur' },
  display_name: { required: true, message: '请输入显示名称', trigger: 'blur' },
  email: { required: true, message: '请输入邮箱', trigger: 'blur' },
  password: { required: true, message: '请输入密码', trigger: 'blur' },
  password_confirm: [
    { required: true, message: '请确认密码', trigger: 'blur' },
    { validator: (_: any, value: string) => value === form.password, message: '两次密码不一致', trigger: 'blur' },
  ],
}

onMounted(async () => {
  try {
    const { data: resp } = await fetchSetupStatus()
    if (resp.data.initialized) {
      router.replace('/login')
    }
  } catch {
    // ignore
  }
})

async function handleSetup() {
  loading.value = true
  error.value = ''
  try {
    await setupSystem({
      username: form.username,
      display_name: form.display_name,
      email: form.email,
      password: form.password,
      bio: form.bio || undefined,
    })
    router.push('/')
  } catch (e: any) {
    error.value = e.response?.data?.error || '初始化失败'
  } finally {
    loading.value = false
  }
}
</script>
