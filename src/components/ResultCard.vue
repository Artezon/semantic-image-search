<template>
  <div class="result-card" ref="cardRef" @click="openPath(result.path)">
    <Transition name="fade">
      <div class="state-loading" v-if="!loaded && !error" key="loading">
        <div class="spinner"></div>
        <div class="loading-text">{{ $t("card.loading_preview") }}</div>
      </div>

      <div class="state-loaded" v-else key="loaded">
        <template v-if="error">
          <div class="error-icon">⚠</div>
          <div class="error-text">{{ $t("card.error_loading") }}</div>
        </template>
        <template v-else>
          <img :src="thumbUrl" loading="lazy" :alt="result.filename" />
          <div v-if="result.fileType === 'VID'" class="video-indicator"></div>
        </template>

        <div class="result-card-overlay">
          <div class="result-card-title">{{ result.filename }}</div>
          <div class="result-card-score">
            {{ $t("card.score", { score: result.score.toFixed(4) }) }}
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { openPath } from "@tauri-apps/plugin-opener";
import type { SearchResult } from "../types";

const props = defineProps<{
  result: SearchResult;
  scrollRoot: HTMLElement | null;
}>();

const cardRef = ref<HTMLDivElement | null>(null);
const loaded = ref(false);
const error = ref(false);
const thumbUrl = ref("");

let observer: IntersectionObserver | null = null;
let objectUrl: string | null = null;

async function loadThumbnail() {
  const { result } = props;

  try {
    const thumbData = await invoke<{
      bytes?: number[];
      mime?: string;
    }>("get_thumbnail", {
      path: result.path,
      fileType: result.fileType,
    });
    if (thumbData?.bytes) {
      const blob = new Blob([new Uint8Array(thumbData.bytes)], {
        type: thumbData.mime,
      });
      objectUrl = URL.createObjectURL(blob);
      thumbUrl.value = objectUrl;
      const img = new Image();
      img.onload = () => (loaded.value = true);
      img.onerror = () => (error.value = true);
      img.src = thumbUrl.value;
    } else {
      error.value = true;
    }
  } catch (e) {
    error.value = true;
  }
}

onMounted(() => {
  if (!cardRef.value) return;

  observer = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (entry.isIntersecting) {
          loadThumbnail();
          observer?.unobserve(entry.target);
        }
      }
    },
    {
      root: props.scrollRoot,
      rootMargin: "500px 0px",
    },
  );
  observer.observe(cardRef.value);
});

onUnmounted(() => {
  observer?.disconnect();
  if (objectUrl) URL.revokeObjectURL(objectUrl);
});
</script>

<style scoped>
.result-card {
  aspect-ratio: 1;
  position: relative;
  border-radius: 8px;
  overflow: hidden;
  cursor: pointer;
  transition: transform 0.2s;
  justify-self: center;
  background-color: var(--surface-container);
  width: 100%;
  height: 100%;
  line-height: 1.5;
  box-shadow: 0 0 7px rgba(0, 0, 0, 0.05);
}

.result-card:hover {
  transform: scale(1.02);
}

.state-loading,
.state-loaded {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.result-card img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.result-card-overlay {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  background-color: rgba(0, 0, 0, 0.3);
  backdrop-filter: blur(10px);
  color: white;
  padding: 6px;
}

.result-card-title {
  font-size: 11px;
  font-weight: bold;
  text-align: center;
  overflow-wrap: break-word;
}

.result-card-score {
  font-size: 10px;
  text-align: center;
}

.loading-text {
  margin-top: 10px;
  color: var(--text-disabled);
  font-size: 12px;
}

.error-icon {
  color: red;
  font-size: 50px;
  margin-bottom: -15px;
}

.error-text {
  font-size: 11px;
  text-align: center;
  margin-bottom: 45px;
  width: 100%;
  padding: 0 10px;
  overflow-wrap: break-word;
}

.video-indicator {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 60px;
  height: 60px;
  background-color: rgba(0, 0, 0, 0.3);
  backdrop-filter: blur(10px);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: none;
}

.video-indicator::after {
  content: "";
  width: 0;
  height: 0;
  border-left: 20px solid #eee;
  border-top: 12px solid transparent;
  border-bottom: 12px solid transparent;
  margin-left: 4px;
}
</style>
