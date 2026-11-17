<template>
  <svg class="connectors-layer">
    <defs>
      <!-- Arrowhead Markers for various colors -->
      <marker
        v-for="color in markerColors"
        :key="`arrow-${color}`"
        :id="`arrow-${sanitizeColor(color)}`"
        viewBox="0 0 10 10"
        refX="7"
        refY="5"
        markerWidth="6"
        markerHeight="6"
        orient="auto-start-reverse"
      >
        <path d="M 0 1.5 L 8 5 L 0 8.5 z" :fill="color" />
      </marker>
    </defs>

    <!-- Connectors -->
    <g v-for="conn in computedConnectors" :key="conn.id" class="connector-group">
      <!-- Invisible wider hit-area path for easy clicking -->
      <path
        :d="conn.path"
        class="hit-path"
        @mousedown.stop="$emit('select-connector', conn.id, $event)"
      />

      <!-- Visible line -->
      <path
        :d="conn.path"
        class="visible-path"
        :class="{ 'is-selected': selectedConnectorId === conn.id }"
        :stroke="conn.color"
        :stroke-width="selectedConnectorId === conn.id ? conn.stroke_width + 1 : conn.stroke_width"
        :stroke-dasharray="getDashArray(conn.stroke_style)"
        :marker-start="conn.arrow_start === 'arrow' ? `url(#arrow-${sanitizeColor(conn.color)})` : undefined"
        :marker-end="conn.arrow_end === 'arrow' ? `url(#arrow-${sanitizeColor(conn.color)})` : undefined"
        @mousedown.stop="$emit('select-connector', conn.id, $event)"
      />

      <!-- Text Label if present -->
      <g
        v-if="conn.label"
        class="connector-label-group"
        :transform="`translate(${conn.midX}, ${conn.midY})`"
        @mousedown.stop="$emit('select-connector', conn.id, $event)"
      >
        <rect
          :x="-(conn.label.length * 4.2 + 10)"
          y="-11"
          :width="conn.label.length * 8.4 + 20"
          height="22"
          rx="6"
          class="label-bg"
        />
        <text
          x="0"
          y="4"
          text-anchor="middle"
          class="label-text"
        >
          {{ conn.label }}
        </text>
      </g>

      <!-- Selection Handles on endpoints when selected -->
      <template v-if="selectedConnectorId === conn.id">
        <rect
          :x="conn.x1 - 4.5"
          :y="conn.y1 - 4.5"
          width="9"
          height="9"
          rx="2"
          class="connector-handle"
        />
        <rect
          :x="conn.x2 - 4.5"
          :y="conn.y2 - 4.5"
          width="9"
          height="9"
          rx="2"
          class="connector-handle"
        />
      </template>
    </g>

    <!-- Draft Connector Preview during live dragging -->
    <g v-if="draftConnector">
      <path
        :d="draftConnector.path"
        class="draft-path"
        stroke="#c084fc"
        stroke-width="2"
        stroke-dasharray="5 4"
        :marker-end="`url(#arrow-${sanitizeColor('#c084fc')})`"
      />
      <circle :cx="draftConnector.x1" :cy="draftConnector.y1" r="4.5" fill="#c084fc" />
      <circle :cx="draftConnector.x2" :cy="draftConnector.y2" r="4.5" fill="#c084fc" />
    </g>
  </svg>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { AnchorPosition, BoardConnector, BoardObject } from '../types';

const props = defineProps<{
  connectors: BoardConnector[];
  objects: BoardObject[];
  selectedConnectorId: string | null;
  draftConnector: { x1: number; y1: number; x2: number; y2: number; path: string } | null;
}>();

const emit = defineEmits<{
  (e: 'select-connector', id: string, event: MouseEvent): void;
}>();

const markerColors = ['#a855f7', '#c084fc', '#ffffff', '#38bdf8', '#4ade80', '#facc15', '#f87171'];

function sanitizeColor(c: string): string {
  return c.replace(/[^a-zA-Z0-9]/g, '');
}

function getDashArray(style: string): string | undefined {
  if (style === 'dashed') return '6 5';
  if (style === 'dotted') return '2 4';
  return undefined;
}

function getObjectAnchorPoint(
  obj: BoardObject,
  anchor?: AnchorPosition
): { x: number; y: number } {
  switch (anchor) {
    case 'top':
      return { x: obj.x + obj.width / 2, y: obj.y };
    case 'bottom':
      return { x: obj.x + obj.width / 2, y: obj.y + obj.height };
    case 'left':
      return { x: obj.x, y: obj.y + obj.height / 2 };
    case 'right':
    default:
      return { x: obj.x + obj.width, y: obj.y + obj.height / 2 };
  }
}

const computedConnectors = computed(() => {
  const objectMap = new Map<string, BoardObject>();
  for (const obj of props.objects) {
    objectMap.set(obj.id, obj);
  }

  return props.connectors.map((c) => {
    let x1 = c.from_x;
    let y1 = c.from_y;
    let x2 = c.to_x;
    let y2 = c.to_y;

    if (c.from_id && objectMap.has(c.from_id)) {
      const fromObj = objectMap.get(c.from_id)!;
      const p = getObjectAnchorPoint(fromObj, c.from_anchor);
      x1 = p.x;
      y1 = p.y;
    }

    if (c.to_id && objectMap.has(c.to_id)) {
      const toObj = objectMap.get(c.to_id)!;
      const p = getObjectAnchorPoint(toObj, c.to_anchor);
      x2 = p.x;
      y2 = p.y;
    }

    // Build path based on line_type
    let path = '';
    let midX = (x1 + x2) / 2;
    let midY = (y1 + y2) / 2;

    if (c.line_type === 'straight') {
      path = `M ${x1} ${y1} L ${x2} ${y2}`;
    } else if (c.line_type === 'orthogonal') {
      const mid = (x1 + x2) / 2;
      path = `M ${x1} ${y1} L ${mid} ${y1} L ${mid} ${y2} L ${x2} ${y2}`;
    } else {
      // Curved bezier (S-curve)
      const dx = Math.abs(x2 - x1);
      const cpDist = Math.max(dx * 0.5, 40);
      const cp1x = (c.from_anchor === 'left') ? x1 - cpDist : x1 + cpDist;
      const cp1y = y1;
      const cp2x = (c.to_anchor === 'right') ? x2 + cpDist : x2 - cpDist;
      const cp2y = y2;
      path = `M ${x1} ${y1} C ${cp1x} ${cp1y}, ${cp2x} ${cp2y}, ${x2} ${y2}`;

      // Bezier mid-point calculation at t = 0.5
      midX = 0.125 * x1 + 0.375 * cp1x + 0.375 * cp2x + 0.125 * x2;
      midY = 0.125 * y1 + 0.375 * cp1y + 0.375 * cp2y + 0.125 * y2;
    }

    return {
      ...c,
      x1,
      y1,
      x2,
      y2,
      path,
      midX,
      midY,
    };
  });
});
</script>

<style scoped>
.connectors-layer {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  overflow: visible;
  z-index: 10;
}

.connector-group {
  pointer-events: auto;
}

.hit-path {
  fill: none;
  stroke: transparent;
  stroke-width: 22px;
  cursor: pointer;
}

.visible-path {
  fill: none;
  cursor: pointer;
  transition: stroke 0.15s ease, filter 0.15s ease;
}

.visible-path.is-selected {
  stroke: #c084fc !important;
  filter: drop-shadow(0 0 8px rgba(192, 132, 252, 0.8));
}

.connector-handle {
  fill: #ffffff;
  stroke: #a855f7;
  stroke-width: 2px;
  pointer-events: none;
}

.connector-label-group {
  cursor: pointer;
}

.label-bg {
  fill: #171828;
  stroke: #31334f;
  stroke-width: 1px;
  filter: drop-shadow(0 2px 6px rgba(0, 0, 0, 0.4));
}

.label-text {
  fill: #e2e8f0;
  font-size: 11px;
  font-weight: 600;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  letter-spacing: 0.3px;
  user-select: none;
}

.draft-path {
  fill: none;
  pointer-events: none;
}
</style>
