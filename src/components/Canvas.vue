<template>
  <div
    ref="canvasViewportRef"
    class="canvas-viewport"
    :class="{
      'is-panning': isPanning,
      'is-space-down': isSpaceDown,
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
      <!-- Board Objects -->
      <BoardObjectView
        v-for="obj in objects"
        :key="obj.id"
        :object="obj"
        :is-selected="selectedIds.has(obj.id)"
        :is-editing="editingId === obj.id"
        :zoom="camera.zoom"
        :class="{ 'flash-highlight': highlightedId === obj.id }"
        @select="onObjectSelect"
        @start-drag="onStartObjectDrag"
        @start-resize="onStartObjectResize"
        @start-editing="startEditing"
        @exit-editing="exitEditing"
        @update-content="updateObjectContent"
        @update-title="updateObjectTitle"
        @copy-content="copyObjectContent"
        @delete="deleteSingleObject"
        @save-request="triggerSave"
      />
    </div>

    <!-- Marquee Selection Rectangle (Screen Space) -->
    <div v-if="selectionBox" class="selection-marquee" :style="selectionBoxStyle"></div>

    <!-- Minimal Floating HUD / Status Bar -->
    <div class="hud-bar">
      <div class="hud-left">
        <span class="hud-logo">PINBOARD</span>
        <span class="hud-badge" :class="editingId ? 'badge-edit' : 'badge-canvas'">
          {{ editingId ? 'EDIT MODE' : 'CANVAS MODE' }}
        </span>
        <span class="hud-stat">{{ objects.length }} objects</span>
        <span v-if="selectedIds.size > 0" class="hud-stat hud-selected">
          {{ selectedIds.size }} selected
        </span>
      </div>

      <div class="hud-center">
        <button class="hud-btn" title="Search board (/)" @click="openSearch">
          <span class="hud-kbd">/</span> Search
        </button>
        <button class="hud-btn" title="New text note (n)" @click="createNewNoteAtCenter">
          <span class="hud-kbd">n</span> New Note
        </button>
        <button class="hud-btn" title="Paste clipboard (Ctrl+V)" @click="pasteFromClipboard">
          <span class="hud-kbd">Ctrl+V</span> Paste
        </button>
        <button class="hud-btn" title="Fit all into view (Ctrl+0)" @click="fitAll">
          <span class="hud-kbd">Ctrl+0</span> Fit All
        </button>
      </div>

      <div class="hud-right">
        <span class="hud-zoom">{{ Math.round(camera.zoom * 100) }}%</span>
        <button
          class="hud-btn-icon"
          :class="{ 'is-active': alwaysOnTop }"
          title="Toggle Always-on-Top"
          @click="toggleAlwaysOnTop"
        >
          📌
        </button>
      </div>
    </div>

    <!-- Brief Toast Message -->
    <transition name="toast">
      <div v-if="toastMessage" class="canvas-toast">
        {{ toastMessage }}
      </div>
    </transition>

    <!-- Search to Focus Modal -->
    <SearchModal
      v-if="isSearchOpen"
      :objects="objects"
      @close="isSearchOpen = false"
      @jump-to="handleSearchJump"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onBeforeUnmount, nextTick } from 'vue';
import type { BoardObject, Camera, SearchResult } from '../types';
import BoardObjectView from './BoardObjectView.vue';
import SearchModal from './SearchModal.vue';
import { api } from '../utils/api';

const props = defineProps<{
  initialObjects?: BoardObject[];
  initialCamera?: Camera;
  boardId?: string;
}>();

const emit = defineEmits<{
  (e: 'save-board', camera: Camera, objects: BoardObject[]): void;
}>();

// --- STATE ---
const canvasViewportRef = ref<HTMLDivElement | null>(null);
const objects = ref<BoardObject[]>([]);
const camera = reactive<Camera>({
  panX: 0,
  panY: 0,
  zoom: 1,
});

// Camera history for Back navigation (e.g. after search jump)
const previousCamera = ref<Camera | null>(null);

const selectedIds = ref<Set<string>>(new Set());
const editingId = ref<string | null>(null);
const highlightedId = ref<string | null>(null);
const isSearchOpen = ref(false);
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

// Mouse pointer coordinates in screen space
const mouseScreen = { x: window.innerWidth / 2, y: window.innerHeight / 2 };

// Undo/Redo Stacks
interface HistorySnapshot {
  objects: BoardObject[];
}
const undoStack = ref<HistorySnapshot[]>([]);
const redoStack = ref<HistorySnapshot[]>([]);

function pushUndo() {
  // Deep clone current objects
  const snapshot: HistorySnapshot = {
    objects: JSON.parse(JSON.stringify(objects.value)),
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
  };
  redoStack.value.push(currentSnapshot);

  const prev = undoStack.value.pop()!;
  objects.value = prev.objects;
  selectedIds.value.clear();
  triggerSave();
  showToast('Undo');
}

function redo() {
  if (redoStack.value.length === 0) return;
  const currentSnapshot: HistorySnapshot = {
    objects: JSON.parse(JSON.stringify(objects.value)),
  };
  undoStack.value.push(currentSnapshot);

  const next = redoStack.value.pop()!;
  objects.value = next.objects;
  selectedIds.value.clear();
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
  const dotSize = Math.max(1, 1.5 * camera.zoom);
  return {
    backgroundSize: `${gridSize}px ${gridSize}px`,
    backgroundPosition: `${camera.panX}px ${camera.panY}px`,
    backgroundImage: `radial-gradient(circle, #2d2d38 ${dotSize}px, transparent ${dotSize}px)`,
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
  }, 2000);
}

// --- LIFECYCLE ---
onMounted(async () => {
  // Load board from backend
  const data = await api.loadBoard(props.boardId || 'default');
  if (data) {
    camera.panX = data.pan_x || 0;
    camera.panY = data.pan_y || 0;
    camera.zoom = data.zoom || 1;
    objects.value = data.objects || [];
  }

  // Intercept middle-click and panning events in capture phase so middle-click
  // pans everywhere (even over notes, images, or editors) without triggering PRIMARY paste
  window.addEventListener('mousedown', onGlobalMouseDownCapture, { capture: true });
  window.addEventListener('auxclick', onGlobalAuxClickCapture, { capture: true });
  window.addEventListener('mousemove', onGlobalMouseMoveCapture, { capture: true });
  window.addEventListener('mouseup', onGlobalMouseUpCapture, { capture: true });
  window.addEventListener('blur', onWindowBlur);

  window.addEventListener('keydown', onKeyDown);
  window.addEventListener('keyup', onKeyUp);
  window.addEventListener('paste', onGlobalPaste);

  // Focus viewport so keyboard events are captured
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
  if (e.button === 0 && isSpaceDown.value) {
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
  // Prevent default middle-click actions (such as PRIMARY paste or autoscroll)
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
}

function onGlobalMouseUpCapture(e: MouseEvent) {
  if (isPanning.value && (e.button === 1 || e.button === 0)) {
    isPanning.value = false;
    e.preventDefault();
    e.stopPropagation();
    e.stopImmediatePropagation();
    triggerSave();
  }
}

function onWindowBlur() {
  isSpaceDown.value = false;
  if (isPanning.value) {
    isPanning.value = false;
    triggerSave();
  }
}

// --- VIEWPORT MOUSE HANDLERS (Left click only) ---
function onViewportMouseDown(e: MouseEvent) {
  // Only handle left-click on empty canvas
  if (e.button !== 0 || isSpaceDown.value) return;

  // If we were editing, exit editing
  if (editingId.value) {
    exitEditing();
  }

  // Deselect unless Shift or Ctrl is pressed
  if (!e.shiftKey && !e.ctrlKey) {
    selectedIds.value.clear();
  }

  // Start marquee selection box
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

  // If panning via global capture, skip normal object drag/marquee
  if (isPanning.value) return;

  // 1. Marquee Selection
  if (selectionBox.value) {
    selectionBox.value.currentX = e.clientX;
    selectionBox.value.currentY = e.clientY;

    // Check intersection with all objects
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

  // 3. Dragging Objects
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

  // 4. Resizing Object
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
    if (handle.includes('s')) newH = Math.max(60, initialH + dy);
    if (handle.includes('w')) {
      const maxW = initialW - dx;
      if (maxW >= 80) {
        newW = maxW;
        newX = initialX + dx;
      }
    }
    if (handle.includes('n')) {
      const maxH = initialH - dy;
      if (maxH >= 60) {
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

// --- ZOOM (Centered around mouse) ---
function onWheel(e: WheelEvent) {
  // If editing an object and hovering over it, let editor scroll
  if (editingId.value) return;

  const zoomFactor = Math.exp(-e.deltaY * 0.0015);
  const newZoom = Math.min(Math.max(camera.zoom * zoomFactor, 0.08), 5.0);

  const mouseWorld = screenToWorld(e.clientX, e.clientY);

  camera.panX = e.clientX - mouseWorld.x * newZoom;
  camera.panY = e.clientY - mouseWorld.y * newZoom;
  camera.zoom = newZoom;

  triggerSaveDebounced();
}

// --- OBJECT INTERACTIONS ---
function onObjectSelect(id: string, multi: boolean) {
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
  if (isSpaceDown.value || e.button !== 0) return;

  pushUndo();

  const initialPositions = new Map<string, { x: number; y: number }>();
  // If dragged object is not in selection, select only it
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

function startEditing(id: string) {
  const obj = objects.value.find((o) => o.id === id);
  if (obj && obj.kind === 'text') {
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

async function copyObjectContent(obj: BoardObject) {
  if (obj.kind === 'text' || obj.kind === 'url') {
    await api.copyText(obj.content);
    showToast(`Copied ${obj.kind} to clipboard`);
  } else if (obj.kind === 'image') {
    await api.copyImage(obj.content);
    showToast('Copied image to clipboard');
  }
}

function deleteSingleObject(id: string) {
  pushUndo();
  objects.value = objects.value.filter((o) => o.id !== id);
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
  selectedIds.value.clear();
  api.deleteObjects(idsToDelete);
  triggerSave();
  showToast(`Deleted ${idsToDelete.length} objects`);
}

// --- CREATION & CLIPBOARD ---
function createNewNoteAtCenter() {
  const centerWorld = screenToWorld(window.innerWidth / 2, window.innerHeight / 2);
  const newObj: BoardObject = {
    id: crypto.randomUUID(),
    kind: 'text',
    x: Math.round(centerWorld.x - 150),
    y: Math.round(centerWorld.y - 100),
    width: 320,
    height: 220,
    z_index: getNextZIndex(),
    content: '',
  };
  pushUndo();
  objects.value.push(newObj);
  selectedIds.value = new Set([newObj.id]);
  startEditing(newObj.id);
  triggerSave();
}

async function pasteFromClipboard() {
  const clip = await api.getClipboard();
  if (clip.kind === 'empty') {
    showToast('Clipboard is empty or unsupported format');
    return;
  }

  // Target coordinates: pointer if within window, else center
  const targetWorld = screenToWorld(mouseScreen.x, mouseScreen.y);

  if (clip.kind === 'image' && clip.image_data) {
    const naturalW = clip.width || 400;
    const naturalH = clip.height || 300;

    // Scale down if extremely large
    let w = naturalW;
    let h = naturalH;
    const maxDimension = 600;
    if (w > maxDimension || h > maxDimension) {
      const scale = maxDimension / Math.max(w, h);
      w = Math.round(w * scale);
      h = Math.round(h * scale);
    }

    const newObj: BoardObject = {
      id: crypto.randomUUID(),
      kind: 'image',
      x: Math.round(targetWorld.x - w / 2),
      y: Math.round(targetWorld.y - h / 2),
      width: w,
      height: h,
      z_index: getNextZIndex(),
      title: clip.filename || 'Pasted Image',
      content: clip.image_data,
      meta: {
        original_filename: clip.filename,
        natural_width: naturalW,
        natural_height: naturalH,
      },
    };

    pushUndo();
    objects.value.push(newObj);
    selectedIds.value = new Set([newObj.id]);
    triggerSave();
    showToast('Pasted image');
  } else if (clip.kind === 'text' && clip.text) {
    const text = clip.text;
    const isUrl = /^https?:\/\/[^\s]+$/i.test(text.trim());

    const newObj: BoardObject = {
      id: crypto.randomUUID(),
      kind: isUrl ? 'url' : 'text',
      x: Math.round(targetWorld.x - 160),
      y: Math.round(targetWorld.y - 100),
      width: isUrl ? 320 : 360,
      height: isUrl ? 90 : Math.min(500, Math.max(160, text.split('\n').length * 24 + 60)),
      z_index: getNextZIndex(),
      title: isUrl ? text.replace(/^https?:\/\/(www\.)?/, '').split('/')[0] : undefined,
      content: text,
    };

    pushUndo();
    objects.value.push(newObj);
    selectedIds.value = new Set([newObj.id]);
    triggerSave();
    showToast(`Pasted ${isUrl ? 'URL' : 'text'}`);
  }
}

function onGlobalPaste(e: ClipboardEvent) {
  if (editingId.value) return; // Allow normal CodeMirror paste
  e.preventDefault();
  pasteFromClipboard();
}

function onDrop(e: DragEvent) {
  if (!e.dataTransfer) return;
  const files = Array.from(e.dataTransfer.files);
  const targetWorld = screenToWorld(e.clientX, e.clientY);

  for (const file of files) {
    if (file.type.startsWith('image/')) {
      const reader = new FileReader();
      reader.onload = async () => {
        const base64Data = (reader.result as string).split(',')[1];
        const res = await api.saveImageFromBytes(base64Data, file.name);
        if (res.image_data) {
          const w = Math.min(600, res.width || 400);
          const h = res.height && res.width ? Math.round((w / res.width) * res.height) : 300;
          const newObj: BoardObject = {
            id: crypto.randomUUID(),
            kind: 'image',
            x: Math.round(targetWorld.x - w / 2),
            y: Math.round(targetWorld.y - h / 2),
            width: w,
            height: h,
            z_index: getNextZIndex(),
            title: file.name,
            content: res.image_data,
            meta: {
              original_filename: file.name,
              natural_width: res.width,
              natural_height: res.height,
            },
          };
          pushUndo();
          objects.value.push(newObj);
          selectedIds.value = new Set([newObj.id]);
          triggerSave();
          showToast('Imported image file');
        }
      };
      reader.readAsDataURL(file);
    }
  }
}

// --- Z-ORDER & UTILS ---
function getNextZIndex(): number {
  if (objects.value.length === 0) return 1;
  return Math.max(...objects.value.map((o) => o.z_index)) + 1;
}

function bringForward() {
  if (selectedIds.value.size === 0) return;
  pushUndo();
  for (const obj of objects.value) {
    if (selectedIds.value.has(obj.id)) {
      obj.z_index += 1;
    }
  }
  triggerSave();
}

function sendBackward() {
  if (selectedIds.value.size === 0) return;
  pushUndo();
  for (const obj of objects.value) {
    if (selectedIds.value.has(obj.id)) {
      obj.z_index = Math.max(0, obj.z_index - 1);
    }
  }
  triggerSave();
}

// --- CAMERA CONTROLS ---
function fitAll() {
  if (objects.value.length === 0) {
    camera.panX = window.innerWidth / 2;
    camera.panY = window.innerHeight / 2;
    camera.zoom = 1;
    return;
  }

  let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
  for (const obj of objects.value) {
    minX = Math.min(minX, obj.x);
    minY = Math.min(minY, obj.y);
    maxX = Math.max(maxX, obj.x + obj.width);
    maxY = Math.max(maxY, obj.y + obj.height);
  }

  const padding = 80;
  const worldW = (maxX - minX) + padding * 2;
  const worldH = (maxY - minY) + padding * 2;

  const viewW = window.innerWidth;
  const viewH = window.innerHeight;

  const scaleX = viewW / worldW;
  const scaleY = viewH / worldH;
  const fitZoom = Math.min(Math.max(Math.min(scaleX, scaleY), 0.1), 1.5);

  const worldCenterX = (minX + maxX) / 2;
  const worldCenterY = (minY + maxY) / 2;

  camera.zoom = fitZoom;
  camera.panX = viewW / 2 - worldCenterX * fitZoom;
  camera.panY = viewH / 2 - worldCenterY * fitZoom;

  triggerSave();
  showToast('Fit all objects into view');
}

function resetCamera() {
  camera.panX = 0;
  camera.panY = 0;
  camera.zoom = 1;
  triggerSave();
}

// --- SEARCH & NAVIGATION ---
function openSearch() {
  if (editingId.value) return;
  isSearchOpen.value = true;
}

function handleSearchJump(result: SearchResult) {
  // Store previous camera state for Back navigation
  previousCamera.value = {
    panX: camera.panX,
    panY: camera.panY,
    zoom: camera.zoom,
  };

  // Center on object
  const viewW = window.innerWidth;
  const viewH = window.innerHeight;
  const targetZoom = Math.min(Math.max(viewH / (result.height * 2.2), 0.6), 1.5);

  const targetCenterX = result.x + result.width / 2;
  const targetCenterY = result.y + result.height / 2;

  camera.zoom = targetZoom;
  camera.panX = viewW / 2 - targetCenterX * targetZoom;
  camera.panY = viewH / 2 - targetCenterY * targetZoom;

  selectedIds.value = new Set([result.id]);

  // Flash highlight ring
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
  // Ignore keyboard shortcuts if CodeMirror or SearchModal input is active
  if (editingId.value || isSearchOpen.value) return;

  // Space for pan mode
  if (e.code === 'Space' && !isSpaceDown.value) {
    isSpaceDown.value = true;
  }

  // Search shortcut `/`
  if (e.key === '/') {
    e.preventDefault();
    openSearch();
    return;
  }

  // Edit selected text object: `i` or `Enter`
  if ((e.key === 'i' || e.key === 'Enter') && selectedIds.value.size === 1) {
    const singleId = Array.from(selectedIds.value)[0];
    const target = objects.value.find((o) => o.id === singleId);
    if (target && target.kind === 'text') {
      e.preventDefault();
      startEditing(singleId);
      return;
    }
  }

  // Create new note: `n`
  if (e.key === 'n' && !e.ctrlKey && !e.metaKey) {
    e.preventDefault();
    createNewNoteAtCenter();
    return;
  }

  // Copy selected object: `Ctrl+C` or `c`
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'c' && selectedIds.value.size === 1) {
    const id = Array.from(selectedIds.value)[0];
    const obj = objects.value.find((o) => o.id === id);
    if (obj) {
      e.preventDefault();
      copyObjectContent(obj);
      return;
    }
  }

  // Delete selected objects: `Delete` or `Backspace`
  if (e.key === 'Delete' || (e.key === 'Backspace' && selectedIds.value.size > 0)) {
    e.preventDefault();
    deleteSelected();
    return;
  }

  // Jump Back to previous camera: `Backspace` (when nothing selected) or `Ctrl+O`
  if ((e.key === 'Backspace' && selectedIds.value.size === 0) || ((e.ctrlKey || e.metaKey) && e.key === 'o')) {
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

  // Reset camera: `Ctrl+1`
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

  // Z-Order: `]` (bring forward), `[` (send backward)
  if (e.key === ']') {
    bringForward();
    return;
  }
  if (e.key === '[') {
    sendBackward();
    return;
  }

  // Esc: deselect all
  if (e.key === 'Escape') {
    selectedIds.value.clear();
    return;
  }

  // Nudge selection with Arrow keys or h/j/k/l
  const step = e.shiftKey ? 50 : 10;
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
  emit('save-board', camera, objects.value);
  api.saveBoard(
    props.boardId || 'default',
    camera.panX,
    camera.panY,
    camera.zoom,
    objects.value
  );
}
</script>

<style scoped>
.canvas-viewport {
  position: relative;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  background-color: #121214;
  outline: none;
  cursor: default;
}

.canvas-viewport.is-space-down {
  cursor: grab;
}

.canvas-viewport.is-panning {
  cursor: grabbing !important;
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

/* Minimal Floating HUD */
.hud-bar {
  position: fixed;
  bottom: 16px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 16px;
  background: rgba(24, 24, 27, 0.85);
  backdrop-filter: blur(8px);
  border: 1px solid #3f3f46;
  border-radius: 24px;
  padding: 6px 16px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  z-index: 1000;
  font-size: 12px;
  user-select: none;
}

.hud-left,
.hud-center,
.hud-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.hud-logo {
  font-weight: 800;
  font-size: 11px;
  letter-spacing: 1px;
  color: #a855f7;
}

.hud-badge {
  font-size: 9px;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 10px;
  letter-spacing: 0.5px;
}

.badge-canvas {
  background: rgba(56, 189, 248, 0.15);
  color: #38bdf8;
}

.badge-edit {
  background: rgba(74, 222, 128, 0.15);
  color: #4ade80;
}

.hud-stat {
  color: #71717a;
  font-size: 11px;
}

.hud-selected {
  color: #c084fc;
}

.hud-btn {
  display: flex;
  align-items: center;
  gap: 5px;
  background: #27272a;
  border: 1px solid #3f3f46;
  color: #e4e4e7;
  padding: 3px 8px;
  border-radius: 14px;
  font-size: 11px;
  cursor: pointer;
  transition: all 0.1s ease;
}

.hud-btn:hover {
  background: #3f3f46;
  color: #fff;
}

.hud-kbd {
  background: #18181b;
  color: #a1a1aa;
  padding: 1px 4px;
  border-radius: 4px;
  font-size: 9px;
  font-family: ui-monospace, SFMono-Regular, monospace;
}

.hud-zoom {
  font-family: ui-monospace, SFMono-Regular, monospace;
  font-size: 11px;
  color: #a1a1aa;
  min-width: 38px;
  text-align: right;
}

.hud-btn-icon {
  background: transparent;
  border: none;
  cursor: pointer;
  font-size: 13px;
  opacity: 0.6;
  transition: opacity 0.1s ease;
}

.hud-btn-icon:hover,
.hud-btn-icon.is-active {
  opacity: 1;
}

/* Toast message */
.canvas-toast {
  position: fixed;
  top: 24px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(24, 24, 27, 0.9);
  backdrop-filter: blur(6px);
  border: 1px solid #a855f7;
  color: #f4f4f5;
  padding: 8px 18px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 500;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
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
  transform: translate(-50%, -10px);
}
</style>
