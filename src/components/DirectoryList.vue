<template>
  <ScrollFadeContainer class="dir-list">
    <VueDraggable
      v-model="indexedDirs"
      :animation="200"
      handle=".drag-handle"
      :forceFallback="true"
      @start="onDragStart"
      @end="onDragEnd"
    >
      <div v-for="dir in indexedDirs" :key="dir" class="dir-row">
        <span class="drag-handle">☰</span>
        <span class="dir-path" :title="dir">{{ dir }}</span>
        <button class="btn icon-btn" @click="removeDirectory(dir)">
          <CloseIcon />
        </button>
      </div>
    </VueDraggable>
    <div v-if="indexedDirs.length === 0" class="status-text">
      {{ $t("sidebar.no_directories") }}
    </div>
  </ScrollFadeContainer>
</template>

<script setup lang="ts">
import { onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { indexedDirs, indexingState, indexedFilesCount } from "../store";
import { CloseIcon } from "./icons";
import { VueDraggable } from "vue-draggable-plus";
import ScrollFadeContainer from "./ScrollFadeContainer.vue";

async function loadDirs() {
  indexedDirs.value = await invoke("get_dirs");
}

function onDragStart() {
  document.documentElement.classList.add("dragging");
}

function onDragEnd() {
  document.documentElement.classList.remove("dragging");
  saveDirectoryOrder();
}

async function saveDirectoryOrder() {
  await invoke("reorder_directories", {
    paths: indexedDirs.value,
  });
}

async function removeDirectory(path: string) {
  if (indexingState.value === "indexing" || indexingState.value === "preparing") {
    indexingState.value = "pausing";
    await invoke("pause_indexing");
  }
  await invoke("remove_directory", { path });
  indexedDirs.value = await invoke("get_dirs");
  indexedFilesCount.value = (await invoke("get_indexed_count")) as number;
}

onMounted(loadDirs);
</script>

<style scoped>
.dir-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-height: 150px;
  border: 1px solid var(--outline);
  border-radius: 8px;
  padding: 4px;
}

.dir-list > .status-text {
  font-size: 13px;
  color: var(--text-secondary);
  text-align: center;
  margin: 6px;
}

.dir-row {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px;
  border-radius: 4px;
  user-select: none;
  transition:
    background-color 0.2s,
    box-shadow 0.2s;
}

.drag-handle {
  color: var(--text-secondary);
  font-size: 15px;
  line-height: 1;
  cursor: grab;
  user-select: none;
}

.sortable-ghost {
  opacity: 0;
}

.sortable-fallback {
  background: var(--surface-container);
  border-radius: 3px;
}

.dir-path {
  flex: 1;
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dir-row .icon-btn {
  padding: 2px;
  min-width: 20px;
  min-height: 20px;
}

.dir-row .icon-btn svg {
  width: 12px;
  height: 12px;
}
</style>

<style>
html.dragging {
  cursor: grabbing !important;
}
</style>
