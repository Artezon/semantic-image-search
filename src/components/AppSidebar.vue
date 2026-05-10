<template>
  <div class="app-sidebar">
    <div class="column-titlebar">
      <h1 class="centered-text">Media Search</h1>
      <div class="splash">v0.1-alpha</div>
    </div>

    <div class="status-text model-status">
      {{ modelStatusText }}
    </div>
    <div class="status-text">Device: {{ deviceText }}</div>

    <div class="horizontal-divider"></div>

    <!-- Indexing Section -->
    <h2>Select folder to index</h2>
    <div class="input-group">
      <label>Search directory</label>
      <div class="input-row">
        <input type="text" v-model="indexDir" placeholder="Select search directory" readonly />
        <button class="btn icon-btn" @click="browseDirectory">
          <FolderIcon />
        </button>
      </div>
    </div>

    <div class="param-row">
      <label>Batch size:</label>
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
      <GenerateIcon v-if="!isIndexing" />
      <StopIcon v-else />
      <span v-if="isStopping">Stopping...</span>
      <span v-else-if="isIndexing">Stop indexing</span>
      <span v-else>Index files</span>
    </button>

    <div class="status-text">{{ indexStatusText }}</div>

    <div class="horizontal-divider"></div>

    <!-- Search Section -->
    <h2>Search</h2>
    <div class="radio-group">
      <label class="radio-label">
        <input type="radio" name="search-type" value="text" v-model="searchType" />
        Text search
      </label>
      <label class="radio-label">
        <input type="radio" name="search-type" value="image" v-model="searchType" />
        Image search
      </label>
    </div>

    <div v-if="searchType === 'text'" class="input-group">
      <input
        type="text"
        v-model="queryText"
        placeholder="Enter search query"
        @keyup.enter="search"
      />
    </div>
    <div v-else class="input-group">
      <div class="input-row">
        <input
          type="text"
          v-model="queryImage"
          placeholder="Click the button to select image"
          readonly
        />
        <button class="btn icon-btn" @click="browseImage">
          <ImageIcon />
        </button>
      </div>
    </div>

    <div class="param-row">
      <label>Max results:</label>
      <input
        type="number"
        v-model.lazy.number="maxResults.value"
        :min="maxResults.min"
        :max="maxResults.max"
      />
    </div>

    <div class="param-row">
      <label>Score threshold:</label>
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
      <span>Search</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open, message } from "@tauri-apps/plugin-dialog";
import {
  modelStatusText,
  modelStatusColor,
  deviceText,
  indexStatusText,
  indexProgress,
  isIndexing,
  searchResults,
  isSearching,
} from "../store";
import type { SearchResult } from "../types";
import { FolderIcon, GenerateIcon, ImageIcon, SearchIcon, StopIcon } from "./icons";
import { numericSetting } from "../utils";

const indexDir = ref("");
const batchSize = numericSetting(1, 64, 8);
const queryText = ref("");
const queryImage = ref("");
const searchType = ref<"text" | "image">("text");
const maxResults = numericSetting(1, 4096, 100);
const threshold = numericSetting(0, 1, 0.05, 0.01);
const isStopping = ref(false);

const progressStyle = ref({
  "--progress": "100%",
  "--progress-transition": "none",
});

watch(indexProgress, (next, prev) => {
  progressStyle.value = {
    "--progress": `${next}%`,
    "--progress-transition": next > prev ? "0.1s linear" : "none",
  };
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
  if (isIndexing.value) {
    isStopping.value = true;
    await invoke("stop_indexing");
    isStopping.value = false;
  } else {
    indexStatusText.value = "Preparing...";
    indexProgress.value = 0;
    await invoke("index_directory", {
      dir: indexDir.value,
      batchSize: batchSize.value,
    });
  }
}

async function search() {
  const query = (searchType.value === "text" ? queryText.value : queryImage.value).trim();

  if (!query) {
    searchResults.value = null;
    await message("Please enter query to search.", {
      title: "Empty query",
    });
    return;
  }

  if (maxResults.value < 1 || maxResults.value > 4096) {
    searchResults.value = null;
    await message("Only values between 1 and 4096 are supported.", {
      title: "Invalid threshold value",
    });
    return;
  }

  isSearching.value = true;
  searchResults.value = null;

  try {
    const results = await invoke<Array<SearchResult>>("search", {
      searchType: searchType.value,
      query,
      maxResults: maxResults.value,
      threshold: threshold.value,
    });
    searchResults.value = results;
  } catch (e) {
    searchResults.value = null;
    await message(e as string, { title: "Search error", kind: "error" });
  } finally {
    isSearching.value = false;
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
