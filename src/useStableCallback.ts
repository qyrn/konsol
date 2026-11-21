import { useCallback, useLayoutEffect, useRef } from "react";

export function useStableCallback<Arguments extends unknown[]>(
  callback: (...args: Arguments) => void,
): (...args: Arguments) => void {
  const latest = useRef(callback);
  useLayoutEffect(() => {
    latest.current = callback;
  });
  return useCallback((...args: Arguments) => latest.current(...args), []);
}
