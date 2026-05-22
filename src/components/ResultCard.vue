<template>
  <div class="result-card" ref="cardRef" @click="openPath(result.path)">
    <!-- Loading -->
    <template v-if="!loaded && !error">
      <div class="spinner"></div>
      <div class="loading-text">Loading preview...</div>
    </template>

    <template v-else>
      <template v-if="error">
        <div class="error-icon">⚠</div>
        <div class="error-text">Error loading thumbnail</div>
      </template>
      <template v-else>
        <img :src="thumbUrl" loading="lazy" :alt="result.filename" />
        <div v-if="result.fileType === 'VID'" class="video-indicator"></div>
      </template>

      <div class="result-card-overlay">
        <div class="result-card-title">{{ result.filename }}</div>
        <div class="result-card-score">Score: {{ result.score.toFixed(4) }}</div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
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

  const isHeic = (filename: string) =>
    [".heic", ".heif"].some((ext) => filename.toLowerCase().endsWith(ext));

  const fromFrontend = result.fileType === "IMG" && !isHeic(result.filename);
  const fromBackend = result.fileType === "VID" || isHeic(result.filename);

  if (fromFrontend) {
    try {
      const src = convertFileSrc(result.path);
      objectUrl = await resizedImage(src);
      thumbUrl.value = objectUrl;
      loaded.value = true;
    } catch {
      error.value = true;
    }
  } else if (fromBackend) {
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
  } else {
    error.value = true;
  }
}

async function resizedImage(src: string): Promise<string> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.crossOrigin = "anonymous";
    img.onload = () => {
      const MAX = 512;
      let { width, height } = img;

      if (width > MAX || height > MAX) {
        if (width > height) {
          height = Math.round((height / width) * MAX);
          width = MAX;
        } else {
          width = Math.round((width / height) * MAX);
          height = MAX;
        }
      }

      const canvas = document.createElement("canvas");
      canvas.width = width;
      canvas.height = height;
      canvas.getContext("2d")!.drawImage(img, 0, 0, width, height);
      canvas.toBlob(
        (blob) => {
          if (!blob) return reject(new Error("toBlob failed"));
          const url = URL.createObjectURL(blob);
          resolve(url);
        },
        "image/jpeg",
        0.85,
      );
    };
    img.onerror = reject;
    img.src = src;
  });
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
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background-color: var(--surface-container);
  width: 100%;
  height: 100%;
  line-height: 1.5;
}

.result-card:hover {
  transform: scale(1.02);
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
