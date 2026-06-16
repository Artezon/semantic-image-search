<template>
  <div class="modal-overlay" :inert="!isTop" @click.self="onOverlayClick">
    <div class="modal" :style="modalStyle">
      <h3 data-tauri-drag-region v-if="title" class="modal-title">{{ title }}</h3>
      <div class="modal-body" ref="bodyRef" :style="maskStyle" @scroll="updateScroll">
        <slot />
      </div>
      <div v-if="$slots.footer" class="modal-footer">
        <slot name="footer" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, nextTick, ref, type StyleValue } from "vue";

const props = withDefaults(
  defineProps<{
    title?: string | null;
    width?: string | null;
    dismissible?: boolean;
    isTop?: boolean;
  }>(),
  { title: null, width: null, dismissible: true, isTop: true },
);

const emit = defineEmits<{ close: [] }>();

const modalStyle = computed<StyleValue>(() => ({
  ...(props.width ? { width: props.width } : {}),
}));

const bodyRef = ref<HTMLElement>();
const canScrollUp = ref(false);
const canScrollDown = ref(false);

function onOverlayClick() {
  if (props.dismissible) emit("close");
}

function onKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape" && props.isTop && props.dismissible) emit("close");
}

function updateScroll() {
  if (!bodyRef.value) return;
  const { scrollTop, scrollHeight, clientHeight } = bodyRef.value;
  canScrollUp.value = scrollTop > 0;
  canScrollDown.value = scrollTop + clientHeight < scrollHeight - 1;
}

const maskStyle = computed<StyleValue>(() => {
  const top = canScrollUp.value ? 30 : 0;
  const bottom = canScrollDown.value ? 30 : 0;

  return {
    maskImage: `linear-gradient(
      to bottom,
      transparent,
      black ${top}px,
      black calc(100% - ${bottom}px),
      transparent
    )`,
  };
});

onMounted(() => {
  document.querySelector<HTMLElement>("[autofocus]")?.focus();
  window.addEventListener("keydown", onKeyDown);
  nextTick(updateScroll);
});
onUnmounted(() => {
  window.removeEventListener("keydown", onKeyDown);
});
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  background: rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(10px);
}

.modal {
  background: var(--surface-container);
  border: 1px solid var(--outline);
  border-radius: 12px;
  padding: 24px;
  margin: 20px;
  min-width: 300px;
  max-width: calc(100vw - 100px);
  max-height: calc(100vh - 100px);
  display: flex;
  flex-direction: column;
  gap: 20px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
  overflow: hidden;
}

.modal-title {
  margin: 0;
  font-size: 16px;
  font-weight: 500;
}

.modal-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
  overflow-y: auto;
  font-size: 13px;
  line-height: 1.6;
  color: var(--text-secondary);
  padding: 2px;
  margin: -2px;
  white-space: pre-wrap;
}

.modal-footer {
  display: flex;
  gap: 10px;
  justify-content: flex-end;
}
</style>
