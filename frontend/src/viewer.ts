declare const $3Dmol: any;

let viewer: any;
let currentStyle: StyleKind = "ballstick";

export type StyleKind = "stick" | "ballstick" | "sphere";

export function initViewer(elementId: string) {
  viewer = $3Dmol.createViewer(document.getElementById(elementId), {
    backgroundColor: "0x1e1e1e",
  });
  return viewer;
}

const STYLE_SPECS: Record<StyleKind, Record<string, unknown>> = {
  stick: { stick: {} },
  ballstick: { stick: {}, sphere: { scale: 0.25 } },
  sphere: { sphere: {} },
};

export function renderMolBlock(molBlock: string) {
  viewer.removeAllModels();
  viewer.addModel(molBlock, "mol");
  viewer.setStyle({}, STYLE_SPECS[currentStyle]);
  viewer.zoomTo();
  viewer.render();
}

export function setStyleKind(kind: StyleKind) {
  currentStyle = kind;
  viewer.setStyle({}, STYLE_SPECS[currentStyle]);
  viewer.render();
}

export function setBackgroundColor(hexColor: string) {
  viewer.setBackgroundColor(parseInt(hexColor.replace("#", ""), 16));
  viewer.render();
}

export function getViewer() {
  return viewer;
}

const HIGHLIGHT_STYLE = { sphere: { scale: 0.4, color: "yellow" } };

/**
 * Registers a click handler over every atom. 3Dmol's atom-picking callback
 * exposes both `.serial` (1-based, as written in the MOL block) and
 * `.index` (0-based position in the model's internal atom array) — since
 * molrs's to_mol_block writes atoms in graph atom_idx order starting at 0,
 * these should both map directly onto our atom_idx, but this hasn't been
 * confirmed against a real running viewer yet (see docs/headless-rendering.md
 * for why) — verify serial-1 and index agree before trusting either alone.
 */
export function enableAtomPicking(onPick: (atomIdx: number) => void) {
  viewer.setClickable({}, true, (atom: { index?: number; serial?: number }) => {
    const idx = typeof atom.index === "number" ? atom.index : (atom.serial ?? 1) - 1;
    onPick(idx);
  });
}

/** Re-applies the base style to every atom, then overlays a highlight on `idx` (or none if null). */
export function setSelectedAtom(idx: number | null) {
  viewer.setStyle({}, STYLE_SPECS[currentStyle]);
  if (idx !== null) {
    viewer.addStyle({ index: idx }, HIGHLIGHT_STYLE);
  }
  viewer.render();
}
