import { memo } from "react";
import { playtimeLabel } from "../format";
import { DEFAULT_ACCENT, cssVariable } from "../images";
import type { Game } from "../types";
import { CoverImage } from "./CoverImage";

interface LibraryCoverProps {
  game: Game;
  running: boolean;
  onOpen: (game: Game) => void;
  onLaunch: (game: Game) => void;
}

export const LibraryCover = memo(function LibraryCover({
  game,
  running,
  onOpen,
  onLaunch,
}: LibraryCoverProps) {
  const playtime = playtimeLabel(game.details.playtimeMinutes);
  return (
    <button
      type="button"
      className="library-cover"
      aria-label={game.title}
      data-installed={game.installed}
      data-hidden={game.hidden}
      data-running={running}
      style={cssVariable("--item-accent", game.accent ?? DEFAULT_ACCENT)}
      onClick={() => onOpen(game)}
      onDoubleClick={() => onLaunch(game)}
    >
      <CoverImage game={game} />
      {game.installed ? (
        playtime && <span className="cover-chip">{playtime}</span>
      ) : (
        <span className="cover-chip cover-chip-muted">Pas installé</span>
      )}
    </button>
  );
});
