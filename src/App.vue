<template>
  <div>
    <TitleBar />
    <main class="app-container">
      <AppSidebar />
      <div class="vertical-divider" />
      <ResultsPanel />
    </main>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { message } from "@tauri-apps/plugin-dialog";
import TitleBar from "./components/TitleBar.vue";
import AppSidebar from "./components/AppSidebar.vue";
import ResultsPanel from "./components/ResultsPanel.vue";
import {
  modelStatusText,
  modelStatusColor,
  deviceText,
  indexStatusText,
  indexProgress,
  isIndexing,
  searchResults,
} from "./store";
import type { IndexStatus, ModelStatus } from "./types";

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

  // Show window after a short delay to prevent flash
  await new Promise(requestAnimationFrame);
  await new Promise((r) => setTimeout(r, 100));
  await appWindow.show();

  await setupListeners();
  const count = await invoke("get_indexed_count");
  indexStatusText.value = `${count} indexed files`;
  await invoke("get_model_status");
});

// Backend listeners
async function setupListeners() {
  await listen<{
    title: string;
    msg: string;
    kind: "info" | "error" | "warning";
  }>("message", (event) => {
    const { title, msg, kind } = event.payload;
    message(msg, { title, kind });
  });

  await listen<ModelStatus>("model-status", (event) => {
    const { status, status_text, device_text } = event.payload;
    const colors = {
      neutral: "var(--text-secondary)",
      success: "var(--text-success)",
      error: "var(--text-failure)",
    };
    modelStatusColor.value = colors[status];
    modelStatusText.value = status_text;
    deviceText.value = device_text || "unknown";
  });

  await listen<IndexStatus>("index-status", (event) => {
    const { progress, text } = event.payload;
    indexProgress.value = progress * 100;
    indexStatusText.value = text;
  });

  await listen<boolean>("is-indexing", (event) => {
    isIndexing.value = event.payload;
  });

  await listen("clear-results", () => {
    searchResults.value = null;
  });
}
</script>

<style scoped>
.app-container {
  display: flex;
  height: 100vh;
}
</style>
