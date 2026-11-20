import { useEffect, useMemo, useState } from "react";
import { errorMessage, launchGame, removeManualGame, setGameHidden } from "./api";
import { AddGameForm } from "./components/AddGameForm";
import { GameGrid } from "./components/GameGrid";
import { LaunchDock } from "./components/LaunchDock";
import { SourceFilter, type SourceFilterValue } from "./components/SourceFilter";
import type { Game } from "./types";
import { useLibrary } from "./useLibrary";

const NOTICE_DURATION_MS = 4000;

export function App() {
  const library = useLibrary();
  const [sourceFilter, setSourceFilter] = useState<SourceFilterValue>("all");
  const [query, setQuery] = useState("");
  const [showHidden, setShowHidden] = useState(false);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [launchingId, setLaunchingId] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [addingGame, setAddingGame] = useState(false);

  useEffect(() => {
    if (!notice) return;
    const timeout = window.setTimeout(() => setNotice(null), NOTICE_DURATION_MS);
    return () => window.clearTimeout(timeout);
  }, [notice]);

  const shelfGames = useMemo(
    () => library.games.filter((game) => showHidden || !game.hidden),
    [library.games, showHidden],
  );
  const hiddenCount = library.games.filter((game) => game.hidden).length;

  const visibleGames = useMemo(() => {
    const normalizedQuery = query.trim().toLocaleLowerCase("fr");
    return shelfGames.filter(
      (game) =>
        (sourceFilter === "all" || game.source === sourceFilter) &&
        game.title.toLocaleLowerCase("fr").includes(normalizedQuery),
    );
  }, [shelfGames, sourceFilter, query]);

  const selectedGame = library.games.find((game) => game.id === selectedId) ?? null;

  async function handleLaunch(game: Game) {
    setSelectedId(game.id);
    setLaunchingId(game.id);
    try {
      await launchGame(game.id);
      setNotice(`${game.title} se lance, ça peut prendre quelques secondes.`);
      window.setTimeout(() => setLaunchingId(null), NOTICE_DURATION_MS);
    } catch (error) {
      setNotice(errorMessage(error));
      setLaunchingId(null);
    }
  }

  async function runAndReload(action: () => Promise<void>, successNotice: string) {
    try {
      await action();
      setNotice(successNotice);
      await library.reload();
    } catch (error) {
      setNotice(errorMessage(error));
    }
  }

  function handleToggleHidden(game: Game) {
    void runAndReload(
      () => setGameHidden(game.id, !game.hidden),
      game.hidden ? `${game.title} est de retour.` : `${game.title} est masqué.`,
    );
  }

  function handleRemove(game: Game) {
    setSelectedId(null);
    void runAndReload(() => removeManualGame(game.id), `${game.title} est retiré de la liste.`);
  }

  function handleAdded(title: string) {
    setAddingGame(false);
    setSourceFilter("manual");
    setNotice(`${title} est ajouté !`);
    void library.reload();
  }

  return (
    <div className="app">
      <aside className="sidebar">
        <h1 className="brand">Konsol</h1>
        <SourceFilter games={shelfGames} value={sourceFilter} onChange={setSourceFilter} />
        <div className="sidebar-footer">
          {hiddenCount > 0 && (
            <label className="toggle">
              <input
                type="checkbox"
                checked={showHidden}
                onChange={(event) => setShowHidden(event.target.checked)}
              />
              Voir les jeux masqués ({hiddenCount})
            </label>
          )}
          <button type="button" className="quiet-button" onClick={() => setAddingGame(true)}>
            Ajouter un jeu
          </button>
        </div>
      </aside>

      <main className="library">
        <header className="toolbar">
          <input
            type="search"
            className="search"
            placeholder="Cherche un jeu…"
            aria-label="Chercher un jeu"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
          />
          <button
            type="button"
            className="quiet-button"
            disabled={library.loading}
            onClick={() => void library.reload()}
          >
            {library.loading ? "Ça cherche…" : "Actualiser"}
          </button>
        </header>

        <section className="shelf">
          {addingGame ? (
            <AddGameForm onAdded={handleAdded} onCancel={() => setAddingGame(false)} />
          ) : library.error ? (
            <p className="shelf-message" role="alert">
              {library.error}
            </p>
          ) : visibleGames.length > 0 ? (
            <GameGrid
              games={visibleGames}
              selectedId={selectedId}
              onSelect={(game) => setSelectedId(game.id)}
              onLaunch={(game) => void handleLaunch(game)}
            />
          ) : (
            !library.loading && (
              <p className="shelf-message">
                {query.trim()
                  ? `Rien ne correspond à « ${query.trim()} ».`
                  : "Aucun jeu trouvé pour l'instant. Si t'as des jeux sans launcher, ajoute-les à la main !"}
              </p>
            )
          )}
        </section>

        <LaunchDock
          game={selectedGame}
          launching={launchingId !== null && launchingId === selectedId}
          onLaunch={(game) => void handleLaunch(game)}
          onToggleHidden={handleToggleHidden}
          onRemove={handleRemove}
        />
      </main>

      <div className="notice" role="status" aria-live="polite">
        {notice}
      </div>
    </div>
  );
}
