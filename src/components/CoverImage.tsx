import { useState } from "react";
import { DEFAULT_ACCENT, cssVariable, imageUrl } from "../images";
import { SOURCE_LABELS } from "../sources";
import type { Game } from "../types";

export function CoverImage({ game }: { game: Game }) {
  const [failedSource, setFailedSource] = useState<string | null>(null);
  const source = imageUrl(game.cover);

  if (!source || failedSource === source) {
    return (
      <span
        className="cover cover-typographic"
        style={cssVariable("--cover-accent", game.accent ?? DEFAULT_ACCENT)}
      >
        <span className="cover-source">{SOURCE_LABELS[game.source]}</span>
        <span className="cover-title">{game.title}</span>
      </span>
    );
  }

  return (
    <img
      className="cover"
      src={source}
      alt=""
      loading="lazy"
      decoding="async"
      draggable={false}
      onError={() => setFailedSource(source)}
    />
  );
}
