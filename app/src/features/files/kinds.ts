// What a file is, from its media type: the icon in the list and how the
// preview shows it.

import {
  File,
  FileArchive,
  FileAudio,
  FileCode,
  FileImage,
  FileSpreadsheet,
  FileText,
  FileVideo,
  type LucideIcon,
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

export function fileIcon(mediaType: string, name: string): LucideIcon {
  if (mediaType.startsWith("image/")) {
    return FileImage;
  }
  if (mediaType.startsWith("audio/")) {
    return FileAudio;
  }
  if (mediaType.startsWith("video/")) {
    return FileVideo;
  }
  if (mediaType === "application/zip") {
    return FileArchive;
  }
  if (mediaType === "text/csv" || mediaType.includes("spreadsheet")) {
    return FileSpreadsheet;
  }
  if (
    mediaType === "text/html" ||
    TEXT.includes(mediaType) ||
    /\.(ts|tsx|js|py|rs|css)$/i.test(name)
  ) {
    return FileCode;
  }
  return mediaType === "application/octet-stream" ? File : FileText;
}
