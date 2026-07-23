<template>
  <div>
    <h2 class="font-bold mb-6" style="color: var(--input-color); font-size: 28px">⚙️ 系统设置</h2>

    <n-tabs type="line" animated>
      <n-tab-pane name="settings" tab="站点设置">
        <n-spin :show="loading">
          <n-card class="mb-4">
            <n-form :model="form" label-placement="left" label-width="100">
              <n-form-item label="网站 LOGO">
                <div class="flex items-center gap-3">
                  <div v-if="logoPreview" class="w-10 h-10 rounded-lg overflow-hidden flex-shrink-0" style="background: var(--color-bg)">
                    <img :src="logoPreview" class="w-full h-full object-contain" />
                  </div>
                  <n-button size="small" @click="showImageSelector = true">设置图片</n-button>
                  <n-button v-if="form.site_logo" size="small" secondary @click="form.site_logo = ''">移除</n-button>
                </div>
                <span class="text-xs mt-1" style="color: var(--color-text-muted)">留空则使用站点标题作为 LOGO</span>
              </n-form-item>
              <n-form-item label="站点标题">
                <n-input v-model:value="form.site_title" placeholder="站点标题" />
              </n-form-item>
              <n-form-item label="站点副标题">
                <n-input v-model:value="form.site_subtitle" placeholder="一句话标语，如「记录技术、分享生活」" />
              </n-form-item>
              <n-form-item label="站点描述">
                <n-input v-model:value="form.site_description" type="textarea" placeholder="站点描述" :rows="10" />
              </n-form-item>
              <n-form-item label="友情链接">
                <div class="flex flex-col gap-1 w-full">
                  <n-input v-model:value="form.friend_links" type="textarea" :rows="5" />
                  <span class="text-xs" style="color: var(--color-text-muted)">JSON 数组格式，每项包含 name（站点名）和 url（链接地址）</span>
                </div>
              </n-form-item>
              <n-form-item label="指定站长">
                <n-select
                  v-model:value="form.site_manager"
                  :options="userOptions"
                  placeholder="选择前台展示的站长用户（留空=首个管理员）"
                  clearable
                  filterable
                  style="max-width:320px"
                />
              </n-form-item>
              <n-form-item label="访客留言审核">
                <n-switch v-model:value="form.comment_moderation" />
                <span class="ml-2 text-gray-400 text-sm">{{ form.comment_moderation ? '开启' : '关闭' }}</span>
              </n-form-item>
              <n-form-item label="侧栏分类折叠">
                <n-switch v-model:value="form.sidebar_collapse" />
                <span class="ml-2 text-gray-400 text-sm">{{ form.sidebar_collapse ? '默认折叠' : '默认展开' }}</span>
              </n-form-item>
              <n-form-item label="开启留言板">
                <n-switch v-model:value="form.guestbook_enabled" />
                <span class="ml-2 text-gray-400 text-sm">{{ form.guestbook_enabled ? '开启' : '关闭' }}</span>
              </n-form-item>
              <n-form-item label="访客复制权限">
                <n-switch v-model:value="form.guest_copy_enabled" />
                <span class="ml-2 text-gray-400 text-sm">
                  {{ form.guest_copy_enabled ? '允许复制正文、代码和使用右键菜单' : '仅登录用户可复制文章内容' }}
                </span>
              </n-form-item>
              <n-form-item label="批量装载数量">
                <n-input-number v-model:value="form.batch_load_size" :min="1" :max="20" style="width:120px" />
                <span class="ml-2 text-gray-400 text-sm">首页/分类等首次加载的文章数</span>
              </n-form-item>
              <n-form-item label="滚动装载数量">
                <n-input-number v-model:value="form.scroll_load_size" :min="1" :max="20" style="width:120px" />
                <span class="ml-2 text-gray-400 text-sm">滚动到底时单次追加文章数</span>
              </n-form-item>
              <n-form-item>
                <n-button type="primary" :loading="saving" @click="handleSave">保存设置</n-button>
              </n-form-item>
            </n-form>
          </n-card>
        </n-spin>
      </n-tab-pane>

      <n-tab-pane name="ipaccess" tab="IP访问设置">
        <n-spin :show="loading">
          <n-card class="mb-4">
            <n-form label-placement="left" label-width="120">
              <!-- 白名单 -->
              <n-form-item label="启用白名单">
                <n-switch v-model:value="ipForm.whitelist_enabled" />
                <span class="ml-2 text-sm" style="color: var(--color-text-muted)">
                  {{ ipForm.whitelist_enabled ? '仅白名单IP可通过API Key访问' : '关闭' }}
                </span>
              </n-form-item>
              <n-form-item v-if="ipForm.whitelist_enabled" label="白名单IP">
                <div class="flex flex-col gap-2 w-full">
                  <!-- 添加行 -->
                  <div class="flex gap-2 items-center">
                    <n-input v-model:value="ipForm.whitelist_ip_input" placeholder="IP地址" style="width:180px" />
                    <n-input v-model:value="ipForm.whitelist_remark_input" placeholder="备注" style="width:160px" />
                    <n-button size="small" type="primary" @click="addIp('whitelist')">添加</n-button>
                  </div>
                  <!-- IP 列表 -->
                  <div v-for="(entry, i) in ipForm.whitelist" :key="i"
                       class="flex items-center gap-2 py-1 px-2 rounded" style="background:var(--color-fill-2)">
                    <n-tag size="small" type="success" :bordered="false">{{ entry.ip }}</n-tag>
                    <span class="text-xs" style="color:var(--color-text-muted);max-width:160px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap">{{ entry.remark || '—' }}</span>
                    <span class="cursor-pointer" style="color:#e74c3c;font-size:14px;line-height:1;flex-shrink:0" @click="removeIp('whitelist', i)" title="删除">✕</span>
                  </div>
                  <span v-if="!ipForm.whitelist.length" class="text-xs" style="color: var(--color-text-muted)">白名单为空时拒绝所有API Key请求</span>
                </div>
              </n-form-item>

              <!-- 黑名单 -->
              <n-form-item label="启用黑名单">
                <n-switch v-model:value="ipForm.blacklist_enabled" />
                <span class="ml-2 text-sm" style="color: var(--color-text-muted)">
                  {{ ipForm.blacklist_enabled ? '黑名单IP完全无法访问网站' : '关闭' }}
                </span>
              </n-form-item>
              <n-form-item v-if="ipForm.blacklist_enabled" label="黑名单IP">
                <div class="flex flex-col gap-2 w-full">
                  <!-- 添加行 -->
                  <div class="flex gap-2 items-center">
                    <n-input v-model:value="ipForm.blacklist_ip_input" placeholder="IP地址" style="width:180px" />
                    <n-input v-model:value="ipForm.blacklist_remark_input" placeholder="备注" style="width:160px" />
                    <n-button size="small" type="primary" @click="addIp('blacklist')">添加</n-button>
                  </div>
                  <!-- IP 列表 -->
                  <div v-for="(entry, i) in ipForm.blacklist" :key="i"
                       class="flex items-center gap-2 py-1 px-2 rounded" style="background:var(--color-fill-2)">
                    <n-tag size="small" type="error" :bordered="false">{{ entry.ip }}</n-tag>
                    <span class="text-xs" style="color:var(--color-text-muted);max-width:160px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap">{{ entry.remark || '—' }}</span>
                    <span class="cursor-pointer" style="color:#e74c3c;font-size:14px;line-height:1;flex-shrink:0" @click="removeIp('blacklist', i)" title="删除">✕</span>
                  </div>
                  <span v-if="!ipForm.blacklist.length" class="text-xs" style="color: var(--color-text-muted)">黑名单为空时不影响任何访问</span>
                </div>
              </n-form-item>

              <n-form-item>
                <n-button type="primary" :loading="ipSaving" @click="handleIpSave">保存IP设置</n-button>
              </n-form-item>
            </n-form>
          </n-card>
        </n-spin>
      </n-tab-pane>

      <n-tab-pane name="changelog" tab="版本维护">
        <n-spin :show="clLoading">
          <n-card class="mb-4">
            <n-form label-placement="left" label-width="100">
              <n-form-item label="版本号">
                <n-space align="center">
                  <n-input v-model:value="clForm.version" placeholder="留空保存草稿，填写后前台可见" style="width:240px" />
                  <n-button v-if="editingId > 0" type="primary" :loading="clSaving" @click="handleChangelogSave">保存修改</n-button>
                  <n-button v-else type="primary" :loading="clSaving" @click="handleChangelogSave">保存</n-button>
                  <n-button v-if="editingId > 0" @click="cancelChangelogEdit">取消编辑</n-button>
                </n-space>
              </n-form-item>
              <n-form-item label="更新内容">
                <n-input v-model:value="clForm.content" type="textarea" placeholder="Markdown 格式的更新说明" :rows="5" />
              </n-form-item>
            </n-form>
          </n-card>

          <n-card v-if="changelogs.length > 0">
            <h3 class="font-bold mb-4" style="color: var(--input-color); font-size: 16px">历史版本信息：</h3>
            <div
              v-for="entry in changelogs"
              :key="entry.id"
              class="version-entry"
              :class="entry.version ? 'entry-published' : 'entry-draft'"
            >
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2 mb-1 flex-wrap">
                  <n-tag v-if="entry.version" type="info" size="small" :bordered="false">{{ entry.version }}</n-tag>
                  <n-tag v-if="entry.version" type="success" size="small" :bordered="false">已发布</n-tag>
                  <n-tag v-else type="warning" size="small" :bordered="false">草稿</n-tag>
                  <span class="text-xs" style="color: var(--color-text-muted)">{{ formatDate(entry.created_at) }}</span>
                </div>
                <p class="text-sm leading-relaxed" style="color: var(--color-text); white-space: pre-wrap; word-break: break-word">{{ entry.content }}</p>
              </div>
              <div class="flex gap-1.5 flex-shrink-0 ml-3">
                <n-button size="tiny" quaternary @click="openChangelogEdit(entry)">编辑</n-button>
                <n-popconfirm @positive-click="handleChangelogDelete(entry.id)">
                  <template #trigger>
                    <n-button size="tiny" type="error" quaternary>删除</n-button>
                  </template>
                  确认删除版本 {{ entry.version || '草稿' }}？
                </n-popconfirm>
              </div>
            </div>
          </n-card>
        </n-spin>
      </n-tab-pane>
    </n-tabs>

    <ImageSelector
      :visible="showImageSelector"
      title="设置网站 LOGO"
      @close="showImageSelector = false"
      @select="onLogoSelected"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, computed } from 'vue'
import { useMessage } from 'naive-ui'
import { fetchSettings, updateSettings } from '@/api/settings'
import { useSettingsStore } from '@/stores/settings'
import ImageSelector from '@/components/shared/ImageSelector.vue'
import { fetchChangelogs, createChangelog, updateChangelog, deleteChangelog, type ChangelogEntry } from '@/api/changelog'
import { fetchNetworkResources, fetchUsers } from '@/api/admin'

import { validateIp } from '../../utils/validation'
const message = useMessage()
const settingsStore = useSettingsStore()
const loading = ref(false)
const saving = ref(false)
const showImageSelector = ref(false)

function onLogoSelected(value: string) {
  if (value.startsWith('http') && value.includes('/uploads/')) {
    form.site_logo = value.replace(/https?:\/\/[^/]+\/uploads\//, '/uploads/')
  } else {
    form.site_logo = value
  }
  showImageSelector.value = false
}

// ── Changelog ──
const changelogs = ref<ChangelogEntry[]>([])
const clLoading = ref(false)
const clSaving = ref(false)
const clForm = reactive({ version: '', content: '' })
const editingId = ref(0)

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleDateString('zh-CN', { year: 'numeric', month: 'long', day: 'numeric' })
}

async function loadChangelogs() {
  clLoading.value = true
  try {
    const { data } = await fetchChangelogs()
    changelogs.value = data.data || []
  } finally {
    clLoading.value = false
  }
}

async function handleChangelogSave() {
  if (!clForm.content.trim()) { message.error('请输入更新内容'); return }
  clSaving.value = true
  try {
    if (editingId.value > 0) {
      await updateChangelog(editingId.value, { version: clForm.version, content: clForm.content })
      message.success('已更新')
    } else {
      await createChangelog({ version: clForm.version.trim() || undefined, content: clForm.content })
      message.success(clForm.version.trim() ? '版本已发布' : '草稿已保存')
    }
    clForm.version = ''
    clForm.content = ''
    editingId.value = 0
    await loadChangelogs()
  } catch (e: any) {
    message.error(e.response?.data?.error || (editingId.value > 0 ? '更新失败' : '发布失败'))
  } finally {
    clSaving.value = false
  }
}

function openChangelogEdit(entry: ChangelogEntry) {
  editingId.value = entry.id
  clForm.version = entry.version
  clForm.content = entry.content
}

function cancelChangelogEdit() {
  editingId.value = 0
  clForm.version = ''
  clForm.content = ''
}

async function handleChangelogDelete(id: number) {
  try {
    await deleteChangelog(id)
    message.success('已删除')
    await loadChangelogs()
  } catch (e: any) {
    message.error(e.response?.data?.error || '删除失败')
  }
}

// ── Settings ──
const form = reactive({
  site_logo: '',
  site_title: '',
  site_subtitle: '',
  site_description: '',
  friend_links: '[]',
  comment_moderation: false,
  sidebar_collapse: false,
  guestbook_enabled: true,
  guest_copy_enabled: true,
  batch_load_size: 5,
  scroll_load_size: 3,
  site_manager: null as number | null,
})

// ── IP 访问控制 ──
interface IpEntry { ip: string; remark: string }

const ipForm = reactive({
  whitelist_enabled: false,
  whitelist: [] as IpEntry[],
  whitelist_ip_input: '',
  whitelist_remark_input: '',
  blacklist_enabled: false,
  blacklist: [] as IpEntry[],
  blacklist_ip_input: '',
  blacklist_remark_input: '',
})
const ipSaving = ref(false)

function addIp(type: 'whitelist' | 'blacklist') {
  const list = type === 'whitelist' ? ipForm.whitelist : ipForm.blacklist
  const ipInput = (type === 'whitelist' ? ipForm.whitelist_ip_input : ipForm.blacklist_ip_input) as string
  const remarkInput = (type === 'whitelist' ? ipForm.whitelist_remark_input : ipForm.blacklist_remark_input) as string
  const ip = ipInput.trim()
  if (!ip) return

  // 校验 IP 格式
  const err = validateIp(ip)
  if (err) { message.warning(err); return }

  if (!list.some(e => e.ip === ip)) {
    list.push({ ip, remark: remarkInput.trim() })
  }
  if (type === 'whitelist') {
    ipForm.whitelist_ip_input = ''
    ipForm.whitelist_remark_input = ''
  } else {
    ipForm.blacklist_ip_input = ''
    ipForm.blacklist_remark_input = ''
  }
}

function removeIp(type: 'whitelist' | 'blacklist', index: number) {
  const list = type === 'whitelist' ? ipForm.whitelist : ipForm.blacklist
  list.splice(index, 1)
}

async function handleIpSave() {
  ipSaving.value = true
  try {
    // 保存前校验所有 IP 格式
    for (const entry of ipForm.whitelist) {
      const err = validateIp(entry.ip)
      if (err) { message.warning(`白名单：${err}`); return }
    }
    for (const entry of ipForm.blacklist) {
      const err = validateIp(entry.ip)
      if (err) { message.warning(`黑名单：${err}`); return }
    }

    await updateSettings({
      ip_whitelist_enabled: ipForm.whitelist_enabled ? 'true' : 'false',
      ip_whitelist: JSON.stringify(ipForm.whitelist),
      ip_blacklist_enabled: ipForm.blacklist_enabled ? 'true' : 'false',
      ip_blacklist: JSON.stringify(ipForm.blacklist),
    })
    message.success('IP设置已保存')
  } catch (e: any) {
    message.error(e.response?.data?.error || '保存失败')
  } finally { ipSaving.value = false }
}

const networkUrlCache = reactive(new Map<number, string>())

const logoPreview = computed(() => {
  if (!form.site_logo) return ''
  if (form.site_logo.startsWith('http')) return form.site_logo
  if (form.site_logo.startsWith('nr:')) {
    const id = Number(form.site_logo.slice(3))
    return networkUrlCache.get(id) || ''
  }
  return form.site_logo.startsWith('/') ? form.site_logo : `/${form.site_logo}`
})

// ── Site manager user picker ──
const userOptions = ref<{ label: string; value: number }[]>([])

async function loadUsers() {
  try {
    const { data: resp } = await fetchUsers({ page_size: 500 })
    userOptions.value = (resp.data.data || []).map((u: any) => ({
      label: `${u.display_name || u.username} (${u.role})`,
      value: u.id,
    }))
  } catch { /* ignore */ }
}

async function loadSettings() {
  loading.value = true
  try {
    const { data: resp } = await fetchSettings()
    const s = (resp.data as any).settings
    Object.assign(form, {
      site_logo: s.site_logo || '',
      site_title: s.site_title || '',
      site_subtitle: s.site_subtitle || '',
      site_description: s.site_description || '',
      friend_links: s.friend_links || '[]',
      comment_moderation: s.comment_moderation === 'true',
      sidebar_collapse: s.sidebar_collapse === 'true',
      guestbook_enabled: s.guestbook_enabled !== 'false',
      guest_copy_enabled: s.guest_copy_enabled !== 'false',
      batch_load_size: Number(s.batch_load_size) || 5,
      scroll_load_size: Number(s.scroll_load_size) || 3,
      site_manager: s['site-manager'] ? Number(s['site-manager']) : null,
    })
    // IP 设置
    ipForm.whitelist_enabled = s.ip_whitelist_enabled === 'true'
    ipForm.whitelist = parseIpArray(s.ip_whitelist)
    ipForm.blacklist_enabled = s.ip_blacklist_enabled === 'true'
    ipForm.blacklist = parseIpArray(s.ip_blacklist)
  } finally { loading.value = false }
}

function parseIpArray(raw: string | undefined): IpEntry[] {
  if (!raw) return []
  try {
    const arr = JSON.parse(raw)
    if (!Array.isArray(arr)) return []
    // 兼容旧格式：["ip1", "ip2"] → [{ip, remark:""}]
    return arr.map(item => {
      if (typeof item === 'string') return { ip: item, remark: '' }
      if (typeof item === 'object' && item !== null) {
        return { ip: String(item.ip || ''), remark: String(item.remark || '') }
      }
      return { ip: '', remark: '' }
    }).filter(e => e.ip)
  } catch { return [] }
}

async function handleSave() {
  saving.value = true
  try {
    await updateSettings({
      site_logo: form.site_logo,
      site_title: form.site_title,
      site_subtitle: form.site_subtitle,
      site_description: form.site_description,
      friend_links: form.friend_links,
      comment_moderation: form.comment_moderation ? 'true' : 'false',
      sidebar_collapse: form.sidebar_collapse ? 'true' : 'false',
      guestbook_enabled: form.guestbook_enabled ? 'true' : 'false',
      guest_copy_enabled: form.guest_copy_enabled ? 'true' : 'false',
      batch_load_size: String(form.batch_load_size),
      scroll_load_size: String(form.scroll_load_size),
      'site-manager': form.site_manager ? String(form.site_manager) : '',
    })
    message.success('设置已保存')
    await settingsStore.fetchSettings()
  } catch (e: any) {
    message.error(e.response?.data?.error || '保存失败')
  } finally { saving.value = false }
}

onMounted(() => {
  loadSettings()
  loadUsers()
  loadChangelogs()
  fetchNetworkResources({ page_size: 500, source_type: 'image' }).then(({ data: resp }) => {
    (resp.data || []).forEach((nr: any) => { networkUrlCache.set(nr.id, nr.url) })
  }).catch(() => {})
})
</script>

<style scoped>
.version-entry {
  display: flex; align-items: flex-start; justify-content: space-between;
  padding: 12px 14px; margin-bottom: 6px;
  border-radius: 8px; border-left: 3px solid;
  transition: background 0.15s;
}
.version-entry:last-child { margin-bottom: 0; }
.version-entry:hover { background: rgba(255,255,255,0.03); }
.entry-published { border-left-color: #10b981; }
.entry-draft { border-left-color: #f59e0b; opacity: 0.85; }
.entry-draft:hover { opacity: 1; }
</style>
