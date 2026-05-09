export type SearchResult = {
  path: string;
  filename: string;
  fileType: "DOC" | "IMG" | "VID" | "AUD";
  score: number;
};

export type ModelStatus = {
  status: "neutral" | "success" | "error";
  status_text: string;
  device_text: string;
};

export type IndexStatus = {
  progress: number;
  text: string;
};
