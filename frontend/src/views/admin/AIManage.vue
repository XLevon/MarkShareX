<template>
  <div class="ai-manage">
    <div class="page-header">
      <h2>🤖 AI 模块</h2>
    </div>

    <n-card>
      <n-tabs v-model:value="activeTab" type="line">
        <!-- Providers -->
        <n-tab-pane name="providers" tab="供应商">
          <div class="tab-toolbar">
            <n-button type="primary" size="small" @click="openProviderForm()">+ 添加供应商</n-button>
          </div>
          <n-data-table :columns="providerColumns" :data="providers" :loading="loading" size="small" />
          <div style="margin-top: 24px">
            <h4 style="margin: 0 0 8px">模型列表</h4>
            <div class="tab-toolbar">
              <n-select v-model:value="modelProviderFilter" :options="providerOptions" placeholder="按供应商筛选" clearable style="width:200px" />
              <n-button type="primary" size="small" @click="openModelForm()" :disabled="!modelProviderFilter">+ 添加模型</n-button>
            </div>
            <n-data-table :columns="modelColumns" :data="filteredModels" :loading="loading" size="small" />
          </div>
        </n-tab-pane>

        <!-- Tools -->
        <n-tab-pane name="tools" tab="工具">
          <div class="tab-toolbar">
            <n-button type="primary" size="small" @click="openToolForm()">+ 添加工具</n-button>
          </div>
          <n-data-table :columns="toolColumns" :data="tools" :loading="loading" size="small" />
        </n-tab-pane>

        <!-- Agent -->
        <n-tab-pane name="agent" tab="智能体">
          <div class="tab-toolbar">
            <n-button type="primary" size="small" @click="openAgentForm()">+ 添加智能体</n-button>
          </div>
          <n-data-table :columns="agentColumns" :data="agentConfigs" :loading="loading" size="small" />
        </n-tab-pane>

        <!-- Skills -->
        <n-tab-pane name="skills" tab="技能">
          <div class="tab-toolbar">
            <n-button type="primary" size="small" @click="openSkillForm()">+ 添加技能</n-button>
          </div>
          <n-data-table :columns="skillColumns" :data="skills" :loading="loading" size="small" />
        </n-tab-pane>

        <!-- Tasks -->
        <n-tab-pane name="tasks" tab="定时任务">
          <div class="tab-toolbar">
            <n-button type="primary" size="small" @click="openTaskForm()">+ 添加任务</n-button>
          </div>
          <n-data-table :columns="taskColumns" :data="tasks" :loading="loading" size="small" />
        </n-tab-pane>
      </n-tabs>
    </n-card>

    <!-- Dialogs follow -->
    <n-modal v-model:show="showProviderModal" :mask-closable="false">
      <n-card style="width:520px;max-width:90vw" :title="editingProviderId ? '编辑供应商' : '添加供应商'">
        <n-form label-placement="left" label-width="80">
          <n-form-item label="名称" required><n-input v-model:value="providerForm.name" placeholder="硅基流动" /></n-form-item>
          <n-form-item label="类型"><n-select v-model:value="providerForm.provider_type" :options="providerTypeOptions" /></n-form-item>
          <n-form-item label="Base URL"><n-input v-model:value="providerForm.base_url" placeholder="https://api.siliconflow.cn/v1" /></n-form-item>
          <n-form-item label="API Key" required>
            <n-input
              v-model:value="providerForm.api_key"
              type="text"
              :placeholder="editingProviderId ? '点击输入新 Key，留空不修改' : 'sk-...'"
              @focus="onApiKeyFocus"
            />
          </n-form-item>
          <n-form-item v-if="editingProviderId" label="启用">
            <n-switch v-model:value="providerForm.is_active" />
          </n-form-item>
        </n-form>
        <template #footer><n-space justify="end">
          <n-button @click="showProviderModal=false">取消</n-button>
          <n-button type="primary" :loading="saving" @click="saveProvider">{{ editingProviderId ? '保存' : '创建' }}</n-button>
        </n-space></template>
      </n-card>
    </n-modal>

    <!-- Agent Modal -->
    <n-modal v-model:show="showAgentModal" :mask-closable="false">
      <n-card style="width:640px;max-width:92vw" :title="editingAgentId ? '编辑智能体' : '添加智能体'">
        <n-form label-placement="left" label-width="80">
          <n-form-item label="名称" required><n-input v-model:value="agentForm.name" placeholder="对话框助手" /></n-form-item>
          <n-form-item label="系统提示词">
            <n-input v-model:value="agentForm.system_prompt" type="textarea" :rows="6"
              placeholder="你是 MarkShareX 的管理助手..." />
          </n-form-item>
          <n-form-item label="用户提示词">
            <n-input v-model:value="agentForm.user_prompt" type="textarea" :rows="4"
              placeholder="用户消息将追加在此提示词之后发送给 AI。" />
          </n-form-item>
          <n-form-item label="设为默认">
            <n-switch v-model:value="agentForm.is_default" />
          </n-form-item>
          <n-form-item label="模型">
            <n-select v-model:value="agentProviderFilter" :options="providerFilterOptions" placeholder="先选供应商" clearable style="margin-bottom:8px" />
            <n-select v-model:value="agentForm.model_id" :options="agentModelOptions" clearable placeholder="不选则使用默认" />
          </n-form-item>
        </n-form>
        <template #footer><n-space justify="end">
          <n-button @click="showAgentModal=false">取消</n-button>
          <n-button type="primary" :loading="saving" @click="saveAgent">{{ editingAgentId ? '保存' : '创建' }}</n-button>
        </n-space></template>
      </n-card>
    </n-modal>

    <!-- Skill Modal -->
    <n-modal v-model:show="showSkillModal" :mask-closable="false">
      <n-card style="width:700px;max-width:92vw" :title="editingSkillId ? '编辑技能' : '添加技能'">
        <n-form label-placement="top">
          <n-form-item label="名称" required><n-input v-model:value="skillForm.name" placeholder="每日AI快讯" /></n-form-item>
          <n-form-item label="描述"><n-input v-model:value="skillForm.description" placeholder="自动生成每日科技资讯摘要" /></n-form-item>
          <n-form-item label="指令内容（Markdown）" required>
            <n-input v-model:value="skillForm.content" type="textarea" :rows="14" placeholder="你是一个科技资讯编辑，请根据以下要求生成内容...
            
支持 {{变量}} 占位符，如 {{topic}}、{{count}}" />
          </n-form-item>
          <n-form-item label="输出格式">
            <n-select v-model:value="skillForm.output_format" :options="outputFormatOptions" />
          </n-form-item>
        </n-form>
        <template #footer><n-space justify="end">
          <n-button @click="showSkillModal=false">取消</n-button>
          <n-button type="primary" :loading="saving" @click="saveSkill">{{ editingSkillId ? '保存' : '创建' }}</n-button>
        </n-space></template>
      </n-card>
    </n-modal>

    <!-- Tool Modal -->
    <n-modal v-model:show="showToolModal" :mask-closable="false">
      <n-card style="width:560px;max-width:90vw" :title="editingToolId ? '编辑工具' : '添加工具'">
        <n-form label-placement="left" label-width="90">
          <n-form-item label="工具名称" required><n-input v-model:value="toolForm.name" placeholder="搜索文章" :disabled="!!editingToolId" /></n-form-item>
          <n-form-item label="功能名" required><n-input v-model:value="toolForm.function_name" placeholder="search_posts" :disabled="!!editingToolId" /></n-form-item>
          <n-form-item label="描述">
            <div style="width:100%">
              <n-input v-model:value="toolForm.description" type="textarea" :rows="4" placeholder="根据关键词搜索已发布的文章" />
              <div style="color:#d03050;font-size:12px;margin-top:4px">该描述会传给LLM，决定了AI何时调用此工具</div>
            </div>
          </n-form-item>
          <n-form-item label="参数 Schema">
            <n-input v-model:value="toolForm.parameters_schema" type="textarea" :rows="6" :disabled="!!editingToolId"
              placeholder='{"type":"object","properties":{"query":{"type":"string","description":"搜索关键词"}},"required":["query"]}' />
          </n-form-item>
          <n-form-item label="启用">
            <n-switch v-model:value="toolForm.enabled" />
          </n-form-item>
        </n-form>
        <template #footer><n-space justify="end">
          <n-button @click="showToolModal=false">取消</n-button>
          <n-button type="primary" :loading="saving" @click="saveTool">{{ editingToolId ? '保存' : '创建' }}</n-button>
        </n-space></template>
      </n-card>
    </n-modal>

    <!-- Task Modal -->
    <n-modal v-model:show="showTaskModal" :mask-closable="false">
      <n-card style="width:520px;max-width:90vw" :title="editingTaskId ? '编辑任务' : '添加任务'">
        <n-form label-placement="left" label-width="80">
          <n-form-item label="任务名称">
            <n-input v-model:value="taskForm.name" placeholder="如：每日AI资讯" />
          </n-form-item>
          <n-form-item label="Agent" required>
            <n-select v-model:value="taskForm.agent_config_id" :options="agentOptions" clearable placeholder="选择智能体" />
          </n-form-item>
          <n-form-item label="技能" required>
            <n-select v-model:value="taskForm.skill_id" :options="skillOptions" />
          </n-form-item>
          <n-form-item label="供应商">
            <n-select v-model:value="taskForm.provider_id" :options="providerOptions" clearable placeholder="不选则使用Agent的配置" />
          </n-form-item>
          <n-form-item label="模型">
            <n-select v-model:value="taskForm.model_id" :options="taskModelOptions" clearable placeholder="不选则使用Agent的配置" />
          </n-form-item>
          <n-form-item label="Cron 表达式" required>
            <n-input v-model:value="taskForm.cron_expr" placeholder="0 8 * * *" />
          </n-form-item>
          <n-form-item label="参数">
            <n-input v-model:value="taskForm.params" placeholder='{"topic":"AI","count":3}' />
          </n-form-item>
          <n-form-item label="启用">
            <n-switch v-model:value="taskForm.enabled" />
          </n-form-item>
        </n-form>
        <template #footer><n-space justify="end">
          <n-button @click="showTaskModal=false">取消</n-button>
          <n-button type="primary" :loading="saving" @click="saveTask">{{ editingTaskId ? '保存' : '创建' }}</n-button>
        </n-space></template>
      </n-card>
    </n-modal>

    <!-- Model Modal -->
    <n-modal v-model:show="showModelModal" :mask-closable="false">
      <n-card style="width:400px;max-width:90vw" :title="editingModelId ? '编辑模型' : '添加模型'">
        <n-form label-placement="left" label-width="80">
          <n-form-item label="供应商" required>
            <n-select v-model:value="modelForm.provider_id" :options="providerOptions" :disabled="!!editingModelId" />
          </n-form-item>
          <n-form-item label="模型名" required>
            <n-input v-model:value="modelForm.name" placeholder="deepseek-ai/DeepSeek-V3" />
          </n-form-item>
          <n-form-item v-if="editingModelId" label="默认">
            <n-switch v-model:value="modelForm.is_default" />
          </n-form-item>
        </n-form>
        <template #footer><n-space justify="end">
          <n-button @click="showModelModal=false">取消</n-button>
          <n-button type="primary" :loading="saving" @click="saveModel">{{ editingModelId ? '保存' : '创建' }}</n-button>
        </n-space></template>
      </n-card>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, h } from 'vue'
import { NButton, NTag, NSpace, NSwitch, NSelect, useMessage } from 'naive-ui'
import {
  fetchProviders, createProvider, updateProvider, deleteProvider, testProvider, type AiProvider,
  fetchSkills, createSkill, updateSkill, deleteSkill, type AiSkill,
  fetchTasks, createTask, updateTask, deleteTask, type AiTask,
  fetchAgentConfigs, createAgentConfig, updateAgentConfig, deleteAgentConfig, type AgentConfig,
  fetchTools, createTool, updateTool, deleteTool, type AiTool,
  fetchModels, createModel, updateModel, deleteModel, type AiModel,
} from '@/api/ai'

const message = useMessage()
const activeTab = ref('providers')
const loading = ref(false)
const saving = ref(false)

// ── Agent Config ──
const agentConfigs = ref<AgentConfig[]>([])
const showAgentModal = ref(false)
const editingAgentId = ref<number | null>(null)
const agentForm = ref({ name: '', system_prompt: '', user_prompt: '', is_default: false, model_id: undefined as number | undefined })
const agentProviderFilter = ref<number | null>(null)
const providerFilterOptions = computed(() => providers.value.map(p => ({ label: p.name, value: p.id })))
const agentModelOptions = computed(() => {
  if (!agentProviderFilter.value) return []
  return models.value.filter(m => m.provider_id === agentProviderFilter.value).map(m => ({ label: m.name, value: m.id }))
})
const agentColumns = [
  { title: '名称', key: 'name', width: 140 },
  { title: '系统提示词', key: 'system_prompt', ellipsis: { tooltip: true }, render(row: AgentConfig) { return row.system_prompt || '-' } },
  { title: '默认', key: 'is_default', width: 70, render(row: AgentConfig) {
    return h(NSwitch, { size: 'small', value: row.is_default, onUpdateValue: (v: boolean) => toggleAgentDefault(row, v) })
  }},
  { title: '操作', key: 'actions', width: 140, render(row: AgentConfig) {
    return h(NSpace, { size: 'small' }, { default: () => [
      h(NButton, { size: 'small', onClick: () => openAgentForm(row) }, { default: () => '编辑' }),
      h(NButton, { size: 'small', type: 'error', onClick: () => handleDeleteAgent(row) }, { default: () => '删除' }),
    ]})
  }},
]

async function loadAgentConfigs() { loading.value = true; try { const r = await fetchAgentConfigs(); agentConfigs.value = r.data.data || [] } catch {} finally { loading.value = false } }
function openAgentForm(row?: AgentConfig) {
  editingAgentId.value = row?.id ?? null
  agentForm.value = row ? { name: row.name, system_prompt: row.system_prompt, user_prompt: row.user_prompt, is_default: row.is_default, model_id: row.model_id ?? undefined } : { name: '', system_prompt: '', user_prompt: '', is_default: false, model_id: undefined }
  // 根据已有模型自动推导供应商筛选
  if (row?.model_id) {
    const m = models.value.find(m => m.id === row.model_id)
    agentProviderFilter.value = m?.provider_id ?? null
  } else {
    agentProviderFilter.value = null
  }
  showAgentModal.value = true
}
async function saveAgent() {
  if (!agentForm.value.name) { message.warning('名称为必填'); return }
  saving.value = true
  try {
    if (editingAgentId.value) await updateAgentConfig(editingAgentId.value, agentForm.value)
    else await createAgentConfig(agentForm.value)
    showAgentModal.value = false
    loadAgentConfigs()
    message.success('Agent 配置已保存')
  } catch { message.error('保存失败') } finally { saving.value = false }
}
async function handleDeleteAgent(row: AgentConfig) {
  if (!confirm(`确定删除「${row.name}」？`)) return
  try { await deleteAgentConfig(row.id); loadAgentConfigs() } catch (e: any) { message.error(e?.response?.data?.error || '删除失败') }
}
async function toggleAgentDefault(row: AgentConfig, v: boolean) {
  try {
    await updateAgentConfig(row.id, { is_default: v })
    if (v) agentConfigs.value.forEach(a => { if (a.id !== row.id) a.is_default = false })
    row.is_default = v
    message.success(v ? '已设为默认' : '已取消默认')
  } catch (e: any) { message.error(e?.response?.data?.error || '切换失败') }
}

const agentOptions = computed(() => agentConfigs.value.map(a => ({ label: a.name + (a.is_default ? ' (默认)' : ''), value: a.id })))

// ── Models ──
const models = ref<AiModel[]>([])
const modelProviderFilter = ref<number | null>(null)
const showModelModal = ref(false)
const editingModelId = ref<number | null>(null)
const modelForm = ref({ provider_id: 0, name: '', is_default: false })
const modelColumns = [
  { title: '供应商', key: 'provider_id', width: 130, render(row: AiModel) { return providers.value.find(p => p.id === row.provider_id)?.name || '-' } },
  { title: '模型名', key: 'name', width: 120 },
  { title: '默认', key: 'is_default', width: 70, render(row: AiModel) {
    return h(NSwitch, { size: 'small', value: row.is_default, onUpdateValue: (v: boolean) => toggleModelDefault(row, v) })
  }},
  { title: '操作', key: 'actions', width: 140, render(row: AiModel) {
    return h(NSpace, { size: 'small' }, { default: () => [
      h(NButton, { size: 'small', onClick: () => openModelForm(row) }, { default: () => '编辑' }),
      h(NButton, { size: 'small', type: 'error', onClick: () => handleDeleteModel(row) }, { default: () => '删除' }),
    ]})
  }},
]
const filteredModels = computed(() => {
  if (!modelProviderFilter.value) return models.value
  return models.value.filter(m => m.provider_id === modelProviderFilter.value)
})
const taskModelOptions = computed(() => {
  if (!taskForm.value.provider_id) return models.value.map(m => ({ label: m.name, value: m.id }))
  return models.value.filter(m => m.provider_id === taskForm.value.provider_id).map(m => ({ label: m.name, value: m.id }))
})
const allModelOptions = computed(() => models.value.map(m => ({ label: m.name, value: m.id })))

async function loadModels() {
  try { const r = await fetchModels(); models.value = r.data.data || [] } catch {}
}
function openModelForm(row?: AiModel) {
  editingModelId.value = row?.id ?? null
  modelForm.value = row ? { provider_id: row.provider_id, name: row.name, is_default: row.is_default } : { provider_id: modelProviderFilter.value || 0, name: '', is_default: false }
  showModelModal.value = true
}
async function saveModel() {
  if (!modelForm.value.provider_id || !modelForm.value.name) { message.warning('请完善必填项'); return }
  saving.value = true
  try {
    if (editingModelId.value) await updateModel(editingModelId.value, modelForm.value)
    else await createModel(modelForm.value)
    showModelModal.value = false
    loadModels()
  } catch { message.error('保存失败') } finally { saving.value = false }
}
async function handleDeleteModel(row: AiModel) {
  if (!confirm('确定删除此模型？')) return
  try { await deleteModel(row.id); loadModels() } catch (e: any) { message.error(e?.response?.data?.error || '删除失败') }
}
async function toggleModelDefault(row: AiModel, v: boolean) {
  try {
    await updateModel(row.id, { is_default: v })
    if (v) models.value.forEach(m => { if (m.provider_id === row.provider_id && m.id !== row.id) m.is_default = false })
    row.is_default = v
    message.success(v ? '已设为默认' : '已取消默认')
  } catch (e: any) { message.error(e?.response?.data?.error || '切换失败') }
}

// ── Providers ──
const providers = ref<AiProvider[]>([])
const showProviderModal = ref(false)
const editingProviderId = ref<number | null>(null)
const providerForm = ref({ name: '', provider_type: 'openai', base_url: '', api_key: '', is_active: true })
const keyCleared = ref(false)
const providerTypeOptions = [
  { label: 'OpenAI 兼容', value: 'openai' },
  { label: 'Anthropic', value: 'anthropic' },
  { label: 'Ollama', value: 'ollama' },
]
const testingId = ref<number | null>(null)
const providerColumns = [
  { title: '名称', key: 'name', width: 130 },
  { title: '类型', key: 'provider_type', width: 120 },
  { title: '状态', key: 'is_active', width: 70, render(row: AiProvider) {
    return h(NSwitch, { size: 'small', value: row.is_active, onUpdateValue: (v: boolean) => toggleProviderActive(row, v) })
  }},
  { title: '操作', key: 'actions', width: 140, render(row: AiProvider) {
    return h(NSpace, { size: 'small' }, { default: () => [
      h(NButton, { size: 'small', loading: testingId === row.id, onClick: () => handleTestProvider(row) }, { default: () => '测试' }),
      h(NButton, { size: 'small', onClick: () => openProviderForm(row) }, { default: () => '编辑' }),
      h(NButton, { size: 'small', type: 'error', onClick: () => handleDeleteProvider(row) }, { default: () => '删除' }),
    ]})
  }},
]

async function loadProviders() { loading.value = true; try { const r = await fetchProviders(); providers.value = r.data.data || [] } catch {} finally { loading.value = false } }
function openProviderForm(row?: AiProvider) {
  editingProviderId.value = row?.id ?? null
  keyCleared.value = false
  if (row) {
    providerForm.value = { name: row.name, provider_type: row.provider_type, base_url: row.base_url, api_key: row.key_preview || '', is_active: row.is_active }
  } else {
    providerForm.value = { name: '', provider_type: 'openai', base_url: '', api_key: '', is_active: true }
  }
  showProviderModal.value = true
}
function onApiKeyFocus() {
  if (editingProviderId.value && !keyCleared.value) {
    providerForm.value.api_key = ''
    keyCleared.value = true
  }
}
async function saveProvider() {
  if (!providerForm.value.name) { message.warning('请输入名称'); return }
  saving.value = true
  try {
    const data: any = { ...providerForm.value }
    if (editingProviderId.value && !data.api_key) delete data.api_key
    if (editingProviderId.value) await updateProvider(editingProviderId.value, data)
    else await createProvider(data)
    showProviderModal.value = false
    loadProviders()
  } catch { message.error('保存失败') } finally { saving.value = false }
}
async function handleDeleteProvider(row: AiProvider) {
  if (!confirm(`确定删除「${row.name}」？`)) return
  try { await deleteProvider(row.id); loadProviders() } catch (e: any) { message.error(e?.response?.data?.error || '删除失败') }
}
async function toggleProviderActive(row: AiProvider, v: boolean) {
  try { await updateProvider(row.id, { is_active: v }); row.is_active = v; message.success(v ? '已启用' : '已停用') }
  catch (e: any) { message.error(e?.response?.data?.error || '切换失败') }
}

async function handleTestProvider(row: AiProvider) {
  testingId.value = row.id
  try {
    const r = await testProvider(row.id)
    const d = r.data.data
    if (d.success) {
      message.success(d.message)
      if (d.models && d.models.length > 0) {
        const names = d.models.slice(0, 10).join(', ')
        message.info(`可用模型：${names}${d.models.length > 10 ? ` ... 等 ${d.models.length} 个` : ''}`, { duration: 8000 })
      }
    } else {
      message.error(d.message, { duration: 8000 })
    }
  } catch { message.error('测试请求失败') }
  finally { testingId.value = null }
}

// ── Skills ──
const skills = ref<AiSkill[]>([])
const showSkillModal = ref(false)
const editingSkillId = ref<number | null>(null)
const skillForm = ref({ name: '', description: '', content: '', output_format: 'markdown' })
const outputFormatOptions = [
  { label: 'Markdown', value: 'markdown' },
  { label: 'JSON', value: 'json' },
  { label: '纯文本', value: 'text' },
  { label: 'HTML', value: 'html' },
]
const toolColumns = [
  { title: '名称', key: 'name', width: 120 },
  { title: '功能名', key: 'function_name', width: 140 },
  { title: '描述', key: 'description', ellipsis: { tooltip: true } },
  { title: '状态', key: 'enabled', width: 70, render(row: AiTool) {
    return h(NSwitch, { size: 'small', value: row.enabled, onUpdateValue: (v: boolean) => toggleToolEnabled(row, v) })
  }},
  { title: '操作', key: 'actions', width: 80, render(row: AiTool) {
    return h(NSpace, { size: 'small' }, { default: () => [
      h(NButton, { size: 'small', onClick: () => openToolForm(row) }, { default: () => '编辑' }),
    ]})
  }},
]
const skillColumns = [
  { title: '名称', key: 'name', width: 150 },
  { title: '描述', key: 'description', ellipsis: { tooltip: true } },
  { title: '操作', key: 'actions', width: 140, render(row: AiSkill) {
    return h(NSpace, { size: 'small' }, { default: () => [
      h(NButton, { size: 'small', onClick: () => openSkillForm(row) }, { default: () => '编辑' }),
      h(NButton, { size: 'small', type: 'error', onClick: () => handleDeleteSkill(row) }, { default: () => '删除' }),
    ]})
  }},
]

async function loadSkills() { loading.value = true; try { const r = await fetchSkills(); skills.value = r.data.data || [] } catch {} finally { loading.value = false } }
function openSkillForm(row?: AiSkill) {
  editingSkillId.value = row?.id ?? null
  skillForm.value = row ? { name: row.name, description: row.description, content: row.content, output_format: row.output_format } : { name: '', description: '', content: '', output_format: 'markdown' }
  showSkillModal.value = true
}
async function saveSkill() {
  if (!skillForm.value.name || !skillForm.value.content) { message.warning('名称和指令内容为必填'); return }
  saving.value = true
  try {
    if (editingSkillId.value) await updateSkill(editingSkillId.value, skillForm.value)
    else await createSkill(skillForm.value)
    showSkillModal.value = false
    loadSkills()
  } catch { message.error('保存失败') } finally { saving.value = false }
}
async function handleDeleteSkill(row: AiSkill) {
  if (!confirm(`确定删除「${row.name}」？`)) return
  try { await deleteSkill(row.id); loadSkills() } catch (e: any) { message.error(e?.response?.data?.error || '删除失败') }
}

// ── Tools ──
const tools = ref<AiTool[]>([])
const showToolModal = ref(false)
const editingToolId = ref<number | null>(null)
const toolForm = ref({ name: '', description: '', function_name: '', parameters_schema: '', enabled: true })

async function loadTools() { loading.value = true; try { const r = await fetchTools(); tools.value = r.data.data || [] } catch {} finally { loading.value = false } }
function openToolForm(row?: AiTool) {
  editingToolId.value = row?.id ?? null
  toolForm.value = row ? { name: row.name, description: row.description, function_name: row.function_name, parameters_schema: row.parameters_schema, enabled: row.enabled } : { name: '', description: '', function_name: '', parameters_schema: '', enabled: true }
  showToolModal.value = true
}
async function saveTool() {
  if (!toolForm.value.name || !toolForm.value.function_name) { message.warning('工具名称和功能名为必填'); return }
  saving.value = true
  try {
    if (editingToolId.value) await updateTool(editingToolId.value, toolForm.value)
    else await createTool(toolForm.value)
    showToolModal.value = false
    loadTools()
  } catch { message.error('保存失败') } finally { saving.value = false }
}
async function handleDeleteTool(row: AiTool) {
  if (!confirm(`确定删除「${row.name}」？`)) return
  try { await deleteTool(row.id); loadTools() } catch (e: any) { message.error(e?.response?.data?.error || '删除失败') }
}
async function toggleToolEnabled(row: AiTool, v: boolean) {
  try { await updateTool(row.id, { enabled: v }); row.enabled = v } catch { message.error('切换失败') }
}

// ── Tasks ──
const tasks = ref<AiTask[]>([])
const showTaskModal = ref(false)
const editingTaskId = ref<number | null>(null)
let skipModelWatch = false
const taskForm = ref({ name: '', skill_id: 0, provider_id: undefined as number | undefined, agent_config_id: undefined as number | undefined, model_id: undefined as number | undefined, cron_expr: '', params: '{}', enabled: true })
const providerOptions = computed(() => providers.value.filter(p => p.is_active).map(p => ({ label: p.name, value: p.id })))
const skillOptions = computed(() => skills.value.map(s => ({ label: s.name, value: s.id })))
const taskColumns = [
  { title: '任务名称', key: 'name', width: 130, render(row: AiTask) { return row.name || '-' } },
  { title: 'Agent', key: 'agent_config_id', width: 100, render(row: AiTask) { return agentConfigs.value.find(a => a.id === row.agent_config_id)?.name || '默认' } },
  { title: '技能', key: 'skill_id', width: 120, render(row: AiTask) { return skills.value.find(s => s.id === row.skill_id)?.name || '-' } },
  { title: '供应商', key: 'provider_id', width: 100, render(row: AiTask) { return providers.value.find(p => p.id === row.provider_id)?.name || 'Agent默认' } },
  { title: '模型', key: 'model_id', width: 120, render(row: AiTask) { return models.value.find(m => m.id === row.model_id)?.name || 'Agent默认' } },
  { title: 'Cron', key: 'cron_expr', width: 130 },
  { title: '状态', key: 'enabled', width: 70, render(row: AiTask) {
    return h(NSwitch, { size: 'small', value: row.enabled, onUpdateValue: (v: boolean) => toggleTaskEnabled(row, v) })
  }},
  { title: '运行次数', key: 'run_count', width: 90 },
  { title: '操作', key: 'actions', width: 140, render(row: AiTask) {
    return h(NSpace, { size: 'small' }, { default: () => [
      h(NButton, { size: 'small', onClick: () => openTaskForm(row) }, { default: () => '编辑' }),
      h(NButton, { size: 'small', type: 'error', onClick: () => handleDeleteTask(row) }, { default: () => '删除' }),
    ]})
  }},
]

async function loadTasks() { loading.value = true; try { const r = await fetchTasks(); tasks.value = r.data.data || [] } catch {} finally { loading.value = false } }
function openTaskForm(row?: AiTask) {
  editingTaskId.value = row?.id ?? null
  skipModelWatch = true
  taskForm.value = row
    ? { name: row.name || '', skill_id: row.skill_id, provider_id: row.provider_id ?? undefined, agent_config_id: row.agent_config_id ?? undefined, model_id: row.model_id ?? undefined, cron_expr: row.cron_expr, params: row.params, enabled: row.enabled }
    : { name: '', skill_id: skillOptions.value[0]?.value ?? 0, provider_id: undefined, agent_config_id: undefined, model_id: undefined, cron_expr: '', params: '{}', enabled: true }
  showTaskModal.value = true
}
async function saveTask() {
  if (!taskForm.value.skill_id || !taskForm.value.agent_config_id || !taskForm.value.cron_expr) { message.warning('请完善必填项（Agent、技能、Cron）'); return }
  saving.value = true
  try {
    const data: any = { ...taskForm.value }
    if (data.agent_config_id === undefined) data.agent_config_id = null
    if (data.provider_id === undefined) data.provider_id = null
    if (data.model_id === undefined) data.model_id = null
    if (editingTaskId.value) await updateTask(editingTaskId.value, data)
    else await createTask(data)
    showTaskModal.value = false
    loadTasks()
  } catch { message.error('保存失败') } finally { saving.value = false }
}
async function handleDeleteTask(row: AiTask) {
  if (!confirm('确定删除此任务？')) return
  try { await deleteTask(row.id); loadTasks() } catch (e: any) { message.error(e?.response?.data?.error || '删除失败') }
}
async function toggleTaskEnabled(row: AiTask, v: boolean) {
  try { await updateTask(row.id, { enabled: v }); row.enabled = v; message.success(v ? '已启用' : '已停用') }
  catch (e: any) { message.error(e?.response?.data?.error || '切换失败') }
}

// 切换供应商时自动选该供应商的首个模型（加载表单时跳过）
watch(() => taskForm.value.provider_id, () => {
  if (skipModelWatch) { skipModelWatch = false; return }
  taskForm.value.model_id = taskModelOptions.value[0]?.value ?? undefined
})

// 切换 Agent 供应商时清空模型选择
watch(agentProviderFilter, () => {
  agentForm.value.model_id = undefined
})

function loadAll() { loadProviders(); loadSkills(); loadTasks(); loadAgentConfigs(); loadTools(); loadModels() }
onMounted(() => loadAll())
</script>

<style scoped>
.ai-manage { padding: 24px; }
.page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
.page-header h2 { margin: 0; font-size: 22px; }
.tab-toolbar { margin-bottom: 12px; display: flex; justify-content: flex-end; }
</style>
