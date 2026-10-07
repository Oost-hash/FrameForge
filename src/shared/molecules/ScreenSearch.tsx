import SearchBar from "../SearchBar";

const SEARCH_ROW = "flex items-center gap-3 px-4 py-2.5 border-b border-border shrink-0";

interface ScreenSearchProps {
  value: string;
  onChange: (value: string) => void;
  placeholder: string;
}

/** Full-width search row shared by Inventory, Foundry and Relics. Sits above the filter chips. */
export function ScreenSearch({ value, onChange, placeholder }: ScreenSearchProps) {
  return (
    <div className={SEARCH_ROW}>
      <SearchBar placeholder={placeholder} value={value} onChange={onChange} />
    </div>
  );
}
