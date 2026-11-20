import { SOURCE_LABELS, SOURCE_ORDER } from "../sources";
import type { Game, GameSource } from "../types";

export type SourceFilterValue = GameSource | "all";

interface SourceFilterProps {
  games: Game[];
  value: SourceFilterValue;
  onChange: (value: SourceFilterValue) => void;
}

export function SourceFilter({ games, value, onChange }: SourceFilterProps) {
  const presentSources = SOURCE_ORDER.filter((source) => games.some((game) => game.source === source));
  const options: { value: SourceFilterValue; label: string; count: number }[] = [
    { value: "all", label: "Tous", count: games.length },
    ...presentSources.map((source) => ({
      value: source,
      label: SOURCE_LABELS[source],
      count: games.filter((game) => game.source === source).length,
    })),
  ];

  return (
    <nav aria-label="Sources">
      <ul className="source-list">
        {options.map((option) => (
          <li key={option.value}>
            <button
              type="button"
              className="source-option"
              data-source={option.value}
              aria-pressed={option.value === value}
              onClick={() => onChange(option.value)}
            >
              <span>{option.label}</span>
              <span className="source-count">{option.count}</span>
            </button>
          </li>
        ))}
      </ul>
    </nav>
  );
}
