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

export function ipcErrorMessage(err: unknown): string {
  if (err && typeof err === "object" && "message" in err) {
    return String((err as IpcError).message);
  }
  return String(err);
}
