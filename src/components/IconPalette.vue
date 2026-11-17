<template>
  <div class="icon-palette" @mousedown.stop @keydown.esc.stop="$emit('close')">
    <div class="palette-header">
      <div class="palette-title">
        <Boxes :size="15" class="palette-icon" />
        <span>Icon Library</span>
      </div>
      <button class="palette-close-btn" @click="$emit('close')" title="Close (Esc)">✕</button>
    </div>

    <!-- Search input -->
    <div class="palette-search-wrap">
      <Search :size="13" class="search-icon" />
      <input
        ref="searchInputRef"
        v-model="searchQuery"
        type="text"
        class="palette-search-input"
        placeholder="Search icons... (e.g. router, laptop, database)"
      />
      <button v-if="searchQuery" class="clear-search-btn" @click="searchQuery = ''">✕</button>
    </div>

    <!-- Category Tabs -->
    <div class="category-tabs">
      <button
        class="category-tab"
        :class="{ 'is-active': activeCategory === 'all' }"
        @click="activeCategory = 'all'"
      >
        All
      </button>
      <button
        v-for="cat in ICON_CATEGORIES"
        :key="cat"
        class="category-tab"
        :class="{ 'is-active': activeCategory === cat }"
        @click="activeCategory = cat"
      >
        {{ cat }}
      </button>
    </div>

    <!-- Icon Grid -->
    <div class="icon-grid">
      <div v-if="filteredIcons.length === 0" class="no-icons-found">
        No icons found matching "{{ searchQuery }}"
      </div>
      <button
        v-for="icon in filteredIcons"
        :key="icon.id"
        class="icon-grid-item"
        :title="`${icon.name} (${icon.category})`"
        @click="onSelect(icon)"
      >
        <div class="icon-preview">
          <component :is="icon.component" :size="20" />
        </div>
        <span class="icon-name">{{ icon.name }}</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick } from 'vue';
import { Search, Boxes } from 'lucide-vue-next';
import { ICON_REGISTRY, ICON_CATEGORIES, type IconItem, type IconCategory } from '../utils/iconRegistry';

const emit = defineEmits<{
  (e: 'select-icon', icon: IconItem): void;
  (e: 'close'): void;
}>();

const searchQuery = ref('');
const activeCategory = ref<'all' | IconCategory>('all');
const searchInputRef = ref<HTMLInputElement | null>(null);

onMounted(() => {
  nextTick(() => {
    searchInputRef.value?.focus();
  });
});

const filteredIcons = computed(() => {
  const q = searchQuery.value.trim().toLowerCase();
  return ICON_REGISTRY.filter((item) => {
    if (activeCategory.value !== 'all' && item.category !== activeCategory.value) {
      return false;
    }
    if (!q) return true;
    return (
      item.name.toLowerCase().includes(q) ||
      item.id.toLowerCase().includes(q) ||
      item.category.toLowerCase().includes(q) ||
      item.keywords.some((kw) => kw.includes(q))
    );
  });
});

function onSelect(icon: IconItem) {
  emit('select-icon', icon);
  emit('close');
}
</script>

<style scoped>
.icon-palette {
  position: absolute;
  top: 60px;
  left: 68px;
  width: 320px;
  background: #141525;
  border: 1px solid #2a2c42;
  border-radius: 14px;
  padding: 14px;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.75);
  z-index: 10060;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.palette-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.palette-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  font-weight: 700;
  color: #f1f5f9;
}

.palette-icon {
  color: #c084fc;
}

.palette-close-btn {
  background: transparent;
  border: none;
  color: #94a3b8;
  font-size: 14px;
  cursor: pointer;
  padding: 3px 6px;
  border-radius: 4px;
}
.palette-close-btn:hover {
  color: #f1f5f9;
  background: rgba(255, 255, 255, 0.08);
}

.palette-search-wrap {
  position: relative;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 10px;
  color: #64748b;
  pointer-events: none;
}

.palette-search-input {
  width: 100%;
  background: #0d0e17;
  border: 1px solid #27283d;
  border-radius: 8px;
  padding: 7px 28px 7px 30px;
  font-size: 12px;
  color: #f8fafc;
  outline: none;
}
.palette-search-input:focus {
  border-color: #a855f7;
  box-shadow: 0 0 0 2px rgba(168, 85, 247, 0.2);
}

.clear-search-btn {
  position: absolute;
  right: 8px;
  background: transparent;
  border: none;
  color: #64748b;
  cursor: pointer;
  font-size: 11px;
}
.clear-search-btn:hover {
  color: #cbd5e1;
}

.category-tabs {
  display: flex;
  align-items: center;
  gap: 4px;
  overflow-x: auto;
  padding-bottom: 2px;
}
.category-tabs::-webkit-scrollbar {
  display: none;
}

.category-tab {
  background: transparent;
  border: 1px solid #24263b;
  border-radius: 6px;
  color: #94a3b8;
  font-size: 11px;
  padding: 3px 8px;
  white-space: nowrap;
  cursor: pointer;
  transition: all 0.15s ease;
}
.category-tab:hover {
  color: #f1f5f9;
  background: rgba(255, 255, 255, 0.04);
}
.category-tab.is-active {
  background: rgba(168, 85, 247, 0.15);
  border-color: #a855f7;
  color: #c084fc;
  font-weight: 600;
}

.icon-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
  max-height: 240px;
  overflow-y: auto;
  padding-right: 2px;
}

.icon-grid::-webkit-scrollbar {
  width: 4px;
}
.icon-grid::-webkit-scrollbar-thumb {
  background: #2a2c42;
  border-radius: 2px;
}

.no-icons-found {
  grid-column: 1 / -1;
  text-align: center;
  padding: 24px 8px;
  color: #64748b;
  font-size: 12px;
}

.icon-grid-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 5px;
  background: #11121d;
  border: 1px solid #23253a;
  border-radius: 8px;
  padding: 10px 4px 8px;
  cursor: pointer;
  transition: all 0.15s ease;
}
.icon-grid-item:hover {
  background: #1e1b4b;
  border-color: #a855f7;
  transform: translateY(-1px);
}

.icon-preview {
  color: #c084fc;
  display: flex;
  align-items: center;
  justify-content: center;
}

.icon-name {
  font-size: 10px;
  color: #cbd5e1;
  text-align: center;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  width: 100%;
  padding: 0 4px;
}
</style>
