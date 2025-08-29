<script setup>
import { computed } from 'vue';

const props = defineProps({
  connected: Boolean,
  status: String,
  search: String,
  sourcesCount: Number,
  itemCount: Number,
  compact: Boolean,
  showLogs: Boolean
});

const emit = defineEmits([
  'update:search',
  'update:compact', 
  'update:show-logs',
  'refresh'
]);

const statusColor = computed(() => {
  if (props.connected) return 'var(--accent-success)';
  return 'var(--accent-danger)';
});
</script>

<template>
  <header class="header">
    <!-- Search Bar -->
    <div class="search-section">
      <div class="search-container">
        <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"/>
          <path d="m21 21-4.35-4.35"/>
        </svg>
        <input
          class="search-input"
          type="search"
          :value="search"
          placeholder="Search articles, sources..."
          @input="emit('update:search', $event.target.value)"
        />
        <kbd v-if="!search" class="search-shortcut">⌘K</kbd>
      </div>
    </div>

    <!-- Status & Actions -->
    <div class="header-actions">
      <!-- Connection Status -->
      <div class="status-indicator" :class="{ connected }">
        <div class="status-dot"></div>
        <span class="status-text">{{ status }}</span>
      </div>

      <!-- View Toggle Buttons -->
      <div class="toggle-group">
        <button 
          class="btn btn-ghost btn-sm"
          :class="{ active: !compact }"
          @click="emit('update:compact', false)"
          title="Comfortable view"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="7" height="7"/>
            <rect x="14" y="3" width="7" height="7"/>
            <rect x="3" y="14" width="7" height="7"/>
            <rect x="14" y="14" width="7" height="7"/>
          </svg>
        </button>
        <button 
          class="btn btn-ghost btn-sm"
          :class="{ active: compact }"
          @click="emit('update:compact', true)"
          title="Compact view"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="3" y1="6" x2="21" y2="6"/>
            <line x1="3" y1="12" x2="21" y2="12"/>
            <line x1="3" y1="18" x2="21" y2="18"/>
          </svg>
        </button>
      </div>

      <!-- Action Buttons -->
      <button 
        class="btn btn-secondary"
        @click="emit('refresh')"
        title="Refresh feeds"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/>
          <path d="M21 3v5h-5"/>
          <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/>
          <path d="M3 21v-5h5"/>
        </svg>
        <span class="btn-label">Refresh</span>
      </button>

      <button 
        class="btn btn-ghost"
        :class="{ active: showLogs }"
        @click="emit('update:show-logs', !showLogs)"
        title="Toggle logs panel"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
          <polyline points="14,2 14,8 20,8"/>
          <line x1="16" y1="13" x2="8" y2="13"/>
          <line x1="16" y1="17" x2="8" y2="17"/>
          <polyline points="10,9 9,9 8,9"/>
        </svg>
        <span class="btn-label">{{ showLogs ? 'Hide' : 'Show' }} Logs</span>
      </button>
    </div>
  </header>
</template>

<style scoped>
.header {
  height: 60px;
  background: var(--surface-glass);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-bottom: 1px solid var(--border-light);
  display: flex;
  align-items: center;
  padding: 0 var(--space-6);
  position: sticky;
  top: 0;
  z-index: 5;
  gap: var(--space-4);
}

.search-section {
  flex: 1;
  max-width: 500px;
}

.search-container {
  position: relative;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: var(--space-3);
  color: var(--text-tertiary);
  z-index: 1;
}

.search-input {
  width: 100%;
  padding: var(--space-2) var(--space-10) var(--space-2) var(--space-10);
  border: 1px solid var(--border-light);
  border-radius: var(--radius-lg);
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-size: var(--text-sm);
  transition: all 0.2s ease;
}

.search-input:focus {
  outline: none;
  border-color: var(--accent-primary);
  box-shadow: 0 0 0 3px rgba(0, 122, 255, 0.1);
  background: var(--surface-elevated);
}

.search-input::placeholder {
  color: var(--text-tertiary);
}

.search-shortcut {
  position: absolute;
  right: var(--space-3);
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  background: var(--bg-tertiary);
  padding: 2px var(--space-1);
  border-radius: var(--radius-xs);
  border: 1px solid var(--border-light);
  font-family: ui-monospace, SFMono-Regular, monospace;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-1) var(--space-3);
  border-radius: var(--radius-full);
  background: var(--bg-tertiary);
  border: 1px solid var(--border-light);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--accent-danger);
  transition: background-color 0.3s ease;
}

.status-indicator.connected .status-dot {
  background: var(--accent-success);
}

.status-text {
  font-size: var(--text-sm);
  font-weight: 500;
  color: var(--text-secondary);
  white-space: nowrap;
}

.status-indicator.connected .status-text {
  color: var(--text-primary);
}

.toggle-group {
  display: flex;
  background: var(--bg-tertiary);
  border-radius: var(--radius-sm);
  padding: 2px;
  border: 1px solid var(--border-light);
}

.toggle-group .btn {
  margin: 0;
  border-radius: calc(var(--radius-sm) - 2px);
  border: none;
  background: transparent;
}

.toggle-group .btn.active {
  background: var(--surface-elevated);
  color: var(--text-primary);
  box-shadow: var(--shadow-xs);
}

.btn-label {
  font-size: var(--text-sm);
  font-weight: 500;
}

/* Responsive Design */
@media (max-width: 768px) {
  .header {
    padding: 0 var(--space-4);
    gap: var(--space-2);
  }
  
  .search-section {
    max-width: none;
  }
  
  .btn-label {
    display: none;
  }
  
  .status-text {
    display: none;
  }
  
  .search-shortcut {
    display: none;
  }
}

@media (max-width: 480px) {
  .header {
    flex-wrap: wrap;
    height: auto;
    padding: var(--space-3) var(--space-4);
  }
  
  .search-section {
    width: 100%;
    order: -1;
  }
  
  .header-actions {
    justify-content: space-between;
    width: 100%;
  }
}
</style>
