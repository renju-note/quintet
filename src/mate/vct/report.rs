use super::generator::Candidate;
use super::proof::PnDn;
use crate::board::Point;

/// What the last search of a [`VCTSolver`] knew about its root and each of
/// the root's moves when it ended, whether or not it decided the root:
/// [`VCTSolver::root_report`].
///
/// The root is the state handed to the search. With the attacker to move
/// (an OR node) its moves are the attacks the search generated; with the
/// defender to move (an AND node, [`VCTState::defending`]) they are the
/// defences against the attacker's threat. Either way every number is about
/// the attacker's VCT: a proven move is one after which the attacker wins.
///
/// - A point that is not among [`Self::moves`] was never a candidate. At an
///   OR root it is no threat, or one the search ruled out, so the attacker
///   does not win by it. At an AND root it does not stop the attacker's
///   threat, so the attacker wins after it.
/// - Proof and disproof numbers mean what the solver's threshold policy
///   makes them mean: with df-pn ([`DFPNSVCTSolver`]) the search keeps the
///   siblings' proof numbers close, so the disproof numbers and the nodes
///   spent under each move tell them apart more than the proof numbers do.
///
/// [`VCTSolver`]: super::VCTSolver
/// [`VCTSolver::root_report`]: super::VCTSolver::root_report
/// [`VCTState::defending`]: super::VCTState::defending
/// [`DFPNSVCTSolver`]: super::DFPNSVCTSolver
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RootReport {
    /// Whether the attacker was to move at the root.
    pub attacking: bool,
    /// The root's numbers when the search ended: `pn == 0` proven, `dn == 0`
    /// disproven, anything else open.
    pub root: PnDn,
    /// The nodes the search used, as its budget counts them.
    pub nodes: u64,
    /// The root's moves, in the order the search generated them. Empty when
    /// the root was decided without them: the attacker has a VCF, the
    /// defender has made a five or has a VCF, there is nothing to try, or the
    /// limit is spent.
    pub moves: Vec<RootMove>,
}

/// One of the root's moves in a [`RootReport`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RootMove {
    pub point: Point,
    /// The number the search starts the move from while it knows nothing
    /// of it: a proof number for an attack, a disproof number for a
    /// defence. Smaller is tried first.
    pub estimate: u32,
    pub status: MoveStatus,
    /// The nodes the search spent under this move, its nested VCF searches
    /// included. 0 for a move it took from the tables or never searched.
    pub nodes: u64,
}

/// Where the attacker's VCT stood after one of the root's moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveStatus {
    /// Never searched, and nothing in the tables: only its
    /// [`RootMove::estimate`] is known.
    Unexpanded,
    /// Searched, or in the tables, without an answer.
    Open { pn: u32, dn: u32 },
    /// The attacker wins after it.
    Proven,
    /// The attacker does not win after it, within the limit.
    Disproven,
    /// A forbidden move of Black's, attacking: never played.
    Forbidden,
}

impl RootReport {
    /// What the report says about `p`; `None` if it was not one of the
    /// root's moves (see [`RootReport`]).
    pub fn get(&self, p: Point) -> Option<&RootMove> {
        self.moves.iter().find(|m| m.point == p)
    }

    /// Starts the report of a search from the root, before anything is
    /// known.
    pub(super) fn start(&mut self, attacking: bool) {
        self.attacking = attacking;
        self.root = PnDn::unknown();
        self.nodes = 0;
        self.moves.clear();
    }

    /// Records the root's expansion as it ended: its candidates, the
    /// children's numbers (`None` for one neither searched nor in the
    /// tables) and the nodes under each. `forbidden` tells a forbidden
    /// attack from the other disproven ones; it is asked only of those.
    pub(super) fn record(
        &mut self,
        candidates: &[Candidate],
        children: &[Option<PnDn>],
        mut forbidden: impl FnMut(Point) -> bool,
        nodes: &[u64],
    ) {
        self.moves.clear();
        self.moves
            .extend(
                candidates
                    .iter()
                    .zip(children)
                    .zip(nodes)
                    .map(|((c, child), &nodes)| RootMove {
                        point: c.point,
                        estimate: c.estimate,
                        status: MoveStatus::of(*child, || forbidden(c.point)),
                        nodes,
                    }),
            );
    }
}

impl MoveStatus {
    fn of(child: Option<PnDn>, forbidden: impl FnOnce() -> bool) -> Self {
        match child {
            None => Self::Unexpanded,
            Some(n) if n.pn == 0 => Self::Proven,
            Some(n) if n.dn == 0 && forbidden() => Self::Forbidden,
            Some(n) if n.dn == 0 => Self::Disproven,
            Some(n) => Self::Open { pn: n.pn, dn: n.dn },
        }
    }
}

impl Default for PnDn {
    fn default() -> Self {
        Self::unknown()
    }
}
