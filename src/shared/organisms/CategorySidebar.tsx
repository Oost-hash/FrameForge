import type { ReactNode } from "react";
import { CategoryButton, CAT_COUNT, CAT_TOTAL } from "../ui/CategoryButton";

const SIDEBAR = "flex w-40 shrink-0 flex-col overflow-y-auto border-r border-border min-h-0 py-3";
const SIDEBAR_CAT = "text-12! px-2.5! min-w-0";
const SIDEBAR_CAT_LABEL = "overflow-hidden text-ellipsis whitespace-nowrap min-w-0";
const SIDEBAR_FOOTER_DIVIDER = "border-t border-border mx-0 my-2.5";

interface CategorySidebarProps<T extends string> {
  categories: readonly T[];
  active: T;
  onSelect: (category: T) => void;
  /** Display text per category; defaults to the category value itself. */
  getLabel?: (category: T) => string;
  /** Plain total per category, shown as a muted count when non-zero. */
  counts?: Partial<Record<T, number>>;
  /** Custom count content; takes precedence over `counts`. */
  renderCount?: (category: T) => ReactNode;
  /** Extra content below the category list, separated by a divider. */
  footer?: ReactNode;
}

/** Left-hand category list shared by Inventory, Foundry and Relics. */
export function CategorySidebar<T extends string>({
  categories, active, onSelect, getLabel, counts, renderCount, footer,
}: CategorySidebarProps<T>) {
  return (
    <div className={SIDEBAR}>
      {categories.map(category => {
        const count = renderCount
          ? renderCount(category)
          : counts?.[category]
            ? <span className={CAT_COUNT}><span className={CAT_TOTAL}>{counts[category]}</span></span>
            : null;
        return (
          <CategoryButton key={category} active={active === category} label={getLabel ? getLabel(category) : category}
            className={SIDEBAR_CAT} labelClassName={SIDEBAR_CAT_LABEL}
            onClick={() => onSelect(category)}>
            {count}
          </CategoryButton>
        );
      })}
      {footer && <>
        <div className={SIDEBAR_FOOTER_DIVIDER} />
        {footer}
      </>}
    </div>
  );
}
