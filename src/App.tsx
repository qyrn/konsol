import { useEffect, useMemo, useState } from "react";
import {
  ACHIEVEMENT_EVENT,
  LIBRARY_EVENT,
  errorMessage,
  launchGame,
  removeManualGame,
  setGameHidden,
} from "./api";
import { AchievementsPanel } from "./components/AchievementsPanel";
import { AddGameForm } from "./components/AddGameForm";
import { Backdrop } from "./components/Backdrop";
import { GameStage } from "./components/GameStage";
import { GameTray } from "./components/GameTray";
import { Rail, type SourceFilterValue } from "./components/Rail";
import { RenameForm } from "./components/RenameForm";
import { SettingsForm } from "./components/SettingsForm";
import { DEFAULT_ACCENT, cssVariable } from "./images";
import type { Game } from "./types";
import { useLibrary } from "./useLibrary";
import { useTauriEvent } from "./useTauriEvent";

const NOTICE_DURATION_MS = 4000;

function isTypingTarget(target: EventTarget | null): boolean {
  return target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement;
}

export function App() {
  const library = useLibrary();
  const [sourceFilter, setSourceFilter] = useState<SourceFilterValue>("all");
  const [query, setQuery] = useState("");
  const [showHidden, setShowHidden] = useState(false);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [launchingId, setLaunchingId] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [addingGame, setAddingGame] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [renaming, setRenaming] = useState(false);
  const [achievementsOpen, setAchievementsOpen] = useState(false);
  const [achievementsRefreshKey, setAchievementsRefreshKey] = useState(0);

  useTauriEvent(LIBRARY_EVENT, () => void library.reload());
  useTauriEvent(ACHIEVEMENT_EVENT, () => {
    setAchievementsRefreshKey((key) => key + 1);
    void library.reload();
  });

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

  const selectedGame = visibleGames.find((game) => game.id === selectedId) ?? visibleGames[0] ?? null;

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

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      const modalOpen = addingGame || settingsOpen || achievementsOpen || renaming;
      if (modalOpen || isTypingTarget(event.target) || visibleGames.length === 0) return;
      const index = selectedGame ? visibleGames.indexOf(selectedGame) : 0;
      if (event.key === "ArrowRight" || event.key === "ArrowLeft") {
        event.preventDefault();
        const step = event.key === "ArrowRight" ? 1 : -1;
        const next = visibleGames[(index + step + visibleGames.length) % visibleGames.length];
        if (next) setSelectedId(next.id);
      }
      if (event.key === "Enter" && !(event.target instanceof HTMLButtonElement) && selectedGame) {
        event.preventDefault();
        void handleLaunch(selectedGame);
      }
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  });

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

  const emptyMessage = query.trim()
    ? `Rien ne correspond à « ${query.trim()} ».`
    : "Aucun jeu trouvé pour l'instant. Si t'as des jeux sans launcher, ajoute-les à la main !";

  return (
    <div className="app" style={cssVariable("--accent", selectedGame?.accent ?? DEFAULT_ACCENT)}>
      <Backdrop game={selectedGame} />

      <Rail
        games={shelfGames}
        filter={sourceFilter}
        hiddenCount={hiddenCount}
        showHidden={showHidden}
        onFilterChange={setSourceFilter}
        onShowHiddenChange={setShowHidden}
        onAddGame={() => setAddingGame(true)}
        onOpenSettings={() => setSettingsOpen(true)}
      />

      <header className="toolbar">
        <input
          type="search"
          className="search glass"
          placeholder="Cherche un jeu…"
          aria-label="Chercher un jeu"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
        />
        <button
          type="button"
          className="glass-button"
          disabled={library.loading}
          onClick={() => void library.reload()}
        >
          {library.loading ? "Ça cherche…" : "Actualiser"}
        </button>
      </header>

      <main className="main">
        {library.error ? (
          <p className="message" role="alert">
            {library.error}
          </p>
        ) : selectedGame ? (
          <GameStage
            game={selectedGame}
            launching={launchingId === selectedGame.id}
            onLaunch={(game) => void handleLaunch(game)}
            onToggleHidden={handleToggleHidden}
            onRemove={handleRemove}
            onOpenAchievements={() => setAchievementsOpen(true)}
            onRename={() => setRenaming(true)}
          />
        ) : (
          !library.loading && <p className="message">{emptyMessage}</p>
        )}
      </main>

      {visibleGames.length > 0 && (
        <GameTray
          games={visibleGames}
          selectedId={selectedGame?.id ?? null}
          onSelect={(game) => setSelectedId(game.id)}
          onLaunch={(game) => void handleLaunch(game)}
        />
      )}

      {addingGame && <AddGameForm onAdded={handleAdded} onCancel={() => setAddingGame(false)} />}

      {settingsOpen && (
        <SettingsForm
          onSaved={() => {
            setSettingsOpen(false);
            setNotice("Réglages enregistrés, Konsol va chercher ce qui manque.");
            void library.reload();
          }}
          onCancel={() => setSettingsOpen(false)}
        />
      )}

      {renaming && selectedGame && (
        <RenameForm
          game={selectedGame}
          onRenamed={(title) => {
            setRenaming(false);
            setNotice(`C'est noté, le jeu s'appelle ${title} maintenant.`);
            void library.reload();
          }}
          onCancel={() => setRenaming(false)}
        />
      )}

      {achievementsOpen && selectedGame && (
        <AchievementsPanel
          game={selectedGame}
          refreshKey={achievementsRefreshKey}
          onClose={() => setAchievementsOpen(false)}
        />
      )}

      <div className="notice glass" role="status" aria-live="polite">
        {notice}
      </div>
    </div>
  );
}
