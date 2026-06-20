import { ref, watch, reactive, type Ref } from "vue";
import type { Config, IndexingState, SearchResult } from "./types";
import { invoke } from "@tauri-apps/api/core";
import { i18n } from "./i18n";

const { t } = i18n.global;

// Config state
export const config = reactive<Config>({} as Config);
export const configDefault = reactive<Config>({} as Config);

export async function loadConfig() {
  const [current, defaults] = await Promise.all([
    invoke<Config>("get_config"),
    invoke<Config>("get_default_config"),
  ]);
  return [current, defaults];
}

export async function updateConfig(partial: Partial<Config>) {
  if (!Object.keys(config).length) return;
  Object.assign(config, partial);
  await invoke("update_config", { updates: partial });
}

export function bindConfig<K extends keyof Config>(key: K, value: Ref<Config[K]>) {
  watch(value, (val) => {
    updateConfig({ [key]: val } as Partial<Config>);
  });
  return value;
}

loadConfig().then(([current, defaults]) => {
  Object.assign(config, current);
  Object.assign(configDefault, defaults);

  batchSize.value = config.batch_size;
  bindConfig("batch_size", batchSize);
});

// Model state
export const modelStatusKey = ref("not_loaded");
export const modelStatusColor = ref("var(--text-secondary)");
export const modelStatusErr = ref<Record<string, unknown>>({});
export const deviceText = ref(t("device_unknown"));

// Indexing state
export const indexProgress = ref(0);
export const indexProcessed = ref(0);
export const indexTotal = ref(0);
export const indexedFilesCount = ref<number | null>(null);
export const indexingState = ref<IndexingState>("idle");

// UI state
export const indexedDirs = ref<string[]>([]);
export const batchSize = ref(0);
export const queryText = ref("");
export const queryImage = ref("");
export const searchType = ref<"text" | "image">("text");
export const maxResults = ref(100);
export const threshold = ref(0.05);

// Results state
export const searchResults = ref<SearchResult[] | null>(null);
export const isSearching = ref(false);
