use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MolBlockDto {
    pub mol_block: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadSmilesRequest {
    pub smiles: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderRequest {
    pub smiles: String,
    pub width: u32,
    pub height: u32,
    pub out_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SvgDto {
    pub svg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSvgRequest {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAtomRequest {
    pub symbol: String,
    pub formal_charge: i8,
    /// (existing atom idx, bond order) — omit to add an unbonded atom.
    pub bonded_to: Option<(usize, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveAtomRequest {
    pub atom_idx: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondRequest {
    pub a: usize,
    pub b: usize,
    pub order: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveBondRequest {
    pub a: usize,
    pub b: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetFormalChargeRequest {
    pub atom_idx: usize,
    pub charge: i8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAtomDto {
    pub mol_block: String,
    pub new_atom_idx: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcError {
    pub message: String,
}

impl std::fmt::Display for IpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for IpcError {}

impl From<String> for IpcError {
    fn from(message: String) -> Self {
        IpcError { message }
    }
}
