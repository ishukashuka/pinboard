<template>
  <aside class="tool-rail-wrapper">
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

      <!-- Network Diagram Nodes Flyout Trigger -->
      <div class="flyout-trigger-wrap" ref="flyoutRef">
        <button
          class="tool-btn"
          :class="{ 'is-active': isDiagramFlyoutOpen || activeTool === 'diagram' }"
          title="Network Nodes Library"
          @click="toggleDiagramFlyout"
        >
          <Layers :size="18" />
          <span class="tool-tooltip">Network Nodes</span>
        </button>

        <!-- Diagram Flyout Palette -->
        <div v-if="isDiagramFlyoutOpen" class="diagram-flyout">
          <div class="flyout-header">Topology Nodes</div>
          <div class="flyout-grid">
            <button
              v-for="node in diagramNodeTypes"
              :key="node.type"
              class="flyout-node-btn"
              @click="spawnNode(node.type, node.label, node.defaultIp)"
            >
              <component :is="node.icon" :size="16" class="node-icon" />
              <div class="node-meta">
                <span class="node-title">{{ node.label }}</span>
                <span class="node-sub">{{ node.defaultIp }}</span>
              </div>
            </button>
          </div>
        </div>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';
import {
  MousePointer2,
  Hand,
  SquareDashed,
  Type,
  Image as ImageIcon,
  Link as LinkIcon,
  Spline,
  Layers,
  Router as RouterIcon,
  Network,
  Server,
  HardDrive,
  Wifi,
  Laptop,
  Smartphone,
  Cloud,
} from 'lucide-vue-next';
import type { ActiveTool, DiagramNodeType } from '../types';

const props = defineProps<{
  activeTool: ActiveTool;
}>();

const emit = defineEmits<{
  (e: 'update:activeTool', tool: ActiveTool): void;
  (e: 'import-image'): void;
  (e: 'create-link'): void;
  (e: 'spawn-diagram-node', type: DiagramNodeType, label: string, defaultIp: string): void;
}>();

const isDiagramFlyoutOpen = ref(false);
const flyoutRef = ref<HTMLDivElement | null>(null);

const tools = [
  { id: 'select' as ActiveTool, label: 'Select', shortcut: 'V', icon: MousePointer2 },
  { id: 'hand' as ActiveTool, label: 'Pan', shortcut: 'H', icon: Hand },
  { id: 'marquee' as ActiveTool, label: 'Marquee', shortcut: 'M', icon: SquareDashed },
  { id: 'text' as ActiveTool, label: 'Text Note', shortcut: 'T', icon: Type },
  { id: 'image' as ActiveTool, label: 'Import Image', shortcut: 'I', icon: ImageIcon },
  { id: 'link' as ActiveTool, label: 'Link Card', shortcut: 'L', icon: LinkIcon },
  { id: 'connector' as ActiveTool, label: 'Connector', shortcut: 'C', icon: Spline },
];

const diagramNodeTypes: Array<{
  type: DiagramNodeType;
  label: string;
  defaultIp: string;
  icon: any;
}> = [
  { type: 'router', label: 'Router', defaultIp: '192.0.2.1', icon: RouterIcon },
  { type: 'switch', label: 'Core Switch', defaultIp: '10.0.0.1', icon: Network },
  { type: 'olt', label: 'OLT', defaultIp: '10.0.10.1', icon: Server },
  { type: 'onu', label: 'ONU', defaultIp: '10.0.10.101', icon: HardDrive },
  { type: 'ap', label: 'AP', defaultIp: '10.0.20.1', icon: Wifi },
  { type: 'client_pc', label: 'Client', defaultIp: '10.0.10.200', icon: Laptop },
  { type: 'client_phone', label: 'Client', defaultIp: '10.0.20.100', icon: Smartphone },
  { type: 'cloud', label: 'Internet', defaultIp: 'WAN', icon: Cloud },
];

function selectTool(id: ActiveTool) {
  isDiagramFlyoutOpen.value = false;
  if (id === 'image') {
    emit('import-image');
    return;
  }
  if (id === 'link') {
    emit('create-link');
    return;
  }
  emit('update:activeTool', id);
}

function toggleDiagramFlyout(e: MouseEvent) {
  e.stopPropagation();
  isDiagramFlyoutOpen.value = !isDiagramFlyoutOpen.value;
}

function spawnNode(type: DiagramNodeType, label: string, defaultIp: string) {
  emit('spawn-diagram-node', type, label, defaultIp);
  isDiagramFlyoutOpen.value = false;
  emit('update:activeTool', 'select');
}

function onWindowClick(e: MouseEvent) {
  if (flyoutRef.value && !flyoutRef.value.contains(e.target as Node)) {
    isDiagramFlyoutOpen.value = false;
  }
}

onMounted(() => {
  window.addEventListener('click', onWindowClick);
});

onBeforeUnmount(() => {
  window.removeEventListener('click', onWindowClick);
});
</script>

<style scoped>
.tool-rail-wrapper {
  position: fixed;
  left: 16px;
  top: 68px;
  z-index: 9990;
  user-select: none;
}

.tool-rail {
  display: flex;
  flex-direction: column;
  gap: 4px;
  background: #131422;
  border: 1px solid #23253a;
  border-radius: 16px;
  padding: 6px;
  box-shadow: 0 10px 32px rgba(0, 0, 0, 0.55);
}

.tool-btn {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 38px;
  height: 38px;
  border-radius: 10px;
  background: transparent;
  border: none;
  color: #94a3b8;
  cursor: pointer;
  transition: all 0.15s ease;
}

.tool-btn:hover {
  background: #1e2035;
  color: #f1f5f9;
}

.tool-btn.is-active {
  background: #7c3aed;
  color: #ffffff;
  box-shadow: 0 2px 10px rgba(124, 58, 237, 0.45);
}

/* Tooltip on hover */
.tool-tooltip {
  position: absolute;
  left: calc(100% + 10px);
  top: 50%;
  transform: translateY(-50%);
  background: #0f101a;
  color: #f1f5f9;
  font-size: 11px;
  font-weight: 500;
  white-space: nowrap;
  padding: 4px 8px;
  border-radius: 6px;
  border: 1px solid #282a40;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
  pointer-events: none;
  opacity: 0;
  transition: opacity 0.15s ease, transform 0.15s ease;
  z-index: 10000;
}

.tool-btn:hover .tool-tooltip {
  opacity: 1;
  transform: translateY(-50%) translateX(2px);
}

/* Flyout Palette */
.flyout-trigger-wrap {
  position: relative;
}

.diagram-flyout {
  position: absolute;
  left: calc(100% + 12px);
  top: -40px;
  width: 240px;
  background: #151626;
  border: 1px solid #2a2c42;
  border-radius: 14px;
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.7);
  padding: 10px;
  z-index: 10001;
}

.flyout-header {
  font-size: 11px;
  font-weight: 700;
  color: #64748b;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 8px;
  padding: 0 4px;
}

.flyout-grid {
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-height: 320px;
  overflow-y: auto;
}

.flyout-node-btn {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 6px 8px;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 8px;
  cursor: pointer;
  text-align: left;
  transition: all 0.12s ease;
}

.flyout-node-btn:hover {
  background: #202238;
  border-color: #3b3e5e;
}

.node-icon {
  color: #c084fc;
}

.node-meta {
  display: flex;
  flex-direction: column;
}

.node-title {
  font-size: 12px;
  font-weight: 600;
  color: #f1f5f9;
}

.node-sub {
  font-size: 10px;
  color: #94a3b8;
  font-family: ui-monospace, monospace;
}
</style>
