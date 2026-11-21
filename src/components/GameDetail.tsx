import type { LaunchStatus } from "../launchStatus";
import { SOURCE_LABELS } from "../sources";
import { titleSizeClass } from "../titleSize";
import type { Game } from "../types";
import { achievementCountLabel } from "./AchievementProgress";
import { CoverImage } from "./CoverImage";
import { LaunchButton } from "./LaunchButton";

interface GameDetailProps {
  game: Game;
  status: LaunchStatus;
  onBack: () => void;
  onLaunch: (game: Game) => void;
  onRename: () => void;
  onToggleHidden: (game: Game) => void;
  onRemove: (game: Game) => void;
  onOpenAchievements: () => void;
}

export function GameDetail({
  game,
  status,
  onBack,
  onLaunch,
  onRename,
  onToggleHidden,
  onRemove,
  onOpenAchievements,
}: GameDetailProps) {
  const achievements = game.achievements;
  const progress =
    achievements?.total && achievements.total > 0
      ? Math.round((achievements.unlocked / achievements.total) * 100)
      : null;

  return (
    <section className="detail">
      <div className="detail-cover">
        <CoverImage game={game} />
      </div>
      <div className="detail-body">
        <button type="button" className="back-link" onClick={onBack}>
          ← Bibliothèque
        </button>
        <p className="detail-meta">{SOURCE_LABELS[game.source]}</p>
        <h1 className={`detail-title ${titleSizeClass(game.title)}`}>{game.title}</h1>
        <div className="stage-actions detail-actions">
          <LaunchButton status={status} onLaunch={() => onLaunch(game)} />
          <button type="button" className="glass-button" onClick={onRename}>
            Renommer
          </button>
          {game.source === "manual" ? (
            <button type="button" className="glass-button" onClick={() => onRemove(game)}>
              Retirer
            </button>
          ) : (
            <button type="button" className="glass-button" onClick={() => onToggleHidden(game)}>
              {game.hidden ? "Réafficher" : "Masquer"}
            </button>
          )}
        </div>
        {achievements && (
          <>
            <dl className="detail-stats">
              <div>
                <dt>{achievements.total ? "Succès" : "Succès débloqués"}</dt>
                <dd>{achievementCountLabel(achievements)}</dd>
              </div>
              {progress !== null && (
                <div>
                  <dt>Progression</dt>
                  <dd>{progress} %</dd>
                </div>
              )}
            </dl>
            <div className="detail-footer">
              <button type="button" className="glass-button small-pill" onClick={onOpenAchievements}>
                Voir les succès
              </button>
            </div>
          </>
        )}
      </div>
    </section>
  );
}
