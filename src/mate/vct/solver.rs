use super::generator::Candidates;
use super::proof::ProofTable;
use super::state::VCTState;
use super::threshold::ThresholdPolicy;
use crate::mate::budget::NodeBudget;
use crate::mate::mate::Mate;
use crate::mate::vcf;
use lru::LruCache;
use std::marker::PhantomData;
use std::num::NonZeroUsize;

/// The VCT solver. The three variants (DFS, PNS, df-pn) share everything
/// except how child thresholds are chosen, which is supplied by `P`.
///
/// The methods are split by phase across sibling modules:
/// `searcher.rs` (AND/OR search), `selector.rs` (most-proving child),
/// `generator.rs` (candidate moves), `nested_vcf.rs` (VCF sub-searches) and
/// `extractor.rs` (recovering the winning line after a proof).
pub struct VCTSolver<P: ThresholdPolicy> {
    pub(super) attacker_table: ProofTable,
    pub(super) defender_table: ProofTable,
    pub(super) attacker_vcf_depth: u8,
    pub(super) defender_vcf_depth: u8,
    pub(super) attacker_vcf_solver: vcf::IDDFSSolver,
    pub(super) defender_vcf_solver: vcf::IDDFSSolver,
    pub(super) attacks_cache: LruCache<u64, Candidates>,
    pub(super) defences_cache: LruCache<u64, Candidates>,
    policy: PhantomData<P>,
}

impl<P: ThresholdPolicy> VCTSolver<P> {
    pub fn init(attacker_vcf_depth: u8, defender_vcf_depth: u8) -> Self {
        Self {
            attacker_table: ProofTable::new(),
            defender_table: ProofTable::new(),
            attacker_vcf_depth,
            defender_vcf_depth,
            attacker_vcf_solver: vcf::IDDFSSolver::init([1].to_vec()),
            defender_vcf_solver: vcf::IDDFSSolver::init([1].to_vec()),
            attacks_cache: LruCache::new(NonZeroUsize::new(1000).unwrap()),
            defences_cache: LruCache::new(NonZeroUsize::new(1000).unwrap()),
            policy: PhantomData,
        }
    }

    /// Forgets everything remembered from earlier searches: both proof
    /// tables, both move caches and the nested VCF solvers' deadends. Call it
    /// before reusing a solver on a position that is not a descendant of the
    /// last one.
    pub fn clear(&mut self) {
        self.attacker_table.clear();
        self.defender_table.clear();
        self.attacks_cache.clear();
        self.defences_cache.clear();
        self.attacker_vcf_solver.clear();
        self.defender_vcf_solver.clear();
    }

    /// Searches for a VCT. `None` means either "no VCT within `state.limit`"
    /// or "gave up"; the two are told apart by `budget.is_exhausted()`.
    pub fn solve(&mut self, state: &mut VCTState, budget: &mut NodeBudget) -> Option<Mate> {
        if self.search(state, budget) {
            // The root is proven, so the winning line is in the tables and
            // recovering it is bounded by the length of that line. It would be
            // pointless to abandon a proof for want of budget, so the walk runs
            // outside the budget.
            self.extract(state, &mut NodeBudget::unlimited())
        } else {
            None
        }
    }
}
