use crate::board::Point;
use crate::mate::mate::Mate;
use crate::mate::vcf;
use crate::mate::vct::generator::*;
use crate::mate::vct::helper::VCFHelper;
use crate::mate::vct::proof::*;
use crate::mate::vct::resolver::Resolver;
use crate::mate::vct::searcher::Searcher;
use crate::mate::vct::selector::*;
use crate::mate::vct::solver::VCTSolver;
use crate::mate::vct::state::VCTState;
use crate::mate::vct::traverser::*;
use lru::LruCache;

pub struct PNSVCTSolver {
    attacker_table: Table,
    defender_table: Table,
    attacker_vcf_depth: u8,
    defender_vcf_depth: u8,
    attacker_vcf_solver: vcf::IDDFSSolver,
    defender_vcf_solver: vcf::IDDFSSolver,
    attacks_cache: LruCache<u64, Result<Vec<Point>, Node>>,
    defences_cache: LruCache<u64, Result<Vec<Point>, Node>>,
}

impl PNSVCTSolver {
    pub fn init(attacker_vcf_depth: u8, defender_vcf_depth: u8) -> Self {
        Self {
            attacker_table: Table::new(),
            defender_table: Table::new(),
            attacker_vcf_depth: attacker_vcf_depth,
            defender_vcf_depth: defender_vcf_depth,
            attacker_vcf_solver: vcf::IDDFSSolver::init([1].to_vec()),
            defender_vcf_solver: vcf::IDDFSSolver::init([1].to_vec()),
            attacks_cache: LruCache::new(1000),
            defences_cache: LruCache::new(1000),
        }
    }
}

impl VCTSolver for PNSVCTSolver {}

impl Searcher for PNSVCTSolver {}

impl Selector for PNSVCTSolver {}

impl Generator for PNSVCTSolver {
    fn attacks_cache(&mut self) -> &mut LruCache<u64, Result<Vec<Point>, Node>> {
        &mut self.attacks_cache
    }

    fn defences_cache(&mut self) -> &mut LruCache<u64, Result<Vec<Point>, Node>> {
        &mut self.defences_cache
    }
}

impl ProofTree for PNSVCTSolver {
    fn attacker_table(&mut self) -> &mut Table {
        &mut self.attacker_table
    }

    fn defender_table(&mut self) -> &mut Table {
        &mut self.defender_table
    }
}

impl Traverser for PNSVCTSolver {
    fn next_threshold_attack(&self, selection: &Selection, threshold: Node) -> Node {
        PNSTraverser::next_threshold_attack(self, selection, threshold)
    }

    fn next_threshold_defence(&self, selection: &Selection, threshold: Node) -> Node {
        PNSTraverser::next_threshold_defence(self, selection, threshold)
    }
}

impl PNSTraverser for PNSVCTSolver {}

impl Resolver for PNSVCTSolver {
    fn solve_attacker_vcf(&mut self, state: &VCTState) -> Option<Mate> {
        VCFHelper::solve_attacker_vcf(self, state)
    }

    fn solve_attacker_threat(&mut self, state: &VCTState) -> Option<Mate> {
        VCFHelper::solve_attacker_threat(self, state)
    }
}

impl VCFHelper for PNSVCTSolver {
    fn attacker_vcf_depth(&self) -> u8 {
        self.attacker_vcf_depth
    }

    fn defender_vcf_depth(&self) -> u8 {
        self.defender_vcf_depth
    }

    fn attacker_vcf_solver(&mut self) -> &mut vcf::IDDFSSolver {
        &mut self.attacker_vcf_solver
    }

    fn defender_vcf_solver(&mut self) -> &mut vcf::IDDFSSolver {
        &mut self.defender_vcf_solver
    }
}
