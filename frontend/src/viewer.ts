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
 * Registers a click handler over every atom. Verified directly against
 * 3Dmol's unvendored V2000 parser source (node_modules/3dmol/build/3Dmol.js,
 * parseV2000: `atom.index = curFrame.length` / `atom.serial = i` are set
 * from the same 0-based loop counter over the MOL block's atom lines, with
 * hydrogens kept by default) — both `.index` and `.serial` equal the atom's
 * 0-based position in the MOL block, which is exactly molrs's atom_idx
 * (to_mol_block writes g.atoms in order starting at 0). No off-by-one
 * adjustment needed on either field; `.index` is used as the primary source
 * since it's 3Dmol's own canonical addressing (same field `setStyle({index})`
 * selection uses elsewhere in this file).
 */
export function enableAtomPicking(onPick: (atomIdx: number) => void) {
  viewer.setClickable({}, true, (atom: { index?: number; serial?: number }) => {
    const idx = typeof atom.index === "number" ? atom.index : (atom.serial ?? 0);
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
