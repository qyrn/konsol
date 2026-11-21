import { useEffect, useState } from "react";
import { errorMessage, getAchievements } from "../api";
import { cssVariable } from "../images";
import type { Achievement, Game } from "../types";
import { CloseButton } from "./CloseButton";
import { CoverImage } from "./CoverImage";
import { Modal } from "./Modal";

interface AchievementsPanelProps {
  game: Game;
  refreshKey: number;
  onClose: () => void;
}

type AchievementFilter = "all" | "unlocked" | "locked";

const FILTER_LABELS: Record<AchievementFilter, string> = {
  all: "Tous",
  unlocked: "Débloqués",
  locked: "À débloquer",
};

const dateFormatter = new Intl.DateTimeFormat("fr-FR", { day: "numeric", month: "short", year: "numeric" });

function matchesFilter(achievement: Achievement, filter: AchievementFilter): boolean {
  if (filter === "all") return true;
  return achievement.unlocked === (filter === "unlocked");
}

export function AchievementsPanel({ game, refreshKey, onClose }: AchievementsPanelProps) {
  const [achievements, setAchievements] = useState<Achievement[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState<AchievementFilter>("all");

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

  const list = achievements ?? [];
  const unlockedCount = list.filter((achievement) => achievement.unlocked).length;
  const total = game.achievements?.total ?? null;
  const progress = total ? Math.round((unlockedCount / total) * 100) : null;
  const counts: Record<AchievementFilter, number> = {
    all: list.length,
    unlocked: unlockedCount,
    locked: list.length - unlockedCount,
  };
  const shown = list.filter((achievement) => matchesFilter(achievement, filter));

  return (
    <Modal labelledBy="achievements-title" className="achievements-panel" onClose={onClose}>
      <header className="achievements-header">
        <span className="achievements-thumb">
          <CoverImage game={game} />
        </span>
        <div className="achievements-heading">
          <p className="overline">Succès</p>
          <h2 id="achievements-title">{game.title}</h2>
        </div>
        <div className="achievements-score">
          <p className="achievements-count">{total ? `${unlockedCount} / ${total}` : unlockedCount}</p>
          <p>{progress !== null ? `${progress} % débloqués` : "débloqués"}</p>
        </div>
        <CloseButton onClose={onClose} />
      </header>
      {progress !== null && (
        <span className="progress-track" aria-hidden="true">
          <span className="progress-fill" style={cssVariable("--progress", `${progress}%`)} />
        </span>
      )}
      <div className="filter-pills" role="group" aria-label="Filtrer les succès">
        {(Object.keys(FILTER_LABELS) as AchievementFilter[]).map((key) => (
          <button
            key={key}
            type="button"
            className="filter-pill"
            aria-pressed={filter === key}
            onClick={() => setFilter(key)}
          >
            {FILTER_LABELS[key]}
            <span>{counts[key]}</span>
          </button>
        ))}
      </div>
      {game.source === "gog" && (
        <p className="panel-note">
          Les succès GOG viennent du cache de Heroic, ils se mettent à jour quand Heroic se synchronise.
        </p>
      )}
      {error && (
        <p className="form-error" role="alert">
          {error}
        </p>
      )}
      {achievements && achievements.length === 0 && <p className="panel-note">Aucun succès pour ce jeu.</p>}
      <ul className="achievement-list">
        {shown.map((achievement) => (
          <li key={achievement.id} className="achievement" data-unlocked={achievement.unlocked}>
            {achievement.iconUrl ? (
              <img className="achievement-icon" src={achievement.iconUrl} alt="" loading="lazy" />
            ) : (
              <span className="achievement-icon achievement-icon-empty">{achievement.unlocked && "✓"}</span>
            )}
            <span className="achievement-text">
              <span className="achievement-name">{achievement.name}</span>
              {achievement.description && (
                <span className="achievement-description">{achievement.description}</span>
              )}
            </span>
            {achievement.unlockedAt !== null ? (
              <time
                className="achievement-date"
                dateTime={new Date(achievement.unlockedAt * 1000).toISOString()}
              >
                {dateFormatter.format(new Date(achievement.unlockedAt * 1000))}
              </time>
            ) : (
              <span className="achievement-date">{achievement.unlocked ? "Débloqué" : "Verrouillé"}</span>
            )}
          </li>
        ))}
      </ul>
    </Modal>
  );
}
