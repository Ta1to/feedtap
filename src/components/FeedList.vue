<script setup>
import { computed, reactive } from 'vue';

const props = defineProps({ 
  items: Array, 
  compact: Boolean 
});

function formatDate(dateString) {
  if (!dateString) return null;
  try {
    return new Date(dateString).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  } catch {
    return null;
  }
}

function isRecentItem(item) {
  return Date.now() - (item._arrivalTs || 0) < 30000; // 30 seconds
}

function getDomainFromUrl(url) {
  try {
    return new URL(url).hostname.replace(/^www\./, '');
  } catch {
    return url;
  }
}

function openLink(url) {
  if (url) {
    // Use window.open which works reliably in Tauri
    window.open(url, '_blank', 'noopener,noreferrer');
  }
}

function shareItem(item) {
  if (navigator.share && item.link) {
    navigator.share({
      title: item.title,
      url: item.link
    }).catch(console.warn);
  } else if (item.link) {
    // Fallback: copy to clipboard
    navigator.clipboard.writeText(item.link).then(() => {
      console.log('Link copied to clipboard');
    }).catch(console.warn);
  }
}

// Preview state and loader
const previews = reactive({}); // id -> { loading, error, html, open }

async function togglePreview(item) {
  const cur = previews[item.id];
  if (cur && cur.html && !cur.open) {
    previews[item.id] = { ...cur, open: true };
    return;
  }
  if (cur && cur.open) {
    previews[item.id] = { ...cur, open: false };
    return;
  }
  previews[item.id] = { loading: true, error: null, html: '', open: true };
  try {
    const resp = await fetch(`http://127.0.0.1:8787/preview?url=${encodeURIComponent(item.link)}`);
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    const data = await resp.json();
    // content_preview may be html; render as text fallback
    previews[item.id] = { loading: false, error: null, html: data.content_preview, open: true };
  } catch (e) {
    previews[item.id] = { loading: false, error: String(e), html: '', open: true };
  }
}
</script>

<template>
  <div class="feed-list" :class="{ compact }">
    <transition-group name="feed-item" tag="div" class="items-container">
      <article 
        v-for="item in items" 
        :key="item.id" 
        class="feed-item card"
        :class="{ 
          'card-fresh': isRecentItem(item),
          'compact': compact
        }"
        role="article"
      >
        <!-- Item Header -->
        <div class="item-header">
          <div class="source-info">
            <span class="source-name badge badge-primary">
              {{ item.source?.name || 'Unknown' }}
            </span>
            <span v-if="item.published_at" class="item-date text-tertiary">
              {{ formatDate(item.published_at) }}
            </span>
          </div>
          
          <div class="item-actions">
            <button 
              class="btn btn-ghost btn-sm"
              @click.stop="shareItem(item)"
              title="Share"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8"/>
                <polyline points="16,6 12,2 8,6"/>
                <line x1="12" y1="2" x2="12" y2="15"/>
              </svg>
            </button>
          </div>
        </div>

        <!-- Main Content -->
        <div class="item-content">
          <h3 class="item-title">
            <a 
              :href="item.link" 
              target="_blank" 
              rel="noopener noreferrer"
              class="title-link"
            >
              {{ item.title }}
            </a>
          </h3>
          
          <p v-if="item.summary && !compact" class="item-summary text-secondary">
            {{ item.summary }}
          </p>
          <div class="preview-controls" v-if="!compact">
            <button class="btn btn-outline btn-xs" @click.prevent="togglePreview(item)">
              {{ (previews[item.id]?.open) ? 'Hide preview' : 'Show preview' }}
            </button>
          </div>
        </div>

        <!-- Preview Panel -->
        <div v-if="previews[item.id]?.open && !compact" class="preview-panel">
          <div v-if="previews[item.id]?.loading" class="preview-loading text-tertiary">Loading preview…</div>
          <div v-else-if="previews[item.id]?.error" class="preview-error text-danger">{{ previews[item.id]?.error }}</div>
          <div v-else class="preview-content" v-html="previews[item.id]?.html"></div>
        </div>

        <!-- Footer with external link info -->
        <div v-if="item.link" class="item-footer">
          <div class="external-link">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
              <polyline points="15,3 21,3 21,9"/>
              <line x1="10" y1="14" x2="21" y2="3"/>
            </svg>
            <span class="domain text-tertiary">{{ getDomainFromUrl(item.link) }}</span>
          </div>
        </div>
      </article>
    </transition-group>
  </div>
</template>

<style scoped>
.feed-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.feed-list.compact {
  gap: var(--space-2);
}

.items-container {
  display: flex;
  flex-direction: column;
  gap: inherit;
}

.feed-item {
  padding: var(--space-4);
  border-radius: var(--radius-md);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  overflow: hidden;
}

.feed-item:focus {
  outline: 2px solid var(--accent-primary);
  outline-offset: 2px;
}

.feed-item.compact {
  padding: var(--space-3);
}

.feed-item:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
}

.feed-item.card-fresh {
  border-left: 4px solid var(--accent-primary);
  background: rgba(0, 122, 255, 0.02);
}

.feed-item.card-fresh::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(90deg, var(--accent-primary), transparent);
  opacity: 0.5;
}

/* Item Header */
.item-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--space-3);
}

.feed-item.compact .item-header {
  margin-bottom: var(--space-2);
}

.source-info {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex: 1;
  min-width: 0;
}

.source-name {
  font-size: var(--text-xs);
  font-weight: 600;
}

.item-date {
  font-size: var(--text-xs);
  white-space: nowrap;
}

.item-actions {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  opacity: 0;
  transition: opacity 0.2s ease;
}

.feed-item:hover .item-actions {
  opacity: 1;
}

/* Main Content */
.item-content {
  margin-bottom: var(--space-3);
}

.feed-item.compact .item-content {
  margin-bottom: var(--space-2);
}

.item-title {
  font-size: var(--text-base);
  font-weight: 600;
  line-height: 1.4;
  margin: 0 0 var(--space-2) 0;
}

.title-link {
  color: var(--text-primary);
  text-decoration: none;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  transition: color 0.2s ease;
}

.title-link:hover {
  color: var(--accent-primary);
  text-decoration: none;
}

.feed-item.compact .item-title {
  font-size: var(--text-sm);
  margin-bottom: var(--space-1);
}

.item-summary {
  font-size: var(--text-sm);
  line-height: 1.5;
  margin: 0;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
  opacity: 0.9;
}

.preview-controls {
  margin-top: var(--space-2);
}

.preview-panel {
  margin-top: var(--space-2);
  padding: var(--space-3);
  background: var(--surface-muted);
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-light);
}

.preview-content {
  font-size: var(--text-sm);
  line-height: 1.6;
}

/* Footer */
.item-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-top: var(--space-2);
  border-top: 1px solid var(--border-light);
}

.feed-item.compact .item-footer {
  padding-top: var(--space-1);
}

.external-link {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  font-size: var(--text-xs);
}

.external-link svg {
  opacity: 0.6;
}

.domain {
  font-weight: 500;
}

/* Animations */
.feed-item-enter-from {
  opacity: 0;
  transform: translateY(-10px) scale(0.98);
}

.feed-item-enter-to {
  opacity: 1;
  transform: translateY(0) scale(1);
}

.feed-item-enter-active {
  transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
}

.feed-item-leave-from {
  opacity: 1;
  transform: translateY(0) scale(1);
}

.feed-item-leave-to {
  opacity: 0;
  transform: translateY(-5px) scale(0.95);
}

.feed-item-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.feed-item-move {
  transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

/* Loading state */
@keyframes shimmer {
  0% { background-position: -200px 0; }
  100% { background-position: calc(200px + 100%) 0; }
}

.feed-item.loading {
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.1), transparent);
  background-size: 200px 100%;
  animation: shimmer 1.5s infinite;
}

/* Responsive Design */
@media (max-width: 768px) {
  .feed-list {
    gap: var(--space-3);
  }
  
  .feed-list.compact {
    gap: var(--space-1);
  }
  
  .feed-item {
    padding: var(--space-3);
  }
  
  .feed-item.compact {
    padding: var(--space-2);
  }
  
  .item-header {
    flex-direction: column;
    align-items: flex-start;
    gap: var(--space-1);
  }
  
  .item-actions {
    opacity: 1; /* Always visible on mobile */
  }
  
  .item-title {
    font-size: var(--text-base);
  }
  
  .feed-item.compact .item-title {
    font-size: var(--text-sm);
  }
}

/* High contrast mode support */
@media (prefers-contrast: high) {
  .feed-item {
    border: 2px solid var(--border-strong);
  }
  
  .feed-item.card-fresh {
    border-left: 6px solid var(--accent-primary);
  }
}

/* Reduced motion support */
@media (prefers-reduced-motion: reduce) {
  .feed-item,
  .title-link,
  .item-actions,
  .feed-item-enter-active,
  .feed-item-leave-active,
  .feed-item-move {
    transition: none;
  }
  
  .feed-item:hover {
    transform: none;
  }
}
</style>
