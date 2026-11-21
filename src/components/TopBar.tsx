import { useNow } from "../useNow";
import type { View } from "../views";
import { BrandMark } from "./BrandMark";

export interface SessionIndicator {
  label: string;
  accent: string;
}

interface TopBarProps {
  view: View;
  session: SessionIndicator | null;
  onNavigate: (view: View) => void;
  onOpenPalette: () => void;
  onAddGame: () => void;
}

const TABS: { view: View; label: string }[] = [
  { view: "home", label: "Accueil" },
  { view: "library", label: "Bibliothèque" },
  { view: "achievements", label: "Succès" },
  { view: "settings", label: "Réglages" },
];

const CLOCK_INTERVAL_MS = 10000;
const clockFormatter = new Intl.DateTimeFormat("fr-FR", { hour: "2-digit", minute: "2-digit" });

function isTabActive(tab: View, view: View): boolean {
  return tab === view || (tab === "library" && view === "detail");
}

export function TopBar({ view, session, onNavigate, onOpenPalette, onAddGame }: TopBarProps) {
  const now = useNow(CLOCK_INTERVAL_MS);

  return (
    <header className="top-bar">
      <div className="wordmark">
        <BrandMark />
        <span>Konsol</span>
      </div>
      <nav className="top-tabs" aria-label="Navigation">
        {TABS.map((tab) => (
          <button
            key={tab.view}
            type="button"
            className="top-tab"
            aria-current={isTabActive(tab.view, view) ? "page" : undefined}
            onClick={() => onNavigate(tab.view)}
          >
            {tab.label}
          </button>
        ))}
      </nav>
      <div className="top-tools">
        {session && (
          <span className="session-indicator">
            <span className="pulse-dot" style={{ background: session.accent, color: session.accent }} />
            {session.label}
          </span>
        )}
        <time className="clock">{clockFormatter.format(now)}</time>
        <button type="button" className="search-pill glass" onClick={onOpenPalette}>
          <span className="search-glyph" aria-hidden="true" />
          Rechercher
          <kbd>Ctrl K</kbd>
        </button>
        <button type="button" className="round-button glass" aria-label="Ajouter un jeu" onClick={onAddGame}>
          +
        </button>
      </div>
    </header>
  );
}
