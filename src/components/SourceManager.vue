<script setup>
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps({ modelValue: Boolean })
const emit = defineEmits(['update:modelValue', 'changed'])

const loading = ref(false)
const list = ref([])
const showAddForm = ref(false)
const selectedCategory = ref('all')

const form = ref({
  id: '',
  name: '',
  url: '',
  kind: 'rss',
  interval_ms: 60000,
  category: 'general',
})

// Predefined crypto RSS feeds for quick addition
const cryptoPresets = [
  {
    name: 'CryptoPanic',
    url: 'https://cryptopanic.com/news/rss/',
    category: 'aggregator',
    interval_ms: 60000
  },
  {
    name: 'CoinDesk',
    url: 'https://www.coindesk.com/arc/outboundfeeds/rss/',
    category: 'news',
    interval_ms: 300000
  },
  {
    name: 'Cointelegraph',
    url: 'https://cointelegraph.com/rss',
    category: 'news',
    interval_ms: 300000
  },
  {
    name: 'The Block',
    url: 'https://www.theblock.co/rss.xml',
    category: 'news',
    interval_ms: 240000
  },
  {
    name: 'Bitcoin Magazine',
    url: 'https://bitcoinmagazine.com/.rss/full/',
    category: 'bitcoin',
    interval_ms: 360000
  },
  {
    name: 'DeFi Pulse',
    url: 'https://defipulse.com/blog/feed',
    category: 'defi',
    interval_ms: 600000
  },
  {
    name: 'Decrypt',
    url: 'https://decrypt.co/feed',
    category: 'news',
    interval_ms: 300000
  },
  {
    name: 'NewsBTC',
    url: 'https://www.newsbtc.com/feed/',
    category: 'news',
    interval_ms: 180000
  }
]

const sourceCategories = {
  all: 'All Sources',
  news: 'Crypto News',
  bitcoin: 'Bitcoin Focus',
  defi: 'DeFi & Web3',
  trading: 'Trading & Markets',
  institutional: 'Institutional',
  aggregator: 'News Aggregators',
  general: 'General'
}

const categorizedSources = computed(() => {
  const sources = list.value || []
  const categorized = {}
  
  Object.keys(sourceCategories).forEach(cat => {
    if (cat === 'all') return
    categorized[cat] = sources.filter(source => 
      getCategoryFromSource(source) === cat
    )
  })
  
  return categorized
})

const filteredSources = computed(() => {
  if (selectedCategory.value === 'all') {
    return list.value || []
  }
  return categorizedSources.value[selectedCategory.value] || []
})

function getCategoryFromSource(source) {
  const url = source.url.toLowerCase()
  const name = source.name.toLowerCase()
  
  if (name.includes('bitcoin') || url.includes('bitcoin')) return 'bitcoin'
  if (name.includes('defi') || url.includes('defi')) return 'defi'
  if (name.includes('panic') || name.includes('aggregator')) return 'aggregator'
  if (url.includes('binance') || url.includes('coinbase') || name.includes('trading')) return 'trading'
  if (name.includes('institutional') || name.includes('grayscale')) return 'institutional'
  if (name.includes('news') || url.includes('news') || 
      ['coindesk', 'cointelegraph', 'decrypt', 'theblock'].some(s => url.includes(s))) return 'news'
  
  return 'general'
}

onMounted(load)

async function load() {
  try {
    loading.value = true
    list.value = await invoke('list_sources')
  } catch (e) {
    console.warn('Failed to load sources:', e)
  } finally {
    loading.value = false
  }
}

function suggestId() {
  if (form.value.id?.trim()) return
  let base = form.value.name?.trim()
  if (!base && form.value.url) {
    try { 
      base = new URL(form.value.url).hostname.replace(/^www\./, '') 
    } catch {}
  }
  if (!base) return
  form.value.id = base.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)/g, '')
}

function addPreset(preset) {
  form.value.name = preset.name
  form.value.url = preset.url
  form.value.interval_ms = preset.interval_ms
  form.value.category = preset.category
  suggestId()
  showAddForm.value = true
}

async function addAllCryptoSources() {
  if (!confirm('Add all recommended crypto RSS sources? This will add 8 high-quality crypto news feeds.')) return
  
  try {
    loading.value = true
    for (const preset of cryptoPresets) {
      const src = {
        id: preset.name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)/g, ''),
        name: preset.name,
        url: preset.url,
        kind: 'rss',
        interval_ms: preset.interval_ms
      }
      await invoke('add_source', { src })
    }
    await load()
    emit('changed', list.value)
  } catch (e) {
    console.error('Failed to add crypto sources:', e)
  } finally {
    loading.value = false
  }
}

async function add() {
  if (!form.value.name?.trim() || !form.value.url?.trim()) return
  
  if (!form.value.id) suggestId()
  
  const src = { ...form.value }
  try {
    loading.value = true
    await invoke('add_source', { src })
    await load()
    emit('changed', list.value)
    // Reset form
    form.value = { id: '', name: '', url: '', kind: 'rss', interval_ms: 60000, category: 'general' }
    showAddForm.value = false
  } catch (e) {
    console.error('Failed to add source:', e)
  } finally {
    loading.value = false
  }
}

async function remove(id) {
  if (!confirm(`Remove source "${list.value.find(s => s.id === id)?.name}"?`)) return
  
  try {
    loading.value = true
    await invoke('remove_source', { id })
    await load()
    emit('changed', list.value)
  } catch (e) {
    console.error('Failed to remove source:', e)
  } finally {
    loading.value = false
  }
}

async function refresh(id) {
  try { 
    await invoke('refresh_now', { id }) 
  } catch (e) {
    console.error('Failed to refresh source:', e)
  }
}

function close() { 
  emit('update:modelValue', false)
  showAddForm.value = false
}

function formatInterval(ms) {
  const seconds = ms / 1000
  const minutes = seconds / 60
  if (minutes >= 60) {
    const hours = minutes / 60
    return `${Math.round(hours)}h`
  }
  return `${Math.round(minutes)}m`
}

function getDomainFromUrl(url) {
  try {
    return new URL(url).hostname
  } catch {
    return url
  }
}
</script>

<template>
  <teleport to="body">
    <div v-if="modelValue" class="modal-overlay" @click.self="close">
      <div class="modal animate-fade-in">
        <!-- Modal Header -->
        <header class="modal-header">
          <div class="header-content">
            <h2 class="modal-title">Manage Sources</h2>
            <p class="modal-subtitle">Configure your RSS feeds and news sources</p>
          </div>
          
          <div class="header-actions">
            <button 
              class="btn btn-primary"
              @click="showAddForm = !showAddForm"
              :class="{ active: showAddForm }"
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="12" y1="5" x2="12" y2="19"/>
                <line x1="5" y1="12" x2="19" y2="12"/>
              </svg>
              {{ showAddForm ? 'Cancel' : 'Add Source' }}
            </button>
            
            <button class="btn btn-ghost" @click="close" title="Close">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="18" y1="6" x2="6" y2="18"/>
                <line x1="6" y1="6" x2="18" y2="18"/>
              </svg>
            </button>
          </div>
        </header>

        <div class="modal-content">
          <!-- Crypto Presets Section -->
          <div v-if="showAddForm" class="crypto-presets-section animate-fade-in">
            <div class="presets-header">
              <h3 class="form-title">🚀 Quick Add Crypto Sources</h3>
              <p class="form-description">Add proven crypto news sources with one click</p>
              <button 
                @click="addAllCryptoSources"
                class="btn btn-primary btn-sm"
                :disabled="loading"
              >
                Add All Crypto Sources ({{ cryptoPresets.length }})
              </button>
            </div>
            
            <div class="presets-grid">
              <div 
                v-for="preset in cryptoPresets" 
                :key="preset.name"
                class="preset-card"
                @click="addPreset(preset)"
              >
                <div class="preset-info">
                  <h4 class="preset-name">{{ preset.name }}</h4>
                  <p class="preset-category">{{ sourceCategories[preset.category] }}</p>
                </div>
                <div class="preset-interval">{{ formatInterval(preset.interval_ms) }}</div>
              </div>
            </div>
          </div>

          <!-- Add Source Form -->
          <div v-if="showAddForm" class="add-form-section animate-fade-in">
            <div class="form-header">
              <h3 class="form-title">Add Custom Source</h3>
              <p class="form-description">Enter the details for your RSS feed or news source</p>
            </div>

            <form @submit.prevent="add" class="source-form">
              <div class="form-grid">
                <div class="form-group">
                  <label for="source-name" class="form-label">Source Name</label>
                  <input
                    id="source-name"
                    v-model="form.name"
                    @blur="suggestId"
                    class="input"
                    placeholder="e.g., TechCrunch"
                    required
                  />
                </div>

                <div class="form-group">
                  <label for="source-url" class="form-label">Feed URL</label>
                  <input
                    id="source-url"
                    v-model="form.url"
                    class="input"
                    type="url"
                    placeholder="https://..."
                    required
                  />
                </div>

                <div class="form-group">
                  <label for="source-id" class="form-label">
                    Identifier 
                    <span class="label-hint">(auto-generated)</span>
                  </label>
                  <input
                    id="source-id"
                    v-model="form.id"
                    class="input"
                    placeholder="Auto-generated from name"
                  />
                </div>

                <div class="form-group">
                  <label for="source-interval" class="form-label">Update Interval</label>
                  <select
                    id="source-interval"
                    v-model.number="form.interval_ms"
                    class="input"
                  >
                    <option :value="30000">30 seconds</option>
                    <option :value="60000">1 minute</option>
                    <option :value="300000">5 minutes</option>
                    <option :value="900000">15 minutes</option>
                    <option :value="1800000">30 minutes</option>
                    <option :value="3600000">1 hour</option>
                  </select>
                </div>

                <div class="form-group">
                  <label for="source-type" class="form-label">Source Type</label>
                  <select
                    id="source-type"
                    v-model="form.kind"
                    class="input"
                  >
                    <option value="rss">RSS/Atom Feed</option>
                  </select>
                </div>
              </div>

              <div class="form-actions">
                <button 
                  type="button" 
                  class="btn btn-secondary"
                  @click="showAddForm = false"
                >
                  Cancel
                </button>
                <button 
                  type="submit"
                  class="btn btn-primary"
                  :disabled="!form.name?.trim() || !form.url?.trim() || loading"
                >
                  <svg v-if="loading" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="animate-pulse">
                    <circle cx="12" cy="12" r="10"/>
                  </svg>
                  {{ loading ? 'Adding...' : 'Add Source' }}
                </button>
              </div>
            </form>
          </div>

          <!-- Sources List -->
          <div class="sources-section">
            <div class="section-header">
              <h3 class="section-title">
                Current Sources 
                <span class="count-badge badge">{{ list.length }}</span>
              </h3>
              
              <div class="section-actions">
                <!-- Category Filter -->
                <div class="category-filter">
                  <select v-model="selectedCategory" class="input input-sm">
                    <option 
                      v-for="(label, value) in sourceCategories" 
                      :key="value" 
                      :value="value"
                    >
                      {{ label }} ({{ value === 'all' ? list.length : (categorizedSources[value] || []).length }})
                    </option>
                  </select>
                </div>
                
                <button 
                  class="btn btn-ghost btn-sm"
                  @click="load"
                  :disabled="loading"
                  title="Refresh list"
                >
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/>
                    <path d="M21 3v5h-5"/>
                    <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/>
                    <path d="M3 21v-5h5"/>
                  </svg>
                </button>
              </div>
            </div>

            <div class="sources-list">
              <div v-if="filteredSources.length === 0 && list.length === 0" class="empty-sources">
                <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" class="empty-icon">
                  <path d="M4 11a9 9 0 0 1 9 9"/>
                  <path d="M4 4a16 16 0 0 1 16 16"/>
                  <circle cx="5" cy="19" r="1"/>
                </svg>
                <h4 class="empty-title">No sources configured</h4>
                <p class="empty-description">Add your first RSS feed to get started</p>
              </div>

              <div v-if="filteredSources.length === 0 && list.length > 0" class="empty-sources">
                <div class="empty-icon">📂</div>
                <h4 class="empty-title">No sources in this category</h4>
                <p class="empty-description">Try selecting a different category or add new sources</p>
              </div>

              <div 
                v-for="source in filteredSources" 
                :key="source.id"
                class="source-item card"
              >
                <div class="source-main">
                  <div class="source-info">
                    <div class="source-header">
                      <h4 class="source-name">{{ source.name }}</h4>
                      <span class="category-badge" :class="`badge-${getCategoryFromSource(source)}`">
                        {{ sourceCategories[getCategoryFromSource(source)] }}
                      </span>
                    </div>
                    <p class="source-url">{{ getDomainFromUrl(source.url) }}</p>
                    <div class="source-meta">
                      <span class="source-id">{{ source.id }}</span>
                      <span class="source-interval">{{ formatInterval(source.interval_ms) }}</span>
                      <span class="source-type badge badge-primary">{{ source.kind.toUpperCase() }}</span>
                    </div>
                  </div>

                  <div class="source-actions">
                    <button 
                      class="btn btn-ghost btn-sm"
                      @click="refresh(source.id)"
                      title="Refresh now"
                    >
                      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/>
                        <path d="M21 3v5h-5"/>
                        <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/>
                        <path d="M3 21v-5h5"/>
                      </svg>
                    </button>

                    <button 
                      class="btn btn-danger btn-sm"
                      @click="remove(source.id)"
                      title="Remove source"
                    >
                      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <polyline points="3,6 5,6 21,6"/>
                        <path d="M19,6v14a2,2 0,0,1-2,2H7a2,2 0,0,1-2-2V6m3,0V4a2,2 0,0,1,2-2h4a2,2 0,0,1,2,2v2"/>
                      </svg>
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </teleport>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  padding: var(--space-4);
}

.modal {
  width: min(900px, 95vw);
  max-height: min(800px, 90vh);
  background: var(--surface-elevated);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-lg);
  border: 1px solid var(--border-light);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* Modal Header */
.modal-header {
  padding: var(--space-6);
  border-bottom: 1px solid var(--border-light);
  background: var(--surface-glass);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-4);
}

.header-content {
  flex: 1;
}

.modal-title {
  font-size: var(--text-2xl);
  font-weight: 700;
  color: var(--text-primary);
  margin: 0 0 var(--space-1) 0;
}

.modal-subtitle {
  font-size: var(--text-base);
  color: var(--text-secondary);
  margin: 0;
}

.header-actions {
  display: flex;
  gap: var(--space-2);
}

/* Modal Content */
.modal-content {
  flex: 1;
  overflow: auto;
  display: flex;
  flex-direction: column;
}

/* Add Form Section */
.add-form-section {
  padding: var(--space-6);
  border-bottom: 1px solid var(--border-light);
  background: rgba(0, 122, 255, 0.02);
}

.form-header {
  margin-bottom: var(--space-6);
}

.form-title {
  font-size: var(--text-lg);
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 var(--space-1) 0;
}

.form-description {
  font-size: var(--text-sm);
  color: var(--text-secondary);
  margin: 0;
}

.source-form {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.form-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--space-4);
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.form-group:first-child,
.form-group:nth-child(2) {
  grid-column: 1 / -1;
}

.form-label {
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--text-primary);
  display: flex;
  align-items: center;
  gap: var(--space-1);
}

.label-hint {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  font-weight: 400;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-3);
  padding-top: var(--space-4);
  border-top: 1px solid var(--border-light);
}

/* Sources Section */
.sources-section {
  flex: 1;
  padding: var(--space-6);
  display: flex;
  flex-direction: column;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--space-4);
}

.section-title {
  font-size: var(--text-lg);
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.count-badge {
  font-size: var(--text-xs);
}

.section-actions {
  display: flex;
  gap: var(--space-2);
}

.sources-list {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  overflow-y: auto;
}

/* Empty State */
.empty-sources {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-12) var(--space-6);
  text-align: center;
}

.empty-icon {
  color: var(--text-tertiary);
  margin-bottom: var(--space-4);
}

.empty-title {
  font-size: var(--text-lg);
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 var(--space-2) 0;
}

.empty-description {
  font-size: var(--text-base);
  color: var(--text-secondary);
  margin: 0;
}

/* Source Items */
.source-item {
  padding: var(--space-4);
  border-radius: var(--radius-md);
  transition: all 0.2s ease;
}

.source-item:hover {
  transform: translateY(-1px);
  box-shadow: var(--shadow-md);
}

.source-main {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-4);
}

.source-info {
  flex: 1;
  min-width: 0;
}

.source-name {
  font-size: var(--text-base);
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 var(--space-1) 0;
  line-height: 1.3;
}

.source-url {
  font-size: var(--text-sm);
  color: var(--text-secondary);
  margin: 0 0 var(--space-2) 0;
  font-family: ui-monospace, SFMono-Regular, monospace;
  word-break: break-all;
}

.source-meta {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex-wrap: wrap;
}

.source-id {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  font-family: ui-monospace, SFMono-Regular, monospace;
  background: var(--bg-tertiary);
  padding: 2px var(--space-2);
  border-radius: var(--radius-xs);
}

.source-interval {
  font-size: var(--text-xs);
  color: var(--text-secondary);
  font-weight: 500;
}

.source-actions {
  display: flex;
  gap: var(--space-2);
  flex-shrink: 0;
}

/* Responsive Design */
@media (max-width: 768px) {
  .modal-overlay {
    padding: var(--space-2);
  }
  
  .modal {
    width: 100%;
    max-height: 95vh;
  }
  
  .modal-header {
    padding: var(--space-4);
    flex-direction: column;
    align-items: stretch;
    gap: var(--space-3);
  }
  
  .header-actions {
    justify-content: space-between;
  }
  
  .add-form-section,
  .sources-section {
    padding: var(--space-4);
  }
  
  .form-grid {
    grid-template-columns: 1fr;
  }
  
  .form-actions {
    flex-direction: column-reverse;
  }
  
  .source-main {
    flex-direction: column;
    align-items: stretch;
    gap: var(--space-3);
  }
  
  .source-actions {
    justify-content: flex-end;
  }
  
  .section-header {
    flex-direction: column;
    align-items: stretch;
    gap: var(--space-2);
  }
}

/* Loading States */
.animate-pulse {
  animation: pulse 2s infinite;
}

/* Crypto Presets Section */
.crypto-presets-section {
  margin-bottom: var(--space-6);
  padding: var(--space-4);
  background: var(--bg-secondary);
  border-radius: var(--radius-lg);
  border: 1px solid var(--border-light);
}

.presets-header {
  margin-bottom: var(--space-4);
  text-align: center;
}

.presets-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: var(--space-3);
  margin-top: var(--space-4);
}

.preset-card {
  padding: var(--space-3);
  background: var(--bg-primary);
  border: 1px solid var(--border-light);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.preset-card:hover {
  background: var(--bg-secondary);
  border-color: var(--border-medium);
  transform: translateY(-1px);
}

.preset-info {
  flex: 1;
}

.preset-name {
  font-size: var(--text-sm);
  font-weight: 600;
  margin: 0 0 var(--space-1) 0;
}

.preset-category {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  margin: 0;
}

.preset-interval {
  font-size: var(--text-xs);
  color: var(--text-secondary);
  font-weight: 500;
  background: var(--bg-tertiary);
  padding: var(--space-1) var(--space-2);
  border-radius: var(--radius-sm);
}

/* Category Filter */
.category-filter {
  margin-right: var(--space-2);
}

.category-filter .input {
  min-width: 180px;
}

/* Category Badges */
.source-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--space-2);
}

.category-badge {
  font-size: var(--text-xs);
  padding: var(--space-1) var(--space-2);
  border-radius: var(--radius-sm);
  font-weight: 500;
}

.badge-news {
  background: #e3f2fd;
  color: #1565c0;
}

.badge-bitcoin {
  background: #fff3e0;
  color: #f57c00;
}

.badge-defi {
  background: #f3e5f5;
  color: #7b1fa2;
}

.badge-trading {
  background: #e8f5e8;
  color: #388e3c;
}

.badge-institutional {
  background: #fce4ec;
  color: #c2185b;
}

.badge-aggregator {
  background: #f3e5ab;
  color: #f57f17;
}

.badge-general {
  background: var(--bg-tertiary);
  color: var(--text-secondary);
}

/* Scrollbar Styling */
.sources-list::-webkit-scrollbar {
  width: 8px;
}

.sources-list::-webkit-scrollbar-track {
  background: transparent;
}

.sources-list::-webkit-scrollbar-thumb {
  background: var(--border-medium);
  border-radius: var(--radius-full);
}

.sources-list::-webkit-scrollbar-thumb:hover {
  background: var(--border-strong);
}
</style>