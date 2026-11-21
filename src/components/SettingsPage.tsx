import { getVersion } from "@tauri-apps/api/app";
import { useEffect, useState } from "react";
import { gameCount } from "../plural";
import { SOURCE_LABELS, SOURCE_ORDER } from "../sources";
import type { Game, GameSource } from "../types";
import { ApiKeysSettings } from "./ApiKeysSettings";
import { BrandMark } from "./BrandMark";

type SettingsTab = "sources" | "keys" | "about";

interface SettingsPageProps {
  games: Game[];
  scanning: boolean;
  onRescan: () => void;
  onKeysSaved: () => void;
}

const TAB_LABELS: Record<SettingsTab, string> = {
  sources: "Sources",
  keys: "Clés API",
  about: "À propos",
};

const SOURCE_LOCATIONS: Record<GameSource, string> = {
  steam: "~/.local/share/Steam",
  gog: "~/.config/heroic",
  epic: "~/.config/heroic",
  amazon: "~/.config/heroic",
  desktop: "~/.local/share/applications",
  manual: "~/.config/dev.qyrn.konsol/config.json",
};

function SourcesSettings({ games, scanning, onRescan }: Omit<SettingsPageProps, "onKeysSaved">) {
  return (
    <>
      <div className="settings-section-header">
        <p className="settings-lead">
          Konsol lit ce que tes launchers ont installé sur ce PC, il va juste chercher les jaquettes en ligne.
        </p>
        <button type="button" className="glass-button" disabled={scanning} onClick={onRescan}>
          {scanning ? "Ça cherche…" : "Tout analyser"}
        </button>
      </div>
      <ul className="settings-panel source-rows">
        {SOURCE_ORDER.map((source) => {
          const count = games.filter((game) => game.source === source).length;
          return (
            <li key={source} className="source-row" data-found={count > 0}>
              <span className="status-dot" aria-hidden="true" />
              <span className="source-row-name">
                <span>{SOURCE_LABELS[source]}</span>
                <span className="source-row-status">
                  {count > 0 ? `${gameCount(count)} trouvé${count > 1 ? "s" : ""}` : "Aucun jeu trouvé"}
                </span>
              </span>
              <span className="source-row-path monospace">{SOURCE_LOCATIONS[source]}</span>
            </li>
          );
        })}
      </ul>
    </>
  );
}

function AboutSettings() {
  const [version, setVersion] = useState<string | null>(null);

  useEffect(() => {
    getVersion()
      .then(setVersion)
      .catch(() => setVersion(null));
  }, []);

  return (
    <div className="settings-panel about-row">
      <BrandMark />
      <div>
        <p className="about-name">Konsol {version}</p>
        <p className="field-help">
          Tes jeux masqués, tes renommages et tes clés sont dans ~/.config/dev.qyrn.konsol.
        </p>
      </div>
    </div>
  );
}

export function SettingsPage({ games, scanning, onRescan, onKeysSaved }: SettingsPageProps) {
  const [tab, setTab] = useState<SettingsTab>("sources");

  return (
    <section className="settings">
      <div className="settings-grid">
        <div>
          <h1 className="settings-title">Réglages</h1>
          <nav className="settings-nav" aria-label="Réglages">
            {(Object.keys(TAB_LABELS) as SettingsTab[]).map((key) => (
              <button
                key={key}
                type="button"
                className="settings-tab"
                aria-current={tab === key ? "page" : undefined}
                onClick={() => setTab(key)}
              >
                {TAB_LABELS[key]}
              </button>
            ))}
          </nav>
        </div>
        <div className="settings-content">
          <h2>{TAB_LABELS[tab]}</h2>
          {tab === "sources" && <SourcesSettings games={games} scanning={scanning} onRescan={onRescan} />}
          {tab === "keys" && <ApiKeysSettings onSaved={onKeysSaved} />}
          {tab === "about" && <AboutSettings />}
        </div>
      </div>
    </section>
  );
}
