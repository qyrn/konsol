import { type KeyboardEvent, useEffect, useState } from "react";
import { errorMessage, getAchievements } from "../api";
import type { Achievement, Game } from "../types";

interface AchievementsPanelProps {
  game: Game;
  refreshKey: number;
  onClose: () => void;
}

const dateFormatter = new Intl.DateTimeFormat("fr-FR", { day: "numeric", month: "short", year: "numeric" });

export function AchievementsPanel({ game, refreshKey, onClose }: AchievementsPanelProps) {
  const [achievements, setAchievements] = useState<Achievement[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    getAchievements(game.id)
      .then((list) => {
        if (!cancelled) setAchievements(list);
      })
      .catch((loadError: unknown) => {
        if (!cancelled) setError(errorMessage(loadError));
      });
    return () => {
      cancelled = true;
    };
  }, [game.id, refreshKey]);

  function handleKeyDown(event: KeyboardEvent<HTMLElement>) {
    if (event.key === "Escape") onClose();
  }

  const unlockedCount = achievements?.filter((achievement) => achievement.unlocked).length ?? 0;
  const summary = game.achievements?.total
    ? `${unlockedCount} sur ${game.achievements.total} débloqués`
    : `${unlockedCount} débloqués`;

  return (
    <div className="modal-layer" onClick={onClose}>
      <aside
        className="achievements-panel glass"
        role="dialog"
        aria-modal="true"
        aria-labelledby="achievements-title"
        onClick={(event) => event.stopPropagation()}
        onKeyDown={handleKeyDown}
      >
        <header className="achievements-header">
          <div>
            <h2 id="achievements-title">Succès</h2>
            <p>
              {game.title}, {summary}
            </p>
          </div>
          <button type="button" className="glass-button" onClick={onClose} autoFocus>
            Fermer
          </button>
        </header>
        {game.source === "gog" && (
          <p className="achievements-note">
            Les succès GOG viennent du cache de Heroic, ils se mettent à jour quand Heroic se synchronise.
          </p>
        )}
        {error && (
          <p className="form-error" role="alert">
            {error}
          </p>
        )}
        {achievements && achievements.length === 0 && (
          <p className="achievements-note">Aucun succès pour ce jeu.</p>
        )}
        <ul className="achievement-list">
          {achievements?.map((achievement) => (
            <li key={achievement.id} className="achievement" data-unlocked={achievement.unlocked}>
              {achievement.iconUrl ? (
                <img className="achievement-icon" src={achievement.iconUrl} alt="" loading="lazy" />
              ) : (
                <span className="achievement-icon achievement-icon-empty" />
              )}
              <div className="achievement-text">
                <p className="achievement-name">{achievement.name}</p>
                {achievement.description && (
                  <p className="achievement-description">{achievement.description}</p>
                )}
              </div>
              {achievement.unlockedAt !== null && (
                <time
                  className="achievement-date"
                  dateTime={new Date(achievement.unlockedAt * 1000).toISOString()}
                >
                  {dateFormatter.format(new Date(achievement.unlockedAt * 1000))}
                </time>
              )}
            </li>
          ))}
        </ul>
      </aside>
    </div>
  );
}
