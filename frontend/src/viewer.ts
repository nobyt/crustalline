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
