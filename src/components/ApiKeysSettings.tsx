import { type FormEvent, useEffect, useState } from "react";
import { errorMessage, getSettings, saveSettings } from "../api";
import type { Settings } from "../types";

interface ApiKeysSettingsProps {
  onSaved: () => void;
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
  const stored = saved && !cleared;
  return (
    <div className="key-field">
      <label htmlFor={id}>{label}</label>
      <p className="field-help">{help}</p>
      <div className="key-field-row">
        <input
          id={id}
          className="monospace"
          type="password"
          value={value}
          onChange={(event) => onChange(event.target.value)}
          placeholder={stored ? "Clé déjà enregistrée" : "Colle ta clé ici"}
          autoComplete="off"
          spellCheck={false}
          maxLength={128}
        />
        {stored && (
          <button type="button" className="glass-button field-button" onClick={onClear}>
            Retirer
          </button>
        )}
      </div>
      <p className="key-status" data-saved={stored}>
        <span className="status-dot" aria-hidden="true" />
        {stored ? "Clé enregistrée" : "Pas de clé pour l'instant"}
      </p>
    </div>
  );
}

function keyChange(value: string, cleared: boolean): string | null {
  if (value.trim()) return value.trim();
  return cleared ? "" : null;
}

export function ApiKeysSettings({ onSaved }: ApiKeysSettingsProps) {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [gridKey, setGridKey] = useState("");
  const [steamKey, setSteamKey] = useState("");
  const [gridCleared, setGridCleared] = useState(false);
  const [steamCleared, setSteamCleared] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  async function loadSettings() {
    try {
      setSettings(await getSettings());
    } catch (loadError) {
      setError(errorMessage(loadError));
    }
  }

  useEffect(() => {
    void loadSettings();
  }, []);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSaving(true);
    setError(null);
    try {
      await saveSettings(keyChange(gridKey, gridCleared), keyChange(steamKey, steamCleared));
      setGridKey("");
      setSteamKey("");
      setGridCleared(false);
      setSteamCleared(false);
      await loadSettings();
      onSaved();
    } catch (submitError) {
      setError(errorMessage(submitError));
    } finally {
      setSaving(false);
    }
  }

  return (
    <form className="settings-panel settings-form" onSubmit={handleSubmit}>
      <p className="settings-lead">Les clés restent sur ton PC, dans la config de Konsol.</p>
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
      {error && (
        <p className="form-error" role="alert">
          {error}
        </p>
      )}
      <div>
        <button type="submit" className="launch-button compact" disabled={saving || !settings}>
          Enregistrer
        </button>
      </div>
    </form>
  );
}
