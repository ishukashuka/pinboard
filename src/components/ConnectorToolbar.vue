<template>
  <div class="connector-toolbar" :style="toolbarStyle" @mousedown.stop>
    <!-- Line Type Dropdown -->
    <div class="tool-dropdown-wrap">
      <button class="tb-btn" @click="toggleMenu('lineType')" title="Line Type">
        <component :is="lineTypeIcon" :size="14" />
        <ChevronDown :size="12" class="chevron" />
      </button>
      <div v-if="activeMenu === 'lineType'" class="tb-dropdown-menu">
        <button
          class="tb-menu-item"
          :class="{ 'is-active': connector.line_type === 'curved' }"
          @click="setLineType('curved')"
        >
          <Spline :size="14" />
          <span>Curved</span>
        </button>
        <button
          class="tb-menu-item"
          :class="{ 'is-active': connector.line_type === 'orthogonal' }"
          @click="setLineType('orthogonal')"
        >
          <CornerDownRight :size="14" />
          <span>Orthogonal</span>
        </button>
        <button
          class="tb-menu-item"
          :class="{ 'is-active': connector.line_type === 'straight' }"
          @click="setLineType('straight')"
        >
          <Slash :size="14" />
          <span>Straight</span>
        </button>
      </div>
    </div>

    <div class="tb-divider"></div>

    <!-- Arrowhead Dropdown -->
    <div class="tool-dropdown-wrap">
      <button class="tb-btn" @click="toggleMenu('arrow')" title="Arrowhead">
        <ArrowRight :size="14" />
        <ChevronDown :size="12" class="chevron" />
      </button>
      <div v-if="activeMenu === 'arrow'" class="tb-dropdown-menu">
        <button
          class="tb-menu-item"
          :class="{ 'is-active': connector.arrow_end === 'none' && connector.arrow_start === 'none' }"
          @click="setArrowheads('none', 'none')"
        >
          <Minus :size="14" />
          <span>None</span>
        </button>
        <button
          class="tb-menu-item"
          :class="{ 'is-active': connector.arrow_end === 'arrow' && connector.arrow_start === 'none' }"
          @click="setArrowheads('none', 'arrow')"
        >
          <ArrowRight :size="14" />
          <span>End (→)</span>
        </button>
        <button
          class="tb-menu-item"
          :class="{ 'is-active': connector.arrow_start === 'arrow' && connector.arrow_end === 'none' }"
          @click="setArrowheads('arrow', 'none')"
        >
          <ArrowLeft :size="14" />
          <span>Start (←)</span>
        </button>
        <button
          class="tb-menu-item"
          :class="{ 'is-active': connector.arrow_start === 'arrow' && connector.arrow_end === 'arrow' }"
          @click="setArrowheads('arrow', 'arrow')"
        >
          <MoveHorizontal :size="14" />
          <span>Both (↔)</span>
        </button>
      </div>
    </div>

    <div class="tb-divider"></div>

    <!-- Stroke Style Dropdown -->
    <div class="tool-dropdown-wrap">
      <button class="tb-btn" @click="toggleMenu('stroke')" title="Stroke Style">
        <span class="stroke-sample" :class="connector.stroke_style">---</span>
        <ChevronDown :size="12" class="chevron" />
      </button>
      <div v-if="activeMenu === 'stroke'" class="tb-dropdown-menu">
        <button
          class="tb-menu-item"
          :class="{ 'is-active': connector.stroke_style === 'solid' }"
          @click="setStrokeStyle('solid')"
        >
          <span>Solid</span>
        </button>
        <button
          class="tb-menu-item"
          :class="{ 'is-active': connector.stroke_style === 'dashed' }"
          @click="setStrokeStyle('dashed')"
        >
          <span>Dashed</span>
        </button>
        <button
          class="tb-menu-item"
          :class="{ 'is-active': connector.stroke_style === 'dotted' }"
          @click="setStrokeStyle('dotted')"
        >
          <span>Dotted</span>
        </button>
      </div>
    </div>

    <div class="tb-divider"></div>

    <!-- Color Picker Dot -->
    <div class="tool-dropdown-wrap">
      <button class="color-btn" @click="toggleMenu('color')" title="Line Color">
        <span class="color-dot" :style="{ backgroundColor: connector.color }"></span>
      </button>
      <div v-if="activeMenu === 'color'" class="color-palette-popover">
        <button
          v-for="c in colorOptions"
          :key="c"
          class="color-swatch"
          :style="{ backgroundColor: c }"
          :class="{ 'is-selected': connector.color === c }"
          @click="setColor(c)"
        ></button>
      </div>
    </div>

    <div class="tb-divider"></div>

    <!-- Label Input -->
    <div class="label-input-wrap">
      <input
        v-model="labelDraft"
        class="tb-label-input"
        placeholder="Label..."
        @input="onLabelChange"
        @keydown.enter.stop
      />
      <button v-if="labelDraft" class="clear-btn" @click="clearLabel" title="Clear label">
        <X :size="12" />
      </button>
    </div>

    <div class="tb-divider"></div>

    <!-- Delete Button -->
    <button class="tb-btn btn-delete" title="Delete Connector" @click="$emit('delete', connector.id)">
      <Trash2 :size="14" />
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue';
import {
  Spline,
  CornerDownRight,
  Slash,
  ArrowRight,
  ArrowLeft,
  MoveHorizontal,
  Minus,
  ChevronDown,
  X,
  Trash2,
} from 'lucide-vue-next';
import type { BoardConnector } from '../types';

const props = defineProps<{
  connector: BoardConnector;
  screenX: number;
  screenY: number;
}>();

const emit = defineEmits<{
  (e: 'update', updated: Partial<BoardConnector>): void;
  (e: 'delete', id: string): void;
}>();

const activeMenu = ref<'lineType' | 'arrow' | 'stroke' | 'color' | null>(null);
const labelDraft = ref(props.connector.label || '');

watch(
  () => props.connector.label,
  (val) => {
    labelDraft.value = val || '';
  }
);

const toolbarStyle = computed(() => ({
  position: 'fixed' as const,
  left: `${Math.max(16, props.screenX)}px`,
  top: `${Math.max(56, props.screenY - 50)}px`,
  zIndex: 10050,
}));

const lineTypeIcon = computed(() => {
  switch (props.connector.line_type) {
    case 'orthogonal':
      return CornerDownRight;
    case 'straight':
      return Slash;
    default:
      return Spline;
  }
});

const colorOptions = [
  '#a855f7', // Purple
  '#ffffff', // White
  '#38bdf8', // Cyan
  '#4ade80', // Green
  '#facc15', // Yellow
  '#f87171', // Red
];

function toggleMenu(menu: 'lineType' | 'arrow' | 'stroke' | 'color') {
  activeMenu.value = activeMenu.value === menu ? null : menu;
}

function setLineType(type: 'curved' | 'orthogonal' | 'straight') {
  emit('update', { line_type: type });
  activeMenu.value = null;
}

function setArrowheads(start: 'none' | 'arrow', end: 'none' | 'arrow') {
  emit('update', { arrow_start: start, arrow_end: end });
  activeMenu.value = null;
}

function setStrokeStyle(style: 'solid' | 'dashed' | 'dotted') {
  emit('update', { stroke_style: style });
  activeMenu.value = null;
}

function setColor(color: string) {
  emit('update', { color });
  activeMenu.value = null;
}

function onLabelChange() {
  emit('update', { label: labelDraft.value.trim() || undefined });
}

function clearLabel() {
  labelDraft.value = '';
  emit('update', { label: undefined });
}

function onDocClick(e: MouseEvent) {
  activeMenu.value = null;
}

onMounted(() => {
  window.addEventListener('click', onDocClick);
});

onBeforeUnmount(() => {
  window.removeEventListener('click', onDocClick);
});
</script>

<style scoped>
.connector-toolbar {
  display: flex;
  align-items: center;
  gap: 4px;
  background: #171828;
  border: 1px solid #2d2f47;
  border-radius: 12px;
  padding: 4px 8px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.7);
  user-select: none;
  animation: fadeIn 0.12s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(4px); }
  to { opacity: 1; transform: translateY(0); }
}

.tool-dropdown-wrap {
  position: relative;
}

.tb-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  background: transparent;
  border: none;
  color: #cbd5e1;
  padding: 5px 6px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.1s;
}

.tb-btn:hover {
  background: #23253d;
  color: #fff;
}

.chevron {
  color: #64748b;
}

.stroke-sample {
  font-family: monospace;
  font-size: 11px;
  color: #cbd5e1;
}

.tb-dropdown-menu {
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  min-width: 130px;
  background: #151624;
  border: 1px solid #282a40;
  border-radius: 8px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.6);
  padding: 4px;
  z-index: 10060;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.tb-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 5px 8px;
  background: transparent;
  border: none;
  border-radius: 6px;
  color: #cbd5e1;
  font-size: 12px;
  cursor: pointer;
  text-align: left;
}

.tb-menu-item:hover {
  background: #23253d;
  color: #fff;
}

.tb-menu-item.is-active {
  background: rgba(168, 85, 247, 0.2);
  color: #c084fc;
  font-weight: 600;
}

.tb-divider {
  width: 1px;
  height: 18px;
  background: #292b42;
  margin: 0 2px;
}

/* Color picker dot */
.color-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  background: transparent;
  border: none;
  border-radius: 6px;
  cursor: pointer;
}

.color-btn:hover {
  background: #23253d;
}

.color-dot {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  border: 1px solid rgba(255, 255, 255, 0.3);
}

.color-palette-popover {
  position: absolute;
  top: calc(100% + 6px);
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  gap: 6px;
  background: #151624;
  border: 1px solid #282a40;
  border-radius: 8px;
  padding: 6px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.6);
  z-index: 10060;
}

.color-swatch {
  width: 18px;
  height: 18px;
  border-radius: 50%;
  border: 1px solid transparent;
  cursor: pointer;
  transition: transform 0.1s;
}

.color-swatch:hover {
  transform: scale(1.15);
}

.color-swatch.is-selected {
  border-color: #fff;
  box-shadow: 0 0 0 2px rgba(168, 85, 247, 0.6);
}

/* Label Input */
.label-input-wrap {
  display: flex;
  align-items: center;
  background: #1f2136;
  border: 1px solid #303352;
  border-radius: 6px;
  padding: 2px 6px;
}

.tb-label-input {
  background: transparent;
  border: none;
  outline: none;
  font-size: 12px;
  color: #f1f5f9;
  width: 90px;
  font-family: inherit;
}

.tb-label-input::placeholder {
  color: #64748b;
}

.clear-btn {
  background: transparent;
  border: none;
  color: #64748b;
  cursor: pointer;
  padding: 2px;
  display: flex;
  align-items: center;
}

.clear-btn:hover {
  color: #f1f5f9;
}

/* Delete button */
.btn-delete {
  color: #f87171;
}

.btn-delete:hover {
  background: rgba(239, 68, 68, 0.2);
  color: #ef4444;
}
</style>
