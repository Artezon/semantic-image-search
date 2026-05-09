import { ref } from "vue";
import type { SearchResult } from "./types";

// Model state
export const modelStatusText = ref("Model not loaded");
export const modelStatusColor = ref("var(--text-secondary)");
export const deviceText = ref("unknown");

// Indexing state
export const indexStatusText = ref("");
export const indexProgress = ref(100);
export const isIndexing = ref(false);

// Results state
export const searchResults = ref<SearchResult[] | null>(null);
export const isSearching = ref(false);
