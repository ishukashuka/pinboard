<template>
  <div class="pinboard-workspace" :class="{ 'tool-hand': activeTool === 'hand' || isSpaceDown }">
    <!-- 1. TOP APPLICATION BAR -->
    <TopBar
      :current-board-id="currentBoardId"
      :current-board-name="currentBoardName"
      :can-undo="undoStack.length > 0"
      :can-redo="redoStack.length > 0"
      :always-on-top="alwaysOnTop"
      @switch-board="switchBoard"
      @board-renamed="onBoardRenamed"
      @open-search="openSearch"
      @undo="undo"
      @redo="redo"
      @fit-all="fitAll"
      @reset-camera="resetCamera"
      @toggle-always-on-top="toggleAlwaysOnTop"
      @toggle-shortcuts="isShortcutsOpen = true"
      @clear-board="openClearConfirm"
    />

    <!-- 2. LEFT FLOATING TOOL RAIL -->
    <ToolRail
      v-model:active-tool="activeTool"
      :is-icon-palette-open="isIconPaletteOpen"
      @import-image="triggerImageFileInput"
      @create-link="isLinkModalOpen = true"
      @toggle-icon-palette="isIconPaletteOpen = !isIconPaletteOpen"
    />

    <!-- Icon Library Palette Dropdown -->
    <IconPalette
      v-if="isIconPaletteOpen"
      @select-icon="onSelectIconFromPalette"
      @close="isIconPaletteOpen = false"
    />

    <!-- Hidden file input for Image tool -->
    <input
      ref="fileInputRef"
      type="file"
      accept="image/*"
      style="display: none"
      @change="onImageFileSelected"
    />

    <!-- 3. INFINITE CANVAS VIEWPORT -->
    <div
      ref="canvasViewportRef"
      class="canvas-viewport"
      :style="[gridStyle, { cursor: canvasCursor }]"
      tabindex="0"
      @mousedown="onViewportMouseDown"
      @mousemove="onMouseMove"
      @mouseup="onMouseUp"
      @wheel.prevent="onWheel"
      @contextmenu.prevent
      @dragover.prevent
      @drop.prevent="onDrop"
    >
      <!-- Infinite World Transform Container -->
      <div class="canvas-world" :style="worldTransformStyle">
        <!-- SVG Connectors Layer -->
        <ConnectorsLayer
          :connectors="connectors"
          :objects="objects"
          :selected-connector-id="selectedConnectorId"
          :draft-connector="draftConnector"
          :snap-target="snapTarget"
          :zoom="camera.zoom"
          @select-connector="onSelectConnector"
          @start-drag-endpoint="onStartDragEndpoint"
        />

        <!-- Board Objects (Images, Notes, Snippets, Links, Icons, Diagram Nodes) -->
        <BoardObjectView
          v-for="obj in objects"
          :key="obj.id"
          :object="obj"
          :is-selected="selectedIds.has(obj.id)"
          :is-editing="editingId === obj.id"
          :is-connector-tool="activeTool === 'connector'"
          :zoom="camera.zoom"
          :class="{ 'flash-highlight': highlightedId === obj.id }"
          @select="onObjectSelect"
          @start-drag="onStartObjectDrag"
          @start-resize="onStartObjectResize"
          @start-connect="onStartConnect"
          @start-editing="startEditing"
          @exit-editing="exitEditing"
          @update-content="updateObjectContent"
          @update-title="updateObjectTitle"
          @update-meta="updateObjectMeta"
          @copy-content="copyObjectContent"
          @delete="deleteSingleObject"
          @save-request="triggerSave"
        />
      </div>

      <!-- Marquee Selection Rectangle (Screen Space) -->
      <div v-if="selectionBox" class="selection-marquee" :style="selectionBoxStyle"></div>

      <!-- Floating Contextual Toolbar for Selected Connector -->
      <ConnectorToolbar
        v-if="selectedConnector && selectedConnectorScreenPos"
        :connector="selectedConnector"
        :screen-x="selectedConnectorScreenPos.x"
        :screen-y="selectedConnectorScreenPos.y"
        @update="updateSelectedConnector"
        @delete="deleteConnector"
      />

      <!-- 4. BOTTOM CONTROLS & HUD -->
      <BottomHUD
        :zoom="camera.zoom"
        @zoom-in="zoomIn"
        @zoom-out="zoomOut"
        @fit-all="fitAll"
        @open-search="openSearch"
        @open-shortcuts="isShortcutsOpen = true"
      />

      <!-- Toast Notification -->
      <transition name="toast">
        <div v-if="toastMessage" class="canvas-toast">
          {{ toastMessage }}
        </div>
      </transition>

      <!-- Search-to-Focus Modal -->
      <SearchModal
        v-if="isSearchOpen"
        :objects="objects"
        :connectors="connectors"
        @close="isSearchOpen = false"
        @jump-to="handleSearchJump"
      />

      <!-- Shortcuts Reference Modal -->
      <ShortcutsModal v-if="isShortcutsOpen" @close="isShortcutsOpen = false" />

      <!-- Link Creation Modal -->
      <LinkModal
        v-if="isLinkModalOpen"
        @create="onCreateLinkFromModal"
        @cancel="isLinkModalOpen = false"
      />

      <!-- Clear Board Confirmation Modal -->
      <div v-if="isClearConfirmOpen" class="confirm-backdrop" @click.self="isClearConfirmOpen = false">
        <div class="confirm-dialog" @keydown.esc.stop="isClearConfirmOpen = false">
          <div class="confirm-title">Clear Board</div>
          <p class="confirm-msg">Are you sure you want to delete all objects and connectors from this board? This action can be undone.</p>
          <div class="confirm-actions">
            <button class="btn btn-secondary" @click="isClearConfirmOpen = false">Cancel</button>
            <button class="btn btn-danger" @click="confirmClearBoard">Clear All</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onBeforeUnmount, nextTick } from 'vue';
import type {
  ActiveTool,
  AnchorPosition,
  BoardConnector,
  BoardObject,
  Camera,
} from '../types';
import { api } from '../utils/api';
import TopBar from './TopBar.vue';
import ToolRail from './ToolRail.vue';
import BottomHUD from './BottomHUD.vue';
import BoardObjectView from './BoardObjectView.vue';
import ConnectorsLayer from './ConnectorsLayer.vue';
import ConnectorToolbar from './ConnectorToolbar.vue';
import SearchModal from './SearchModal.vue';
import ShortcutsModal from './ShortcutsModal.vue';
import LinkModal from './LinkModal.vue';
import IconPalette from './IconPalette.vue';
import type { IconItem } from '../utils/iconRegistry';
import type { ExtendedSearchResult } from './SearchModal.vue';

const props = defineProps<{
  boardId?: string;
}>();

const canvasViewportRef = ref<HTMLDivElement | null>(null);
const fileInputRef = ref<HTMLInputElement | null>(null);

const currentBoardId = ref<string>(props.boardId || 'default');
const currentBoardName = ref<string>('Untitled Board');

const objects = ref<BoardObject[]>([]);
const connectors = ref<BoardConnector[]>([]);

const camera = reactive<Camera>({
  panX: 0,
  panY: 0,
  zoom: 1,
});

const previousCamera = ref<Camera | null>(null);

// Active Tool & UI States
const activeTool = ref<ActiveTool>('select');
const selectedIds = ref<Set<string>>(new Set());
const selectedConnectorId = ref<string | null>(null);
const editingId = ref<string | null>(null);

const isSearchOpen = ref(false);
const isShortcutsOpen = ref(false);
const isLinkModalOpen = ref(false);
const isIconPaletteOpen = ref(false);
const isClearConfirmOpen = ref(false);
const alwaysOnTop = ref(false);
const highlightedId = ref<string | null>(null);
const toastMessage = ref<string | null>(null);
let toastTimer: any = null;

// Marquee Selection State
const marqueeStart = ref<{ x: number; y: number } | null>(null);
const marqueeCurrent = ref<{ x: number; y: number } | null>(null);

// Dragging and Resizing State
const isDraggingObjects = ref(false);
const dragStartPos = ref<{ x: number; y: number } | null>(null);
const dragObjectOffsets = ref<Map<string, { x: number; y: number }>>(new Map());

const resizeState = ref<{
  objectId: string;
  handle: string;
  startX: number;
  startY: number;
  initialX: number;
  initialY: number;
  initialW: number;
  initialH: number;
  aspectRatio: number;
} | null>(null);

// Connector Drafting State
const connectingState = ref<{
  fromId?: string;
  fromAnchor?: AnchorPosition;
  startX: number;
  startY: number;
} | null>(null);

// Connector Endpoint Dragging State
const endpointDragState = ref<{
  connectorId: string;
  endpoint: 'start' | 'end';
} | null>(null);

// Snapping Indicator for anchors
const snapTarget = ref<{
  x: number;
  y: number;
  objectId: string;
  anchor: AnchorPosition;
} | null>(null);

const draftConnector = ref<{
  x1: number;
  y1: number;
  x2: number;
  y2: number;
  path: string;
} | null>(null);

// Panning states
const isPanning = ref(false);
const panStart = ref({ x: 0, y: 0 });
const panCameraStart = ref({ x: 0, y: 0 });
const isSpaceDown = ref(false);

// Mouse pointer coordinates in screen space
const mouseScreen = { x: window.innerWidth / 2, y: window.innerHeight / 2 };

// Undo/Redo Stacks
interface HistorySnapshot {
  objects: BoardObject[];
  connectors: BoardConnector[];
}
const undoStack = ref<HistorySnapshot[]>([]);
const redoStack = ref<HistorySnapshot[]>([]);

function pushUndo() {
  const snapshot: HistorySnapshot = {
    objects: JSON.parse(JSON.stringify(objects.value)),
    connectors: JSON.parse(JSON.stringify(connectors.value)),
  };
  undoStack.value.push(snapshot);
  if (undoStack.value.length > 50) {
    undoStack.value.shift();
  }
  redoStack.value = [];
}

function undo() {
  if (undoStack.value.length === 0) return;
  const currentSnapshot: HistorySnapshot = {
    objects: JSON.parse(JSON.stringify(objects.value)),
    connectors: JSON.parse(JSON.stringify(connectors.value)),
  };
  redoStack.value.push(currentSnapshot);

  const prev = undoStack.value.pop()!;
  objects.value = prev.objects;
  connectors.value = prev.connectors;
  selectedIds.value.clear();
  selectedConnectorId.value = null;
  triggerSave();
  showToast('Undo');
}

function redo() {
  if (redoStack.value.length === 0) return;
  const currentSnapshot: HistorySnapshot = {
    objects: JSON.parse(JSON.stringify(objects.value)),
    connectors: JSON.parse(JSON.stringify(connectors.value)),
  };
  undoStack.value.push(currentSnapshot);

  const next = redoStack.value.pop()!;
  objects.value = next.objects;
  connectors.value = next.connectors;
  selectedIds.value.clear();
  selectedConnectorId.value = null;
  triggerSave();
  showToast('Redo');
}

// Single source of truth cursor
const canvasCursor = computed(() => {
  if (isPanning.value) return 'grabbing';
  if (isSpaceDown.value || activeTool.value === 'hand') return 'grab';
  if (activeTool.value === 'connector') return 'crosshair';
  if (activeTool.value === 'text') return 'text';
  if (activeTool.value === 'marquee') return 'crosshair';
  return 'default';
});

// Viewport styles
const worldTransformStyle = computed(() => ({
  transform: `translate(${camera.panX}px, ${camera.panY}px) scale(${camera.zoom})`,
  transformOrigin: '0 0',
}));

const gridStyle = computed(() => {
  const size = 28 * camera.zoom;
  const offsetX = camera.panX % size;
  const offsetY = camera.panY % size;
  return {
    backgroundPosition: `${offsetX}px ${offsetY}px`,
    backgroundSize: `${size}px ${size}px`,
  };
});

const selectionBox = computed(() => {
  if (!marqueeStart.value || !marqueeCurrent.value) return null;
  const x1 = Math.min(marqueeStart.value.x, marqueeCurrent.value.x);
  const y1 = Math.min(marqueeStart.value.y, marqueeCurrent.value.y);
  const x2 = Math.max(marqueeStart.value.x, marqueeCurrent.value.x);
  const y2 = Math.max(marqueeStart.value.y, marqueeCurrent.value.y);
  return { x: x1, y: y1, width: x2 - x1, height: y2 - y1 };
});

const selectionBoxStyle = computed(() => {
  if (!selectionBox.value) return {};
  return {
    left: `${selectionBox.value.x}px`,
    top: `${selectionBox.value.y}px`,
    width: `${selectionBox.value.width}px`,
    height: `${selectionBox.value.height}px`,
  };
});

const selectedConnector = computed(() => {
  if (!selectedConnectorId.value) return null;
  return connectors.value.find((c) => c.id === selectedConnectorId.value) || null;
});

function getObjectAnchorPoint(obj: BoardObject, anchor?: AnchorPosition): { x: number; y: number } {
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

const selectedConnectorScreenPos = computed(() => {
  if (!selectedConnector.value) return null;
  const conn = selectedConnector.value;

  let x1 = conn.from_x;
  let y1 = conn.from_y;
  let x2 = conn.to_x;
  let y2 = conn.to_y;

  if (conn.from_id) {
    const fromObj = objects.value.find((o) => o.id === conn.from_id);
    if (fromObj) {
      const p = getObjectAnchorPoint(fromObj, conn.from_anchor as AnchorPosition);
      x1 = p.x;
      y1 = p.y;
    }
  }
  if (conn.to_id) {
    const toObj = objects.value.find((o) => o.id === conn.to_id);
    if (toObj) {
      const p = getObjectAnchorPoint(toObj, conn.to_anchor as AnchorPosition);
      x2 = p.x;
      y2 = p.y;
    }
  }

  let midWorldX = (x1 + x2) / 2;
  let midWorldY = (y1 + y2) / 2;

  if (conn.line_type === 'curved') {
    const dx = Math.abs(x2 - x1);
    const cpDist = Math.max(dx * 0.45, 45);
    const cp1x = (conn.from_anchor === 'left') ? x1 - cpDist : x1 + cpDist;
    const cp1y = y1;
    const cp2x = (conn.to_anchor === 'right') ? x2 + cpDist : x2 - cpDist;
    const cp2y = y2;
    midWorldX = 0.125 * x1 + 0.375 * cp1x + 0.375 * cp2x + 0.125 * x2;
    midWorldY = 0.125 * y1 + 0.375 * cp1y + 0.375 * cp2y + 0.125 * y2;
  }

  return worldToScreen(midWorldX, midWorldY);
});

// --- COORDINATE CONVERSION ---
function screenToWorld(sx: number, sy: number): { x: number; y: number } {
  return {
    x: (sx - camera.panX) / camera.zoom,
    y: (sy - camera.panY) / camera.zoom,
  };
}

function worldToScreen(wx: number, wy: number): { x: number; y: number } {
  return {
    x: wx * camera.zoom + camera.panX,
    y: wy * camera.zoom + camera.panY,
  };
}

// Find closest anchor on any object (snapping within 40px screen distance)
function findClosestAnchor(
  worldX: number,
  worldY: number,
  excludeObjId?: string
): { x: number; y: number; objectId: string; anchor: AnchorPosition } | null {
  const SNAP_DISTANCE = 42 / camera.zoom;
  let closest: { x: number; y: number; objectId: string; anchor: AnchorPosition; dist: number } | null = null;

  for (const obj of objects.value) {
    if (excludeObjId && obj.id === excludeObjId) continue;

    const anchorsList: { anchor: AnchorPosition; x: number; y: number }[] = [
      { anchor: 'top', x: obj.x + obj.width / 2, y: obj.y },
      { anchor: 'bottom', x: obj.x + obj.width / 2, y: obj.y + obj.height },
      { anchor: 'left', x: obj.x, y: obj.y + obj.height / 2 },
      { anchor: 'right', x: obj.x + obj.width, y: obj.y + obj.height / 2 },
    ];

    for (const a of anchorsList) {
      const d = Math.hypot(worldX - a.x, worldY - a.y);
      if (d <= SNAP_DISTANCE) {
        if (!closest || d < closest.dist) {
          closest = { x: a.x, y: a.y, objectId: obj.id, anchor: a.anchor, dist: d };
        }
      }
    }
  }

  return closest ? { x: closest.x, y: closest.y, objectId: closest.objectId, anchor: closest.anchor } : null;
}

// Build SVG path for draft connector preview
function buildDraftPath(x1: number, y1: number, x2: number, y2: number): string {
  const dx = Math.abs(x2 - x1);
  const cpDist = Math.max(dx * 0.45, 45);
  return `M ${x1} ${y1} C ${x1 + cpDist} ${y1}, ${x2 - cpDist} ${y2}, ${x2} ${y2}`;
}

// --- TOAST HELPER ---
function showToast(msg: string) {
  toastMessage.value = msg;
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = setTimeout(() => {
    toastMessage.value = null;
  }, 2200);
}

// --- LIFECYCLE ---
onMounted(async () => {
  await loadBoardData(currentBoardId.value);

  // Global capture interception for middle-mouse panning
  window.addEventListener('mousedown', onGlobalMouseDownCapture, { capture: true });
  window.addEventListener('auxclick', onGlobalAuxClickCapture, { capture: true });
  window.addEventListener('mousemove', onGlobalMouseMoveCapture, { capture: true });
  window.addEventListener('mouseup', onGlobalMouseUpCapture, { capture: true });
  window.addEventListener('blur', onWindowBlur);

  window.addEventListener('keydown', onKeyDown);
  window.addEventListener('keyup', onKeyUp);
  window.addEventListener('paste', onGlobalPaste);

  nextTick(() => {
    canvasViewportRef.value?.focus();
  });
});

onBeforeUnmount(() => {
  window.removeEventListener('mousedown', onGlobalMouseDownCapture, { capture: true });
  window.removeEventListener('auxclick', onGlobalAuxClickCapture, { capture: true });
  window.removeEventListener('mousemove', onGlobalMouseMoveCapture, { capture: true });
  window.removeEventListener('mouseup', onGlobalMouseUpCapture, { capture: true });
  window.removeEventListener('blur', onWindowBlur);

  window.removeEventListener('keydown', onKeyDown);
  window.removeEventListener('keyup', onKeyUp);
  window.removeEventListener('paste', onGlobalPaste);
});

async function loadBoardData(id: string) {
  const data = await api.loadBoard(id);
  if (data) {
    currentBoardId.value = data.board?.id || id;
    currentBoardName.value = data.board?.name || 'Untitled Board';
    camera.panX = data.board?.pan_x || 0;
    camera.panY = data.board?.pan_y || 0;
    camera.zoom = data.board?.zoom || 1;
    objects.value = data.objects || [];
    connectors.value = data.connectors || [];

    // If initial load contains reference layout, preset selection to match screenshot
    if (objects.value.some((o) => o.id === 'sample-img-mountain')) {
      selectedIds.value = new Set(['sample-img-mountain']);
      selectedConnectorId.value = 'conn-olt-onu1';
      activeTool.value = 'connector';
    }
  }
}

async function switchBoard(id: string) {
  currentBoardId.value = id;
  selectedIds.value.clear();
  selectedConnectorId.value = null;
  undoStack.value = [];
  redoStack.value = [];
  await loadBoardData(id);
  showToast(`Switched to ${currentBoardName.value}`);
}

function onBoardRenamed(name: string) {
  currentBoardName.value = name;
  showToast(`Board renamed to ${name}`);
}

// --- GLOBAL CAPTURE HANDLERS FOR PANNING & MIDDLE-CLICK ---
function onGlobalMouseDownCapture(e: MouseEvent) {
  mouseScreen.x = e.clientX;
  mouseScreen.y = e.clientY;

  // Middle mouse button (button === 1) starts panning
  if (e.button === 1) {
    e.preventDefault();
    e.stopPropagation();
    startPanning(e.clientX, e.clientY);
  }
}

function onGlobalAuxClickCapture(e: MouseEvent) {
  if (e.button === 1) {
    e.preventDefault();
    e.stopPropagation();
  }
}

function onGlobalMouseMoveCapture(e: MouseEvent) {
  mouseScreen.x = e.clientX;
  mouseScreen.y = e.clientY;

  if (isPanning.value) {
    const dx = e.clientX - panStart.value.x;
    const dy = e.clientY - panStart.value.y;
    camera.panX = panCameraStart.value.x + dx;
    camera.panY = panCameraStart.value.y + dy;
  }
}

function onGlobalMouseUpCapture(e: MouseEvent) {
  if (e.button === 1 && isPanning.value) {
    stopPanning();
  }
}

function onWindowBlur() {
  if (isPanning.value) stopPanning();
  isSpaceDown.value = false;
  isDraggingObjects.value = false;
  resizeState.value = null;
  connectingState.value = null;
  endpointDragState.value = null;
  snapTarget.value = null;
  draftConnector.value = null;
}

// --- PANNING HELPERS ---
function startPanning(clientX: number, clientY: number) {
  isPanning.value = true;
  panStart.value = { x: clientX, y: clientY };
  panCameraStart.value = { x: camera.panX, y: camera.panY };
}

function stopPanning() {
  if (isPanning.value) {
    isPanning.value = false;
    triggerSave();
  }
}

// --- ZOOM ---
function onWheel(e: WheelEvent) {
  if (editingId.value) return;

  const zoomFactor = 1.12;
  const direction = e.deltaY < 0 ? 1 : -1;
  const newZoom = direction > 0 ? camera.zoom * zoomFactor : camera.zoom / zoomFactor;
  const clampedZoom = Math.min(Math.max(newZoom, 0.1), 5.0);

  const rect = canvasViewportRef.value?.getBoundingClientRect();
  const mouseX = rect ? e.clientX - rect.left : e.clientX;
  const mouseY = rect ? e.clientY - rect.top : e.clientY;

  const worldX = (mouseX - camera.panX) / camera.zoom;
  const worldY = (mouseY - camera.panY) / camera.zoom;

  camera.zoom = clampedZoom;
  camera.panX = mouseX - worldX * clampedZoom;
  camera.panY = mouseY - worldY * clampedZoom;

  triggerSave();
}

function zoomIn() {
  const centerWorld = screenToWorld(window.innerWidth / 2, window.innerHeight / 2);
  camera.zoom = Math.min(camera.zoom * 1.25, 5.0);
  camera.panX = window.innerWidth / 2 - centerWorld.x * camera.zoom;
  camera.panY = window.innerHeight / 2 - centerWorld.y * camera.zoom;
  triggerSave();
}

function zoomOut() {
  const centerWorld = screenToWorld(window.innerWidth / 2, window.innerHeight / 2);
  camera.zoom = Math.max(camera.zoom / 1.25, 0.1);
  camera.panX = window.innerWidth / 2 - centerWorld.x * camera.zoom;
  camera.panY = window.innerHeight / 2 - centerWorld.y * camera.zoom;
  triggerSave();
}

function fitAll() {
  if (objects.value.length === 0) {
    camera.panX = 0;
    camera.panY = 0;
    camera.zoom = 1;
    triggerSave();
    return;
  }

  previousCamera.value = { panX: camera.panX, panY: camera.panY, zoom: camera.zoom };

  let minX = Infinity;
  let minY = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;

  for (const obj of objects.value) {
    minX = Math.min(minX, obj.x);
    minY = Math.min(minY, obj.y);
    maxX = Math.max(maxX, obj.x + obj.width);
    maxY = Math.max(maxY, obj.y + obj.height);
  }

  const padding = 100;
  const bboxW = maxX - minX + padding * 2;
  const bboxH = maxY - minY + padding * 2;

  const viewW = window.innerWidth;
  const viewH = window.innerHeight - 48; // minus topbar

  const zoomX = viewW / bboxW;
  const zoomY = viewH / bboxH;
  const fitZoom = Math.min(Math.max(Math.min(zoomX, zoomY), 0.15), 1.5);

  const centerX = (minX + maxX) / 2;
  const centerY = (minY + maxY) / 2;

  camera.zoom = fitZoom;
  camera.panX = viewW / 2 - centerX * fitZoom;
  camera.panY = (viewH / 2 + 48) - centerY * fitZoom;

  triggerSave();
  showToast('Fit view');
}

function resetCamera() {
  previousCamera.value = { panX: camera.panX, panY: camera.panY, zoom: camera.zoom };
  camera.panX = 0;
  camera.panY = 0;
  camera.zoom = 1;
  triggerSave();
  showToast('Reset view (100%)');
}

function restorePreviousCamera() {
  if (previousCamera.value) {
    const tmp = { panX: camera.panX, panY: camera.panY, zoom: camera.zoom };
    camera.panX = previousCamera.value.panX;
    camera.panY = previousCamera.value.panY;
    camera.zoom = previousCamera.value.zoom;
    previousCamera.value = tmp;
    triggerSave();
    showToast('Restored previous camera');
  }
}

// --- VIEWPORT MOUSE INTERACTIONS ---
function onViewportMouseDown(e: MouseEvent) {
  if (e.button === 1) return; // Handled globally for pan

  // Hand tool: Left drag pans
  if (activeTool.value === 'hand' || isSpaceDown.value) {
    if (e.button === 0) {
      startPanning(e.clientX, e.clientY);
      return;
    }
  }

  // Text tool: Click to place text note
  if (activeTool.value === 'text' && e.button === 0) {
    const world = screenToWorld(e.clientX, e.clientY);
    spawnTextNote(world.x, world.y);
    activeTool.value = 'select';
    return;
  }

  // Connector tool: Click/drag anywhere on canvas or near an object
  if (activeTool.value === 'connector' && e.button === 0) {
    const world = screenToWorld(e.clientX, e.clientY);
    const snap = findClosestAnchor(world.x, world.y);
    const startX = snap ? snap.x : world.x;
    const startY = snap ? snap.y : world.y;

    connectingState.value = {
      fromId: snap?.objectId,
      fromAnchor: snap?.anchor,
      startX,
      startY,
    };
    snapTarget.value = snap;
    return;
  }

  // Marquee or Select tool on empty space
  if (e.button === 0) {
    selectedConnectorId.value = null;
    if (!e.shiftKey) {
      selectedIds.value.clear();
    }
    marqueeStart.value = { x: e.clientX, y: e.clientY };
    marqueeCurrent.value = { x: e.clientX, y: e.clientY };
  }
}

function onMouseMove(e: MouseEvent) {
  // 1. Connector Drafting Live Preview
  if (connectingState.value) {
    const world = screenToWorld(e.clientX, e.clientY);
    const snap = findClosestAnchor(world.x, world.y, connectingState.value.fromId);
    snapTarget.value = snap;

    const toX = snap ? snap.x : world.x;
    const toY = snap ? snap.y : world.y;

    draftConnector.value = {
      x1: connectingState.value.startX,
      y1: connectingState.value.startY,
      x2: toX,
      y2: toY,
      path: buildDraftPath(connectingState.value.startX, connectingState.value.startY, toX, toY),
    };
    return;
  }

  // 2. Connector Endpoint Dragging (Excalidraw-like interactive editing)
  if (endpointDragState.value) {
    const conn = connectors.value.find((c) => c.id === endpointDragState.value?.connectorId);
    if (conn) {
      const world = screenToWorld(e.clientX, e.clientY);
      const excludeObjId = endpointDragState.value.endpoint === 'start' ? conn.to_id : conn.from_id;
      const snap = findClosestAnchor(world.x, world.y, excludeObjId);
      snapTarget.value = snap;

      const targetX = snap ? snap.x : world.x;
      const targetY = snap ? snap.y : world.y;

      if (endpointDragState.value.endpoint === 'start') {
        conn.from_id = snap?.objectId;
        conn.from_anchor = snap?.anchor;
        conn.from_x = targetX;
        conn.from_y = targetY;
      } else {
        conn.to_id = snap?.objectId;
        conn.to_anchor = snap?.anchor;
        conn.to_x = targetX;
        conn.to_y = targetY;
      }
    }
    return;
  }

  // 3. Resizing Objects
  if (resizeState.value) {
    const { objectId, handle, startX, startY, initialX, initialY, initialW, initialH, aspectRatio } = resizeState.value;
    const dx = (e.clientX - startX) / camera.zoom;
    const dy = (e.clientY - startY) / camera.zoom;

    const obj = objects.value.find((o) => o.id === objectId);
    if (!obj) return;

    let newW = initialW;
    let newH = initialH;
    let newX = initialX;
    let newY = initialY;

    if (handle.includes('e')) newW = Math.max(40, initialW + dx);
    if (handle.includes('s')) newH = Math.max(30, initialH + dy);
    if (handle.includes('w')) {
      const candidateW = initialW - dx;
      if (candidateW >= 40) {
        newW = candidateW;
        newX = initialX + dx;
      }
    }
    if (handle.includes('n')) {
      const candidateH = initialH - dy;
      if (candidateH >= 30) {
        newH = candidateH;
        newY = initialY + dy;
      }
    }

    // Preserve aspect ratio for images unless shift is held
    if (obj.kind === 'image' && !e.shiftKey) {
      if (handle.includes('e') || handle.includes('w')) {
        newH = Math.round(newW / aspectRatio);
      } else {
        newW = Math.round(newH * aspectRatio);
      }
    }

    obj.x = Math.round(newX);
    obj.y = Math.round(newY);
    obj.width = Math.round(newW);
    obj.height = Math.round(newH);
    return;
  }

  // 4. Dragging Selected Objects
  if (isDraggingObjects.value && dragStartPos.value) {
    const dx = (e.clientX - dragStartPos.value.x) / camera.zoom;
    const dy = (e.clientY - dragStartPos.value.y) / camera.zoom;

    for (const obj of objects.value) {
      if (selectedIds.value.has(obj.id)) {
        const init = dragObjectOffsets.value.get(obj.id);
        if (init) {
          obj.x = Math.round(init.x + dx);
          obj.y = Math.round(init.y + dy);
        }
      }
    }
    return;
  }

  // 5. Marquee Selection
  if (marqueeStart.value) {
    marqueeCurrent.value = { x: e.clientX, y: e.clientY };

    const wP1 = screenToWorld(marqueeStart.value.x, marqueeStart.value.y);
    const wP2 = screenToWorld(e.clientX, e.clientY);

    const minX = Math.min(wP1.x, wP2.x);
    const minY = Math.min(wP1.y, wP2.y);
    const maxX = Math.max(wP1.x, wP2.x);
    const maxY = Math.max(wP1.y, wP2.y);

    const newSelected = new Set(e.shiftKey ? selectedIds.value : []);
    for (const obj of objects.value) {
      const objRight = obj.x + obj.width;
      const objBottom = obj.y + obj.height;
      const intersects =
        obj.x <= maxX && objRight >= minX && obj.y <= maxY && objBottom >= minY;
      if (intersects) {
        newSelected.add(obj.id);
      }
    }
    selectedIds.value = newSelected;
  }
}

function onMouseUp(e: MouseEvent) {
  // 1. Finish Connector Drafting
  if (connectingState.value) {
    const world = screenToWorld(e.clientX, e.clientY);
    const dist = Math.hypot(world.x - connectingState.value.startX, world.y - connectingState.value.startY);

    if (dist >= 10) {
      const toX = snapTarget.value ? snapTarget.value.x : world.x;
      const toY = snapTarget.value ? snapTarget.value.y : world.y;

      const newConn: BoardConnector = {
        id: crypto.randomUUID(),
        board_id: currentBoardId.value,
        from_id: connectingState.value.fromId,
        from_anchor: connectingState.value.fromAnchor,
        from_x: connectingState.value.startX,
        from_y: connectingState.value.startY,
        to_id: snapTarget.value?.objectId,
        to_anchor: snapTarget.value?.anchor,
        to_x: toX,
        to_y: toY,
        line_type: 'curved',
        arrow_start: 'none',
        arrow_end: 'arrow',
        stroke_style: 'solid',
        color: '#c084fc',
        stroke_width: 2.0,
        created_at: Date.now(),
        updated_at: Date.now(),
      };

      pushUndo();
      connectors.value.push(newConn);
      selectedConnectorId.value = newConn.id;
      selectedIds.value.clear();
      triggerSave();
      showToast('Created connector');
    }

    connectingState.value = null;
    draftConnector.value = null;
    snapTarget.value = null;
    return;
  }

  // 2. Finish Endpoint Dragging
  if (endpointDragState.value) {
    endpointDragState.value = null;
    snapTarget.value = null;
    triggerSave();
    return;
  }

  // 3. Finish Object Dragging
  if (isDraggingObjects.value) {
    isDraggingObjects.value = false;
    dragStartPos.value = null;
    dragObjectOffsets.value.clear();
    triggerSave();
  }

  // 4. Finish Resizing
  if (resizeState.value) {
    resizeState.value = null;
    triggerSave();
  }

  // 5. Finish Marquee
  if (marqueeStart.value) {
    marqueeStart.value = null;
    marqueeCurrent.value = null;
  }

  if (isPanning.value && e.button === 0) {
    stopPanning();
  }
}

// --- OBJECT INTERACTIONS ---
function onObjectSelect(id: string, multi: boolean) {
  selectedConnectorId.value = null;
  if (multi) {
    if (selectedIds.value.has(id)) {
      selectedIds.value.delete(id);
    } else {
      selectedIds.value.add(id);
    }
  } else {
    if (!selectedIds.value.has(id)) {
      selectedIds.value = new Set([id]);
    }
  }
}

function onStartObjectDrag(id: string, e: MouseEvent) {
  if (activeTool.value === 'hand' || isSpaceDown.value || activeTool.value === 'connector') return;

  pushUndo();

  if (!selectedIds.value.has(id)) {
    selectedIds.value = new Set([id]);
  }

  isDraggingObjects.value = true;
  dragStartPos.value = { x: e.clientX, y: e.clientY };
  dragObjectOffsets.value.clear();

  for (const obj of objects.value) {
    if (selectedIds.value.has(obj.id)) {
      dragObjectOffsets.value.set(obj.id, { x: obj.x, y: obj.y });
    }
  }
}

function onStartObjectResize(id: string, handle: string, e: MouseEvent) {
  pushUndo();
  const obj = objects.value.find((o) => o.id === id);
  if (!obj) return;

  resizeState.value = {
    objectId: id,
    handle,
    startX: e.clientX,
    startY: e.clientY,
    initialX: obj.x,
    initialY: obj.y,
    initialW: obj.width,
    initialH: obj.height,
    aspectRatio: obj.width / obj.height,
  };
}

// Connector dragging from anchor
function onStartConnect(objectId: string, anchor: AnchorPosition, e: MouseEvent) {
  const obj = objects.value.find((o) => o.id === objectId);
  if (!obj) return;

  const p = getObjectAnchorPoint(obj, anchor);
  connectingState.value = {
    fromId: objectId,
    fromAnchor: anchor,
    startX: p.x,
    startY: p.y,
  };
}

function onSelectConnector(id: string, e: MouseEvent) {
  selectedIds.value.clear();
  selectedConnectorId.value = id;
}

function onStartDragEndpoint(connectorId: string, endpoint: 'start' | 'end', e: MouseEvent) {
  pushUndo();
  selectedConnectorId.value = connectorId;
  endpointDragState.value = {
    connectorId,
    endpoint,
  };
}

function updateSelectedConnector(updated: Partial<BoardConnector>) {
  if (!selectedConnectorId.value) return;
  const conn = connectors.value.find((c) => c.id === selectedConnectorId.value);
  if (conn) {
    pushUndo();
    Object.assign(conn, updated);
    conn.updated_at = Date.now();
    triggerSave();
  }
}

function deleteConnector(id: string) {
  pushUndo();
  connectors.value = connectors.value.filter((c) => c.id !== id);
  if (selectedConnectorId.value === id) {
    selectedConnectorId.value = null;
  }
  triggerSave();
  showToast('Deleted connector');
}

function startEditing(id: string) {
  const obj = objects.value.find((o) => o.id === id);
  if (obj && (obj.kind === 'text' || obj.kind === 'code')) {
    editingId.value = id;
    selectedIds.value = new Set([id]);
  }
}

function exitEditing() {
  editingId.value = null;
}

function updateObjectContent(id: string, content: string) {
  const obj = objects.value.find((o) => o.id === id);
  if (obj) {
    obj.content = content;
    obj.updated_at = Date.now();
    triggerSave();
  }
}

function updateObjectTitle(id: string, title: string) {
  pushUndo();
  const obj = objects.value.find((o) => o.id === id);
  if (obj) {
    obj.title = title;
    obj.updated_at = Date.now();
    triggerSave();
    showToast(`Renamed to "${title}"`);
  }
}

function updateObjectMeta(id: string, meta: any) {
  const obj = objects.value.find((o) => o.id === id);
  if (obj) {
    obj.meta = { ...obj.meta, ...meta };
    obj.updated_at = Date.now();
    triggerSave();
  }
}

function copyObjectContent(obj: BoardObject) {
  if (obj.kind === 'image') {
    api.copyImage(obj.content);
    showToast('Copied image to clipboard');
  } else {
    api.copyText(obj.content);
    showToast('Copied content to clipboard');
  }
}

function deleteSingleObject(id: string) {
  pushUndo();
  objects.value = objects.value.filter((o) => o.id !== id);
  // Safely detach connectors instead of destroying whole diagram
  for (const c of connectors.value) {
    if (c.from_id === id) {
      c.from_id = undefined;
      c.from_anchor = undefined;
    }
    if (c.to_id === id) {
      c.to_id = undefined;
      c.to_anchor = undefined;
    }
  }
  selectedIds.value.delete(id);
  if (editingId.value === id) editingId.value = null;
  api.deleteObjects([id]);
  triggerSave();
  showToast('Deleted object');
}

function deleteSelected() {
  if (selectedIds.value.size === 0 || editingId.value) return;
  pushUndo();
  const idsToDelete = Array.from(selectedIds.value);
  objects.value = objects.value.filter((o) => !selectedIds.value.has(o.id));
  for (const c of connectors.value) {
    if (c.from_id && selectedIds.value.has(c.from_id)) {
      c.from_id = undefined;
      c.from_anchor = undefined;
    }
    if (c.to_id && selectedIds.value.has(c.to_id)) {
      c.to_id = undefined;
      c.to_anchor = undefined;
    }
  }
  selectedIds.value.clear();
  api.deleteObjects(idsToDelete);
  triggerSave();
  showToast(`Deleted ${idsToDelete.length} item(s)`);
}

function getNextZIndex(): number {
  if (objects.value.length === 0) return 1;
  return Math.max(...objects.value.map((o) => o.z_index)) + 1;
}

// --- Z-ORDER ACTIONS ---
function bringForward() {
  if (selectedIds.value.size === 0 || editingId.value) return;
  pushUndo();
  const sorted = [...objects.value].sort((a, b) => a.z_index - b.z_index);
  sorted.forEach((obj, idx) => { obj.z_index = idx + 1; });

  for (let i = sorted.length - 2; i >= 0; i--) {
    if (selectedIds.value.has(sorted[i].id) && !selectedIds.value.has(sorted[i + 1].id)) {
      const tempZ = sorted[i].z_index;
      sorted[i].z_index = sorted[i + 1].z_index;
      sorted[i + 1].z_index = tempZ;
      const temp = sorted[i];
      sorted[i] = sorted[i + 1];
      sorted[i + 1] = temp;
    }
  }
  triggerSave();
  showToast('Brought forward');
}

function sendBackward() {
  if (selectedIds.value.size === 0 || editingId.value) return;
  pushUndo();
  const sorted = [...objects.value].sort((a, b) => a.z_index - b.z_index);
  sorted.forEach((obj, idx) => { obj.z_index = idx + 1; });

  for (let i = 1; i < sorted.length; i++) {
    if (selectedIds.value.has(sorted[i].id) && !selectedIds.value.has(sorted[i - 1].id)) {
      const tempZ = sorted[i].z_index;
      sorted[i].z_index = sorted[i - 1].z_index;
      sorted[i - 1].z_index = tempZ;
      const temp = sorted[i];
      sorted[i] = sorted[i - 1];
      sorted[i - 1] = temp;
    }
  }
  triggerSave();
  showToast('Sent backward');
}

// --- CREATION ACTIONS ---
function spawnTextNote(worldX?: number, worldY?: number) {
  const centerWorld = screenToWorld(window.innerWidth / 2, window.innerHeight / 2);
  const targetX = worldX !== undefined ? worldX : centerWorld.x;
  const targetY = worldY !== undefined ? worldY : centerWorld.y;

  const newObj: BoardObject = {
    id: crypto.randomUUID(),
    board_id: currentBoardId.value,
    kind: 'text',
    x: Math.round(targetX - 160),
    y: Math.round(targetY - 110),
    width: 320,
    height: 220,
    z_index: getNextZIndex(),
    title: 'New Note',
    content: '- [ ] Item 1\n- [ ] Item 2',
    meta: { tags: ['#todo'] },
  };
  pushUndo();
  objects.value.push(newObj);
  selectedIds.value = new Set([newObj.id]);
  startEditing(newObj.id);
  triggerSave();
  showToast('Created text note');
}

function onSelectIconFromPalette(icon: IconItem) {
  const centerWorld = screenToWorld(window.innerWidth / 2, window.innerHeight / 2);
  const size = 68;
  const newObj: BoardObject = {
    id: crypto.randomUUID(),
    board_id: currentBoardId.value,
    kind: 'icon',
    x: Math.round(centerWorld.x - size / 2),
    y: Math.round(centerWorld.y - size / 2),
    width: size,
    height: size,
    z_index: getNextZIndex(),
    title: icon.name,
    content: icon.id,
    meta: {
      icon_id: icon.id,
      icon_category: icon.category,
      icon_label: icon.name,
    },
  };

  pushUndo();
  objects.value.push(newObj);
  selectedIds.value = new Set([newObj.id]);
  triggerSave();
  showToast(`Placed ${icon.name}`);
}

function onCreateLinkFromModal(payload: { url: string; title: string }) {
  isLinkModalOpen.value = false;
  const centerWorld = screenToWorld(window.innerWidth / 2, window.innerHeight / 2);
  const newObj: BoardObject = {
    id: crypto.randomUUID(),
    board_id: currentBoardId.value,
    kind: 'url',
    x: Math.round(centerWorld.x - 140),
    y: Math.round(centerWorld.y - 40),
    width: 280,
    height: 75,
    z_index: getNextZIndex(),
    title: payload.title,
    content: payload.url,
  };
  pushUndo();
  objects.value.push(newObj);
  selectedIds.value = new Set([newObj.id]);
  triggerSave();
  showToast('Created link card');
}

function triggerImageFileInput() {
  fileInputRef.value?.click();
}

function onImageFileSelected(e: Event) {
  const target = e.target as HTMLInputElement;
  if (!target.files || target.files.length === 0) return;
  const file = target.files[0];
  const reader = new FileReader();
  reader.onload = async () => {
    const base64Data = (reader.result as string).split(',')[1];
    const res = await api.saveImageFromBytes(base64Data, file.name);
    if (res.image_data) {
      spawnImageOnCanvas(res.image_data, res.width, res.height, file.name);
    }
  };
  reader.readAsDataURL(file);
  target.value = '';
}

function spawnImageOnCanvas(dataUri: string, naturalW?: number, naturalH?: number, filename?: string) {
  const centerWorld = screenToWorld(window.innerWidth / 2, window.innerHeight / 2);
  const origW = naturalW || 450;
  const origH = naturalH || 300;

  const viewWorldWidth = window.innerWidth / camera.zoom;
  const viewWorldHeight = window.innerHeight / camera.zoom;

  let targetW = origW;
  let targetH = origH;

  // Scale down if larger than 45% of viewport, but NEVER upscale small originals!
  const maxW = viewWorldWidth * 0.45;
  const maxH = viewWorldHeight * 0.45;

  if (targetW > maxW || targetH > maxH) {
    const scale = Math.min(maxW / targetW, maxH / targetH);
    targetW = Math.round(targetW * scale);
    targetH = Math.round(targetH * scale);
  }

  const newObj: BoardObject = {
    id: crypto.randomUUID(),
    board_id: currentBoardId.value,
    kind: 'image',
    x: Math.round(centerWorld.x - targetW / 2),
    y: Math.round(centerWorld.y - targetH / 2),
    width: targetW,
    height: targetH,
    z_index: getNextZIndex(),
    title: filename || 'Pasted Image',
    content: dataUri,
    meta: {
      original_filename: filename,
      natural_width: origW,
      natural_height: origH,
    },
  };

  pushUndo();
  objects.value.push(newObj);
  selectedIds.value = new Set([newObj.id]);
  triggerSave();
  showToast('Pasted image');
}

async function pasteFromClipboard() {
  const clip = await api.getClipboard();
  if (clip.kind === 'empty') {
    showToast('Clipboard is empty');
    return;
  }

  if (clip.kind === 'image' && clip.image_data) {
    spawnImageOnCanvas(clip.image_data, clip.width, clip.height, clip.filename);
  } else if (clip.kind === 'text' && clip.text) {
    const text = clip.text;
    const isUrl = /^https?:\/\/[^\s]+$/i.test(text.trim());
    const targetWorld = screenToWorld(mouseScreen.x, mouseScreen.y);

    if (isUrl) {
      const newObj: BoardObject = {
        id: crypto.randomUUID(),
        board_id: currentBoardId.value,
        kind: 'url',
        x: Math.round(targetWorld.x - 140),
        y: Math.round(targetWorld.y - 40),
        width: 280,
        height: 75,
        z_index: getNextZIndex(),
        title: text.replace(/^https?:\/\/(www\.)?/, '').split('/')[0],
        content: text.trim(),
      };
      pushUndo();
      objects.value.push(newObj);
      selectedIds.value = new Set([newObj.id]);
    } else {
      const isCode = text.includes('function') || text.includes('const ') || text.includes('import ') || text.includes('def ') || (text.includes('{') && text.includes('}'));
      const newObj: BoardObject = {
        id: crypto.randomUUID(),
        board_id: currentBoardId.value,
        kind: 'text',
        x: Math.round(targetWorld.x - 160),
        y: Math.round(targetWorld.y - 100),
        width: isCode ? 360 : 300,
        height: isCode ? 220 : 180,
        z_index: getNextZIndex(),
        title: isCode ? 'Snippet' : 'Note',
        content: text,
        meta: isCode ? { is_code: true } : undefined,
      };
      pushUndo();
      objects.value.push(newObj);
      selectedIds.value = new Set([newObj.id]);
    }
    triggerSave();
    showToast(`Pasted content`);
  }
}

function onGlobalPaste(e: ClipboardEvent) {
  if (editingId.value) return;
  pasteFromClipboard();
}

function onDrop(e: DragEvent) {
  if (!e.dataTransfer || editingId.value) return;
  const files = Array.from(e.dataTransfer.files);
  for (const file of files) {
    if (file.type.startsWith('image/')) {
      const reader = new FileReader();
      reader.onload = async () => {
        const base64Data = (reader.result as string).split(',')[1];
        const res = await api.saveImageFromBytes(base64Data, file.name);
        if (res.image_data) {
          spawnImageOnCanvas(res.image_data, res.width, res.height, file.name);
        }
      };
      reader.readAsDataURL(file);
    }
  }
}

function openClearConfirm() {
  isClearConfirmOpen.value = true;
}

function confirmClearBoard() {
  pushUndo();
  objects.value = [];
  connectors.value = [];
  selectedIds.value.clear();
  selectedConnectorId.value = null;
  isClearConfirmOpen.value = false;
  triggerSave();
  showToast('Board cleared');
}

// --- SEARCH TO FOCUS ---
function openSearch() {
  isSearchOpen.value = true;
}

function handleSearchJump(result: ExtendedSearchResult) {
  previousCamera.value = { panX: camera.panX, panY: camera.panY, zoom: camera.zoom };

  const targetZoom = Math.max(camera.zoom, 1.0);
  const targetCenterX = result.x + result.width / 2;
  const targetCenterY = result.y + result.height / 2;

  const viewW = window.innerWidth;
  const viewH = window.innerHeight - 48;

  camera.zoom = targetZoom;
  camera.panX = viewW / 2 - targetCenterX * targetZoom;
  camera.panY = (viewH / 2 + 48) - targetCenterY * targetZoom;

  if (result.isConnector) {
    selectedConnectorId.value = result.id;
    selectedIds.value.clear();
  } else {
    selectedIds.value = new Set([result.id]);
    selectedConnectorId.value = null;
    highlightedId.value = result.id;
    setTimeout(() => {
      if (highlightedId.value === result.id) {
        highlightedId.value = null;
      }
    }, 1800);
  }

  triggerSave();
}

// --- SHORTCUTS & KEYBOARD DISPATCHER ---
function onKeyDown(e: KeyboardEvent) {
  // 1. If shortcuts modal is open, let its own Escape listener handle it
  if (isShortcutsOpen.value) {
    if (e.key === 'Escape') {
      isShortcutsOpen.value = false;
      return;
    }
    return;
  }

  // 2. If search modal is open
  if (isSearchOpen.value) {
    if (e.key === 'Escape') {
      isSearchOpen.value = false;
      return;
    }
    return;
  }

  // 3. If link modal is open
  if (isLinkModalOpen.value) {
    if (e.key === 'Escape') {
      isLinkModalOpen.value = false;
      return;
    }
    return;
  }

  // 4. If icon palette is open
  if (isIconPaletteOpen.value) {
    if (e.key === 'Escape') {
      isIconPaletteOpen.value = false;
      return;
    }
    return;
  }

  // 5. If clear confirm dialog is open
  if (isClearConfirmOpen.value) {
    if (e.key === 'Escape') {
      isClearConfirmOpen.value = false;
      return;
    }
    return;
  }

  // 6. Check if user is typing in an active text input, textarea, or CodeMirror editor
  const target = e.target as HTMLElement | null;
  const isTyping =
    editingId.value !== null ||
    (target && (
      target.tagName === 'INPUT' ||
      target.tagName === 'TEXTAREA' ||
      target.isContentEditable ||
      Boolean(target.closest('.cm-editor')) ||
      Boolean(target.closest('.cm-content'))
    ));

  if (isTyping) {
    if (e.key === 'Escape' && editingId.value) {
      exitEditing();
    }
    return;
  }

  // --- GENERAL CANVAS MODE SHORTCUTS ---

  // Escape: prioritize cancellation of ongoing interaction, then selection, then tool
  if (e.key === 'Escape') {
    e.preventDefault();
    if (connectingState.value) {
      connectingState.value = null;
      draftConnector.value = null;
      snapTarget.value = null;
      return;
    }
    if (endpointDragState.value) {
      endpointDragState.value = null;
      snapTarget.value = null;
      return;
    }
    if (selectedConnectorId.value) {
      selectedConnectorId.value = null;
      return;
    }
    if (selectedIds.value.size > 0) {
      selectedIds.value.clear();
      return;
    }
    if (activeTool.value !== 'select') {
      activeTool.value = 'select';
      return;
    }
    return;
  }

  // Space for pan mode
  if (e.code === 'Space' && !isSpaceDown.value) {
    isSpaceDown.value = true;
  }

  // Tool shortcuts
  if (e.key === 'v' || e.key === 'V') {
    activeTool.value = 'select';
    return;
  }
  if (e.key === 'h' || e.key === 'H') {
    activeTool.value = 'hand';
    return;
  }
  if (e.key === 'c' || e.key === 'C') {
    activeTool.value = 'connector';
    showToast('Connector mode: drag between any objects or canvas');
    return;
  }
  if (e.key === 'n' || e.key === 'N' || e.key === 't' || e.key === 'T') {
    e.preventDefault();
    spawnTextNote();
    return;
  }

  // Z-Order: `[` and `]`
  if (e.key === ']') {
    e.preventDefault();
    bringForward();
    return;
  }
  if (e.key === '[') {
    e.preventDefault();
    sendBackward();
    return;
  }

  // Search shortcut `/`
  if (e.key === '/') {
    e.preventDefault();
    openSearch();
    return;
  }

  // Shortcuts modal `?`
  if (e.key === '?') {
    e.preventDefault();
    isShortcutsOpen.value = true;
    return;
  }

  // Edit selected note: `i` or `Enter`
  if ((e.key === 'i' || e.key === 'Enter') && selectedIds.value.size === 1) {
    const singleId = Array.from(selectedIds.value)[0];
    const targetObj = objects.value.find((o) => o.id === singleId);
    if (targetObj && (targetObj.kind === 'text' || targetObj.kind === 'code')) {
      e.preventDefault();
      startEditing(singleId);
      return;
    }
  }

  // Delete selected objects or connector
  if (e.key === 'Delete' || e.key === 'Backspace') {
    if (selectedConnectorId.value) {
      e.preventDefault();
      deleteConnector(selectedConnectorId.value);
      return;
    }
    if (selectedIds.value.size > 0) {
      e.preventDefault();
      deleteSelected();
      return;
    }
  }

  // Copy: `Ctrl+C`
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'c' && selectedIds.value.size === 1) {
    const id = Array.from(selectedIds.value)[0];
    const obj = objects.value.find((o) => o.id === id);
    if (obj) {
      e.preventDefault();
      copyObjectContent(obj);
      return;
    }
  }

  // Jump Back to previous camera: `Ctrl+O`
  if ((e.ctrlKey || e.metaKey) && e.key === 'o') {
    if (previousCamera.value) {
      e.preventDefault();
      restorePreviousCamera();
      return;
    }
  }

  // Fit all: `Ctrl+0`
  if ((e.ctrlKey || e.metaKey) && e.key === '0') {
    e.preventDefault();
    fitAll();
    return;
  }

  // Reset 100%: `Ctrl+1`
  if ((e.ctrlKey || e.metaKey) && e.key === '1') {
    e.preventDefault();
    resetCamera();
    return;
  }

  // Undo: `Ctrl+Z`
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'z' && !e.shiftKey) {
    e.preventDefault();
    undo();
    return;
  }

  // Redo: `Ctrl+Shift+Z` or `Ctrl+Y`
  if (((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === 'z') || ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'y')) {
    e.preventDefault();
    redo();
    return;
  }

  // Nudge selection with Arrow keys or h/j/k/l
  const step = e.shiftKey ? 40 : 8;
  let nudgeX = 0;
  let nudgeY = 0;
  if (e.key === 'ArrowLeft' || e.key === 'h') nudgeX = -step;
  if (e.key === 'ArrowRight' || e.key === 'l') nudgeX = step;
  if (e.key === 'ArrowUp' || e.key === 'k') nudgeY = -step;
  if (e.key === 'ArrowDown' || e.key === 'j') nudgeY = step;

  if (nudgeX !== 0 || nudgeY !== 0) {
    if (selectedIds.value.size > 0) {
      e.preventDefault();
      nudgeSelection(nudgeX, nudgeY);
    }
  }
}

function onKeyUp(e: KeyboardEvent) {
  if (e.code === 'Space') {
    isSpaceDown.value = false;
  }
}

function nudgeSelection(dx: number, dy: number) {
  pushUndo();
  for (const obj of objects.value) {
    if (selectedIds.value.has(obj.id)) {
      obj.x += dx;
      obj.y += dy;
    }
  }
  triggerSave();
}

// --- PERSISTENCE ---
let saveDebounceTimer: any = null;
function triggerSave() {
  if (saveDebounceTimer) clearTimeout(saveDebounceTimer);
  saveDebounceTimer = setTimeout(async () => {
    try {
      await api.saveBoard(
        currentBoardId.value,
        camera.panX,
        camera.panY,
        camera.zoom,
        objects.value,
        connectors.value
      );
    } catch (err) {
      console.error('Failed to autosave board:', err);
    }
  }, 400);
}

async function toggleAlwaysOnTop() {
  alwaysOnTop.value = !alwaysOnTop.value;
  await api.setAlwaysOnTop(alwaysOnTop.value);
  showToast(alwaysOnTop.value ? 'Pinboard is always on top' : 'Always on top disabled');
}
</script>

<style scoped>
.pinboard-workspace {
  position: relative;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  background-color: #0b0c14;
  user-select: none;
}

.canvas-viewport {
  position: absolute;
  top: 48px;
  left: 0;
  right: 0;
  bottom: 0;
  overflow: hidden;
  outline: none;
  background-color: #0b0c14;
  background-image: radial-gradient(rgba(255, 255, 255, 0.08) 1.2px, transparent 1.2px);
}

.canvas-world {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: auto;
}

/* Marquee Selection Box */
.selection-marquee {
  position: absolute;
  border: 1px dashed rgba(192, 132, 252, 0.85);
  background: rgba(192, 132, 252, 0.12);
  pointer-events: none;
  z-index: 10030;
}

/* Toast */
.canvas-toast {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(20, 21, 37, 0.95);
  backdrop-filter: blur(8px);
  color: #f8fafc;
  border: 1px solid #2d304a;
  border-radius: 8px;
  padding: 8px 16px;
  font-size: 12.5px;
  font-weight: 500;
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.6);
  z-index: 10090;
  pointer-events: none;
}

.toast-enter-active,
.toast-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}
.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translate(-50%, 8px);
}

/* Flash highlight animation for search jump */
:deep(.flash-highlight) {
  animation: flashRing 1.8s ease-out;
}

@keyframes flashRing {
  0% { box-shadow: 0 0 0 6px rgba(192, 132, 252, 0.9); }
  50% { box-shadow: 0 0 0 12px rgba(192, 132, 252, 0.4); }
  100% { box-shadow: none; }
}

/* Clear Board Confirmation Dialog */
.confirm-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(4px);
  z-index: 10090;
  display: flex;
  align-items: center;
  justify-content: center;
}

.confirm-dialog {
  width: 380px;
  max-width: 90vw;
  background: #141525;
  border: 1px solid #2a2c42;
  border-radius: 14px;
  padding: 20px;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.75);
}

.confirm-title {
  font-size: 15px;
  font-weight: 700;
  color: #f1f5f9;
  margin-bottom: 8px;
}

.confirm-msg {
  font-size: 13px;
  color: #94a3b8;
  line-height: 1.5;
  margin-bottom: 18px;
}

.confirm-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 10px;
}

.btn {
  padding: 7px 16px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-secondary {
  background: transparent;
  border: 1px solid #2f324d;
  color: #cbd5e1;
}
.btn-secondary:hover {
  background: rgba(255, 255, 255, 0.06);
  color: #f8fafc;
}

.btn-danger {
  background: #e11d48;
  border: 1px solid #f43f5e;
  color: #ffffff;
}
.btn-danger:hover {
  background: #f43f5e;
}
</style>
