import { memo, useMemo, useState } from "react";
import { lastPlayedLabel, playtimeLabel } from "../format";
import { gameCount } from "../plural";
import { categorize, suggestGames } from "../suggestions";
import type { Game } from "../types";
import { LibraryCover } from "./LibraryCover";
import { WideCard } from "./WideCard";

interface LibraryProps {
  games: Game[];
  hiddenCount: number;
  showHidden: boolean;
  runningIds: ReadonlySet<string>;
  onToggleShowHidden: () => void;
  onOpen: (game: Game) => void;
  onLaunch: (game: Game) => void;
}

const ALL = "Tout";
const INSTALLED = "Installés";
const RECENT_COUNT = 4;

function resumeCaption(game: Game): string {
  return [playtimeLabel(game.details.playtimeMinutes), lastPlayedLabel(game.details.lastPlayed, Date.now())]
    .filter(Boolean)
    .join(" · ");
}

export const Library = memo(function Library({
  games,
  hiddenCount,
  showHidden,
  runningIds,
  onToggleShowHidden,
  onOpen,
  onLaunch,
}: LibraryProps) {
  const [filter, setFilter] = useState(ALL);
  const [query, setQuery] = useState("");

  const categories = useMemo(() => categorize(games), [games]);
  const suggestions = useMemo(() => suggestGames(games), [games]);
  const recent = useMemo(
    () => games.filter((game) => game.installed && game.details.lastPlayed).slice(0, RECENT_COUNT),
    [games],
  );
  const installedCount = useMemo(() => games.filter((game) => game.installed).length, [games]);

  const normalizedQuery = query.trim().toLocaleLowerCase("fr");
  const searching = normalizedQuery.length > 0;
  const shownCategories = useMemo(() => {
    const matches = (game: Game) =>
      (!searching || game.title.toLocaleLowerCase("fr").includes(normalizedQuery)) &&
      (filter !== INSTALLED || game.installed);
    return categories
      .filter((category) => filter === ALL || filter === INSTALLED || category.name === filter)
      .map((category) => ({ ...category, games: category.games.filter(matches) }))
      .filter((category) => category.games.length > 0);
  }, [categories, filter, normalizedQuery, searching]);

  const showHighlights = filter === ALL && !searching;

  return (
    <section className="library">
      <header className="library-header">
        <div className="library-heading">
          <h1>Bibliothèque</h1>
          <span>
            {gameCount(games.length)}, {installedCount} installé{installedCount > 1 ? "s" : ""}
          </span>
        </div>
        <input
          type="search"
          className="library-search"
          placeholder="Filtrer par nom"
          aria-label="Filtrer par nom"
          value={query}
          spellCheck={false}
          onChange={(event) => setQuery(event.target.value)}
        />
      </header>

      <nav className="library-chips" aria-label="Catégories">
        {[ALL, INSTALLED].map((name) => (
          <button
            key={name}
            type="button"
            className="chip"
            aria-pressed={filter === name}
            onClick={() => setFilter(name)}
          >
            {name}
          </button>
        ))}
        <span className="chip-divider" aria-hidden="true" />
        {categories.map((category) => (
          <button
            key={category.name}
            type="button"
            className="chip"
            aria-pressed={filter === category.name}
            onClick={() => setFilter(filter === category.name ? ALL : category.name)}
          >
            {category.name}
            <span>{category.games.length}</span>
          </button>
        ))}
        {hiddenCount > 0 && (
          <button
            type="button"
            className="chip chip-end"
            aria-pressed={showHidden}
            onClick={onToggleShowHidden}
          >
            Masqués
            <span>{hiddenCount}</span>
          </button>
        )}
      </nav>

      {showHighlights && recent.length > 0 && (
        <section className="library-band" aria-label="Reprendre">
          <h2>Reprendre</h2>
          <div className="wide-row">
            {recent.map((game) => (
              <WideCard key={game.id} game={game} caption={resumeCaption(game)} onOpen={onOpen} />
            ))}
          </div>
        </section>
      )}

      {showHighlights && suggestions.length > 0 && (
        <section className="library-band" aria-label="À essayer">
          <h2>
            À essayer
            <span>d'après les jeux où tu passes le plus de temps</span>
          </h2>
          <div className="wide-row wide-row-scroll">
            {suggestions.map(({ game, because }) => (
              <WideCard
                key={game.id}
                game={game}
                caption={`Dans le même esprit que ${because.title}`}
                onOpen={onOpen}
              />
            ))}
          </div>
        </section>
      )}

      {shownCategories.length === 0 && <p className="library-empty">Aucun jeu ne correspond.</p>}

      {shownCategories.map((category) => (
        <section key={category.name} className="library-category" aria-label={category.name}>
          <h2>
            {category.name}
            <span>{category.games.length}</span>
          </h2>
          <div className="cover-grid">
            {category.games.map((game) => (
              <LibraryCover
                key={game.id}
                game={game}
                running={runningIds.has(game.id)}
                onOpen={onOpen}
                onLaunch={onLaunch}
              />
            ))}
          </div>
        </section>
      ))}
    </section>
  );
});
