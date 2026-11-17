<template>
  <aside class="tool-rail-wrapper" @mousedown.stop>
    <div class="tool-rail">
      <button
        v-for="tool in tools"
        :key="tool.id"
        class="tool-btn"
        :class="{ 'is-active': activeTool === tool.id }"
        :title="`${tool.label} (${tool.shortcut})`"
        @click="selectTool(tool.id)"
      >
        <component :is="tool.icon" :size="18" />
        <span class="tool-tooltip">{{ tool.label }} ({{ tool.shortcut }})</span>
      </button>

      <div class="rail-divider"></div>

      <!-- Icon Library Palette Trigger -->
      <button
        class="tool-btn"
        :class="{ 'is-active': isIconPaletteOpen }"
        title="Icon Library"
        @click="$emit('toggle-icon-palette')"
      >
        <Boxes :size="18" />
        <span class="tool-tooltip">Icon Library</span>
      </button>
    </div>
  </aside>
</template>

<script setup lang="ts">
import {
  MousePointer2,
  Hand,
  SquareDashed,
  Type,
  Image as ImageIcon,
  Link as LinkIcon,
  Spline,
  Boxes,
} from 'lucide-vue-next';
import type { ActiveTool } from '../types';

const props = defineProps<{
  activeTool: ActiveTool;
  isIconPaletteOpen: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:activeTool', tool: ActiveTool): void;
  (e: 'import-image'): void;
  (e: 'create-link'): void;
  (e: 'toggle-icon-palette'): void;
}>();

const tools = [
  { id: 'select' as ActiveTool, label: 'Select', shortcut: 'V', icon: MousePointer2 },
  { id: 'hand' as ActiveTool, label: 'Pan', shortcut: 'H', icon: Hand },
  { id: 'marquee' as ActiveTool, label: 'Marquee', shortcut: 'M', icon: SquareDashed },
  { id: 'text' as ActiveTool, label: 'Text Note', shortcut: 'N', icon: Type },
  { id: 'image' as ActiveTool, label: 'Import Image', shortcut: 'Upload', icon: ImageIcon },
  { id: 'link' as ActiveTool, label: 'Link Card', shortcut: 'L', icon: LinkIcon },
  { id: 'connector' as ActiveTool, label: 'Connector', shortcut: 'C', icon: Spline },
];

function selectTool(toolId: ActiveTool) {
  if (toolId === 'image') {
    emit('import-image');
    return;
  }
  if (toolId === 'link') {
    emit('create-link');
    return;
  }
  emit('update:activeTool', toolId);
}
</script>

<style scoped>
.tool-rail-wrapper {
  position: fixed;
  left: 12px;
  top: 60px;
  z-index: 10040;
  user-select: none;
}

.tool-rail {
  display: flex;
  flex-direction: column;
  gap: 4px;
  background: #141525;
  border: 1px solid #23253a;
  border-radius: 12px;
  padding: 6px;
  box-shadow: 0 12px 36px rgba(0, 0, 0, 0.55);
}

.rail-divider {
  height: 1px;
  background: #23253a;
  margin: 3px 2px;
}

.tool-btn {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 8px;
  border: 1px solid transparent;
  background: transparent;
  color: #94a3b8;
  cursor: pointer;
  transition: all 0.15s ease;
}

.tool-btn:hover {
  background: rgba(255, 255, 255, 0.06);
  color: #f1f5f9;
}

.tool-btn.is-active {
  background: rgba(168, 85, 247, 0.2);
  border-color: rgba(168, 85, 247, 0.45);
  color: #c084fc;
}

/* Floating Tooltip */
.tool-tooltip {
  position: absolute;
  left: calc(100% + 10px);
  top: 50%;
  transform: translateY(-50%);
  background: #0f101d;
  color: #f1f5f9;
  border: 1px solid #282a40;
  border-radius: 6px;
  padding: 4px 8px;
  font-size: 11px;
  white-space: nowrap;
  pointer-events: none;
  opacity: 0;
  transition: opacity 0.15s ease;
  box-shadow: 0 4px 14px rgba(0, 0, 0, 0.5);
  z-index: 10050;
}

.tool-btn:hover .tool-tooltip {
  opacity: 1;
}
</style>
