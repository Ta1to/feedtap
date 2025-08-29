<script setup>
import { ref, computed } from 'vue';

const props = defineProps({
  collapsed: Boolean,
  currentView: String,
  sources: Array,
  activeSources: Array,
  counts: Object,
  total: Number
});

const emit = defineEmits([
  'update:collapsed',
  'update:view',
  'toggle-source',
  'clear-sources',
  'manage-sources'
]);

const views = [
  { id: 'feed', label: 'Feed', icon: 'newspaper' },
  { id: 'sources', label: 'Sources', icon: 'rss' },
  { id: 'settings', label: 'Settings', icon: 'settings' }
];

function getIconSvg(iconName) {
  const icons = {
    newspaper: '<path d="M4 22h16a2 2 0 0 0 2-2V4a2 2 0 0 0-2-2H8a2 2 0 0 0-2 2v16a2 2 0 0 1-2 2Zm0 0a2 2 0 0 1-2-2v-9c0-1.1.9-2 2-2h2"/><path d="M18 14h-8"/><path d="M15 18h-5"/><path d="M10 6h8v4h-8V6Z"/>',
    rss: '<path d="M4 11a9 9 0 0 1 9 9"/><path d="M4 4a16 16 0 0 1 16 16"/><circle cx="5" cy="19" r="1"/>',
    settings: '<circle cx="12" cy="12" r="3"/><path d="M12 1v6m0 6v6"/><path d="m21 12-6 0m-6 0-6 0"/>'
  };
  return icons[iconName] || '';
}
</script>

<template>
  <aside class="sidebar" :class="{ collapsed }">
    <!-- Brand Header -->
    <div class="sidebar-header">
      <div class="brand">
        <div class="brand-icon">📰</div>
        <div v-if="!collapsed" class="brand-text">
          <h1 class="brand-title">FeedTap</h1>
          <p class="brand-subtitle">Live Feed Reader</p>
        </div>
      </div>
      <button 
        class="btn btn-ghost btn-sm collapse-toggle"
        @click="emit('update:collapsed', !collapsed)"
        :title="collapsed ? 'Expand Sidebar' : 'Collapse Sidebar'"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path :d="collapsed ? 'M9 18l6-6-6-6' : 'M15 18l-6-6 6-6'"/>
        </svg>
      </button>
    </div>

    <!-- Navigation -->
    <nav class="sidebar-nav">
      <ul class="nav-list">
        <li v-for="view in views" :key="view.id" class="nav-item">
          <button 
            class="nav-link" 
            :class="{ active: currentView === view.id }"
            @click="emit('update:view', view.id)"
          >
            <svg class="nav-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <g v-html="getIconSvg(view.icon)"></g>
            </svg>
            <span v-if="!collapsed" class="nav-label">{{ view.label }}</span>
          </button>
        </li>
      </ul>
    </nav>

    <!-- Sources Filter (only visible in feed view) -->
    <div v-if="currentView === 'feed' && !collapsed" class="sources-section">
      <div class="section-header">
        <h3 class="section-title">Sources</h3>
        <button 
          class="btn btn-ghost btn-sm"
          @click="emit('manage-sources')"
          title="Manage Sources"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z"/>
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1 1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1Z"/>
          </svg>
        </button>
      </div>

      <div class="sources-list">
        <!-- All Sources Button -->
        <button 
          class="source-item" 
          :class="{ active: activeSources.length === 0 }"
          @click="emit('clear-sources')"
        >
          <div class="source-info">
            <span class="source-name">All Sources</span>
          </div>
          <span class="source-count badge">{{ total || 0 }}</span>
        </button>

        <!-- Individual Sources -->
        <button 
          v-for="source in sources" 
          :key="source.id"
          class="source-item"
          :class="{ active: activeSources.includes(source.id) }"
          @click="emit('toggle-source', source.id)"
        >
          <div class="source-info">
            <span class="source-name">{{ source.name }}</span>
            <span class="source-url">{{ source.url.replace(/^https?:\/\//, '').split('/')[0] }}</span>
          </div>
          <span class="source-count badge">{{ counts[source.id] || 0 }}</span>
        </button>
      </div>
    </div>

    <!-- Footer -->
    <div v-if="!collapsed" class="sidebar-footer">
      <div class="footer-stats">
        <div class="stat">
          <span class="stat-value">{{ sources.length }}</span>
          <span class="stat-label">Sources</span>
        </div>
        <div class="stat">
          <span class="stat-value">{{ total }}</span>
          <span class="stat-label">Items</span>
        </div>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 240px;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border-light);
  display: flex;
  flex-direction: column;
  position: fixed;
  top: 0;
  left: 0;
  bottom: 0;
  z-index: 10;
  transition: width 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.sidebar.collapsed {
  width: 60px;
}

/* Header */
.sidebar-header {
  padding: var(--space-4);
  border-bottom: 1px solid var(--border-light);
  display: flex;
  align-items: center;
  justify-content: space-between;
  min-height: 80px;
}

.brand {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  flex: 1;
}

.brand-icon {
  font-size: 1.5rem;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.brand-text {
  flex: 1;
  min-width: 0;
}

.brand-title {
  font-size: var(--text-lg);
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
  line-height: 1.2;
}

.brand-subtitle {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  margin: 0;
  line-height: 1.2;
}

.collapse-toggle {
  padding: var(--space-1);
  opacity: 0.7;
}

.collapse-toggle:hover {
  opacity: 1;
}

/* Navigation */
.sidebar-nav {
  padding: var(--space-2) 0;
}

.nav-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.nav-item {
  margin: 0 var(--space-2) var(--space-1);
}

.nav-link {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  border-radius: var(--radius-sm);
  background: transparent;
  border: none;
  color: var(--text-secondary);
  font-size: var(--text-sm);
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  width: 100%;
  text-align: left;
}

.nav-link:hover {
  background: var(--surface-overlay);
  color: var(--text-primary);
}

.nav-link.active {
  background: rgba(0, 122, 255, 0.1);
  color: var(--accent-primary);
}

.nav-icon {
  flex-shrink: 0;
}

.nav-label {
  flex: 1;
  min-width: 0;
}

/* Sources Section */
.sources-section {
  flex: 1;
  padding: var(--space-4) var(--space-2) 0;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 var(--space-2) var(--space-2);
  margin-bottom: var(--space-2);
}

.section-title {
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin: 0;
}

.sources-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  padding: 0 var(--space-2);
}

.source-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-sm);
  background: transparent;
  border: none;
  cursor: pointer;
  transition: all 0.2s ease;
  text-align: left;
  width: 100%;
}

.source-item:hover {
  background: var(--surface-overlay);
}

.source-item.active {
  background: rgba(0, 122, 255, 0.08);
  border-left: 3px solid var(--accent-primary);
  padding-left: calc(var(--space-3) - 3px);
}

.source-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.source-name {
  font-size: var(--text-sm);
  font-weight: 500;
  color: var(--text-primary);
  line-height: 1.3;
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.source-url {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  line-height: 1.2;
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.source-count {
  font-size: var(--text-xs);
  font-weight: 600;
  min-width: auto;
  padding: 2px var(--space-2);
}

/* Footer */
.sidebar-footer {
  padding: var(--space-4);
  border-top: 1px solid var(--border-light);
  margin-top: auto;
}

.footer-stats {
  display: flex;
  gap: var(--space-6);
}

.stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  flex: 1;
}

.stat-value {
  font-size: var(--text-lg);
  font-weight: 700;
  color: var(--text-primary);
  line-height: 1.2;
}

.stat-label {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  line-height: 1.2;
}

/* Scrollbar Styling */
.sources-list::-webkit-scrollbar {
  width: 4px;
}

.sources-list::-webkit-scrollbar-track {
  background: transparent;
}

.sources-list::-webkit-scrollbar-thumb {
  background: var(--border-light);
  border-radius: var(--radius-full);
}

.sources-list::-webkit-scrollbar-thumb:hover {
  background: var(--border-medium);
}

/* Mobile Responsive */
@media (max-width: 768px) {
  .sidebar {
    transform: translateX(-100%);
    transition: transform 0.3s ease;
    z-index: 50;
  }
  
  .sidebar.open {
    transform: translateX(0);
  }
}
</style>
