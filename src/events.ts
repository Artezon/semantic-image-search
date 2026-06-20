import { listen } from "@tauri-apps/api/event";
import {
  modelStatusKey,
  modelStatusColor,
  deviceText,
  modelStatusParams,
  indexingState,
  indexProgress,
  indexProcessed,
  indexTotal,
  searchResults,
} from "./store";
import type { IndexStatus, ModelStatus } from "./types";
import { showErrorToast } from "./toast";
import { i18n } from "./i18n";

export async function setupTauriListeners() {
  const { t, locale, availableLocales } = i18n.global;

  await listen<{
    id: string;
    params?: Record<string, unknown>;
  }>("message", (event) => {
    const p = event.payload.params ?? {};
    const title = t("indexing.error.toast.header", p);
    const path = String(p.path || "");
    const reason = t(String(p.detail || "error.unknown"));
    const msg = `${t("error.detail.skipped", { detail: path })}\n${t("error.detail.reason", { detail: reason })}`;
    showErrorToast(msg, title);
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
    const newLocale = event.payload as (typeof availableLocales)[number];
    if (newLocale !== locale.value) {
      locale.value = availableLocales.includes(newLocale) ? newLocale : "en";
    }
  });
}
