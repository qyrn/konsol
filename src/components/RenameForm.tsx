import { type FormEvent, type KeyboardEvent, useState } from "react";
import { errorMessage, renameGame } from "../api";
import type { Game } from "../types";

interface RenameFormProps {
  game: Game;
  onRenamed: (title: string) => void;
  onCancel: () => void;
}

export function RenameForm({ game, onRenamed, onCancel }: RenameFormProps) {
  const [title, setTitle] = useState(game.title);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSaving(true);
    setError(null);
    try {
      await renameGame(game.id, title);
      onRenamed(title.trim());
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
        aria-labelledby="rename-title"
        onSubmit={handleSubmit}
        onKeyDown={handleKeyDown}
      >
        <h2 id="rename-title">Renommer</h2>
        <p className="add-form-help">
          Konsol utilise ce nom pour chercher la jaquette, du coup mets le vrai nom du jeu.
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
            Renommer
          </button>
        </div>
      </form>
    </div>
  );
}
