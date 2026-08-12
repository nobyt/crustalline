use molrs::conformer::{self, Conformer, EmbedParams};
use molrs::depict::{self, LayoutParams, Style};
use molrs::edit;
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
    /// Snapshot-based undo/redo (plan §6) — cheap at interactive molecule sizes.
    history: Vec<MoleculeGraph>,
    redo_stack: Vec<MoleculeGraph>,
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
            history: Vec::new(),
            redo_stack: Vec::new(),
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

    fn reembed(&mut self) -> Result<(), CoreError> {
        let params = EmbedParams {
            seed: self.embed_seed,
            ..EmbedParams::default()
        };
        self.conformer = conformer::embed_molecule(&self.graph, &params)?;
        Ok(())
    }

    /// Runs a molrs::edit mutator against `self.graph`, atomically: on any
    /// failure (validation or re-embedding) the graph is rolled back to its
    /// pre-edit state, so a failed edit never leaves partial/inconsistent
    /// state for the caller to clean up. On success, the pre-edit graph is
    /// pushed onto the undo stack and the redo stack is cleared.
    ///
    /// Note: molrs::edit mutators return an old-idx -> new-idx remap
    /// (docs/molrs-api-contract.md §3.2/Deviation 1), which this discards —
    /// crustalline currently re-fetches and re-renders the whole molecule
    /// after every edit rather than holding any frontend-side atom-index
    /// state across edit boundaries, so there is nothing to translate yet.
    /// Revisit if a future UI feature (e.g. a persistent selection) needs it.
    fn with_edit<T>(
        &mut self,
        f: impl FnOnce(&mut MoleculeGraph) -> Result<T, edit::EditError>,
    ) -> Result<T, CoreError> {
        let before = self.graph.clone();
        match f(&mut self.graph) {
            Ok(result) => match self.reembed() {
                Ok(()) => {
                    self.history.push(before);
                    self.redo_stack.clear();
                    Ok(result)
                }
                Err(e) => {
                    self.graph = before;
                    Err(e)
                }
            },
            Err(e) => {
                self.graph = before;
                Err(CoreError::from(e))
            }
        }
    }

    pub fn add_atom(
        &mut self,
        symbol: &str,
        formal_charge: i8,
        bonded_to: Option<(usize, f64)>,
    ) -> Result<usize, CoreError> {
        self.with_edit(|g| edit::add_atom(g, symbol, formal_charge, bonded_to).map(|(idx, _remap)| idx))
    }

    pub fn remove_atom(&mut self, atom_idx: usize) -> Result<(), CoreError> {
        self.with_edit(|g| edit::remove_atom(g, atom_idx).map(|_remap| ()))
    }

    pub fn add_bond(&mut self, a: usize, b: usize, order: f64) -> Result<(), CoreError> {
        self.with_edit(|g| edit::add_bond(g, a, b, order).map(|_remap| ()))
    }

    pub fn remove_bond(&mut self, a: usize, b: usize) -> Result<(), CoreError> {
        self.with_edit(|g| edit::remove_bond(g, a, b))
    }

    pub fn set_bond_order(&mut self, a: usize, b: usize, order: f64) -> Result<(), CoreError> {
        self.with_edit(|g| edit::set_bond_order(g, a, b, order).map(|_remap| ()))
    }

    pub fn set_formal_charge(&mut self, atom_idx: usize, charge: i8) -> Result<(), CoreError> {
        self.with_edit(|g| edit::set_formal_charge(g, atom_idx, charge))
    }

    pub fn undo(&mut self) -> Result<(), CoreError> {
        let prev = self.history.pop().ok_or(CoreError::NothingToUndo)?;
        let current = std::mem::replace(&mut self.graph, prev);
        self.redo_stack.push(current);
        self.reembed()
    }

    pub fn redo(&mut self) -> Result<(), CoreError> {
        let next = self.redo_stack.pop().ok_or(CoreError::NothingToRedo)?;
        let current = std::mem::replace(&mut self.graph, next);
        self.history.push(current);
        self.reembed()
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

    #[test]
    fn add_atom_extends_molecule_and_reembeds() {
        let mut state = MoleculeState::from_smiles("C").expect("methane should embed");
        let new_idx = state.add_atom("C", 0, Some((0, 1.0))).expect("add_atom should succeed");
        assert_eq!(new_idx, 1);
        assert_eq!(state.conformer.coords.len(), state.graph.atoms.len());
        assert!(state.mol_block().contains("V2000"));
    }

    #[test]
    fn add_atom_over_valence_leaves_state_unchanged() {
        let mut state =
            MoleculeState::from_smiles("C(C)(C)(C)C").expect("neopentane core should embed");
        let before = state.graph.atoms.len();
        let err = state.add_atom("C", 0, Some((0, 1.0)));
        assert!(err.is_err());
        assert_eq!(state.graph.atoms.len(), before, "failed edit must not mutate state");
    }

    #[test]
    fn undo_redo_round_trips() {
        let mut state = MoleculeState::from_smiles("C").expect("methane should embed");
        let atoms_before = state.graph.atoms.len();
        state.add_atom("C", 0, Some((0, 1.0))).expect("add_atom should succeed");
        assert_ne!(state.graph.atoms.len(), atoms_before);

        state.undo().expect("undo should succeed");
        assert_eq!(state.graph.atoms.len(), atoms_before);

        state.redo().expect("redo should succeed");
        assert_ne!(state.graph.atoms.len(), atoms_before);

        assert!(matches!(state.redo(), Err(CoreError::NothingToRedo)));
    }

    #[test]
    fn undo_with_empty_history_is_an_error() {
        let mut state = MoleculeState::from_smiles("C").expect("methane should embed");
        assert!(matches!(state.undo(), Err(CoreError::NothingToUndo)));
    }
}
