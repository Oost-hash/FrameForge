import type { Dispatch, SetStateAction } from "react";
import { HelpTip } from "../shared/HelpTip";
import type { InventoryFilters } from "../types/filters";
import SearchBar from "../shared/SearchBar";
import { ViewToggle } from "../shared/ViewToggle";
import type { ViewMode } from "../types/ui";
import type { FilterPresetModule, FilterPresetSettings } from "../types/filterPresets";
import FilterPresets from "../shared/FilterPresets";

const TOOLBAR = "flex items-center gap-[12px] px-[16px] py-[10px] border-b border-border shrink-0";
const ITEM_COUNT_LABEL = "text-muted text-[11px] whitespace-nowrap";
const IMAGE_TOGGLE =
  "inventory-image-toggle flex items-center gap-[5px] shrink-0 text-muted cursor-pointer text-[11px] whitespace-nowrap hover:text-foreground";
const CTRL_WRAP =
  "flex items-center shrink-0 h-[25px] overflow-hidden border border-border rounded-[5px] text-muted text-[10px] tabular-nums";
const CTRL_SPAN = "min-w-[42px] text-center";
const CTRL_BTN =
  "self-stretch w-[24px] border-0 bg-[rgba(255,255,255,.03)] text-muted cursor-pointer text-[15px] leading-none hover:enabled:bg-[rgba(255,255,255,.08)] hover:enabled:text-foreground disabled:opacity-35 disabled:cursor-default";

interface InventoryToolbarProps {
  filters: InventoryFilters;
  onFiltersChange: Dispatch<SetStateAction<InventoryFilters>>;
  onToggleRecent: () => void;
  availableRanks: number[];
  showRankFilters: boolean;
  itemCount: number;
  view: ViewMode;
  onViewChange: (view: ViewMode) => void;
  cardColumns: number;
  onCardColumnsChange: (columns: number) => void;
  listTextScale: number;
  onListTextScaleChange: (scale: number) => void;
  filterPresets: FilterPresetSettings;
  onFilterPresetsChange: Dispatch<SetStateAction<FilterPresetSettings>>;
  onOpenSettings: (module: FilterPresetModule) => void;
}

export default function InventoryToolbar({
  filters, onFiltersChange, onToggleRecent, availableRanks, showRankFilters, itemCount, view, onViewChange, cardColumns, onCardColumnsChange, listTextScale, onListTextScaleChange, filterPresets, onFilterPresetsChange, onOpenSettings,
}: InventoryToolbarProps) {
  const { search, filterOwned, filterRecent, filterPrime, filterVaulted, filterUnvaulted, filterTradeable, filterDucats, filterRank, sortMode } = filters;
  const isCardView = view === "cards" || view === "text-cards";
  const isListView = view === "list" || view === "list-compact";
  const imagesVisible = view === "cards" || view === "list";
  return (
    <>
      <div className={TOOLBAR}>
        <SearchBar
          placeholder="Search items (comma-separated)…"
          value={search}
          onChange={search => onFiltersChange(previous => ({ ...previous, search }))}
        />
      </div>
      <div className="filter-bar">
        <button className={`fchip ${filterOwned ? "fchip-on" : ""}`} onClick={() => onFiltersChange(previous => ({ ...previous, filterOwned: !previous.filterOwned }))}>Owned</button>
        <button className={`fchip ${filterRecent ? "fchip-on" : ""}`} onClick={onToggleRecent}>Changed recently</button>
        <button className={`fchip ${filterPrime ? "fchip-on" : ""}`} onClick={() => onFiltersChange(previous => ({ ...previous, filterPrime: !previous.filterPrime }))}>Prime</button>
        <button className={`fchip ${filterVaulted ? "fchip-on" : ""}`} onClick={() => onFiltersChange(previous => ({ ...previous, filterVaulted: !previous.filterVaulted }))}>🔒 Vaulted</button>
        <button className={`fchip ${filterUnvaulted ? "fchip-on" : ""}`} onClick={() => onFiltersChange(previous => ({ ...previous, filterUnvaulted: !previous.filterUnvaulted }))}>🔓 Unvaulted</button>
        <button className={`fchip ${filterTradeable ? "fchip-on" : ""}`} aria-pressed={filterTradeable} onClick={() => onFiltersChange(previous => ({ ...previous, filterTradeable: !previous.filterTradeable }))}>Tradeable</button>
        <button className={`fchip ${filterDucats ? "fchip-on" : ""}`} aria-pressed={filterDucats} onClick={() => onFiltersChange(previous => ({ ...previous, filterDucats: !previous.filterDucats }))}>Ducats</button>
        {showRankFilters && <>
          <span className="fbar-sep" />
          <span className="fbar-label">Rank:</span>
          <button className={`fchip ${filterRank === "unranked" ? "fchip-on" : ""}`} onClick={() => onFiltersChange(previous => ({ ...previous, filterRank: previous.filterRank === "unranked" ? null : "unranked" }))}>Unranked</button>
          {availableRanks.map(rank => (
            <button key={rank} className={`fchip ${filterRank === rank ? "fchip-on" : ""}`} onClick={() => onFiltersChange(previous => ({ ...previous, filterRank: previous.filterRank === rank ? null : rank }))}>R{rank}</button>
          ))}
        </>}
        <span className="fbar-sep" />
        <FilterPresets module="inventory" {...{ filters, onFiltersChange, filterPresets, onFilterPresetsChange, onOpenSettings }} />
        <span className="fbar-sep" />
        <span className="fbar-label">Sort:</span>
        <button className={`fchip ${sortMode === "qty-desc" ? "fchip-on" : ""}`} onClick={() => onFiltersChange(previous => ({ ...previous, sortMode: "qty-desc" }))}>Qty ↓</button>
        <button className={`fchip ${sortMode === "qty-asc" ? "fchip-on" : ""}`} onClick={() => onFiltersChange(previous => ({ ...previous, sortMode: "qty-asc" }))}>Qty ↑</button>
        <button className={`fchip ${sortMode === "name-asc" ? "fchip-on" : ""}`} onClick={() => onFiltersChange(previous => ({ ...previous, sortMode: "name-asc" }))}>A-Z</button>
        <button className={`fchip ${sortMode === "name-desc" ? "fchip-on" : ""}`} onClick={() => onFiltersChange(previous => ({ ...previous, sortMode: "name-desc" }))}>Z-A</button>
        <span className={`${ITEM_COUNT_LABEL} ml-auto`}>{itemCount} item{itemCount !== 1 ? "s" : ""}{itemCount === 1000 ? " (capped)" : ""}</span>
        <ViewToggle
          view={view === "text-cards" ? "cards" : view === "list-compact" ? "list" : view}
          onChange={onViewChange}
          modes={["cards", "icons", "list"]}
        />
        {(isCardView || isListView) && (
          <>
            <label className={IMAGE_TOGGLE}>
              <input type="checkbox" checked={imagesVisible}
                onChange={event => onViewChange(isCardView
                  ? (event.target.checked ? "cards" : "text-cards")
                  : (event.target.checked ? "list" : "list-compact"))} />
              Images
            </label>
            {isCardView && (
              <div className={CTRL_WRAP} aria-label="Maximum card columns">
                <button title="Fewer columns" aria-label="Fewer columns" className={CTRL_BTN} disabled={cardColumns <= 5}
                  onClick={() => onCardColumnsChange(cardColumns - 1)}>−</button>
                <span className={CTRL_SPAN}>{cardColumns} cols</span>
                <button title="More columns" aria-label="More columns" className={CTRL_BTN} disabled={cardColumns >= 24}
                  onClick={() => onCardColumnsChange(cardColumns + 1)}>+</button>
              </div>
            )}
            {isListView && (
              <div className={CTRL_WRAP} aria-label="List text size">
                <button title="Smaller text" aria-label="Smaller text" className={CTRL_BTN} disabled={listTextScale <= 80}
                  onClick={() => onListTextScaleChange(listTextScale - 10)}>−</button>
                <span className={CTRL_SPAN}>{listTextScale}%</span>
                <button title="Larger text" aria-label="Larger text" className={CTRL_BTN} disabled={listTextScale >= 150}
                  onClick={() => onListTextScaleChange(listTextScale + 10)}>+</button>
              </div>
            )}
          </>
        )}
        <HelpTip items={[
          { icon: "★", label: "★  Mastered", desc: "Shown above image — item levelled to rank 30" },
          { icon: "R5", label: "R{n}  Rank", desc: "Shown above image — current rank, not yet mastered" },
          { icon: "⚒", label: "⚒  Building", desc: "Shown on image — currently crafting in Foundry" },
          { swatch: "rgba(63,185,80,.5)", label: "Green border", desc: "Item recently gained" },
          { swatch: "rgba(248,81,73,.5)", label: "Red border", desc: "Item recently lost or consumed" },
        ]} />
      </div>
    </>
  );
}
