<template>
  <header class="top-bar">
    <!-- Left: Logo & Board Selector -->
    <div class="top-left">
      <div class="app-brand" @click="resetCamera" title="Reset View">
        <Pin class="app-icon" :size="18" />
        <span class="app-title">Pinboard</span>
      </div>

      <div class="board-selector-wrap" ref="dropdownRef">
        <button class="board-dropdown-btn" @click="toggleDropdown">
          <span class="board-name">{{ currentBoardName }}</span>
          <ChevronDown :size="14" class="chevron-icon" :class="{ 'is-open': isDropdownOpen }" />
        </button>

        <!-- Dropdown Menu -->
        <div v-if="isDropdownOpen" class="board-dropdown-menu">
          <div class="dropdown-header">Switch Board</div>
          <div class="board-list">
            <div
              v-for="b in boards"
              :key="b.id"
              class="board-item"
              :class="{ 'is-active': b.id === currentBoardId }"
              @click="selectBoard(b.id)"
            >
              <span class="board-item-name">{{ b.name }}</span>
              <span v-if="b.id === currentBoardId" class="active-dot">●</span>
            </div>
          </div>
          <div class="dropdown-divider"></div>
          <div class="dropdown-actions">
            <button class="action-btn" @click="promptNewBoard">
              <Plus :size="14" />
              <span>New Board</span>
            </button>
            <button class="action-btn" @click="promptRenameBoard">
              <Edit2 :size="14" />
              <span>Rename Board</span>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Right: Search, Undo/Redo, Fit, More, Window Controls -->
    <div class="top-right">
      <button class="search-btn" title="Search board (/)" @click="$emit('open-search')">
        <Search :size="14" class="search-icon" />
        <span class="search-kbd">/</span>
      </button>

      <div class="nav-btn-group">
        <button
          class="nav-btn"
          :disabled="!canUndo"
          title="Undo (Ctrl+Z)"
          @click="$emit('undo')"
        >
          <Undo2 :size="15" />
        </button>
        <button
          class="nav-btn"
          :disabled="!canRedo"
          title="Redo (Ctrl+Shift+Z)"
          @click="$emit('redo')"
        >
          <Redo2 :size="15" />
        </button>
      </div>

      <button class="nav-btn" title="Fit to Viewport (Ctrl+0)" @click="$emit('fit-all')">
        <Maximize2 :size="15" />
      </button>

      <!-- More Menu -->
      <div class="more-wrap" ref="moreMenuRef">
        <button class="nav-btn" title="More Options" @click="isMoreOpen = !isMoreOpen">
          <MoreHorizontal :size="16" />
        </button>
        <div v-if="isMoreOpen" class="more-menu">
          <button class="more-item" @click="toggleAlwaysOnTop">
            <span>Always on Top</span>
            <span class="check-indicator">{{ alwaysOnTop ? '✓' : '' }}</span>
          </button>
          <button class="more-item" @click="$emit('toggle-shortcuts')">
            <span>Keyboard Shortcuts</span>
            <span class="item-kbd">?</span>
          </button>
          <div class="dropdown-divider"></div>
          <button class="more-item text-danger" @click="$emit('clear-board')">
            <span>Clear Current Board</span>
          </button>
        </div>
      </div>

      <!-- Window Controls -->
      <div class="window-controls">
        <button class="win-btn win-min" title="Minimize" @click="minimizeWindow">
          <Minus :size="12" />
        </button>
        <button class="win-btn win-max" title="Maximize" @click="maximizeWindow">
          <Square :size="10" />
        </button>
        <button class="win-btn win-close" title="Close" @click="closeWindow">
          <X :size="12" />
        </button>
      </div>
    </div>
  </header>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';
import {
  Pin,
  ChevronDown,
  Search,
  Undo2,
  Redo2,
  Maximize2,
  MoreHorizontal,
  Minus,
  Square,
  X,
  Plus,
  Edit2,
} from 'lucide-vue-next';
import type { BoardInfo } from '../types';
import { api } from '../utils/api';
import { getCurrentWindow } from '@tauri-apps/api/window';

const props = defineProps<{
  currentBoardId: string;
  currentBoardName: string;
  canUndo: boolean;
  canRedo: boolean;
  alwaysOnTop: boolean;
}>();

const emit = defineEmits<{
  (e: 'switch-board', id: string): void;
  (e: 'board-renamed', name: string): void;
  (e: 'open-search'): void;
  (e: 'undo'): void;
  (e: 'redo'): void;
  (e: 'fit-all'): void;
  (e: 'reset-camera'): void;
  (e: 'toggle-always-on-top'): void;
  (e: 'toggle-shortcuts'): void;
  (e: 'clear-board'): void;
}>();

const boards = ref<BoardInfo[]>([]);
const isDropdownOpen = ref(false);
const isMoreOpen = ref(false);
const dropdownRef = ref<HTMLDivElement | null>(null);
const moreMenuRef = ref<HTMLDivElement | null>(null);

onMounted(async () => {
  loadBoards();
  window.addEventListener('click', onWindowClick);
});

onBeforeUnmount(() => {
  window.removeEventListener('click', onWindowClick);
});

async function loadBoards() {
  boards.value = await api.getBoards();
}

function toggleDropdown(e: MouseEvent) {
  e.stopPropagation();
  isDropdownOpen.value = !isDropdownOpen.value;
  if (isDropdownOpen.value) {
    loadBoards();
  }
}

function onWindowClick(e: MouseEvent) {
  if (dropdownRef.value && !dropdownRef.value.contains(e.target as Node)) {
    isDropdownOpen.value = false;
  }
  if (moreMenuRef.value && !moreMenuRef.value.contains(e.target as Node)) {
    isMoreOpen.value = false;
  }
}

function selectBoard(id: string) {
  emit('switch-board', id);
  isDropdownOpen.value = false;
}

async function promptNewBoard() {
  const name = prompt('Enter new board name:', 'Project Board');
  if (name && name.trim()) {
    const newBoard = await api.createBoard(name.trim());
    await loadBoards();
    emit('switch-board', newBoard.id);
  }
  isDropdownOpen.value = false;
}

async function promptRenameBoard() {
  const name = prompt('Rename board:', props.currentBoardName);
  if (name && name.trim()) {
    await api.renameBoard(props.currentBoardId, name.trim());
    await loadBoards();
    emit('board-renamed', name.trim());
  }
  isDropdownOpen.value = false;
}

function resetCamera() {
  emit('reset-camera');
}

function toggleAlwaysOnTop() {
  emit('toggle-always-on-top');
  isMoreOpen.value = false;
}

// Window control handlers
async function minimizeWindow() {
  try {
    const win = getCurrentWindow();
    await win.minimize();
  } catch (err) {
    console.warn('Window minimize fallback:', err);
  }
}

async function maximizeWindow() {
  try {
    const win = getCurrentWindow();
    await win.toggleMaximize();
  } catch (err) {
    console.warn('Window maximize fallback:', err);
  }
}

async function closeWindow() {
  try {
    const win = getCurrentWindow();
    await win.close();
  } catch (err) {
    console.warn('Window close fallback:', err);
  }
}
</script>

<style scoped>
.top-bar {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 48px;
  background: #0f101a;
  border-bottom: 1px solid #1e2030;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 14px;
  z-index: 10000;
  user-select: none;
  box-sizing: border-box;
}

.top-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.app-brand {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  color: #fff;
  font-weight: 700;
  font-size: 15px;
  letter-spacing: -0.2px;
}

.app-icon {
  color: #c084fc;
  transform: rotate(-35deg);
}

.board-selector-wrap {
  position: relative;
}

.board-dropdown-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  background: transparent;
  border: none;
  color: #94a3b8;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 6px;
  transition: all 0.15s ease;
}

.board-dropdown-btn:hover {
  background: #181926;
  color: #e2e8f0;
}

.chevron-icon {
  transition: transform 0.2s ease;
}

.chevron-icon.is-open {
  transform: rotate(180deg);
}

.board-dropdown-menu {
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  width: 220px;
  background: #151624;
  border: 1px solid #282a40;
  border-radius: 10px;
  box-shadow: 0 12px 28px rgba(0, 0, 0, 0.6);
  padding: 6px;
  z-index: 10001;
}

.dropdown-header {
  font-size: 11px;
  font-weight: 600;
  color: #64748b;
  padding: 4px 8px 6px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.board-list {
  max-height: 180px;
  overflow-y: auto;
}

.board-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 7px 10px;
  border-radius: 6px;
  font-size: 13px;
  color: #cbd5e1;
  cursor: pointer;
  transition: background 0.1s;
}

.board-item:hover {
  background: #202236;
  color: #fff;
}

.board-item.is-active {
  background: rgba(168, 85, 247, 0.18);
  color: #c084fc;
  font-weight: 600;
}

.active-dot {
  font-size: 10px;
  color: #c084fc;
}

.dropdown-divider {
  height: 1px;
  background: #23253a;
  margin: 6px 0;
}

.dropdown-actions {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 6px 8px;
  background: transparent;
  border: none;
  border-radius: 6px;
  color: #94a3b8;
  font-size: 12px;
  cursor: pointer;
  text-align: left;
}

.action-btn:hover {
  background: #202236;
  color: #f1f5f9;
}

/* Top Right Section */
.top-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.search-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  background: #161726;
  border: 1px solid #282a40;
  border-radius: 8px;
  padding: 4px 10px;
  color: #94a3b8;
  cursor: pointer;
  transition: all 0.15s ease;
}

.search-btn:hover {
  background: #202238;
  border-color: #3b3e5e;
  color: #f1f5f9;
}

.search-kbd {
  font-size: 11px;
  font-family: ui-monospace, monospace;
  background: #0f101a;
  padding: 1px 5px;
  border-radius: 4px;
  border: 1px solid #282a40;
  color: #cbd5e1;
}

.nav-btn-group {
  display: flex;
  align-items: center;
  background: #161726;
  border: 1px solid #282a40;
  border-radius: 8px;
  overflow: hidden;
}

.nav-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 28px;
  background: transparent;
  border: none;
  color: #94a3b8;
  cursor: pointer;
  transition: all 0.15s ease;
}

.nav-btn:hover:not(:disabled) {
  background: #23253d;
  color: #fff;
}

.nav-btn:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

.more-wrap {
  position: relative;
}

.more-menu {
  position: absolute;
  top: calc(100% + 6px);
  right: 0;
  width: 200px;
  background: #151624;
  border: 1px solid #282a40;
  border-radius: 10px;
  box-shadow: 0 12px 28px rgba(0, 0, 0, 0.6);
  padding: 6px;
  z-index: 10001;
}

.more-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 7px 10px;
  background: transparent;
  border: none;
  border-radius: 6px;
  color: #cbd5e1;
  font-size: 12px;
  cursor: pointer;
  text-align: left;
}

.more-item:hover {
  background: #202236;
  color: #fff;
}

.more-item.text-danger:hover {
  background: rgba(239, 68, 68, 0.15);
  color: #ef4444;
}

.item-kbd {
  font-size: 11px;
  font-family: ui-monospace, monospace;
  color: #64748b;
}

.check-indicator {
  color: #c084fc;
  font-weight: 700;
}

/* Window Controls */
.window-controls {
  display: flex;
  align-items: center;
  margin-left: 8px;
}

.win-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 28px;
  background: transparent;
  border: none;
  color: #94a3b8;
  cursor: pointer;
  transition: all 0.1s ease;
  border-radius: 4px;
}

.win-btn:hover {
  background: #23253d;
  color: #fff;
}

.win-close:hover {
  background: #dc2626;
  color: #fff;
}
</style>
