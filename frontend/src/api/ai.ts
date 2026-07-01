import api from './index'

// ── Provider ──
export interface AiProvider {
  id: number
  name: string
  provider_type: string
  base_url: string
  is_active: boolean
  created_at: string
  updated_at: string
  key_preview?: string
}

export function fetchProviders() {
  return api.get<{ data: AiProvider[] }>('/admin/ai/providers')
}
export function createProvider(data: { name: string; provider_type?: string; base_url?: string; api_key: string }) {
  return api.post<{ data: AiProvider }>('/admin/ai/providers', data)
}
export function updateProvider(id: number, data: Partial<{ name: string; provider_type: string; base_url: string; api_key: string; default_model: string; is_active: boolean }>) {
  return api.put<{ data: AiProvider }>(`/admin/ai/providers/${id}`, data)
}
export function deleteProvider(id: number) {
  return api.delete(`/admin/ai/providers/${id}`)
}
export function testProvider(id: number) {
  return api.post<{ data: { success: boolean; message: string; models?: string[] } }>(`/admin/ai/providers/${id}/test`)
}

// ── Skill ──
export interface AiSkill {
  id: number
  name: string
  description: string
  content: string
  output_format: string
  created_at: string
  updated_at: string
}

export function fetchSkills() {
  return api.get<{ data: AiSkill[] }>('/admin/ai/skills')
}
export function createSkill(data: { name: string; description?: string; content?: string; output_format?: string }) {
  return api.post<{ data: AiSkill }>('/admin/ai/skills', data)
}
export function updateSkill(id: number, data: Partial<AiSkill>) {
  return api.put<{ data: AiSkill }>(`/admin/ai/skills/${id}`, data)
}
export function deleteSkill(id: number) {
  return api.delete(`/admin/ai/skills/${id}`)
}

// ── Task ──
export interface AiTask {
  id: number
  name: string
  skill_id: number
  provider_id: number
  agent_config_id: number | null
  model_id: number | null
  cron_expr: string
  params: string
  enabled: boolean
  last_run_at: string | null
  run_count: number
  created_at: string
  updated_at: string
}

export function fetchTasks() {
  return api.get<{ data: AiTask[] }>('/admin/ai/tasks')
}
export function createTask(data: { skill_id: number; provider_id: number; cron_expr: string; params?: string; agent_config_id?: number; model_id?: number }) {
  return api.post<{ data: AiTask }>('/admin/ai/tasks', data)
}
export function updateTask(id: number, data: Partial<AiTask>) {
  return api.put<{ data: AiTask }>(`/admin/ai/tasks/${id}`, data)
}
export function deleteTask(id: number) {
  return api.delete(`/admin/ai/tasks/${id}`)
}

// ── Agent Config ──
export interface AgentConfig {
  id: number
  name: string
  system_prompt: string
  user_prompt: string
  is_default: boolean
  model_id: number | null
  created_at: string
  updated_at: string
}

export function fetchAgentConfigs() {
  return api.get<{ data: AgentConfig[] }>('/admin/ai/agent-configs')
}
export function createAgentConfig(data: { name: string; system_prompt?: string; user_prompt?: string }) {
  return api.post<{ data: AgentConfig }>('/admin/ai/agent-configs', data)
}
export function updateAgentConfig(id: number, data: Partial<AgentConfig>) {
  return api.put<{ data: AgentConfig }>(`/admin/ai/agent-configs/${id}`, data)
}
export function deleteAgentConfig(id: number) {
  return api.delete(`/admin/ai/agent-configs/${id}`)
}

// ── Model ──
export interface AiModel {
  id: number
  provider_id: number
  name: string
  is_default: boolean
  created_at: string
  updated_at: string
}

export function fetchModels(providerId?: number) {
  const params = providerId ? `?provider_id=${providerId}` : ''
  return api.get<{ data: AiModel[] }>(`/admin/ai/models${params}`)
}
export function createModel(data: { provider_id: number; name: string }) {
  return api.post<{ data: AiModel }>('/admin/ai/models', data)
}
export function updateModel(id: number, data: Partial<AiModel>) {
  return api.put<{ data: AiModel }>(`/admin/ai/models/${id}`, data)
}
export function deleteModel(id: number) {
  return api.delete(`/admin/ai/models/${id}`)
}

// ── Chat ──
export interface ChatMessage {
  role: string
  content: string
}

export interface ChatRequest {
  message: string
  history: ChatMessage[]
  agent_config_id?: number
  session_id?: number
}

export interface ChatResponse {
  reply: string
  session_id: number
}

export function sendChatMessage(data: ChatRequest) {
  return api.post<{ data: ChatResponse }>('/admin/ai/chat', data, { timeout: 120000 })
}

// ── Session ──
export interface ChatSession {
  id: number
  title: string
  user_id: number
  agent_config_id: number | null
  msg_count: number
  created_at: string
  updated_at: string
}

export interface ChatMessageDetail {
  id: number
  role: string
  content: string
  tool_calls: string | null
  created_at: string
}

export interface ChatSessionDetail {
  id: number
  title: string
  user_id: number
  agent_config_id: number | null
  messages: ChatMessageDetail[]
  created_at: string
  updated_at: string
}

export function fetchSessions() {
  return api.get<{ data: ChatSession[] }>('/admin/ai/sessions')
}
export function getSession(id: number) {
  return api.get<{ data: ChatSessionDetail }>(`/admin/ai/sessions/${id}`)
}
export function deleteSession(id: number) {
  return api.delete(`/admin/ai/sessions/${id}`)
}

// ── Tool ──
export interface AiTool {
  id: number
  name: string
  description: string
  function_name: string
  parameters_schema: string
  enabled: boolean
  created_at: string
  updated_at: string
}

export function fetchTools() {
  return api.get<{ data: AiTool[] }>('/admin/ai/tools')
}
export function createTool(data: { name: string; description?: string; function_name: string; parameters_schema?: string }) {
  return api.post<{ data: AiTool }>('/admin/ai/tools', data)
}
export function updateTool(id: number, data: Partial<AiTool>) {
  return api.put<{ data: AiTool }>(`/admin/ai/tools/${id}`, data)
}
export function deleteTool(id: number) {
  return api.delete(`/admin/ai/tools/${id}`)
}
