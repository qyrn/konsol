import { open } from "@tauri-apps/plugin-dialog";
import { type FormEvent, useState } from "react";
import { addManualGame, errorMessage } from "../api";
import { cssVariable } from "../images";
import type { Game } from "../types";
import { CloseButton } from "./CloseButton";
import { CoverImage } from "./CoverImage";
import { Modal } from "./Modal";

interface AddGameFormProps {
  onAdded: (id: string, title: string) => void;
  onCancel: () => void;
}

const SWATCHES = ["#ff5a36", "#ffb03a", "#6bd16b", "#3fb6a8", "#4a7bff", "#c070ff"] as const;
const DEFAULT_SWATCH = "#4a7bff";

function previewGame(title: string, accent: string): Game {
  return {
    id: "preview",
    title: title.trim() || "Ton jeu",
    source: "manual",
    cover: null,
    hero: null,
    accent,
    hidden: false,
    achievements: null,
    details: {
      playtimeMinutes: null,
      lastPlayed: null,
      sizeBytes: null,
      developer: null,
      year: null,
      genre: null,
      location: null,
    },
  };
}

function fileName(path: string): string {
  return path.split("/").pop() ?? path;
}

async function pickFile(title: string, extensions?: string[]): Promise<string | null> {
  const selection = await open({
    title,
    multiple: false,
    directory: false,
    filters: extensions ? [{ name: "Images", extensions }] : undefined,
  });
  return typeof selection === "string" ? selection : null;
}

export function AddGameForm({ onAdded, onCancel }: AddGameFormProps) {
  const [title, setTitle] = useState("");
  const [executable, setExecutable] = useState("");
  const [launchArguments, setLaunchArguments] = useState("");
  const [accent, setAccent] = useState<string>(DEFAULT_SWATCH);
  const [cover, setCover] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSaving(true);
    setError(null);
    try {
      const id = await addManualGame({ title, executable, arguments: launchArguments, accent, cover });
      onAdded(id, title.trim());
    } catch (submitError) {
      setError(errorMessage(submitError));
      setSaving(false);
    }
  }

  async function browseExecutable() {
    try {
      const path = await pickFile("Choisis le fichier qui lance le jeu");
      if (path) setExecutable(path);
    } catch (pickError) {
      setError(errorMessage(pickError));
    }
  }

  async function browseCover() {
    try {
      const path = await pickFile("Choisis une jaquette", ["png", "jpg", "jpeg", "webp"]);
      if (path) setCover(path);
    } catch (pickError) {
      setError(errorMessage(pickError));
    }
  }

  return (
    <Modal labelledBy="add-game-title" className="form-panel" onClose={onCancel}>
      <form onSubmit={handleSubmit} style={cssVariable("--accent", accent)}>
        <header className="form-panel-header">
          <div>
            <h2 id="add-game-title">Ajouter un jeu</h2>
            <p>Pour ce que Konsol trouve pas tout seul, genre un repack, un recomp ou un émulateur.</p>
          </div>
          <CloseButton onClose={onCancel} />
        </header>
        <div className="add-grid">
          <div className="add-preview-column">
            <span className="add-preview" aria-hidden="true">
              <CoverImage game={previewGame(title, accent)} />
            </span>
            <button type="button" className="cover-picker" onClick={() => void browseCover()}>
              {cover ? fileName(cover) : "Choisir une jaquette"}
            </button>
            {cover && (
              <button type="button" className="text-button" onClick={() => setCover(null)}>
                Enlever la jaquette
              </button>
            )}
          </div>
          <div className="form-fields">
            <label>
              Nom
              <input
                value={title}
                onChange={(event) => setTitle(event.target.value)}
                maxLength={120}
                required
                autoFocus
              />
            </label>
            <label>
              Exécutable
              <span className="field-with-button">
                <input
                  className="monospace"
                  value={executable}
                  onChange={(event) => setExecutable(event.target.value)}
                  placeholder="/home/…/Skate3Recomp-Linux/skate3"
                  spellCheck={false}
                  required
                />
                <button
                  type="button"
                  className="glass-button field-button"
                  onClick={() => void browseExecutable()}
                >
                  Parcourir
                </button>
              </span>
            </label>
            <label>
              Arguments
              <input
                value={launchArguments}
                onChange={(event) => setLaunchArguments(event.target.value)}
                placeholder="Optionnel, genre --fullscreen"
                maxLength={500}
                spellCheck={false}
              />
            </label>
            <div className="swatch-field" role="radiogroup" aria-label="Couleur">
              Couleur
              <span className="swatches">
                {SWATCHES.map((swatch) => (
                  <button
                    key={swatch}
                    type="button"
                    className="swatch"
                    role="radio"
                    aria-checked={accent === swatch}
                    aria-label={swatch}
                    style={{ background: swatch }}
                    onClick={() => setAccent(swatch)}
                  />
                ))}
              </span>
            </div>
          </div>
        </div>
        {error && (
          <p className="form-error" role="alert">
            {error}
          </p>
        )}
        <div className="form-actions">
          <button type="button" className="glass-button" onClick={onCancel}>
            Annuler
          </button>
          <button type="submit" className="launch-button compact" disabled={saving}>
            Ajouter à la bibliothèque
          </button>
        </div>
      </form>
    </Modal>
  );
}
