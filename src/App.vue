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
import { onMounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { message } from "@tauri-apps/plugin-dialog";
import TitleBar from "./components/TitleBar.vue";
import AppSidebar from "./components/AppSidebar.vue";
import ResultsPanel from "./components/ResultsPanel.vue";
import {
  modelStatusKey,
  modelStatusColor,
  deviceText,
  modelStatusParams,
  indexProgress,
  indexProcessed,
  indexTotal,
  indexErrors,
  indexTextKey,
  indexedFilesCount,
  searchResults,
} from "./store";
import type { IndexStatus, ModelStatus } from "./types";

const { t, locale, availableLocales } = useI18n({ useScope: "global" });
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

  await invoke("apply_locale");
  indexedFilesCount.value = (await invoke("get_indexed_count")) as number;
  await invoke("get_model_status");
});

// Backend listeners
async function setupListeners() {
  await listen<{
    key: string;
    kind: "info" | "error" | "warning";
    params?: Record<string, unknown>;
  }>("message", (event) => {
    const { key, kind, params } = event.payload;

    const title = t(`message.${key}.title`, params || {});
    const msg = t(`message.${key}.msg`, params || {});
    message(msg, { title, kind });
  });

  await listen<ModelStatus>("model-status", (event) => {
    const { status, status_key, device_text, params } = event.payload;
    const colors = {
      neutral: "var(--text-secondary)",
      success: "var(--text-success)",
      error: "var(--text-failure)",
    };
    modelStatusColor.value = colors[status];
    modelStatusKey.value = status_key;
    deviceText.value = device_text || t("device_unknown");
    modelStatusParams.value = params || {};
  });

  await listen<IndexStatus>("index-status", (event) => {
    const { processed, total, errors, text_key } = event.payload;
    indexProcessed.value = processed;
    indexTotal.value = total;
    indexErrors.value = errors;
    indexTextKey.value = text_key;
  });

  await listen("clear-results", () => {
    searchResults.value = null;
  });

  await listen<string>("update-locale", (event) => {
    const newLocale = event.payload;
    if (newLocale !== locale.value) {
      locale.value = availableLocales.includes(newLocale) ? newLocale : "en";
    }
  });
}

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
