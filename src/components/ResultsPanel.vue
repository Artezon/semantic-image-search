<template>
  <div class="results-panel" ref="panelRef">
    <div class="column-titlebar results-titlebar">
      <h1 class="centered-text">Search Results</h1>
    </div>

    <div class="results-container">
      <!-- Searching spinner -->
      <div v-if="isSearching" class="centered">
        <div class="spinner"></div>
        <div class="no-results">Searching...</div>
      </div>

      <!-- No results -->
      <div
        v-else-if="searchResults !== null && searchResults.length === 0"
        class="no-results centered"
      >
        No results found
      </div>

      <!-- Results grid -->
      <div v-else-if="searchResults && searchResults.length > 0" class="results-grid">
        <ResultCard
          v-for="result in searchResults"
          :key="result.path"
          :result="result"
          :scroll-root="panelRef"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import ResultCard from "./ResultCard.vue";
import { searchResults, isSearching } from "../store";

const panelRef = ref<HTMLDivElement | null>(null);
</script>

<style scoped>
.results-panel {
  flex: 1;
  padding: 20px;
  overflow-y: auto;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.results-titlebar {
  background-color: var(--surface-translucent);
}

.results-container {
  flex: 1;
}

.results-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 20px;
  margin-top: 20px;
}

.no-results {
  padding: 20px;
  color: var(--text-secondary);
  font-size: 14px;
  text-align: center;
}
</style>
