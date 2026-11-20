import { convertFileSrc } from "@tauri-apps/api/core";
import { useState } from "react";
import type { Game } from "../types";

function coverSource(game: Game): string | null {
  if (!game.cover) return null;
  return game.cover.kind === "file" ? convertFileSrc(game.cover.value) : game.cover.value;
}

export function GameCover({ game }: { game: Game }) {
  const [failed, setFailed] = useState(false);
  const source = coverSource(game);

  if (!source || failed) {
    return (
      <div className="cover cover-typographic" data-source={game.source}>
        <span>{game.title}</span>
      </div>
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
