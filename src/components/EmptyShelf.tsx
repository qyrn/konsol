import type { CSSProperties } from "react";
import { carouselSlot } from "../carouselLayout";

interface EmptyShelfProps {
  scanning: boolean;
  onRescan: () => void;
  onAddGame: () => void;
}

const GHOST_SLOTS = [-2, -1, 0, 1, 2];

export function EmptyShelf({ scanning, onRescan, onAddGame }: EmptyShelfProps) {
  return (
    <section className="empty-shelf">
      <div className="empty-slots" aria-hidden="true">
        {GHOST_SLOTS.map((slot) => (
          <span
            key={slot}
            className="empty-slot"
            data-rank={Math.abs(slot)}
            style={
              { "--offset": carouselSlot(slot).offset, "--scale": carouselSlot(slot).scale } as CSSProperties
            }
          >
            {slot === 0 && "+"}
          </span>
        ))}
      </div>
      <div className="empty-text">
        <h1>Ton étagère est vide</h1>
        <p>
          Konsol trouve tout seul tes jeux Steam, ceux de Heroic et ceux qui ont un raccourci dans le menu des
          applis. Pour le reste, ajoute-les à la main !
        </p>
        <div className="stage-actions">
          <button type="button" className="launch-button" disabled={scanning} onClick={onRescan}>
            {scanning ? "Ça cherche…" : "Analyser ce PC"}
          </button>
          <button type="button" className="glass-button" onClick={onAddGame}>
            Ajouter un jeu
          </button>
        </div>
      </div>
    </section>
  );
}
