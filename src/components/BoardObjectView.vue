<template>
  <div
    class="board-object"
    :class="{
      'is-selected': isSelected,
      'is-editing': isEditing,
      'connector-mode': isConnectorTool,
      'kind-image': object.kind === 'image',
      'kind-text': object.kind === 'text',
      'kind-code': isCodeSnippet,
      'kind-url': object.kind === 'url',
      'kind-icon': object.kind === 'icon',
      'kind-diagram': object.kind === 'diagram_node',
    }"
    :style="objectStyle"
    @mouseenter="isHovered = true"
    @mouseleave="isHovered = false"
    @mousedown="onMouseDown"
    @dblclick="onDoubleClick"
  >
    <!-- 1. IMAGE OBJECT (Clean, transparent, borderless when unselected) -->
    <template v-if="object.kind === 'image'">
      <div class="image-container">
        <!-- Floating title pill when selected or hovering -->
        <div v-if="isSelected || isHovered" class="image-title-pill" @mousedown.stop>
          <input
            v-if="isEditingImageTitle"
            ref="imageTitleInputRef"
            v-model="imageTitleDraft"
            class="image-title-input"
            @blur="saveImageTitle"
            @keydown.enter.stop="saveImageTitle"
            @keydown.esc.stop="cancelEditImageTitle"
          />
          <div v-else class="image-title-text" @dblclick.stop="startEditImageTitle">
            <span class="img-name">{{ object.title || 'Image' }}</span>
            <button class="image-rename-btn" title="Rename Image" @click.stop="startEditImageTitle">
              <Pencil :size="10" />
            </button>
          </div>
        </div>

        <img
          :src="object.content"
          :alt="object.title || 'Image'"
          draggable="false"
          class="board-image"
        />
      </div>
    </template>

    <!-- 2. ICON OBJECT (Scalable vector symbol, clean card, no hardcoded IP) -->
    <template v-else-if="object.kind === 'icon'">
      <div class="icon-node-card">
        <component
          :is="resolvedIconComponent"
          :size="Math.max(20, Math.min(object.width * 0.5, 48))"
          class="icon-glyph"
        />
        <div v-if="object.title" class="icon-label-pill" @dblclick.stop="startEditIconTitle">
          <input
            v-if="isEditingIconTitle"
            ref="iconTitleInputRef"
            v-model="iconTitleDraft"
            class="icon-title-input"
            @blur="saveIconTitle"
            @keydown.enter.stop="saveIconTitle"
            @keydown.esc.stop="cancelEditIconTitle"
            @mousedown.stop
          />
          <span v-else>{{ object.title }}</span>
        </div>
      </div>
    </template>

    <!-- 3. CODE SNIPPET OBJECT -->
    <template v-else-if="isCodeSnippet">
      <div class="code-card">
        <div class="code-header">
          <div class="code-header-left">
            <Code2 :size="14" class="code-icon" />
            <span class="code-title">{{ object.title || 'Snippet' }}</span>
          </div>
          <button class="code-copy-btn" title="Copy Code" @click.stop="$emit('copy-content', object)">
            <Copy :size="13" />
          </button>
        </div>
        <div class="code-body">
          <div class="code-lines">
            <span v-for="n in codeLineCount" :key="n" class="line-num">{{ n }}</span>
          </div>
          <pre class="code-pre"><code>{{ object.content }}</code></pre>
        </div>
        <div class="code-footer">
          <span class="tag-pill">#dev</span>
        </div>
      </div>
    </template>

    <!-- 4. TEXT / MARKDOWN NOTE OBJECT -->
    <template v-else-if="object.kind === 'text'">
      <div class="note-card">
        <div class="note-header" @mousedown="onHeaderMouseDown">
          <div class="note-header-left">
            <BookOpen :size="14" class="note-icon" />
            <span class="note-title" @dblclick.stop="startEditTitle">
              {{ object.title || 'Note' }}
            </span>
          </div>
          <button
            class="note-action-btn"
            title="Copy Note Content"
            @click.stop="$emit('copy-content', object)"
          >
            <Copy :size="12" />
          </button>
        </div>

        <div class="note-body">
          <div v-if="isEditing" class="editor-wrap" @mousedown.stop>
            <CodeMirrorEditor
              :model-value="object.content"
              :font-size="object.meta?.font_size || 13"
              :font-family="object.meta?.font_family || 'sans'"
              @update:model-value="onContentChange"
              @exit="$emit('exit-editing')"
              @save="$emit('save-request')"
            />
          </div>
          <div v-else class="note-content-preview">
            <div v-if="parsedLines.length === 0" class="empty-note-hint">
              Empty note. Double-click or press Enter to edit...
            </div>
            <!-- Formatted Markdown Checkboxes & Content -->
            <div
              v-for="(line, idx) in parsedLines"
              :key="idx"
              class="note-line"
              :class="{ 'is-checkbox': line.isCheckbox }"
            >
              <template v-if="line.isCheckbox">
                <span
                  class="checkbox-box"
                  :class="{ 'is-checked': line.checked }"
                  @click.stop="toggleCheckbox(idx)"
                >
                  <Check v-if="line.checked" :size="11" class="check-mark" />
                </span>
                <span class="checkbox-text" :class="{ 'is-strike': line.checked }">
                  {{ line.text }}
                </span>
              </template>
              <template v-else-if="line.isTagLine">
                <!-- Tag line rendered in footer -->
              </template>
              <template v-else>
                <span>{{ line.text }}</span>
              </template>
            </div>
          </div>
        </div>

        <!-- Tag Pills Footer -->
        <div v-if="computedTags.length > 0" class="note-footer">
          <span v-for="tag in computedTags" :key="tag" class="tag-pill">
            {{ tag.startsWith('#') ? tag : `#${tag}` }}
          </span>
        </div>
      </div>
    </template>

    <!-- 5. LINK CARD OBJECT -->
    <template v-else-if="object.kind === 'url'">
      <div class="link-card">
        <div class="link-header">
          <LinkIcon :size="14" class="link-icon" />
          <span class="link-header-title">Link</span>
        </div>
        <div class="link-body">
          <div class="link-title">{{ object.title || 'Documentation' }}</div>
          <div class="link-address" @click.stop="openUrl">{{ object.content }}</div>
        </div>
        <button class="link-ext-btn" title="Open Link" @click.stop="openUrl">
          <ExternalLink :size="15" />
        </button>
      </div>
    </template>

    <!-- 6. BACKWARD COMPATIBLE DIAGRAM NODE OBJECT -->
    <template v-else-if="object.kind === 'diagram_node'">
      <div class="diagram-node-card">
        <div class="node-icon-box">
          <component :is="nodeIconComponent" :size="20" class="node-icon" />
        </div>
        <div class="node-info">
          <input
            v-if="isEditingNodeTitle"
            ref="nodeTitleInput"
            v-model="nodeTitleDraft"
            class="node-inline-input title-input"
            @blur="saveNodeTitle"
            @keydown.enter="saveNodeTitle"
            @mousedown.stop
          />
          <div v-else class="node-name" @dblclick.stop="startEditNodeTitle">
            {{ object.title || 'Device' }}
          </div>

          <input
            v-if="isEditingNodeSub"
            ref="nodeSubInput"
            v-model="nodeSubDraft"
            class="node-inline-input sub-input"
            @blur="saveNodeSub"
            @keydown.enter="saveNodeSub"
            @mousedown.stop
          />
          <div v-else-if="object.content" class="node-sub" @dblclick.stop="startEditNodeSub">
            {{ object.content }}
          </div>
        </div>

        <!-- Wireless Broadcast Icon decoration for AP -->
        <div v-if="object.meta?.node_type === 'ap'" class="wifi-broadcast">
          <Radio :size="16" />
        </div>
      </div>
    </template>

    <!-- Connection Anchors (top, right, bottom, left) -->
    <!-- Visible when connector tool is active, or hovered, or selected -->
    <div
      v-for="anchor in anchors"
      :key="anchor"
      class="connection-anchor"
      :class="[`anchor-${anchor}`, { 'is-visible': isConnectorTool || isHovered || isSelected }]"
      @mousedown.stop="onAnchorMouseDown(anchor, $event)"
      :title="`Connect from ${anchor}`"
    ></div>

    <!-- Resize Handles (visible only when selected and not editing) -->
    <template v-if="isSelected && !isEditing">
      <div
        v-for="handle in handles"
        :key="handle"
        :class="['resize-handle', `handle-${handle}`]"
        @mousedown="onHandleMouseDown(handle, $event)"
      ></div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, nextTick } from 'vue';
import {
  BookOpen,
  Copy,
  Check,
  Code2,
  Link as LinkIcon,
  ExternalLink,
  Router as RouterIcon,
  Network,
  Server,
  HardDrive,
  Wifi,
  Laptop,
  Smartphone,
  Cloud,
  Radio,
  Pencil,
} from 'lucide-vue-next';
import type { AnchorPosition, BoardObject } from '../types';
import CodeMirrorEditor from './CodeMirrorEditor.vue';
import { getIconComponent } from '../utils/iconRegistry';

const props = defineProps<{
  object: BoardObject;
  isSelected: boolean;
  isEditing: boolean;
  isConnectorTool?: boolean;
  zoom: number;
}>();

const emit = defineEmits<{
  (e: 'select', id: string, multi: boolean): void;
  (e: 'start-drag', id: string, event: MouseEvent): void;
  (e: 'start-resize', id: string, handle: string, event: MouseEvent): void;
  (e: 'start-connect', id: string, anchor: AnchorPosition, event: MouseEvent): void;
  (e: 'start-editing', id: string): void;
  (e: 'exit-editing'): void;
  (e: 'update-content', id: string, content: string): void;
  (e: 'update-title', id: string, title: string): void;
  (e: 'update-meta', id: string, meta: any): void;
  (e: 'copy-content', object: BoardObject): void;
  (e: 'delete', id: string): void;
  (e: 'save-request'): void;
}>();

const handles = ['nw', 'n', 'ne', 'e', 'se', 's', 'sw', 'w'];
const anchors: AnchorPosition[] = ['top', 'right', 'bottom', 'left'];

const isHovered = ref(false);

const objectStyle = computed(() => ({
  transform: `translate(${props.object.x}px, ${props.object.y}px)`,
  width: `${props.object.width}px`,
  height: `${props.object.height}px`,
  zIndex: props.isSelected ? 500 + props.object.z_index : 10 + props.object.z_index,
}));

// Check if this text object represents a code snippet
const isCodeSnippet = computed(() => {
  return props.object.kind === 'code' || props.object.meta?.is_code === true;
});

const codeLineCount = computed(() => {
  return props.object.content ? props.object.content.split('\n').length : 1;
});

// Icon resolution for kind === 'icon'
const resolvedIconComponent = computed(() => {
  const iconId = props.object.meta?.icon_id || props.object.title?.toLowerCase() || 'square';
  return getIconComponent(iconId);
});

// Parse note lines for checklist items and tags
interface ParsedLine {
  text: string;
  isCheckbox: boolean;
  checked: boolean;
  isTagLine: boolean;
}

const parsedLines = computed<ParsedLine[]>(() => {
  if (!props.object.content) return [];
  const lines = props.object.content.split('\n');
  return lines.map((line) => {
    const trimmed = line.trim();
    if (trimmed.startsWith('- [ ]')) {
      return { text: trimmed.slice(5).trim(), isCheckbox: true, checked: false, isTagLine: false };
    }
    if (trimmed.startsWith('- [x]') || trimmed.startsWith('- [X]')) {
      return { text: trimmed.slice(5).trim(), isCheckbox: true, checked: true, isTagLine: false };
    }
    if (trimmed.startsWith('#') && !trimmed.startsWith('##') && trimmed.includes('#')) {
      return { text: trimmed, isCheckbox: false, checked: false, isTagLine: true };
    }
    return { text: line, isCheckbox: false, checked: false, isTagLine: false };
  });
});

const computedTags = computed<string[]>(() => {
  if (props.object.meta?.tags && Array.isArray(props.object.meta.tags)) {
    return props.object.meta.tags;
  }
  const matches = props.object.content.match(/#([a-zA-Z0-9_\-]+)/g);
  if (matches) {
    return Array.from(new Set(matches));
  }
  return [];
});

function toggleCheckbox(lineIndex: number) {
  const lines = props.object.content.split('\n');
  if (lines[lineIndex]) {
    const trimmed = lines[lineIndex].trim();
    if (trimmed.startsWith('- [ ]')) {
      lines[lineIndex] = lines[lineIndex].replace('- [ ]', '- [x]');
    } else if (trimmed.startsWith('- [x]') || trimmed.startsWith('- [X]')) {
      lines[lineIndex] = lines[lineIndex].replace(/- \[[xX]\]/, '- [ ]');
    }
    emit('update-content', props.object.id, lines.join('\n'));
  }
}

// Image Renaming
const isEditingImageTitle = ref(false);
const imageTitleDraft = ref('');
const imageTitleInputRef = ref<HTMLInputElement | null>(null);

function startEditImageTitle() {
  imageTitleDraft.value = props.object.title || 'Image';
  isEditingImageTitle.value = true;
  nextTick(() => {
    imageTitleInputRef.value?.focus();
    imageTitleInputRef.value?.select();
  });
}

function saveImageTitle() {
  if (isEditingImageTitle.value) {
    isEditingImageTitle.value = false;
    const trimmed = imageTitleDraft.value.trim();
    if (trimmed && trimmed !== props.object.title) {
      emit('update-title', props.object.id, trimmed);
    }
  }
}

function cancelEditImageTitle() {
  isEditingImageTitle.value = false;
}

// Icon Title Editing
const isEditingIconTitle = ref(false);
const iconTitleDraft = ref('');
const iconTitleInputRef = ref<HTMLInputElement | null>(null);

function startEditIconTitle() {
  iconTitleDraft.value = props.object.title || '';
  isEditingIconTitle.value = true;
  nextTick(() => {
    iconTitleInputRef.value?.focus();
    iconTitleInputRef.value?.select();
  });
}

function saveIconTitle() {
  if (isEditingIconTitle.value) {
    isEditingIconTitle.value = false;
    emit('update-title', props.object.id, iconTitleDraft.value.trim());
  }
}

function cancelEditIconTitle() {
  isEditingIconTitle.value = false;
}

// Diagram Node Icon & Title Editing
const nodeIconComponent = computed(() => {
  const t = props.object.meta?.node_type;
  switch (t) {
    case 'router':
      return RouterIcon;
    case 'switch':
      return Network;
    case 'olt':
      return Server;
    case 'onu':
      return HardDrive;
    case 'ap':
      return Wifi;
    case 'client_pc':
      return Laptop;
    case 'client_phone':
      return Smartphone;
    case 'cloud':
      return Cloud;
    default:
      return Network;
  }
});

const isEditingNodeTitle = ref(false);
const nodeTitleDraft = ref('');
const nodeTitleInput = ref<HTMLInputElement | null>(null);

const isEditingNodeSub = ref(false);
const nodeSubDraft = ref('');
const nodeSubInput = ref<HTMLInputElement | null>(null);

function startEditNodeTitle() {
  nodeTitleDraft.value = props.object.title || '';
  isEditingNodeTitle.value = true;
  nextTick(() => {
    nodeTitleInput.value?.focus();
  });
}

function saveNodeTitle() {
  if (isEditingNodeTitle.value) {
    isEditingNodeTitle.value = false;
    emit('update-title', props.object.id, nodeTitleDraft.value);
  }
}

function startEditNodeSub() {
  nodeSubDraft.value = props.object.content || '';
  isEditingNodeSub.value = true;
  nextTick(() => {
    nodeSubInput.value?.focus();
  });
}

function saveNodeSub() {
  if (isEditingNodeSub.value) {
    isEditingNodeSub.value = false;
    emit('update-content', props.object.id, nodeSubDraft.value);
  }
}

function startEditTitle() {
  const newTitle = prompt('Edit Title:', props.object.title || '');
  if (newTitle !== null) {
    emit('update-title', props.object.id, newTitle);
  }
}

function openUrl() {
  if (props.object.content) {
    let url = props.object.content.trim();
    if (!url.startsWith('http://') && !url.startsWith('https://')) {
      url = `https://${url}`;
    }
    window.open(url, '_blank');
  }
}

function onMouseDown(e: MouseEvent) {
  if (props.isEditing) return;
  if (props.isConnectorTool) {
    // If connector tool is active, let parent canvas start connector drafting!
    return;
  }
  if (e.button === 0) {
    e.stopPropagation();
    emit('select', props.object.id, e.shiftKey);
    emit('start-drag', props.object.id, e);
  }
}

function onDoubleClick(e: MouseEvent) {
  if (props.object.kind === 'text' || props.object.kind === 'code') {
    emit('start-editing', props.object.id);
  } else if (props.object.kind === 'image') {
    startEditImageTitle();
  }
}

function onHeaderMouseDown(e: MouseEvent) {
  if (props.isEditing) {
    onMouseDown(e);
  }
}

function onHandleMouseDown(handle: string, e: MouseEvent) {
  if (e.button === 0) {
    e.stopPropagation();
    emit('start-resize', props.object.id, handle, e);
  }
}

function onAnchorMouseDown(anchor: AnchorPosition, e: MouseEvent) {
  if (e.button !== 0) return;
  e.stopPropagation();
  emit('start-connect', props.object.id, anchor, e);
}

function onContentChange(newContent: string) {
  emit('update-content', props.object.id, newContent);
}
</script>

<style scoped>
.board-object {
  position: absolute;
  top: 0;
  left: 0;
  display: flex;
  flex-direction: column;
  box-sizing: border-box;
  transform-style: flat;
  backface-visibility: visible;
  overflow: visible;
  cursor: move;
}

.board-object.connector-mode {
  cursor: crosshair;
}

.board-object.is-editing {
  cursor: auto;
}

/* Selected state across all objects */
.board-object.is-selected {
  outline: 2px solid #c084fc;
  box-shadow: 0 0 0 2px rgba(192, 132, 252, 0.3), 0 12px 32px rgba(0, 0, 0, 0.6);
  border-radius: 12px;
}

/* 1. IMAGE OBJECT */
.kind-image {
  background: transparent;
}

.image-container {
  width: 100%;
  height: 100%;
  border-radius: 12px;
  overflow: hidden;
  background: transparent;
  position: relative;
}

.board-image {
  width: 100%;
  height: 100%;
  object-fit: contain;
  display: block;
  pointer-events: none;
  image-rendering: -webkit-optimize-contrast;
}

.image-title-pill {
  position: absolute;
  top: 6px;
  left: 6px;
  background: rgba(15, 16, 29, 0.85);
  backdrop-filter: blur(4px);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 6px;
  padding: 3px 8px;
  z-index: 10;
  display: flex;
  align-items: center;
  max-width: 90%;
}

.image-title-text {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: #f1f5f9;
  font-weight: 500;
  cursor: pointer;
}

.img-name {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.image-rename-btn {
  background: transparent;
  border: none;
  color: #94a3b8;
  cursor: pointer;
  padding: 1px;
  display: flex;
  align-items: center;
}
.image-rename-btn:hover {
  color: #c084fc;
}

.image-title-input {
  background: transparent;
  border: none;
  font-size: 11px;
  color: #ffffff;
  outline: none;
  width: 120px;
}

/* 2. ICON OBJECT */
.icon-node-card {
  width: 100%;
  height: 100%;
  background: rgba(20, 21, 37, 0.7);
  backdrop-filter: blur(8px);
  border: 1px solid #282a42;
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 8px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  transition: border-color 0.15s ease;
}

.icon-glyph {
  color: #c084fc;
}

.icon-label-pill {
  font-size: 11px;
  font-weight: 600;
  color: #cbd5e1;
  text-align: center;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 90%;
}

.icon-title-input {
  background: #0d0e17;
  border: 1px solid #a855f7;
  border-radius: 4px;
  font-size: 10px;
  color: #f8fafc;
  padding: 1px 4px;
  text-align: center;
  width: 80px;
  outline: none;
}

/* 3. NOTE / MARKDOWN CARD */
.note-card {
  width: 100%;
  height: 100%;
  background: #171828;
  border: 1px solid #282a42;
  border-radius: 14px;
  box-shadow: 0 8px 28px rgba(0, 0, 0, 0.5);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.note-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  border-bottom: 1px solid #222338;
}

.note-header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.note-icon {
  color: #c084fc;
}

.note-title {
  font-size: 13px;
  font-weight: 700;
  color: #f1f5f9;
}

.note-action-btn {
  background: transparent;
  border: none;
  color: #94a3b8;
  cursor: pointer;
  padding: 2px 4px;
  border-radius: 4px;
}
.note-action-btn:hover {
  color: #f8fafc;
  background: rgba(255, 255, 255, 0.08);
}

.note-body {
  flex: 1;
  padding: 12px 14px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.editor-wrap {
  width: 100%;
  height: 100%;
}

.note-content-preview {
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 12.5px;
  line-height: 1.5;
  color: #cbd5e1;
}

.empty-note-hint {
  color: #64748b;
  font-style: italic;
  font-size: 12px;
  padding: 8px 0;
}

.note-line {
  display: flex;
  align-items: flex-start;
  gap: 8px;
}

.checkbox-box {
  width: 14px;
  height: 14px;
  border: 1px solid #475569;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-top: 2px;
  cursor: pointer;
  background: #0f101d;
  flex-shrink: 0;
}
.checkbox-box.is-checked {
  background: #9333ea;
  border-color: #a855f7;
}

.check-mark {
  color: #ffffff;
}

.checkbox-text.is-strike {
  text-decoration: line-through;
  color: #64748b;
}

.note-footer {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px 10px;
  flex-wrap: wrap;
}

.tag-pill {
  font-size: 10px;
  font-weight: 600;
  color: #c084fc;
  background: rgba(192, 132, 252, 0.12);
  padding: 2px 8px;
  border-radius: 12px;
  border: 1px solid rgba(192, 132, 252, 0.25);
}

/* 4. CODE SNIPPET CARD */
.code-card {
  width: 100%;
  height: 100%;
  background: #141525;
  border: 1px solid #282a42;
  border-radius: 14px;
  box-shadow: 0 8px 28px rgba(0, 0, 0, 0.5);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.code-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: #10111e;
  border-bottom: 1px solid #222338;
}

.code-header-left {
  display: flex;
  align-items: center;
  gap: 6px;
}

.code-icon {
  color: #4ade80;
}

.code-title {
  font-size: 12px;
  font-weight: 700;
  color: #f1f5f9;
}

.code-copy-btn {
  background: transparent;
  border: none;
  color: #94a3b8;
  cursor: pointer;
  padding: 2px 4px;
  border-radius: 4px;
}
.code-copy-btn:hover {
  color: #f8fafc;
  background: rgba(255, 255, 255, 0.08);
}

.code-body {
  flex: 1;
  display: flex;
  background: #0d0e17;
  padding: 8px;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 12px;
  line-height: 1.5;
  overflow: hidden;
}

.code-lines {
  display: flex;
  flex-direction: column;
  padding-right: 10px;
  color: #475569;
  user-select: none;
  text-align: right;
  border-right: 1px solid #1e2030;
  margin-right: 10px;
}

.code-pre {
  margin: 0;
  color: #f8fafc;
  overflow-x: auto;
  white-space: pre;
}

.code-footer {
  padding: 6px 10px 8px;
  background: #10111e;
  border-top: 1px solid #1e2030;
}

/* 5. LINK PREVIEW CARD */
.link-card {
  width: 100%;
  height: 100%;
  background: #141525;
  border: 1px solid #282a42;
  border-radius: 12px;
  padding: 10px 12px;
  display: flex;
  align-items: center;
  gap: 10px;
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.45);
}

.link-header {
  display: flex;
  align-items: center;
  gap: 5px;
  color: #a855f7;
}

.link-header-title {
  font-size: 12px;
  font-weight: 700;
  color: #f1f5f9;
}

.link-body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.link-title {
  font-size: 12.5px;
  font-weight: 600;
  color: #f8fafc;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.link-address {
  font-size: 11px;
  color: #818cf8;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  cursor: pointer;
}
.link-address:hover {
  text-decoration: underline;
}

.link-ext-btn {
  background: transparent;
  border: none;
  color: #94a3b8;
  cursor: pointer;
  padding: 4px;
}
.link-ext-btn:hover {
  color: #c084fc;
}

/* 6. NETWORK DIAGRAM NODES */
.diagram-node-card {
  width: 100%;
  height: 100%;
  background: #161729;
  border: 1px solid #282a42;
  border-radius: 12px;
  display: flex;
  align-items: center;
  padding: 8px 12px;
  gap: 10px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
  position: relative;
}

.node-icon-box {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: #1c1d32;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #c084fc;
  flex-shrink: 0;
}

.node-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.node-name {
  font-size: 12.5px;
  font-weight: 700;
  color: #f8fafc;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.node-sub {
  font-size: 11px;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  color: #94a3b8;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.node-inline-input {
  background: #0d0e17;
  border: 1px solid #a855f7;
  border-radius: 4px;
  font-size: 11px;
  color: #f8fafc;
  padding: 1px 4px;
  outline: none;
}

.wifi-broadcast {
  position: absolute;
  right: -24px;
  top: 50%;
  transform: translateY(-50%);
  color: #c084fc;
  animation: pulse 1.8s infinite;
}

@keyframes pulse {
  0% { opacity: 0.4; transform: translateY(-50%) scale(0.95); }
  50% { opacity: 1; transform: translateY(-50%) scale(1.1); }
  100% { opacity: 0.4; transform: translateY(-50%) scale(0.95); }
}

/* CONNECTION ANCHORS */
.connection-anchor {
  position: absolute;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: #c084fc;
  border: 2px solid #141525;
  box-shadow: 0 0 8px rgba(192, 132, 252, 0.7);
  opacity: 0;
  transform: translate(-50%, -50%);
  pointer-events: none;
  transition: opacity 0.15s ease, transform 0.15s ease;
  z-index: 100;
}

.connection-anchor.is-visible {
  opacity: 1;
  pointer-events: auto;
  cursor: crosshair;
}

.connection-anchor:hover {
  transform: translate(-50%, -50%) scale(1.35);
  background: #e9d5ff;
}

.anchor-top { top: 0; left: 50%; }
.anchor-right { top: 50%; left: 100%; }
.anchor-bottom { top: 100%; left: 50%; }
.anchor-left { top: 50%; left: 0; }

/* RESIZE HANDLES */
.resize-handle {
  position: absolute;
  width: 8px;
  height: 8px;
  background: #c084fc;
  border: 1px solid #141525;
  border-radius: 2px;
  z-index: 10020;
}

.handle-nw { top: -4px; left: -4px; cursor: nwse-resize; }
.handle-n  { top: -4px; left: calc(50% - 4px); cursor: ns-resize; }
.handle-ne { top: -4px; right: -4px; cursor: nesw-resize; }
.handle-e  { top: calc(50% - 4px); right: -4px; cursor: ew-resize; }
.handle-se { bottom: -4px; right: -4px; cursor: nwse-resize; }
.handle-s  { bottom: -4px; left: calc(50% - 4px); cursor: ns-resize; }
.handle-sw { bottom: -4px; left: -4px; cursor: nesw-resize; }
.handle-w  { top: calc(50% - 4px); left: -4px; cursor: ew-resize; }
</style>
