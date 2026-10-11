//! VCT — victory by continuous threats.
//!
//! The attacker may make threes as well as fours, so the defender has a
//! choice of replies and the tree is a proper AND/OR tree: an OR node where
//! the attacker picks a move, an AND node where every defence must be
//! answered. The search is proof-number search over that tree; a node's
//! proof number is how many leaves would still have to be won to prove it,
//! its disproof number how many to refute it, and the search keeps
//! descending into the child that promises the cheapest decision.
//!
//! One [`VCTSolver`] does all of it. What makes it DFS, PNS or df-pn is a
//! [`ThresholdPolicy`]: how far a child may be searched before control
//! returns to its parent. The solver's methods are split by phase:
//!
//! | phase | module | what it does |
//! | --- | --- | --- |
//! | search | `searcher.rs` | `search_attacks` / `search_defences` evaluate an OR / AND node; `expand_*` loop into the most-proving child until the threshold is exceeded; `select_*` read the children's table entries, aggregate them into the node's own numbers and pick that child |
//! | generate | `generator.rs` | `generate_attacks` / `generate_defences` list a node's candidate moves, best first, or decide the node outright ([`Candidates`]) |
//! | nested VCF | `nested_vcf.rs` | [`NestedVCF`]: the VCF sub-searches the generator asks — has a side a VCF now, would it have one after a pass (a threat) |
//! | extract | `extractor.rs` | `extract` walks the tables after a proof and recovers the winning line |
//! | auto | `auto.rs` | `solve_auto` searches with only a node budget, under several `threat_limit`s in turn |
//!
//! The state they share is [`VCTState`] (`state.rs`): the game, the remaining
//! limit, the `threat_limit` and a shape map for move ordering. What they
//! remember is a [`ProofTable`] per side (`proof.rs`), holding [`PnDn`]s —
//! proof and disproof numbers — keyed by position (with the `threat_limit`)
//! and limit.
//!
//! `docs/06-solver-vct.en.md` walks through all of this with
//! examples.
//!
//! [`Candidates`]: generator::Candidates
//! [`NestedVCF`]: nested_vcf::NestedVCF
//! [`ProofTable`]: proof::ProofTable

mod auto;
mod extractor;
mod generator;
mod nested_vcf;
mod proof;
mod searcher;
mod solver;
mod state;
mod threshold;
mod tree;

pub use auto::{AUTO_FIRST_SLICE, AUTO_LIMIT, AUTO_THREAT_LIMITS};
pub use proof::PnDn;
pub use solver::VCTSolver;
pub use state::VCTState;
pub use threshold::{DFPNSThreshold, DFSThreshold, PNSThreshold, ThresholdPolicy};
pub use tree::ProofTree;

pub type DFSVCTSolver = VCTSolver<DFSThreshold>;
pub type PNSVCTSolver = VCTSolver<PNSThreshold>;
pub type DFPNSVCTSolver = VCTSolver<DFPNSThreshold>;
