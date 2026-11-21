import type { Preferences } from "../types";

interface GeneralSettingsProps {
  preferences: Preferences;
  onChange: (preferences: Preferences) => void;
}

const TOGGLES: { key: keyof Preferences; label: string; description: string }[] = [
  {
    key: "launchAtLogin",
    label: "Lancer Konsol au démarrage du PC",
    description: "Konsol s'ouvre tout seul quand tu te connectes.",
  },
  {
    key: "minimizeDuringSession",
    label: "Réduire Konsol pendant une session",
    description: "La fenêtre revient quand le jeu se ferme.",
  },
  {
    key: "keepRunningInBackground",
    label: "Garder Konsol en fond",
    description:
      "Quand tu fermes la fenêtre, Konsol reste dans la barre système pour compter ton temps de jeu et repérer les nouveaux jeux.",
  },
  {
    key: "accentFollowsGame",
    label: "Couleur d'accent suivant le jeu",
    description: "Le halo et le bouton Lancer prennent la couleur du jeu sélectionné.",
  },
  {
    key: "reduceMotion",
    label: "Réduire les animations",
    description: "Coupe les glissements et les fondus.",
  },
];

export function GeneralSettings({ preferences, onChange }: GeneralSettingsProps) {
  return (
    <ul className="settings-panel toggle-rows">
      {TOGGLES.map((toggle) => (
        <li key={toggle.key} className="toggle-row">
          <span id={`toggle-${toggle.key}`}>
            <span className="toggle-label">{toggle.label}</span>
            <span className="toggle-description">{toggle.description}</span>
          </span>
          <button
            type="button"
            className="switch"
            role="switch"
            aria-checked={preferences[toggle.key]}
            aria-labelledby={`toggle-${toggle.key}`}
            onClick={() => onChange({ ...preferences, [toggle.key]: !preferences[toggle.key] })}
          >
            <span className="switch-knob" />
          </button>
        </li>
      ))}
    </ul>
  );
}
