<script setup>
import { ref, computed, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { connectFeed } from "./lib/websocket";
import HeaderBar from "./components/HeaderBar.vue";
import Sidebar from "./components/Sidebar.vue";
import FeedList from "./components/FeedList.vue";
import LogPanel from "./components/LogPanel.vue";
import SourceManager from "./components/SourceManager.vue";

const items = ref([]);
const sources = ref([]);
const status = ref("Connecting...");
const logs = ref([]);
const showLogs = ref(false);
const itemCount = ref(0);
const connected = ref(false);
const search = ref("");
const compact = ref(false);
const activeSources = ref([]);
const showSourceManager = ref(false);
const sidebarCollapsed = ref(false);
const currentView = ref('feed'); // 'feed', 'sources', 'settings'

// Performance optimization: Limit items in memory
const MAX_ITEMS = 500;
const MAX_LOGS = 200;

let wsConnection = null;

onMounted(async () => {
  await loadSources();
  setupWebSocket();
});

onUnmounted(() => {
  if (wsConnection) {
    wsConnection.close();
  }
});

async function loadSources() {
  try {
    sources.value = await invoke("list_sources");
  } catch (e) {
    addLog("error", "Failed to load sources");
  }
}

function setupWebSocket() {
  if (wsConnection) {
    wsConnection.close();
  }

  wsConnection = connectFeed();
  wsConnection.subscribe((msg) => {
    handleWebSocketMessage(msg);
  });
}

function handleWebSocketMessage(msg) {
  if (msg.type === "__status__") {
    if (msg.event === "open") { 
      connected.value = true; 
      status.value = "Connected"; 
      addLog("info", "WebSocket connected");
    }
    if (msg.event === "close") { 
      connected.value = false; 
      status.value = "Reconnecting..."; 
      addLog("warn", "WebSocket disconnected");
    }
    if (msg.event === "error") { 
      connected.value = false; 
      addLog("error", "WebSocket error");
    }
  } else if (msg.type === "hello") {
    status.value = `Connected (v${msg.payload.server_version})`;
    addLog("info", "Server hello received");
  } else if (msg.type === "item") {
    addItem(msg.payload);
    itemCount.value++;
    addLog("recv", `New item: ${msg.payload.title?.substring(0, 50)}...`);
  } else if (msg.type === "heartbeat") {
    addLog("hb", "Heartbeat");
  }
}

function addLog(level, message) {
  logs.value.unshift({ 
    ts: Date.now(), 
    level, 
    msg: message 
  });
  // Keep only last 100 logs
  if (logs.value.length > 100) {
    logs.value.length = 100;
  }
}

function addItem(item) {
  const withMeta = { ...item, _arrivalTs: Date.now() };
  
  // Check if this item already exists (avoid duplicates)
  const existingIndex = items.value.findIndex(existing => existing.id === item.id);
  if (existingIndex !== -1) {
    // Update existing item if it's newer
    const existing = items.value[existingIndex];
    const existingTime = safeTs(existing.published_at) || existing._arrivalTs || 0;
    const newTime = safeTs(withMeta.published_at) || withMeta._arrivalTs || 0;
    
    if (newTime > existingTime) {
      items.value[existingIndex] = withMeta;
    }
  } else {
    // Add new item at the beginning
    items.value.unshift(withMeta);
  }
  
  // Sort by published_at desc (newest first), then by arrival time
  items.value.sort((a, b) => {
    // Use published_at if available and valid, otherwise use arrival time
    const timeA = safeTs(a.published_at) || a._arrivalTs || 0;
    const timeB = safeTs(b.published_at) || b._arrivalTs || 0;
    
    // Always sort by newest first
    return timeB - timeA;
  });
  
  // Keep only last 1000 items for better performance
  if (items.value.length > 1000) {
    items.value.length = 1000;
  }
}

function safeTs(s) {
  if (!s) return 0;
  const t = Date.parse(s);
  return isNaN(t) ? 0 : t;
}

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase();
  const act = activeSources.value;
  
  return items.value.filter(item => {
    const inSource = act.length === 0 || act.includes(item.source?.id);
    const inText = q.length === 0 || 
      (item.title?.toLowerCase().includes(q) || 
       item.summary?.toLowerCase().includes(q));
    return inSource && inText;
  });
});

const countsBySource = computed(() => {
  const q = search.value.trim().toLowerCase();
  const map = {};
  
  for (const item of items.value) {
    const inText = q.length === 0 || 
      (item.title?.toLowerCase().includes(q) || 
       item.summary?.toLowerCase().includes(q));
    if (!inText) continue;
    
    const id = item.source?.id || 'unknown';
    map[id] = (map[id] || 0) + 1;
  }
  
  return map;
});

const totalCountBySearch = computed(() => 
  Object.values(countsBySource.value).reduce((a, b) => a + b, 0)
);

async function refreshNow() {
  try { 
    await invoke("refresh_now", { id: null }); 
    addLog("info", "Manual refresh triggered");
  } catch (e) { 
    addLog("error", `Refresh failed: ${e}`);
  }
}

function toggleSource(id) {
  const idx = activeSources.value.indexOf(id);
  if (idx >= 0) {
    activeSources.value.splice(idx, 1);
  } else {
    activeSources.value.push(id);
  }
}

function onSourcesChanged(newSources) {
  sources.value = newSources;
}
</script>

<template>
  <div class="app">
    <!-- Sidebar Navigation -->
    <Sidebar 
      :collapsed="sidebarCollapsed"
      :current-view="currentView"
      :sources="sources"
      :active-sources="activeSources"
      :counts="countsBySource"
      :total="totalCountBySearch"
      @update:collapsed="sidebarCollapsed = $event"
      @update:view="currentView = $event"
      @toggle-source="toggleSource"
      @clear-sources="activeSources = []"
      @manage-sources="showSourceManager = true"
    />

    <!-- Main Content Area -->
    <div class="main-content">
      <!-- Header Bar -->
      <HeaderBar
        :connected="connected"
        :status="status"
        :search="search"
        :sources-count="sources.length"
        :item-count="itemCount"
        :compact="compact"
        :show-logs="showLogs"
        @update:search="search = $event"
        @update:compact="compact = $event"
        @update:show-logs="showLogs = $event"
        @refresh="refreshNow"
      />

      <!-- Content Area -->
      <div class="content-area">
        <div class="main-panel" :class="{ 'with-logs': showLogs }">
          <!-- Feed Content -->
          <div class="feed-container">
            <div v-if="items.length === 0" class="empty-state">
              <div class="empty-icon">📰</div>
              <h3 class="empty-title">No items received yet</h3>
              <p class="empty-description">
                Waiting for feed updates... Make sure your sources are configured correctly.
              </p>
              <div class="empty-actions">
                <button class="btn btn-primary" @click="refreshNow">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/>
                    <path d="M21 3v5h-5"/>
                    <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/>
                    <path d="M3 21v-5h5"/>
                  </svg>
                  Refresh Now
                </button>
                <button class="btn btn-secondary" @click="showSourceManager = true">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="12" cy="12" r="3"/>
                    <path d="M12 1v6m0 6v6"/>
                    <path d="m21 12-6 0m-6 0-6 0"/>
                  </svg>
                  Manage Sources
                </button>
              </div>
            </div>
            
            <FeedList 
              v-else 
              :items="filtered" 
              :compact="compact"
              class="animate-fade-in" 
            />
          </div>

          <!-- Logs Panel -->
          <LogPanel 
            v-if="showLogs" 
            :logs="logs" 
            class="logs-panel" 
          />
        </div>
      </div>
    </div>

    <!-- Modals -->
    <SourceManager 
      v-model="showSourceManager" 
      @changed="onSourcesChanged" 
    />
  </div>
</template>

<style scoped>
.app {
  display: flex;
  height: 100vh;
  width: 100vw;
  background: var(--bg-primary);
}

.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  margin-left: 240px;
  transition: margin-left 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.content-area {
  flex: 1;
  overflow: hidden;
}

.main-panel {
  display: flex;
  height: 100%;
  overflow: hidden;
}

.feed-container {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-6);
  background: var(--bg-primary);
}

.logs-panel {
  width: 350px;
  flex-shrink: 0;
  border-left: 1px solid var(--border-light);
  background: var(--bg-secondary);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 60vh;
  text-align: center;
  padding: var(--space-8);
}

.empty-icon {
  font-size: 4rem;
  margin-bottom: var(--space-4);
  opacity: 0.6;
}

.empty-title {
  font-size: var(--text-xl);
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: var(--space-2);
}

.empty-description {
  font-size: var(--text-base);
  color: var(--text-secondary);
  max-width: 400px;
  line-height: 1.6;
  margin-bottom: var(--space-6);
}

.empty-actions {
  display: flex;
  gap: var(--space-3);
  flex-wrap: wrap;
  justify-content: center;
}

/* Mobile Responsive */
@media (max-width: 768px) {
  .main-content {
    margin-left: 0;
  }
  
  .main-panel.with-logs {
    flex-direction: column;
  }
  
  .logs-panel {
    width: 100%;
    height: 250px;
    border-left: none;
    border-top: 1px solid var(--border-light);
  }
  
  .feed-container {
    padding: var(--space-4);
  }
  
  .empty-actions {
    flex-direction: column;
    align-items: center;
  }
  
  .empty-actions .btn {
    min-width: 200px;
  }
}

/* Smooth transitions for layout changes */
.feed-container,
.logs-panel {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}
</style>
