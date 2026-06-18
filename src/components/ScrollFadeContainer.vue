<template>
  <div class="scroll-fade-container" ref="containerRef" :style="maskStyle" @scroll="updateScroll">
    <slot />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, type StyleValue } from "vue";

const props = withDefaults(defineProps<{ fade?: number }>(), { fade: 15 });

const containerRef = ref<HTMLElement>();
const canScrollUp = ref(false);
const canScrollDown = ref(false);

function updateScroll() {
  if (!containerRef.value) return;
  const { scrollTop, scrollHeight, clientHeight } = containerRef.value;
  canScrollUp.value = scrollTop > 0;
  canScrollDown.value = scrollTop + clientHeight < scrollHeight - 1;
}

const maskStyle = computed<StyleValue>(() => {
  const top = canScrollUp.value ? props.fade : 0;
  const bottom = canScrollDown.value ? props.fade : 0;

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

let observer: ResizeObserver;

onMounted(() => {
  updateScroll();
  observer = new ResizeObserver(updateScroll);
  observer.observe(containerRef.value!);
});

onUnmounted(() => {
  observer?.disconnect();
});
</script>

<style scoped>
.scroll-fade-container {
  overflow-y: auto;
}
</style>
