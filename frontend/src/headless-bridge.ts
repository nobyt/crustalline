import { emit } from "@tauri-apps/api/event";
import { initViewer, renderMolBlock, getViewer } from "./viewer";
import { loadSmiles } from "./ipc";

interface RenderRequest {
  smiles: string;
  width: number;
  height: number;
  out_path: string;
}

declare global {
  interface Window {
    __CRUSTALLINE_HEADLESS__?: boolean;
    __CRUSTALLINE_RENDER_REQUEST__?: RenderRequest;
  }
}

export function isHeadless(): boolean {
  return window.__CRUSTALLINE_HEADLESS__ === true;
}

export async function runHeadless(): Promise<void> {
  const req = window.__CRUSTALLINE_RENDER_REQUEST__;
  if (!req) {
    throw new Error("headless mode active but no __CRUSTALLINE_RENDER_REQUEST__ was injected");
  }

  // An invisible OS window may never receive a real layout pass, leaving
  // percentage-sized containers at 0x0 (and 3Dmol's canvas along with it).
  // Force explicit pixel dimensions on the viewport so canvas sizing doesn't
  // depend on the window actually being composited on screen.
  const viewportEl = document.getElementById("viewport");
  if (viewportEl) {
    viewportEl.style.width = `${req.width}px`;
    viewportEl.style.height = `${req.height}px`;
  }

  initViewer("viewport");
  const { mol_block } = await loadSmiles(req.smiles);
  renderMolBlock(mol_block);

  // 3Dmol's WebGL draw in viewer.render() (called by renderMolBlock) is
  // synchronous, so pngURI() immediately after reads the just-drawn frame.
  const dataUrl: string = getViewer().pngURI();
  await emit("frame-ready", dataUrl);
}
