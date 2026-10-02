// What a file is, from its name and media type: the icon and color in the
// list and how the preview shows it.

import {
  File,
  FileArchive,
  FileAudio,
  FileCode,
  FileImage,
  FileSpreadsheet,
  FileText,
  FileType,
  FileVideo,
  type LucideIcon,
  Presentation,
} from "lucide-react";

export type PreviewKind = "image" | "pdf" | "markdown" | "text" | "none";

const IMAGES = ["image/png", "image/jpeg", "image/gif", "image/webp", "image/svg+xml"];
const TEXT = ["application/json", "application/xml", "application/yaml"];

export function previewKind(mediaType: string): PreviewKind {
  if (IMAGES.includes(mediaType)) {
    return "image";
  }
  if (mediaType === "application/pdf") {
    return "pdf";
  }
  if (mediaType === "text/markdown") {
    return "markdown";
  }
  return mediaType.startsWith("text/") || TEXT.includes(mediaType) ? "text" : "none";
}

/** The family a file belongs to; each has its own icon and color (files.css). */
export type FileKind =
  | "pdf"
  | "word"
  | "sheet"
  | "slides"
  | "image"
  | "audio"
  | "video"
  | "archive"
  | "code"
  | "text"
  | "other";

// By extension first: the folder scan sends old Office, OpenDocument and most
// media files as application/octet-stream.
const BY_EXTENSION: Record<string, FileKind> = {
  pdf: "pdf",
  doc: "word",
  docx: "word",
  odt: "word",
  rtf: "word",
  xls: "sheet",
  xlsx: "sheet",
  ods: "sheet",
  csv: "sheet",
  ppt: "slides",
  pptx: "slides",
  odp: "slides",
  zip: "archive",
  rar: "archive",
  "7z": "archive",
  mp3: "audio",
  wav: "audio",
  m4a: "audio",
  ogg: "audio",
  mp4: "video",
  mov: "video",
  webm: "video",
};

const CODE = /\.(ts|tsx|js|jsx|py|rs|css|html?|json|xml|ya?ml|toml|sql|sh|ps1|bat)$/i;

export function fileKind(mediaType: string, name: string): FileKind {
  const dot = name.lastIndexOf(".");
  const known = dot > 0 ? BY_EXTENSION[name.slice(dot + 1).toLowerCase()] : undefined;
  if (known) {
    return known;
  }
  if (mediaType === "application/pdf") {
    return "pdf";
  }
  if (mediaType.includes("wordprocessing")) {
    return "word";
  }
  if (mediaType === "text/csv" || mediaType.includes("spreadsheet")) {
    return "sheet";
  }
  if (mediaType.includes("presentation")) {
    return "slides";
  }
  for (const kind of ["image", "audio", "video"] as const) {
    if (mediaType.startsWith(`${kind}/`)) {
      return kind;
    }
  }
  if (mediaType === "application/zip") {
    return "archive";
  }
  if (mediaType === "text/html" || TEXT.includes(mediaType) || CODE.test(name)) {
    return "code";
  }
  return mediaType === "application/octet-stream" ? "other" : "text";
}

const ICONS: Record<FileKind, LucideIcon> = {
  pdf: FileText,
  word: FileType,
  sheet: FileSpreadsheet,
  slides: Presentation,
  image: FileImage,
  audio: FileAudio,
  video: FileVideo,
  archive: FileArchive,
  code: FileCode,
  text: FileText,
  other: File,
};

export function kindIcon(kind: FileKind): LucideIcon {
  return ICONS[kind];
}
