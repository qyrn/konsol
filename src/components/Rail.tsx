import { SOURCE_LABELS, SOURCE_ORDER } from "../sources";
import type { Game, GameSource } from "../types";
import { BrandMark } from "./BrandMark";

export type SourceFilterValue = GameSource | "all";

interface RailProps {
  games: Game[];
  filter: SourceFilterValue;
  hiddenCount: number;
  showHidden: boolean;
  onFilterChange: (value: SourceFilterValue) => void;
  onShowHiddenChange: (value: boolean) => void;
  onAddGame: () => void;
}

export function Rail({
  games,
  filter,
  hiddenCount,
  showHidden,
  onFilterChange,
  onShowHiddenChange,
  onAddGame,
}: RailProps) {
  const options: { value: SourceFilterValue; label: string; count: number }[] = [
    { value: "all", label: "Tous", count: games.length },
    ...SOURCE_ORDER.map((source) => ({
      value: source,
      label: SOURCE_LABELS[source],
      count: games.filter((game) => game.source === source).length,
    })).filter((option) => option.count > 0),
  ];

  return (
    <aside className="rail glass">
      <div className="wordmark">
        <BrandMark />
        <span>Konsol</span>
      </div>
      <nav aria-label="Sources">
        <ul className="source-list">
          {options.map((option) => (
            <li key={option.value}>
              <button
                type="button"
                className="source-option"
                aria-pressed={option.value === filter}
                onClick={() => onFilterChange(option.value)}
              >
                <span>{option.label}</span>
                <span className="source-count">{option.count}</span>
              </button>
            </li>
          ))}
        </ul>
      </nav>
      <div className="rail-footer">
        {hiddenCount > 0 && (
          <label className="toggle">
            <input
              type="checkbox"
              checked={showHidden}
              onChange={(event) => onShowHiddenChange(event.target.checked)}
            />
            Voir les masqués ({hiddenCount})
          </label>
        )}
        <button type="button" className="glass-button" onClick={onAddGame}>
          Ajouter un jeu
        </button>
      </div>
    </aside>
  );
}
