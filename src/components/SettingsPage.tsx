import { getVersion } from "@tauri-apps/api/app";
import { useEffect, useState } from "react";
import { gameCount } from "../plural";
import type { UpdaterControls, UpdateState } from "../useUpdater";
import { SOURCE_LABELS, SOURCE_ORDER } from "../sources";
import type { Game, GameSource, Preferences } from "../types";
import { ApiKeysSettings } from "./ApiKeysSettings";
import { BrandMark } from "./BrandMark";
import { GameFoldersSettings } from "./GameFoldersSettings";
import { GeneralSettings } from "./GeneralSettings";

type SettingsTab = "general" | "sources" | "keys" | "about";

interface SettingsPageProps {
  games: Game[];
  scanning: boolean;
  preferences: Preferences;
  onPreferencesChange: (preferences: Preferences) => void;
  updates: UpdaterControls;
  onRescan: () => void;
  onKeysSaved: () => void;
}

const TAB_LABELS: Record<SettingsTab, string> = {
  general: "Général",
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

function SourcesSettings({
  games,
  scanning,
  onRescan,
}: Pick<SettingsPageProps, "games" | "scanning" | "onRescan">) {
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
          const owned = games.filter((game) => game.source === source);
          const count = owned.filter((game) => game.installed).length;
          const notInstalled = owned.length - count;
          return (
            <li key={source} className="source-row" data-found={count > 0}>
              <span className="status-dot" aria-hidden="true" />
              <span className="source-row-name">
                <span>{SOURCE_LABELS[source]}</span>
                <span className="source-row-status">
                  {count > 0 ? `${gameCount(count)} installé${count > 1 ? "s" : ""}` : "Aucun jeu installé"}
                  {notInstalled > 0 && `, ${notInstalled} de plus dans ta bibliothèque`}
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

function updateLine(state: UpdateState): string {
  switch (state.kind) {
    case "checking":
      return "Konsol regarde s'il y a une nouvelle version…";
    case "up-to-date":
      return "T'as la dernière version !";
    case "downloading":
      return `La version ${state.version} se télécharge…`;
    case "ready":
      return `La version ${state.version} est prête, redémarre Konsol pour l'avoir.`;
    case "failed":
      return `Impossible de vérifier : ${state.message}`;
    case "idle":
      return "Les mises à jour s'installent toutes seules au lancement.";
  }
}

interface AboutSettingsProps {
  updates: UpdaterControls;
}

function AboutSettings({ updates }: AboutSettingsProps) {
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
        <p className="field-help">{updateLine(updates.state)}</p>
      </div>
      {updates.state.kind === "ready" ? (
        <button type="button" className="launch-button compact" onClick={updates.restart}>
          Redémarrer
        </button>
      ) : (
        <button
          type="button"
          className="glass-button small-pill"
          disabled={updates.state.kind === "checking" || updates.state.kind === "downloading"}
          onClick={updates.checkNow}
        >
          Vérifier
        </button>
      )}
    </div>
  );
}

export function SettingsPage({
  games,
  scanning,
  preferences,
  onPreferencesChange,
  updates,
  onRescan,
  onKeysSaved,
}: SettingsPageProps) {
  const [tab, setTab] = useState<SettingsTab>("general");

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
          {tab === "general" && <GeneralSettings preferences={preferences} onChange={onPreferencesChange} />}
          {tab === "sources" && (
            <>
              <SourcesSettings games={games} scanning={scanning} onRescan={onRescan} />
              <h2>Dossiers de jeux</h2>
              <GameFoldersSettings onChanged={onRescan} />
            </>
          )}
          {tab === "keys" && <ApiKeysSettings onSaved={onKeysSaved} />}
          {tab === "about" && <AboutSettings updates={updates} />}
        </div>
      </div>
    </section>
  );
}
