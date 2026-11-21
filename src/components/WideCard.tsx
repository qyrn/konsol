import { memo } from "react";
import { DEFAULT_ACCENT, cssVariable, imageUrl } from "../images";
import type { Game } from "../types";

interface WideCardProps {
  game: Game;
  caption: string;
  onOpen: (game: Game) => void;
  onMenu: (game: Game, x: number, y: number) => void;
}

export const WideCard = memo(function WideCard({ game, caption, onOpen, onMenu }: WideCardProps) {
  const capsule = imageUrl(game.capsule);
  const art = capsule ?? imageUrl(game.hero);
  return (
    <button
      type="button"
      className="wide-card"
      data-installed={game.installed}
      data-with-logo={capsule !== null}
      style={cssVariable("--item-accent", game.accent ?? DEFAULT_ACCENT)}
      onClick={() => onOpen(game)}
      onContextMenu={(event) => {
        event.preventDefault();
        onMenu(game, event.clientX, event.clientY);
      }}
    >
      <span className="wide-card-picture">
        {art && <img src={art} alt="" loading="lazy" decoding="async" draggable={false} />}
        {!capsule && <span className="wide-card-title">{game.title}</span>}
      </span>
      <span className="wide-card-caption">
        {capsule && <span className="wide-card-name">{game.title}</span>}
        {caption}
      </span>
    </button>
  );
});
