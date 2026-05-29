/** Spacing rhythm from ux.md (proximity / chunking). */
export const UX = {
  withinGroup: "gap-3",
  betweenGroups: "gap-8",
  section: "space-y-3",
  page: "flex flex-col gap-8",
  /** Readable line length for forms and copy-heavy pages. */
  prose: "max-w-2xl",
  /** Full-width tables and dashboards. */
  wide: "w-full max-w-none",
} as const;
