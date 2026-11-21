import { useEffect, useRef } from "react";
import { DEFAULT_ACCENT, cssVariable } from "../images";
import type { Game } from "../types";
import { CoverImage } from "./CoverImage";

interface GameTrayProps {
  games: Game[];
  selectedId: string | null;
  onSelect: (game: Game) => void;
  onLaunch: (game: Game) => void;
}

export function GameTray({ games, selectedId, onSelect, onLaunch }: GameTrayProps) {
  const selectedRef = useRef<HTMLButtonElement | null>(null);

  useEffect(() => {
    selectedRef.current?.scrollIntoView({ block: "nearest", inline: "nearest", behavior: "smooth" });
  }, [selectedId]);

  return (
    <nav className="tray glass" aria-label="Jeux">
      <ul className="tray-list">
        {games.map((game) => {
          const selected = game.id === selectedId;
          return (
            <li key={game.id}>
              <button
                ref={selected ? selectedRef : undefined}
                type="button"
                className="tray-item"
                data-hidden={game.hidden}
                aria-pressed={selected}
                aria-label={game.title}
                style={cssVariable("--item-accent", game.accent ?? DEFAULT_ACCENT)}
                onClick={() => onSelect(game)}
                onDoubleClick={() => onLaunch(game)}
              >
                <CoverImage game={game} />
              </button>
            </li>
          );
        })}
      </ul>
    </nav>
  );
}
