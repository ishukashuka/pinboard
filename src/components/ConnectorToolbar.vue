<template>
  <div ref="toolbarRef" class="connector-toolbar" :style="toolbarStyle" @mousedown.stop @keydown.esc.stop="closeAllMenus">
    <!-- 1. Line Type Dropdown -->
    <div class="tool-dropdown-wrap">
      <button class="tb-btn" :class="{ 'is-open': activeMenu === 'lineType' }" title="Line Shape" @click.stop="toggleMenu('lineType')">
        <component :is="lineTypeIcon" :size="14" />
        <ChevronDown :size="12" class="chevron" />
      </button>
      <div v-if="activeMenu === 'lineType'" class="tb-dropdown-menu" @click.stop>
        <button
          class="tb-menu-item"
          :class="{ 'is-active': connector.line_type === 'curved' }"
          @click="setLineType('curved')"
        >
          <Spline :size="14" />
          <span>Curved (S-Curve)</span>
        </button>
        <button
          class="tb-menu-item"
          :class="{ 'is-active': connector.line_type === 'orthogonal' }"
          @click="setLineType('orthogonal')"
        >
          <CornerDownRight :size="14" />
          <span>Elbow (90°)</span>
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

    <!-- 2. Arrowhead Dropdown -->
    <div class="tool-dropdown-wrap">
      <button class="tb-btn" :class="{ 'is-open': activeMenu === 'arrow' }" title="Arrowheads" @click.stop="toggleMenu('arrow')">
        <ArrowRight :size="14" />
        <ChevronDown :size="12" class="chevron" />
      </button>
      <div v-if="activeMenu === 'arrow'" class="tb-dropdown-menu" @click.stop>
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

    <!-- 3. Stroke Style & Width Dropdown -->
    <div class="tool-dropdown-wrap">
      <button class="tb-btn" :class="{ 'is-open': activeMenu === 'stroke' }" title="Stroke Style & Width" @click.stop="toggleMenu('stroke')">
        <span class="stroke-sample" :class="connector.stroke_style">---</span>
        <ChevronDown :size="12" class="chevron" />
      </button>
      <div v-if="activeMenu === 'stroke'" class="tb-dropdown-menu" @click.stop>
        <div class="menu-section-title">Stroke Pattern</div>
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

        <div class="menu-section-title">Thickness</div>
        <div class="width-selector">
          <button
            v-for="w in [1.5, 2.5, 4.0]"
            :key="w"
            class="width-btn"
            :class="{ 'is-active': connector.stroke_width === w }"
            @click="setStrokeWidth(w)"
          >
            <div class="width-bar" :style="{ height: `${w * 1.5}px` }"></div>
          </button>
        </div>
      </div>
    </div>

    <div class="tb-divider"></div>

    <!-- 4. Color Swatches -->
    <div class="tool-dropdown-wrap">
      <button class="color-btn" :class="{ 'is-open': activeMenu === 'color' }" title="Line Color" @click.stop="toggleMenu('color')">
        <span class="color-dot" :style="{ backgroundColor: connector.color }"></span>
      </button>
      <div v-if="activeMenu === 'color'" class="color-palette-popover" @click.stop>
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

    <!-- 5. Label Input -->
    <div class="label-input-wrap">
      <input
        v-model="labelDraft"
        class="tb-label-input"
        placeholder="Add label... (e.g. PON, 10G)"
        @input="onLabelChange"
        @keydown.enter.stop="blurInput"
      />
      <button v-if="labelDraft" class="clear-btn" title="Clear label" @click.stop="clearLabel">
        <X :size="12" />
      </button>
    </div>

    <div class="tb-divider"></div>

    <!-- 6. Duplicate Button -->
    <button class="tb-btn" title="Duplicate Connector (Ctrl+D)" @click="$emit('duplicate', connector.id)">
      <CopyPlus :size="14" />
    </button>

    <div class="tb-divider"></div>

    <!-- 7. Delete Button -->
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
  CopyPlus,
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
  (e: 'duplicate', id: string): void;
}>();

const toolbarRef = ref<HTMLDivElement | null>(null);
const activeMenu = ref<'lineType' | 'arrow' | 'stroke' | 'color' | null>(null);
const labelDraft = ref(props.connector.label || '');

watch(
  () => props.connector.label,
  (val) => {
    labelDraft.value = val || '';
  }
);

const toolbarStyle = computed(() => {
  // Center toolbar horizontally on screenX, place 45px above connector
  const tbWidth = 340;
  let left = Math.max(16, props.screenX - tbWidth / 2);
  if (left + tbWidth > window.innerWidth - 16) {
    left = window.innerWidth - tbWidth - 16;
  }
  let top = props.screenY - 50;
  if (top < 56) top = props.screenY + 30; // flip below if off top
  return {
    position: 'fixed' as const,
    left: `${left}px`,
    top: `${top}px`,
    zIndex: 10050,
  };
});

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
  '#c084fc', // Purple (target accent)
  '#a855f7', // Violet
  '#38bdf8', // Cyan
  '#4ade80', // Emerald Green
  '#facc15', // Amber Yellow
  '#f87171', // Rose Red
  '#f1f5f9', // Crisp White
  '#94a3b8', // Muted Slate
];

function toggleMenu(menu: 'lineType' | 'arrow' | 'stroke' | 'color') {
  activeMenu.value = activeMenu.value === menu ? null : menu;
}

function closeAllMenus() {
  activeMenu.value = null;
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

function setStrokeWidth(width: number) {
  emit('update', { stroke_width: width });
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

function blurInput(e: Event) {
  (e.target as HTMLInputElement)?.blur();
}

function onGlobalClick(e: MouseEvent) {
  if (toolbarRef.value && !toolbarRef.value.contains(e.target as Node)) {
    activeMenu.value = null;
  }
}

onMounted(() => {
  window.addEventListener('click', onGlobalClick);
});

onBeforeUnmount(() => {
  window.removeEventListener('click', onGlobalClick);
});
</script>

<style scoped>
.connector-toolbar {
  display: flex;
  align-items: center;
  gap: 3px;
  background: #141525;
  border: 1px solid #282a42;
  border-radius: 10px;
  padding: 4px 6px;
  box-shadow: 0 12px 36px rgba(0, 0, 0, 0.65);
  user-select: none;
}

.tb-divider {
  width: 1px;
  height: 18px;
  background: #24263b;
  margin: 0 2px;
}

.tool-dropdown-wrap {
  position: relative;
}

.tb-btn {
  display: flex;
  align-items: center;
  gap: 3px;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 6px;
  padding: 5px 6px;
  color: #cbd5e1;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.15s ease;
}
.tb-btn:hover,
.tb-btn.is-open {
  background: rgba(255, 255, 255, 0.08);
  color: #f8fafc;
}

.btn-delete:hover {
  background: rgba(244, 63, 94, 0.18);
  color: #fb7185;
}

.chevron {
  color: #64748b;
}

.stroke-sample {
  font-family: ui-monospace, monospace;
  font-size: 11px;
  letter-spacing: -1px;
  color: #cbd5e1;
}

.tb-dropdown-menu {
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  background: #141525;
  border: 1px solid #2a2c42;
  border-radius: 8px;
  padding: 4px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.7);
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 130px;
  z-index: 10060;
}

.menu-section-title {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  color: #64748b;
  padding: 4px 8px 2px;
}

.tb-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  background: transparent;
  border: none;
  border-radius: 5px;
  padding: 5px 8px;
  color: #cbd5e1;
  font-size: 12px;
  cursor: pointer;
  text-align: left;
  transition: all 0.15s ease;
}
.tb-menu-item:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #f8fafc;
}
.tb-menu-item.is-active {
  background: rgba(168, 85, 247, 0.2);
  color: #c084fc;
  font-weight: 600;
}

.width-selector {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 6px;
}

.width-btn {
  flex: 1;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #11121d;
  border: 1px solid #23253a;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.15s ease;
}
.width-btn:hover {
  background: #1e1b4b;
  border-color: #a855f7;
}
.width-btn.is-active {
  background: rgba(168, 85, 247, 0.25);
  border-color: #c084fc;
}

.width-bar {
  width: 18px;
  background: #f1f5f9;
  border-radius: 1px;
}

.color-btn {
  background: transparent;
  border: 1px solid transparent;
  border-radius: 6px;
  padding: 5px 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
}
.color-btn:hover,
.color-btn.is-open {
  background: rgba(255, 255, 255, 0.08);
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
  background: #141525;
  border: 1px solid #2a2c42;
  border-radius: 8px;
  padding: 6px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.7);
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 5px;
  z-index: 10060;
}

.color-swatch {
  width: 18px;
  height: 18px;
  border-radius: 4px;
  border: 1px solid transparent;
  cursor: pointer;
  transition: transform 0.15s ease;
}
.color-swatch:hover {
  transform: scale(1.15);
}
.color-swatch.is-selected {
  border-color: #ffffff;
  box-shadow: 0 0 0 1px #a855f7;
}

.label-input-wrap {
  position: relative;
  display: flex;
  align-items: center;
}

.tb-label-input {
  background: #0f101d;
  border: 1px solid #24263b;
  border-radius: 6px;
  padding: 4px 22px 4px 8px;
  font-size: 11px;
  color: #f1f5f9;
  width: 125px;
  outline: none;
}
.tb-label-input:focus {
  border-color: #a855f7;
}

.clear-btn {
  position: absolute;
  right: 5px;
  background: transparent;
  border: none;
  color: #64748b;
  cursor: pointer;
  padding: 1px;
}
.clear-btn:hover {
  color: #f1f5f9;
}
</style>
