<template>
  <div class="modal-backdrop" @click.self="$emit('cancel')">
    <div class="link-modal-card" @keydown.esc.stop="$emit('cancel')">
      <div class="modal-header">
        <div class="modal-title-row">
          <div class="modal-icon-badge">
            <LinkIcon :size="16" />
          </div>
          <span class="modal-title">Add Link Card</span>
        </div>
        <button class="modal-close-btn" @click="$emit('cancel')" title="Close (Esc)">✕</button>
      </div>

      <form class="modal-body" @submit.prevent="submit">
        <div class="form-group">
          <label class="form-label">URL <span class="required">*</span></label>
          <input
            ref="urlInputRef"
            v-model="url"
            type="text"
            class="form-input"
            :class="{ 'has-error': urlTouched && !isValidUrl }"
            placeholder="https://example.com"
            @blur="urlTouched = true"
            @keydown.enter.prevent="submit"
          />
          <span v-if="urlTouched && !isValidUrl" class="error-msg">Please enter a valid URL</span>
        </div>

        <div class="form-group">
          <label class="form-label">Title <span class="optional">(optional)</span></label>
          <input
            v-model="title"
            type="text"
            class="form-input"
            placeholder="e.g. Godot 4.7 Docs"
            @keydown.enter.prevent="submit"
          />
        </div>

        <div class="modal-actions">
          <button type="button" class="btn btn-secondary" @click="$emit('cancel')">
            Cancel
          </button>
          <button
            type="submit"
            class="btn btn-primary"
            :disabled="!isValidUrl"
          >
            Create Link
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick } from 'vue';
import { Link as LinkIcon } from 'lucide-vue-next';

const emit = defineEmits<{
  (e: 'create', payload: { url: string; title: string }): void;
  (e: 'cancel'): void;
}>();

const url = ref('https://');
const title = ref('');
const urlTouched = ref(false);
const urlInputRef = ref<HTMLInputElement | null>(null);

const isValidUrl = computed(() => {
  const val = url.value.trim();
  if (!val || val === 'https://' || val === 'http://') return false;
  try {
    const parsed = new URL(val);
    return parsed.protocol === 'http:' || parsed.protocol === 'https:';
  } catch {
    return false;
  }
});

onMounted(() => {
  nextTick(() => {
    urlInputRef.value?.focus();
    // Select all text in input so user can easily paste or replace
    urlInputRef.value?.select();
  });
});

function submit() {
  urlTouched.value = true;
  if (!isValidUrl.value) return;

  let computedTitle = title.value.trim();
  if (!computedTitle) {
    try {
      computedTitle = new URL(url.value.trim()).hostname.replace(/^www\./, '');
    } catch {
      computedTitle = 'Link';
    }
  }

  emit('create', {
    url: url.value.trim(),
    title: computedTitle,
  });
}
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

.link-modal-card {
  width: 440px;
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

.modal-title-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.modal-icon-badge {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  background: rgba(168, 85, 247, 0.15);
  color: #c084fc;
  display: flex;
  align-items: center;
  justify-content: center;
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

.modal-body {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-label {
  font-size: 12px;
  font-weight: 600;
  color: #94a3b8;
}

.required {
  color: #f43f5e;
}

.optional {
  color: #64748b;
  font-weight: 400;
}

.form-input {
  background: #0d0e17;
  border: 1px solid #27283d;
  border-radius: 8px;
  padding: 9px 12px;
  font-size: 13px;
  color: #f8fafc;
  outline: none;
  transition: border-color 0.15s ease, box-shadow 0.15s ease;
}

.form-input:focus {
  border-color: #a855f7;
  box-shadow: 0 0 0 2px rgba(168, 85, 247, 0.25);
}

.form-input.has-error {
  border-color: #f43f5e;
}

.error-msg {
  font-size: 11px;
  color: #f43f5e;
}

.modal-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 8px;
  padding-top: 12px;
  border-top: 1px solid #23253a;
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

.btn-primary {
  background: #9333ea;
  border: 1px solid #a855f7;
  color: #ffffff;
}
.btn-primary:hover:not(:disabled) {
  background: #a855f7;
}
.btn-primary:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
</style>
