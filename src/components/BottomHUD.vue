<template>
  <div class="bottom-hud-wrapper">
    <!-- Bottom Left: Zoom & Fit Controls -->
    <div class="bottom-left-hud">
      <button class="hud-btn" @click="$emit('zoom-out')" title="Zoom Out (-)">
        <Minus :size="14" />
      </button>
      <span class="hud-zoom-text">{{ Math.round(zoom * 100) }}%</span>
      <button class="hud-btn" @click="$emit('zoom-in')" title="Zoom In (+)">
        <Plus :size="14" />
      </button>
      <div class="hud-sep"></div>
      <button class="hud-btn" @click="$emit('fit-all')" title="Fit to Viewport (Ctrl+0)">
        <Maximize2 :size="14" />
      </button>
    </div>

    <!-- Bottom Right: Keyboard & Mouse Interaction Guide -->
    <div class="bottom-right-hud">
      <div class="hint-item">
        <span class="hint-icon">🖱️</span>
        <span class="hint-text">Pan: MMB or Space+Drag</span>
      </div>
      <div class="hint-item">
        <span class="hint-icon">🔍</span>
        <span class="hint-text">Zoom: Scroll</span>
      </div>
      <div class="hint-item cursor-pointer" @click="$emit('open-search')">
        <span class="hint-kbd">/</span>
        <span class="hint-text">Search</span>
      </div>
      <div class="hint-item">
        <span class="hint-kbd">Esc</span>
        <span class="hint-text">Canvas</span>
      </div>
      <div class="hint-item cursor-pointer" @click="$emit('open-shortcuts')">
        <span class="hint-kbd">?</span>
        <span class="hint-text">Shortcuts</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Minus, Plus, Maximize2 } from 'lucide-vue-next';

defineProps<{
  zoom: number;
}>();

defineEmits<{
  (e: 'zoom-in'): void;
  (e: 'zoom-out'): void;
  (e: 'fit-all'): void;
  (e: 'open-search'): void;
  (e: 'open-shortcuts'): void;
}>();
</script>

<style scoped>
.bottom-hud-wrapper {
  pointer-events: none;
  user-select: none;
  z-index: 9980;
}

/* Bottom Left HUD */
.bottom-left-hud {
  position: fixed;
  bottom: 16px;
  left: 16px;
  display: flex;
  align-items: center;
  gap: 4px;
  background: #131422;
  border: 1px solid #23253a;
  border-radius: 12px;
  padding: 4px 6px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
  pointer-events: auto;
}

.hud-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 26px;
  background: transparent;
  border: none;
  color: #94a3b8;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.12s;
}

.hud-btn:hover {
  background: #1e2035;
  color: #f1f5f9;
}

.hud-zoom-text {
  font-family: ui-monospace, monospace;
  font-size: 12px;
  font-weight: 600;
  color: #cbd5e1;
  min-width: 44px;
  text-align: center;
}

.hud-sep {
  width: 1px;
  height: 16px;
  background: #23253a;
  margin: 0 2px;
}

/* Bottom Right HUD */
.bottom-right-hud {
  position: fixed;
  bottom: 16px;
  right: 16px;
  display: flex;
  align-items: center;
  gap: 16px;
  background: rgba(19, 20, 34, 0.75);
  backdrop-filter: blur(8px);
  border: 1px solid #23253a;
  border-radius: 12px;
  padding: 6px 14px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  pointer-events: auto;
  font-size: 11px;
  color: #94a3b8;
}

.hint-item {
  display: flex;
  align-items: center;
  gap: 5px;
}

.cursor-pointer {
  cursor: pointer;
}

.cursor-pointer:hover .hint-text {
  color: #f1f5f9;
}

.hint-kbd {
  background: #1c1e30;
  border: 1px solid #2b2e47;
  color: #cbd5e1;
  font-family: ui-monospace, monospace;
  padding: 1px 5px;
  border-radius: 4px;
  font-size: 10px;
}

.hint-icon {
  font-size: 11px;
}
</style>
