use molrs::conformer::{self, Conformer, EmbedParams};
use molrs::depict::{self, LayoutParams, Style};
use molrs::graph::{self, MoleculeGraph};

use crate::error::CoreError;

/// Rust-side source of truth for the currently loaded molecule (plan §6).
/// The frontend holds no independent copy — it only ever displays whatever
/// MOL block it was last handed.
pub struct MoleculeState {
    pub graph: MoleculeGraph,
    pub conformer: Conformer,
    /// Reused across re-embeds so the 3D view doesn't jump around between edits.
    pub embed_seed: u64,
}

const DEFAULT_EMBED_SEED: u64 = 0xC0FFEE;

impl MoleculeState {
    pub fn from_smiles(smiles: &str) -> Result<Self, CoreError> {
        let graph = graph::build_molecule_graph(smiles)?;
        let params = EmbedParams {
            seed: DEFAULT_EMBED_SEED,
            ..EmbedParams::default()
        };
        let conformer = conformer::embed_molecule(&graph, &params)?;
        Ok(MoleculeState {
            graph,
            conformer,
            embed_seed: DEFAULT_EMBED_SEED,
        })
    }

    pub fn mol_block(&self) -> String {
        conformer::molblock::to_mol_block(&self.graph, &self.conformer, "crustalline")
    }

    pub fn svg(&self) -> Result<String, CoreError> {
        let coords = depict::compute_coords_2d(&self.graph, &LayoutParams::default())?;
        Ok(depict::to_svg(&self.graph, &coords, &Style::iupac_default()))
    }

    pub fn export_svg(&self, path: &str) -> Result<(), CoreError> {
        let svg = self.svg()?;
        std::fs::write(path, svg).map_err(|source| CoreError::Io {
            path: path.to_string(),
            source,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn benzene_round_trips_through_molrs() {
        let state = MoleculeState::from_smiles("c1ccccc1").expect("benzene should embed");
        assert_eq!(state.graph.atoms.len(), 12); // 6 C + 6 H, hydrogens explicit
        assert_eq!(state.conformer.coords.len(), 12);
        let mol_block = state.mol_block();
        assert!(mol_block.contains("V2000"));
    }

    #[test]
    fn invalid_smiles_is_rejected() {
        assert!(MoleculeState::from_smiles("not a smiles(").is_err());
    }

    #[test]
    fn svg_export_writes_a_file() {
        let state = MoleculeState::from_smiles("c1ccccc1").expect("benzene should embed");
        let svg = state.svg().expect("2D depiction should succeed for benzene");
        assert!(svg.contains("<svg"));

        let dir = std::env::temp_dir().join(format!("crustalline-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("benzene.svg");
        state.export_svg(path.to_str().unwrap()).expect("export_svg should succeed");
        let written = std::fs::read_to_string(&path).unwrap();
        assert_eq!(written, svg);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
