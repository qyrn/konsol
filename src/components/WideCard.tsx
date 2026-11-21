import { memo } from "react";
import { DEFAULT_ACCENT, cssVariable, imageUrl } from "../images";
import type { Game } from "../types";

interface WideCardProps {
  game: Game;
  caption: string;
  onOpen: (game: Game) => void;
}

export const WideCard = memo(function WideCard({ game, caption, onOpen }: WideCardProps) {
  const art = imageUrl(game.hero) ?? imageUrl(game.cover);
  return (
    <button
      type="button"
      className="wide-card"
      data-installed={game.installed}
      style={cssVariable("--item-accent", game.accent ?? DEFAULT_ACCENT)}
      onClick={() => onOpen(game)}
    >
      {art && (
        <img className="wide-card-art" src={art} alt="" loading="lazy" decoding="async" draggable={false} />
      )}
      <span className="wide-card-text">
        <span className="wide-card-title">{game.title}</span>
        <span className="wide-card-caption">{caption}</span>
      </span>
    </button>
  );
});
