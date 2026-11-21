import { memo, useEffect, useState } from "react";
import { discoverGames, errorMessage, openStorePage } from "../api";
import type { Discovery } from "../types";

interface DiscoverRowProps {
  onError: (message: string) => void;
}

export const DiscoverRow = memo(function DiscoverRow({ onError }: DiscoverRowProps) {
  const [items, setItems] = useState<Discovery[]>([]);

  useEffect(() => {
    let cancelled = false;
    discoverGames()
      .then((found) => {
        if (!cancelled) setItems(found);
      })
      .catch(() => {
        if (!cancelled) setItems([]);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  if (items.length === 0) return null;

  return (
    <section className="library-band" aria-label="À découvrir">
      <h2>
        À découvrir
        <span>des jeux que t'as pas encore, bien notés et qui tournent sous Linux</span>
      </h2>
      <div className="wide-row wide-row-scroll">
        {items.map((item) => (
          <button
            key={item.appId}
            type="button"
            className="wide-card"
            data-installed="true"
            data-with-logo={item.capsule !== null}
            onClick={() =>
              void openStorePage(item.appId).catch((error: unknown) => onError(errorMessage(error)))
            }
          >
            <span className="wide-card-picture">
              {item.capsule && (
                <img src={item.capsule} alt="" loading="lazy" decoding="async" draggable={false} />
              )}
              {!item.capsule && <span className="wide-card-title">{item.name}</span>}
            </span>
            <span className="wide-card-caption">
              <span className="wide-card-name">{item.name}</span>
              Dans le même esprit que {item.because}
              {item.reviewLabel && <span className="wide-card-review">{item.reviewLabel}</span>}
            </span>
          </button>
        ))}
      </div>
    </section>
  );
});
