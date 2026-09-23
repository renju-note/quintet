use super::generator::Candidates;
use super::nested_vcf::NestedVCF;
use super::proof::{DEFAULT_CARRY_CAPACITY, ProofTable};
use super::state::VCTState;
use super::threshold::ThresholdPolicy;
use crate::mate::budget::NodeBudget;
use crate::mate::mate::Mate;
use crate::mate::memo::ZobristBuildHasher;
use crate::mate::solver::Solver;
use lru::LruCache;
use std::marker::PhantomData;
use std::num::NonZeroUsize;

/// The VCT solver. The three variants (DFS, PNS, df-pn) share everything
/// except how child thresholds are chosen, which is supplied by `P`.
///
/// The methods are split by phase across sibling modules:
/// `searcher.rs` (AND/OR search and choosing the most-proving child),
/// `generator.rs` (candidate moves) and `extractor.rs` (recovering the
/// winning line after a proof); `nested_vcf.rs` is the VCF sub-search the
/// generator and the extractor ask.
pub struct VCTSolver<P: ThresholdPolicy> {
    /// Proof numbers of the positions after an attack (defender to move).
    pub(super) attacker_table: ProofTable,
    /// Proof numbers of the positions after a defence (attacker to move).
    pub(super) defender_table: ProofTable,
    /// The attacker's nested VCF search, `threat_limit` deep.
    pub(super) attacker_vcf: NestedVCF,
    /// The defender's nested VCF search, `defender_vcf_depth` deep.
    pub(super) defender_vcf: NestedVCF,
    pub(super) attacks_cache: LruCache<u64, Candidates, ZobristBuildHasher>,
    pub(super) defences_cache: LruCache<u64, Candidates, ZobristBuildHasher>,
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
    /// search into the next (see [`Solver::advance_generation`]).
    pub fn with_carry_capacity(
        attacker_vcf_depth: u8,
        defender_vcf_depth: u8,
        carry_capacity: usize,
    ) -> Self {
        // Candidate generation asks the nested VCF solvers with
        // `min(state.limit, depth)`, so below this the moves generated still
        // move with the limit and one limit's decision says nothing about
        // another's. `search_defences` also cuts off at `limit <= 1`.
        let transfer_from = attacker_vcf_depth
            .max(defender_vcf_depth.saturating_add(1))
            .max(2);
        Self {
            attacker_table: ProofTable::with_carry_capacity(carry_capacity, transfer_from),
            defender_table: ProofTable::with_carry_capacity(carry_capacity, transfer_from),
            attacker_vcf: NestedVCF::new(true, attacker_vcf_depth, carry_capacity),
            defender_vcf: NestedVCF::new(false, defender_vcf_depth, carry_capacity),
            attacks_cache: LruCache::with_hasher(
                NonZeroUsize::new(1000).unwrap(),
                ZobristBuildHasher::default(),
            ),
            defences_cache: LruCache::with_hasher(
                NonZeroUsize::new(1000).unwrap(),
                ZobristBuildHasher::default(),
            ),
            policy: PhantomData,
        }
    }
}

impl<P: ThresholdPolicy> Solver for VCTSolver<P> {
    type State = VCTState;

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
    fn solve(&mut self, state: &mut VCTState, budget: &mut NodeBudget) -> Option<Mate> {
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

    /// Forgets both proof tables, both move caches and the nested VCF
    /// solvers' deadends.
    fn clear(&mut self) {
        self.attacker_table.clear();
        self.defender_table.clear();
        self.attacks_cache.clear();
        self.defences_cache.clear();
        self.attacker_vcf.clear();
        self.defender_vcf.clear();
    }

    /// A caller driving [`Self::search`] or [`Self::extract`] by hand does
    /// this itself, once per question.
    fn advance_generation(&mut self) {
        self.attacker_table.advance_generation();
        self.defender_table.advance_generation();
        self.attacker_vcf.advance_generation();
        self.defender_vcf.advance_generation();
        // The two candidate caches are `LruCache`s, already bounded.
    }

    /// The two proof tables and the two nested VCF solvers' deadends.
    fn memo_len(&self) -> usize {
        self.attacker_table.len()
            + self.defender_table.len()
            + self.attacker_vcf.memo_len()
            + self.defender_vcf.memo_len()
    }
}
