import { useCallback, useEffect, useState } from "react";
import { errorMessage, getCollections, saveCollections } from "./api";
import type { Collection } from "./types";

export function useCollections(onError: (message: string) => void) {
  const [collections, setCollections] = useState<Collection[]>([]);

  useEffect(() => {
    getCollections()
      .then(setCollections)
      .catch((error: unknown) => onError(errorMessage(error)));
  }, [onError]);

  const save = useCallback(
    async (next: Collection[]) => {
      const previous = collections;
      setCollections(next);
      try {
        await saveCollections(next);
      } catch (error) {
        setCollections(previous);
        onError(errorMessage(error));
      }
    },
    [collections, onError],
  );

  const toggleGame = useCallback(
    (name: string, gameId: string) =>
      save(
        collections.map((collection) =>
          collection.name !== name
            ? collection
            : {
                ...collection,
                gameIds: collection.gameIds.includes(gameId)
                  ? collection.gameIds.filter((id) => id !== gameId)
                  : [...collection.gameIds, gameId],
              },
        ),
      ),
    [collections, save],
  );

  const create = useCallback(
    (name: string, gameId: string) => save([...collections, { name, gameIds: [gameId] }]),
    [collections, save],
  );

  const remove = useCallback(
    (name: string) => save(collections.filter((collection) => collection.name !== name)),
    [collections, save],
  );

  return { collections, toggleGame, create, remove };
}
