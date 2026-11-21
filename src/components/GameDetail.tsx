import { lastPlayedLabel, playtimeLabel, sizeLabel } from "../format";
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

interface Stat {
  label: string;
  value: string;
  small?: boolean;
}

function detailStats(game: Game): Stat[] {
  const { details, achievements } = game;
  const stats: (Stat | null)[] = [
    { label: "Temps de jeu", value: playtimeLabel(details.playtimeMinutes) ?? "0 h" },
    achievements && {
      label: achievements.total ? "Succès" : "Succès débloqués",
      value: achievementCountLabel(achievements),
    },
    sizeLabel(details.sizeBytes) !== null
      ? { label: "Sur le disque", value: sizeLabel(details.sizeBytes) ?? "" }
      : null,
    {
      label: "Dernière session",
      value: lastPlayedLabel(details.lastPlayed, Date.now()) ?? "Jamais lancé",
      small: true,
    },
  ];
  return stats.filter((stat): stat is Stat => stat !== null);
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
  const { details } = game;
  const meta = [details.developer, details.year?.toString(), details.genre].filter(Boolean);

  return (
    <section className="detail">
      <div className="detail-cover">
        <CoverImage game={game} />
      </div>
      <div className="detail-body">
        <button type="button" className="back-link" onClick={onBack}>
          ← Bibliothèque
        </button>
        <p className="detail-meta">{meta.length > 0 ? meta.join(" · ") : SOURCE_LABELS[game.source]}</p>
        <h1 className={`detail-title ${titleSizeClass(game.title)}`}>{game.title}</h1>
        <div className="stage-actions detail-actions">
          <LaunchButton status={status} installed={game.installed} onLaunch={() => onLaunch(game)} />
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
        <dl className="detail-stats">
          {detailStats(game).map((stat) => (
            <div key={stat.label} data-small={stat.small ?? false}>
              <dt>{stat.label}</dt>
              <dd>{stat.value}</dd>
            </div>
          ))}
        </dl>
        <div className="detail-footer">
          <div className="detail-location">
            <p>Emplacement · {SOURCE_LABELS[game.source]}</p>
            {details.location && <p className="monospace">{details.location}</p>}
          </div>
          {game.achievements && (
            <button type="button" className="glass-button small-pill" onClick={onOpenAchievements}>
              Voir les succès
            </button>
          )}
        </div>
      </div>
    </section>
  );
}
