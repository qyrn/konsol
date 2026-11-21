import { useState } from "react";
import { imageUrl } from "../images";
import type { Game } from "../types";

export function CoverImage({ game }: { game: Game }) {
  const [failed, setFailed] = useState(false);
  const source = imageUrl(game.cover);

  if (!source || failed) {
    return (
      <span className="cover cover-typographic">
        <span>{game.title}</span>
      </span>
    );
  }

  return (
    <img
      className="cover"
      src={source}
      alt=""
      loading="lazy"
      draggable={false}
      onError={() => setFailed(true)}
    />
  );
}
