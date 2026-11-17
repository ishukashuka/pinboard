<template>
  <div class="modal-backdrop" @click.self="$emit('close')">
    <div class="shortcuts-card" @keydown.esc.stop="$emit('close')">
      <div class="modal-header">
        <span class="modal-title">Keyboard & Mouse Shortcuts</span>
        <button class="modal-close-btn" @click="$emit('close')" title="Close (Esc)">✕</button>
      </div>

      <div class="shortcuts-grid">
        <div class="shortcut-section">
          <div class="section-title">Tools</div>
          <div class="shortcut-row"><span class="sc-key">V</span><span class="sc-desc">Select Tool</span></div>
          <div class="shortcut-row"><span class="sc-key">H</span><span class="sc-desc">Pan / Hand Tool</span></div>
          <div class="shortcut-row"><span class="sc-key">N / T</span><span class="sc-desc">New Text Note</span></div>
          <div class="shortcut-row"><span class="sc-key">C</span><span class="sc-desc">Connector Tool</span></div>
          <div class="shortcut-row"><span class="sc-key">/</span><span class="sc-desc">Search to Focus</span></div>
          <div class="shortcut-row"><span class="sc-key">Esc</span><span class="sc-desc">Exit / Deselect / Cancel</span></div>
        </div>

        <div class="shortcut-section">
          <div class="section-title">Canvas Navigation</div>
          <div class="shortcut-row"><span class="sc-key">MMB</span><span class="sc-desc">Pan Canvas</span></div>
          <div class="shortcut-row"><span class="sc-key">Space + Drag</span><span class="sc-desc">Pan Canvas</span></div>
          <div class="shortcut-row"><span class="sc-key">Scroll</span><span class="sc-desc">Zoom Around Cursor</span></div>
          <div class="shortcut-row"><span class="sc-key">Ctrl + 0</span><span class="sc-desc">Fit Viewport</span></div>
          <div class="shortcut-row"><span class="sc-key">Ctrl + 1</span><span class="sc-desc">Reset Camera (100%)</span></div>
          <div class="shortcut-row"><span class="sc-key">Ctrl + O</span><span class="sc-desc">Restore Previous Camera</span></div>
        </div>

        <div class="shortcut-section">
          <div class="section-title">Object Actions</div>
          <div class="shortcut-row"><span class="sc-key">Ctrl + V</span><span class="sc-desc">Paste System Clipboard</span></div>
          <div class="shortcut-row"><span class="sc-key">Ctrl + C</span><span class="sc-desc">Copy Selected Item</span></div>
          <div class="shortcut-row"><span class="sc-key">Delete</span><span class="sc-desc">Delete Selection</span></div>
          <div class="shortcut-row"><span class="sc-key">Ctrl + Z</span><span class="sc-desc">Undo</span></div>
          <div class="shortcut-row"><span class="sc-key">Ctrl + Shift + Z</span><span class="sc-desc">Redo</span></div>
          <div class="shortcut-row"><span class="sc-key">] / [</span><span class="sc-desc">Bring Forward / Send Back</span></div>
          <div class="shortcut-row"><span class="sc-key">Arrows / h j k l</span><span class="sc-desc">Nudge Selection</span></div>
        </div>

        <div class="shortcut-section">
          <div class="section-title">Text & Vim Editing</div>
          <div class="shortcut-row"><span class="sc-key">i / Enter</span><span class="sc-desc">Edit Note (CodeMirror)</span></div>
          <div class="shortcut-row"><span class="sc-key">Esc</span><span class="sc-desc">Exit Vim Insert/Normal Mode</span></div>
          <div class="shortcut-row"><span class="sc-key">:w / :wq</span><span class="sc-desc">Vim Save & Exit</span></div>
          <div class="shortcut-row"><span class="sc-key">/</span><span class="sc-desc">Search in Buffer (in Vim)</span></div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount } from 'vue';

const emit = defineEmits<{
  (e: 'close'): void;
}>();

function onKeyDown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    e.preventDefault();
    e.stopPropagation();
    emit('close');
  }
}

onMounted(() => {
  window.addEventListener('keydown', onKeyDown, { capture: true });
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeyDown, { capture: true });
});
</script>

<style scoped>
.modal-backdrop {
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

.shortcuts-card {
  width: 640px;
  max-width: 90vw;
  background: #141525;
  border: 1px solid #2a2c42;
  border-radius: 14px;
  padding: 20px;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.75);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid #23253a;
}

.modal-title {
  font-weight: 700;
  font-size: 15px;
  color: #f1f5f9;
}

.modal-close-btn {
  background: transparent;
  border: none;
  color: #94a3b8;
  font-size: 15px;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
}
.modal-close-btn:hover {
  color: #f1f5f9;
  background: rgba(255, 255, 255, 0.08);
}

.shortcuts-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.shortcut-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.section-title {
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: #a855f7;
  margin-bottom: 4px;
}

.shortcut-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 13px;
  color: #cbd5e1;
}

.sc-key {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 11px;
  font-weight: 600;
  background: #0b0c14;
  border: 1px solid #282a40;
  border-radius: 5px;
  padding: 2px 7px;
  color: #f8fafc;
}

.sc-desc {
  font-size: 12px;
  color: #94a3b8;
}
</style>
