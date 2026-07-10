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
            <div class="tab-toolbar" style="justify-content:space-between">
              <n-select v-model:value="modelProviderFilter" :options="providerOptions" placeholder="按供应商筛选" clearable style="width:200px" />
              <n-button type="primary" size="small" @click="openModelForm()" :disabled="!modelProviderFilter">+ 添加模型</n-button>
            </div>
            <n-data-table :columns="modelColumns" :data="filteredModels" :loading="loading" size="small" />
          </div>
        </n-tab-pane>

        <!-- Tools -->
        <n-tab-pane name="tools" tab="工具">
          <div class="tab-toolbar">
            <!-- 「添加工具」暂未实现执行逻辑，隐藏入口 -->
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
    <n-modal v-model:show="showAgentModal" :mask-closable="false" :key="'agent-'+agentModalKey">
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
            <div style="display:flex;gap:8px;width:100%">
              <n-select v-model:value="agentProviderFilter" :options="providerFilterOptions" placeholder="先选供应商" clearable style="flex:1" />
              <n-select v-model:value="agentForm.model_id" :options="agentModelOptions" clearable placeholder="不选则使用默认" style="flex:1" />
            </div>
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
          <n-form-item>
            <template #label>
              <span style="display:inline-flex;align-items:center;gap:10px">
                参数模板
                <n-button size="tiny" @click="extractParamsFromContent">🔍 从指令识别</n-button>
              </span>
            </template>
            <div style="width:100%">
              <n-input v-model:value="skillForm.params_template" type="textarea" :rows="4" placeholder='{"topic":"","count":3}' />
              <div style="color:var(--color-text-muted);font-size:12px;margin-top:4px">
                JSON 格式，可用系统变量：<code>{<!-- -->{date}}</code> <code>{<!-- -->{datetime}}</code> <code>{<!-- -->{time}}</code>。点击识别可从指令内容自动提取 <code>{<!-- -->{变量}}</code>
              </div>
            </div>
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
            <n-input v-model:value="toolForm.parameters_schema" type="textarea" :rows="6"
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
    <n-modal v-model:show="showTaskModal" :mask-closable="false" :key="'task-'+taskModalKey">
      <n-card style="width:520px;max-width:90vw" :title="editingTaskId ? '编辑任务' : '添加任务'">
        <n-form label-placement="left" label-width="80">
          <n-form-item label="任务名称">
            <n-input v-model:value="taskForm.name" placeholder="如：每日AI资讯" />
          </n-form-item>
          <n-form-item label="Agent" required>
            <n-select v-model:value="taskForm.agent_config_id" :options="agentOptions" clearable placeholder="选择智能体" />
          </n-form-item>
          <n-form-item label="技能" required>
            <n-select v-model:value="taskForm.skill_id" :options="skillOptions" @update:value="applySkillParamsTemplate" />
          </n-form-item>
          <n-form-item label="供应商">
            <n-select v-model:value="taskForm.provider_id" :options="providerOptions" clearable placeholder="不选则使用Agent的配置" />
          </n-form-item>
          <n-form-item label="模型">
            <n-select v-model:value="taskForm.model_id" :options="taskModelOptions" clearable placeholder="不选则使用Agent的配置" />
          </n-form-item>
          <n-form-item label="工具轮次">
            <n-input-number v-model:value="taskForm.max_tool_rounds" :min="1" :max="99" placeholder="留空=使用全局默认" style="width:100%" />
          </n-form-item>
          <n-form-item label="Cron 表达式" required>
            <n-input v-model:value="taskForm.cron_expr" placeholder="* 0 8 * * * *" />
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

    <!-- Task Trace Modal -->
    <n-modal v-model:show="showTraceModal" :mask-closable="false" style="width:720px;max-width:95vw">
      <n-card :title="`📋 执行追踪 — ${traceTaskName}`" :bordered="false" size="small">
        <!-- 等待首轮数据 -->
        <div v-if="traceRunning && traceSteps.length === 0" style="text-align:center;padding:60px 0">
          <n-spin size="large" /><p style="margin-top:16px;color:var(--color-text-muted)">正在执行任务，请稍候...</p>
        </div>
        <!-- 执行完成但无数据 -->
        <div v-else-if="!traceRunning && traceSteps.length === 0" style="text-align:center;padding:40px 0;color:var(--color-text-muted)">
          无追踪数据
        </div>
        <!-- 有数据：执行中逐轮显示 / 完成后完整展示 -->
        <div v-else style="max-height:60vh;overflow-y:auto">
          <div v-if="traceRunning" style="display:flex;align-items:center;gap:8px;margin-bottom:16px;color:var(--color-primary)">
            <n-spin size="small" /><span style="font-size:13px">执行中，已获取 {{ traceSteps.length }} 轮数据...</span>
          </div>
          <div v-for="(step, si) in traceSteps" :key="si" style="margin-bottom:20px;border:1px solid var(--color-border);border-radius:8px;padding:12px">
            <div style="font-weight:bold;margin-bottom:8px;color:var(--color-primary)">🔄 第 {{ step.round }} 轮</div>
            <div v-if="step.llm_content" style="background:var(--color-bg-secondary);border-radius:6px;padding:10px;margin-bottom:10px;white-space:pre-wrap;font-size:13px">{{ step.llm_content }}</div>
            <div v-for="(tc, tci) in step.tool_calls" :key="tci" style="margin-bottom:8px">
              <n-collapse>
                <n-collapse-item :title="`🔧 ${tc.function_name}`">
                  <div style="font-size:12px">
                    <div style="margin-bottom:6px"><b>参数：</b><code style="background:var(--color-bg-secondary);padding:2px 6px;border-radius:4px;word-break:break-all">{{ JSON.stringify(tc.arguments, null, 2) }}</code></div>
                    <div><b>结果：</b><pre style="background:var(--color-bg-secondary);padding:8px;border-radius:4px;white-space:pre-wrap;word-break:break-all;max-height:300px;overflow-y:auto;margin:0">{{ tc.result_preview }}</pre></div>
                  </div>
                </n-collapse-item>
              </n-collapse>
            </div>
          </div>
          <div style="margin-top:16px;padding:12px;background:var(--color-card-bg);border-radius:8px;border:1px solid var(--color-success, #67c23a);border-left:4px solid var(--color-success, #67c23a)">
            <div style="font-weight:bold;margin-bottom:6px;color:var(--color-success, #67c23a)">✅ 最终结果</div>
            <div style="white-space:pre-wrap;font-size:14px;line-height:1.7;color:var(--color-text)">{{ traceFinalReply }}</div>
          </div>
        </div>
        <template #footer>
          <n-space justify="end">
            <n-button @click="showTraceModal=false">关闭</n-button>
          </n-space>
        </template>
      </n-card>
    </n-modal>

    <!-- Task Log List Modal -->
    <n-modal v-model:show="showLogModal" :mask-closable="false" style="width:600px;max-width:92vw">
      <n-card :title="`📜 ${logTaskName} — 执行日志`" :bordered="false" size="small">
        <n-spin :show="logLoading">
          <div v-if="logItems.length === 0 && !logLoading" style="text-align:center;padding:40px 0;color:var(--color-text-muted)">暂无执行记录</div>
          <n-data-table v-else :columns="logColumns" :data="logItems" :loading="logLoading" size="small" />
        </n-spin>
        <template #footer>
          <n-space justify="end">
            <n-button @click="showLogModal=false">关闭</n-button>
          </n-space>
        </template>
      </n-card>
    </n-modal>

    <!-- Task Log Detail Modal (复用 trace 展示) -->
    <n-modal v-model:show="showLogDetailModal" :mask-closable="false" style="width:720px;max-width:95vw">
      <n-card :title="`📋 执行详情`" :bordered="false" size="small">
        <div v-if="logDetailSteps.length === 0" style="text-align:center;padding:40px 0;color:var(--color-text-muted)">无追踪数据</div>
        <div v-else style="max-height:60vh;overflow-y:auto">
          <div v-for="(step, i) in logDetailSteps" :key="i" style="margin-bottom:20px;border:1px solid var(--color-border);border-radius:8px;padding:12px">
            <div style="font-weight:bold;margin-bottom:8px;color:var(--color-primary)">🔄 第 {{ step.round }} 轮</div>
            <div v-if="step.llm_content" style="background:var(--color-bg-secondary);border-radius:6px;padding:10px;margin-bottom:10px;white-space:pre-wrap;font-size:13px">{{ step.llm_content }}</div>
            <div v-for="(tc, j) in step.tool_calls" :key="j" style="margin-bottom:8px">
              <n-collapse>
                <n-collapse-item :title="`🔧 ${tc.function_name}`">
                  <div style="font-size:12px">
                    <div style="margin-bottom:6px"><b>参数：</b><code style="background:var(--color-bg-secondary);padding:2px 6px;border-radius:4px;word-break:break-all">{{ JSON.stringify(tc.arguments, null, 2) }}</code></div>
                    <div><b>结果：</b><pre style="background:var(--color-bg-secondary);padding:8px;border-radius:4px;white-space:pre-wrap;word-break:break-all;max-height:300px;overflow-y:auto;margin:0">{{ tc.result_preview }}</pre></div>
                  </div>
                </n-collapse-item>
              </n-collapse>
            </div>
          </div>
          <div v-if="logDetailReply" style="margin-top:16px;padding:12px;background:var(--color-card-bg);border-radius:8px;border:1px solid var(--color-success, #67c23a);border-left:4px solid var(--color-success, #67c23a)">
            <div v-if="logDetailStatus === 'failed'" style="font-weight:bold;margin-bottom:6px;color:#d03050">❌ 执行失败</div>
            <div v-else style="font-weight:bold;margin-bottom:6px;color:var(--color-success, #67c23a)">✅ 最终结果</div>
            <div style="white-space:pre-wrap;font-size:14px;line-height:1.7;color:var(--color-text)">{{ logDetailReply }}</div>
          </div>
          <div v-if="logDetailError" style="margin-top:16px;padding:12px;background:#fff0f0;border-radius:8px;border:1px solid #d03050;border-left:4px solid #d03050">
            <div style="font-weight:bold;margin-bottom:6px;color:#d03050">❌ 错误信息</div>
            <div style="white-space:pre-wrap;font-size:14px">{{ logDetailError }}</div>
          </div>
        </div>
        <template #footer>
          <n-space justify="end">
            <n-button @click="showLogDetailModal=false">关闭</n-button>
          </n-space>
        </template>
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
  fetchTasks, createTask, updateTask, deleteTask, runTask, getTaskTrace, type AiTask, type TaskTraceStep,
  listTaskLogs, getTaskLog, type TaskLogItem, type TaskLogDetail,
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
const agentModalKey = ref(0)
const editingAgentId = ref<number | null>(null)
const agentForm = ref({ name: '', system_prompt: '', user_prompt: '', is_default: false, model_id: undefined as number | undefined })
const agentProviderFilter = ref<number | null>(null)
let skipAgentModelWatch = false
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
  // 根据已有模型自动推导供应商筛选；模型被删除或未选时保留当前筛选（不清空）
  skipAgentModelWatch = true
  if (row?.model_id) {
    const m = models.value.find(m => m.id === row.model_id)
    if (m) agentProviderFilter.value = m.provider_id ?? null
    // 模型已被删除时：不碰 supplier（让用户手动重选）
  }
  // 模型为空时：不碰 supplier（保留用户之前的选择）
  showAgentModal.value = true
  agentModalKey.value++
}
async function saveAgent() {
  if (!agentForm.value.name) { message.warning('名称为必填'); return }
  saving.value = true
  try {
    // undefined → null so JSON.stringify sends the field (backend needs it to clear the value)
    const payload = { ...agentForm.value, model_id: agentForm.value.model_id ?? null }
    if (editingAgentId.value) await updateAgentConfig(editingAgentId.value, payload)
    else await createAgentConfig(payload)
    showAgentModal.value = false
    loadAgentConfigs()
    message.success('Agent 配置已保存')
    window.dispatchEvent(new CustomEvent('marksharex:default-agent-changed'))
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
    window.dispatchEvent(new CustomEvent('marksharex:default-agent-changed'))
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
  { title: '启用', key: 'is_active', width: 70, render(row: AiProvider) {
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
const skillForm = ref({ name: '', description: '', content: '', output_format: 'markdown', params_template: '{}' })
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
  { title: '启用', key: 'enabled', width: 70, render(row: AiTool) {
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
  { title: '参数模板', key: 'params_template', width: 260, ellipsis: { tooltip: true }, render(row: AiSkill) { return row.params_template || '{}' } },
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
  skillForm.value = row ? { name: row.name, description: row.description, content: row.content, output_format: row.output_format, params_template: row.params_template } : { name: '', description: '', content: '', output_format: 'markdown', params_template: '{}' }
  showSkillModal.value = true
}
/** 从指令内容中提取 {{变量}} 占位符，生成参数模板 JSON */
function extractParamsFromContent() {
  const content = skillForm.value.content
  if (!content) { message.warning('请先填写指令内容'); return }
  const matches = content.match(/\{\{(\w+)\}\}/g)
  if (!matches) { message.info('未识别到 {{变量}} 占位符'); return }
  const vars = [...new Set(matches.map(m => m.slice(2, -2)))]
  const template: Record<string, string> = {}
  vars.forEach(v => { template[v] = '' })
  skillForm.value.params_template = JSON.stringify(template, null, 2)
  message.success(`已识别 ${vars.length} 个变量: ${vars.join(', ')}`)
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
const taskModalKey = ref(0)
const editingTaskId = ref<number | null>(null)
let skipModelWatch = false
const taskForm = ref({ name: '', skill_id: 0, provider_id: undefined as number | undefined, agent_config_id: undefined as number | undefined, model_id: undefined as number | undefined, max_tool_rounds: null as number | null, cron_expr: '', params: '{}', enabled: true })
const providerOptions = computed(() => providers.value.filter(p => p.is_active).map(p => ({ label: p.name, value: p.id })))
const skillOptions = computed(() => skills.value.map(s => ({ label: s.name, value: s.id })))
// 日志列表列定义
const logColumns = [
  { title: '时间', key: 'created_at', width: 150, render(row: TaskLogItem) {
    return new Date(row.created_at).toLocaleString('zh-CN', { month:'2-digit', day:'2-digit', hour:'2-digit', minute:'2-digit', second:'2-digit' })
  }},
  { title: '状态', key: 'status', width: 80, render(row: TaskLogItem) {
    return row.status === 'completed' ? '✅ 完成' : row.status === 'failed' ? '❌ 失败' : row.status
  }},
  { title: '轮次', key: 'rounds', width: 60 },
  { title: '结果预览', key: 'final_reply_preview', ellipsis: { tooltip: true }, render(row: TaskLogItem) {
    return row.error || row.final_reply_preview || '-'
  }},
  { title: '操作', key: 'actions', width: 60, render(row: TaskLogItem) {
    return h(NButton, { size: 'tiny', onClick: () => openLogDetail(row.id) }, { default: () => '查看' })
  }},
]

const taskColumns = [
  { title: '名称', key: 'name', width: 100, ellipsis: { tooltip: true }, render(row: AiTask) { return row.name || '-' } },
  { title: 'Agent', key: 'agent_config_id', width: 75, ellipsis: { tooltip: true }, render(row: AiTask) { return agentConfigs.value.find(a => a.id === row.agent_config_id)?.name || '-' } },
  { title: '技能', key: 'skill_id', width: 75, ellipsis: { tooltip: true }, render(row: AiTask) { return skills.value.find(s => s.id === row.skill_id)?.name || '-' } },
  { title: '模型', key: 'model_id', width: 95, ellipsis: { tooltip: true }, render(row: AiTask) { return models.value.find(m => m.id === row.model_id)?.name || '默认' } },
  { title: '轮次', key: 'max_tool_rounds', width: 50, render(row: AiTask) { return row.max_tool_rounds ?? '-' } },
  { title: 'Cron', key: 'cron_expr', width: 100, ellipsis: { tooltip: true } },
  { title: '上次执行', key: 'last_run_at', width: 110, render(row: AiTask) {
    return row.last_run_at ? new Date(row.last_run_at).toLocaleString('zh-CN', { hour: '2-digit', minute: '2-digit', month: '2-digit', day: '2-digit' }) : '-'
  }},
  { title: '次数', key: 'run_count', width: 50, render(row: AiTask) {
    return row.run_count > 0
      ? h('a', { style: { color: 'var(--color-primary)', cursor: 'pointer', textDecoration: 'underline' }, onClick: () => openTaskLogs(row) }, row.run_count)
      : '0'
  }},
  { title: '启用', key: 'enabled', width: 55, render(row: AiTask) {
    return h(NSwitch, { size: 'small', value: row.enabled, onUpdateValue: (v: boolean) => toggleTaskEnabled(row, v) })
  }},
  { title: '操作', key: 'actions', width: 165, render(row: AiTask) {
    return h(NSpace, { size: 'small' }, { default: () => [
      h(NButton, { size: 'tiny', type: 'primary', onClick: () => handleRunTask(row) }, { default: () => '执行' }),
      h(NButton, { size: 'tiny', onClick: () => openTaskForm(row) }, { default: () => '编辑' }),
      h(NButton, { size: 'tiny', type: 'error', onClick: () => handleDeleteTask(row) }, { default: () => '删除' }),
    ]})
  }},
]

async function loadTasks() { loading.value = true; try { const r = await fetchTasks(); tasks.value = r.data.data || [] } catch {} finally { loading.value = false } }
function openTaskForm(row?: AiTask) {
  editingTaskId.value = row?.id ?? null
  skipModelWatch = true
  taskForm.value = row
    ? { name: row.name || '', skill_id: row.skill_id, provider_id: row.provider_id ?? undefined, agent_config_id: row.agent_config_id ?? undefined, model_id: row.model_id ?? undefined, max_tool_rounds: row.max_tool_rounds ?? null, cron_expr: row.cron_expr, params: row.params, enabled: row.enabled }
    : { name: '', skill_id: skillOptions.value[0]?.value ?? 0, provider_id: undefined, agent_config_id: undefined, model_id: undefined, max_tool_rounds: null, cron_expr: '', params: '{}', enabled: true }
  // 新建任务时自动带入技能参数模板
  if (!row) {
    applySkillParamsTemplate()
  }
  showTaskModal.value = true
  taskModalKey.value++
}
async function saveTask() {
  if (!taskForm.value.skill_id || !taskForm.value.agent_config_id || !taskForm.value.cron_expr) { message.warning('请完善必填项（Agent、技能、Cron）'); return }
  saving.value = true
  try {
    const data: any = { ...taskForm.value }
    if (data.agent_config_id === undefined) data.agent_config_id = null
    if (data.provider_id === undefined) data.provider_id = null
    if (data.model_id === undefined) data.model_id = null
    if (data.max_tool_rounds === undefined) data.max_tool_rounds = null
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
let tracePollTimer: ReturnType<typeof setInterval> | null = null

async function handleRunTask(row: AiTask) {
  traceTaskName.value = row.name || `任务 #${row.id}`
  traceSteps.value = []
  traceFinalReply.value = ''
  traceRunning.value = true
  showTraceModal.value = true

  try {
    const r = await runTask(row.id)
    const taskId = r.data.data.task_id
    // 开始轮询
    tracePollTimer = setInterval(async () => {
      try {
        const t = await getTaskTrace(taskId)
        const d = t.data.data
        traceSteps.value = d.steps || []
        if (d.status === 'completed') {
          traceFinalReply.value = d.final_reply || ''
          traceRunning.value = false
          clearPollTimer()
          await loadTasks()
        } else if (d.status === 'failed') {
          traceFinalReply.value = d.error || '执行失败'
          traceRunning.value = false
          clearPollTimer()
        }
      } catch {}
    }, 1500)
  } catch (e: any) {
    traceFinalReply.value = `启动失败: ${e?.response?.data?.error || e.message || '未知错误'}`
    traceRunning.value = false
  }
}

function clearPollTimer() {
  if (tracePollTimer) { clearInterval(tracePollTimer); tracePollTimer = null }
}

async function openTaskLogs(row: AiTask) {
  logTaskId.value = row.id
  logTaskName.value = row.name || `任务 #${row.id}`
  logLoading.value = true
  showLogModal.value = true
  try {
    const r = await listTaskLogs(row.id)
    logItems.value = r.data.data || []
  } catch { message.error('加载日志失败') } finally { logLoading.value = false }
}

async function openLogDetail(logId: number) {
  try {
    const r = await getTaskLog(logTaskId.value, logId)
    const d = r.data.data
    logDetailSteps.value = d.steps || []
    logDetailReply.value = d.final_reply
    logDetailStatus.value = d.status
    logDetailError.value = d.error
    showLogDetailModal.value = true
  } catch { message.error('加载日志详情失败') }
}

// ── Trace state ──
const showTraceModal = ref(false)
const traceRunning = ref(false)
const traceTaskName = ref('')
const traceTaskId = ref(0)

// 任务日志列表
const showLogModal = ref(false)
const logTaskId = ref(0)
const logTaskName = ref('')
const logItems = ref<TaskLogItem[]>([])
const logLoading = ref(false)

// 日志详情（复用 trace 的结构）
const showLogDetailModal = ref(false)
const logDetailSteps = ref<TaskTraceStep[]>([])
const logDetailReply = ref('')
const logDetailStatus = ref('')
const logDetailError = ref<string | null>(null)
const traceSteps = ref<TaskTraceStep[]>([])
const traceFinalReply = ref('')

// 关闭弹窗时停止轮询
watch(showTraceModal, (v) => { if (!v) clearPollTimer() })

// 切换供应商时自动选该供应商的首个模型（加载表单时跳过）
watch(() => taskForm.value.provider_id, () => {
  if (skipModelWatch) { skipModelWatch = false; return }
  taskForm.value.model_id = taskModelOptions.value[0]?.value ?? undefined
})

// 切换技能时自动带入参数模板
function applySkillParamsTemplate() {
  const skill = skills.value.find(s => s.id === taskForm.value.skill_id)
  taskForm.value.params = (skill?.params_template && skill.params_template !== '{}') ? skill.params_template : '{}'
}
watch(() => taskForm.value.skill_id, () => {
  applySkillParamsTemplate()
})

// 切换 Agent 供应商时清空模型选择（加载表单时跳过）
watch(agentProviderFilter, () => {
  if (skipAgentModelWatch) { skipAgentModelWatch = false; return }
  agentForm.value.model_id = undefined
})

function loadAll() { loadProviders(); loadSkills(); loadTasks(); loadAgentConfigs(); loadTools(); loadModels() }

onMounted(() => loadAll())
</script>

<style scoped>
.ai-manage { padding: 0 0 24px; }
.page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px; }
.page-header h2 { margin: 0; font-size: 28px; font-weight: 700; color: var(--input-color); }
.tab-toolbar { margin-bottom: 12px; display: flex; justify-content: flex-end; align-items: center; }
</style>
