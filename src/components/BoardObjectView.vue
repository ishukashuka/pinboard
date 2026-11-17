<template>
  <div
    class="board-object"
    :class="{
      'is-selected': isSelected,
      'is-editing': isEditing,
      'kind-image': object.kind === 'image',
      'kind-text': object.kind === 'text',
      'kind-code': isCodeSnippet,
      'kind-url': object.kind === 'url',
      'kind-diagram': object.kind === 'diagram_node',
    }"
    :style="objectStyle"
    @mousedown="onMouseDown"
    @dblclick="onDoubleClick"
  >
    <!-- IMAGE OBJECT (Clean, borderless when unselected, pure image with rounded corners) -->
    <template v-if="object.kind === 'image'">
      <div class="image-container">
        <img
          :src="object.content"
          :alt="object.title || 'Image'"
          draggable="false"
          class="board-image"
        />
      </div>
    </template>

    <!-- CODE SNIPPET OBJECT -->
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

    <!-- TEXT / MARKDOWN NOTE OBJECT -->
    <template v-else-if="object.kind === 'text'">
      <div class="note-card">
        <div class="note-header" @mousedown="onHeaderMouseDown">
          <div class="note-header-left">
            <BookOpen :size="14" class="note-icon" />
            <span class="note-title" @dblclick.stop="startEditTitle">
              {{ object.title || 'Ideas for Mind Studio' }}
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

    <!-- LINK CARD OBJECT -->
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

    <!-- NETWORK DIAGRAM NODE OBJECT -->
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
          <div v-else class="node-sub" @dblclick.stop="startEditNodeSub">
            {{ object.content || object.meta?.subtitle || '10.0.0.1' }}
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
      :class="[`anchor-${anchor}`, { 'is-visible': isConnectorTool || isHovered }]"
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
} from 'lucide-vue-next';
import type { AnchorPosition, BoardObject } from '../types';
import CodeMirrorEditor from './CodeMirrorEditor.vue';

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
  zIndex: props.object.z_index,
}));

// Check if this text object represents a code snippet
const isCodeSnippet = computed(() => {
  return props.object.kind === 'code' || props.object.meta?.is_code === true;
});

const codeLineCount = computed(() => {
  return props.object.content ? props.object.content.split('\n').length : 1;
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
  // Extract tags from content
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
  nextTick(() => nodeTitleInput.value?.focus());
}

function saveNodeTitle() {
  if (isEditingNodeTitle.value) {
    emit('update-title', props.object.id, nodeTitleDraft.value.trim() || 'Device');
    isEditingNodeTitle.value = false;
  }
}

function startEditNodeSub() {
  nodeSubDraft.value = props.object.content || props.object.meta?.subtitle || '';
  isEditingNodeSub.value = true;
  nextTick(() => nodeSubInput.value?.focus());
}

function saveNodeSub() {
  if (isEditingNodeSub.value) {
    const val = nodeSubDraft.value.trim();
    emit('update-content', props.object.id, val);
    emit('update-meta', props.object.id, {
      ...props.object.meta,
      subtitle: val,
    });
    isEditingNodeSub.value = false;
  }
}

function onMouseDown(e: MouseEvent) {
  if (e.button !== 0) return; // Middle click allowed to bubble for canvas panning!
  e.stopPropagation();

  if (props.isEditing) return;
  const isMulti = e.shiftKey || e.ctrlKey || e.metaKey;
  emit('select', props.object.id, isMulti);
  emit('start-drag', props.object.id, e);
}

function onHeaderMouseDown(e: MouseEvent) {
  if (e.button !== 0) return;
  onMouseDown(e);
}

function onDoubleClick(e: MouseEvent) {
  if (e.button !== 0) return;
  if (props.object.kind === 'text') {
    emit('start-editing', props.object.id);
  } else if (props.object.kind === 'image') {
    emit('copy-content', props.object);
  } else if (props.object.kind === 'url') {
    openUrl();
  }
}

function onHandleMouseDown(handle: string, e: MouseEvent) {
  if (e.button !== 0) return;
  e.stopPropagation();
  emit('start-resize', props.object.id, handle, e);
}

function onAnchorMouseDown(anchor: AnchorPosition, e: MouseEvent) {
  if (e.button !== 0) return;
  e.stopPropagation();
  emit('start-connect', props.object.id, anchor, e);
}

function onContentChange(newContent: string) {
  emit('update-content', props.object.id, newContent);
}

function startEditTitle() {
  const current = props.object.title || '';
  const newTitle = prompt('Edit note title:', current);
  if (newTitle !== null) {
    emit('update-title', props.object.id, newTitle.trim());
  }
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
  box-sizing: border-box;
  transform-style: flat;
  backface-visibility: visible;
  overflow: visible;
  cursor: grab;
}

.board-object:active {
  cursor: grabbing;
}

/* Selected state across all objects */
.board-object.is-selected {
  outline: 2px solid #c084fc;
  box-shadow: 0 0 0 2px rgba(192, 132, 252, 0.3), 0 12px 32px rgba(0, 0, 0, 0.6);
  border-radius: 12px;
}

/* 1. IMAGE OBJECT */
.kind-image {
  border-radius: 12px;
  overflow: visible;
  background: transparent;
}

.image-container {
  width: 100%;
  height: 100%;
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45);
  background: #11121d;
}

.board-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
  pointer-events: none;
  image-rendering: -webkit-optimize-contrast;
}

/* 2. NOTE / MARKDOWN CARD ("Ideas for Mind Studio") */
.note-card {
  width: 100%;
  height: 100%;
  background: #171828;
  border: 1px solid #282a42;
  border-radius: 14px;
  box-shadow: 0 8px 28px rgba(0, 0, 0, 0.5);
  display: flex;
  flex-direction: column;
  padding: 14px 16px;
  box-sizing: border-box;
}

.note-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
  user-select: none;
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
  font-weight: 700;
  font-size: 14px;
  color: #f1f5f9;
  letter-spacing: -0.1px;
}

.note-action-btn {
  background: transparent;
  border: none;
  color: #64748b;
  cursor: pointer;
  padding: 2px;
  border-radius: 4px;
}

.note-action-btn:hover {
  color: #f1f5f9;
  background: #23253d;
}

.note-body {
  flex: 1;
  overflow-y: auto;
}

.note-content-preview {
  display: flex;
  flex-direction: column;
  gap: 8px;
  font-size: 13px;
  color: #cbd5e1;
  line-height: 1.5;
}

.note-line {
  display: flex;
  align-items: center;
  gap: 8px;
}

.checkbox-box {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  border: 1.5px solid #474b70;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.12s;
  flex-shrink: 0;
}

.checkbox-box.is-checked {
  background: #a855f7;
  border-color: #a855f7;
}

.check-mark {
  color: #ffffff;
}

.checkbox-text.is-strike {
  color: #94a3b8;
  text-decoration: line-through;
}

.note-footer {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 12px;
  padding-top: 8px;
}

.tag-pill {
  font-size: 11px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 6px;
  background: rgba(168, 85, 247, 0.16);
  color: #c084fc;
}

.editor-wrap {
  width: 100%;
  height: 100%;
}

/* 3. CODE SNIPPET CARD */
.code-card {
  width: 100%;
  height: 100%;
  background: #141524;
  border: 1px solid #282a42;
  border-radius: 14px;
  padding: 12px 14px;
  display: flex;
  flex-direction: column;
  box-shadow: 0 8px 28px rgba(0, 0, 0, 0.5);
  box-sizing: border-box;
}

.code-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
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
  font-weight: 700;
  font-size: 13px;
  color: #f1f5f9;
}

.code-copy-btn {
  background: transparent;
  border: none;
  color: #64748b;
  cursor: pointer;
  padding: 2px 4px;
  border-radius: 4px;
}

.code-copy-btn:hover {
  color: #f1f5f9;
  background: #23253d;
}

.code-body {
  flex: 1;
  display: flex;
  gap: 10px;
  background: #0f101c;
  border-radius: 8px;
  padding: 8px 10px;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, monospace;
  font-size: 12px;
  overflow: auto;
}

.code-lines {
  display: flex;
  flex-direction: column;
  color: #475569;
  user-select: none;
}

.code-pre {
  margin: 0;
  color: #e2e8f0;
  line-height: 1.6;
}

.code-footer {
  margin-top: 8px;
}

/* 4. LINK CARD */
.link-card {
  width: 100%;
  height: 100%;
  background: #161726;
  border: 1px solid #282a42;
  border-radius: 12px;
  padding: 12px 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  box-shadow: 0 6px 24px rgba(0, 0, 0, 0.45);
  box-sizing: border-box;
}

.link-header {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-right: 12px;
}

.link-icon {
  color: #c084fc;
}

.link-header-title {
  font-size: 12px;
  font-weight: 700;
  color: #94a3b8;
}

.link-body {
  flex: 1;
  overflow: hidden;
}

.link-title {
  font-weight: 600;
  font-size: 13px;
  color: #f1f5f9;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.link-address {
  font-size: 11px;
  color: #c084fc;
  cursor: pointer;
  text-decoration: underline;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.link-ext-btn {
  background: transparent;
  border: none;
  color: #c084fc;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
}

.link-ext-btn:hover {
  background: rgba(192, 132, 252, 0.15);
}

/* 5. NETWORK DIAGRAM NODES */
.diagram-node-card {
  width: 100%;
  height: 100%;
  background: #161726;
  border: 1px solid #2a2c42;
  border-radius: 12px;
  padding: 8px 12px;
  display: flex;
  align-items: center;
  gap: 12px;
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.5);
  box-sizing: border-box;
}

.node-icon-box {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  background: #1e2035;
  border-radius: 8px;
  flex-shrink: 0;
}

.node-icon {
  color: #c084fc;
}

.node-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.node-name {
  font-weight: 700;
  font-size: 13px;
  color: #f1f5f9;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.node-sub {
  font-size: 11px;
  color: #94a3b8;
  font-family: ui-monospace, monospace;
}

.node-inline-input {
  background: #0f101a;
  border: 1px solid #a855f7;
  color: #fff;
  border-radius: 4px;
  padding: 1px 4px;
  outline: none;
}

.title-input {
  font-size: 13px;
  font-weight: 700;
}

.sub-input {
  font-size: 11px;
  font-family: ui-monospace, monospace;
}

.wifi-broadcast {
  color: #c084fc;
  opacity: 0.8;
  animation: pulseWifi 1.5s infinite;
}

@keyframes pulseWifi {
  0% { opacity: 0.3; transform: scale(0.9); }
  50% { opacity: 1; transform: scale(1.1); }
  100% { opacity: 0.3; transform: scale(0.9); }
}

/* Connection Anchors */
.connection-anchor {
  position: absolute;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #c084fc;
  border: 1.5px solid #ffffff;
  box-shadow: 0 0 4px rgba(192, 132, 252, 0.8);
  opacity: 0;
  transition: opacity 0.15s ease, transform 0.15s ease;
  z-index: 100;
  cursor: crosshair;
}

.board-object:hover .connection-anchor,
.connection-anchor.is-visible {
  opacity: 1;
}

.anchor-top {
  top: -4px;
  left: calc(50% - 4px);
}

.anchor-right {
  top: calc(50% - 4px);
  right: -4px;
}

.anchor-bottom {
  bottom: -4px;
  left: calc(50% - 4px);
}

.anchor-left {
  top: calc(50% - 4px);
  left: -4px;
}

/* Resize Handles */
.resize-handle {
  position: absolute;
  width: 10px;
  height: 10px;
  background-color: #c084fc;
  border: 1.5px solid #ffffff;
  border-radius: 2px;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.6);
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
