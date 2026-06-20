<template>
  <div class="app-sidebar">
    <div class="column-titlebar">
      <h1 class="centered-text">{{ $t("app.title") }}</h1>
      <div class="splash">{{ $t("app.version") }}</div>
    </div>

    <div class="sidebar-section">
      <div class="status-text model-status">
        {{ modelStatusText }}
      </div>
      <div class="status-text">{{ $t("sidebar.device", { device: deviceText }) }}</div>
    </div>

    <!-- Indexing Section -->
    <div class="sidebar-section">
      <h2>{{ $t("sidebar.library") }}</h2>

      <DirectoryList />

      <button class="btn full-width-btn" @click="addDirectory">
        <AddFolderIcon />
        <span>{{ $t("sidebar.add_directory") }}</span>
      </button>

      <div class="param-row">
        <label>{{ $t("sidebar.batch_size") }}</label>
        <NumberInput v-model="batchSize" :min="1" :max="64" />
      </div>

      <button class="btn full-width-btn" @click="handleIndexingButton">
        <RichProgressBar
          :progress="indexingState !== 'idle' ? indexProgress : 1"
          :animated="indexingState !== 'idle' && indexProgress > 0"
        >
          <RefreshIcon v-if="indexingState === 'idle'" />
          <PlayIcon v-else-if="indexingState === 'paused'" />
          <PauseIcon v-else />
          <span v-if="indexingState === 'pausing'">{{ $t("sidebar.pausing") }}</span>
          <span v-else-if="indexingState === 'indexing' || indexingState === 'preparing'">{{
            $t("sidebar.pause_indexing")
          }}</span>
          <span v-else-if="indexingState === 'paused'">{{ $t("sidebar.resume_indexing") }}</span>
          <span v-else>{{ $t("sidebar.rescan_directories") }}</span>
        </RichProgressBar>
      </button>

      <div class="status-text">{{ indexStatusText }}</div>
    </div>

    <!-- Search Section -->
    <div class="sidebar-section">
      <h2>{{ $t("sidebar.search") }}</h2>
      <div class="radio-group">
        <label class="radio-label">
          <input type="radio" name="search-type" value="text" v-model="searchType" />
          {{ $t("sidebar.text_search") }}
        </label>
        <label class="radio-label">
          <input type="radio" name="search-type" value="image" v-model="searchType" />
          {{ $t("sidebar.image_search") }}
        </label>
      </div>

      <div v-if="searchType === 'text'" class="input-group">
        <input
          type="text"
          v-model="queryText"
          :placeholder="$t('sidebar.search_placeholder')"
          @keyup.enter="search"
        />
      </div>
      <div v-else class="input-group">
        <div class="input-row">
          <input
            type="text"
            v-model="queryImage"
            :placeholder="$t('sidebar.image_placeholder')"
            readonly
          />
          <button class="btn icon-btn" @click="browseImage">
            <ImageIcon />
          </button>
        </div>
      </div>

      <div class="param-row">
        <label>{{ $t("sidebar.max_results") }}</label>
        <NumberInput v-model="maxResults" :min="1" :max="4096" />
      </div>

      <div class="param-row">
        <label>{{ $t("sidebar.score_threshold") }}</label>
        <NumberInput v-model="threshold" :min="0" :max="1" :step="0.01" />
      </div>

      <button class="btn primary full-width-btn" @click="search">
        <SearchIcon />
        <span>{{ $t("sidebar.search") }}</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import {
  modelStatusKey,
  modelStatusColor,
  deviceText,
  modelStatusParams,
  indexProgress,
  indexProcessed,
  indexTotal,
  indexedFilesCount,
  indexingState,
  indexedDirs,
  searchResults,
  isSearching,
  queryText,
  queryImage,
  searchType,
} from "../store";
import type { AppError, IndexingResult, SearchResult } from "../types";
import { ImageIcon, SearchIcon, RefreshIcon, PauseIcon, PlayIcon, AddFolderIcon } from "./icons";
import { batchSize, maxResults, threshold } from "../store";
import NumberInput from "./NumberInput.vue";
import { showInfoModal } from "./modals";
import { showToast } from "../toast";
import { formatSeconds } from "../utils";
import RichProgressBar from "./RichProgressBar.vue";
import DirectoryList from "./DirectoryList.vue";

const { t } = useI18n();

const modelStatusText = computed(() => {
  const key = modelStatusKey.value as string;
  const params = modelStatusParams.value;
  if (key === "error") return t("model_status.error", { error: params.error || "" });
  return t(`model_status.${key}`, params);
});

const indexStatusText = computed(() => {
  const state = indexingState.value;
  if (state === "idle") return t("sidebar.indexed_count", { count: indexedFilesCount.value });
  if (state === "indexing") {
    const processed = indexProcessed.value;
    const total = indexTotal.value;
    return t("index_status.indexing", { processed, total });
  }
  return t(`index_status.${state}`, {
    processed: indexProcessed.value,
    total: indexTotal.value,
  });
});

async function addDirectory() {
  const path = await open({ directory: true });
  if (typeof path !== "string") return;

  if (indexedDirs.value.includes(path)) {
    await showInfoModal(
      t("message.directory_already_added.msg"),
      t("message.directory_already_added.title"),
    );
    return;
  }

  try {
    await invoke("add_directory", { path });
    indexedDirs.value = await invoke("get_dirs");
  } catch (e) {
    const err = e as AppError;
    await showInfoModal(
      t(`error.${err.code}`, { message: err.msg }),
      t("message.invalid_directory.title"),
    );
  }
}

async function browseImage() {
  const path = await open({
    multiple: false,
    filters: [
      {
        name: "Images",
        extensions: ["jpg", "jpeg", "png", "bmp", "gif", "webp", "tiff", "avif"],
      },
    ],
  });
  if (typeof path === "string") queryImage.value = path;
}

async function handleIndexingButton() {
  if (indexingState.value === "indexing" || indexingState.value === "preparing") {
    indexingState.value = "pausing";
    await invoke("pause_indexing");
  } else if (indexingState.value === "idle" || indexingState.value === "paused") {
    const resuming = indexingState.value === "paused";
    indexingState.value = "preparing";
    if (!resuming) {
      indexProcessed.value = 0;
      indexTotal.value = 0;
      indexProgress.value = 0;
    }

    try {
      const result = await invoke<IndexingResult>("index_directories");
      indexedFilesCount.value = (await invoke("get_indexed_count")) as number;

      const { processed, total, elapsed_secs, was_paused, errors } = result;

      if (was_paused) {
        indexingState.value = "paused";
        return;
      }

      indexingState.value = "idle";

      let summary = t("message.index_result.msg", {
        processed,
        total,
        elapsed: formatSeconds(elapsed_secs),
      });

      if (errors.length === 0) {
        showToast(summary, t("message.index_result.complete.title"), "info", true);
      } else {
        summary += `\n${t("message.index_result.errors.header", { count: errors.length })}`;
        showToast(summary, t("message.index_result.complete.title"), "info", true, {
          label: t("action.show_errors"),
          onClick: () => {
            const lines = errors.map(
              ([path, err]) =>
                `<b>${t("message.index_result.errors.skipped")}:</b> ${path}\n<b>${t("message.index_result.errors.reason")}:</b> ${err.msg ? t(err.msg) : err.code}`,
            );
            showInfoModal(
              lines.join("\n\n"),
              t("message.index_result.errors.title", { count: errors.length }),
            );
          },
          closeToast: true,
        });
      }
    } catch (e) {
      indexingState.value = "idle";
      const err = e as AppError;
      const errorMsg = t(`error.${err.code}`, { message: err.msg });
      await showInfoModal(errorMsg, t("message.indexing_error.title"));
    }
  }
}

async function search() {
  searchResults.value = null;

  const query = (searchType.value === "text" ? queryText.value : queryImage.value).trim();

  if (!query) {
    await showInfoModal(t("message.empty_query.msg"), t("message.empty_query.title"));
    return;
  }

  if (maxResults.value < 1 || maxResults.value > 4096) {
    await showInfoModal(t("message.invalid_threshold.msg"), t("message.invalid_threshold.title"));
    return;
  }

  if (indexedDirs.value.length == 0) {
    await showInfoModal(t("message.no_index.msg"), t("message.no_index.title"));
    return;
  }

  try {
    isSearching.value = true;
    const results = await invoke<Array<SearchResult>>("search", {
      searchType: searchType.value,
      query,
      maxResults: maxResults.value,
      threshold: threshold.value,
    });
    searchResults.value = results;
    isSearching.value = false;
  } catch (e) {
    isSearching.value = false;
    const err = e as AppError;
    const errorMsg = t(`error.${err.code}`, { message: err.msg });
    await showInfoModal(errorMsg, t("message.search_error.title"));
  }
}
</script>

<style scoped>
.app-sidebar {
  width: 320px;
  background-color: var(--surface-container);
  overflow-x: hidden;
  overflow-y: auto;
  overflow-wrap: break-word;
  display: flex;
  flex-direction: column;
}

.sidebar-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 20px;
}

.sidebar-section:not(:last-child) {
  border-bottom: 1px solid var(--outline);
}

.status-text {
  font-size: 12px;
  color: var(--text-secondary);
  text-align: center;
}

.model-status {
  color: v-bind(modelStatusColor);
}

.splash {
  position: absolute;
  top: 25px;
  right: 60px;

  color: #ffeb3b;
  font-size: 12px;
  font-weight: bold;

  transform: rotate(-15deg);
  transform-origin: center;

  text-shadow:
    -1px -1px 1px #000,
    1px -1px 1px #000,
    -1px 1px 1px #000,
    1px 1px 1px #000,
    -2px -2px 5px rgba(0, 0, 0, 0.3),
    2px -2px 5px rgba(0, 0, 0, 0.3),
    -2px 2px 5px rgba(0, 0, 0, 0.3),
    2px 2px 5px rgba(0, 0, 0, 0.3);
}
</style>
