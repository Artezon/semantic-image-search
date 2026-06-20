import { listen } from "@tauri-apps/api/event";
import {
  modelStatusKey,
  modelStatusColor,
  deviceText,
  modelStatusErr,
  indexingState,
  indexProgress,
  indexProcessed,
  indexTotal,
  searchResults,
} from "./store";
import type { BackendMessage, IndexStatus, ModelStatus } from "./types";
import { showErrorToast } from "./toast";
import { i18n } from "./i18n";

export async function setupTauriListeners() {
  const { t, locale, availableLocales } = i18n.global;

  await listen<BackendMessage>("model-status", (event) => {
    const p = event.payload.params as ModelStatus;
    const status = p.status;
    const colors = {
      neutral: "var(--text-secondary)",
      success: "var(--text-success)",
      error: "var(--text-failure)",
    };
    modelStatusColor.value = colors[status];
    modelStatusKey.value = p.status_key;
    deviceText.value = p.device_text || t("device_unknown");
    modelStatusErr.value = p.error_details;
  });

  await listen<BackendMessage>("index-status", (event) => {
    const p = event.payload.params as IndexStatus;
    indexingState.value = p.state;
    if (indexingState.value === "idle") indexProgress.value = 0;
    indexProcessed.value = p.processed;
    indexTotal.value = p.total;
  });

  await listen<BackendMessage>("indexing-error", (event) => {
    const p = event.payload.params;
    const title = t("indexing.error.toast.header", p);
    const path = String(p.path || "");
    const reason = t(String(p.detail || "error.unknown"));
    const msg = `${t("error.detail.skipped", { detail: path })}\n${t("error.detail.reason", { detail: reason })}`;
    showErrorToast(msg, title);
  });

  await listen("clear-results", () => {
    searchResults.value = null;
  });

  await listen<string>("update-locale", (event) => {
    const newLocale = event.payload as (typeof availableLocales)[number];
    if (newLocale !== locale.value) {
      locale.value = availableLocales.includes(newLocale) ? newLocale : "en";
    }
  });
}
