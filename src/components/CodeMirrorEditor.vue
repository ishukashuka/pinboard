<template>
  <div
    class="cm-editor-container"
    :class="{ 'font-sans': fontFamily === 'sans', 'font-mono': fontFamily !== 'sans' }"
    @keydown.stop
    @auxclick="onAuxClick"
  >
    <div ref="editorHost" class="cm-host"></div>
    <div class="vim-status-bar">
      <span class="vim-mode-badge" :class="vimModeClass">{{ currentModeText }}</span>
      <span class="vim-info">Esc in Normal mode to exit | :w, :q supported</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch } from 'vue';
import { EditorState } from '@codemirror/state';
import { EditorView, keymap } from '@codemirror/view';
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
import { markdown } from '@codemirror/lang-markdown';
import { oneDark } from '@codemirror/theme-one-dark';
import { vim, getCM, Vim } from '@replit/codemirror-vim';

const props = withDefaults(
  defineProps<{
    modelValue: string;
    fontSize?: number;
    fontFamily?: 'mono' | 'sans';
    autoFocus?: boolean;
  }>(),
  {
    fontSize: 14,
    fontFamily: 'mono',
    autoFocus: true,
  }
);

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
  (e: 'exit'): void;
  (e: 'save'): void;
}>();

const editorHost = ref<HTMLDivElement | null>(null);
let view: EditorView | null = null;
const currentMode = ref<'normal' | 'insert' | 'visual' | 'replace'>('normal');

const currentModeText = computed(() => {
  switch (currentMode.value) {
    case 'insert':
      return '-- INSERT --';
    case 'visual':
      return '-- VISUAL --';
    case 'replace':
      return '-- REPLACE --';
    default:
      return '-- NORMAL --';
  }
});

const vimModeClass = computed(() => {
  return `mode-${currentMode.value}`;
});

// Configure custom Vim ex commands
Vim.defineEx('write', 'w', () => {
  emit('save');
});
Vim.defineEx('quit', 'q', () => {
  emit('exit');
});
Vim.defineEx('wq', 'wq', () => {
  emit('save');
  emit('exit');
});

import { computed } from 'vue';

onMounted(() => {
  if (!editorHost.value) return;

  const customTheme = EditorView.theme({
    '&': {
      height: '100%',
      backgroundColor: '#18181b',
      color: '#f4f4f5',
      fontSize: `${props.fontSize}px`,
    },
    '.cm-scroller': {
      overflow: 'auto',
      fontFamily: props.fontFamily === 'sans' ? 'inherit' : 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace',
      lineHeight: '1.6',
      padding: '8px',
    },
    '.cm-content': {
      caretColor: '#a855f7',
    },
    '&.cm-focused .cm-cursor': {
      borderLeftColor: '#a855f7',
    },
    '&.cm-focused .cm-fat-cursor': {
      backgroundColor: '#a855f7 !important',
      color: '#18181b !important',
    },
    '.cm-gutters': {
      backgroundColor: '#18181b',
      color: '#52525b',
      border: 'none',
    },
  });

  const updateListener = EditorView.updateListener.of((update) => {
    if (update.docChanged) {
      emit('update:modelValue', update.state.doc.toString());
    }
  });

  const state = EditorState.create({
    doc: props.modelValue,
    extensions: [
      vim(),
      history(),
      markdown(),
      oneDark,
      customTheme,
      EditorView.lineWrapping,
      updateListener,
      keymap.of([...defaultKeymap, ...historyKeymap]),
    ],
  });

  view = new EditorView({
    state,
    parent: editorHost.value,
  });

  // Track Vim mode changes
  const cm = getCM(view);
  if (cm) {
    cm.on('vim-mode-change', (e: any) => {
      if (e.mode === 'insert') {
        currentMode.value = 'insert';
      } else if (e.mode === 'visual') {
        currentMode.value = 'visual';
      } else if (e.mode === 'replace') {
        currentMode.value = 'replace';
      } else {
        currentMode.value = 'normal';
      }
    });

    // Block middle-click inside CodeMirror from triggering X11/Wayland PRIMARY paste
    const blockMiddleClick = (e: MouseEvent) => {
      if (e.button === 1) {
        e.preventDefault();
        e.stopPropagation();
        e.stopImmediatePropagation();
      }
    };
    editorHost.value.addEventListener('mousedown', blockMiddleClick, { capture: true });
    editorHost.value.addEventListener('mouseup', blockMiddleClick, { capture: true });
    editorHost.value.addEventListener('auxclick', blockMiddleClick, { capture: true });
    editorHost.value.addEventListener('pointerdown', blockMiddleClick, { capture: true });
    editorHost.value.addEventListener('pointerup', blockMiddleClick, { capture: true });

    // Custom Escape handler:
    // If user presses Escape while ALREADY in normal mode, exit editor back to canvas mode!
    editorHost.value.addEventListener('keydown', (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        const vimState = (cm as any).state?.vim;
        const isInsert = vimState?.insertMode;
        const isVisual = vimState?.visualMode;
        if (!isInsert && !isVisual) {
          e.preventDefault();
          e.stopPropagation();
          emit('exit');
        }
      }
    });
  }

  if (props.autoFocus) {
    view.focus();
  }
});

function onAuxClick(e: MouseEvent) {
  if (e.button === 1) {
    e.preventDefault();
    e.stopPropagation();
    e.stopImmediatePropagation();
  }
}

watch(
  () => props.modelValue,
  (newVal) => {
    if (!view) return;
    const currentDoc = view.state.doc.toString();
    if (newVal !== currentDoc) {
      view.dispatch({
        changes: { from: 0, to: currentDoc.length, insert: newVal },
      });
    }
  }
);

onBeforeUnmount(() => {
  if (view) {
    view.destroy();
    view = null;
  }
});
</script>

<style scoped>
.cm-editor-container {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  border-radius: 6px;
  overflow: hidden;
  background: #18181b;
  border: 1px solid #3f3f46;
}

.font-sans .cm-host {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
}

.font-mono .cm-host {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}

.cm-host {
  flex: 1;
  overflow: hidden;
  text-align: left;
}

.vim-status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 3px 8px;
  background: #09090b;
  font-size: 11px;
  font-family: ui-monospace, SFMono-Regular, monospace;
  border-top: 1px solid #27272a;
  user-select: none;
}

.vim-mode-badge {
  font-weight: 700;
  padding: 1px 6px;
  border-radius: 3px;
  letter-spacing: 0.5px;
}

.mode-normal {
  color: #38bdf8;
  background: rgba(56, 189, 248, 0.15);
}

.mode-insert {
  color: #4ade80;
  background: rgba(74, 222, 128, 0.15);
}

.mode-visual {
  color: #facc15;
  background: rgba(250, 204, 21, 0.15);
}

.mode-replace {
  color: #f87171;
  background: rgba(248, 113, 113, 0.15);
}

.vim-info {
  color: #71717a;
  font-size: 10px;
}
</style>
