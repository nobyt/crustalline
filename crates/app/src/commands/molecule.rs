use std::sync::Mutex;

use crustalline_core::MoleculeState;
use crustalline_ipc_types::{ExportSvgRequest, IpcError, LoadSmilesRequest, MolBlockDto, SvgDto};
use tauri::State;

pub struct AppState(pub Mutex<Option<MoleculeState>>);

#[tauri::command]
pub fn load_smiles(
    state: State<AppState>,
    req: LoadSmilesRequest,
) -> Result<MolBlockDto, IpcError> {
    let molecule = MoleculeState::from_smiles(&req.smiles).map_err(IpcError::from)?;
    let mol_block = molecule.mol_block();
    *state.0.lock().unwrap() = Some(molecule);
    Ok(MolBlockDto { mol_block })
}

#[tauri::command]
pub fn get_mol_block(state: State<AppState>) -> Result<MolBlockDto, IpcError> {
    let guard = state.0.lock().unwrap();
    match guard.as_ref() {
        Some(molecule) => Ok(MolBlockDto {
            mol_block: molecule.mol_block(),
        }),
        None => Err(IpcError::from("no molecule loaded yet".to_string())),
    }
}

#[tauri::command]
pub fn get_svg(state: State<AppState>) -> Result<SvgDto, IpcError> {
    let guard = state.0.lock().unwrap();
    match guard.as_ref() {
        Some(molecule) => Ok(SvgDto {
            svg: molecule.svg().map_err(IpcError::from)?,
        }),
        None => Err(IpcError::from("no molecule loaded yet".to_string())),
    }
}

#[tauri::command]
pub fn export_svg(state: State<AppState>, req: ExportSvgRequest) -> Result<(), IpcError> {
    let guard = state.0.lock().unwrap();
    match guard.as_ref() {
        Some(molecule) => molecule.export_svg(&req.path).map_err(IpcError::from),
        None => Err(IpcError::from("no molecule loaded yet".to_string())),
    }
}
