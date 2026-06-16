<template>
  <div
    class="progress-bar"
    :style="{
      '--progress': `${progress * 100}%`,
      '--progress-transition': animated ? '0.1s linear' : 'none',
    }"
  >
    <div class="content">
      <slot />
    </div>

    <div class="content content-filled">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  progress: number;
  animated?: boolean;
}>();
</script>

<style scoped>
.progress-bar {
  --progress: 0%;
  --progress-transition: none;

  position: relative;
  background: var(--secondary);
  color: var(--text);
  width: 100%;
  height: 100%;
}

.progress-bar::before {
  content: "";
  position: absolute;
  inset: 0;
  width: var(--progress);
  background: var(--primary);
  transition: width var(--progress-transition);
}

.content {
  position: relative;
  inset: 0;
  z-index: 1;
}

.content-filled {
  position: absolute;
  inset: 0;
  color: white;
  z-index: 2;
  clip-path: inset(0 calc(100% - var(--progress)) 0 0);
  transition: clip-path var(--progress-transition);
}
</style>
