import { enableAtomPicking, renderMolBlock, setSelectedAtom } from "./viewer";
import {
  addAtom,
  ipcErrorMessage,
  redoEdit,
  removeAtom,
  undoEdit,
} from "./ipc";

let selectedAtomIdx: number | null = null;
let selectFn: ((idx: number | null) => void) | null = null;

/** Clears any selection left over from a prior molecule (e.g. after a fresh SMILES load, not an edit). */
export function clearSelection() {
  selectFn?.(null);
}

export function initEditing(setStatus: (message: string) => void) {
  const selectionLabel = document.getElementById("selection-label") as HTMLSpanElement;
  const addAtomSelect = document.getElementById("add-atom-select") as HTMLSelectElement;
  const addAtomBtn = document.getElementById("add-atom-btn") as HTMLButtonElement;
  const removeAtomBtn = document.getElementById("remove-atom-btn") as HTMLButtonElement;
  const undoBtn = document.getElementById("undo-btn") as HTMLButtonElement;
  const redoBtn = document.getElementById("redo-btn") as HTMLButtonElement;

  const select = (idx: number | null) => {
    selectedAtomIdx = idx;
    setSelectedAtom(idx);
    addAtomBtn.disabled = idx === null;
    removeAtomBtn.disabled = idx === null;
    selectionLabel.textContent =
      idx === null ? "No atom selected — click an atom in the 3D view" : `Selected atom #${idx}`;
  };
  selectFn = select;

  enableAtomPicking((idx) => {
    select(selectedAtomIdx === idx ? null : idx);
  });

  addAtomBtn.addEventListener("click", async () => {
    if (selectedAtomIdx === null) return;
    try {
      const { mol_block, new_atom_idx } = await addAtom(
        addAtomSelect.value,
        0,
        [selectedAtomIdx, 1.0],
      );
      renderMolBlock(mol_block);
      select(new_atom_idx);
      setStatus(`added ${addAtomSelect.value} (atom #${new_atom_idx})`);
    } catch (err) {
      setStatus(`add atom error: ${ipcErrorMessage(err)}`);
    }
  });

  removeAtomBtn.addEventListener("click", async () => {
    if (selectedAtomIdx === null) return;
    try {
      const { mol_block } = await removeAtom(selectedAtomIdx);
      renderMolBlock(mol_block);
      select(null); // indices may have shifted (docs/molrs-api-contract.md §3.2) — clear rather than guess
      setStatus("removed atom");
    } catch (err) {
      setStatus(`remove atom error: ${ipcErrorMessage(err)}`);
    }
  });

  const doUndo = async () => {
    try {
      const { mol_block } = await undoEdit();
      renderMolBlock(mol_block);
      select(null);
      setStatus("undone");
    } catch (err) {
      setStatus(`undo error: ${ipcErrorMessage(err)}`);
    }
  };

  const doRedo = async () => {
    try {
      const { mol_block } = await redoEdit();
      renderMolBlock(mol_block);
      select(null);
      setStatus("redone");
    } catch (err) {
      setStatus(`redo error: ${ipcErrorMessage(err)}`);
    }
  };

  undoBtn.addEventListener("click", doUndo);
  redoBtn.addEventListener("click", doRedo);

  window.addEventListener("keydown", (e) => {
    const mod = e.ctrlKey || e.metaKey;
    if (!mod || e.key.toLowerCase() !== "z") return;
    e.preventDefault();
    if (e.shiftKey) doRedo();
    else doUndo();
  });
}
