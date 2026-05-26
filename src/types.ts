export type Config = {
  lang: string;
  batch_size: number;
  video_frames: number;
};

export type SearchResult = {
  path: string;
  filename: string;
  fileType: "DOC" | "IMG" | "VID" | "AUD";
  score: number;
};

export type ModelStatus = {
  status: "neutral" | "success" | "error";
  status_key: string;
  device_text: string;
  params?: Record<string, string>;
};

export type AppError = {
  code: string;
  msg?: string;
};

export type IndexingState = "idle" | "preparing" | "indexing" | "stopping" | "fatal_error";

export type IndexStatus = {
  state: IndexingState;
  processed: number;
  total: number;
  errors: number;
};

export type IndexingResult = {
  processed: number;
  total: number;
  elapsed_secs: number;
  stopped: boolean;
  errors: [string, AppError][];
};
