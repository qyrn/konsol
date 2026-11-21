import { DEFAULT_ACCENT, cssVariable } from "../images";
import { gameCount } from "../plural";
import { SOURCE_LABELS, SOURCE_ORDER } from "../sources";
import type { Game } from "../types";
import { CoverImage } from "./CoverImage";

interface LibraryShelvesProps {
  games: Game[];
  hiddenCount: number;
  showHidden: boolean;
  runningIds: ReadonlySet<string>;
  onToggleShowHidden: () => void;
  onOpen: (game: Game) => void;
  onLaunch: (game: Game) => void;
}

export function LibraryShelves({
  games,
  hiddenCount,
  showHidden,
  runningIds,
  onToggleShowHidden,
  onOpen,
  onLaunch,
}: LibraryShelvesProps) {
  const shelves = SOURCE_ORDER.map((source) => ({
    source,
    games: games.filter((game) => game.source === source),
  })).filter((shelf) => shelf.games.length > 0);

  return (
    <section className="library">
      <header className="library-header">
        <div className="library-heading">
          <h1>Bibliothèque</h1>
          <span>{gameCount(games.length)}</span>
        </div>
        {hiddenCount > 0 && (
          <button type="button" className="glass-button small-pill" onClick={onToggleShowHidden}>
            {showHidden ? "Cacher les masqués" : `Voir les masqués (${hiddenCount})`}
          </button>
        )}
      </header>
      {shelves.map((shelf) => (
        <section key={shelf.source} className="shelf" aria-label={SOURCE_LABELS[shelf.source]}>
          <h2>
            {SOURCE_LABELS[shelf.source]}
            <span>{shelf.games.length}</span>
          </h2>
          <ul className="shelf-row">
            {shelf.games.map((game) => (
              <li key={game.id}>
                <button
                  type="button"
                  className="shelf-item"
                  aria-label={game.title}
                  data-hidden={game.hidden}
                  data-running={runningIds.has(game.id)}
                  style={cssVariable("--item-accent", game.accent ?? DEFAULT_ACCENT)}
                  onClick={() => onOpen(game)}
                  onDoubleClick={() => onLaunch(game)}
                >
                  <CoverImage game={game} />
                </button>
              </li>
            ))}
          </ul>
        </section>
      ))}
    </section>
  );
}
