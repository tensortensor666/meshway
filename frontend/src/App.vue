<script setup lang="ts">
import axios from 'axios'
import { computed, onMounted, ref } from 'vue'

type Provider = { id: string; name: string; type: string; base_url: string; enabled: number }
type ApiKey = { id: string; name: string; enabled: number; created_at: string; last_used?: string }
type Log = { id: string; provider: string; model?: string; path: string; status: number; latency_ms: number; created_at: string }

const token = ref(localStorage.getItem('meshway_admin_token') ?? '')
const active = ref('概览')
const stats = ref({ total_requests: 0, successful_requests: 0, success_rate: 0, average_latency_ms: 0 })
const providers = ref<Provider[]>([])
const keys = ref<ApiKey[]>([])
const logs = ref<Log[]>([])
const message = ref('')
const newKeyName = ref('')
const providerForm = ref({ name: '', type: 'openai', base_url: 'https://api.openai.com/v1', api_key: '' })
const api = computed(() => axios.create({ headers: { Authorization: `Bearer ${token.value}` } }))

async function refresh() {
  localStorage.setItem('meshway_admin_token', token.value)
  try {
    const [s, p, k, l] = await Promise.all([
      api.value.get('/api/admin/stats'), api.value.get('/api/admin/providers'),
      api.value.get('/api/admin/api-keys'), api.value.get('/api/admin/logs')
    ])
    stats.value = s.data; providers.value = p.data; keys.value = k.data; logs.value = l.data
    message.value = ''
  } catch { message.value = '无法连接管理接口，请检查 Admin Token 和后端地址' }
}

async function saveProvider() {
  await api.value.post('/api/admin/providers', providerForm.value)
  providerForm.value.api_key = ''
  message.value = 'Provider 已保存'
  await refresh()
}

async function createKey() {
  if (!newKeyName.value.trim()) return
  const result = await api.value.post('/api/admin/api-keys', { name: newKeyName.value })
  message.value = `新 Key（仅显示一次）：${result.data.key}`
  newKeyName.value = ''
  await refresh()
}

async function toggleKey(key: ApiKey) {
  await api.value.put(`/api/admin/api-keys/${key.id}`, { enabled: !key.enabled })
  await refresh()
}

onMounted(refresh)
</script>

<template>
  <div class="shell">
    <aside class="sidebar">
      <div class="brand"><span class="brand-mark">M</span><span>Meshway</span></div>
      <nav>
        <button v-for="item in ['概览', 'Providers', 'API Keys', '请求日志']" :key="item" :class="{ selected: active === item }" @click="active = item">
          <span class="nav-dot" />{{ item }}
        </button>
      </nav>
      <div class="side-foot">LOCAL GATEWAY<br /><span>v0.1.0</span></div>
    </aside>
    <main class="content">
      <header class="topbar"><div><p class="eyebrow">CONTROL PLANE</p><h1>{{ active }}</h1></div><div class="connection"><span class="status-dot" />本地服务</div></header>
      <section class="token-row"><label>Admin Token</label><input v-model="token" type="password" placeholder="MESHWAY_ADMIN_TOKEN" @keyup.enter="refresh" /><button class="primary" @click="refresh">连接</button></section>
      <p v-if="message" class="notice">{{ message }}</p>

      <section v-if="active === '概览'" class="view">
        <div class="metric-grid"><article><span>请求总量</span><strong>{{ stats.total_requests }}</strong></article><article><span>成功率</span><strong>{{ (stats.success_rate * 100).toFixed(1) }}%</strong></article><article><span>平均延迟</span><strong>{{ Math.round(stats.average_latency_ms) }}<small> ms</small></strong></article><article><span>可用 Provider</span><strong>{{ providers.filter(p => p.enabled).length }}</strong></article></div>
        <div class="section-head"><div><p class="eyebrow">RUNTIME</p><h2>最近请求</h2></div><button class="quiet" @click="active = '请求日志'">查看全部 →</button></div>
        <div class="table-wrap"><table><thead><tr><th>路径</th><th>Provider</th><th>模型</th><th>状态</th><th>延迟</th><th>时间</th></tr></thead><tbody><tr v-for="log in logs.slice(0, 6)" :key="log.id"><td class="mono">{{ log.path }}</td><td>{{ log.provider }}</td><td>{{ log.model || '-' }}</td><td><span :class="['pill', log.status < 300 ? 'ok' : 'bad']">{{ log.status }}</span></td><td>{{ log.latency_ms }} ms</td><td>{{ new Date(log.created_at).toLocaleString() }}</td></tr><tr v-if="!logs.length"><td colspan="6" class="empty">暂无请求记录</td></tr></tbody></table></div>
      </section>

      <section v-else-if="active === 'Providers'" class="view split"><div><div class="section-head"><div><p class="eyebrow">UPSTREAMS</p><h2>Provider 列表</h2></div></div><div class="provider-list"><article v-for="provider in providers" :key="provider.id"><div><strong>{{ provider.name }}</strong><span>{{ provider.type }} · {{ provider.base_url }}</span></div><span :class="['pill', provider.enabled ? 'ok' : 'muted']">{{ provider.enabled ? '启用' : '停用' }}</span></article><p v-if="!providers.length" class="empty">还没有配置 Provider</p></div></div><form class="form" @submit.prevent="saveProvider"><p class="eyebrow">ADD PROVIDER</p><h2>添加 Provider</h2><label>名称<input v-model="providerForm.name" required /></label><label>类型<select v-model="providerForm.type"><option value="openai">OpenAI 兼容</option><option value="anthropic">Anthropic</option></select></label><label>Base URL<input v-model="providerForm.base_url" required /></label><label>API Key<input v-model="providerForm.api_key" type="password" required /></label><button class="primary" type="submit">保存 Provider</button></form></section>

      <section v-else-if="active === 'API Keys'" class="view"><div class="section-head"><div><p class="eyebrow">ACCESS</p><h2>访问 Key</h2></div><form class="inline-form" @submit.prevent="createKey"><input v-model="newKeyName" placeholder="Key 名称" required /><button class="primary" type="submit">创建 Key</button></form></div><div class="table-wrap"><table><thead><tr><th>名称</th><th>状态</th><th>创建时间</th><th>最近使用</th><th></th></tr></thead><tbody><tr v-for="key in keys" :key="key.id"><td>{{ key.name }}</td><td><span :class="['pill', key.enabled ? 'ok' : 'muted']">{{ key.enabled ? '启用' : '停用' }}</span></td><td>{{ new Date(key.created_at).toLocaleString() }}</td><td>{{ key.last_used ? new Date(key.last_used).toLocaleString() : '-' }}</td><td><button class="quiet" @click="toggleKey(key)">{{ key.enabled ? '停用' : '启用' }}</button></td></tr><tr v-if="!keys.length"><td colspan="5" class="empty">暂无访问 Key</td></tr></tbody></table></div></section>

      <section v-else class="view"><div class="section-head"><div><p class="eyebrow">AUDIT TRAIL</p><h2>请求日志</h2></div></div><div class="table-wrap"><table><thead><tr><th>路径</th><th>Provider</th><th>模型</th><th>状态</th><th>延迟</th><th>时间</th></tr></thead><tbody><tr v-for="log in logs" :key="log.id"><td class="mono">{{ log.path }}</td><td>{{ log.provider }}</td><td>{{ log.model || '-' }}</td><td><span :class="['pill', log.status < 300 ? 'ok' : 'bad']">{{ log.status }}</span></td><td>{{ log.latency_ms }} ms</td><td>{{ new Date(log.created_at).toLocaleString() }}</td></tr><tr v-if="!logs.length"><td colspan="6" class="empty">暂无请求记录</td></tr></tbody></table></div></section>
    </main>
  </div>
</template>
