import { SOURCE_LABELS } from "../sources";
import type { Game } from "../types";

interface GameStageProps {
  game: Game;
  launching: boolean;
  onLaunch: (game: Game) => void;
  onToggleHidden: (game: Game) => void;
  onRemove: (game: Game) => void;
}

export function GameStage({ game, launching, onLaunch, onToggleHidden, onRemove }: GameStageProps) {
  return (
    <section className="stage" aria-live="polite">
      <p className="stage-source">{SOURCE_LABELS[game.source]}</p>
      <h1 key={game.id} className="stage-title">
        {game.title}
      </h1>
      <div className="stage-actions">
        <button type="button" className="launch-button" disabled={launching} onClick={() => onLaunch(game)}>
          {launching ? "Ça se lance…" : "Lancer"}
        </button>
        {game.source === "manual" ? (
          <button type="button" className="glass-button" onClick={() => onRemove(game)}>
            Retirer
          </button>
        ) : (
          <button type="button" className="glass-button" onClick={() => onToggleHidden(game)}>
            {game.hidden ? "Réafficher" : "Masquer"}
          </button>
        )}
      </div>
    </section>
  );
}
