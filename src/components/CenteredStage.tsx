import type { LaunchStatus } from "../launchStatus";
import { SOURCE_LABELS } from "../sources";
import { titleSizeClass } from "../titleSize";
import type { Game } from "../types";
import { AchievementProgress } from "./AchievementProgress";
import { LaunchButton } from "./LaunchButton";

interface CenteredStageProps {
  game: Game;
  status: LaunchStatus;
  onLaunch: (game: Game) => void;
  onOpenAchievements: () => void;
  onOpenDetail: () => void;
}

export function CenteredStage({
  game,
  status,
  onLaunch,
  onOpenAchievements,
  onOpenDetail,
}: CenteredStageProps) {
  return (
    <section className="centered-stage" aria-live="polite">
      <p className="stage-meta">
        <span className="source-pill">{SOURCE_LABELS[game.source]}</span>
      </p>
      <h1 className={`stage-title ${titleSizeClass(game.title)}`}>{game.title}</h1>
      <div className="stage-actions">
        <LaunchButton status={status} onLaunch={() => onLaunch(game)} />
        {game.achievements && <AchievementProgress summary={game.achievements} onOpen={onOpenAchievements} />}
        <button type="button" className="glass-button" onClick={onOpenDetail}>
          Fiche du jeu
        </button>
      </div>
    </section>
  );
}
