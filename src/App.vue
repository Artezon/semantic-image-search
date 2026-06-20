<template>
  <TitleBar />
  <main class="app-container" :inert="modalStack.length > 0">
    <AppSidebar />
    <div class="vertical-divider" />
    <ResultsPanel />
  </main>
  <Teleport to="body">
    <TransitionGroup name="modal">
      <component
        v-for="(modal, i) in modalStack"
        :key="i"
        :is="modal.component"
        v-bind="modal.props"
        :is-top="i === modalStack.length - 1"
        @close="closeModal()"
      />
    </TransitionGroup>
    <Toaster position="bottom-right" :expand="true" :visibleToasts="5" />
  </Teleport>
</template>

<script setup lang="ts">
import { onMounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import TitleBar from "./components/TitleBar.vue";
import AppSidebar from "./components/AppSidebar.vue";
import ResultsPanel from "./components/ResultsPanel.vue";
import {
  modelStatusKey,
  modelStatusColor,
  deviceText,
  modelStatusParams,
  indexingState,
  indexProgress,
  indexProcessed,
  indexTotal,
  indexedFilesCount,
  searchResults,
} from "./store";
import type { IndexStatus, ModelStatus } from "./types";
import { useModalStack } from "./composables/useModal";
import { Toaster } from "vue-sonner";
import { showErrorToast } from "./toast";

const { t, locale, availableLocales } = useI18n({ useScope: "global" });
const appWindow = getCurrentWindow();
const { modalStack, closeModal } = useModalStack();

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
    id: string;
    params?: Record<string, unknown>;
  }>("message", (event) => {
    const { id, params } = event.payload;
    const title = t(`message.${id}.title`, params || {});
    const p = params || {};
    const errorMsgStr = typeof p.msg === "string" ? p.msg : "";
    const filePathStr = typeof p.path === "string" ? p.path : "";
    const reason = errorMsgStr ? t(errorMsgStr) : "unknown";
    const displayMsg = filePathStr
      ? `<b>${t("message.index_result.errors.skipped")}:</b> ${filePathStr}\n<b>${t("message.index_result.errors.reason")}:</b> ${reason}`
      : reason;
    showErrorToast(displayMsg, title);
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
    const { state, processed, total } = event.payload;
    indexingState.value = state;
    if (state === "idle") indexProgress.value = 0;
    indexProcessed.value = processed;
    indexTotal.value = total;
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
