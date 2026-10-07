import { useEffect, useMemo, useState } from "react";

/**
 * Pages a list. The page goes back to 1 when any of `resetDeps` change (filters, search, category),
 * but not when the items themselves update in the background (e.g. the 10-second inventory scan).
 */
export function usePagedItems<T>(items: readonly T[], pageSize: number, resetDeps: readonly unknown[]) {
  const [requestedPage, setPage] = useState(0);

  useEffect(() => { setPage(0); }, [...resetDeps, pageSize]); // eslint-disable-line react-hooks/exhaustive-deps

  const pageCount = Math.ceil(items.length / pageSize);
  // Clamp so a shrinking result set never leaves the page past the end.
  const page = Math.min(requestedPage, Math.max(0, pageCount - 1));
  const pagedItems = useMemo(
    () => items.slice(page * pageSize, (page + 1) * pageSize),
    [items, page, pageSize],
  );

  return { page, pageCount, pagedItems, setPage };
}
