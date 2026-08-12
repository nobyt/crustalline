import { invoke } from "@tauri-apps/api/core";

export interface MolBlockDto {
  mol_block: string;
}

export interface IpcError {
  message: string;
}

export async function loadSmiles(smiles: string): Promise<MolBlockDto> {
  return invoke<MolBlockDto>("load_smiles", { req: { smiles } });
}

export async function getSvg(): Promise<string> {
  const { svg } = await invoke<{ svg: string }>("get_svg");
  return svg;
}

export async function exportSvg(path: string): Promise<void> {
  await invoke("export_svg", { req: { path } });
}

export interface AddAtomDto {
  mol_block: string;
  new_atom_idx: number;
}

export async function addAtom(
  symbol: string,
  formalCharge: number,
  bondedTo: [number, number] | null,
): Promise<AddAtomDto> {
  return invoke<AddAtomDto>("add_atom", {
    req: { symbol, formal_charge: formalCharge, bonded_to: bondedTo },
  });
}

export async function removeAtom(atomIdx: number): Promise<MolBlockDto> {
  return invoke<MolBlockDto>("remove_atom", { req: { atom_idx: atomIdx } });
}

export async function addBond(a: number, b: number, order: number): Promise<MolBlockDto> {
  return invoke<MolBlockDto>("add_bond", { req: { a, b, order } });
}

export async function removeBond(a: number, b: number): Promise<MolBlockDto> {
  return invoke<MolBlockDto>("remove_bond", { req: { a, b } });
}

export async function setBondOrder(a: number, b: number, order: number): Promise<MolBlockDto> {
  return invoke<MolBlockDto>("set_bond_order", { req: { a, b, order } });
}

export async function setFormalCharge(atomIdx: number, charge: number): Promise<MolBlockDto> {
  return invoke<MolBlockDto>("set_formal_charge", { req: { atom_idx: atomIdx, charge } });
}

export async function undoEdit(): Promise<MolBlockDto> {
  return invoke<MolBlockDto>("undo");
}

export async function redoEdit(): Promise<MolBlockDto> {
  return invoke<MolBlockDto>("redo");
}

export function ipcErrorMessage(err: unknown): string {
  if (err && typeof err === "object" && "message" in err) {
    return String((err as IpcError).message);
  }
  return String(err);
}
