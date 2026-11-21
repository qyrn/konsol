import { playtimeLabel } from "../format";
import { DEFAULT_ACCENT, cssVariable } from "../images";
import type { Game } from "../types";
import { CoverImage } from "./CoverImage";

interface ShelfItemProps {
  game: Game;
  running: boolean;
  caption?: string;
  onOpen: (game: Game) => void;
  onLaunch: (game: Game) => void;
}

export function ShelfItem({ game, running, caption, onOpen, onLaunch }: ShelfItemProps) {
  const badge = game.installed ? playtimeLabel(game.details.playtimeMinutes) : "Pas installé";
  return (
    <li className="shelf-entry">
      <button
        type="button"
        className="shelf-item"
        aria-label={game.title}
        data-hidden={game.hidden}
        data-installed={game.installed}
        data-running={running}
        style={cssVariable("--item-accent", game.accent ?? DEFAULT_ACCENT)}
        onClick={() => onOpen(game)}
        onDoubleClick={() => game.installed && onLaunch(game)}
      >
        <CoverImage game={game} />
        {badge && <span className="shelf-badge">{badge}</span>}
      </button>
      {caption && <span className="shelf-caption">{caption}</span>}
    </li>
  );
}
