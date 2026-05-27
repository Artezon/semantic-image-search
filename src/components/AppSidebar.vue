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
      <div class="input-group">
        <label>{{ $t("sidebar.search_directory") }}</label>
        <div class="input-row">
          <input
            type="text"
            v-model="indexDir"
            :placeholder="$t('sidebar.directory_placeholder')"
            readonly
          />
          <button class="btn icon-btn" @click="browseDirectory">
            <FolderIcon />
          </button>
        </div>
      </div>

      <div class="param-row">
        <label>{{ $t("sidebar.batch_size") }}</label>
        <NumberInput v-model="batchSize" :min="1" :max="64" />
      </div>

      <button
        class="btn full-width-btn progress-btn"
        :style="progressStyle"
        @click="handleIndexingButton"
      >
        <GenerateIcon v-if="indexingState === 'idle'" />
        <StopIcon v-else />
        <span v-if="indexingState === 'stopping'">{{ $t("sidebar.stopping") }}</span>
        <span v-else-if="indexingState === 'indexing' || indexingState === 'preparing'">{{
          $t("sidebar.stop_indexing")
        }}</span>
        <span v-else>{{ $t("sidebar.index_files") }}</span>
      </button>

      <div class="status-text">{{ indexStatusText }}</div>

      <button class="btn full-width-btn" @click="clearIndex">
        <DeleteIcon />
        <span>{{ $t("sidebar.clear_index") }}</span>
      </button>
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

      <button class="btn full-width-btn" @click="search">
        <SearchIcon />
        <span>{{ $t("sidebar.search") }}</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { open, message } from "@tauri-apps/plugin-dialog";
import {
  modelStatusKey,
  modelStatusColor,
  deviceText,
  modelStatusParams,
  indexProgress,
  indexProcessed,
  indexTotal,
  indexErrors,
  indexedFilesCount,
  indexingState,
  searchResults,
  isSearching,
  indexDir,
  queryText,
  queryImage,
  searchType,
} from "../store";
import type { IndexingResult, SearchResult } from "../types";
import { FolderIcon, GenerateIcon, ImageIcon, SearchIcon, StopIcon, DeleteIcon } from "./icons";
import { batchSize, maxResults, threshold } from "../store";
import NumberInput from "./NumberInput.vue";

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
    const errors = indexErrors.value;
    if (errors > 0) return t("index_status.indexing_with_errors", { processed, total, errors });
    return t("index_status.indexing", { processed, total });
  }
  return t(`index_status.${state}`, {
    processed: indexProcessed.value,
    total: indexTotal.value,
  });
});

const progressStyle = ref({
  "--progress": "100%",
  "--progress-transition": "none",
});

watch(indexProgress, (p) => {
  if (indexingState.value === "idle") return;
  progressStyle.value = {
    "--progress": `${p * 100}%`,
    "--progress-transition": p > 0 ? "0.1s linear" : "none",
  };
});

watch(indexingState, (val, oldVal) => {
  if (val === "idle" && oldVal !== "idle") {
    progressStyle.value = {
      "--progress": "100%",
      "--progress-transition": "none",
    };
  } else {
    progressStyle.value = {
      "--progress": `${indexProgress.value * 100}%`,
      "--progress-transition": indexProgress.value > 0 ? "0.1s linear" : "none",
    };
  }
});

async function browseDirectory() {
  const path = await open({ directory: true });
  if (typeof path === "string") indexDir.value = path;
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
    indexingState.value = "stopping";
    await invoke("stop_indexing");
  } else if (indexingState.value === "idle") {
    indexingState.value = "preparing";
    indexProcessed.value = 0;
    indexTotal.value = 0;
    indexErrors.value = 0;
    indexProgress.value = 0;
    const result = await invoke<IndexingResult | null>("index_directory", {
      dir: indexDir.value,
    });
    indexingState.value = "idle";
    indexedFilesCount.value = (await invoke("get_indexed_count")) as number;

    if (result) {
      const { processed, total, elapsed_secs, stopped, errors: errorsArr } = result;
      const suffix = stopped ? "stopped" : "complete";

      let summary = t("message.index_result.msg", { processed, total, elapsed: elapsed_secs });

      if (errorsArr.length > 0) {
        const maxShow = 5;
        const shown = errorsArr.slice(0, maxShow);
        const lines = shown.map(
          ([path, err]) =>
            `${t("message.index_result.errors.skipped")} ${path}\n${err.msg ?? err.code}`,
        );
        summary += `\n\n${t("message.index_result.errors.header", { count: errorsArr.length })}:\n\n${lines.join("\n")}`;
        if (errorsArr.length > maxShow) {
          const extra = errorsArr.length - maxShow;
          summary += `\n${t("message.index_result.errors.more", { count: extra })}`;
        }
      }

      await message(summary, {
        title: t(`message.index_result.${suffix}.title`),
        kind: "info",
      });
    }
  }
}

async function clearIndex() {
  if (indexingState.value === "indexing" || indexingState.value === "preparing") {
    indexingState.value = "stopping";
    await invoke("stop_indexing");
  }
  await invoke("clear_index");
  indexedFilesCount.value = (await invoke("get_indexed_count")) as number;
}

async function search() {
  searchResults.value = null;

  const query = (searchType.value === "text" ? queryText.value : queryImage.value).trim();

  if (!query) {
    await message(t("message.empty_query.msg"), {
      title: t("message.empty_query.title"),
    });
    return;
  }

  if (maxResults.value < 1 || maxResults.value > 4096) {
    await message(t("message.invalid_threshold.msg"), {
      title: t("message.invalid_threshold.title"),
    });
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
    const err = e as { code: string; msg?: string };
    if (err.code === "no_index") {
      await message(t("message.no_index.msg"), {
        title: t("message.no_index.title"),
        kind: "error",
      });
    } else {
      const errorMsg = t(`error.${err.code}`, { message: err.msg });
      await message(errorMsg, {
        title: t("message.search_error.title"),
        kind: "error",
      });
    }
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
