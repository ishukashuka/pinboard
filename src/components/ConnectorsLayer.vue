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
        :stroke-width="Math.max(16, 20 / zoom)"
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
    </g>
  </svg>

  <!-- High z-index overlay for selection handles, draft connector, and snap ring -->
  <svg class="connectors-overlay">
    <defs>
      <marker
        id="arrow-draft"
        viewBox="0 0 10 10"
        refX="7"
        refY="5"
        markerWidth="6"
        markerHeight="6"
        orient="auto-start-reverse"
      >
        <path d="M 0 1.5 L 8 5 L 0 8.5 z" fill="#c084fc" />
      </marker>
    </defs>

    <!-- Selection Handles on endpoints when selected (Excalidraw-like interactive editing) -->
    <template v-if="selectedConnector">
      <!-- Start Handle -->
      <g class="handle-group">
        <circle
          :cx="selectedConnector.x1"
          :cy="selectedConnector.y1"
          :r="Math.max(12, 14 / zoom)"
          class="endpoint-hit"
          @mousedown.stop="$emit('start-drag-endpoint', selectedConnector.id, 'start', $event)"
        />
        <rect
          :x="selectedConnector.x1 - (4.5 / zoom)"
          :y="selectedConnector.y1 - (4.5 / zoom)"
          :width="9 / zoom"
          :height="9 / zoom"
          :rx="2 / zoom"
          class="connector-handle"
        />
      </g>

      <!-- End Handle -->
      <g class="handle-group">
        <circle
          :cx="selectedConnector.x2"
          :cy="selectedConnector.y2"
          :r="Math.max(12, 14 / zoom)"
          class="endpoint-hit"
          @mousedown.stop="$emit('start-drag-endpoint', selectedConnector.id, 'end', $event)"
        />
        <rect
          :x="selectedConnector.x2 - (4.5 / zoom)"
          :y="selectedConnector.y2 - (4.5 / zoom)"
          :width="9 / zoom"
          :height="9 / zoom"
          :rx="2 / zoom"
          class="connector-handle"
        />
      </g>
    </template>

    <!-- Snap Highlight Indicator during connection or endpoint drag -->
    <g v-if="snapTarget" class="snap-indicator">
      <circle
        :cx="snapTarget.x"
        :cy="snapTarget.y"
        :r="Math.max(10, 12 / zoom)"
        class="snap-ring"
      />
      <circle
        :cx="snapTarget.x"
        :cy="snapTarget.y"
        :r="Math.max(3, 4 / zoom)"
        class="snap-dot"
      />
    </g>

    <!-- Draft Connector Preview during live dragging -->
    <g v-if="draftConnector">
      <path
        :d="draftConnector.path"
        class="draft-path"
        stroke="#c084fc"
        stroke-width="2.5"
        stroke-dasharray="6 4"
        marker-end="url(#arrow-draft)"
      />
      <circle :cx="draftConnector.x1" :cy="draftConnector.y1" :r="5 / zoom" fill="#c084fc" />
      <circle :cx="draftConnector.x2" :cy="draftConnector.y2" :r="5 / zoom" fill="#c084fc" />
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
  snapTarget?: { x: number; y: number } | null;
  zoom: number;
}>();

const emit = defineEmits<{
  (e: 'select-connector', id: string, event: MouseEvent): void;
  (e: 'start-drag-endpoint', id: string, endpoint: 'start' | 'end', event: MouseEvent): void;
}>();

const markerColors = [
  '#c084fc',
  '#a855f7',
  '#ffffff',
  '#38bdf8',
  '#4ade80',
  '#facc15',
  '#f87171',
  '#f1f5f9',
  '#94a3b8',
];

function sanitizeColor(c: string): string {
  return c.replace(/[^a-zA-Z0-9]/g, '');
}

function getDashArray(style: string): string | undefined {
  if (style === 'dashed') return '7 5';
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
      // Clean orthogonal 90-degree elbow route
      const dx = x2 - x1;
      const dy = y2 - y1;

      // Determine routing direction based on anchors
      if (c.from_anchor === 'left' || c.from_anchor === 'right' || (!c.from_anchor && Math.abs(dx) >= Math.abs(dy))) {
        const mid = x1 + dx / 2;
        path = `M ${x1} ${y1} L ${mid} ${y1} L ${mid} ${y2} L ${x2} ${y2}`;
        midX = mid;
        midY = (y1 + y2) / 2;
      } else {
        const mid = y1 + dy / 2;
        path = `M ${x1} ${y1} L ${x1} ${mid} L ${x2} ${mid} L ${x2} ${y2}`;
        midX = (x1 + x2) / 2;
        midY = mid;
      }
    } else {
      // Curved bezier (S-curve)
      const dx = Math.abs(x2 - x1);
      const dy = Math.abs(y2 - y1);
      const cpDist = Math.max(dx * 0.45, 45);

      let cp1x = (c.from_anchor === 'left') ? x1 - cpDist : x1 + cpDist;
      let cp1y = y1;
      let cp2x = (c.to_anchor === 'right') ? x2 + cpDist : x2 - cpDist;
      let cp2y = y2;

      if (c.from_anchor === 'top') { cp1x = x1; cp1y = y1 - cpDist; }
      else if (c.from_anchor === 'bottom') { cp1x = x1; cp1y = y1 + cpDist; }

      if (c.to_anchor === 'top') { cp2x = x2; cp2y = y2 - cpDist; }
      else if (c.to_anchor === 'bottom') { cp2x = x2; cp2y = y2 + cpDist; }

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

const selectedConnector = computed(() => {
  return computedConnectors.value.find((c) => c.id === props.selectedConnectorId) || null;
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
  z-index: 5;
}

.connectors-overlay {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  overflow: visible;
  z-index: 600;
}

.hit-path {
  fill: none;
  stroke: transparent;
  pointer-events: stroke;
  cursor: pointer;
}

.visible-path {
  fill: none;
  stroke-linecap: round;
  stroke-linejoin: round;
  pointer-events: none;
  transition: stroke 0.15s ease;
}

.visible-path.is-selected {
  filter: drop-shadow(0 0 6px rgba(192, 132, 252, 0.75));
}

.connector-label-group {
  pointer-events: auto;
  cursor: pointer;
}

.label-bg {
  fill: #141525;
  stroke: #2a2c42;
  stroke-width: 1;
  filter: drop-shadow(0 4px 10px rgba(0, 0, 0, 0.5));
}

.label-text {
  fill: #f1f5f9;
  font-size: 11px;
  font-weight: 600;
  user-select: none;
  dominant-baseline: central;
}

.connector-group:hover .visible-path {
  filter: drop-shadow(0 0 4px rgba(192, 132, 252, 0.5));
}

/* Endpoint Selection Handles */
.handle-group {
  pointer-events: auto;
}

.endpoint-hit {
  fill: transparent;
  cursor: grab;
}
.endpoint-hit:active {
  cursor: grabbing;
}

.connector-handle {
  fill: #c084fc;
  stroke: #141525;
  stroke-width: 1;
  pointer-events: none;
}

/* Snap Indicator */
.snap-ring {
  fill: rgba(192, 132, 252, 0.25);
  stroke: #c084fc;
  stroke-width: 2;
  animation: pulseSnap 1.2s infinite ease-in-out;
}

.snap-dot {
  fill: #ffffff;
}

@keyframes pulseSnap {
  0% { transform: scale(0.9); opacity: 0.7; }
  50% { transform: scale(1.15); opacity: 1; }
  100% { transform: scale(0.9); opacity: 0.7; }
}

.draft-path {
  fill: none;
  stroke-linecap: round;
}
</style>
