import { useEffect, useLayoutEffect, useRef, useState } from "react";
import type { Collection, Game } from "../types";

export interface GameMenuPosition {
  game: Game;
  x: number;
  y: number;
}

interface GameMenuProps {
  menu: GameMenuPosition;
  collections: Collection[];
  onClose: () => void;
  onLaunch: (game: Game) => void;
  onOpen: (game: Game) => void;
  onToggleHidden: (game: Game) => void;
  onUninstall: (game: Game) => void;
  onToggleCollection: (name: string, game: Game) => void;
  onNewCollection: (game: Game) => void;
  onAddPlaytime: (game: Game) => void;
}

const UNINSTALLABLE: Game["source"][] = ["steam", "gog", "epic", "amazon"];
const EDGE_MARGIN = 12;

export function GameMenu({
  menu,
  collections,
  onClose,
  onLaunch,
  onOpen,
  onToggleHidden,
  onUninstall,
  onToggleCollection,
  onNewCollection,
  onAddPlaytime,
}: GameMenuProps) {
  const panel = useRef<HTMLDivElement | null>(null);
  const [position, setPosition] = useState({ left: menu.x, top: menu.y });
  const { game } = menu;

  useLayoutEffect(() => {
    const element = panel.current;
    if (!element) return;
    const { width, height } = element.getBoundingClientRect();
    setPosition({
      left: Math.min(menu.x, window.innerWidth - width - EDGE_MARGIN),
      top: Math.min(menu.y, window.innerHeight - height - EDGE_MARGIN),
    });
  }, [menu.x, menu.y]);

  useEffect(() => {
    function close(event: Event) {
      if (event instanceof KeyboardEvent && event.key !== "Escape") return;
      if (event.target instanceof Node && panel.current?.contains(event.target)) return;
      onClose();
    }
    window.addEventListener("pointerdown", close);
    window.addEventListener("keydown", close);
    window.addEventListener("wheel", close, { passive: true });
    window.addEventListener("blur", onClose);
    return () => {
      window.removeEventListener("pointerdown", close);
      window.removeEventListener("keydown", close);
      window.removeEventListener("wheel", close);
      window.removeEventListener("blur", onClose);
    };
  }, [onClose]);

  function run(action: () => void) {
    action();
    onClose();
  }

  return (
    <div ref={panel} className="game-menu" role="menu" aria-label={game.title} style={position}>
      <p className="game-menu-title">{game.title}</p>
      <button type="button" role="menuitem" onClick={() => run(() => onLaunch(game))}>
        {game.installed ? "Lancer" : "Installer"}
      </button>
      <button type="button" role="menuitem" onClick={() => run(() => onOpen(game))}>
        Fiche du jeu
      </button>
      <button type="button" role="menuitem" onClick={() => run(() => onToggleHidden(game))}>
        {game.hidden ? "Réafficher" : "Masquer"}
      </button>
      <button type="button" role="menuitem" onClick={() => run(() => onAddPlaytime(game))}>
        Ajouter du temps de jeu
      </button>
      {game.installed && UNINSTALLABLE.includes(game.source) && (
        <button type="button" role="menuitem" onClick={() => run(() => onUninstall(game))}>
          Désinstaller
        </button>
      )}
      <span className="game-menu-separator" aria-hidden="true" />
      <p className="game-menu-label">Collections</p>
      {collections.map((collection) => (
        <button
          key={collection.name}
          type="button"
          role="menuitemcheckbox"
          aria-checked={collection.gameIds.includes(game.id)}
          onClick={() => run(() => onToggleCollection(collection.name, game))}
        >
          {collection.name}
        </button>
      ))}
      <button type="button" role="menuitem" onClick={() => run(() => onNewCollection(game))}>
        Nouvelle collection…
      </button>
    </div>
  );
}
