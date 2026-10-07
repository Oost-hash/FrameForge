import { SecondaryButton } from "../ui/ActionButton";

const PAGINATION = "flex shrink-0 items-center justify-center gap-2.5 border-t border-border px-2 py-2.5";
const PAGE_LABEL = "min-w-20 text-center text-12 text-muted";

interface PaginationProps {
  page: number;
  pageCount: number;
  onPageChange: (page: number) => void;
}

/** Prev / Next footer for Inventory, Foundry and Relics. Renders nothing when everything fits on one page. */
export function Pagination({ page, pageCount, onPageChange }: PaginationProps) {
  if (pageCount <= 1) return null;
  return (
    <div className={PAGINATION}>
      <SecondaryButton disabled={page === 0} onClick={() => onPageChange(page - 1)}>← Prev</SecondaryButton>
      <span className={PAGE_LABEL}>Page {page + 1} of {pageCount}</span>
      <SecondaryButton disabled={page >= pageCount - 1} onClick={() => onPageChange(page + 1)}>Next →</SecondaryButton>
    </div>
  );
}
