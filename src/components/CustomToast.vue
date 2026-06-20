<template>
  <div class="toast" :class="kind">
    <div class="toast-header">
      <div class="toast-icon">
        <ToastInfoIcon v-if="kind === 'info'" />
        <ToastSuccessIcon v-else-if="kind === 'success'" />
        <ToastErrorIcon v-else-if="kind === 'error'" />
      </div>
      <span v-if="title" class="toast-title">{{ title }}</span>
      <div class="toast-spacer" />
      <button class="toast-btn" @click="copyMsg" :title="$t('action.copy_to_clipboard')">
        <ToastCopyIcon />
      </button>
      <button class="toast-btn" @click="close()" :title="$t('action.close')">
        <ToastCloseIcon />
      </button>
    </div>
    <div class="toast-body">
      <div class="toast-msg" v-html="msg" />
      <button v-if="action" class="toast-action-btn" @click="handleAction">
        {{ action.label }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  ToastInfoIcon,
  ToastSuccessIcon,
  ToastErrorIcon,
  ToastCopyIcon,
  ToastCloseIcon,
} from "./icons";
import type { ToastKind, ToastAction } from "../toast";

const props = defineProps<{
  msg: string;
  title?: string;
  kind: ToastKind;
  action?: ToastAction;
  close: () => void;
}>();

async function copyMsg() {
  try {
    await navigator.clipboard.writeText(props.msg.replace(/<[^>]*>/g, ""));
  } catch {}
}

function handleAction() {
  props.action?.onClick();
  if (props.action?.closeToast) props.close();
}
</script>

<style scoped>
.toast {
  width: 400px;
  border-radius: 10px;
  overflow: hidden;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.25);
  backdrop-filter: blur(25px);
  border: 1px solid;
  color: var(--text);
  padding: 16px;
}

.toast.info {
  --accent: var(--text-secondary);
  background: var(--toast-info-bg);
  border-color: var(--accent);
}

.toast.success {
  --accent: var(--toast-success-border);
  background: var(--toast-success-bg);
  border-color: var(--accent);
}

.toast.error {
  --accent: var(--toast-error-border);
  background: var(--toast-error-bg);
  border-color: var(--accent);
}

.toast-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: -3px -3px 8px 0;
}

.toast-icon {
  flex-shrink: 0;
  width: 20px;
  height: 20px;
  color: var(--accent);
}

.toast-icon svg {
  width: 100%;
  height: 100%;
}

.toast-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--accent);
}

.toast-spacer {
  flex: 1;
}

.toast-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: inherit;
  cursor: pointer;
  opacity: 0.85;
}

.toast-btn:hover {
  opacity: 1;
  background: var(--toast-btn-hover);
}

.toast-btn:active {
  opacity: 1;
  background: var(--toast-btn-pressed);
}

.toast-btn svg {
  width: 50%;
  height: 50%;
}

.toast-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.toast-msg {
  font-size: 13px;
  line-height: 1.4;
  overflow-wrap: break-word;
  white-space: pre-wrap;
}

.toast-action-btn {
  align-self: flex-start;
  border: 1px solid var(--accent);
  border-radius: 6px;
  padding: 6px 10px;
  font-size: 12px;
  font-weight: 500;
  background: transparent;
  color: inherit;
  cursor: pointer;
  opacity: 0.85;
}

.toast-action-btn:hover {
  opacity: 1;
  background: var(--toast-btn-hover);
}

.toast-action-btn:active {
  opacity: 1;
  background: var(--toast-btn-pressed);
}
</style>
