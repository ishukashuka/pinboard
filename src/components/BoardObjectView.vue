<template>
  <div
    class="board-object"
    :class="{
      'is-selected': isSelected,
      'is-editing': isEditing,
      'kind-text': object.kind === 'text',
      'kind-image': object.kind === 'image',
      'kind-url': object.kind === 'url',
    }"
    :style="objectStyle"
    @mousedown.stop="onMouseDown"
    @dblclick.stop="onDoubleClick"
  >
    <!-- Object Header / Label Bar (only visible when named, hovered or selected) -->
    <div v-if="object.title || isSelected" class="object-header" @mousedown.stop="onHeaderMouseDown">
      <div v-if="editingTitle" class="title-edit-wrap" @mousedown.stop>
        <input
          ref="titleInputRef"
          v-model="titleDraft"
          class="title-input"
          @keydown.enter="saveTitle"
          @keydown.esc="cancelTitle"
          @blur="saveTitle"
        />
      </div>
      <div v-else class="object-title" @dblclick.stop="startEditTitle">
        <span class="kind-tag">{{ object.kind.toUpperCase() }}</span>
        <span class="title-text">{{ object.title || 'Untitled' }}</span>
      </div>

      <div class="header-actions" @mousedown.stop>
        <button
          v-if="object.kind === 'url'"
          class="header-btn"
          title="Open in Browser"
          @click="openUrl"
        >
          ↗
        </button>
        <button
          class="header-btn"
          :title="object.kind === 'image' ? 'Copy Image to Clipboard' : 'Copy Text to Clipboard'"
          @click="$emit('copy-content', object)"
        >
          📋
        </button>
        <button
          class="header-btn btn-delete"
          title="Delete Object"
          @click="$emit('delete', object.id)"
        >
          ✕
        </button>
      </div>
    </div>

    <!-- Object Content Area -->
    <div class="object-body">
      <!-- TEXT OBJECT -->
      <template v-if="object.kind === 'text'">
        <div v-if="isEditing" class="editor-wrapper" @mousedown.stop>
          <CodeMirrorEditor
            :model-value="object.content"
            :font-size="object.meta?.font_size || 14"
            :font-family="object.meta?.font_family || 'mono'"
            @update:model-value="onContentChange"
            @exit="$emit('exit-editing')"
            @save="$emit('save-request')"
          />
        </div>
        <div v-else class="text-preview" :class="{ 'font-sans': object.meta?.font_family === 'sans' }">
          <div class="text-content">{{ object.content || 'Empty text. Press `i` to edit.' }}</div>
          <div class="text-edit-hint">Double-click or press `i` to edit</div>
        </div>
      </template>

      <!-- IMAGE OBJECT -->
      <template v-else-if="object.kind === 'image'">
        <div class="image-wrapper">
          <img
            :src="object.content"
            :alt="object.title || 'Image'"
            draggable="false"
            class="board-image"
          />
        </div>
      </template>

      <!-- URL OBJECT -->
      <template v-else-if="object.kind === 'url'">
        <div class="url-card">
          <div class="url-icon">🌐</div>
          <div class="url-info">
            <div class="url-title">{{ object.title || object.content }}</div>
            <div class="url-link" @click.stop="openUrl">{{ object.content }}</div>
          </div>
        </div>
      </template>
    </div>

    <!-- Resize Handles (visible only when selected and not editing) -->
    <template v-if="isSelected && !isEditing">
      <div
        v-for="handle in handles"
        :key="handle"
        :class="['resize-handle', `handle-${handle}`]"
        @mousedown.stop="onHandleMouseDown(handle, $event)"
      ></div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, nextTick } from 'vue';
import type { BoardObject } from '../types';
import CodeMirrorEditor from './CodeMirrorEditor.vue';

const props = defineProps<{
  object: BoardObject;
  isSelected: boolean;
  isEditing: boolean;
  zoom: number;
}>();

const emit = defineEmits<{
  (e: 'select', id: string, multi: boolean): void;
  (e: 'start-drag', id: string, event: MouseEvent): void;
  (e: 'start-resize', id: string, handle: string, event: MouseEvent): void;
  (e: 'start-editing', id: string): void;
  (e: 'exit-editing'): void;
  (e: 'update-content', id: string, content: string): void;
  (e: 'update-title', id: string, title: string): void;
  (e: 'copy-content', object: BoardObject): void;
  (e: 'delete', id: string): void;
  (e: 'save-request'): void;
}>();

const handles = ['nw', 'n', 'ne', 'e', 'se', 's', 'sw', 'w'];

const editingTitle = ref(false);
const titleDraft = ref('');
const titleInputRef = ref<HTMLInputElement | null>(null);

const objectStyle = computed(() => ({
  transform: `translate3d(${props.object.x}px, ${props.object.y}px, 0)`,
  width: `${props.object.width}px`,
  height: `${props.object.height}px`,
  zIndex: props.object.z_index,
}));

function onMouseDown(e: MouseEvent) {
  if (props.isEditing) return;
  const isMulti = e.shiftKey || e.ctrlKey || e.metaKey;
  emit('select', props.object.id, isMulti);
  emit('start-drag', props.object.id, e);
}

function onHeaderMouseDown(e: MouseEvent) {
  onMouseDown(e);
}

function onDoubleClick() {
  if (props.object.kind === 'text') {
    emit('start-editing', props.object.id);
  } else if (props.object.kind === 'image') {
    emit('copy-content', props.object);
  } else if (props.object.kind === 'url') {
    openUrl();
  }
}

function onHandleMouseDown(handle: string, e: MouseEvent) {
  emit('start-resize', props.object.id, handle, e);
}

function onContentChange(newContent: string) {
  emit('update-content', props.object.id, newContent);
}

function startEditTitle() {
  titleDraft.value = props.object.title || '';
  editingTitle.value = true;
  nextTick(() => {
    titleInputRef.value?.focus();
    titleInputRef.value?.select();
  });
}

function saveTitle() {
  if (editingTitle.value) {
    emit('update-title', props.object.id, titleDraft.value.trim());
    editingTitle.value = false;
  }
}

function cancelTitle() {
  editingTitle.value = false;
}

function openUrl() {
  if (props.object.content) {
    const url = props.object.content.startsWith('http')
      ? props.object.content
      : `https://${props.object.content}`;
    window.open(url, '_blank');
  }
}
</script>

<style scoped>
.board-object {
  position: absolute;
  top: 0;
  left: 0;
  display: flex;
  flex-direction: column;
  background-color: #1e1e24;
  border: 1px solid #2d2d38;
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  box-sizing: border-box;
  transition: box-shadow 0.15s ease, border-color 0.15s ease;
  overflow: visible;
}

.board-object:hover {
  border-color: #4b4b5c;
}

.board-object.is-selected {
  border-color: #a855f7;
  box-shadow: 0 0 0 2px rgba(168, 85, 247, 0.3), 0 8px 24px rgba(0, 0, 0, 0.5);
}

.board-object.is-editing {
  border-color: #38bdf8;
  box-shadow: 0 0 0 2px rgba(56, 189, 248, 0.35), 0 8px 30px rgba(0, 0, 0, 0.6);
}

.object-header {
  height: 28px;
  min-height: 28px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 8px;
  background: #18181f;
  border-top-left-radius: 7px;
  border-top-right-radius: 7px;
  border-bottom: 1px solid #2d2d38;
  cursor: grab;
  font-size: 11px;
}

.object-header:active {
  cursor: grabbing;
}

.object-title {
  display: flex;
  align-items: center;
  gap: 6px;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
  cursor: text;
}

.kind-tag {
  font-size: 9px;
  font-weight: 700;
  color: #a1a1aa;
  background: #27272a;
  padding: 1px 4px;
  border-radius: 3px;
}

.title-text {
  font-weight: 500;
  color: #d4d4d8;
}

.title-edit-wrap {
  flex: 1;
  margin-right: 8px;
}

.title-input {
  width: 100%;
  background: #27272a;
  border: 1px solid #a855f7;
  color: #fff;
  border-radius: 3px;
  padding: 1px 6px;
  font-size: 11px;
  outline: none;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 4px;
}

.header-btn {
  background: transparent;
  border: none;
  color: #a1a1aa;
  cursor: pointer;
  padding: 2px 4px;
  font-size: 11px;
  border-radius: 3px;
  line-height: 1;
}

.header-btn:hover {
  background: #27272a;
  color: #f4f4f5;
}

.header-btn.btn-delete:hover {
  background: rgba(239, 68, 68, 0.2);
  color: #ef4444;
}

.object-body {
  flex: 1;
  width: 100%;
  height: calc(100% - 28px);
  position: relative;
  overflow: hidden;
}

/* Image Object */
.kind-image .object-body {
  background: #121214;
}

.image-wrapper {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  user-select: none;
}

.board-image {
  width: 100%;
  height: 100%;
  object-fit: contain;
  display: block;
  pointer-events: none;
}

/* Text Object */
.editor-wrapper {
  width: 100%;
  height: 100%;
}

.text-preview {
  width: 100%;
  height: 100%;
  padding: 12px;
  box-sizing: border-box;
  overflow-y: auto;
  user-select: text;
  cursor: text;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 13px;
  line-height: 1.6;
  color: #e4e4e7;
  display: flex;
  flex-direction: column;
}

.text-preview.font-sans {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
}

.text-content {
  flex: 1;
  white-space: pre-wrap;
  word-break: break-word;
}

.text-edit-hint {
  font-size: 10px;
  color: #52525b;
  margin-top: 8px;
  user-select: none;
  font-style: italic;
}

/* URL Object */
.url-card {
  padding: 16px;
  display: flex;
  gap: 12px;
  align-items: center;
  height: 100%;
  box-sizing: border-box;
}

.url-icon {
  font-size: 24px;
}

.url-info {
  flex: 1;
  overflow: hidden;
}

.url-title {
  font-weight: 600;
  font-size: 14px;
  color: #f4f4f5;
  margin-bottom: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.url-link {
  font-size: 12px;
  color: #38bdf8;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  cursor: pointer;
  text-decoration: underline;
}

/* Resize Handles */
.resize-handle {
  position: absolute;
  width: 10px;
  height: 10px;
  background-color: #a855f7;
  border: 1px solid #ffffff;
  border-radius: 2px;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.4);
  z-index: 99;
}

.handle-nw { top: -5px; left: -5px; cursor: nwse-resize; }
.handle-n  { top: -5px; left: calc(50% - 5px); cursor: ns-resize; }
.handle-ne { top: -5px; right: -5px; cursor: nesw-resize; }
.handle-e  { top: calc(50% - 5px); right: -5px; cursor: ew-resize; }
.handle-se { bottom: -5px; right: -5px; cursor: nwse-resize; }
.handle-s  { bottom: -5px; left: calc(50% - 5px); cursor: ns-resize; }
.handle-sw { bottom: -5px; left: -5px; cursor: nesw-resize; }
.handle-w  { top: calc(50% - 5px); left: -5px; cursor: ew-resize; }
</style>
