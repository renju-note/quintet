use super::searcher::Selection;
use crate::mate::vct::proof::*;

/// How far a child may be searched before control returns to its parent.
///
/// This is the only thing that differs between the DFS, PNS and df-pn VCT
/// solvers. `expand_attacks` / `expand_defences` in `searcher.rs` call these
/// with the parent's current threshold and the result of selecting the
/// most-proving child.
pub trait ThresholdPolicy {
    fn next_threshold_attack(selection: &Selection, threshold: PnDn) -> PnDn;
    fn next_threshold_defence(selection: &Selection, threshold: PnDn) -> PnDn;
}

/// No threshold: the chosen child is searched to completion before the parent
/// looks at the next one. Proof numbers are used only for move ordering.
pub struct DFSThreshold;

impl ThresholdPolicy for DFSThreshold {
    fn next_threshold_attack(_selection: &Selection, _threshold: PnDn) -> PnDn {
        PnDn::no_threshold()
    }

    fn next_threshold_defence(_selection: &Selection, _threshold: PnDn) -> PnDn {
        PnDn::no_threshold()
    }
}

/// The child returns as soon as its numbers change, so the most-proving child
/// is re-selected at every level. Emulates best-first PNS inside a recursive
/// search.
pub struct PNSThreshold;

impl ThresholdPolicy for PNSThreshold {
    fn next_threshold_attack(selection: &Selection, _threshold: PnDn) -> PnDn {
        Self::just_past_best(selection)
    }

    fn next_threshold_defence(selection: &Selection, _threshold: PnDn) -> PnDn {
        Self::just_past_best(selection)
    }
}

impl PNSThreshold {
    /// One more than the best child's numbers, on both sides alike: exceeded
    /// by any change to them.
    fn just_past_best(selection: &Selection) -> PnDn {
        let best = selection.best_child;
        PnDn::new(
            best.pn.saturating_add(1),
            best.dn.saturating_add(1),
            best.limit,
        )
    }
}

/*
Df-pn algorithm is proposed in the following paper:

Nagai, Ayumu, and Hiroshi Imai.
"Proof for the equivalence between some best-first algorithms and depth-first algorithms for AND/OR trees."
IEICE TRANSACTIONS on Information and Systems 85.10 (2002): 1645-1653.
*/

/// Df-pn thresholds: stay in the best child as long as it remains the best
/// (bounded by the second-best child, with a margin: see
/// [`Self::past_second`]), and never exceed the parent's budget.
pub struct DFPNSThreshold;

impl ThresholdPolicy for DFPNSThreshold {
    fn next_threshold_attack(selection: &Selection, threshold: PnDn) -> PnDn {
        let pn = threshold
            .pn
            .min(Self::past_second(selection.second_child.pn));
        let dn = (threshold.dn - selection.node.dn).saturating_add(selection.best_child.dn);
        PnDn::new(pn, dn, selection.best_child.limit)
    }

    fn next_threshold_defence(selection: &Selection, threshold: PnDn) -> PnDn {
        let pn = (threshold.pn - selection.node.pn).saturating_add(selection.best_child.pn);
        let dn = threshold
            .dn
            .min(Self::past_second(selection.second_child.dn));
        PnDn::new(pn, dn, selection.best_child.limit)
    }
}

impl DFPNSThreshold {
    /// What the best child may reach before the second-best one takes
    /// over: a little past it, by `1/EPSILON_INVERSE` of it, rather than
    /// just one past it.
    ///
    /// With one past, the search keeps switching between two children whose
    /// numbers are close, going back up to the parent and down again every time. The
    /// margin lets it stay a while longer in the best one: the "1 + ε
    /// trick" of Pawlewicz and Lew, "Improving Depth-First PN-Search: 1 + ε
    /// Trick" (Computers and Games 2006).
    fn past_second(second: u32) -> u32 {
        second
            .saturating_add(1)
            .saturating_add(second / EPSILON_INVERSE)
    }
}

/// ε of [`DFPNSThreshold::past_second`] is one over this. On the benchmark
/// (07) it took the whole set, `heavy` included, from 137M nodes to 99M.
/// 1/4 to 1/2 did as well or a little better on most cases, but not on
/// `vct_unstable` and `vct_small_but_long`, which swing widely with any
/// change to the search (docs/06-solver-vct.en.md §5).
const EPSILON_INVERSE: u32 = 8;
