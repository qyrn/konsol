import { type FormEvent, useState } from "react";
import { errorMessage, renameGame } from "../api";
import type { Game } from "../types";
import { CloseButton } from "./CloseButton";
import { Modal } from "./Modal";

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

  return (
    <Modal labelledBy="rename-title" className="form-panel form-panel-narrow" onClose={onCancel}>
      <form onSubmit={handleSubmit}>
        <header className="form-panel-header">
          <div>
            <h2 id="rename-title">Renommer</h2>
            <p>Konsol utilise ce nom pour chercher la jaquette, du coup mets le vrai nom du jeu.</p>
          </div>
          <CloseButton onClose={onCancel} />
        </header>
        <div className="form-fields">
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
            Renommer
          </button>
        </div>
      </form>
    </Modal>
  );
}
