<template>
  <TitleBar />
  <main class="app-container" :inert="modalStack.length > 0">
    <AppSidebar />
    <div class="vertical-divider" />
    <ResultsPanel />
  </main>
  <ModalsContainer />
  <Toaster position="bottom-right" :expand="true" :visibleToasts="5" />
</template>

<script setup lang="ts">
import { onMounted, watch } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { setupTauriListeners } from "./events";
import TitleBar from "./components/TitleBar.vue";
import AppSidebar from "./components/AppSidebar.vue";
import ResultsPanel from "./components/ResultsPanel.vue";
import ModalsContainer from "./components/modals/ModalsContainer.vue";
import { indexProgress, indexProcessed, indexTotal, indexedFilesCount } from "./store";
import { modalStack } from "./components/modals";
import { Toaster } from "vue-sonner";
import { startOrResumeIndexing } from "./indexing";

const appWindow = getCurrentWindow();

onMounted(async () => {
  // Prevent context menu on non-editable elements
  document.oncontextmenu = (event) => {
    const target = event.target as HTMLElement;
    if (
      ((target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement) &&
        !target.disabled &&
        !target.readOnly) ||
      target.isContentEditable
    ) {
      return;
    }
    event.preventDefault();
  };

  // Prevent refreshing (e.g. F5, Ctrl+R)
  if (import.meta.env.PROD) {
    document.onkeydown = (event) => {
      if (
        event.key === "F5" ||
        ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "r")
      ) {
        event.preventDefault();
      }
    };
  }

  // Show window after a short delay to prevent flash
  await new Promise(requestAnimationFrame);
  await new Promise((r) => setTimeout(r, 100));
  await appWindow.show();

  await setupTauriListeners();

  await invoke("apply_locale");
  indexedFilesCount.value = await invoke<number>("get_indexed_count");
  await invoke("get_model_status");

  await startOrResumeIndexing();
});

watch([indexProcessed, indexTotal], ([processed, total]) => {
  indexProgress.value = total > 0 ? processed / total : 0;
});
</script>

<style scoped>
.app-container {
  display: flex;
  height: 100vh;
}
</style>
