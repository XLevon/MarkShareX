<template>
  <div class="users-page">
    <div class="page-header">
      <h1 class="page-title">👥 用户管理</h1>
      <button class="btn-add" @click="openCreateModal">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
        添加用户
      </button>
    </div>

    <!-- 筛选 -->
    <div class="toolbar">
      <div class="filter-tabs">
        <button
          v-for="tab in statusTabs"
          :key="tab.key"
          class="filter-tab"
          :class="{ active: filterStatus === tab.key }"
          @click="filterStatus = tab.key; currentPage = 1; loadUsers()"
        >
          {{ tab.label }}
          <span v-if="tab.key === 'pending_apply' && pendingCount > 0" class="pending-badge">{{ pendingCount }}</span>
        </button>
      </div>
      <div class="search-box">
        <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
        <input v-model="searchQuery" type="text" placeholder="搜索用户名或邮箱..." class="search-input" @keyup.enter="currentPage = 1; loadUsers()" />
      </div>
    </div>

    <!-- 加载 -->
    <div v-if="loading" class="loading-state">
      <div class="spinner"></div>
      <span>加载中...</span>
    </div>

    <!-- 空状态 -->
    <div v-else-if="users.length === 0" class="empty-state">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
      <p>暂无用户</p>
    </div>

    <!-- 用户表格 -->
    <div v-else class="users-table-wrap">
      <table class="users-table">
        <thead>
          <tr>
            <th>用户名</th>
            <th>邮箱</th>
            <th>角色</th>
            <th>状态</th>
            <th v-if="isPendingApply">申请理由</th>
            <th v-if="isPendingApply">分享内容</th>
            <th v-if="isPendingApply">申请时间</th>
            <th>注册时间</th>
            <th>最后登录</th>
            <th>操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="user in users" :key="user.id" :class="{ 'row-banned': user.status === 'banned' }">
            <td class="user-name">
              <span class="user-avatar-sm">{{ (user.display_name || user.username)[0] }}</span>
              <span>{{ user.display_name || user.username }}</span>
            </td>
            <td class="text-muted">{{ user.email }}</td>
            <td>
              <span class="role-badge" :class="user.role">{{ roleLabel(user.role) }}</span>
            </td>
            <td>
              <span class="status-badge" :class="user.status">{{ statusLabel(user.status) }}</span>
            </td>
            <td v-if="isPendingApply" class="text-muted text-sm app-reason-cell" :title="user.application?.reason">{{ user.application?.reason || '—' }}</td>
            <td v-if="isPendingApply" class="text-muted text-sm app-content-cell" :title="user.application?.content_description">{{ user.application?.content_description || '—' }}</td>
            <td v-if="isPendingApply" class="text-muted text-sm">{{ user.application ? formatDate(user.application.created_at) : '—' }}</td>
            <td class="text-muted text-sm">{{ formatDate(user.created_at) }}</td>
            <td class="text-muted text-sm">{{ user.last_login_at ? formatDate(user.last_login_at) : '从未登录' }}</td>
            <td>
              <div class="action-cell">
                <button v-if="isPendingApply && user.application" class="btn-approve-sm" @click="openApproveModal(user)" title="通过申请">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
                </button>
                <button v-if="isPendingApply && user.application" class="btn-reject-sm" @click="openRejectModal(user)" title="拒绝申请">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                </button>
                <button v-if="!isPendingApply && !(isSubAdmin && user.role === 'admin')" class="btn-edit-sm" @click="openEditModal(user)" title="编辑">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
                </button>
                <button
                  v-if="user.role !== 'admin'"
                  class="btn-delete-sm"
                  @click="confirmDelete(user)"
                  title="删除用户"
                >
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/></svg>
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 分页 -->
    <div v-if="totalPages > 1" class="pagination">
      <button :disabled="currentPage <= 1" @click="currentPage--; loadUsers()">上一页</button>
      <span>{{ currentPage }} / {{ totalPages }}</span>
      <button :disabled="currentPage >= totalPages" @click="currentPage++; loadUsers()">下一页</button>
    </div>

    <!-- ==================== 创建用户弹窗 ==================== -->
    <div v-if="showCreate" class="modal-overlay">
      <div class="modal-box modal-wide">
        <h3>添加用户</h3>
        <form @submit.prevent="handleCreate" class="modal-form">
          <div class="form-row">
            <label>用户名 <span class="required">*</span></label>
            <input v-model="createForm.username" type="text" required class="form-input" placeholder="3-32 位字母数字" />
          </div>
          <div class="form-row">
            <label>邮箱 <span class="required">*</span></label>
            <input v-model="createForm.email" type="email" required class="form-input" placeholder="user@example.com" />
          </div>
          <div class="form-row">
            <label>密码 <span class="required">*</span></label>
            <input v-model="createForm.password" type="password" required class="form-input" placeholder="至少 8 位" />
          </div>
          <div class="form-row">
            <label>昵称</label>
            <input v-model="createForm.display_name" type="text" class="form-input" placeholder="显示名称" />
          </div>
          <div class="form-row">
            <label>角色</label>
            <select v-model="createForm.role" class="form-select">
              <option v-if="!isSubAdmin" value="admin">管理员</option>
              <option value="sub_admin">子管理员</option>
              <option value="author">作者</option>
              <option value="visitor">访客</option>
            </select>
          </div>
          <div class="form-row">
            <label>状态</label>
            <select v-model="createForm.status" class="form-select">
              <option value="active">正常</option>
              <option value="muted">禁言</option>
              <option value="banned">拉黑</option>
            </select>
          </div>
          <div v-if="createError" class="form-error">{{ createError }}</div>
          <div class="modal-actions">
            <button type="button" class="btn-secondary" @click="showCreate = false">取消</button>
            <button type="submit" class="btn-primary" :disabled="creating">{{ creating ? '创建中...' : '创建' }}</button>
          </div>
        </form>
      </div>
    </div>

    <!-- ==================== 编辑用户弹窗 ==================== -->
    <div v-if="editUser" class="modal-overlay">
      <div class="modal-box modal-wide">
        <h3>编辑用户 — {{ editUser.display_name || editUser.username }}</h3>
        <form @submit.prevent="handleEdit" class="modal-form">
          <div class="form-row">
            <label>昵称</label>
            <input v-model="editForm.display_name" type="text" class="form-input" placeholder="显示名称" />
          </div>
          <div class="form-row">
            <label>邮箱</label>
            <input v-model="editForm.email" type="email" class="form-input" placeholder="user@example.com" />
          </div>
          <div class="form-row">
            <label>角色</label>
            <select v-model="editForm.role" class="form-select">
              <option v-if="!isSubAdmin" value="admin">管理员</option>
              <option value="sub_admin">子管理员</option>
              <option value="author">作者</option>
              <option value="visitor">访客</option>
            </select>
          </div>
          <div class="form-row">
            <label>状态</label>
            <select v-model="editForm.status" class="form-select">
              <option value="active">正常</option>
              <option value="muted">禁言</option>
              <option value="banned">拉黑</option>
            </select>
          </div>
          <div v-if="editError" class="form-error">{{ editError }}</div>
          <div class="modal-actions">
            <button type="button" class="btn-secondary" @click="editUser = null">取消</button>
            <button type="submit" class="btn-primary" :disabled="saving">{{ saving ? '保存中...' : '保存' }}</button>
          </div>
        </form>
      </div>
    </div>

    <!-- ==================== 删除确认弹窗 ==================== -->
    <div v-if="deleteTarget" class="modal-overlay" @click.self="deleteTarget = null">
      <div class="modal-box">
        <h3>确认删除</h3>
        <p>确定要删除用户「{{ deleteTarget.username }}」吗？此操作不可恢复。</p>
        <div class="modal-actions">
          <button class="btn-secondary" @click="deleteTarget = null">取消</button>
          <button class="btn-danger" @click="handleDelete">删除</button>
        </div>
      </div>
    </div>

    <!-- ==================== 审批申请弹窗 ==================== -->
    <div v-if="approveTarget" class="modal-overlay" @click.self="approveTarget = null">
      <div class="modal-box modal-wide">
        <h3>审批作者申请</h3>
        <div class="app-detail">
          <div class="app-row"><span class="app-label">用户</span><span>{{ approveTarget.display_name || approveTarget.username }}</span></div>
          <div class="app-row"><span class="app-label">邮箱</span><span>{{ approveTarget.email }}</span></div>
          <div class="app-row"><span class="app-label">当前角色</span><span class="role-badge visitor">访客</span></div>
          <div class="app-row"><span class="app-label">申请理由</span></div>
          <div class="app-content">{{ approveTarget.application?.reason }}</div>
          <div class="app-row"><span class="app-label">分享内容说明</span></div>
          <div class="app-content">{{ approveTarget.application?.content_description }}</div>
          <div class="app-row"><span class="app-label">申请时间</span><span>{{ approveTarget.application ? formatDate(approveTarget.application.created_at) : '' }}</span></div>
        </div>
        <div v-if="approveError" class="form-error">{{ approveError }}</div>
        <div class="modal-actions">
          <button type="button" class="btn-secondary" @click="approveTarget = null">取消</button>
          <button type="button" class="btn-reject-lg" @click="openRejectModal(approveTarget)" :disabled="approving">拒绝</button>
          <button type="button" class="btn-primary" @click="handleApprove" :disabled="approving">{{ approving ? '处理中...' : '通过申请' }}</button>
        </div>
      </div>
    </div>

    <!-- ==================== 拒绝申请弹窗 ==================== -->
    <div v-if="rejectTarget" class="modal-overlay" @click.self="rejectTarget = null">
      <div class="modal-box">
        <h3>拒绝申请 — {{ rejectTarget.display_name || rejectTarget.username }}</h3>
        <form @submit.prevent="handleReject" class="modal-form">
          <div class="form-row">
            <label>拒绝原因（选填）</label>
            <textarea v-model="rejectRemark" class="form-textarea" rows="3" placeholder="可选：告知用户拒绝原因"></textarea>
          </div>
          <div v-if="rejectError" class="form-error">{{ rejectError }}</div>
          <div class="modal-actions">
            <button type="button" class="btn-secondary" @click="rejectTarget = null">取消</button>
            <button type="submit" class="btn-danger" :disabled="rejecting">{{ rejecting ? '处理中...' : '确认拒绝' }}</button>
          </div>
        </form>
      </div>
    </div>

    <!-- 角色权限说明 -->
    <div class="role-guide">
      <h3 class="guide-title">🔐 用户角色权限说明</h3>
      <p class="guide-desc">系统支持四种用户角色，不同角色拥有不同的操作权限：</p>
      <div class="role-cards">
        <div class="role-card admin">
          <div class="role-card-header">
            <span class="role-icon">👑</span>
            <span class="role-name">管理员</span>
            <code>admin</code>
          </div>
          <ul>
            <li>完全控制权限，可管理所有内容</li>
            <li>管理用户、修改系统设置</li>
            <li>审核评论、管理分类和标签</li>
            <li>批量导入导出文章</li>
          </ul>
        </div>
        <div class="role-card sub-admin">
          <div class="role-card-header">
            <span class="role-icon">🛡️</span>
            <span class="role-name">子管理员</span>
            <code>sub_admin</code>
          </div>
          <ul>
            <li>可管理文章、分类、标签</li>
            <li>审核评论、管理用户</li>
            <li>批量导入导出文章</li>
            <li>不可修改系统设置</li>
          </ul>
        </div>
        <div class="role-card author">
          <div class="role-card-header">
            <span class="role-icon">✍️</span>
            <span class="role-name">作者</span>
            <code>author</code>
          </div>
          <ul>
            <li>可撰写和管理自己的文章</li>
            <li>评论自动通过，无需审核</li>
            <li>可在文章中使用网络图片</li>
            <li>不可管理其他用户</li>
          </ul>
        </div>
        <div class="role-card visitor">
          <div class="role-card-header">
            <span class="role-icon">👤</span>
            <span class="role-name">访客</span>
            <code>visitor</code>
          </div>
          <ul>
            <li>仅可浏览已发布内容</li>
            <li>可发表评论（可能需审核）</li>
            <li>不可进入管理后台</li>
            <li>不可撰写文章</li>
          </ul>
        </div>
      </div>
    </div>

    <!-- 登录日志 -->
    <div class="login-logs-section">
      <div class="flex items-center justify-between mb-4">
        <h3 class="guide-title">🔑 最近登录记录</h3>
        <div class="flex gap-2">
          <button class="btn-refresh" @click="loadLoginLogs">刷新</button>
          <button class="btn-refresh" @click="openLogsModal">更多</button>
        </div>
      </div>
      <div v-if="loginLogs.length > 0" class="table-scroll">
      <table class="logs-table">
        <thead>
          <tr>
            <th>用户</th>
            <th>IP</th>
            <th>设备</th>
            <th>方式</th>
            <th>结果</th>
            <th>时间</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="log in loginLogs" :key="log.id">
            <td>{{ log.username }}</td>
            <td>{{ log.ip_address || '—' }}</td>
            <td>{{ log.device_type || '—' }}</td>
            <td>{{ log.login_method === 'password' ? '密码' : log.login_method }}</td>
            <td><span class="login-result" :class="log.success ? 'ok' : 'fail'">{{ log.success ? '成功' : '失败' }}</span></td>
            <td>{{ dayjs(log.created_at).format('YYYY-MM-DD HH:mm') }}</td>
          </tr>
        </tbody>
      </table>
      </div>
      <div v-else class="text-dim text-sm">暂无登录记录</div>
    </div>

    <!-- 登录日志详情弹窗 -->
    <div v-if="showLogsModal" class="modal-overlay" @click.self="showLogsModal = false">
      <div class="modal-box modal-logs">
        <h3 class="modal-title">登录记录</h3>
        <!-- 筛选条件 -->
        <div class="logs-filter flex flex-wrap gap-3 mb-4">
          <select v-model="logsFilter.success" class="filter-select">
            <option :value="undefined">全部结果</option>
            <option :value="true">成功</option>
            <option :value="false">失败</option>
          </select>
          <input v-model="logsFilter.userId" type="number" class="filter-input" placeholder="用户 ID（可选）" />
          <button class="btn-refresh" @click="loadAllLogs(1)">查询</button>
          <button class="btn-refresh" @click="showLogsModal = false">关闭</button>
        </div>
        <!-- 表格 -->
        <div v-if="allLoginLogs.length > 0" class="table-scroll">
        <table class="logs-table">
          <thead>
            <tr>
              <th>用户</th>
              <th>IP</th>
              <th>设备</th>
              <th>方式</th>
              <th>结果</th>
              <th>时间</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="log in allLoginLogs" :key="log.id">
              <td>{{ log.username }}</td>
              <td>{{ log.ip_address || '—' }}</td>
              <td>{{ log.device_type || '—' }}</td>
              <td>{{ log.login_method === 'password' ? '密码' : log.login_method }}</td>
              <td><span class="login-result" :class="log.success ? 'ok' : 'fail'">{{ log.success ? '成功' : '失败' }}</span></td>
              <td>{{ dayjs(log.created_at).format('YYYY-MM-DD HH:mm') }}</td>
            </tr>
          </tbody>
        </table>
        </div>
        <div v-else class="text-dim text-sm py-4">暂无匹配记录</div>
        <!-- 分页 -->
        <div v-if="logsTotalPages > 1" class="flex justify-center gap-2 mt-4">
          <button v-for="p in logsTotalPages" :key="p" class="btn-page" :class="{ active: logsPage === p }" @click="loadAllLogs(p)">{{ p }}</button>
        </div>
      </div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { useMessage } from 'naive-ui'
import type { AdminUser } from '@/api/admin'
import { fetchUsers, createUser, updateUser, deleteUser, approveApplication, rejectApplication, getPendingCount, fetchLoginLogs } from '@/api/admin'
import type { LoginLog } from '@/api/admin'
import dayjs from 'dayjs'

// 当前登录用户角色（用于子管理员权限控制）
const currentUserRole = computed(() => {
  try {
    const raw = localStorage.getItem('marksharex_user') || sessionStorage.getItem('marksharex_user')
    if (raw) return JSON.parse(raw).role || 'visitor'
  } catch {}
  return 'visitor'
})
const isSubAdmin = computed(() => currentUserRole.value === 'sub_admin')

const loading = ref(false)
const message = useMessage()
const loginLogs = ref<LoginLog[]>([])
const users = ref<AdminUser[]>([])
const filterStatus = ref('')
const searchQuery = ref('')
const currentPage = ref(1)
const totalPages = ref(1)
const deleteTarget = ref<AdminUser | null>(null)
const pendingCount = ref(0)

// Approval state
const approveTarget = ref<AdminUser | null>(null)
const rejectTarget = ref<AdminUser | null>(null)
const approving = ref(false)
const rejecting = ref(false)
const approveError = ref('')
const rejectError = ref('')
const rejectRemark = ref('')

const isPendingApply = computed(() => filterStatus.value === 'pending_apply')

// Create user
const showCreate = ref(false)
const creating = ref(false)
const createError = ref('')
const createForm = reactive({
  username: '',
  email: '',
  password: '',
  display_name: '',
  role: 'author',
  status: 'active',
})

// Edit user
const editUser = ref<AdminUser | null>(null)
const saving = ref(false)
const editError = ref('')
const editForm = reactive({
  display_name: '',
  email: '',
  role: '',
  status: '',
})

const statusTabs = [
  { label: '全部', key: '' },
  { label: '正常', key: 'active' },
  { label: '禁言', key: 'muted' },
  { label: '拉黑', key: 'banned' },
  { label: '待审申请', key: 'pending_apply' },
]

function statusLabel(s: string) {
  const map: Record<string, string> = { active: '正常', muted: '禁言', banned: '拉黑' }
  return map[s] || s
}

function roleLabel(r: string) {
  const map: Record<string, string> = { admin: '管理员', sub_admin: '子管理员', author: '作者', visitor: '访客' }
  return map[r] || r
}

function formatDate(d: string) {
  return dayjs(d).format('YYYY-MM-DD HH:mm')
}

async function loadUsers() {
  loading.value = true
  try {
    const params: Record<string, any> = { page: currentPage.value, page_size: 20 }
    if (filterStatus.value) params.status = filterStatus.value
    if (searchQuery.value.trim()) params.search = searchQuery.value.trim()
    const { data: resp } = await fetchUsers(params)
    users.value = resp.data.data
    totalPages.value = Math.max(1, resp.data.pagination.pages)
  } catch {
    users.value = []
  } finally {
    loading.value = false
  }
}

async function loadPendingCount() {
  try {
    const { data: resp } = await getPendingCount()
    pendingCount.value = resp.data
  } catch { /* ignore */ }
}

// ── Create ──
function openCreateModal() {
  createForm.username = ''
  createForm.email = ''
  createForm.password = ''
  createForm.display_name = ''
  createForm.role = 'author'
  createForm.status = 'active'
  createError.value = ''
  showCreate.value = true
}

async function handleCreate() {
  creating.value = true
  createError.value = ''
  try {
    await createUser({
      username: createForm.username,
      email: createForm.email,
      password: createForm.password,
      display_name: createForm.display_name || undefined,
      role: createForm.role,
      status: createForm.status,
    })
    showCreate.value = false
    loadUsers()
  } catch (e: any) {
    createError.value = e?.response?.data?.message || '创建失败'
  } finally {
    creating.value = false
  }
}

// ── Edit ──
function openEditModal(user: AdminUser) {
  editForm.display_name = user.display_name || ''
  editForm.email = user.email
  editForm.role = user.role
  editForm.status = user.status
  editError.value = ''
  editUser.value = user
}

async function handleEdit() {
  if (!editUser.value) return
  saving.value = true
  editError.value = ''
  try {
    const payload: Record<string, string> = {}
    if (editForm.display_name !== (editUser.value.display_name || '')) {
      payload.display_name = editForm.display_name
    }
    if (editForm.email !== editUser.value.email) {
      payload.email = editForm.email
    }
    if (editForm.role !== editUser.value.role) {
      payload.role = editForm.role
    }
    if (editForm.status !== editUser.value.status) {
      payload.status = editForm.status
    }
    await updateUser(editUser.value.id, payload)
    editUser.value = null
    loadUsers()
  } catch (e: any) {
    editError.value = e?.response?.data?.message || '保存失败'
  } finally {
    saving.value = false
  }
}

// ── Delete ──
function confirmDelete(user: AdminUser) {
  deleteTarget.value = user
}

async function handleDelete() {
  if (!deleteTarget.value) return
  try {
    await deleteUser(deleteTarget.value.id)
    deleteTarget.value = null
    message.success('用户已删除')
    loadUsers()
  } catch (e: any) {
    message.error(e?.response?.data?.message || '删除失败')
  }
}

// ── Approve / Reject ──

function openApproveModal(user: AdminUser) {
  approveTarget.value = user
  approveError.value = ''
}

function openRejectModal(user: AdminUser) {
  rejectTarget.value = user
  rejectRemark.value = ''
  rejectError.value = ''
}

async function handleApprove() {
  if (!approveTarget.value?.application) return
  approving.value = true
  approveError.value = ''
  try {
    await approveApplication(approveTarget.value.application.id)
    approveTarget.value = null
    loadUsers()
    loadPendingCount()
  } catch (e: any) {
    approveError.value = e?.response?.data?.message || e?.response?.data?.error || '操作失败'
  } finally {
    approving.value = false
  }
}

async function handleReject() {
  if (!rejectTarget.value?.application) return
  rejecting.value = true
  rejectError.value = ''
  try {
    await rejectApplication(rejectTarget.value.application.id, rejectRemark.value || undefined)
    rejectTarget.value = null
    loadUsers()
    loadPendingCount()
  } catch (e: any) {
    rejectError.value = e?.response?.data?.message || e?.response?.data?.error || '操作失败'
  } finally {
    rejecting.value = false
  }
}

async function loadLoginLogs() {
  try {
    const { data: resp } = await fetchLoginLogs({ page_size: 20 })
    loginLogs.value = resp.data.data
  } catch { /* silent */ }
}

// ── 全部登录记录弹窗 ──
const showLogsModal = ref(false)
const allLoginLogs = ref<LoginLog[]>([])
const logsPage = ref(1)
const logsTotalPages = ref(1)
const logsFilter = reactive({
  userId: null as number | null,
  success: undefined as boolean | undefined,
})

function openLogsModal() {
  showLogsModal.value = true
  loadAllLogs(1)
}

async function loadAllLogs(page: number) {
  try {
    const params: any = { page, page_size: 20 }
    if (logsFilter.userId && logsFilter.userId > 0) params.user_id = logsFilter.userId
    if (logsFilter.success !== undefined) params.success = logsFilter.success
    const { data: resp } = await fetchLoginLogs(params)
    allLoginLogs.value = resp.data.data
    logsPage.value = resp.data.pagination?.page || page
    logsTotalPages.value = resp.data.pagination?.pages || 1
  } catch { /* silent */ }
}

onMounted(async () => {
  await loadUsers()
  loadPendingCount()
  loadLoginLogs()
})
</script>

<style scoped>
.users-page {
  animation: fadeIn 0.3s ease;
}
@keyframes fadeIn {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
}

.page-header {
  display: flex; align-items: center; justify-content: space-between;
  margin-bottom: 20px;
}
.page-title { font-size: 28px; font-weight: 700; color: var(--input-color); }

.btn-add {
  display: inline-flex; align-items: center; gap: 6px;
  padding: 9px 18px; border-radius: 10px;  border: 1px solid var(--card-border-color);
  background: #4f46e5; color: #fff; font-size: 13px; font-weight: 500;
  cursor: pointer; transition: background 0.15s;
}
.btn-add:hover { background: #4338ca; }

/* Toolbar */
.toolbar {
  display: flex; align-items: center; justify-content: space-between;
  margin-bottom: 20px; flex-wrap: wrap; gap: 12px;
}
.filter-tabs { display: flex; gap: 4px; background: var(--card-bg); border-radius: 10px; padding: 4px; }
.filter-tab {
  padding: 8px 16px; border-radius: 8px; border: none;
  background: transparent; color: var(--text-secondary); font-size: 13px; font-weight: 500;
  cursor: pointer; transition: all 0.15s;
}
.filter-tab:hover { color: var(--text-primary); }
.filter-tab.active { background: rgba(79, 70, 229, 0.15); color: #818cf8; }
.search-box {
  display: flex; align-items: center; background: var(--card-bg);
  border: 1px solid rgba(255,255,255,0.06); border-radius: 10px; padding: 0 14px;
}
.search-box:focus-within { border-color: rgba(79,70,229,0.4); }
.search-icon { color: var(--text-dim); flex-shrink: 0; }
.search-input {
  background: none; border: none; color: var(--input-color);
  padding: 9px 10px; font-size: 13px; outline: none; width: 200px;
}
.search-input::placeholder { color: #4b5563; }

/* Loading / Empty */
.loading-state { display: flex; flex-direction: column; align-items: center; gap: 12px; padding: 60px 0; color: var(--text-dim); }
.spinner { width: 32px; height: 32px; border: 3px solid rgba(79,70,229,0.15); border-top-color: #4f46e5; border-radius: 50%; animation: spin 0.6s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
.empty-state { display: flex; flex-direction: column; align-items: center; gap: 16px; padding: 80px 0; color: #4b5563; }

/* Table */
.users-table-wrap { background: var(--card-bg); border: 1px solid rgba(255,255,255,0.05); border-radius: 12px; overflow: hidden; }
.users-table { width: 100%; border-collapse: collapse; }
.users-table th {
  text-align: left; font-size: 12px; color: var(--text-dim); font-weight: 500;
  padding: 14px 16px; border-bottom: 1px solid rgba(255,255,255,0.05);
}
.users-table td {
  padding: 14px 16px; border-bottom: 1px solid rgba(255,255,255,0.03);
  font-size: 14px; color: var(--text-primary);
}
.users-table tr:last-child td { border-bottom: none; }
.users-table tr:hover { background: rgba(255,255,255,0.02); }
.row-banned { opacity: 0.5; }
.row-banned:hover { opacity: 0.7; }

.user-name { display: flex; align-items: center; gap: 10px; }
.user-avatar-sm {
  width: 32px; height: 32px; border-radius: 50%; background: #4f46e5;
  color: #fff; display: flex; align-items: center; justify-content: center;
  font-size: 13px; font-weight: 600; flex-shrink: 0;
}
.text-muted { color: var(--text-dim); }
.text-sm { font-size: 13px; }

.role-badge {
  padding: 2px 10px; border-radius: 10px; font-size: 12px; font-weight: 500;
}
.role-badge.admin { background: rgba(79,70,229,0.12); color: #818cf8; }
.role-badge.sub_admin { background: rgba(99,102,241,0.1); color: #a5b4fc; }
.role-badge.author { background: rgba(16,185,129,0.1); color: #34d399; }
.role-badge.visitor { background: rgba(107,114,128,0.1); color: var(--text-secondary); }

.status-badge {
  padding: 2px 10px; border-radius: 10px; font-size: 12px; font-weight: 500;
}
.status-badge.active { background: rgba(16,185,129,0.1); color: #34d399; }
.status-badge.muted { background: rgba(245,158,11,0.1); color: #fbbf24; }
.status-badge.banned { background: rgba(239,68,68,0.1); color: #f87171; }

/* Action cell */
.action-cell { display: flex; align-items: center; gap: 8px; }
.btn-edit-sm {
  width: 30px; height: 30px; border-radius: 8px;  border: 1px solid var(--card-border-color);
  background: rgba(79,70,229,0.1); color: #818cf8;
  display: flex; align-items: center; justify-content: center;
  cursor: pointer; transition: background 0.15s;
}
.btn-edit-sm:hover { background: rgba(79,70,229,0.25); }
.btn-delete-sm {
  width: 30px; height: 30px; border-radius: 8px;  border: 1px solid var(--card-border-color);
  background: rgba(239,68,68,0.08); color: #f87171;
  display: flex; align-items: center; justify-content: center;
  cursor: pointer; transition: background 0.15s;
}
.btn-delete-sm:hover { background: rgba(239,68,68,0.2); }

/* Pagination */
.pagination {
  display: flex; align-items: center; justify-content: center; gap: 16px; margin-top: 24px;
}
.pagination button {
  padding: 8px 16px; border-radius: 8px;
  border: 1px solid rgba(255,255,255,0.08); background: var(--card-bg);
  color: var(--text-primary); font-size: 13px; cursor: pointer; transition: all 0.15s;
}
.pagination button:hover:not(:disabled) { border-color: rgba(79,70,229,0.3); color: var(--input-color); }
.pagination button:disabled { opacity: 0.4; cursor: not-allowed; }
.pagination span { font-size: 13px; color: var(--text-dim); }

/* Modal */
.modal-overlay {
  position: fixed; inset: 0; background: rgba(0,0,0,0.6);
  display: flex; align-items: center; justify-content: center; z-index: 300;
}
.modal-box { background: var(--modal-bg); border: 1px solid rgba(255,255,255,0.08); border-radius: 14px; padding: 28px; max-width: 400px; width: 90%; }
.modal-box h3 { margin: 0 0 20px; font-size: 17px; color: var(--input-color); }
.modal-box p { font-size: 14px; color: var(--text-secondary); line-height: 1.6; margin: 0 0 20px; }
.modal-wide { max-width: 480px; }

.modal-form { display: flex; flex-direction: column; gap: 16px; }
.form-row { display: flex; flex-direction: column; gap: 6px; }
.form-row label { font-size: 13px; color: var(--text-secondary); font-weight: 500; }
.form-row .required { color: #f87171; }
.form-input {
  padding: 9px 14px; border-radius: 8px;
  border: 1px solid rgba(255,255,255,0.08); background: var(--card-bg);
  color: var(--input-color); font-size: 14px; outline: none; transition: border-color 0.15s;
}
.form-input:focus { border-color: rgba(79,70,229,0.4); }
.form-input::placeholder { color: #4b5563; }
.form-select {
  padding: 9px 14px; border-radius: 8px;
  border: 1px solid rgba(255,255,255,0.08); background: var(--card-bg);
  color: var(--input-color); font-size: 14px; outline: none; cursor: pointer;
  appearance: none;
}
.form-select:focus { border-color: rgba(79,70,229,0.4); }
.form-error { padding: 8px 12px; border-radius: 8px; background: rgba(239,68,68,0.1); color: #f87171; font-size: 13px; }

.modal-actions { display: flex; justify-content: flex-end; gap: 10px; margin-top: 4px; }
.btn-secondary { padding: 8px 18px; border-radius: 8px; border: 1px solid rgba(255,255,255,0.1); background: transparent; color: var(--text-primary); font-size: 13px; cursor: pointer; }
.btn-secondary:hover { background: rgba(255,255,255,0.04); }
.btn-primary { padding: 8px 18px; border-radius: 8px;  border: 1px solid var(--card-border-color);background: #4f46e5; color: #fff; font-size: 13px; cursor: pointer; }
.btn-primary:hover { background: #4338ca; }
.btn-primary:disabled { opacity: 0.6; cursor: not-allowed; }
.btn-danger { padding: 8px 18px; border-radius: 8px;  border: 1px solid var(--card-border-color);background: #dc2626; color: #fff; font-size: 13px; cursor: pointer; }
.btn-danger:hover { background: #b91c1c; }

@media (max-width: 768px) {
  .users-table-wrap { overflow-x: auto; }
  .toolbar { flex-direction: column; align-items: stretch; }
  .search-input { width: 100%; }
  .page-header { flex-direction: column; align-items: flex-start; gap: 12px; }
  .role-cards { grid-template-columns: 1fr; }
}

/* ── 角色权限说明 ── */
.role-guide {
  padding: 20px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  margin-top: 24px;
}
.guide-title {
  margin: 0 0 12px 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--input-color);
}
.guide-desc {
  margin: 0 0 16px 0;
  font-size: 14px;
  color: var(--text-secondary);
}
.role-cards {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 12px;
}
.role-card {
  padding: 16px;
  border-radius: 10px;
  border: 1px solid var(--card-border-color);
  background: var(--card-bg);
}
.role-card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
}
.role-icon { font-size: 18px; }
.role-name { font-size: 14px; font-weight: 600; color: var(--input-color); }
.role-card code {
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 4px;
  background: var(--card-bg);
  color: var(--text-secondary);
  margin-left: auto;
  border: 1px solid var(--card-border-color);
}
.role-card ul {
  margin: 0;
  padding-left: 18px;
}
.role-card li {
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.7;
}
.role-card.admin { border-left: 3px solid #818cf8; }
.role-card.sub-admin { border-left: 3px solid #a5b4fc; }
.role-card.author { border-left: 3px solid #34d399; }
.role-card.visitor { border-left: 3px solid #9ca3af; }

/* ── 审批相关 ── */
.app-reason-cell {
  max-width: 180px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.app-content-cell {
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.pending-badge {
  display: inline-flex; align-items: center; justify-content: center;
  min-width: 18px; height: 18px; padding: 0 5px;
  border-radius: 9px; background: #ef4444; color: #fff;
  font-size: 11px; font-weight: 600; margin-left: 4px;
}
.btn-approve-sm {
  width: 30px; height: 30px; border-radius: 8px;  border: 1px solid var(--card-border-color);
  background: rgba(16, 185, 129, 0.1); color: #34d399;
  display: flex; align-items: center; justify-content: center;
  cursor: pointer; transition: background 0.15s;
}
.btn-approve-sm:hover { background: rgba(16, 185, 129, 0.25); }
.btn-reject-sm {
  width: 30px; height: 30px; border-radius: 8px;  border: 1px solid var(--card-border-color);
  background: rgba(239, 68, 68, 0.08); color: #f87171;
  display: flex; align-items: center; justify-content: center;
  cursor: pointer; transition: background 0.15s;
}
.btn-reject-sm:hover { background: rgba(239, 68, 68, 0.2); }
.btn-reject-lg {
  padding: 8px 18px; border-radius: 8px; border: 1px solid rgba(239, 68, 68, 0.3);
  background: transparent; color: #f87171; font-size: 13px; cursor: pointer; transition: background 0.15s;
}
.btn-reject-lg:hover { background: rgba(239, 68, 68, 0.1); }
.btn-reject-lg:disabled { opacity: 0.5; cursor: not-allowed; }

.app-detail { display: flex; flex-direction: column; gap: 12px; margin-bottom: 16px; }
.app-row { display: flex; align-items: flex-start; gap: 10px; }
.app-label { color: var(--text-dim); font-size: 13px; min-width: 72px; flex-shrink: 0; padding-top: 2px; }
.app-row span:last-child { color: var(--text-primary); font-size: 14px; word-break: break-all; }
.app-content {
  background: rgba(255,255,255,0.03); border: 1px solid rgba(255,255,255,0.05);
  border-radius: 8px; padding: 12px; color: var(--text-secondary); font-size: 13px; line-height: 1.6;
  white-space: pre-wrap; word-break: break-word;
}

/* ── Login Logs ── */
.login-logs-section {
  margin-top: 24px;
  padding: 16px;
  background: var(--card-bg);
  border: 1px solid var(--card-border-color);
  border-radius: 12px;
  min-width: 0;
  max-width: 100%;
  overflow: hidden;
  box-sizing: border-box;
}
.table-scroll {
  width: 100%;
  max-width: 100%;
  overflow-x: auto;
  overscroll-behavior-x: contain;
  -webkit-overflow-scrolling: touch;
}
.logs-table {
  width: 100%;
  min-width: 640px;
  border-collapse: collapse;
  font-size: 13px;
}
.logs-table th {
  text-align: left;
  padding: 8px 12px;
  color: var(--text-dim);
  font-weight: 500;
  border-bottom: 1px solid var(--card-border-color);
}
.logs-table td {
  padding: 8px 12px;
  color: var(--text-primary);
  border-bottom: 1px solid rgba(255,255,255,0.03);
}
.login-result.ok { color: #34d399; }
.login-result.fail { color: #f87171; }
.btn-refresh {
  padding: 4px 12px; border-radius: 6px; border: 1px solid var(--card-border-color);
  background: var(--card-bg); color: var(--text-secondary); font-size: 12px; cursor: pointer;
}
.btn-refresh:hover { border-color: var(--color-primary-light); color: var(--input-color); }
.text-dim { color: var(--text-dim); }

/* ── Login Logs Modal ── */
.modal-overlay {
  position: fixed; inset: 0; z-index: 100;
  background: rgba(0,0,0,0.45);
  display: flex; align-items: center; justify-content: center;
}
.modal-box.modal-logs {
  background: var(--card-bg);
  border: 1px solid var(--card-border-color);
  border-radius: 14px;
  padding: 28px;
  width: 90%;
  max-width: 900px;
  max-height: 85vh;
  overflow-y: auto;
}
.modal-title {
  margin: 0 0 16px; font-size: 18px; color: var(--input-color);
}
.filter-select, .filter-input {
  padding: 6px 10px; border-radius: 6px;
  border: 1px solid var(--card-border-color);
  background: var(--input-bg); color: var(--input-color);
  font-size: 13px; outline: none;
}
.filter-select:focus, .filter-input:focus {
  border-color: var(--color-primary-light);
}
.btn-page {
  padding: 4px 10px; border-radius: 6px;
  border: 1px solid var(--card-border-color);
  background: var(--card-bg); color: var(--text-secondary);
  font-size: 12px; cursor: pointer;
}
.btn-page.active {
  background: #4f46e5; color: #fff; border-color: #4f46e5;
}
.btn-page:hover:not(.active) { border-color: var(--color-primary-light); }
</style>
