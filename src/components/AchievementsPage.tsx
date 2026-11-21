import { useEffect, useMemo, useState } from "react";
import { achievementOverview, errorMessage } from "../api";
import { DEFAULT_ACCENT, cssVariable } from "../images";
import type { Achievement, Game, GameAchievements } from "../types";
import { CoverImage } from "./CoverImage";

interface AchievementsPageProps {
  games: Game[];
  refreshKey: number;
  onOpenGame: (game: Game) => void;
}

interface RecentUnlock {
  achievement: Achievement;
  game: Game;
}

const RECENT_LIMIT = 12;
const dateFormatter = new Intl.DateTimeFormat("fr-FR", { day: "numeric", month: "short", year: "numeric" });

export function AchievementsPage({ games, refreshKey, onOpenGame }: AchievementsPageProps) {
  const [overview, setOverview] = useState<GameAchievements[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    achievementOverview()
      .then((result) => {
        if (!cancelled) setOverview(result);
      })
      .catch((loadError: unknown) => {
        if (!cancelled) setError(errorMessage(loadError));
      });
    return () => {
      cancelled = true;
    };
  }, [refreshKey]);

  const gamesById = useMemo(() => new Map(games.map((game) => [game.id, game])), [games]);
  const progress = useMemo(
    () =>
      (overview ?? [])
        .flatMap((entry) => {
          const game = gamesById.get(entry.gameId);
          return game ? [{ entry, game }] : [];
        })
        .sort((first, second) => ratio(second.entry) - ratio(first.entry)),
    [overview, gamesById],
  );
  const recent: RecentUnlock[] = useMemo(
    () =>
      progress
        .flatMap(({ entry, game }) => entry.recent.map((achievement) => ({ achievement, game })))
        .sort((first, second) => (second.achievement.unlockedAt ?? 0) - (first.achievement.unlockedAt ?? 0))
        .slice(0, RECENT_LIMIT),
    [progress],
  );
  const unlockedTotal = progress.reduce((total, { entry }) => total + entry.unlocked, 0);
  const completed = progress.filter(
    ({ entry }) => entry.total !== null && entry.unlocked === entry.total,
  ).length;

  return (
    <section className="achievements-page">
      <header className="achievements-page-header">
        <h1>Succès</h1>
        {overview && (
          <dl className="achievement-totals">
            <div>
              <dt>Débloqués</dt>
              <dd>{unlockedTotal}</dd>
            </div>
            <div>
              <dt>Jeux finis à 100 %</dt>
              <dd>{completed}</dd>
            </div>
            <div>
              <dt>Jeux avec des succès</dt>
              <dd>{progress.length}</dd>
            </div>
          </dl>
        )}
      </header>
      {error && (
        <p className="form-error" role="alert">
          {error}
        </p>
      )}
      {recent.length > 0 && (
        <section className="achievements-section">
          <h2>Derniers débloqués</h2>
          <ul className="recent-unlocks">
            {recent.map(({ achievement, game }) => (
              <li key={`${game.id}-${achievement.id}`}>
                <button type="button" className="recent-unlock" onClick={() => onOpenGame(game)}>
                  {achievement.iconUrl ? (
                    <img className="achievement-icon" src={achievement.iconUrl} alt="" loading="lazy" />
                  ) : (
                    <span className="achievement-icon achievement-icon-empty">✓</span>
                  )}
                  <span className="recent-unlock-text">
                    <span className="achievement-name">{achievement.name}</span>
                    <span className="achievement-description">{game.title}</span>
                  </span>
                  {achievement.unlockedAt !== null && (
                    <time className="achievement-date">
                      {dateFormatter.format(new Date(achievement.unlockedAt * 1000))}
                    </time>
                  )}
                </button>
              </li>
            ))}
          </ul>
        </section>
      )}
      <section className="achievements-section">
        <h2>Par jeu</h2>
        <ul className="game-progress-list">
          {progress.map(({ entry, game }) => (
            <li key={game.id}>
              <button
                type="button"
                className="game-progress"
                style={cssVariable("--item-accent", game.accent ?? DEFAULT_ACCENT)}
                onClick={() => onOpenGame(game)}
              >
                <span className="game-progress-thumb">
                  <CoverImage game={game} />
                </span>
                <span className="game-progress-body">
                  <span className="game-progress-title">{game.title}</span>
                  {entry.total !== null && (
                    <span className="progress-track" aria-hidden="true">
                      <span
                        className="progress-fill"
                        style={cssVariable("--progress", `${Math.round(ratio(entry) * 100)}%`)}
                      />
                    </span>
                  )}
                </span>
                <span className="game-progress-count">
                  {entry.total !== null
                    ? `${entry.unlocked} / ${entry.total}`
                    : `${entry.unlocked} débloqués`}
                </span>
              </button>
            </li>
          ))}
        </ul>
      </section>
    </section>
  );
}

function ratio(entry: GameAchievements): number {
  return entry.total ? entry.unlocked / entry.total : 0;
}
