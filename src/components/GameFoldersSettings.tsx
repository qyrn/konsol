import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useState } from "react";
import { errorMessage, getSettings, saveGameFolders } from "../api";

interface GameFoldersSettingsProps {
  onChanged: () => void;
}

export function GameFoldersSettings({ onChanged }: GameFoldersSettingsProps) {
  const [folders, setFolders] = useState<string[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getSettings()
      .then((settings) => setFolders(settings.gameFolders))
      .catch((loadError: unknown) => setError(errorMessage(loadError)));
  }, []);

  async function save(next: string[]) {
    setError(null);
    try {
      await saveGameFolders(next);
      setFolders(next);
      onChanged();
    } catch (saveError) {
      setError(errorMessage(saveError));
    }
  }

  async function addFolder() {
    try {
      const selection = await open({ title: "Choisis un dossier de jeux", directory: true, multiple: false });
      if (typeof selection === "string" && folders && !folders.includes(selection)) {
        await save([...folders, selection]);
      }
    } catch (pickError) {
      setError(errorMessage(pickError));
    }
  }

  return (
    <>
      <div className="settings-section-header">
        <p className="settings-lead">
          Konsol fouille ces dossiers pour trouver tout seul les jeux sans launcher, même ceux installés dans
          un préfixe Wine.
        </p>
        <button type="button" className="glass-button" disabled={!folders} onClick={() => void addFolder()}>
          Ajouter un dossier
        </button>
      </div>
      <ul className="settings-panel source-rows">
        {folders?.length === 0 && <li className="source-row-empty">Aucun dossier pour l'instant.</li>}
        {folders?.map((folder) => (
          <li key={folder} className="folder-row">
            <span className="source-row-path monospace">{folder}</span>
            <button
              type="button"
              className="glass-button field-button"
              onClick={() => void save(folders.filter((other) => other !== folder))}
            >
              Retirer
            </button>
          </li>
        ))}
      </ul>
      {error && (
        <p className="form-error" role="alert">
          {error}
        </p>
      )}
    </>
  );
}
