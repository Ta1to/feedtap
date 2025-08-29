<script setup>
const props = defineProps({ sources: Array, active: Array, counts: Object, total: Number });
const emit = defineEmits(["toggle", "clear"]);
</script>

<template>
  <div class="sources">
    <strong>Quellen:</strong>
    <button class="chip" :data-active="active?.length===0" @click="emit('clear')">
      <span class="count">{{ total ?? 0 }}</span>
      Alle
    </button>
    <button
      v-for="s in sources"
      :key="s.id"
      class="chip"
      :data-active="active?.includes(s.id)"
      @click="emit('toggle', s.id)"
    >
      <span class="count">{{ counts?.[s.id] ?? 0 }}</span>
      {{ s.name }}
    </button>
  </div>
</template>

<style scoped>
.sources { margin: 8px 0 16px; display: flex; align-items: center; gap: 6px; flex-wrap: wrap; }
.chip { background: var(--chip-bg); color: var(--chip-fg); padding: 4px 10px; border-radius: 999px; font-size: 0.85em; border: 1px solid var(--chip-brd); cursor: pointer; display: inline-flex; align-items: center; gap: 8px; }
.chip[data-active="true"] { background: color-mix(in srgb, var(--primary) 18%, var(--chip-bg)); color: var(--fg); border-color: color-mix(in srgb, var(--primary) 40%, var(--chip-brd)); }
.count { background: color-mix(in srgb, var(--primary) 12%, var(--surface)); color: var(--fg); border: 1px solid var(--chip-brd); border-radius: 999px; padding: 1px 6px; font-weight: 600; font-size: .85em; }
</style>
