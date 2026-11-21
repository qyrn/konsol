import { memo, useMemo, useState } from "react";
import { lastPlayedLabel, playtimeLabel } from "../format";
import { isOldGame } from "../oldGames";
import { gameCount } from "../plural";
import { categorize, suggestGames } from "../suggestions";
import type { Collection, Game } from "../types";
import { DiscoverRow } from "./DiscoverRow";
import { LibraryCover } from "./LibraryCover";
import { WideCard } from "./WideCard";

interface LibraryProps {
  games: Game[];
  collections: Collection[];
  hiddenCount: number;
  showHidden: boolean;
  runningIds: ReadonlySet<string>;
  onToggleShowHidden: () => void;
  onOpen: (game: Game) => void;
  onLaunch: (game: Game) => void;
  onMenu: (game: Game, x: number, y: number) => void;
  onError: (message: string) => void;
}

type Filter =
  | { kind: "all" }
  | { kind: "installed" }
  | { kind: "old" }
  | { kind: "category"; name: string }
  | { kind: "collection"; name: string };

const RECENT_COUNT = 4;

function resumeCaption(game: Game): string {
  return [playtimeLabel(game.details.playtimeMinutes), lastPlayedLabel(game.details.lastPlayed, Date.now())]
    .filter(Boolean)
    .join(" · ");
}

function sameFilter(first: Filter, second: Filter): boolean {
  return JSON.stringify(first) === JSON.stringify(second);
}

export const Library = memo(function Library({
  games,
  collections,
  hiddenCount,
  showHidden,
  runningIds,
  onToggleShowHidden,
  onOpen,
  onLaunch,
  onMenu,
  onError,
}: LibraryProps) {
  const [filter, setFilter] = useState<Filter>({ kind: "all" });
  const [query, setQuery] = useState("");

  const nowSeconds = Math.floor(Date.now() / 1000);
  const { current, old } = useMemo(() => {
    const isOld = (game: Game) => isOldGame(game, nowSeconds);
    return { current: games.filter((game) => !isOld(game)), old: games.filter(isOld) };
  }, [games, nowSeconds]);
  const categories = useMemo(() => categorize(current), [current]);
  const suggestions = useMemo(() => suggestGames(current), [current]);
  const recent = useMemo(
    () => current.filter((game) => game.installed && game.details.lastPlayed).slice(0, RECENT_COUNT),
    [current],
  );
  const installedCount = useMemo(() => games.filter((game) => game.installed).length, [games]);

  const normalizedQuery = query.trim().toLocaleLowerCase("fr");
  const searching = normalizedQuery.length > 0;
  const shownCategories = useMemo(() => {
    const collection =
      filter.kind === "collection" ? collections.find((item) => item.name === filter.name) : undefined;
    const pool = filter.kind === "old" ? old : filter.kind === "collection" ? games : current;
    const matches = (game: Game) =>
      (!searching || game.title.toLocaleLowerCase("fr").includes(normalizedQuery)) &&
      (filter.kind !== "installed" || game.installed) &&
      (!collection || collection.gameIds.includes(game.id));
    return categorize(pool)
      .filter((category) => filter.kind !== "category" || category.name === filter.name)
      .map((category) => ({ ...category, games: category.games.filter(matches) }))
      .filter((category) => category.games.length > 0);
  }, [collections, current, filter, games, normalizedQuery, old, searching]);

  const showHighlights = filter.kind === "all" && !searching;

  function chip(label: string, target: Filter, count?: number, extraClass = "") {
    const active = sameFilter(filter, target);
    return (
      <button
        key={`${target.kind}-${label}`}
        type="button"
        className={`chip ${extraClass}`}
        aria-pressed={active}
        onClick={() => setFilter(active ? { kind: "all" } : target)}
      >
        {label}
        {count !== undefined && <span>{count}</span>}
      </button>
    );
  }

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

      <nav className="library-chips" aria-label="Filtres">
        {chip("Tout", { kind: "all" })}
        {chip("Installés", { kind: "installed" })}
        {collections.map((collection) =>
          chip(
            collection.name,
            { kind: "collection", name: collection.name },
            collection.gameIds.length,
            "chip-collection",
          ),
        )}
        <span className="chip-divider" aria-hidden="true" />
        {categories.map((category) =>
          chip(category.name, { kind: "category", name: category.name }, category.games.length),
        )}
        <span className="chip-end">
          {old.length > 0 && chip("Plus de 5 ans", { kind: "old" }, old.length)}
          {hiddenCount > 0 && (
            <button type="button" className="chip" aria-pressed={showHidden} onClick={onToggleShowHidden}>
              Masqués
              <span>{hiddenCount}</span>
            </button>
          )}
        </span>
      </nav>

      {showHighlights && recent.length > 0 && (
        <section className="library-band" aria-label="Reprendre">
          <h2>Reprendre</h2>
          <div className="wide-row">
            {recent.map((game) => (
              <WideCard
                key={game.id}
                game={game}
                caption={resumeCaption(game)}
                onOpen={onOpen}
                onMenu={onMenu}
              />
            ))}
          </div>
        </section>
      )}

      {showHighlights && suggestions.length > 0 && (
        <section className="library-band" aria-label="À essayer">
          <h2>
            À essayer
            <span>des jeux que t'as déjà, jamais lancés, qui ressemblent à tes préférés</span>
          </h2>
          <div className="wide-row wide-row-scroll">
            {suggestions.map(({ game, because }) => (
              <WideCard
                key={game.id}
                game={game}
                caption={`Dans le même esprit que ${because.title}`}
                onOpen={onOpen}
                onMenu={onMenu}
              />
            ))}
          </div>
        </section>
      )}

      {showHighlights && <DiscoverRow onError={onError} />}

      {shownCategories.length === 0 && (
        <p className="library-empty">
          {filter.kind === "collection"
            ? "Cette collection est vide, fais un clic droit sur un jeu pour l'y ajouter."
            : "Aucun jeu ne correspond."}
        </p>
      )}

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
                onMenu={onMenu}
              />
            ))}
          </div>
        </section>
      ))}
    </section>
  );
});
