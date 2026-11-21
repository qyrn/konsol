import { type KeyboardEvent, useState } from "react";
import { DEFAULT_ACCENT, cssVariable } from "../images";
import { SOURCE_LABELS } from "../sources";
import type { Game } from "../types";
import { CoverImage } from "./CoverImage";
import { Modal } from "./Modal";

interface PaletteAction {
  glyph: string;
  label: string;
  hint: string;
  run: () => void;
}

interface CommandPaletteProps {
  games: Game[];
  hiddenCount: number;
  showHidden: boolean;
  onClose: () => void;
  onOpenGame: (game: Game) => void;
  onLaunch: (game: Game) => void;
  onAddGame: () => void;
  onRescan: () => void;
  onToggleShowHidden: () => void;
  onOpenSettings: () => void;
}

const MAX_RESULTS = 5;

export function CommandPalette({
  games,
  hiddenCount,
  showHidden,
  onClose,
  onOpenGame,
  onLaunch,
  onAddGame,
  onRescan,
  onToggleShowHidden,
  onOpenSettings,
}: CommandPaletteProps) {
  const [query, setQuery] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);

  const normalizedQuery = query.trim().toLocaleLowerCase("fr");
  const results = games
    .filter((game) => game.title.toLocaleLowerCase("fr").includes(normalizedQuery))
    .slice(0, MAX_RESULTS);
  const activeGame = results[Math.min(activeIndex, results.length - 1)] ?? null;

  const actions: PaletteAction[] = [
    { glyph: "+", label: "Ajouter un jeu", hint: "Repack, recomp, émulateur", run: onAddGame },
    { glyph: "↻", label: "Analyser ce PC", hint: "Steam, Heroic, raccourcis", run: onRescan },
    ...(hiddenCount > 0
      ? [
          {
            glyph: "◐",
            label: showHidden ? "Cacher les jeux masqués" : "Voir les jeux masqués",
            hint: hiddenCount === 1 ? "1 masqué" : `${hiddenCount} masqués`,
            run: onToggleShowHidden,
          },
        ]
      : []),
    { glyph: "≡", label: "Réglages", hint: "Sources, clés API", run: onOpenSettings },
  ];

  function handleKeyDown(event: KeyboardEvent<HTMLInputElement>) {
    if (results.length === 0) return;
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      const step = event.key === "ArrowDown" ? 1 : -1;
      setActiveIndex((index) => (index + step + results.length) % results.length);
    }
    if (event.key === "Enter" && activeGame) {
      event.preventDefault();
      onLaunch(activeGame);
    }
    if (event.key === "Tab" && activeGame) {
      event.preventDefault();
      onOpenGame(activeGame);
    }
  }

  return (
    <Modal labelledBy="palette-input" className="palette" placement="top" onClose={onClose}>
      <div className="palette-search">
        <span className="search-glyph" aria-hidden="true" />
        <input
          id="palette-input"
          aria-label="Rechercher un jeu ou une action"
          placeholder="Rechercher un jeu ou une action"
          value={query}
          autoFocus
          spellCheck={false}
          onChange={(event) => {
            setQuery(event.target.value);
            setActiveIndex(0);
          }}
          onKeyDown={handleKeyDown}
        />
        <kbd>Échap</kbd>
      </div>
      <div className="palette-sections">
        <p className="palette-heading">Jeux</p>
        {results.length === 0 && <p className="palette-empty">Aucun jeu ne correspond.</p>}
        <ul className="palette-list">
          {results.map((game) => (
            <li
              key={game.id}
              className="palette-game"
              data-active={game.id === activeGame?.id}
              style={cssVariable("--item-accent", game.accent ?? DEFAULT_ACCENT)}
              onMouseEnter={() => setActiveIndex(results.indexOf(game))}
            >
              <button type="button" className="palette-game-open" onClick={() => onOpenGame(game)}>
                <span className="palette-thumb">
                  <CoverImage game={game} />
                </span>
                <span className="palette-game-text">
                  <span className="palette-game-title">{game.title}</span>
                  <span className="palette-game-meta">{SOURCE_LABELS[game.source]}</span>
                </span>
              </button>
              <button type="button" className="palette-launch" onClick={() => onLaunch(game)}>
                Lancer
              </button>
            </li>
          ))}
        </ul>
        <p className="palette-heading">Actions</p>
        <ul className="palette-list">
          {actions.map((action) => (
            <li key={action.label}>
              <button type="button" className="palette-action" onClick={action.run}>
                <span className="palette-glyph" aria-hidden="true">
                  {action.glyph}
                </span>
                <span className="palette-action-label">{action.label}</span>
                <span className="palette-action-hint">{action.hint}</span>
              </button>
            </li>
          ))}
        </ul>
      </div>
      <footer className="palette-footer">
        <span>↑↓ Naviguer</span>
        <span>↵ Lancer</span>
        <span>Tab Fiche du jeu</span>
        <span className="palette-footer-end">Ctrl K partout</span>
      </footer>
    </Modal>
  );
}
