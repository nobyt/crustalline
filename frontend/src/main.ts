import { initViewer, renderMolBlock, setBackgroundColor, setStyleKind, StyleKind } from "./viewer";
import { exportSvg, getSvg, ipcErrorMessage, loadSmiles } from "./ipc";
import { isHeadless, runHeadless } from "./headless-bridge";
import { clearSelection, initEditing } from "./editing";

const DEFAULT_SMILES = "c1ccccc1"; // benzene

function setStatus(message: string) {
  const el = document.getElementById("status");
  if (el) el.textContent = message;
}

async function main() {
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
    // Load through molrs (not a hardcoded MOL block) so Rust-side AppState
    // actually holds a molecule from startup — editing needs that state to
    // exist, not just something rendered client-side.
    const { mol_block } = await loadSmiles(DEFAULT_SMILES);
    renderMolBlock(mol_block);
    initEditing(setStatus);
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
      clearSelection(); // stale selection from the previous molecule no longer applies
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
