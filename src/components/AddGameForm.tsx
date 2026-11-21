import { type FormEvent, type KeyboardEvent, useState } from "react";
import { addManualGame, errorMessage } from "../api";

interface AddGameFormProps {
  onAdded: (title: string) => void;
  onCancel: () => void;
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
      await addManualGame(title, executable);
      onAdded(title.trim());
    } catch (submitError) {
      setError(errorMessage(submitError));
      setSaving(false);
    }
  }

  function handleKeyDown(event: KeyboardEvent<HTMLFormElement>) {
    if (event.key === "Escape") onCancel();
  }

  return (
    <div className="modal-layer">
      <form
        className="add-form glass"
        role="dialog"
        aria-modal="true"
        aria-labelledby="add-game-title"
        onSubmit={handleSubmit}
        onKeyDown={handleKeyDown}
      >
        <h2 id="add-game-title">Ajouter un jeu</h2>
        <p className="add-form-help">
          Pour un jeu sans launcher, genre un repack ou un recomp, mets juste le chemin complet du fichier qui
          le lance.
        </p>
        <label>
          Nom du jeu
          <input
            value={title}
            onChange={(event) => setTitle(event.target.value)}
            maxLength={120}
            required
            autoFocus
          />
        </label>
        <label>
          Chemin de l'exécutable
          <input
            value={executable}
            onChange={(event) => setExecutable(event.target.value)}
            placeholder="/home/…/Skate3Recomp-Linux/skate3"
            spellCheck={false}
            required
          />
        </label>
        {error && (
          <p className="form-error" role="alert">
            {error}
          </p>
        )}
        <div className="add-form-actions">
          <button type="button" className="glass-button" onClick={onCancel}>
            Annuler
          </button>
          <button type="submit" className="launch-button" disabled={saving}>
            Ajouter
          </button>
        </div>
      </form>
    </div>
  );
}
