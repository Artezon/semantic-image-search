import { ref } from "vue";
import type { SearchResult } from "./types";

// Model state
export const modelStatusKey = ref("not_loaded");
export const modelStatusColor = ref("var(--text-secondary)");
export const modelStatusParams = ref<Record<string, string>>({});
export const deviceText = ref("unknown");

// Indexing state
export type IndexingStatus = "idle" | "indexing" | "stopping";
export const indexProgress = ref(100);
export const indexProcessed = ref(0);
export const indexTotal = ref(0);
export const indexErrors = ref(0);
export const indexTextKey = ref<string | undefined>(undefined);
export const indexedFilesCount = ref<number | null>(null);
export const indexingStatus = ref<IndexingStatus>("idle");

// Results state
export const searchResults = ref<SearchResult[] | null>(null);
export const isSearching = ref(false);
