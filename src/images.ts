import { convertFileSrc } from "@tauri-apps/api/core";
import type { CSSProperties } from "react";
import type { ImageSource } from "./types";

export const DEFAULT_ACCENT = "#4a7bff";

export function imageUrl(source: ImageSource | null): string | null {
  if (!source) return null;
  return source.kind === "file" ? convertFileSrc(source.value) : source.value;
}

export function cssVariable(name: `--${string}`, value: string): CSSProperties {
  return { [name]: value } as CSSProperties;
}
