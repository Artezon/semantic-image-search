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
  error_details: Record<string, unknown>;
};

export type AppError = {
  code: string;
  detail?: string;
};

export type BackendMessage = {
  id: string;
  params: Record<string, unknown>;
};

export type IndexingState =
  | "idle"
  | "preparing"
  | "indexing"
  | "pausing"
  | "paused"
  | "fatal_error";

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
  was_paused: boolean;
  errors: [string, AppError][];
};
