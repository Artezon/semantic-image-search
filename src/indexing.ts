import { invoke } from "@tauri-apps/api/core";
import {
  indexProgress,
  indexProcessed,
  indexTotal,
  indexedFilesCount,
  indexingState,
  indexedDirs,
} from "./store";
import type { AppError, IndexingResult } from "./types";
import { showInfoModal } from "./components/modals";
import { showIndexingResultToast } from "./toast";
import { i18n } from "./i18n";

const { t } = i18n.global;

export async function startOrResumeIndexing() {
  if (indexedDirs.value.length === 0) {
    indexingState.value = "idle";
    return;
  }

  const resuming = indexingState.value === "paused";
  indexingState.value = "preparing";
  if (!resuming) {
    indexProcessed.value = 0;
    indexTotal.value = 0;
    indexProgress.value = 0;
  }

  try {
    const result = await invoke<IndexingResult>("index_directories");
    indexedFilesCount.value = await invoke<number>("get_indexed_count");

    if (result.was_paused) {
      indexingState.value = "paused";
      return;
    }

    indexingState.value = "idle";
    showIndexingResultToast(result);
  } catch (e) {
    indexingState.value = "idle";
    const err = e as AppError;
    const errorMsg = t(`error.${err.code}`, { detail: err.detail });
    await showInfoModal(errorMsg, t("indexing.error.fatal.modal.header"));
  }
}

export async function pauseIndexing() {
  indexingState.value = "pausing";
  await invoke("pause_indexing");
}
