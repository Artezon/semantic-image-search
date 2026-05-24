<template>
  <div class="app-sidebar">
    <div class="column-titlebar">
      <h1 class="centered-text">{{ $t("app.title") }}</h1>
      <div class="splash">{{ $t("app.version") }}</div>
    </div>

    <div class="status-text model-status">
      {{ modelStatusText }}
    </div>
    <div class="status-text">{{ $t("sidebar.device", { device: deviceText }) }}</div>

    <div class="horizontal-divider"></div>

    <!-- Indexing Section -->
    <h2>{{ $t("sidebar.select_folder") }}</h2>
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
      <input
        type="number"
        v-model.lazy.number="batchSize.value"
        :min="batchSize.min"
        :max="batchSize.max"
      />
    </div>

    <button
      class="btn full-width-btn progress-btn"
      :style="progressStyle"
      @click="handleIndexingButton"
    >
      <GenerateIcon v-if="indexingStatus === 'idle'" />
      <StopIcon v-else />
      <span v-if="indexingStatus === 'stopping'">{{ $t("sidebar.stopping") }}</span>
      <span v-else-if="indexingStatus === 'indexing'">{{ $t("sidebar.stop_indexing") }}</span>
      <span v-else>{{ $t("sidebar.index_files") }}</span>
    </button>

    <div class="status-text">
      <template v-if="indexTextKey !== undefined">{{ indexStatusText }}</template>
      <template v-else-if="indexedFilesCount !== null">{{
        $t("sidebar.indexed_count", { count: indexedFilesCount })
      }}</template>
    </div>

    <div class="horizontal-divider"></div>

    <!-- Search Section -->
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
      <input
        type="number"
        v-model.lazy.number="maxResults.value"
        :min="maxResults.min"
        :max="maxResults.max"
      />
    </div>

    <div class="param-row">
      <label>{{ $t("sidebar.score_threshold") }}</label>
      <input
        type="number"
        v-model.lazy.number="threshold.value"
        :min="threshold.min"
        :max="threshold.max"
        :step="threshold.step"
      />
    </div>

    <button class="btn full-width-btn" @click="search">
      <SearchIcon />
      <span>{{ $t("sidebar.search") }}</span>
    </button>
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
  indexTextKey,
  indexedFilesCount,
  indexingStatus,
  searchResults,
  isSearching,
} from "../store";
import type { IndexingResult, SearchResult } from "../types";
import { FolderIcon, GenerateIcon, ImageIcon, SearchIcon, StopIcon } from "./icons";
import { numericSetting } from "../utils";

const { t } = useI18n();

const indexDir = ref("");
const batchSize = numericSetting(1, 64, 8);
const queryText = ref("");
const queryImage = ref("");
const searchType = ref<"text" | "image">("text");
const maxResults = numericSetting(1, 4096, 100);
const threshold = numericSetting(0, 1, 0.05, 0.01);

const modelStatusText = computed(() => {
  const key = modelStatusKey.value as string;
  const params = modelStatusParams.value;
  if (key === "error") return t("model_status.error", { error: params.error || "" });
  return t(`model_status.${key}`, params);
});

const indexStatusText = computed(() => {
  const key = indexTextKey.value;
  if (key === "idle") return "";
  if (key === "indexing") {
    const processed = indexProcessed.value;
    const total = indexTotal.value;
    const errors = indexErrors.value;
    if (errors > 0) return t("index_status.indexing_with_errors", { processed, total, errors });
    return t("index_status.indexing", { processed, total });
  }
  return t(`index_status.${key}`, {
    processed: indexProcessed.value,
    total: indexTotal.value,
  });
});

const progressStyle = ref({
  "--progress": "100%",
  "--progress-transition": "none",
});

watch(indexProgress, (next, prev) => {
  if (indexingStatus.value !== "indexing") return;
  progressStyle.value = {
    "--progress": `${next * 100}%`,
    "--progress-transition": next > prev ? "0.1s linear" : "none",
  };
});

watch(indexingStatus, (val, oldVal) => {
  if (val === "idle" && oldVal !== "idle") {
    progressStyle.value = {
      "--progress": "100%",
      "--progress-transition": "none",
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
  if (indexingStatus.value === "indexing") {
    indexingStatus.value = "stopping";
    await invoke("stop_indexing");
  } else if (indexingStatus.value === "idle") {
    indexProcessed.value = 0;
    indexTotal.value = 0;
    indexErrors.value = 0;
    indexTextKey.value = "preparing";
    indexProgress.value = 0;
    indexingStatus.value = "indexing";
    const result = await invoke<IndexingResult | null>("index_directory", {
      dir: indexDir.value,
      batchSize: batchSize.value,
    });
    indexingStatus.value = "idle";

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
  padding: 20px;
  overflow-x: hidden;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 10px;
  overflow-wrap: break-word;
}

.status-text {
  font-size: 12px;
  color: var(--text-secondary);
  text-align: center;
}

.model-status {
  color: v-bind(modelStatusColor);
}

.horizontal-divider {
  margin: 0 -20px;
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
