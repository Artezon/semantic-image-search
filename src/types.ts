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

export type IndexingResult = {
  processed: number;
  total: number;
  elapsed_secs: number;
  stopped: boolean;
  errors: [string, AppError][];
};

export type IndexStatus = {
  processed: number;
  total: number;
  errors: number;
  text_key?: string;
};
