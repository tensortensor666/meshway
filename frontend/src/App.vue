<script setup lang="ts">
import axios from 'axios'
import {
  Activity, ArrowUpRight, Boxes, CheckCircle2, Copy, KeyRound,
  LayoutDashboard, Plus, RefreshCw, ScrollText, Server, ShieldCheck,
  Sparkles, TerminalSquare, X
} from 'lucide-vue-next'
import { computed, onMounted, ref } from 'vue'
import Badge from './components/ui/Badge.vue'
import Button from './components/ui/Button.vue'
import Card from './components/ui/Card.vue'
import Input from './components/ui/Input.vue'

type Provider = { id: string; name: string; type: string; base_url: string; enabled: number }
type ApiKey = { id: string; name: string; enabled: number; created_at: string; last_used?: string }
type Log = { id: string; provider: string; model?: string; path: string; status: number; latency_ms: number; created_at: string }
type NewApiCapability = { name: string; path: string; status?: number; supported: boolean; detail: string }
type NewApiModel = { id: string; owned_by?: string }
type NewApiInspection = { provider_type: string; suggested_name: string; base_url: string; key_masked: string; models: NewApiModel[]; model_count: number; models_status: number; capabilities: NewApiCapability[] }

const token = ref(localStorage.getItem('meshway_admin_token') ?? '')
const active = ref('概览')
const loading = ref(false)
const notice = ref<{ text: string; error: boolean } | null>(null)
const stats = ref({ total_requests: 0, successful_requests: 0, success_rate: 0, average_latency_ms: 0 })
const providers = ref<Provider[]>([])
const keys = ref<ApiKey[]>([])
const logs = ref<Log[]>([])
const newKeyName = ref('')
const createdKey = ref('')
const providerForm = ref({ name: '', type: 'openai', base_url: 'https://api.openai.com/v1', api_key: '' })
const newApiSource = ref('')
const newApiKey = ref('')
const newApiName = ref('')
const newApiLoading = ref(false)
const newApiInspection = ref<NewApiInspection | null>(null)

const navItems = [
  { label: '概览', caption: 'Overview', icon: LayoutDashboard },
  { label: 'Providers', caption: 'Upstreams', icon: Boxes },
  { label: 'API Keys', caption: 'Access', icon: KeyRound },
  { label: '请求日志', caption: 'Audit trail', icon: ScrollText }
]
const api = computed(() => axios.create({ headers: { Authorization: `Bearer ${token.value}` } }))
const onlineProviders = computed(() => providers.value.filter((provider) => provider.enabled).length)

function notify(text: string, error = false) { notice.value = { text, error } }
function formatDate(value?: string) { return value ? new Date(value).toLocaleString() : '-' }
function requestError(error: unknown) {
  if (axios.isAxiosError(error)) return error.response?.data?.error?.message || `请求失败（${error.response?.status || '网络错误'}）`
  return '操作失败，请稍后重试'
}

function keyFromNewApiSource(source: string) {
  try {
    const value = JSON.parse(source) as { key?: unknown }
    return typeof value.key === 'string' ? value.key.trim() : ''
  } catch {
    return source.match(/["']key["']\s*:\s*["']([^"']+)["']/)?.[1]?.trim() || ''
  }
}

async function inspectNewApi() {
  newApiLoading.value = true
  newApiInspection.value = null
  const sourceKey = keyFromNewApiSource(newApiSource.value)
  if (!newApiKey.value.trim() && sourceKey) newApiKey.value = sourceKey
  try {
    const result = await api.value.post('/api/admin/providers/newapi/test', {
      source: newApiSource.value,
      key: newApiKey.value.trim() || undefined
    })
    newApiInspection.value = result.data
    newApiName.value = result.data.suggested_name
    notify('NewAPI 探测完成')
  } catch (error) { notify(requestError(error), true) } finally { newApiLoading.value = false }
}

async function addNewApiProvider(type = 'openai') {
  if (!newApiInspection.value || !newApiKey.value.trim()) {
    notify('请先完成探测，并确认 NewAPI Key 已填写', true)
    return
  }
  try {
    await api.value.post('/api/admin/providers', {
      name: newApiName.value.trim() || newApiInspection.value.suggested_name,
      type,
      base_url: newApiInspection.value.base_url,
      api_key: newApiKey.value.trim(),
      enabled: true
    })
    newApiSource.value = ''
    newApiKey.value = ''
    newApiName.value = ''
    newApiInspection.value = null
    notify(`${type === 'anthropic' ? 'Anthropic' : 'OpenAI'} Provider 已添加`)
    await refresh(true)
  } catch (error) { notify(requestError(error), true) }
}

async function refresh(silent = false) {
  loading.value = true
  localStorage.setItem('meshway_admin_token', token.value)
  try {
    const [s, p, k, l] = await Promise.all([
      api.value.get('/api/admin/stats'), api.value.get('/api/admin/providers'),
      api.value.get('/api/admin/api-keys'), api.value.get('/api/admin/logs')
    ])
    stats.value = s.data; providers.value = p.data; keys.value = k.data; logs.value = l.data
    if (!silent) notify('数据已刷新')
  } catch (error) { notify(requestError(error), true) } finally { loading.value = false }
}

async function saveProvider() {
  try {
    await api.value.post('/api/admin/providers', providerForm.value)
    providerForm.value.api_key = ''
    notify('Provider 已保存')
    await refresh(true)
  } catch (error) { notify(requestError(error), true) }
}

async function createKey() {
  if (!newKeyName.value.trim()) return
  try {
    const result = await api.value.post('/api/admin/api-keys', { name: newKeyName.value })
    createdKey.value = result.data.key
    newKeyName.value = ''
    notify('Key 已创建，请立即复制保存')
    await refresh(true)
  } catch (error) { notify(requestError(error), true) }
}

async function toggleKey(key: ApiKey) {
  try {
    await api.value.put(`/api/admin/api-keys/${key.id}`, { enabled: !key.enabled })
    notify(key.enabled ? 'Key 已停用' : 'Key 已启用')
    await refresh(true)
  } catch (error) { notify(requestError(error), true) }
}

async function copyKey() {
  await navigator.clipboard.writeText(createdKey.value)
  notify('Key 已复制到剪贴板')
}

onMounted(() => refresh(true))
</script>

<template>
  <div class="min-h-screen bg-background text-foreground lg:flex">
    <aside class="border-b bg-slate-950 text-slate-200 lg:fixed lg:inset-y-0 lg:w-64 lg:border-b-0 lg:border-r lg:border-slate-800">
      <div class="flex h-20 items-center gap-3 px-5 lg:h-24">
        <span class="flex h-10 w-10 items-center justify-center rounded-lg bg-teal-500 text-slate-950 shadow-lg shadow-teal-950/40"><Sparkles :size="20" /></span>
        <span><strong class="block text-base text-white">Meshway</strong><small class="block text-[10px] font-semibold tracking-[.18em] text-slate-500">LOCAL AI GATEWAY</small></span>
      </div>
      <div class="mx-4 hidden items-center gap-3 rounded-lg border border-slate-800 bg-slate-900 px-3 py-3 lg:flex"><TerminalSquare :size="17" class="text-teal-400" /><span class="min-w-0 flex-1"><small class="block text-[9px] font-bold tracking-[.15em] text-slate-500">WORKSPACE</small><strong class="block truncate text-xs text-slate-200">Local instance</strong></span><ArrowUpRight :size="14" class="text-slate-500" /></div>
      <nav class="subtle-scrollbar flex gap-1 overflow-x-auto px-3 pb-3 lg:mt-7 lg:block lg:space-y-1 lg:overflow-visible lg:px-4"><p class="mb-2 hidden px-3 text-[9px] font-bold tracking-[.17em] text-slate-600 lg:block">CONTROL CENTER</p><Button v-for="item in navItems" :key="item.label" variant="ghost" :class="['h-11 shrink-0 justify-start px-3 text-slate-400 hover:bg-slate-900 hover:text-white lg:w-full', active === item.label && 'bg-slate-800 text-white hover:bg-slate-800']" @click="active = item.label"><component :is="item.icon" :size="17" /><span class="text-left"><strong class="block text-xs font-medium">{{ item.label }}</strong><small class="hidden text-[10px] font-normal text-slate-500 lg:block">{{ item.caption }}</small></span></Button></nav>
      <div class="absolute bottom-0 hidden w-full border-t border-slate-800 p-5 lg:block"><div class="flex items-center gap-2 text-xs font-medium text-slate-300"><span class="h-2 w-2 rounded-full bg-emerald-400 shadow-[0_0_0_4px_rgba(52,211,153,.12)]" /> Gateway online</div><div class="mt-2 text-[10px] text-slate-600">v0.1.2 · Self-hosted</div></div>
    </aside>

    <main class="min-w-0 flex-1 lg:pl-64">
      <header class="sticky top-0 z-20 flex h-20 items-center justify-between border-b bg-white/90 px-5 backdrop-blur-md sm:px-8 lg:h-24 lg:px-10"><div><p class="text-[10px] font-bold tracking-[.16em] text-muted-foreground">MESHWAY <span class="mx-1 text-slate-300">/</span> CONTROL PLANE</p><h1 class="mt-1 text-2xl font-semibold tracking-tight">{{ active }}</h1></div><div class="flex items-center gap-2"><Badge variant="outline" class="hidden gap-2 py-1.5 font-mono font-normal sm:flex"><span class="h-1.5 w-1.5 rounded-full bg-emerald-500" />127.0.0.1:8080</Badge><Button variant="outline" size="icon" title="刷新数据" :disabled="loading" @click="refresh()"><RefreshCw :size="16" :class="loading && 'animate-spin'" /></Button><div class="ml-1 flex h-9 w-9 items-center justify-center rounded-full bg-slate-900 text-[11px] font-bold text-white">MW</div></div></header>

      <div class="meshway-grid min-h-[calc(100vh-6rem)] p-5 sm:p-8 lg:p-10">
        <div class="mx-auto max-w-[1480px]">
          <div v-if="notice" :class="['mb-5 flex items-center gap-2 rounded-lg border px-4 py-3 text-sm shadow-sm', notice.error ? 'border-red-200 bg-red-50 text-red-700' : 'border-emerald-200 bg-emerald-50 text-emerald-700']"><X v-if="notice.error" :size="16" /><CheckCircle2 v-else :size="16" /><span class="flex-1">{{ notice.text }}</span><button title="关闭" @click="notice = null"><X :size="14" /></button></div>

          <Card class="mb-7 flex flex-col gap-4 border-slate-200/80 bg-white/95 p-4 sm:flex-row sm:items-center"><span class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-teal-50 text-teal-700"><ShieldCheck :size="19" /></span><div class="min-w-0 flex-1"><strong class="block text-sm">管理控制台认证</strong><span class="text-xs text-muted-foreground">输入 MESHWAY_ADMIN_TOKEN 连接当前实例</span></div><Input v-model="token" type="password" placeholder="Admin Token" class="sm:w-72" @keyup.enter="refresh()" /><Button @click="refresh()">连接实例</Button></Card>

          <section v-if="active === '概览'" class="space-y-7">
            <div class="flex flex-col justify-between gap-3 sm:flex-row sm:items-end"><div><p class="text-[10px] font-bold tracking-[.16em] text-teal-700">SYSTEM SNAPSHOT</p><h2 class="mt-1 text-xl font-semibold">运行概览</h2><p class="mt-1 text-sm text-muted-foreground">统一管理模型供应商、访问凭证和请求流量。</p></div><Badge variant="success" class="w-fit gap-1.5 py-1.5"><Activity :size="13" />实时监控中</Badge></div>
            <div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-4"><Card class="relative overflow-hidden p-5"><span class="absolute inset-x-0 top-0 h-1 bg-teal-500" /><div class="flex items-center justify-between text-sm text-muted-foreground"><span>请求总量</span><Activity :size="17" /></div><strong class="mt-5 block text-3xl font-semibold">{{ stats.total_requests }}</strong><small class="mt-1 block text-xs text-muted-foreground">全部 API 请求</small></Card><Card class="p-5"><div class="flex items-center justify-between text-sm text-muted-foreground"><span>成功率</span><CheckCircle2 :size="17" /></div><strong class="mt-5 block text-3xl font-semibold">{{ (stats.success_rate * 100).toFixed(1) }}<small class="ml-1 text-sm font-medium text-muted-foreground">%</small></strong><small class="mt-1 block text-xs text-muted-foreground">{{ stats.successful_requests }} 次成功响应</small></Card><Card class="p-5"><div class="flex items-center justify-between text-sm text-muted-foreground"><span>平均延迟</span><Server :size="17" /></div><strong class="mt-5 block text-3xl font-semibold">{{ Math.round(stats.average_latency_ms) }}<small class="ml-1 text-sm font-medium text-muted-foreground">ms</small></strong><small class="mt-1 block text-xs text-muted-foreground">端到端平均耗时</small></Card><Card class="p-5"><div class="flex items-center justify-between text-sm text-muted-foreground"><span>在线 Provider</span><Boxes :size="17" /></div><strong class="mt-5 block text-3xl font-semibold">{{ onlineProviders }}<small class="ml-1 text-sm font-medium text-muted-foreground">/ {{ providers.length }}</small></strong><small class="mt-1 block text-xs text-muted-foreground">当前可用上游</small></Card></div>
            <div class="grid gap-5 xl:grid-cols-[minmax(0,1fr)_320px]"><Card class="overflow-hidden"><div class="flex items-center justify-between border-b px-5 py-4"><div><p class="text-[9px] font-bold tracking-[.16em] text-muted-foreground">LATEST ACTIVITY</p><h3 class="mt-1 text-sm font-semibold">最近请求</h3></div><Button variant="ghost" size="sm" @click="active = '请求日志'">查看全部<ArrowUpRight :size="14" /></Button></div><div class="subtle-scrollbar overflow-x-auto"><table class="data-table w-full min-w-[720px]"><thead><tr><th>接口</th><th>Provider</th><th>模型</th><th>状态</th><th>延迟</th><th>时间</th></tr></thead><tbody><tr v-for="log in logs.slice(0, 6)" :key="log.id"><td class="font-mono text-xs text-teal-700">{{ log.path }}</td><td>{{ log.provider }}</td><td>{{ log.model || '-' }}</td><td><Badge :variant="log.status < 300 ? 'success' : 'destructive'">{{ log.status < 300 ? '成功' : '失败' }}</Badge></td><td>{{ log.latency_ms }} ms</td><td>{{ formatDate(log.created_at) }}</td></tr><tr v-if="!logs.length"><td colspan="6" class="h-36 text-center text-muted-foreground">还没有请求记录</td></tr></tbody></table></div></Card><Card class="p-5"><p class="text-[9px] font-bold tracking-[.16em] text-muted-foreground">INSTANCE</p><h3 class="mt-1 text-sm font-semibold">连接状态</h3><div class="mt-5 flex items-start gap-3 rounded-lg bg-emerald-50 p-4"><CheckCircle2 :size="19" class="mt-0.5 text-emerald-600" /><div><strong class="block text-sm text-emerald-900">Gateway 已就绪</strong><span class="mt-1 block text-xs text-emerald-700">监听 127.0.0.1:8080</span></div></div><dl class="mt-5 space-y-4 text-xs"><div class="flex justify-between"><dt class="text-muted-foreground">认证方式</dt><dd class="font-medium">Bearer Token</dd></div><div class="flex justify-between"><dt class="text-muted-foreground">数据库</dt><dd class="font-medium">SQLite</dd></div><div class="flex justify-between"><dt class="text-muted-foreground">管理后台</dt><dd class="font-medium text-emerald-600">已启用</dd></div></dl></Card></div>
          </section>

          <section v-else-if="active === 'Providers'" class="space-y-6"><div><p class="text-[10px] font-bold tracking-[.16em] text-teal-700">UPSTREAMS</p><h2 class="mt-1 text-xl font-semibold">Provider 管理</h2><p class="mt-1 text-sm text-muted-foreground">配置模型供应商的 API 根地址和访问密钥。</p></div><Card class="overflow-hidden border-teal-200/80"><div class="flex flex-col gap-4 border-b bg-teal-50/60 p-5 sm:flex-row sm:items-start sm:justify-between"><div class="flex gap-3"><span class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-teal-600 text-white"><Sparkles :size="18" /></span><div><p class="text-[9px] font-bold tracking-[.16em] text-teal-700">NEWAPI IMPORT</p><h3 class="mt-1 text-sm font-semibold">一键导入 NewAPI</h3><p class="mt-1 max-w-2xl text-xs text-muted-foreground">粘贴 NewAPI 渠道连接 JSON 或接口地址，Meshway 会读取模型列表并探测兼容接口。</p></div></div><Badge variant="outline" class="w-fit border-teal-200 bg-white text-teal-700">只读探测</Badge></div><div class="grid gap-5 p-5 xl:grid-cols-[minmax(0,1fr)_360px]"><div class="space-y-4"><label class="block text-xs font-medium">NewAPI 配置 JSON 或 URL<textarea v-model="newApiSource" rows="5" placeholder="粘贴 { &quot;_type&quot;: &quot;newapi_channel_conn&quot;, &quot;key&quot;: &quot;sk-...&quot;, &quot;url&quot;: &quot;https://example.com&quot; }" class="mt-2 flex w-full resize-y rounded-md border border-input bg-background px-3 py-2 font-mono text-xs shadow-sm outline-none transition-colors placeholder:text-muted-foreground focus:ring-2 focus:ring-ring" /></label><label class="block text-xs font-medium">API Key <span class="font-normal text-muted-foreground">（JSON 已包含时可留空）</span><Input v-model="newApiKey" type="password" placeholder="sk-..." class="mt-2" /></label><Button :disabled="newApiLoading || !newApiSource.trim()" @click="inspectNewApi"><RefreshCw :size="16" :class="newApiLoading && 'animate-spin'" />{{ newApiLoading ? '探测中...' : '探测接口和模型' }}</Button></div><div v-if="newApiInspection" class="rounded-lg border bg-background p-4"><div class="flex items-start justify-between gap-3"><div><p class="text-[9px] font-bold tracking-[.16em] text-muted-foreground">DETECTED ENDPOINT</p><strong class="mt-1 block text-sm">{{ newApiInspection.suggested_name }}</strong><code class="mt-1 block truncate text-[11px] text-muted-foreground">{{ newApiInspection.base_url }}</code></div><Badge variant="success">{{ newApiInspection.model_count }} 个模型</Badge></div><div class="mt-4 space-y-2"><div v-for="capability in newApiInspection.capabilities" :key="capability.path" class="flex items-center gap-2 text-xs"><span :class="['h-2 w-2 rounded-full', capability.supported ? 'bg-emerald-500' : 'bg-slate-300']" /><span class="min-w-0 flex-1 truncate">{{ capability.name }}</span><span class="text-muted-foreground">{{ capability.status || '网络错误' }}</span></div></div><div class="mt-4 rounded-md bg-slate-50 p-3 text-xs"><span class="text-muted-foreground">Key：</span><code>{{ newApiInspection.key_masked }}</code><div class="mt-1 text-muted-foreground">模型接口 HTTP {{ newApiInspection.models_status || '网络错误' }}</div></div><div v-if="newApiInspection.models.length" class="mt-4 flex flex-wrap gap-1.5"><Badge v-for="model in newApiInspection.models.slice(0, 8)" :key="model.id" variant="outline" class="max-w-full truncate font-mono text-[10px]">{{ model.id }}</Badge><span v-if="newApiInspection.models.length > 8" class="text-[10px] text-muted-foreground">+{{ newApiInspection.models.length - 8 }} 个</span></div><label class="mt-4 block text-xs font-medium">保存名称<Input v-model="newApiName" class="mt-2" /></label><div class="mt-4 flex flex-wrap gap-2"><Button size="sm" :disabled="!newApiInspection.capabilities.some((item) => item.supported)" @click="addNewApiProvider('openai')"><Plus :size="14" />添加 OpenAI</Button><Button v-if="newApiInspection.capabilities.find((item) => item.path === '/messages')?.supported" size="sm" variant="outline" @click="addNewApiProvider('anthropic')"><Plus :size="14" />添加 Anthropic</Button></div></div><div v-else class="flex min-h-48 items-center justify-center rounded-lg border border-dashed bg-slate-50 p-6 text-center text-xs text-muted-foreground">探测结果会显示在这里<br />不会发送聊天请求</div></div></Card><div class="grid gap-5 xl:grid-cols-[minmax(0,1fr)_380px]"><Card class="overflow-hidden"><div class="flex items-center justify-between border-b px-5 py-4"><h3 class="text-sm font-semibold">已配置 Provider <Badge variant="outline" class="ml-2">{{ providers.length }}</Badge></h3><span class="text-xs text-muted-foreground">按创建时间排序</span></div><div class="divide-y"><article v-for="provider in providers" :key="provider.id" class="flex items-center gap-4 p-5"><span class="flex h-10 w-10 items-center justify-center rounded-lg bg-slate-900 text-sm font-bold text-white">{{ provider.type === 'anthropic' ? 'A' : 'O' }}</span><div class="min-w-0 flex-1"><strong class="block text-sm">{{ provider.name }}</strong><span class="mt-1 block truncate text-xs text-muted-foreground">{{ provider.type }} · {{ provider.base_url }}</span></div><Badge :variant="provider.enabled ? 'success' : 'outline'">{{ provider.enabled ? '运行中' : '已停用' }}</Badge></article><div v-if="!providers.length" class="flex h-56 flex-col items-center justify-center text-center text-muted-foreground"><Boxes :size="28" /><strong class="mt-3 text-sm text-foreground">还没有 Provider</strong><span class="mt-1 text-xs">添加上游服务后即可转发请求</span></div></div></Card><Card class="p-5"><div class="mb-5 flex items-center justify-between"><div><p class="text-[9px] font-bold tracking-[.16em] text-muted-foreground">NEW UPSTREAM</p><h3 class="mt-1 text-sm font-semibold">添加 Provider</h3></div><Plus :size="17" class="text-muted-foreground" /></div><form class="space-y-4" @submit.prevent="saveProvider"><label class="block text-xs font-medium">显示名称<Input v-model="providerForm.name" required placeholder="例如 OpenAI Production" class="mt-2" /></label><label class="block text-xs font-medium">Provider 类型<select v-model="providerForm.type" class="mt-2 flex h-10 w-full rounded-md border border-input bg-background px-3 text-sm shadow-sm focus:outline-none focus:ring-2 focus:ring-ring"><option value="openai">OpenAI 兼容</option><option value="anthropic">Anthropic</option></select></label><label class="block text-xs font-medium">Base URL<Input v-model="providerForm.base_url" required class="mt-2" /></label><label class="block text-xs font-medium">API Key<Input v-model="providerForm.api_key" type="password" required placeholder="sk-..." class="mt-2" /></label><Button type="submit" class="w-full"><Plus :size="16" />保存 Provider</Button></form></Card></div></section>

          <section v-else-if="active === 'API Keys'" class="space-y-6"><div class="flex flex-col justify-between gap-4 sm:flex-row sm:items-end"><div><p class="text-[10px] font-bold tracking-[.16em] text-teal-700">ACCESS CONTROL</p><h2 class="mt-1 text-xl font-semibold">API Keys</h2><p class="mt-1 text-sm text-muted-foreground">创建并管理调用 Meshway API 的访问凭证。</p></div><form class="flex gap-2" @submit.prevent="createKey"><Input v-model="newKeyName" required placeholder="Key 名称" class="w-48" /><Button type="submit"><Plus :size="16" />创建 Key</Button></form></div><Card v-if="createdKey" class="flex flex-col gap-4 border-teal-200 bg-teal-50 p-4 sm:flex-row sm:items-center"><KeyRound :size="19" class="text-teal-700" /><div class="min-w-0 flex-1"><strong class="block text-sm text-teal-900">新 Key 只显示这一次</strong><code class="mt-1 block truncate text-xs text-teal-700">{{ createdKey }}</code></div><Button variant="outline" size="sm" class="border-teal-300 bg-white" @click="copyKey"><Copy :size="14" />复制</Button></Card><Card class="overflow-hidden"><div class="flex items-center justify-between border-b px-5 py-4"><h3 class="text-sm font-semibold">访问凭证 <Badge variant="outline" class="ml-2">{{ keys.length }}</Badge></h3><span class="text-xs text-muted-foreground">数据库仅保存哈希</span></div><div class="subtle-scrollbar overflow-x-auto"><table class="data-table w-full min-w-[720px]"><thead><tr><th>名称</th><th>状态</th><th>创建时间</th><th>最近使用</th><th>操作</th></tr></thead><tbody><tr v-for="key in keys" :key="key.id"><td class="font-medium text-foreground"><span class="mr-2 inline-flex h-7 w-7 items-center justify-center rounded-md bg-slate-100"><KeyRound :size="13" /></span>{{ key.name }}</td><td><Badge :variant="key.enabled ? 'success' : 'outline'">{{ key.enabled ? '启用' : '停用' }}</Badge></td><td>{{ formatDate(key.created_at) }}</td><td>{{ formatDate(key.last_used) }}</td><td><Button variant="ghost" size="sm" @click="toggleKey(key)">{{ key.enabled ? '停用' : '启用' }}</Button></td></tr><tr v-if="!keys.length"><td colspan="5" class="h-36 text-center text-muted-foreground">暂无访问 Key</td></tr></tbody></table></div></Card></section>

          <section v-else class="space-y-6"><div class="flex items-end justify-between"><div><p class="text-[10px] font-bold tracking-[.16em] text-teal-700">AUDIT TRAIL</p><h2 class="mt-1 text-xl font-semibold">请求日志</h2><p class="mt-1 text-sm text-muted-foreground">最近 200 条 API 请求状态和性能记录。</p></div><Badge variant="success" class="gap-1.5"><span class="h-1.5 w-1.5 rounded-full bg-emerald-500" />自动记录中</Badge></div><Card class="overflow-hidden"><div class="subtle-scrollbar overflow-x-auto"><table class="data-table w-full min-w-[760px]"><thead><tr><th>接口</th><th>Provider</th><th>模型</th><th>状态</th><th>延迟</th><th>时间</th></tr></thead><tbody><tr v-for="log in logs" :key="log.id"><td class="font-mono text-xs text-teal-700">{{ log.path }}</td><td>{{ log.provider }}</td><td>{{ log.model || '-' }}</td><td><Badge :variant="log.status < 300 ? 'success' : 'destructive'">{{ log.status }} {{ log.status < 300 ? '成功' : '失败' }}</Badge></td><td>{{ log.latency_ms }} ms</td><td>{{ formatDate(log.created_at) }}</td></tr><tr v-if="!logs.length"><td colspan="6" class="h-40 text-center text-muted-foreground">暂无请求记录</td></tr></tbody></table></div></Card></section>
        </div>
      </div>
    </main>
  </div>
</template>
