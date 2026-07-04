<template>
  <div class="app-sidebar">
    <div class="column-titlebar">
      <h1 class="centered-text">{{ $t("app.title") }}</h1>
      <div class="splash">v{{ appVersion }}</div>
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

      <button class="btn full-width-btn" :disabled="!modelLoaded" @click="handleIndexingButton">
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

      <button class="btn primary full-width-btn" :disabled="!modelLoaded" @click="search">
        <SearchIcon />
        <span>{{ $t("sidebar.search") }}</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, watch } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import {
  modelStatusKey,
  modelStatusColor,
  deviceText,
  modelStatusErr,
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
import type { AppError, SearchResult } from "../types";
import { ImageIcon, SearchIcon, RefreshIcon, PauseIcon, PlayIcon, AddFolderIcon } from "./icons";
import { batchSize, maxResults, threshold } from "../store";
import NumberInput from "./NumberInput.vue";
import { showInfoModal } from "./modals";
import RichProgressBar from "./RichProgressBar.vue";
import DirectoryList from "./DirectoryList.vue";
import { pauseIndexing, startOrResumeIndexing } from "../indexing";

const { t } = useI18n({ useScope: "global" });

const appVersion = __APP_VERSION__.replace(/\.0(-|$)/, "$1");

const modelLoaded = computed(() => modelStatusKey.value === "loaded");

const modelStatusText = computed(() => {
  const key = modelStatusKey.value as string;
  const params = modelStatusErr.value;
  if (key === "error") return params || "";
  return t(`model_status.${key}`, params);
});

const indexStatusText = computed(() => {
  const state = indexingState.value;
  if (state === "idle") return t("sidebar.indexed_count", { count: indexedFilesCount.value });
  const total = indexTotal.value;
  const fmt =
    total > 0 ? { processed: indexProcessed.value, total } : { processed: "-", total: "-" };
  if (state === "indexing") {
    return t("indexing.state.indexing", fmt);
  }
  if (state === "preparing") {
    if (indexProcessed.value > 0) {
      return t("indexing.state.resuming", fmt);
    }
    return t("indexing.state.preparing", fmt);
  }
  return t(`indexing.state.${state}`, fmt);
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

    if (indexingState.value === "paused") return;
    const isStarted = indexingState.value === "indexing" || indexingState.value === "preparing";
    if (isStarted) {
      await pauseIndexing();
      const unwatch = watch(indexingState, async (newState) => {
        if (newState === "paused") {
          unwatch();
          await startOrResumeIndexing();
        }
      });
    } else if (indexingState.value === "idle") {
      await startOrResumeIndexing();
    }
  } catch (e) {
    indexingState.value = "idle";
    const err = e as AppError;
    await showInfoModal(
      t(`error.${err.code}`, { detail: err.detail }),
      t("message.invalid_directory.title"),
    );
  }
}

async function handleIndexingButton() {
  if (indexingState.value === "idle" || indexingState.value === "paused") {
    if (indexedDirs.value.length === 0) {
      await showInfoModal(
        t("message.no_index.msg"),
        t("indexing.error.empty_library.modal.header"),
      );
      return;
    }
    await startOrResumeIndexing();
  } else if (indexingState.value === "indexing" || indexingState.value === "preparing") {
    await pauseIndexing();
  }
}

async function browseImage() {
  const path = await open({
    multiple: false,
    filters: [
      {
        name: "Images",
        extensions: ["jpg", "jpeg", "png", "bmp", "gif", "webp", "tiff", "avif", "heic", "heif"],
      },
    ],
  });
  if (typeof path === "string") queryImage.value = path;
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
    const errorMsg = t(`error.${err.code}`, { detail: err.detail });
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
