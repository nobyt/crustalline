import { initViewer, renderMolBlock, setBackgroundColor, setStyleKind, StyleKind } from "./viewer";
import { exportSvg, getSvg, ipcErrorMessage, loadSmiles } from "./ipc";
import { isHeadless, runHeadless } from "./headless-bridge";

const BENZENE_MOL_BLOCK = `benzene
  crustalline-M1

 12 12  0  0  0  0  0  0  0  0999 V2000
    1.2124    0.7000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
    1.2124   -0.7000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
    0.0000   -1.4000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
   -1.2124   -0.7000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
   -1.2124    0.7000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
    0.0000    1.4000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
    2.1489    1.2444    0.0000 H   0  0  0  0  0  0  0  0  0  0  0  0
    2.1489   -1.2444    0.0000 H   0  0  0  0  0  0  0  0  0  0  0  0
    0.0000   -2.4889    0.0000 H   0  0  0  0  0  0  0  0  0  0  0  0
   -2.1489   -1.2444    0.0000 H   0  0  0  0  0  0  0  0  0  0  0  0
   -2.1489    1.2444    0.0000 H   0  0  0  0  0  0  0  0  0  0  0  0
    0.0000    2.4889    0.0000 H   0  0  0  0  0  0  0  0  0  0  0  0
  1  2  2  0
  2  3  1  0
  3  4  2  0
  4  5  1  0
  5  6  2  0
  6  1  1  0
  1  7  1  0
  2  8  1  0
  3  9  1  0
  4 10  1  0
  5 11  1  0
  6 12  1  0
M  END
`;

function setStatus(message: string) {
  const el = document.getElementById("status");
  if (el) el.textContent = message;
}

function main() {
  if (isHeadless()) {
    // Headless render window (plan §5): no toolbar/mouse handlers needed —
    // just load the requested SMILES, render, and hand a PNG back to Rust.
    runHeadless().catch((err) => {
      console.error(err);
      setStatus(`headless error: ${err instanceof Error ? err.message : String(err)}`);
    });
    return;
  }

  try {
    const canvas = document.createElement("canvas");
    const gl =
      canvas.getContext("webgl2") ||
      canvas.getContext("webgl") ||
      canvas.getContext("experimental-webgl");
    setStatus(gl ? "webgl: ok" : "webgl: UNAVAILABLE");

    initViewer("viewport");
    renderMolBlock(BENZENE_MOL_BLOCK);
  } catch (err) {
    setStatus(`init error: ${err instanceof Error ? err.message : String(err)}`);
    console.error(err);
  }

  const input = document.getElementById("smiles-input") as HTMLInputElement;
  const btn = document.getElementById("load-btn") as HTMLButtonElement;

  const doLoad = async () => {
    const smiles = input.value.trim();
    if (!smiles) return;
    btn.disabled = true;
    setStatus(`loading ${smiles}...`);
    try {
      const { mol_block } = await loadSmiles(smiles);
      renderMolBlock(mol_block);
      setStatus(`loaded ${smiles}`);
    } catch (err) {
      setStatus(`load error: ${ipcErrorMessage(err)}`);
    } finally {
      btn.disabled = false;
    }
  };

  btn.addEventListener("click", doLoad);
  input.addEventListener("keydown", (e) => {
    if (e.key === "Enter") doLoad();
  });

  const styleSelect = document.getElementById("style-select") as HTMLSelectElement;
  styleSelect.addEventListener("change", () => {
    setStyleKind(styleSelect.value as StyleKind);
  });

  const bgColor = document.getElementById("bg-color") as HTMLInputElement;
  bgColor.addEventListener("input", () => {
    setBackgroundColor(bgColor.value);
  });

  const panel2d = document.getElementById("panel-2d") as HTMLDivElement;
  const toggle2dBtn = document.getElementById("toggle-2d-btn") as HTMLButtonElement;
  toggle2dBtn.addEventListener("click", async () => {
    const opening = !panel2d.classList.contains("open");
    if (!opening) {
      panel2d.classList.remove("open");
      return;
    }
    try {
      panel2d.innerHTML = await getSvg();
      panel2d.classList.add("open");
    } catch (err) {
      setStatus(`2D depiction error: ${ipcErrorMessage(err)}`);
    }
  });

  const exportBtn = document.getElementById("export-svg-btn") as HTMLButtonElement;
  exportBtn.addEventListener("click", async () => {
    const path = window.prompt("Export SVG to path:", "./molecule.svg");
    if (!path) return;
    try {
      await exportSvg(path);
      setStatus(`exported ${path}`);
    } catch (err) {
      setStatus(`export error: ${ipcErrorMessage(err)}`);
    }
  });
}

window.addEventListener("error", (e) => setStatus(`window error: ${e.message}`));

main();
