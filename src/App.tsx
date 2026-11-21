import { useEffect, useMemo, useState } from "react";
import {
  ACHIEVEMENT_EVENT,
  LIBRARY_EVENT,
  errorMessage,
  launchGame,
  removeManualGame,
  setGameHidden,
  stopGame,
  uninstallGame,
} from "./api";
import { AchievementsPanel } from "./components/AchievementsPanel";
import { AddGameForm } from "./components/AddGameForm";
import { AchievementsPage } from "./components/AchievementsPage";
import { Backdrop } from "./components/Backdrop";
import { CenteredStage } from "./components/CenteredStage";
import { CommandPalette } from "./components/CommandPalette";
import { CoverCarousel } from "./components/CoverCarousel";
import { EmptyShelf } from "./components/EmptyShelf";
import { GameDetail } from "./components/GameDetail";
import { Library } from "./components/Library";
import { NowPlaying } from "./components/NowPlaying";
import { RenameForm } from "./components/RenameForm";
import { SettingsPage } from "./components/SettingsPage";
import { type SessionIndicator, TopBar } from "./components/TopBar";
import { byMostRecent } from "./format";
import { SOURCE_LAUNCHERS } from "./sources";
import { DEFAULT_ACCENT, cssVariable } from "./images";
import type { LaunchStatus } from "./launchStatus";
import type { Game } from "./types";
import { useLibrary } from "./useLibrary";
import { usePreferences } from "./usePreferences";
import { useStableCallback } from "./useStableCallback";
import { useUpdater } from "./useUpdater";
import { UpdateBanner } from "./components/UpdateBanner";
import { useRunningGames } from "./useRunningGames";
import { useSessionStarts } from "./useSessionStarts";
import { useTauriEvent } from "./useTauriEvent";
import { type Overlay, VIEW_VEIL, type View } from "./views";

const NOTICE_DURATION_MS = 4000;
const LAUNCH_TIMEOUT_MS = 90000;

function isTypingTarget(target: EventTarget | null): boolean {
  return target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement;
}

export function App() {
  const library = useLibrary();
  const runningIds = useRunningGames();
  const sessionStarts = useSessionStarts(runningIds);
  const [view, setView] = useState<View>("home");
  const [overlay, setOverlay] = useState<Overlay | null>(null);
  const [showHidden, setShowHidden] = useState(false);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [launchingId, setLaunchingId] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [achievementsRefreshKey, setAchievementsRefreshKey] = useState(0);
  const [stoppingId, setStoppingId] = useState<string | null>(null);
  const updates = useUpdater();
  const { preferences, update: updatePreferences } = usePreferences(setNotice);

  useTauriEvent(LIBRARY_EVENT, () => void library.reload());
  useTauriEvent(ACHIEVEMENT_EVENT, () => {
    setAchievementsRefreshKey((key) => key + 1);
    void library.reload();
  });

  useEffect(() => {
    if (launchingId && runningIds.has(launchingId)) setLaunchingId(null);
  }, [launchingId, runningIds]);

  useEffect(() => {
    if (!launchingId) return;
    const timeout = window.setTimeout(() => setLaunchingId(null), LAUNCH_TIMEOUT_MS);
    return () => window.clearTimeout(timeout);
  }, [launchingId]);

  useEffect(() => {
    if (!notice) return;
    const timeout = window.setTimeout(() => setNotice(null), NOTICE_DURATION_MS);
    return () => window.clearTimeout(timeout);
  }, [notice]);

  const libraryGames = useMemo(
    () => library.games.filter((game) => showHidden || !game.hidden).sort(byMostRecent),
    [library.games, showHidden],
  );
  const visibleGames = useMemo(() => libraryGames.filter((game) => game.installed), [libraryGames]);
  const hiddenCount = library.games.filter((game) => game.hidden).length;
  const selectedGame = visibleGames.find((game) => game.id === selectedId) ?? visibleGames[0] ?? null;
  const focusedGame = libraryGames.find((game) => game.id === selectedId) ?? selectedGame;
  const selectedIndex = selectedGame ? visibleGames.indexOf(selectedGame) : 0;
  const stageGame = view === "home" ? selectedGame : focusedGame;
  const opaqueView = view === "library" || view === "achievements" || view === "settings";
  const backdropGames = opaqueView
    ? []
    : stageGame && !stageGame.installed
      ? [...visibleGames, stageGame]
      : visibleGames;

  function launchStatus(game: Game): LaunchStatus {
    if (runningIds.has(game.id)) return "running";
    return launchingId === game.id ? "launching" : "idle";
  }

  async function handleLaunch(game: Game) {
    setSelectedId(game.id);
    setOverlay(null);
    if (!game.installed) {
      try {
        await launchGame(game.id);
        setNotice(`${SOURCE_LAUNCHERS[game.source]} s'ouvre pour installer ${game.title}.`);
      } catch (error) {
        setNotice(errorMessage(error));
      }
      return;
    }
    if (launchStatus(game) !== "idle") return;
    setLaunchingId(game.id);
    try {
      await launchGame(game.id);
    } catch (error) {
      setNotice(errorMessage(error));
      setLaunchingId(null);
    }
  }

  useEffect(() => {
    if (stoppingId && !runningIds.has(stoppingId)) setStoppingId(null);
  }, [stoppingId, runningIds]);

  async function handleStop(game: Game) {
    setStoppingId(game.id);
    try {
      await stopGame(game.id);
    } catch (error) {
      setNotice(errorMessage(error));
      setStoppingId(null);
    }
  }

  function selectBy(step: number) {
    if (visibleGames.length === 0) return;
    const next = visibleGames[(selectedIndex + step + visibleGames.length) % visibleGames.length];
    if (next) setSelectedId(next.id);
  }

  function navigate(nextView: View) {
    setView(nextView);
    setOverlay(null);
  }

  const openFromLibrary = useStableCallback((game: Game) => openGame(game));
  const launchFromLibrary = useStableCallback((game: Game) => void handleLaunch(game));
  const toggleShowHidden = useStableCallback(() => setShowHidden((value) => !value));

  function openGame(game: Game) {
    setSelectedId(game.id);
    navigate("detail");
  }

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setOverlay("palette");
        return;
      }
      if (event.key === "Escape" && overlay) {
        setOverlay(null);
        return;
      }
      if (overlay || view !== "home" || isTypingTarget(event.target)) return;
      if (event.key === "ArrowRight" || event.key === "ArrowLeft") {
        event.preventDefault();
        selectBy(event.key === "ArrowRight" ? 1 : -1);
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
    if (!game.hidden && !showHidden) navigate("library");
    void runAndReload(
      () => setGameHidden(game.id, !game.hidden),
      game.hidden ? `${game.title} est de retour !` : `${game.title} est masqué.`,
    );
  }

  async function handleUninstall(game: Game) {
    try {
      await uninstallGame(game.id);
      setNotice(
        game.source === "steam"
          ? `Steam te demande de confirmer la désinstallation de ${game.title}.`
          : `Heroic s'ouvre, désinstalle ${game.title} depuis sa page.`,
      );
    } catch (error) {
      setNotice(errorMessage(error));
    }
  }

  function handleRemove(game: Game) {
    setSelectedId(null);
    navigate("library");
    void runAndReload(() => removeManualGame(game.id), `${game.title} est retiré de la liste.`);
  }

  function handleAdded(id: string, title: string) {
    setSelectedId(id);
    navigate("home");
    setNotice(`${title} est ajouté !`);
    void library.reload();
  }

  function rescan() {
    setOverlay(null);
    void library.reload();
  }

  const runningGame = library.games.find((game) => runningIds.has(game.id)) ?? null;
  const launchingGame = library.games.find((game) => game.id === launchingId) ?? null;
  const sessionGame = runningGame ?? launchingGame;
  const session: SessionIndicator | null = sessionGame
    ? { label: runningGame ? "En jeu" : "Lancement…", accent: sessionGame.accent ?? DEFAULT_ACCENT }
    : null;
  const selectedStart = selectedGame ? sessionStarts.get(selectedGame.id) : undefined;

  function renderHome() {
    if (library.error) {
      return (
        <p className="message" role="alert">
          {library.error}
        </p>
      );
    }
    if (library.games.length === 0) {
      if (library.loading) return null;
      return <EmptyShelf scanning={library.loading} onRescan={rescan} onAddGame={() => setOverlay("add")} />;
    }
    if (!selectedGame) {
      return (
        <div className="message">
          <p>Tous tes jeux sont masqués.</p>
          <button type="button" className="glass-button" onClick={() => setShowHidden(true)}>
            Voir les masqués
          </button>
        </div>
      );
    }
    return (
      <div className="home">
        {selectedStart !== undefined ? (
          <NowPlaying
            game={selectedGame}
            startedAt={selectedStart}
            returnsWhenDone={preferences.minimizeDuringSession}
            stopping={stoppingId === selectedGame.id}
            onStop={() => void handleStop(selectedGame)}
          />
        ) : (
          <>
            <CoverCarousel
              games={visibleGames}
              selectedId={selectedGame.id}
              onSelect={(game) => setSelectedId(game.id)}
              onLaunch={(game) => void handleLaunch(game)}
              onPrevious={() => selectBy(-1)}
              onNext={() => selectBy(1)}
            />
            <CenteredStage
              game={selectedGame}
              status={launchStatus(selectedGame)}
              onLaunch={(game) => void handleLaunch(game)}
              onOpenAchievements={() => setOverlay("achievements")}
              onOpenDetail={() => navigate("detail")}
            />
          </>
        )}
      </div>
    );
  }

  return (
    <div
      className="app"
      data-view={view}
      data-flat-accent={!preferences.accentFollowsGame}
      data-reduce-motion={preferences.reduceMotion}
      style={cssVariable(
        "--accent",
        preferences.accentFollowsGame ? (stageGame?.accent ?? DEFAULT_ACCENT) : DEFAULT_ACCENT,
      )}
    >
      <Backdrop games={backdropGames} selectedId={stageGame?.id ?? null} veil={VIEW_VEIL[view]} />

      <TopBar
        view={view}
        session={session}
        onNavigate={navigate}
        onOpenPalette={() => setOverlay("palette")}
        onAddGame={() => setOverlay("add")}
      />

      <main className="view">
        {view === "home" && renderHome()}
        {view === "library" && (
          <Library
            games={libraryGames}
            hiddenCount={hiddenCount}
            showHidden={showHidden}
            runningIds={runningIds}
            onToggleShowHidden={toggleShowHidden}
            onOpen={openFromLibrary}
            onLaunch={launchFromLibrary}
          />
        )}
        {view === "achievements" && (
          <AchievementsPage
            games={library.games}
            refreshKey={achievementsRefreshKey}
            onOpenGame={(game) => {
              setSelectedId(game.id);
              setOverlay("achievements");
            }}
          />
        )}
        {view === "detail" && focusedGame && (
          <GameDetail
            game={focusedGame}
            status={launchStatus(focusedGame)}
            onBack={() => navigate("library")}
            onLaunch={(game) => void handleLaunch(game)}
            onRename={() => setOverlay("rename")}
            onToggleHidden={handleToggleHidden}
            onRemove={handleRemove}
            onUninstall={(game) => void handleUninstall(game)}
            onOpenAchievements={() => setOverlay("achievements")}
          />
        )}
        {view === "settings" && (
          <SettingsPage
            games={library.games}
            scanning={library.loading}
            preferences={preferences}
            updates={updates}
            onPreferencesChange={(next) => void updatePreferences(next)}
            onRescan={rescan}
            onKeysSaved={() => {
              setNotice("Clés enregistrées, Konsol va chercher ce qui manque.");
              void library.reload();
            }}
          />
        )}
      </main>

      {overlay === "palette" && (
        <CommandPalette
          games={libraryGames}
          hiddenCount={hiddenCount}
          showHidden={showHidden}
          onClose={() => setOverlay(null)}
          onOpenGame={openGame}
          onLaunch={(game) => void handleLaunch(game)}
          onAddGame={() => setOverlay("add")}
          onRescan={rescan}
          onToggleShowHidden={() => {
            setShowHidden((value) => !value);
            setOverlay(null);
          }}
          onOpenSettings={() => navigate("settings")}
        />
      )}

      {overlay === "add" && <AddGameForm onAdded={handleAdded} onCancel={() => setOverlay(null)} />}

      {overlay === "rename" && focusedGame && (
        <RenameForm
          game={focusedGame}
          onRenamed={(title) => {
            setOverlay(null);
            setNotice(`C'est noté, le jeu s'appelle ${title} maintenant.`);
            void library.reload();
          }}
          onCancel={() => setOverlay(null)}
        />
      )}

      {overlay === "achievements" && stageGame && (
        <AchievementsPanel
          game={stageGame}
          refreshKey={achievementsRefreshKey}
          onClose={() => setOverlay(null)}
        />
      )}

      {updates.state.kind === "ready" && (
        <UpdateBanner version={updates.state.version} onRestart={updates.restart} />
      )}

      <div className="notice glass" role="status" aria-live="polite">
        {notice}
      </div>
    </div>
  );
}
