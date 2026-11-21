import { gameCount } from "../plural";
import { shelvesByGenre, suggestGames } from "../suggestions";
import type { Game } from "../types";
import { ShelfItem } from "./ShelfItem";

interface LibraryShelvesProps {
  games: Game[];
  hiddenCount: number;
  showHidden: boolean;
  installedOnly: boolean;
  runningIds: ReadonlySet<string>;
  onToggleShowHidden: () => void;
  onToggleInstalledOnly: () => void;
  onOpen: (game: Game) => void;
  onLaunch: (game: Game) => void;
}

export function LibraryShelves({
  games,
  hiddenCount,
  showHidden,
  installedOnly,
  runningIds,
  onToggleShowHidden,
  onToggleInstalledOnly,
  onOpen,
  onLaunch,
}: LibraryShelvesProps) {
  const shown = installedOnly ? games.filter((game) => game.installed) : games;
  const suggestions = suggestGames(games);
  const installedCount = games.filter((game) => game.installed).length;

  return (
    <section className="library">
      <header className="library-header">
        <div className="library-heading">
          <h1>Bibliothèque</h1>
          <span>
            {gameCount(games.length)}, {installedCount} installé{installedCount > 1 ? "s" : ""}
          </span>
        </div>
        <div className="library-filters">
          {installedCount < games.length && (
            <button
              type="button"
              className="filter-pill"
              aria-pressed={installedOnly}
              onClick={onToggleInstalledOnly}
            >
              Installés seulement
            </button>
          )}
          {hiddenCount > 0 && (
            <button
              type="button"
              className="filter-pill"
              aria-pressed={showHidden}
              onClick={onToggleShowHidden}
            >
              Masqués ({hiddenCount})
            </button>
          )}
        </div>
      </header>
      {suggestions.length > 0 && (
        <section className="shelf shelf-suggestions" aria-label="À essayer">
          <h2>
            À essayer
            <span>d'après ce que tu joues le plus</span>
          </h2>
          <ul className="shelf-row">
            {suggestions.map(({ game, because }) => (
              <ShelfItem
                key={game.id}
                game={game}
                running={runningIds.has(game.id)}
                caption={`Parce que tu joues à ${because.title}`}
                onOpen={onOpen}
                onLaunch={onLaunch}
              />
            ))}
          </ul>
        </section>
      )}
      {shelvesByGenre(shown).map((shelf) => (
        <section key={shelf.genre} className="shelf" aria-label={shelf.genre}>
          <h2>
            {shelf.genre}
            <span>{shelf.games.length}</span>
          </h2>
          <ul className="shelf-row">
            {shelf.games.map((game) => (
              <ShelfItem
                key={game.id}
                game={game}
                running={runningIds.has(game.id)}
                onOpen={onOpen}
                onLaunch={onLaunch}
              />
            ))}
          </ul>
        </section>
      ))}
    </section>
  );
}
