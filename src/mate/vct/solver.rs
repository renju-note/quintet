use super::generator::Candidates;
use super::proof::{DEFAULT_CARRY_CAPACITY, ProofTable};
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
        Self::with_carry_capacity(
            attacker_vcf_depth,
            defender_vcf_depth,
            DEFAULT_CARRY_CAPACITY,
        )
    }

    /// Like [`Self::init`], but setting how much each memo carries from one
    /// search into the next (see [`ProofTable::advance_generation`]).
    pub fn with_carry_capacity(
        attacker_vcf_depth: u8,
        defender_vcf_depth: u8,
        carry_capacity: usize,
    ) -> Self {
        Self {
            attacker_table: ProofTable::with_carry_capacity(carry_capacity),
            defender_table: ProofTable::with_carry_capacity(carry_capacity),
            attacker_vcf_depth,
            defender_vcf_depth,
            attacker_vcf_solver: vcf::IDDFSSolver::with_carry_capacity(
                [1].to_vec(),
                carry_capacity,
            ),
            defender_vcf_solver: vcf::IDDFSSolver::with_carry_capacity(
                [1].to_vec(),
                carry_capacity,
            ),
            attacks_cache: LruCache::new(NonZeroUsize::new(1000).unwrap()),
            defences_cache: LruCache::new(NonZeroUsize::new(1000).unwrap()),
            policy: PhantomData,
        }
    }

    /// Forgets everything remembered from earlier searches: both proof
    /// tables, both move caches and the nested VCF solvers' deadends.
    ///
    /// Nothing requires this: every memo is keyed by the position, the turn,
    /// the remaining limit *and* the attacker, so what a search leaves behind
    /// stays true whatever is asked next, and [`Self::solve`] keeps the
    /// memory bounded on its own. Use it to hand a solver back to a caller
    /// with a clean slate, or to give back the memory.
    pub fn clear(&mut self) {
        self.attacker_table.clear();
        self.defender_table.clear();
        self.attacks_cache.clear();
        self.defences_cache.clear();
        self.attacker_vcf_solver.clear();
        self.defender_vcf_solver.clear();
    }

    /// Opens a new generation in every memo the solver keeps, which is how
    /// a reused solver's memory stays bounded. [`Self::solve`] does it; a
    /// caller driving [`Self::search`] or [`Self::extract`] by hand does it
    /// itself, once per question.
    pub fn advance_generation(&mut self) {
        self.attacker_table.advance_generation();
        self.defender_table.advance_generation();
        self.attacker_vcf_solver.advance_generation();
        self.defender_vcf_solver.advance_generation();
        // The two candidate caches are `LruCache`s, already bounded.
    }

    /// How many entries the two proof tables and the two nested VCF solvers
    /// hold between them, for a caller sizing `carry_capacity`.
    pub fn memo_len(&self) -> usize {
        self.attacker_table.len()
            + self.defender_table.len()
            + self.attacker_vcf_solver.deadends_len()
            + self.defender_vcf_solver.deadends_len()
    }

    /// Searches for a VCT. `None` means either "no VCT within `state.limit`"
    /// or "gave up"; the two are told apart by `budget.is_exhausted()`.
    ///
    /// Ask as many questions of one solver as you like, about either
    /// attacker: what it remembers is keyed so that answers cannot be
    /// confused, and each call opens a new generation so that the memos do
    /// not grow without bound.
    ///
    /// What reuse buys is asking the *same* question again, which costs a
    /// few nodes instead of the whole search. Two different positions share
    /// much less than one might hope, because the remaining limit is part of
    /// the key: the same board reached from two roots is two entries unless
    /// the roots are the same depth away from it.
    pub fn solve(&mut self, state: &mut VCTState, budget: &mut NodeBudget) -> Option<Mate> {
        self.advance_generation();
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
