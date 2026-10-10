use super::proof::PnDn;
use super::solver::VCTSolver;
use super::state::VCTState;
use super::threshold::ThresholdPolicy;
use crate::board::RowKind::Sword;
use crate::board::*;
use crate::feature::area::Area;
use crate::mate::budget::NodeBudget;
use crate::mate::game::*;
use crate::mate::mate::Mate;
use crate::mate::memo::ZobristBuildHasher;
use crate::mate::state::State;
use std::collections::HashMap;

/// A VCT as a whole proof rather than one line: one attack at each node
/// where the attacker moves, every defence worth trying at each node where
/// the defender does, and the attacker's VCF at each leaf.
///
/// [`VCTSolver::proof_tree`] reads it off the tables after a search, and
/// [`VCTSolver::proves`] checks it on another position, typically the same
/// one with one more stone of the defender's. That answers "does this move
/// stop the mate?" for many moves at a fraction of a new search each, and
/// [`Self::zone`] answers it for most moves without even that.
///
/// Positions that two orders of the same moves reach are one node, as in
/// the search's tables, so the tree is a DAG: unfolded, a long proof can
/// have millions of lines.
#[derive(Debug, Clone)]
pub struct ProofTree {
    /// The nodes where the attacker moves; the first is the root.
    attacks: Vec<AttackNode>,
    /// The nodes where the defender moves.
    defences: Vec<DefenceNode>,
    zone: Area,
}

/// A position with the attacker to move.
#[derive(Debug, Clone)]
enum AttackNode {
    /// The attacker wins by fours from here: the VCF's moves.
    Vcf(Vec<Point>),
    /// The attacker plays the point (the forced one if the defender has a
    /// four); the position after it, in `defences`.
    Move(Point, usize),
}

/// A position with the defender to move, after an attack.
#[derive(Debug, Clone)]
enum DefenceNode {
    /// The attack already won: two fives, or one the defender may not take.
    Won,
    /// The attack is a four: its one reply, and the position after it, in
    /// `attacks`.
    Forced(Point, usize),
    /// The attack is a threat: the attacker's VCF should the defender pass,
    /// and the position after each move worth trying against it
    /// ([`VCTState::threat_defences`]); none if every one is forbidden.
    Threat {
        threat: Vec<Point>,
        defences: Vec<(Point, usize)>,
    },
}

impl ProofTree {
    /// The points where one more stone of the defender's may break the
    /// proof. A stone anywhere else leaves it standing: on the position
    /// with that stone, [`VCTSolver::proves`] holds.
    ///
    /// It is the union of what the nodes of the proof depend on:
    ///
    /// - every point the proof plays, attacks, defences and the moves of
    ///   every VCF in it;
    /// - around each attacker move, its four lines up to four cells away
    ///   (five for Black, whose moves may become forbidden), where a stone
    ///   would block a four;
    /// - the points that end each VCF: both fives, or around a five Black
    ///   may not take, five cells;
    /// - around each defence left out because it is forbidden, five cells;
    /// - the points where the defender's stone would join a segment holding
    ///   two of the defender's stones, at any position of the proof after
    ///   one of the defender's moves: new fours for the defender to defend
    ///   with, or to win by;
    /// - at each node where the defender moves, the zone of the search that
    ///   showed the defender has no VCF
    ///   ([`DFSSolver::search_zone`](crate::mate::DFSSolver::search_zone)).
    ///
    /// The distance-five rings for forbidden points are the same
    /// approximation as in `search_zone` and
    /// [`VCTState::threat_defences`]: a three may be no real three because
    /// of a point further away.
    pub fn zone(&self) -> &Area {
        &self.zone
    }

    /// How many positions the proof has, those where the attacker moves and
    /// those where the defender does. The work [`VCTSolver::proves`] does
    /// grows with it.
    pub fn size(&self) -> usize {
        self.attacks.len() + self.defences.len()
    }

    /// The first attack, or the first move of the VCF.
    pub fn first_move(&self) -> Option<Point> {
        match &self.attacks[0] {
            AttackNode::Vcf(path) => path.first().copied(),
            &AttackNode::Move(p, _) => Some(p),
        }
    }
}

impl DefenceNode {
    /// The position after `defence`, in `attacks`.
    fn child(&self, defence: Point) -> Option<usize> {
        match self {
            DefenceNode::Won => None,
            &DefenceNode::Forced(p, child) => (p == defence).then_some(child),
            DefenceNode::Threat { defences, .. } => defences
                .iter()
                .find(|&&(p, _)| p == defence)
                .map(|&(_, child)| child),
        }
    }
}

/// Reading a proof off the tables, and checking it elsewhere.
impl<P: ThresholdPolicy> VCTSolver<P> {
    /// The proof of the VCT just found for `state`, as a tree, with its
    /// [zone](ProofTree::zone). Ask it right after [`Solver::solve`]
    /// (or `search`) proved `state`, before another question opens a new
    /// generation and the tables start to forget.
    ///
    /// `None` if `state` is not proven in the tables, if a branch the proof
    /// needs is gone and does not prove again, or if `budget` runs out: the
    /// walk asks the nested VCF searches again at each node, each position
    /// of the proof costs a node, and so does searching a branch the tables
    /// lack (see `tree_defence`).
    ///
    /// [`Solver::solve`]: crate::mate::Solver::solve
    pub fn proof_tree(
        &mut self,
        state: &mut VCTState,
        budget: &mut NodeBudget,
    ) -> Option<ProofTree> {
        if !state.attacking() {
            return None;
        }
        let defender = state.attacker().opponent();
        let mut walk = Walk {
            tree: ProofTree {
                attacks: vec![],
                defences: vec![],
                zone: swords(state.game().board(), defender),
            },
            seen_attacks: HashMap::default(),
            seen_defences: HashMap::default(),
        };
        let root = self.tree_attacks(state, budget, &mut walk)?;
        debug_assert_eq!(root, 0);
        Some(walk.tree)
    }

    /// Whether `tree` proves a VCT for the attacker of `state` within
    /// `state.limit()`, checked node by node the way the search decides
    /// them: each attack still legal, each threat's VCF still playing, the
    /// defender still without a VCF, and every move worth trying against
    /// each threat, as [`VCTState::threat_defences`] lists them on this
    /// position, still answered by the proof.
    ///
    /// `true` is a proof as good as the search's own. `false` says only
    /// that this proof does not carry over; there may be another. A `budget`
    /// that runs out also gives `false`. Searching is kept to the
    /// defender's VCF, asked anew at each node where the defender moves;
    /// the VCFs of the proof are replayed, not searched for. Each node is
    /// checked once, however many lines reach it.
    pub fn proves(
        &mut self,
        state: &mut VCTState,
        tree: &ProofTree,
        budget: &mut NodeBudget,
    ) -> bool {
        let mut check = Check {
            tree,
            attacks: vec![None; tree.attacks.len()],
            defences: vec![None; tree.defences.len()],
        };
        state.attacking() && self.proves_attacks(state, 0, budget, &mut check)
    }

    /// The node for `state`, the attacker to move, as an index into
    /// `attacks`.
    fn tree_attacks(
        &mut self,
        state: &mut VCTState,
        budget: &mut NodeBudget,
        walk: &mut Walk,
    ) -> Option<usize> {
        let key = state.zobrist_hash();
        if let Some(&index) = walk.seen_attacks.get(&key) {
            return Some(index);
        }
        if !budget.consume() {
            return None;
        }
        // A forbidden attack is left out even when the table has its child
        // proven: the same position may have been reached by another order
        // of the moves, one that played the point while it was not
        // forbidden.
        let attack = match state.check_event() {
            Some(Forced(attack)) if state.is_forbidden_move(attack) => return None,
            Some(Forced(attack)) => Some(attack),
            Some(Defeated(_)) => return None,
            // The proven child the extractor would take, so that the tree
            // and the line `solve` gives begin alike.
            None => state.empty_points().into_iter().find(|&attack| {
                self.attacker_table
                    .lookup_next(state, Some(attack))
                    .is_some_and(|node| node.is_proven())
                    && !state.is_forbidden_move(attack)
            }),
        };
        let node = match attack {
            // No proven child: the attacker's VCF proved the node.
            None => {
                let mate = self.attacker_vcf.vcf(state, budget)?;
                walk.vcf(state.game().fork(mate.path.len()), &mate.path);
                AttackNode::Vcf(mate.path)
            }
            Some(attack) => {
                let proven = self
                    .attacker_table
                    .lookup_next(state, Some(attack))
                    .is_some_and(|node| node.is_proven());
                if !proven {
                    return None;
                }
                walk.attack(state, attack);
                // The slot is taken before the children, so that the root
                // is the first.
                let index = walk.push_attack(key, AttackNode::Vcf(vec![]));
                let child =
                    state.with_move(Some(attack), |s| self.tree_defences(s, budget, walk))?;
                walk.tree.attacks[index] = AttackNode::Move(attack, child);
                return Some(index);
            }
        };
        Some(walk.push_attack(key, node))
    }

    /// The node for `state`, the defender to move, as an index into
    /// `defences`.
    fn tree_defences(
        &mut self,
        state: &mut VCTState,
        budget: &mut NodeBudget,
        walk: &mut Walk,
    ) -> Option<usize> {
        let key = state.zobrist_hash();
        if let Some(&index) = walk.seen_defences.get(&key) {
            return Some(index);
        }
        if !budget.consume() {
            return None;
        }
        let node = match state.check_event() {
            Some(Defeated(end)) => {
                walk.end(end);
                DefenceNode::Won
            }
            Some(Forced(defence)) => {
                if state.limit() <= 1 {
                    return None;
                }
                let child = self.tree_defence(state, defence, budget, walk)?;
                DefenceNode::Forced(defence, child)
            }
            None => {
                if state.limit() <= 1 {
                    return None;
                }
                let threat = self.attacker_vcf.threat(state, budget)?;
                match self.defender_vcf.vcf_or_zone(state, budget) {
                    Ok(_) => return None,
                    Err(zone) => walk.tree.zone |= zone,
                }
                if budget.is_exhausted() {
                    return None;
                }
                let mut game = state.game().fork(1 + threat.path.len());
                game.play(None);
                walk.vcf(game, &threat.path);
                let mut defences = vec![];
                for defence in dedup(state.threat_defences(&threat)) {
                    if state.is_forbidden_move(defence) {
                        walk.tree.zone |= Area::around(defence, 5);
                        continue;
                    }
                    let child = self.tree_defence(state, defence, budget, walk)?;
                    defences.push((defence, child));
                }
                DefenceNode::Threat {
                    threat: threat.path,
                    defences,
                }
            }
        };
        let index = walk.tree.defences.len();
        walk.tree.defences.push(node);
        walk.seen_defences.insert(key, index);
        Some(index)
    }

    /// The node after `defence`. A defence the table does not know is
    /// searched: the threat asked again here may not be the one the search
    /// generated its defences from, and its defences then differ. Those the
    /// search did not try leave its own threat standing, so they are
    /// quickly proven.
    fn tree_defence(
        &mut self,
        state: &mut VCTState,
        defence: Point,
        budget: &mut NodeBudget,
        walk: &mut Walk,
    ) -> Option<usize> {
        let known = self.defender_table.lookup_next(state, Some(defence));
        walk.tree.zone.insert(defence);
        state.with_move(Some(defence), |s| {
            let proven = match known {
                Some(node) => node.is_proven(),
                None => {
                    let node = self.search_attacks(s, PnDn::no_threshold(), budget);
                    if !budget.is_exhausted() {
                        self.defender_table.insert(s, node);
                    }
                    node.is_proven()
                }
            };
            if !proven {
                return None;
            }
            walk.defended(s);
            self.tree_attacks(s, budget, walk)
        })
    }

    fn proves_attacks(
        &mut self,
        state: &mut VCTState,
        index: usize,
        budget: &mut NodeBudget,
        check: &mut Check,
    ) -> bool {
        if let Some(known) = check.attacks[index] {
            return known;
        }
        let result = self.check_attacks(state, index, budget, check);
        // A check the budget cut short is not an answer.
        if !budget.is_exhausted() {
            check.attacks[index] = Some(result);
        }
        result
    }

    fn check_attacks(
        &mut self,
        state: &mut VCTState,
        index: usize,
        budget: &mut NodeBudget,
        check: &mut Check,
    ) -> bool {
        if !budget.consume() || state.limit() == 0 {
            return false;
        }
        let event = state.check_event();
        if matches!(event, Some(Defeated(_))) {
            return false;
        }
        match &check.tree.attacks[index] {
            AttackNode::Vcf(path) => {
                path.len().div_ceil(2) <= state.limit() as usize
                    && replay_vcf(&mut state.game().fork(path.len()), path).is_some()
            }
            &AttackNode::Move(attack, child) => {
                if matches!(event, Some(Forced(p)) if p != attack)
                    || state.game().board().stone(attack).is_some()
                    || state.is_forbidden_move(attack)
                {
                    return false;
                }
                state.with_move(Some(attack), |s| {
                    self.proves_defences(s, child, budget, check)
                })
            }
        }
    }

    fn proves_defences(
        &mut self,
        state: &mut VCTState,
        index: usize,
        budget: &mut NodeBudget,
        check: &mut Check,
    ) -> bool {
        if let Some(known) = check.defences[index] {
            return known;
        }
        let result = self.check_defences(state, index, budget, check);
        if !budget.is_exhausted() {
            check.defences[index] = Some(result);
        }
        result
    }

    fn check_defences(
        &mut self,
        state: &mut VCTState,
        index: usize,
        budget: &mut NodeBudget,
        check: &mut Check,
    ) -> bool {
        if !budget.consume() {
            return false;
        }
        let tree = check.tree;
        let node = &tree.defences[index];
        match state.check_event() {
            Some(Defeated(_)) => return true,
            Some(Forced(defence)) => {
                return state.limit() > 1
                    && node.child(defence).is_some_and(|child| {
                        state.with_move(Some(defence), |s| {
                            self.proves_attacks(s, child, budget, check)
                        })
                    });
            }
            None => {}
        }
        let DefenceNode::Threat { threat, .. } = node else {
            return false;
        };
        if state.limit() <= 1 || threat.len().div_ceil(2) > state.limit() as usize {
            return false;
        }
        let mut game = state.game().fork(1 + threat.len());
        game.play(None);
        let Some(end) = replay_vcf(&mut game, threat) else {
            return false;
        };
        if self.defender_vcf.vcf(state, budget).is_some() || budget.is_exhausted() {
            return false;
        }
        let threat = Mate::new(end, threat.clone());
        for defence in dedup(state.threat_defences(&threat)) {
            if state.is_forbidden_move(defence) {
                continue;
            }
            let holds = node.child(defence).is_some_and(|child| {
                state.with_move(Some(defence), |s| {
                    self.proves_attacks(s, child, budget, check)
                })
            });
            if !holds {
                return false;
            }
        }
        true
    }
}

/// What [`VCTSolver::proof_tree`] builds and collects on its way through
/// the proof.
struct Walk {
    tree: ProofTree,
    /// The nodes made so far, by [`State::zobrist_hash`]: the position and
    /// the limit.
    seen_attacks: HashMap<u64, usize, ZobristBuildHasher>,
    seen_defences: HashMap<u64, usize, ZobristBuildHasher>,
}

impl Walk {
    fn push_attack(&mut self, key: u64, node: AttackNode) -> usize {
        let index = self.tree.attacks.len();
        self.tree.attacks.push(node);
        self.seen_attacks.insert(key, index);
        index
    }

    /// The attacker, to move in `state`, plays `attack`.
    fn attack(&mut self, state: &VCTState, attack: Point) {
        let reach = if state.game().turn.is_black() { 5 } else { 4 };
        self.tree.zone |= Area::around(attack, reach);
    }

    /// The defender has just moved in `state`.
    fn defended(&mut self, state: &VCTState) {
        self.tree.zone |= swords(state.game().board(), state.game().turn.opponent());
    }

    fn end(&mut self, end: End) {
        match end {
            Fours(p1, p2) => {
                self.tree.zone.insert(p1);
                self.tree.zone.insert(p2);
            }
            Forbidden(p) => self.tree.zone |= Area::around(p, 5),
            Unknown => {}
        }
    }

    /// Plays the VCF `path` on `game`, the attacker to move, collecting as
    /// it goes.
    fn vcf(&mut self, mut game: Game, path: &[Point]) {
        let attacker = game.turn;
        for &p in path {
            if game.turn == attacker {
                let reach = if game.turn.is_black() { 5 } else { 4 };
                self.tree.zone |= Area::around(p, reach);
                game.play(Some(p));
            } else {
                self.tree.zone.insert(p);
                game.play(Some(p));
                self.tree.zone |= swords(game.board(), game.turn.opponent());
            }
        }
        if let Some(Defeated(end)) = game.check_event() {
            self.end(end);
        }
    }
}

/// What [`VCTSolver::proves`] has found so far, node by node.
struct Check<'a> {
    tree: &'a ProofTree,
    attacks: Vec<Option<bool>>,
    defences: Vec<Option<bool>>,
}

/// The empty points where a stone of `r`'s joins a segment holding two or
/// more of `r`'s stones and none of the opponent's: a [`Shape::Sword`] or
/// more along some line.
///
/// [`Shape::Sword`]: crate::feature::shape::Shape::Sword
fn swords(board: &Board, r: Player) -> Area {
    let mut result = Area::new();
    for (d, i, line) in board.lines() {
        let starts = line.score_mask(r, 2) | line.score_mask(r, 3) | line.score_mask(r, 4);
        if starts == 0 {
            continue;
        }
        for j in Bits(line.eyes_of(starts, Sword.eye_cells())) {
            result.insert(Index::new(d, i, j).to_point());
        }
    }
    result
}

/// `points` without duplicates, in their order.
fn dedup(mut points: Vec<Point>) -> Vec<Point> {
    let mut seen = Area::new();
    points.retain(|&p| seen.insert(p));
    points
}

/// Whether `path` is still a VCF on `game`, the attacker to move: each
/// attack a legal four (the forced move, if the defender has a four),
/// each defence its one reply, until an attack leaves two fives or one the
/// defender may not take. Its end if so. The moves are taken back.
fn replay_vcf(game: &mut Game, path: &[Point]) -> Option<End> {
    let mut played = 0;
    let mut i = 0;
    let end = loop {
        let Some(&attack) = path.get(i) else {
            break None;
        };
        match game.check_event() {
            Some(Defeated(_)) => break None,
            Some(Forced(p)) if p != attack => break None,
            _ => {}
        }
        if game.board().stone(attack).is_some() || game.is_forbidden_move(attack) {
            break None;
        }
        game.play(Some(attack));
        played += 1;
        match game.check_event() {
            Some(Defeated(end)) => break Some(end),
            Some(Forced(p)) if path.get(i + 1) == Some(&p) => {
                game.play(Some(p));
                played += 1;
                i += 2;
            }
            _ => break None,
        }
    };
    for _ in 0..played {
        game.undo();
    }
    end
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mate::solver::Solver;
    use crate::mate::vct::DFPNSVCTSolver;

    /// A proven VCT case of the benchmark: the board, the attacker, the
    /// limit and the threat limit.
    struct Case {
        name: String,
        board: Board,
        attacker: Player,
        limit: u8,
        threat_limit: u8,
    }

    /// The benchmark's cases tagged `vct proven` and `quick`, read the way
    /// `benches/solvers.rs` reads them (the fields used here only).
    fn quick_proven_cases() -> Vec<Case> {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/benches/cases");
        let mut names: Vec<_> = std::fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|x| x == "txt"))
            .collect();
        names.sort();
        let mut cases = vec![];
        for path in names {
            let text = std::fs::read_to_string(&path).unwrap();
            let (head, board) = text.split_once("board:").unwrap();
            let field = |key: &str| {
                head.lines()
                    .filter_map(|l| l.split_once(':'))
                    .find(|(k, _)| k.trim() == key)
                    .map(|(_, v)| v.trim().to_string())
            };
            let tags = field("tags").unwrap_or_default();
            let tags: Vec<_> = tags.split_whitespace().collect();
            if !["vct", "proven", "quick"].iter().all(|t| tags.contains(t)) {
                continue;
            }
            cases.push(Case {
                name: path.file_stem().unwrap().to_string_lossy().into_owned(),
                board: board.trim().parse().unwrap(),
                attacker: field("attacker").unwrap().parse().unwrap(),
                limit: field("limit").unwrap().parse().unwrap(),
                threat_limit: field("threat_limit").map_or(1, |v| v.parse().unwrap()),
            });
        }
        assert!(cases.len() > 20, "{}", cases.len());
        cases
    }

    fn new_solver(case: &Case) -> DFPNSVCTSolver {
        DFPNSVCTSolver::new(case.threat_limit, crate::mate::DEFAULT_DEFENDER_VCF_DEPTH)
    }

    /// The proof of each case, read off the tables, proves the case; and
    /// with one more stone of the defender's outside its zone, at every
    /// point, it still does. Where `proves` holds inside the zone, a new
    /// search finds a VCT too (for the first few such points of each case,
    /// within 100,000 nodes: the search is what `proves` saves, and it is
    /// slow).
    #[test]
    fn test_proof_tree_zone_and_proves() {
        let (mut points, mut outside, mut held_inside) = (0, 0, 0);
        for case in quick_proven_cases() {
            let mut searched = 0;
            let solver = &mut new_solver(&case);
            let state = &mut VCTState::from_board(&case.board, case.attacker, case.limit);
            let unlimited = &mut NodeBudget::unlimited();
            let mate = solver.solve(state, unlimited).expect(&case.name);
            let tree = solver
                .proof_tree(state, &mut NodeBudget::unlimited())
                .unwrap_or_else(|| panic!("{}: no proof tree", case.name));
            assert_eq!(
                tree.first_move(),
                mate.path.first().copied(),
                "{}",
                case.name
            );
            assert!(
                solver.proves(state, &tree, &mut NodeBudget::unlimited()),
                "{}: the proof does not prove its own position",
                case.name
            );
            let defender = case.attacker.opponent();
            for p in case.board.empty_points() {
                if defender.is_black() && case.board.forbidden(p).is_some() {
                    continue;
                }
                let board = case.board.with_stone(defender, p);
                let state = &mut VCTState::from_board(&board, case.attacker, case.limit);
                let holds = solver.proves(state, &tree, &mut NodeBudget::unlimited());
                points += 1;
                if !tree.zone().contains(p) {
                    outside += 1;
                    assert!(
                        holds,
                        "{}: {p} is outside the zone but breaks it",
                        case.name
                    );
                } else if holds {
                    held_inside += 1;
                    if searched == 3 {
                        continue;
                    }
                    searched += 1;
                    let fresh = &mut new_solver(&case);
                    let budget = &mut NodeBudget::new(100_000);
                    let found = fresh.solve(state, budget);
                    assert!(
                        found.is_some() || budget.is_exhausted(),
                        "{}: proves after {p}, no VCT",
                        case.name
                    );
                }
            }
        }
        // The zone rules a good part of the board out (37% of the points
        // when written), and `proves` some more.
        assert!(outside * 4 > points, "{outside} of {points}");
        assert!(held_inside > 0);
    }

    /// `proves` turns a proof down that a stone on its path breaks.
    #[test]
    fn test_a_stone_on_the_line_breaks_it() {
        for case in quick_proven_cases().into_iter().take(10) {
            let solver = &mut new_solver(&case);
            let state = &mut VCTState::from_board(&case.board, case.attacker, case.limit);
            let mate = solver
                .solve(state, &mut NodeBudget::unlimited())
                .expect(&case.name);
            let tree = solver
                .proof_tree(state, &mut NodeBudget::unlimited())
                .unwrap();
            let first = mate.path[0];
            assert!(tree.zone().contains(first));
            let board = case.board.with_stone(case.attacker.opponent(), first);
            let state = &mut VCTState::from_board(&board, case.attacker, case.limit);
            assert!(
                !solver.proves(state, &tree, &mut NodeBudget::unlimited()),
                "{}",
                case.name
            );
        }
    }
}
