import type { KeyboardEvent } from "react";
import type { Game } from "../types";
import { GameCover } from "./GameCover";

interface GameGridProps {
  games: Game[];
  selectedId: string | null;
  onSelect: (game: Game) => void;
  onLaunch: (game: Game) => void;
}

export function GameGrid({ games, selectedId, onSelect, onLaunch }: GameGridProps) {
  function handleKeyDown(event: KeyboardEvent<HTMLButtonElement>, game: Game) {
    if (event.key !== "Enter") return;
    event.preventDefault();
    onLaunch(game);
  }

  return (
    <ul className="game-grid">
      {games.map((game) => (
        <li key={game.id}>
          <button
            type="button"
            className="game-tile"
            data-hidden={game.hidden}
            aria-pressed={game.id === selectedId}
            aria-label={game.title}
            onClick={() => onSelect(game)}
            onFocus={() => onSelect(game)}
            onDoubleClick={() => onLaunch(game)}
            onKeyDown={(event) => handleKeyDown(event, game)}
          >
            <GameCover game={game} />
          </button>
        </li>
      ))}
    </ul>
  );
}
