<template>
  <div class="search-backdrop" @click.self="emit('close')">
    <div class="search-modal" @keydown.stop>
      <div class="search-input-wrapper">
        <span class="search-icon">🔍</span>
        <input
          ref="inputRef"
          v-model="query"
          type="text"
          class="search-input"
          placeholder="Search text, titles, filenames, URLs... (↑/↓ to navigate, Enter to jump, Esc to exit)"
          @keydown.down.prevent="selectNext"
          @keydown.up.prevent="selectPrev"
          @keydown.ctrl.n.prevent="selectNext"
          @keydown.ctrl.p.prevent="selectPrev"
          @keydown.enter.prevent="confirmSelection"
          @keydown.esc.prevent="emit('close')"
        />
        <span v-if="query" class="result-count">{{ filteredResults.length }} results</span>
      </div>

      <div class="search-results-list" ref="listRef">
        <div v-if="filteredResults.length === 0" class="no-results">
          <span v-if="query">No matching objects found</span>
          <span v-else class="search-tip">Type to search anything on the canvas...</span>
        </div>

        <div
          v-for="(item, idx) in filteredResults"
          :key="item.id"
          class="result-item"
          :class="{ 'is-active': idx === activeIndex }"
          @mouseenter="activeIndex = idx"
          @click="selectItem(item)"
        >
          <div class="result-icon-badge" :class="`kind-${item.kind}`">
            <span v-if="item.kind === 'text'">TXT</span>
            <span v-else-if="item.kind === 'image'">IMG</span>
            <span v-else>URL</span>
          </div>

          <div class="result-details">
            <div class="result-title-row">
              <span class="result-title">{{ item.title || item.kind.toUpperCase() }}</span>
              <span class="result-match-field">in {{ item.matchField }}</span>
            </div>
            <div class="result-snippet">{{ item.snippet }}</div>
          </div>

          <div class="result-coords">
            {{ Math.round(item.x) }}, {{ Math.round(item.y) }}
          </div>
        </div>
      </div>

      <div class="search-footer">
        <span><kbd>↑</kbd><kbd>↓</kbd> or <kbd>Ctrl+N</kbd>/<kbd>P</kbd> to navigate</span>
        <span><kbd>Enter</kbd> jump to object</span>
        <span><kbd>Esc</kbd> close</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick, watch } from 'vue';
import type { BoardObject, SearchResult } from '../types';

const props = defineProps<{
  objects: BoardObject[];
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'jump-to', result: SearchResult): void;
}>();

const query = ref('');
const activeIndex = ref(0);
const inputRef = ref<HTMLInputElement | null>(null);
const listRef = ref<HTMLDivElement | null>(null);

onMounted(() => {
  nextTick(() => {
    inputRef.value?.focus();
  });
});

const filteredResults = computed<SearchResult[]>(() => {
  const q = query.value.trim().toLowerCase();
  if (!q) {
    // Show first 15 objects as recent/overview
    return props.objects.slice(0, 15).map((obj) => ({
      id: obj.id,
      kind: obj.kind,
      title: obj.title || `${obj.kind.toUpperCase()} Object`,
      snippet: obj.kind === 'image'
        ? (obj.meta?.original_filename || 'Embedded Image')
        : (obj.content ? obj.content.slice(0, 120).replace(/\n/g, ' ') : ''),
      x: obj.x,
      y: obj.y,
      width: obj.width,
      height: obj.height,
      matchField: 'canvas',
    }));
  }

  const results: SearchResult[] = [];

  for (const obj of props.objects) {
    let matched = false;
    let matchField = '';
    let snippet = '';

    // 1. Check title
    if (obj.title && obj.title.toLowerCase().includes(q)) {
      matched = true;
      matchField = 'title';
      snippet = obj.title;
    }

    // 2. Check content (text, url)
    if (!matched && obj.content && obj.content.toLowerCase().includes(q)) {
      matched = true;
      matchField = obj.kind === 'url' ? 'URL' : 'content';
      const idx = obj.content.toLowerCase().indexOf(q);
      const start = Math.max(0, idx - 30);
      const end = Math.min(obj.content.length, idx + 70);
      snippet = (start > 0 ? '...' : '') + obj.content.slice(start, end).replace(/\n/g, ' ') + (end < obj.content.length ? '...' : '');
    }

    // 3. Check meta (filename, labels)
    if (!matched && obj.meta) {
      if (obj.meta.original_filename && obj.meta.original_filename.toLowerCase().includes(q)) {
        matched = true;
        matchField = 'filename';
        snippet = obj.meta.original_filename;
      }
    }

    if (matched) {
      results.push({
        id: obj.id,
        kind: obj.kind,
        title: obj.title || (obj.kind === 'image' ? (obj.meta?.original_filename || 'Image') : 'Text Block'),
        snippet: snippet || (obj.kind === 'image' ? 'Image object' : obj.content.slice(0, 80)),
        x: obj.x,
        y: obj.y,
        width: obj.width,
        height: obj.height,
        matchField,
      });
    }
  }

  return results;
});

watch(filteredResults, () => {
  activeIndex.value = 0;
});

function selectNext() {
  if (filteredResults.value.length === 0) return;
  activeIndex.value = (activeIndex.value + 1) % filteredResults.value.length;
  scrollActiveIntoView();
}

function selectPrev() {
  if (filteredResults.value.length === 0) return;
  activeIndex.value = (activeIndex.value - 1 + filteredResults.value.length) % filteredResults.value.length;
  scrollActiveIntoView();
}

function scrollActiveIntoView() {
  nextTick(() => {
    if (!listRef.value) return;
    const activeEl = listRef.value.querySelector('.result-item.is-active') as HTMLElement;
    if (activeEl) {
      activeEl.scrollIntoView({ block: 'nearest' });
    }
  });
}

function confirmSelection() {
  if (filteredResults.value.length > 0) {
    const item = filteredResults.value[activeIndex.value];
    selectItem(item);
  }
}

function selectItem(item: SearchResult) {
  emit('jump-to', item);
  emit('close');
}
</script>

<style scoped>
.search-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.65);
  backdrop-filter: blur(4px);
  z-index: 9999;
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding-top: 12vh;
}

.search-modal {
  width: 640px;
  max-width: 90vw;
  background: #18181b;
  border: 1px solid #3f3f46;
  border-radius: 12px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.7);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  animation: fadeIn 0.12s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(-8px) scale(0.98); }
  to { opacity: 1; transform: translateY(0) scale(1); }
}

.search-input-wrapper {
  display: flex;
  align-items: center;
  padding: 14px 16px;
  border-bottom: 1px solid #27272a;
  background: #202025;
  gap: 12px;
}

.search-icon {
  font-size: 16px;
  opacity: 0.6;
}

.search-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  font-size: 15px;
  color: #f4f4f5;
  font-family: inherit;
}

.search-input::placeholder {
  color: #71717a;
}

.result-count {
  font-size: 11px;
  color: #a1a1aa;
  background: #27272a;
  padding: 2px 8px;
  border-radius: 4px;
}

.search-results-list {
  max-height: 380px;
  overflow-y: auto;
  padding: 8px;
}

.no-results {
  padding: 32px 16px;
  text-align: center;
  color: #71717a;
  font-size: 14px;
}

.search-tip {
  font-size: 13px;
  color: #52525b;
}

.result-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.1s ease;
  user-select: none;
}

.result-item:hover,
.result-item.is-active {
  background: #27272a;
}

.result-icon-badge {
  font-size: 10px;
  font-weight: 700;
  padding: 3px 6px;
  border-radius: 4px;
  letter-spacing: 0.5px;
  flex-shrink: 0;
}

.result-icon-badge.kind-text {
  background: rgba(56, 189, 248, 0.2);
  color: #38bdf8;
}

.result-icon-badge.kind-image {
  background: rgba(168, 85, 247, 0.2);
  color: #c084fc;
}

.result-icon-badge.kind-url {
  background: rgba(74, 222, 128, 0.2);
  color: #4ade80;
}

.result-details {
  flex: 1;
  overflow: hidden;
}

.result-title-row {
  display: flex;
  align-items: baseline;
  gap: 8px;
}

.result-title {
  font-weight: 600;
  font-size: 13px;
  color: #f4f4f5;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.result-match-field {
  font-size: 11px;
  color: #71717a;
}

.result-snippet {
  font-size: 12px;
  color: #a1a1aa;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-top: 2px;
  font-family: ui-monospace, SFMono-Regular, monospace;
}

.result-coords {
  font-size: 11px;
  color: #52525b;
  font-family: ui-monospace, SFMono-Regular, monospace;
}

.search-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 16px;
  background: #141417;
  border-top: 1px solid #27272a;
  font-size: 11px;
  color: #71717a;
}

kbd {
  background: #27272a;
  color: #d4d4d8;
  padding: 1px 5px;
  border-radius: 3px;
  font-size: 10px;
  font-family: ui-monospace, SFMono-Regular, monospace;
  margin: 0 2px;
}
</style>
