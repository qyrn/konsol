import { SOURCE_LABELS } from "../sources";
import type { Game } from "../types";

interface LaunchDockProps {
  game: Game | null;
  launching: boolean;
  onLaunch: (game: Game) => void;
  onToggleHidden: (game: Game) => void;
  onRemove: (game: Game) => void;
}

export function LaunchDock({ game, launching, onLaunch, onToggleHidden, onRemove }: LaunchDockProps) {
  if (!game) {
    return (
      <footer className="dock dock-empty">
        <p>Choisis un jeu, double-clic ou Entrée pour le lancer direct.</p>
      </footer>
    );
  }

  return (
    <footer className="dock" data-source={game.source}>
      <div className="dock-info">
        <span className="dock-source">{SOURCE_LABELS[game.source]}</span>
        <h2 className="dock-title">{game.title}</h2>
      </div>
      <div className="dock-actions">
        {game.source === "manual" ? (
          <button type="button" className="quiet-button" onClick={() => onRemove(game)}>
            Retirer
          </button>
        ) : (
          <button type="button" className="quiet-button" onClick={() => onToggleHidden(game)}>
            {game.hidden ? "Réafficher" : "Masquer"}
          </button>
        )}
        <button type="button" className="launch-button" disabled={launching} onClick={() => onLaunch(game)}>
          {launching ? "Ça se lance…" : "Lancer"}
        </button>
      </div>
    </footer>
  );
}
