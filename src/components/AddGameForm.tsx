import { type FormEvent, useState } from "react";
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

  return (
    <form className="add-form" onSubmit={handleSubmit}>
      <h2>Ajouter un jeu</h2>
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
        <button type="button" className="quiet-button" onClick={onCancel}>
          Annuler
        </button>
        <button type="submit" className="launch-button" disabled={saving}>
          Ajouter
        </button>
      </div>
    </form>
  );
}
