use super::generator::Candidates;
use super::proof::ProofTable;
use super::state::VCTState;
use super::threshold::ThresholdPolicy;
use crate::mate::mate::Mate;
use crate::mate::vcf;
use lru::LruCache;
use std::marker::PhantomData;

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
            attacks_cache: LruCache::new(1000),
            defences_cache: LruCache::new(1000),
            policy: PhantomData,
        }
    }

    pub fn solve(&mut self, state: &mut VCTState) -> Option<Mate> {
        if self.search(state) {
            self.extract(state)
        } else {
            None
        }
    }
}
