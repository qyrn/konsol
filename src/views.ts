export type View = "home" | "library" | "detail" | "settings";

export type Overlay = "palette" | "add" | "achievements" | "rename";

export const VIEW_VEIL: Record<View, number> = {
  home: 0,
  detail: 0.2,
  library: 0.84,
  settings: 0.9,
};
