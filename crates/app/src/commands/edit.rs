use crustalline_ipc_types::{
    AddAtomDto, AddAtomRequest, BondRequest, IpcError, MolBlockDto, RemoveAtomRequest,
    RemoveBondRequest, SetFormalChargeRequest,
};
use tauri::State;

use crate::commands::AppState;

fn with_molecule<T>(
    state: &State<AppState>,
    f: impl FnOnce(&mut crustalline_core::MoleculeState) -> Result<T, crustalline_core::CoreError>,
) -> Result<T, IpcError> {
    let mut guard = state.0.lock().unwrap();
    match guard.as_mut() {
        Some(molecule) => f(molecule).map_err(IpcError::from),
        None => Err(IpcError::from("no molecule loaded yet".to_string())),
    }
}

#[tauri::command]
pub fn add_atom(state: State<AppState>, req: AddAtomRequest) -> Result<AddAtomDto, IpcError> {
    with_molecule(&state, |molecule| {
        let new_atom_idx = molecule.add_atom(&req.symbol, req.formal_charge, req.bonded_to)?;
        Ok(AddAtomDto {
            mol_block: molecule.mol_block(),
            new_atom_idx,
        })
    })
}

#[tauri::command]
pub fn remove_atom(state: State<AppState>, req: RemoveAtomRequest) -> Result<MolBlockDto, IpcError> {
    with_molecule(&state, |molecule| {
        molecule.remove_atom(req.atom_idx)?;
        Ok(MolBlockDto {
            mol_block: molecule.mol_block(),
        })
    })
}

#[tauri::command]
pub fn add_bond(state: State<AppState>, req: BondRequest) -> Result<MolBlockDto, IpcError> {
    with_molecule(&state, |molecule| {
        molecule.add_bond(req.a, req.b, req.order)?;
        Ok(MolBlockDto {
            mol_block: molecule.mol_block(),
        })
    })
}

#[tauri::command]
pub fn remove_bond(state: State<AppState>, req: RemoveBondRequest) -> Result<MolBlockDto, IpcError> {
    with_molecule(&state, |molecule| {
        molecule.remove_bond(req.a, req.b)?;
        Ok(MolBlockDto {
            mol_block: molecule.mol_block(),
        })
    })
}

#[tauri::command]
pub fn set_bond_order(state: State<AppState>, req: BondRequest) -> Result<MolBlockDto, IpcError> {
    with_molecule(&state, |molecule| {
        molecule.set_bond_order(req.a, req.b, req.order)?;
        Ok(MolBlockDto {
            mol_block: molecule.mol_block(),
        })
    })
}

#[tauri::command]
pub fn set_formal_charge(
    state: State<AppState>,
    req: SetFormalChargeRequest,
) -> Result<MolBlockDto, IpcError> {
    with_molecule(&state, |molecule| {
        molecule.set_formal_charge(req.atom_idx, req.charge)?;
        Ok(MolBlockDto {
            mol_block: molecule.mol_block(),
        })
    })
}

#[tauri::command]
pub fn undo(state: State<AppState>) -> Result<MolBlockDto, IpcError> {
    with_molecule(&state, |molecule| {
        molecule.undo()?;
        Ok(MolBlockDto {
            mol_block: molecule.mol_block(),
        })
    })
}

#[tauri::command]
pub fn redo(state: State<AppState>) -> Result<MolBlockDto, IpcError> {
    with_molecule(&state, |molecule| {
        molecule.redo()?;
        Ok(MolBlockDto {
            mol_block: molecule.mol_block(),
        })
    })
}
