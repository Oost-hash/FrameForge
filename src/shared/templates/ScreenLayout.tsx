import type { ReactNode } from "react";

const LAYOUT = "flex flex-1 overflow-hidden min-w-0 min-h-0";
const MAIN = "flex flex-1 flex-col overflow-hidden border-r border-border min-w-0 min-h-0";

interface ScreenLayoutProps {
  sidebar: ReactNode;
  children: ReactNode;
}

/** Two-column screen template: sidebar on the left, search + filters + content + footer stacked on the right. */
export function ScreenLayout({ sidebar, children }: ScreenLayoutProps) {
  return (
    <div className={LAYOUT}>
      {sidebar}
      <div className={MAIN}>{children}</div>
    </div>
  );
}
