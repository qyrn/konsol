import { type FormEvent, useState } from "react";
import { addManualGame, errorMessage } from "../api";
import type { Game } from "../types";
import { CloseButton } from "./CloseButton";
import { CoverImage } from "./CoverImage";
import { Modal } from "./Modal";

interface AddGameFormProps {
  onAdded: (id: string, title: string) => void;
  onCancel: () => void;
}

function previewGame(title: string): Game {
  return {
    id: "preview",
    title: title.trim() || "Ton jeu",
    source: "manual",
    cover: null,
    hero: null,
    accent: null,
    hidden: false,
    achievements: null,
  };
}

export function AddGameForm({ onAdded, onCancel }: AddGameFormProps) {
  const [title, setTitle] = useState("");
  const [executable, setExecutable] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSaving(true);
    setError(null);
    try {
      const id = await addManualGame(title, executable);
      onAdded(id, title.trim());
    } catch (submitError) {
      setError(errorMessage(submitError));
      setSaving(false);
    }
  }

  return (
    <Modal labelledBy="add-game-title" className="form-panel" onClose={onCancel}>
      <form onSubmit={handleSubmit}>
        <header className="form-panel-header">
          <div>
            <h2 id="add-game-title">Ajouter un jeu</h2>
            <p>Pour ce que Konsol trouve pas tout seul, genre un repack, un recomp ou un émulateur.</p>
          </div>
          <CloseButton onClose={onCancel} />
        </header>
        <div className="add-grid">
          <span className="add-preview" aria-hidden="true">
            <CoverImage game={previewGame(title)} />
          </span>
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
              <input
                className="monospace"
                value={executable}
                onChange={(event) => setExecutable(event.target.value)}
                placeholder="/home/…/Skate3Recomp-Linux/skate3"
                spellCheck={false}
                required
              />
            </label>
            <p className="field-help">Mets le chemin complet du fichier qui lance le jeu.</p>
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
