<template>
  <div
    class="cm-editor-container"
    :class="{ 'font-sans': fontFamily === 'sans', 'font-mono': fontFamily !== 'sans' }"
    @keydown.stop
  >
    <div ref="editorHost" class="cm-host"></div>
    <div class="vim-status-bar">
      <span class="vim-mode-badge" :class="vimModeClass">{{ currentModeText }}</span>
      <span class="vim-hints">
        <span class="cmd">:w</span> save · <span class="cmd">:q</span> exit · <span class="cmd">Esc</span> canvas
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue';
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
    fontSize: 13,
    fontFamily: 'sans',
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
      return 'INSERT';
    case 'visual':
      return 'VISUAL';
    case 'replace':
      return 'REPLACE';
    default:
      return 'NORMAL';
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

onMounted(() => {
  if (!editorHost.value) return;

  const customTheme = EditorView.theme({
    '&': {
      height: '100%',
      backgroundColor: 'transparent',
      color: '#f1f5f9',
      fontSize: `${props.fontSize}px`,
    },
    '.cm-scroller': {
      overflow: 'auto',
      fontFamily: props.fontFamily === 'sans' ? 'inherit' : 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace',
      lineHeight: '1.6',
      padding: '10px 14px',
    },
    '.cm-content': {
      caretColor: '#c084fc',
    },
    '&.cm-focused .cm-cursor': {
      borderLeftColor: '#c084fc',
    },
    '&.cm-focused .cm-fat-cursor': {
      backgroundColor: '#c084fc !important',
      color: '#0f101d !important',
    },
    '.cm-gutters': {
      backgroundColor: 'transparent',
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
  border-radius: 0;
  overflow: hidden;
  background: transparent;
  border: none;
}

.font-sans .cm-host {
  font-family: inherit;
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
  height: 24px;
  min-height: 24px;
  padding: 0 10px;
  background: #11121e;
  font-size: 11px;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  border-top: 1px solid #23253b;
  user-select: none;
  gap: 8px;
  overflow: hidden;
}

.vim-mode-badge {
  font-weight: 700;
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 3px;
  letter-spacing: 0.6px;
  flex-shrink: 0;
}

.mode-normal {
  color: #38bdf8;
  background: rgba(56, 189, 248, 0.18);
}

.mode-insert {
  color: #4ade80;
  background: rgba(74, 222, 128, 0.18);
}

.mode-visual {
  color: #facc15;
  background: rgba(250, 204, 21, 0.18);
}

.mode-replace {
  color: #f87171;
  background: rgba(248, 113, 113, 0.18);
}

.vim-hints {
  color: #71717a;
  font-size: 10px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  text-align: right;
}

.vim-hints .cmd {
  color: #a1a1aa;
  font-weight: 600;
}
</style>
