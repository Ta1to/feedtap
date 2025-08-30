<script setup>
import { onUpdated, ref, computed } from 'vue';

const props = defineProps({ logs: Array });

const container = ref(null);
const autoScroll = ref(true);

onUpdated(() => { 
  if (container.value && autoScroll.value) { 
    container.value.scrollTop = 0; 
  } 
});

const logLevels = {
  hb: { label: 'HB', color: 'var(--text-tertiary)', icon: '💓' },
  recv: { label: 'RECV', color: 'var(--accent-primary)', icon: '📨' },
  sent: { label: 'SENT', color: 'var(--accent-secondary)', icon: '📤' },
  info: { label: 'INFO', color: 'var(--accent-success)', icon: 'ℹ️' },
  warn: { label: 'WARN', color: 'var(--accent-warning)', icon: '⚠️' },
  error: { label: 'ERROR', color: 'var(--accent-danger)', icon: '❌' }
};

function formatTime(timestamp) {
  return new Date(timestamp).toLocaleTimeString('en-US', {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  });
}

function clearLogs() {
  if (confirm('Clear all logs?')) {
    // Emit event to parent to clear logs
    // For now, we'll just hide them visually
  }
}
</script>

<template>
  <aside class="log-panel">
    <!-- Header -->
    <div class="log-header">
      <div class="header-title">
        <h3 class="title text-primary">Activity Logs</h3>
        <span class="log-count badge">{{ logs.length }}</span>
      </div>
      
      <div class="header-actions">
        <button 
          class="btn btn-ghost btn-sm"
          @click="autoScroll = !autoScroll"
          :class="{ active: autoScroll }"
          title="Auto-scroll"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="17,13 12,18 7,13"/>
            <polyline points="17,6 12,11 7,6"/>
          </svg>
        </button>
        
        <button 
          class="btn btn-ghost btn-sm"
          @click="clearLogs"
          title="Clear logs"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="3,6 5,6 21,6"/>
            <path d="M19,6v14a2,2 0,0,1-2,2H7a2,2 0,0,1-2-2V6m3,0V4a2,2 0,0,1,2-2h4a2,2 0,0,1,2,2v2"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- Logs Container -->
    <div class="logs-container" ref="container">
      <div v-if="logs.length === 0" class="empty-logs">
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" class="empty-icon">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
          <polyline points="14,2 14,8 20,8"/>
        </svg>
        <p class="empty-text">No activity yet</p>
      </div>

      <div v-else class="logs-list">
        <div 
          v-for="(log, index) in logs" 
          :key="`${log.ts}-${index}`"
          class="log-entry"
          :class="`log-${log.level}`"
        >
          <div class="log-time">{{ formatTime(log.ts) }}</div>
          
          <div class="log-level" :style="{ color: logLevels[log.level]?.color }">
            <span class="level-icon">{{ logLevels[log.level]?.icon || '📝' }}</span>
            <span class="level-text">{{ logLevels[log.level]?.label || log.level.toUpperCase() }}</span>
          </div>
          
          <div class="log-message" :title="log.msg">{{ log.msg }}</div>
        </div>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.log-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-secondary);
  border-left: 1px solid var(--border-light);
}

.log-header {
  padding: var(--space-4);
  border-bottom: 1px solid var(--border-light);
  background: var(--surface-glass);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  position: sticky;
  top: 0;
  z-index: 2;
}

.header-title {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-bottom: var(--space-3);
}

.title {
  font-size: var(--text-base);
  font-weight: 600;
  margin: 0;
}

.log-count {
  font-size: var(--text-xs);
  font-weight: 600;
}

.header-actions {
  display: flex;
  gap: var(--space-1);
}

.header-actions .btn.active {
  background: var(--surface-overlay);
  color: var(--accent-primary);
}

.logs-container {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

.empty-logs {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 200px;
  text-align: center;
  padding: var(--space-6);
}

.empty-icon {
  color: var(--text-tertiary);
  margin-bottom: var(--space-2);
}

.empty-text {
  color: var(--text-tertiary);
  font-size: var(--text-sm);
  margin: 0;
}

.logs-list {
  padding: var(--space-2) 0;
}

.log-entry {
  display: grid;
  grid-template-columns: 60px 80px 1fr;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-4);
  border-bottom: 1px solid var(--border-light);
  font-family: ui-monospace, SFMono-Regular, 'Cascadia Code', 'Roboto Mono', monospace;
  font-size: var(--text-xs);
  line-height: 1.4;
  transition: background-color 0.2s ease;
}

.log-entry:hover {
  background: var(--surface-overlay);
}

.log-entry:last-child {
  border-bottom: none;
}

.log-time {
  color: var(--text-tertiary);
  font-weight: 500;
  white-space: nowrap;
}

.log-level {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  font-weight: 600;
  white-space: nowrap;
}

.level-icon {
  font-size: 10px;
}

.level-text {
  font-size: var(--text-xs);
}

.log-message {
  color: var(--text-primary);
  word-break: break-word;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

/* Specific log level styling */
.log-hb {
  opacity: 0.7;
}

.log-hb .log-message {
  color: var(--text-tertiary);
}

.log-recv {
  /* Fresh received items get subtle highlight */
  background: rgba(0, 122, 255, 0.02);
}

.log-sent {
  /* Sent items get subtle highlight */
  background: rgba(52, 199, 89, 0.02);
}

.log-error {
  background: rgba(255, 59, 48, 0.03);
  border-left: 3px solid var(--accent-danger);
  padding-left: calc(var(--space-4) - 3px);
}

.log-warn {
  background: rgba(255, 149, 0, 0.03);
  border-left: 3px solid var(--accent-warning);
  padding-left: calc(var(--space-4) - 3px);
}

/* Scrollbar styling */
.logs-container::-webkit-scrollbar {
  width: 8px;
}

.logs-container::-webkit-scrollbar-track {
  background: transparent;
}

.logs-container::-webkit-scrollbar-thumb {
  background: var(--border-medium);
  border-radius: var(--radius-full);
}

.logs-container::-webkit-scrollbar-thumb:hover {
  background: var(--border-strong);
}

/* Responsive Design */
@media (max-width: 768px) {
  .log-entry {
    grid-template-columns: 50px 60px 1fr;
    gap: var(--space-1);
    padding: var(--space-2) var(--space-3);
    font-size: 10px;
  }
  
  .log-header {
    padding: var(--space-3);
  }
  
  .level-icon {
    display: none;
  }
  
  .level-text {
    font-size: 10px;
  }
}

/* Animation for new log entries */
@keyframes slideInFromTop {
  from {
    opacity: 0;
    transform: translateY(-10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.log-entry {
  animation: slideInFromTop 0.3s ease-out;
}

/* High contrast mode */
@media (prefers-contrast: high) {
  .log-entry {
    border-bottom: 2px solid var(--border-strong);
  }
  
  .log-error {
    border-left-width: 5px;
  }
  
  .log-warn {
    border-left-width: 5px;
  }
}

/* Reduced motion */
@media (prefers-reduced-motion: reduce) {
  .log-entry {
    animation: none;
    transition: none;
  }
}
</style>
