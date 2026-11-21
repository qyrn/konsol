import { type FormEvent, type KeyboardEvent, useEffect, useState } from "react";
import { errorMessage, getSettings, saveSettings } from "../api";
import type { Settings } from "../types";

interface SettingsFormProps {
  onSaved: () => void;
  onCancel: () => void;
}

interface KeyFieldProps {
  id: string;
  label: string;
  help: string;
  saved: boolean;
  value: string;
  cleared: boolean;
  onChange: (value: string) => void;
  onClear: () => void;
}

function KeyField({ id, label, help, saved, value, cleared, onChange, onClear }: KeyFieldProps) {
  return (
    <div className="key-field">
      <label htmlFor={id}>{label}</label>
      <p className="add-form-help">{help}</p>
      <div className="key-field-row">
        <input
          id={id}
          type="password"
          value={value}
          onChange={(event) => onChange(event.target.value)}
          placeholder={saved && !cleared ? "Clé déjà enregistrée" : "Colle ta clé ici"}
          autoComplete="off"
          spellCheck={false}
          maxLength={128}
        />
        {saved && !cleared && (
          <button type="button" className="glass-button" onClick={onClear}>
            Retirer
          </button>
        )}
      </div>
    </div>
  );
}

function keyChange(value: string, cleared: boolean): string | null {
  if (value.trim()) return value.trim();
  return cleared ? "" : null;
}

export function SettingsForm({ onSaved, onCancel }: SettingsFormProps) {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [gridKey, setGridKey] = useState("");
  const [steamKey, setSteamKey] = useState("");
  const [gridCleared, setGridCleared] = useState(false);
  const [steamCleared, setSteamCleared] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    getSettings()
      .then(setSettings)
      .catch((loadError: unknown) => setError(errorMessage(loadError)));
  }, []);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSaving(true);
    setError(null);
    try {
      await saveSettings(keyChange(gridKey, gridCleared), keyChange(steamKey, steamCleared));
      onSaved();
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
        aria-labelledby="settings-title"
        onSubmit={handleSubmit}
        onKeyDown={handleKeyDown}
      >
        <h2 id="settings-title">Réglages</h2>
        <KeyField
          id="steamgriddb-key"
          label="Clé SteamGridDB"
          help="Pour trouver les jaquettes des jeux qui sont pas sur Steam, genre Skate 3. Tu la récupères sur steamgriddb.com, dans Preferences puis API."
          saved={settings?.hasSteamGridDbKey ?? false}
          value={gridKey}
          cleared={gridCleared}
          onChange={setGridKey}
          onClear={() => setGridCleared(true)}
        />
        <KeyField
          id="steam-web-key"
          label="Clé API Steam"
          help="Pour avoir les vrais noms et les icônes des succès des repacks. Elle se crée en deux minutes sur steamcommunity.com/dev/apikey."
          saved={settings?.hasSteamWebApiKey ?? false}
          value={steamKey}
          cleared={steamCleared}
          onChange={setSteamKey}
          onClear={() => setSteamCleared(true)}
        />
        <p className="add-form-help">Les clés restent sur ton PC, dans la config de Konsol.</p>
        {error && (
          <p className="form-error" role="alert">
            {error}
          </p>
        )}
        <div className="add-form-actions">
          <button type="button" className="glass-button" onClick={onCancel}>
            Annuler
          </button>
          <button type="submit" className="launch-button" disabled={saving || !settings}>
            Enregistrer
          </button>
        </div>
      </form>
    </div>
  );
}
