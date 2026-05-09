<template>
  <div class="titlebar">
    <div data-tauri-drag-region></div>
    <div class="controls">
      <button @click="appWindow.minimize()">
        <div>
          <WindowMinimize />
        </div>
      </button>
      <button @click="appWindow.toggleMaximize()">
        <div>
          <WindowRestore v-if="isMaximized" />
          <WindowMaximize v-else />
        </div>
      </button>
      <button id="window-close" @click="appWindow.close()">
        <div>
          <WIndowClose />
        </div>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { UnlistenFn } from "@tauri-apps/api/event";
import { WindowMinimize, WindowMaximize, WindowRestore, WIndowClose } from "./icons";

const appWindow = getCurrentWindow();
const isMaximized = ref(false);
let unlisten: UnlistenFn;

async function updateMaximized() {
  isMaximized.value = await appWindow.isMaximized();
}

onMounted(async () => {
  await updateMaximized();
  unlisten = await appWindow.onResized(updateMaximized);
});

onUnmounted(() => unlisten?.());
</script>

<style scoped>
.titlebar {
  height: 48px;
  user-select: none;
  display: grid;
  grid-template-columns: auto max-content;
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 20;
}

.titlebar > .controls {
  display: flex;
  align-items: center;
  padding: 0 10px;
}

.titlebar button {
  border: none;
  width: 40px;
  height: 100%;
  background-color: transparent;
}

.titlebar button > div {
  border-radius: 5px;
  height: 30px;
  width: 40px;
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 12px;
}

.titlebar #window-close {
  margin-right: -10px;
  padding-right: 50px;
}

.titlebar button:hover > div {
  background: var(--titlebar-btn-hover);
}

.titlebar button:active > div {
  background: var(--titlebar-btn-pressed);
}

.titlebar #window-close:hover > div {
  background: var(--titlebar-btn-close-hover);
}

.titlebar #window-close:active > div {
  background: var(--titlebar-btn-close-pressed);
}
</style>
