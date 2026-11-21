import type { ReactNode } from "react";
import { DEFAULT_ACCENT, cssVariable } from "../images";

interface KonsolRootProps {
  accent?: string;
  children: ReactNode;
}

export function KonsolRoot({ accent = DEFAULT_ACCENT, children }: KonsolRootProps) {
  return (
    <div className="konsol-root" style={cssVariable("--accent", accent)}>
      {children}
    </div>
  );
}
