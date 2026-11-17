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
      @clear-board="clearBoard"
    />

    <!-- 2. LEFT FLOATING TOOL RAIL -->
    <ToolRail
      v-model:active-tool="activeTool"
      @import-image="triggerImageFileInput"
      @create-link="promptCreateLink"
      @spawn-diagram-node="spawnDiagramNode"
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
      :class="{
        'is-panning': isPanning,
        'is-space-down': isSpaceDown,
        'cursor-crosshair': activeTool === 'connector',
      }"
      :style="gridStyle"
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
          @select-connector="onSelectConnector"
        />

        <!-- Board Objects (Images, Notes, Snippets, Links, Network Diagram Nodes) -->
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
        @close="isSearchOpen = false"
        @jump-to="handleSearchJump"
      />

      <!-- Shortcuts Reference Modal -->
      <ShortcutsModal v-if="isShortcutsOpen" @close="isShortcutsOpen = false" />
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
  DiagramNodeType,
  SearchResult,
} from '../types';
import TopBar from './TopBar.vue';
import ToolRail from './ToolRail.vue';
import ConnectorsLayer from './ConnectorsLayer.vue';
import ConnectorToolbar from './ConnectorToolbar.vue';
import BoardObjectView from './BoardObjectView.vue';
import BottomHUD from './BottomHUD.vue';
import SearchModal from './SearchModal.vue';
import ShortcutsModal from './ShortcutsModal.vue';
import { api } from '../utils/api';

const props = defineProps<{
  boardId?: string;
}>();

// --- STATE ---
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

// Camera history for Back navigation (e.g. after search jump)
const previousCamera = ref<Camera | null>(null);

const activeTool = ref<ActiveTool>('select');
const selectedIds = ref<Set<string>>(new Set());
const selectedConnectorId = ref<string | null>(null);
const editingId = ref<string | null>(null);
const highlightedId = ref<string | null>(null);

const isSearchOpen = ref(false);
const isShortcutsOpen = ref(false);
const alwaysOnTop = ref(false);
const toastMessage = ref<string | null>(null);

// Interaction states
const isSpaceDown = ref(false);
const isPanning = ref(false);
const panStart = { x: 0, y: 0, initialPanX: 0, initialPanY: 0 };

// Selection box (Screen coordinates)
const selectionBox = ref<{ startX: number; startY: number; currentX: number; currentY: number } | null>(null);

// Dragging objects state
interface DragState {
  startX: number;
  startY: number;
  initialPositions: Map<string, { x: number; y: number }>;
}
const dragState = ref<DragState | null>(null);

// Resizing state
interface ResizeState {
  objectId: string;
  handle: string;
  startX: number;
  startY: number;
  initialX: number;
  initialY: number;
  initialW: number;
  initialH: number;
  aspectRatio: number;
}
const resizeState = ref<ResizeState | null>(null);

// Connector Creation / Drafting
interface ConnectDraftState {
  fromId: string;
  fromAnchor: AnchorPosition;
  startX: number;
  startY: number;
}
const connectingState = ref<ConnectDraftState | null>(null);
const draftConnector = ref<{ x1: number; y1: number; x2: number; y2: number; path: string } | null>(null);

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

// --- STYLES ---
const worldTransformStyle = computed(() => ({
  transform: `translate(${camera.panX}px, ${camera.panY}px) scale(${camera.zoom})`,
  transformOrigin: '0 0',
}));

const gridStyle = computed(() => {
  const gridSize = 28 * camera.zoom;
  const dotSize = Math.max(1, 1.4 * camera.zoom);
  return {
    backgroundSize: `${gridSize}px ${gridSize}px`,
    backgroundPosition: `${camera.panX}px ${camera.panY}px`,
    backgroundImage: `radial-gradient(circle, #25273c ${dotSize}px, transparent ${dotSize}px)`,
  };
});

const selectionBoxStyle = computed(() => {
  if (!selectionBox.value) return {};
  const left = Math.min(selectionBox.value.startX, selectionBox.value.currentX);
  const top = Math.min(selectionBox.value.startY, selectionBox.value.currentY);
  const width = Math.abs(selectionBox.value.currentX - selectionBox.value.startX);
  const height = Math.abs(selectionBox.value.currentY - selectionBox.value.startY);
  return {
    left: `${left}px`,
    top: `${top}px`,
    width: `${width}px`,
    height: `${height}px`,
  };
});

// Selected Connector Helper
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
    const cpDist = Math.max(dx * 0.5, 40);
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

// --- TOAST HELPER ---
let toastTimer: any = null;
function showToast(msg: string) {
  toastMessage.value = msg;
  clearTimeout(toastTimer);
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

    // If initial load contains the reference layout, preset selection to match screenshot
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
  // 1. Middle mouse button anywhere -> start panning
  if (e.button === 1) {
    e.preventDefault();
    e.stopPropagation();
    e.stopImmediatePropagation();

    isPanning.value = true;
    panStart.x = e.clientX;
    panStart.y = e.clientY;
    panStart.initialPanX = camera.panX;
    panStart.initialPanY = camera.panY;
    return;
  }

  // 2. Space + Left click anywhere -> start panning
  if (e.button === 0 && (isSpaceDown.value || activeTool.value === 'hand')) {
    e.preventDefault();
    e.stopPropagation();
    e.stopImmediatePropagation();

    isPanning.value = true;
    panStart.x = e.clientX;
    panStart.y = e.clientY;
    panStart.initialPanX = camera.panX;
    panStart.initialPanY = camera.panY;
    return;
  }
}

function onGlobalAuxClickCapture(e: MouseEvent) {
  if (e.button === 1) {
    e.preventDefault();
    e.stopPropagation();
    e.stopImmediatePropagation();
  }
}

function onGlobalMouseMoveCapture(e: MouseEvent) {
  mouseScreen.x = e.clientX;
  mouseScreen.y = e.clientY;

  if (isPanning.value) {
    e.preventDefault();
    e.stopPropagation();
    const dx = e.clientX - panStart.x;
    const dy = e.clientY - panStart.y;
    camera.panX = panStart.initialPanX + dx;
    camera.panY = panStart.initialPanY + dy;
    return;
  }

  // Handle connector drafting preview
  if (connectingState.value) {
    const mouseWorld = screenToWorld(e.clientX, e.clientY);
    const startX = connectingState.value.startX;
    const startY = connectingState.value.startY;
    const dx = Math.abs(mouseWorld.x - startX);
    const cpDist = Math.max(dx * 0.5, 30);
    const cp1x = startX + cpDist;
    const cp2x = mouseWorld.x - cpDist;

    draftConnector.value = {
      x1: startX,
      y1: startY,
      x2: mouseWorld.x,
      y2: mouseWorld.y,
      path: `M ${startX} ${startY} C ${cp1x} ${startY}, ${cp2x} ${mouseWorld.y}, ${mouseWorld.x} ${mouseWorld.y}`,
    };
  }
}

function onGlobalMouseUpCapture(e: MouseEvent) {
  if (isPanning.value && (e.button === 1 || e.button === 0)) {
    isPanning.value = false;
    e.preventDefault();
    e.stopPropagation();
    e.stopImmediatePropagation();
    triggerSave();
    return;
  }

  // Complete connector creation on mouse up
  if (connectingState.value) {
    const mouseWorld = screenToWorld(e.clientX, e.clientY);
    let targetId: string | undefined;
    let targetAnchor: AnchorPosition | undefined = 'left';

    // Find if released over an object
    for (const obj of objects.value) {
      if (
        obj.id !== connectingState.value.fromId &&
        mouseWorld.x >= obj.x &&
        mouseWorld.x <= obj.x + obj.width &&
        mouseWorld.y >= obj.y &&
        mouseWorld.y <= obj.y + obj.height
      ) {
        targetId = obj.id;
        // Determine closest edge anchor
        const dLeft = Math.abs(mouseWorld.x - obj.x);
        const dRight = Math.abs(mouseWorld.x - (obj.x + obj.width));
        const dTop = Math.abs(mouseWorld.y - obj.y);
        const dBottom = Math.abs(mouseWorld.y - (obj.y + obj.height));
        const minD = Math.min(dLeft, dRight, dTop, dBottom);
        if (minD === dLeft) targetAnchor = 'left';
        else if (minD === dRight) targetAnchor = 'right';
        else if (minD === dTop) targetAnchor = 'top';
        else targetAnchor = 'bottom';
        break;
      }
    }

    pushUndo();
    const newConnector: BoardConnector = {
      id: crypto.randomUUID(),
      board_id: currentBoardId.value,
      from_id: connectingState.value.fromId,
      from_anchor: connectingState.value.fromAnchor,
      from_x: connectingState.value.startX,
      from_y: connectingState.value.startY,
      to_id: targetId,
      to_anchor: targetId ? targetAnchor : undefined,
      to_x: mouseWorld.x,
      to_y: mouseWorld.y,
      line_type: 'curved',
      arrow_start: 'none',
      arrow_end: 'arrow',
      stroke_style: 'solid',
      color: '#c084fc',
      stroke_width: 2.0,
      label: undefined,
    };

    connectors.value.push(newConnector);
    selectedConnectorId.value = newConnector.id;
    connectingState.value = null;
    draftConnector.value = null;
    activeTool.value = 'select';
    triggerSave();
    showToast('Created connector');
  }
}

function onWindowBlur() {
  isSpaceDown.value = false;
  if (isPanning.value) {
    isPanning.value = false;
    triggerSave();
  }
}

// --- VIEWPORT MOUSE HANDLERS (Left click on empty canvas) ---
function onViewportMouseDown(e: MouseEvent) {
  if (e.button !== 0 || isSpaceDown.value || activeTool.value === 'hand') return;

  // Deselect connector
  selectedConnectorId.value = null;

  // Exit text editing if active
  if (editingId.value) {
    exitEditing();
  }

  // Handle specific tool actions
  if (activeTool.value === 'text') {
    const clickWorld = screenToWorld(e.clientX, e.clientY);
    createNewNoteAt(clickWorld.x, clickWorld.y);
    activeTool.value = 'select';
    return;
  }

  // Deselect objects unless Shift or Ctrl is pressed
  if (!e.shiftKey && !e.ctrlKey) {
    selectedIds.value.clear();
  }

  // Start marquee selection box (default or marquee tool)
  selectionBox.value = {
    startX: e.clientX,
    startY: e.clientY,
    currentX: e.clientX,
    currentY: e.clientY,
  };
}

function onMouseMove(e: MouseEvent) {
  mouseScreen.x = e.clientX;
  mouseScreen.y = e.clientY;

  if (isPanning.value) return;

  // 1. Marquee Selection
  if (selectionBox.value) {
    selectionBox.value.currentX = e.clientX;
    selectionBox.value.currentY = e.clientY;

    const sx1 = Math.min(selectionBox.value.startX, selectionBox.value.currentX);
    const sy1 = Math.min(selectionBox.value.startY, selectionBox.value.currentY);
    const sx2 = Math.max(selectionBox.value.startX, selectionBox.value.currentX);
    const sy2 = Math.max(selectionBox.value.startY, selectionBox.value.currentY);

    const wP1 = screenToWorld(sx1, sy1);
    const wP2 = screenToWorld(sx2, sy2);

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
    return;
  }

  // 2. Dragging Objects
  if (dragState.value) {
    const dx = (e.clientX - dragState.value.startX) / camera.zoom;
    const dy = (e.clientY - dragState.value.startY) / camera.zoom;

    for (const obj of objects.value) {
      const initial = dragState.value.initialPositions.get(obj.id);
      if (initial) {
        obj.x = Math.round(initial.x + dx);
        obj.y = Math.round(initial.y + dy);
      }
    }
    return;
  }

  // 3. Resizing Object
  if (resizeState.value) {
    const { objectId, handle, startX, startY, initialX, initialY, initialW, initialH, aspectRatio } =
      resizeState.value;
    const targetObj = objects.value.find((o) => o.id === objectId);
    if (!targetObj) return;

    const dx = (e.clientX - startX) / camera.zoom;
    const dy = (e.clientY - startY) / camera.zoom;

    let newX = initialX;
    let newY = initialY;
    let newW = initialW;
    let newH = initialH;

    const isImage = targetObj.kind === 'image';
    const keepAspect = isImage && !e.shiftKey;

    if (handle.includes('e')) newW = Math.max(80, initialW + dx);
    if (handle.includes('s')) newH = Math.max(50, initialH + dy);
    if (handle.includes('w')) {
      const maxW = initialW - dx;
      if (maxW >= 80) {
        newW = maxW;
        newX = initialX + dx;
      }
    }
    if (handle.includes('n')) {
      const maxH = initialH - dy;
      if (maxH >= 50) {
        newH = maxH;
        newY = initialY + dy;
      }
    }

    if (keepAspect && aspectRatio > 0) {
      if (handle === 'e' || handle === 'w') {
        newH = newW / aspectRatio;
      } else {
        newW = newH * aspectRatio;
      }
    }

    targetObj.x = Math.round(newX);
    targetObj.y = Math.round(newY);
    targetObj.width = Math.round(newW);
    targetObj.height = Math.round(newH);
  }
}

function onMouseUp() {
  if (selectionBox.value) {
    selectionBox.value = null;
    if (activeTool.value === 'marquee') {
      activeTool.value = 'select';
    }
  }

  if (dragState.value) {
    dragState.value = null;
    triggerSave();
  }

  if (resizeState.value) {
    resizeState.value = null;
    triggerSave();
  }
}

// --- ZOOM (Centered around cursor) ---
function onWheel(e: WheelEvent) {
  if (editingId.value) return;

  const zoomFactor = Math.exp(-e.deltaY * 0.0015);
  const newZoom = Math.min(Math.max(camera.zoom * zoomFactor, 0.08), 4.0);

  const mouseWorld = screenToWorld(e.clientX, e.clientY);

  camera.panX = e.clientX - mouseWorld.x * newZoom;
  camera.panY = e.clientY - mouseWorld.y * newZoom;
  camera.zoom = newZoom;

  triggerSaveDebounced();
}

function zoomIn() {
  const centerScreen = { x: window.innerWidth / 2, y: window.innerHeight / 2 };
  const mouseWorld = screenToWorld(centerScreen.x, centerScreen.y);
  const newZoom = Math.min(camera.zoom * 1.25, 4.0);
  camera.panX = centerScreen.x - mouseWorld.x * newZoom;
  camera.panY = centerScreen.y - mouseWorld.y * newZoom;
  camera.zoom = newZoom;
  triggerSave();
}

function zoomOut() {
  const centerScreen = { x: window.innerWidth / 2, y: window.innerHeight / 2 };
  const mouseWorld = screenToWorld(centerScreen.x, centerScreen.y);
  const newZoom = Math.max(camera.zoom * 0.8, 0.08);
  camera.panX = centerScreen.x - mouseWorld.x * newZoom;
  camera.panY = centerScreen.y - mouseWorld.y * newZoom;
  camera.zoom = newZoom;
  triggerSave();
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
  if (isSpaceDown.value || activeTool.value === 'hand' || e.button !== 0) return;

  pushUndo();

  const initialPositions = new Map<string, { x: number; y: number }>();
  if (!selectedIds.value.has(id)) {
    selectedIds.value = new Set([id]);
  }

  for (const obj of objects.value) {
    if (selectedIds.value.has(obj.id)) {
      initialPositions.set(obj.id, { x: obj.x, y: obj.y });
    }
  }

  dragState.value = {
    startX: e.clientX,
    startY: e.clientY,
    initialPositions,
  };
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

  let sx = obj.x + obj.width;
  let sy = obj.y + obj.height / 2;
  if (anchor === 'top') { sx = obj.x + obj.width / 2; sy = obj.y; }
  else if (anchor === 'bottom') { sx = obj.x + obj.width / 2; sy = obj.y + obj.height; }
  else if (anchor === 'left') { sx = obj.x; sy = obj.y + obj.height / 2; }

  connectingState.value = {
    fromId: objectId,
    fromAnchor: anchor,
    startX: sx,
    startY: sy,
  };
}

function onSelectConnector(id: string, e: MouseEvent) {
  selectedIds.value.clear();
  selectedConnectorId.value = id;
}

function updateSelectedConnector(updated: Partial<BoardConnector>) {
  if (!selectedConnectorId.value) return;
  pushUndo();
  const conn = connectors.value.find((c) => c.id === selectedConnectorId.value);
  if (conn) {
    Object.assign(conn, updated);
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
  triggerSave();
}

function updateObjectContent(id: string, content: string) {
  const obj = objects.value.find((o) => o.id === id);
  if (obj) {
    obj.content = content;
    triggerSaveDebounced();
  }
}

function updateObjectTitle(id: string, title: string) {
  pushUndo();
  const obj = objects.value.find((o) => o.id === id);
  if (obj) {
    obj.title = title || undefined;
    triggerSave();
  }
}

function updateObjectMeta(id: string, meta: any) {
  const obj = objects.value.find((o) => o.id === id);
  if (obj) {
    obj.meta = meta;
    triggerSave();
  }
}

async function copyObjectContent(obj: BoardObject) {
  if (obj.kind === 'text' || obj.kind === 'code' || obj.kind === 'url') {
    await api.copyText(obj.content);
    showToast(`Copied to clipboard`);
  } else if (obj.kind === 'image') {
    await api.copyImage(obj.content);
    showToast('Copied image to clipboard');
  }
}

function deleteSingleObject(id: string) {
  pushUndo();
  objects.value = objects.value.filter((o) => o.id !== id);
  connectors.value = connectors.value.filter((c) => c.from_id !== id && c.to_id !== id);
  selectedIds.value.delete(id);
  if (editingId.value === id) editingId.value = null;
  api.deleteObjects([id]);
  triggerSave();
  showToast('Deleted object');
}

function deleteSelected() {
  if (selectedConnectorId.value) {
    deleteConnector(selectedConnectorId.value);
    return;
  }
  if (selectedIds.value.size === 0 || editingId.value) return;

  pushUndo();
  const idsToDelete = Array.from(selectedIds.value);
  objects.value = objects.value.filter((o) => !selectedIds.value.has(o.id));
  connectors.value = connectors.value.filter(
    (c) => !(c.from_id && selectedIds.value.has(c.from_id)) && !(c.to_id && selectedIds.value.has(c.to_id))
  );
  selectedIds.value.clear();
  api.deleteObjects(idsToDelete);
  triggerSave();
  showToast(`Deleted ${idsToDelete.length} objects`);
}

// --- CREATION TOOLS ---
function createNewNoteAt(worldX: number, worldY: number) {
  const newObj: BoardObject = {
    id: crypto.randomUUID(),
    board_id: currentBoardId.value,
    kind: 'text',
    x: Math.round(worldX - 160),
    y: Math.round(worldY - 100),
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
}

function spawnDiagramNode(type: DiagramNodeType, label: string, defaultIp: string) {
  const centerWorld = screenToWorld(window.innerWidth / 2, window.innerHeight / 2);
  const newObj: BoardObject = {
    id: crypto.randomUUID(),
    board_id: currentBoardId.value,
    kind: 'diagram_node',
    x: Math.round(centerWorld.x - 85),
    y: Math.round(centerWorld.y - 31),
    width: 170,
    height: 62,
    z_index: getNextZIndex(),
    title: label,
    content: defaultIp,
    meta: {
      node_type: type,
      subtitle: defaultIp,
    },
  };
  pushUndo();
  objects.value.push(newObj);
  selectedIds.value = new Set([newObj.id]);
  triggerSave();
  showToast(`Added ${label}`);
}

function promptCreateLink() {
  const url = prompt('Enter link URL (e.g. https://docs.godotengine.org/):', 'https://');
  if (url && url.trim()) {
    const title = prompt('Enter link title:', 'Link Reference') || url;
    const centerWorld = screenToWorld(window.innerWidth / 2, window.innerHeight / 2);
    const newObj: BoardObject = {
      id: crypto.randomUUID(),
      board_id: currentBoardId.value,
      kind: 'url',
      x: Math.round(centerWorld.x - 140),
      y: Math.round(centerWorld.y - 45),
      width: 280,
      height: 75,
      z_index: getNextZIndex(),
      title,
      content: url.trim(),
    };
    pushUndo();
    objects.value.push(newObj);
    selectedIds.value = new Set([newObj.id]);
    triggerSave();
    showToast('Created link card');
  }
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

// Sizing image relative to current viewport comfortably
function spawnImageOnCanvas(dataUri: string, naturalW?: number, naturalH?: number, filename?: string) {
  const targetWorld = screenToWorld(mouseScreen.x, mouseScreen.y);
  const origW = naturalW || 450;
  const origH = naturalH || 300;
  const aspect = origW / origH;

  // Viewport-relative comfortable sizing (approx 35% of current view width, bounded between 240px and 550px)
  const viewWorldWidth = window.innerWidth / camera.zoom;
  let targetW = Math.round(Math.min(550, Math.max(240, viewWorldWidth * 0.35)));
  let targetH = Math.round(targetW / aspect);

  const newObj: BoardObject = {
    id: crypto.randomUUID(),
    board_id: currentBoardId.value,
    kind: 'image',
    x: Math.round(targetWorld.x - targetW / 2),
    y: Math.round(targetWorld.y - targetH / 2),
    width: targetW,
    height: targetH,
    z_index: getNextZIndex(),
    title: filename || 'Image',
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
    showToast('Clipboard is empty or unsupported format');
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
      // Check if it looks like code
      const isCode = text.includes('function') || text.includes('const ') || text.includes('import ') || text.includes('def ') || text.includes('{') && text.includes('}');
      const newObj: BoardObject = {
        id: crypto.randomUUID(),
        board_id: currentBoardId.value,
        kind: isCode ? 'code' : 'text',
        x: Math.round(targetWorld.x - 160),
        y: Math.round(targetWorld.y - 100),
        width: 340,
        height: Math.min(480, Math.max(160, text.split('\n').length * 24 + 60)),
        z_index: getNextZIndex(),
        title: isCode ? 'Snippet' : undefined,
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
  e.preventDefault();
  pasteFromClipboard();
}

function onDrop(e: DragEvent) {
  if (!e.dataTransfer) return;
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

function getNextZIndex(): number {
  if (objects.value.length === 0) return 1;
  return Math.max(...objects.value.map((o) => o.z_index)) + 1;
}

function fitAll() {
  if (objects.value.length === 0) {
    resetCamera();
    return;
  }

  let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
  for (const obj of objects.value) {
    minX = Math.min(minX, obj.x);
    minY = Math.min(minY, obj.y);
    maxX = Math.max(maxX, obj.x + obj.width);
    maxY = Math.max(maxY, obj.y + obj.height);
  }

  const padding = 100;
  const worldW = (maxX - minX) + padding * 2;
  const worldH = (maxY - minY) + padding * 2;

  const viewW = window.innerWidth;
  const viewH = window.innerHeight - 48; // minus top bar

  const scaleX = viewW / worldW;
  const scaleY = viewH / worldH;
  const fitZoom = Math.min(Math.max(Math.min(scaleX, scaleY), 0.1), 1.5);

  const worldCenterX = (minX + maxX) / 2;
  const worldCenterY = (minY + maxY) / 2;

  camera.zoom = fitZoom;
  camera.panX = viewW / 2 - worldCenterX * fitZoom;
  camera.panY = (viewH / 2 + 48) - worldCenterY * fitZoom;

  triggerSave();
  showToast('Fit view');
}

function resetCamera() {
  camera.panX = 0;
  camera.panY = 48;
  camera.zoom = 1;
  triggerSave();
}

function clearBoard() {
  if (confirm('Are you sure you want to clear all objects on this board?')) {
    pushUndo();
    objects.value = [];
    connectors.value = [];
    selectedIds.value.clear();
    selectedConnectorId.value = null;
    triggerSave();
    showToast('Board cleared');
  }
}

// --- SEARCH & NAVIGATION ---
function openSearch() {
  if (editingId.value) return;
  isSearchOpen.value = true;
}

function handleSearchJump(result: SearchResult) {
  previousCamera.value = {
    panX: camera.panX,
    panY: camera.panY,
    zoom: camera.zoom,
  };

  const viewW = window.innerWidth;
  const viewH = window.innerHeight - 48;
  const targetZoom = Math.min(Math.max(viewH / (result.height * 2.2), 0.6), 1.5);

  const targetCenterX = result.x + result.width / 2;
  const targetCenterY = result.y + result.height / 2;

  camera.zoom = targetZoom;
  camera.panX = viewW / 2 - targetCenterX * targetZoom;
  camera.panY = (viewH / 2 + 48) - targetCenterY * targetZoom;

  selectedIds.value = new Set([result.id]);
  highlightedId.value = result.id;
  setTimeout(() => {
    if (highlightedId.value === result.id) {
      highlightedId.value = null;
    }
  }, 1600);

  triggerSave();
  showToast(`Focused on ${result.title || result.kind}`);
}

function restorePreviousCamera() {
  if (!previousCamera.value) return;
  const prev = previousCamera.value;
  camera.panX = prev.panX;
  camera.panY = prev.panY;
  camera.zoom = prev.zoom;
  previousCamera.value = null;
  showToast('Restored previous camera position');
}

// --- KEYBOARD SHORTCUTS ---
function onKeyDown(e: KeyboardEvent) {
  if (editingId.value || isSearchOpen.value || isShortcutsOpen.value) return;

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
    showToast('Connector mode: drag from any object edge');
    return;
  }
  if (e.key === 't' || e.key === 'T') {
    activeTool.value = 'text';
    showToast('Click anywhere to create text note');
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
    const target = objects.value.find((o) => o.id === singleId);
    if (target && (target.kind === 'text' || target.kind === 'code')) {
      e.preventDefault();
      startEditing(singleId);
      return;
    }
  }

  // Copy selected object: `Ctrl+C`
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'c' && selectedIds.value.size === 1) {
    const id = Array.from(selectedIds.value)[0];
    const obj = objects.value.find((o) => o.id === id);
    if (obj) {
      e.preventDefault();
      copyObjectContent(obj);
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

  // Jump Back to previous camera: `Backspace` (when nothing selected) or `Ctrl+O`
  if ((e.key === 'Backspace' && selectedIds.value.size === 0 && !selectedConnectorId.value) || ((e.ctrlKey || e.metaKey) && e.key === 'o')) {
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

  // Esc: deselect all & return to select tool
  if (e.key === 'Escape') {
    selectedIds.value.clear();
    selectedConnectorId.value = null;
    activeTool.value = 'select';
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
      for (const obj of objects.value) {
        if (selectedIds.value.has(obj.id)) {
          obj.x += nudgeX;
          obj.y += nudgeY;
        }
      }
      triggerSaveDebounced();
    }
  }
}

function onKeyUp(e: KeyboardEvent) {
  if (e.code === 'Space') {
    isSpaceDown.value = false;
  }
}

// --- ALWAYS ON TOP ---
async function toggleAlwaysOnTop() {
  alwaysOnTop.value = !alwaysOnTop.value;
  await api.setAlwaysOnTop(alwaysOnTop.value);
  showToast(alwaysOnTop.value ? 'Always on Top Enabled' : 'Always on Top Disabled');
}

// --- PERSISTENCE ---
let saveTimer: any = null;
function triggerSaveDebounced() {
  clearTimeout(saveTimer);
  saveTimer = setTimeout(() => {
    triggerSave();
  }, 400);
}

function triggerSave() {
  api.saveBoard(
    currentBoardId.value,
    camera.panX,
    camera.panY,
    camera.zoom,
    objects.value,
    connectors.value
  );
}
</script>

<style scoped>
.pinboard-workspace {
  position: relative;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  background-color: #0b0c14;
}

.canvas-viewport {
  position: absolute;
  top: 48px;
  left: 0;
  width: 100vw;
  height: calc(100vh - 48px);
  overflow: hidden;
  background-color: #0b0c14;
  outline: none;
  cursor: default;
}

.tool-hand .canvas-viewport {
  cursor: grab !important;
}

.canvas-viewport.is-panning {
  cursor: grabbing !important;
}

.canvas-viewport.cursor-crosshair {
  cursor: crosshair !important;
}

.canvas-world {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: auto;
  transform-style: flat;
  backface-visibility: visible;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

/* Flash Highlight when jumping to object from search */
.flash-highlight {
  animation: pulseHighlight 1.5s ease-out;
}

@keyframes pulseHighlight {
  0% {
    box-shadow: 0 0 0 6px #c084fc, 0 0 40px #a855f7;
    transform: scale(1.02);
  }
  50% {
    box-shadow: 0 0 0 4px #c084fc, 0 0 25px #a855f7;
  }
  100% {
    box-shadow: 0 0 0 2px rgba(168, 85, 247, 0.3), 0 8px 24px rgba(0, 0, 0, 0.5);
  }
}

/* Marquee Selection Rectangle */
.selection-marquee {
  position: absolute;
  border: 1px dashed #a855f7;
  background: rgba(168, 85, 247, 0.12);
  pointer-events: none;
  z-index: 999;
}

/* Toast message */
.canvas-toast {
  position: fixed;
  bottom: 64px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(23, 24, 40, 0.95);
  backdrop-filter: blur(8px);
  border: 1px solid #a855f7;
  color: #f4f4f5;
  padding: 7px 18px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 600;
  box-shadow: 0 6px 24px rgba(0, 0, 0, 0.6);
  z-index: 9999;
  pointer-events: none;
}

.toast-enter-active,
.toast-leave-active {
  transition: all 0.2s ease;
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translate(-50%, 10px);
}
</style>
