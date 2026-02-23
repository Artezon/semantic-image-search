import { invoke } from "@tauri-apps/api/core";
import { open, message } from "@tauri-apps/plugin-dialog";
import { openPath } from "@tauri-apps/plugin-opener";
import { getCurrentWindow } from "@tauri-apps/api/window";

const appWindow = getCurrentWindow();

document.getElementById("titlebar-minimize")!.onclick = appWindow.minimize;
document.getElementById("titlebar-maximize")!.onclick =
  appWindow.toggleMaximize;
document.getElementById("titlebar-close")!.onclick = appWindow.close;

window.onload = () => setTimeout(appWindow.show, 100);

const icons = {
  generate: `<path fill-rule="evenodd" d="M9 4.5a.75.75 0 0 1 .721.544l.813 2.846a3.75 3.75 0 0 0 2.576 2.576l2.846.813a.75.75 0 0 1 0 1.442l-2.846.813a3.75 3.75 0 0 0-2.576 2.576l-.813 2.846a.75.75 0 0 1-1.442 0l-.813-2.846a3.75 3.75 0 0 0-2.576-2.576l-2.846-.813a.75.75 0 0 1 0-1.442l2.846-.813A3.75 3.75 0 0 0 7.466 7.89l.813-2.846A.75.75 0 0 1 9 4.5ZM18 1.5a.75.75 0 0 1 .728.568l.258 1.036c.236.94.97 1.674 1.91 1.91l1.036.258a.75.75 0 0 1 0 1.456l-1.036.258c-.94.236-1.674.97-1.91 1.91l-.258 1.036a.75.75 0 0 1-1.456 0l-.258-1.036a2.625 2.625 0 0 0-1.91-1.91l-1.036-.258a.75.75 0 0 1 0-1.456l1.036-.258a2.625 2.625 0 0 0 1.91-1.91l.258-1.036A.75.75 0 0 1 18 1.5ZM16.5 15a.75.75 0 0 1 .712.513l.394 1.183c.15.447.5.799.948.948l1.183.395a.75.75 0 0 1 0 1.422l-1.183.395c-.447.15-.799.5-.948.948l-.395 1.183a.75.75 0 0 1-1.422 0l-.395-1.183a1.5 1.5 0 0 0-.948-.948l-1.183-.395a.75.75 0 0 1 0-1.422l1.183-.395c.447-.15.799-.5.948-.948l.395-1.183A.75.75 0 0 1 16.5 15Z" clip-rule="evenodd" />`,
  stop: `<path fill-rule="evenodd" d="M4.5 7.5a3 3 0 0 1 3-3h9a3 3 0 0 1 3 3v9a3 3 0 0 1-3 3h-9a3 3 0 0 1-3-3v-9Z" clip-rule="evenodd" />`,
};

const modelStatus = document.getElementById("model-status")!;
const deviceLabel = document.getElementById("device-label")!;

const indexDirInput = document.getElementById("index-dir") as HTMLInputElement;
const indexBatchSize = document.getElementById(
  "batch-size",
) as HTMLInputElement;
const indexButton = document.getElementById("indexing-btn")!;
const indexButtonIcon = document.querySelector("#indexing-btn > svg")!;
const indexButtonText = document.querySelector("#indexing-btn > span")!;
const indexStatus = document.getElementById("index-status")!;

const searchByTextContainer = document.getElementById("text-input-container")!;
const searchByImageContainer = document.getElementById(
  "image-input-container",
)!;
const queryText = document.getElementById("query-text") as HTMLInputElement;
const queryImage = document.getElementById("query-image") as HTMLInputElement;
const maxResultsInput = document.getElementById(
  "max-results",
) as HTMLInputElement;
const thresholdInput = document.getElementById("threshold") as HTMLInputElement;

const rightPanel = document.getElementById("right-panel");
const resultsContainer = document.getElementById("results-container")!;

type SearchResult = {
  path: string;
  file_type: "IMG" | "VID";
  score: number;
  filename: string;
};

interface SearchResultCard extends HTMLDivElement {
  result: SearchResult;
  thumbUrl: string;
}

let isIndexing = false;

document
  .querySelectorAll<HTMLInputElement>('input[name="search-type"]')
  .forEach((el) => {
    el.onchange = (event) => onSearchTypeChange(event);
  });
let searchType = document.querySelector<HTMLInputElement>(
  'input[name="search-type"]:checked',
)!.value;

document.getElementById("index-dir-browse-btn")!.onclick = browseDirectory;
indexButton.onclick = handleIndexingButton;
document.getElementById("query-img-browse-btn")!.onclick = browseImage;
document.getElementById("search-btn")!.onclick = search;

const thumbnailObserver = new IntersectionObserver(
  (entries) => {
    for (const entry of entries) {
      if (entry.isIntersecting) {
        loadThumbnail(entry.target as SearchResultCard);
        thumbnailObserver.unobserve(entry.target);
      }
    }
  },
  {
    root: rightPanel,
    rootMargin: "200px 0px",
  },
);

async function showMessage(
  title: string,
  msg: string,
  kind: "error" | "info" | "warning" | undefined = "info",
): Promise<void> {
  await message(msg, { title: title, kind: kind });
}

function onSearchTypeChange(event: Event): void {
  const input = event.target as HTMLInputElement;
  if (input.checked) {
    searchType = input.value;

    searchByTextContainer.classList.toggle("hidden", searchType !== "text");
    searchByImageContainer.classList.toggle("hidden", searchType !== "image");
  }
}

async function browseDirectory(): Promise<void> {
  const path = await open({ directory: true });

  if (typeof path === "string") {
    indexDirInput.value = path;
  }
}

async function browseImage(): Promise<void> {
  const path = await open({
    multiple: false,
    filters: [
      {
        name: "Images",
        extensions: ["jpg", "jpeg", "png", "bmp", "gif", "webp", "tiff"],
      },
    ],
  });

  if (typeof path === "string") {
    queryImage.value = path;
  }
}

function updateModelStatus(
  status: string,
  color: string,
  device: string,
): void {
  modelStatus.textContent = status;
  modelStatus.style.color = color;
  deviceLabel.textContent = `Device: ${device}`;
}

function updateIndexStatus(progress: number, text: string): void {
  indexButton.style.setProperty("--progress", `${progress * 100}%`);
  indexStatus.textContent = text;
}

async function handleIndexingButton(): Promise<void> {
  if (isIndexing) {
    await invoke("stop_indexing");
  } else {
    await invoke("process_directory");
    updateIndexStatus(0, "Preparing...");
  }
}

function setProcessingButtonState(indexing: boolean): void {
  isIndexing = indexing;

  indexButtonIcon.innerHTML = indexing ? icons.stop : icons.generate;
  indexButtonText.textContent = indexing
    ? "Stop indexing"
    : "Generate embeddings";
}

async function search(): Promise<void> {
  const query = searchType === "text" ? queryText.value : queryImage.value;

  const maxResults = Number(maxResultsInput.value);
  const threshold = Number(thresholdInput.value);

  await invoke("search", {
    searchType,
    query,
    maxResults,
    threshold,
  });
}

function searchingNow() {
  clearResults();
  resultsContainer.innerHTML = `<div class="centered">
      <div class="spinner"></div>
      <div class="no-results">Searching...</div>
  </div>`;
}

function showThumbnailError(
  card: SearchResultCard,
  result: SearchResult,
): void {
  card.innerHTML = `
    <div style="color: red; font-size: 50px;">⚠</div>
    <div style="font-size: 10px; text-align: center; margin-top: 10px;">
      Error loading thumbnail<br>${result.filename}
    </div>
  `;
}

function populateCard(
  card: SearchResultCard,
  result: SearchResult,
  thumbnailSrc: string,
): void {
  card.classList.remove("loading-card");

  card.innerHTML = `
    <img src="${thumbnailSrc}" loading="lazy" alt="${result.filename}">
    ${result.file_type === "VID" ? '<div class="video-indicator"></div>' : ""}
    <div class="result-card-overlay">
      <div class="result-card-title">${result.filename}</div>
      <div class="result-card-score">
        Score: ${result.score.toFixed(4)}
      </div>
    </div>
  `;
}

function loadThumbnail(card: SearchResultCard) {
  const result = card.result;

  invoke<{
    bytes?: Uint8Array;
    mime?: string;
  }>("get_thumbnail", {
    path: result.path,
    fileType: result.file_type,
  })
    .then((thumbData) => {
      if (thumbData?.bytes) {
        const blob = new Blob([thumbData.bytes], { type: thumbData.mime });
        const url = URL.createObjectURL(blob);

        populateCard(card, result, url);
        card.thumbUrl = url;
      } else {
        showThumbnailError(card, result);
      }
    })
    .catch(() => {
      showThumbnailError(card, result);
    });
}

async function displayResults(results: SearchResult[] | null): Promise<void> {
  clearResults();

  if (!results || results.length === 0) {
    resultsContainer.innerHTML =
      '<div class="no-results centered">No results found</div>';
    return;
  }

  const grid = document.createElement("div");
  grid.className = "results-grid";

  for (const result of results) {
    const card = document.createElement("div") as SearchResultCard;
    card.className = "result-card loading-card";
    card.onclick = () => openPath(result.path);

    card.result = result; // store search result for deferred thumbnail loading

    card.innerHTML = `
      <div class="spinner"></div>
      <div style="margin-top: 10px; color: var(--text-disabled); font-size: 12px;">
        Loading preview...
      </div>
    `;

    grid.appendChild(card);

    thumbnailObserver.observe(card);
  }

  resultsContainer.appendChild(grid);
}

function clearResults() {
  resultsContainer
    .querySelectorAll<SearchResultCard>(".result-card")
    .forEach(destroyCard);
  resultsContainer.innerHTML = "";
}

function destroyCard(card: SearchResultCard) {
  if (card.thumbUrl) {
    URL.revokeObjectURL(card.thumbUrl);
  }
  card.remove();
}
