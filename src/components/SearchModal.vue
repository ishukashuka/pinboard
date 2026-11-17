<template>
  <div class="search-backdrop" @click.self="$emit('close')">
    <div class="search-modal" @keydown.esc.stop="$emit('close')">
      <!-- Search input header -->
      <div class="search-input-wrapper">
        <Search :size="16" class="search-icon" />
        <input
          ref="inputRef"
          v-model="query"
          type="text"
          class="search-input"
          placeholder="Search notes, code, links, images, connectors..."
          @keydown.down.prevent="selectNext"
          @keydown.up.prevent="selectPrev"
          @keydown.ctrl.n.prevent="selectNext"
          @keydown.ctrl.p.prevent="selectPrev"
          @keydown.enter.prevent="confirmSelection"
        />
        <span v-if="filteredResults.length > 0" class="result-count">
          {{ filteredResults.length }} {{ filteredResults.length === 1 ? 'match' : 'matches' }}
        </span>
        <button class="modal-close-btn" @click="$emit('close')">✕</button>
      </div>

      <!-- Results list -->
      <div ref="listRef" class="search-results-list">
        <div v-if="filteredResults.length === 0" class="no-results">
          <span v-if="query">No matching objects or connectors found</span>
          <span v-else class="search-tip">Type to search anything on the canvas...</span>
        </div>

        <div
          v-for="(item, idx) in filteredResults"
          :key="`${item.isConnector ? 'conn-' : 'obj-'}${item.id}`"
          class="result-item"
          :class="{ 'is-active': idx === activeIndex }"
          @mouseenter="activeIndex = idx"
          @click="selectItem(item)"
        >
          <div class="result-icon-badge" :class="`kind-${item.kind}`">
            <component :is="getResultIcon(item)" :size="14" />
          </div>

          <div class="result-details">
            <div class="result-title-row">
              <span class="result-title">{{ item.title }}</span>
              <span class="result-match-field">{{ item.matchField }}</span>
            </div>
            <div v-if="item.snippet" class="result-snippet">{{ item.snippet }}</div>
          </div>

          <div class="result-coords">
            {{ Math.round(item.x) }}, {{ Math.round(item.y) }}
          </div>
        </div>
      </div>

      <!-- Footer tips -->
      <div class="search-footer">
        <div class="footer-tip">
          <kbd>↑</kbd><kbd>↓</kbd> or <kbd>Ctrl+N</kbd>/<kbd>P</kbd> to navigate
        </div>
        <div class="footer-tip">
          <kbd>Enter</kbd> jump to object
        </div>
        <div class="footer-tip">
          <kbd>Esc</kbd> close
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick } from 'vue';
import {
  Search,
  FileText,
  Image as ImageIcon,
  Link as LinkIcon,
  Code2,
  Spline,
  Boxes,
} from 'lucide-vue-next';
import type { BoardConnector, BoardObject } from '../types';

export interface ExtendedSearchResult {
  id: string;
  kind: string;
  title: string;
  snippet: string;
  x: number;
  y: number;
  width: number;
  height: number;
  matchField: string;
  isConnector?: boolean;
}

const props = defineProps<{
  objects: BoardObject[];
  connectors: BoardConnector[];
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'jump-to', result: ExtendedSearchResult): void;
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

const filteredResults = computed<ExtendedSearchResult[]>(() => {
  const q = query.value.trim().toLowerCase();
  const results: ExtendedSearchResult[] = [];

  // Search Objects
  for (const obj of props.objects) {
    const title = (obj.title || '').toLowerCase();
    const content = (obj.content || '').toLowerCase();
    const origFile = (obj.meta?.original_filename || '').toLowerCase();
    const iconLabel = (obj.meta?.icon_label || '').toLowerCase();

    if (!q) {
      // Recent overview items
      let displayTitle = obj.title;
      let snippet = obj.content;
      if (obj.kind === 'image') {
        displayTitle = obj.title || obj.meta?.original_filename || 'Image';
        snippet = obj.meta?.original_filename || 'Embedded Image';
      } else if (obj.kind === 'icon') {
        displayTitle = obj.title || obj.meta?.icon_label || 'Icon';
        snippet = obj.meta?.icon_category || 'Symbol';
      }
      results.push({
        id: obj.id,
        kind: obj.kind,
        title: displayTitle || `${obj.kind.toUpperCase()}`,
        snippet: snippet ? snippet.slice(0, 100).replace(/\n/g, ' ') : '',
        x: obj.x,
        y: obj.y,
        width: obj.width,
        height: obj.height,
        matchField: 'Overview',
      });
      continue;
    }

    if (title.includes(q)) {
      results.push({
        id: obj.id,
        kind: obj.kind,
        title: obj.title || `${obj.kind.toUpperCase()}`,
        snippet: obj.content ? obj.content.slice(0, 100).replace(/\n/g, ' ') : '',
        x: obj.x,
        y: obj.y,
        width: obj.width,
        height: obj.height,
        matchField: 'Title',
      });
    } else if (content.includes(q)) {
      results.push({
        id: obj.id,
        kind: obj.kind,
        title: obj.title || `${obj.kind.toUpperCase()}`,
        snippet: obj.content ? obj.content.slice(0, 100).replace(/\n/g, ' ') : '',
        x: obj.x,
        y: obj.y,
        width: obj.width,
        height: obj.height,
        matchField: obj.kind === 'url' ? 'URL' : 'Content',
      });
    } else if (origFile.includes(q)) {
      results.push({
        id: obj.id,
        kind: obj.kind,
        title: obj.title || obj.meta?.original_filename || 'Image',
        snippet: obj.meta?.original_filename || '',
        x: obj.x,
        y: obj.y,
        width: obj.width,
        height: obj.height,
        matchField: 'Filename',
      });
    } else if (iconLabel.includes(q)) {
      results.push({
        id: obj.id,
        kind: obj.kind,
        title: obj.title || obj.meta?.icon_label || 'Icon',
        snippet: obj.meta?.icon_category || '',
        x: obj.x,
        y: obj.y,
        width: obj.width,
        height: obj.height,
        matchField: 'Icon',
      });
    }
  }

  // Search Connectors (Labels)
  for (const conn of props.connectors) {
    if (conn.label) {
      const labelLower = conn.label.toLowerCase();
      if (!q || labelLower.includes(q)) {
        results.push({
          id: conn.id,
          kind: 'connector',
          title: conn.label,
          snippet: `Connector (${conn.line_type}, ${conn.color})`,
          x: (conn.from_x + conn.to_x) / 2,
          y: (conn.from_y + conn.to_y) / 2,
          width: 60,
          height: 30,
          matchField: 'Connector Label',
          isConnector: true,
        });
      }
    }
  }

  return results.slice(0, 25);
});

function getResultIcon(item: ExtendedSearchResult) {
  if (item.isConnector || item.kind === 'connector') return Spline;
  if (item.kind === 'image') return ImageIcon;
  if (item.kind === 'url') return LinkIcon;
  if (item.kind === 'code') return Code2;
  if (item.kind === 'icon') return Boxes;
  return FileText;
}

function selectNext() {
  if (filteredResults.value.length === 0) return;
  activeIndex.value = (activeIndex.value + 1) % filteredResults.value.length;
  scrollToActive();
}

function selectPrev() {
  if (filteredResults.value.length === 0) return;
  activeIndex.value = (activeIndex.value - 1 + filteredResults.value.length) % filteredResults.value.length;
  scrollToActive();
}

function scrollToActive() {
  nextTick(() => {
    const list = listRef.value;
    if (!list) return;
    const activeEl = list.children[activeIndex.value] as HTMLElement;
    if (activeEl) {
      activeEl.scrollIntoView({ block: 'nearest' });
    }
  });
}

function confirmSelection() {
  if (filteredResults.value.length === 0) return;
  const target = filteredResults.value[activeIndex.value];
  if (target) {
    selectItem(target);
  }
}

function selectItem(item: ExtendedSearchResult) {
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
  z-index: 10090;
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding-top: 100px;
}

.search-modal {
  width: 580px;
  max-width: 90vw;
  background: #141525;
  border: 1px solid #2a2c42;
  border-radius: 14px;
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.8);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.search-input-wrapper {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  border-bottom: 1px solid #23253a;
  background: #10111f;
}

.search-icon {
  color: #c084fc;
}

.search-input {
  flex: 1;
  background: transparent;
  border: none;
  font-size: 14px;
  color: #f8fafc;
  outline: none;
}
.search-input::placeholder {
  color: #64748b;
}

.result-count {
  font-size: 11px;
  color: #94a3b8;
  background: #1c1d30;
  padding: 2px 8px;
  border-radius: 6px;
}

.modal-close-btn {
  background: transparent;
  border: none;
  color: #94a3b8;
  font-size: 14px;
  cursor: pointer;
  padding: 3px 6px;
  border-radius: 4px;
}
.modal-close-btn:hover {
  color: #f1f5f9;
  background: rgba(255, 255, 255, 0.08);
}

.search-results-list {
  max-height: 380px;
  overflow-y: auto;
  padding: 8px;
}
.search-results-list::-webkit-scrollbar {
  width: 5px;
}
.search-results-list::-webkit-scrollbar-thumb {
  background: #2a2c42;
  border-radius: 3px;
}

.no-results {
  text-align: center;
  padding: 32px 16px;
  color: #64748b;
  font-size: 13px;
}

.result-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s ease;
  border: 1px solid transparent;
}
.result-item:hover,
.result-item.is-active {
  background: rgba(168, 85, 247, 0.12);
  border-color: rgba(168, 85, 247, 0.35);
}

.result-icon-badge {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: #1c1d30;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #c084fc;
}

.result-details {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.result-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.result-title {
  font-size: 13px;
  font-weight: 600;
  color: #f1f5f9;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.result-match-field {
  font-size: 10px;
  color: #a855f7;
  background: rgba(168, 85, 247, 0.12);
  padding: 1px 6px;
  border-radius: 4px;
}

.result-snippet {
  font-size: 11px;
  color: #94a3b8;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.result-coords {
  font-size: 11px;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  color: #64748b;
}

.search-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  background: #10111f;
  border-top: 1px solid #23253a;
  font-size: 11px;
  color: #64748b;
}

.footer-tip {
  display: flex;
  align-items: center;
  gap: 4px;
}

kbd {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  background: #1b1d30;
  border: 1px solid #2b2e45;
  border-radius: 4px;
  padding: 1px 5px;
  color: #cbd5e1;
  font-size: 10px;
}
</style>
