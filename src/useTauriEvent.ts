import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";

export function useTauriEvent(eventName: string, handler: () => void) {
  const handlerRef = useRef(handler);
  handlerRef.current = handler;

  useEffect(() => {
    const unlisten = listen(eventName, () => handlerRef.current());
    return () => {
      void unlisten.then((stop) => stop());
    };
  }, [eventName]);
}
