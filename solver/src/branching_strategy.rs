use crate::{cnf, dpll};
use auto_impl::auto_impl;
use cnf::{VarId, CNFVar};
use dpll::{Clauses, Clause, VarValue, Variables, Variable};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::linked_list::{LinkedList, Cursor, CursorMut};

/// A trait to choose the next variable to set during DPLL algorithm.
/// Often a variable must have a specific boolean value
/// because otherwise the whole formula would be unsatisfied.
/// If this not the case anymore,
/// i.e. all variables can be choosen arbitrarily,
/// one have to decide which literal should be set true next.
/// To do so, one can consider several heuristics deciding w.r.t. the current state of the algorithm
/// in order to achieve a fast convergence and thus improve runtime significantly.
/// This traits abstracts these branching heuristics by allowing them to have a look at the
/// current variables to suggest a promising literal to be set true.
/// If all variables are set, the heuristic is supposed to return None.
#[auto_impl(&, Box)]
pub trait BranchingStrategy {
    /// Funtion that picks the next variable to be chosen for branching.
    /// Returns the index of the next variable, or None if there is no Variable to be picked
    fn pick_branching_variable(&self, variables: &Variables, clauses: &Clauses) -> Option<CNFVar>;
}

/// A branching heuristic that chooses just the next unset variable
/// according to the order in `variables`.
/// Thus no information about clauses are taken into account.
pub struct NaiveBranching;

impl BranchingStrategy for NaiveBranching {
    fn pick_branching_variable(&self, variables: &Variables, _clauses: &Clauses) -> Option<CNFVar> {
        variables
            .iter()
            .enumerate()
            // Only consider unset variables
            .filter_map(|(i, v)| match v.value {
                VarValue::Free => Some(CNFVar::new(i, true)),
                _ => None,
            })
            .next() // Take the first one
    }
}

impl crate::cdcl::Initialisation for NaiveBranching {
    fn initialise(_clauses: &crate::cdcl::Clauses, _variables: &crate::cdcl::Variables) -> Self where Self: Sized {
        NaiveBranching
    }
}

impl crate::cdcl::Update for NaiveBranching {}

impl crate::cdcl::BranchingStrategy for NaiveBranching {
    fn pick_literal(&mut self, _clauses: &crate::cdcl::Clauses, variables: &crate::cdcl::Variables) -> Option<CNFVar> {
        variables
            .iter()
            .enumerate()
            .find_map(|(id, var)| 
                if var.assignment.is_some() { None }
                else { Some(CNFVar::new(id, true)) })
    }
}

/// A small convenience function counting the number of clauses that are mentioned in occ and not
/// satisfied at the moment.
#[inline]
fn count_number_of_clauses(occ: &Vec<usize>, clauses: &Clauses) -> usize {
    occ.iter()
        .filter(|clause| clauses[**clause].satisfied.is_none())
        .count()
}

/// Dynamic Largest Individual Sum (DLIS) is a branching heuristics that chooses the literal
/// appearing most in satisfied clauses, thus reducing the clauses to be considered.
pub struct DLIS;

impl BranchingStrategy for DLIS {
    fn pick_branching_variable(&self, variables: &Variables, clauses: &Clauses) -> Option<CNFVar> {
        let mut max = 0;
        let mut cnf_var: Option<CNFVar> = None;
        for (i, v) in variables.iter().enumerate() {
            if v.value == VarValue::Free {
                /*let mut local_max = v.pos_occ.len();
                let mut local_cnf_var = CNFVar {id: i, sign: true};
                if v.pos_occ.len() < v.neg_occ.len() {
                    local_max = v.neg_occ.len();
                    local_cnf_var.sign = false;
                }
                if local_max > max {
                    max = local_max;
                    cnf_var = Some(local_cnf_var);
                }*/
                let pos = count_number_of_clauses(&v.pos_occ, clauses);
                let neg = count_number_of_clauses(&v.neg_occ, clauses);
                let (sign, local_max) = if pos > neg { (true, pos) } else { (false, neg) };
                if local_max > max {
                    max = local_max;
                    cnf_var = Some(CNFVar::new(i, sign));
                }
            }
        }
        cnf_var
    }
}

/// Dynamic Largest Combined Sum (DLIS) is a branching heuristics that chooses the variable
/// appearing most in satisfied clauses, and then decides if it is set to true or false depending
/// of the number of occurences.
pub struct DLCS;

impl BranchingStrategy for DLCS {
    fn pick_branching_variable(&self, variables: &Variables, clauses: &Clauses) -> Option<CNFVar> {
        let mut max = 0;
        let mut cnf_var: Option<CNFVar> = None;
        for (i, v) in variables.iter().enumerate() {
            if v.value == VarValue::Free {
                /*let h = v.neg_occ.len() + v.pos_occ.len();
                let local_cnf_var = CNFVar {id: i, sign: v.pos_occ.len() > v.neg_occ.len()};
                if h > max {
                    max = h;
                    cnf_var = Some(local_cnf_var);
                }*/
                let pos = count_number_of_clauses(&v.pos_occ, clauses);
                let neg = count_number_of_clauses(&v.neg_occ, clauses);
                let h = pos + neg;
                if h > max {
                    max = h;
                    cnf_var = Some(CNFVar::new(i, pos > neg));
                }
            }
        }
        cnf_var
    }
}

/// A branching heuristics that chooses the literal appearing most in the formula
/// prefering (i.e. weighting) short clauses.
pub struct JeroslawWang;

impl BranchingStrategy for JeroslawWang {
    fn pick_branching_variable(&self, variables: &Variables, clauses: &Clauses) -> Option<CNFVar> {
        let values: Vec<_> = clauses
            .iter()
            .map(|clause| {
                if clause.satisfied.is_none() {
                    Some(JeroslawWang::literal_measure(clause.active_lits))
                } else {
                    None
                }
            })
            .collect();

        variables
            .iter()
            .enumerate()
            .filter(|(_, var)| var.value == VarValue::Free)
            .map(|(id, var)| {
                let pos = JeroslawWang::compute(&var.pos_occ, &values);
                let neg = JeroslawWang::compute(&var.neg_occ, &values);
                if pos < neg {
                    (neg, false, id)
                } else {
                    (pos, true, id)
                }
            })
            .max_by(|(v1, _, _), (v2, _, _)| v1.partial_cmp(v2).unwrap_or(Ordering::Equal))
            .map(|(_, sign, id)| CNFVar::new(id, sign))
    }
}

impl JeroslawWang {
    #[inline]
    fn literal_measure(w: usize) -> f32 {
        2f32.powf(-(w as f32))
    }

    #[inline]
    fn compute(occ: &Vec<usize>, values: &[Option<f32>]) -> f32 {
        occ.iter()
            .filter_map(|clause_index| unsafe { *values.get_unchecked(*clause_index) })
            .sum()
    }
}

/// Maximum number of Occurrences in the Minimum length clauses (MOM)
/// is a branching heuristic that chooses the literal that appears most in the shortest clauses.
pub struct MOM;

impl BranchingStrategy for MOM {
    fn pick_branching_variable(&self, variables: &Variables, clauses: &Clauses) -> Option<CNFVar> {
        let min_clause_width = usize::max(2, clauses.iter().map(|c| c.active_lits).min()?);

        variables
            .iter()
            .enumerate()
            .filter(|(_, var)| var.value == VarValue::Free)
            .max_by_key(|(_, v)| {
                let hp = v
                    .pos_occ
                    .iter()
                    .filter(|c| clauses[**c].active_lits == min_clause_width)
                    .count();
                let hn = v
                    .neg_occ
                    .iter()
                    .filter(|c| clauses[**c].active_lits == min_clause_width)
                    .count();

                (hp + hn) * clauses.len() ^ 2 + hp * hn
            })
            .map(|(i, _)| CNFVar::pos(i))
    }
}

#[auto_impl(&mut, Box)]
pub trait CDCLBranchingStrategy {
    fn pick_branching_variable(&mut self,
                               variables: &Variables,
                               clauses: &Clauses
    ) -> Option<CNFVar>;

    fn learn_clause_hook(&mut self, clause: &Clause);

    fn branch_hook(&mut self);

    fn assign_var_hook(&mut self, id: VarId);

    fn unassign_var_hook(&mut self, id: VarId);
}

pub struct VSIDS {
    resort_period: u32,
    search_index: usize,
    branchings: u32,
    scores: Vec<u32>,
    counters: Vec<u32>,
    queue: Vec<VarId>,
    coqueue: Vec<usize>
}

impl VSIDS {
    pub fn new(resort_period: u32, vars: &Variables, clauses: &Clauses) -> VSIDS {
        let mut scores: Vec<u32> = (0..vars.len() as u32).collect();
        let counters = std::iter::repeat(0).take(vars.len()).collect();

        for (i, v) in vars.iter().enumerate() {
            let hp = v.pos_occ.len();

            scores[i] = hp as u32;
        }

        let mut queue: Vec<VarId> = vars.iter().map(|v| v.index).collect();
        queue.sort_by_key(|x| scores[*x]);
        let mut coqueue: Vec<usize> = (0..vars.len()).collect();
        for i in 0..queue.len() {
            coqueue[queue[i]] = i;
        }

        VSIDS {
            resort_period: resort_period,
            search_index: vars.len() - 1,
            branchings: 0,
            scores: scores,
            counters: counters,
            queue: queue,
            coqueue: coqueue,
        }
    }
}

impl CDCLBranchingStrategy for VSIDS {
    fn pick_branching_variable(&mut self, variables: &Variables, _clauses: &Clauses) -> Option<CNFVar> {
        while self.search_index < self.queue.len() { // Actually, > 0 but overflow
            if variables[self.queue[self.search_index]].value == VarValue::Free {
                self.search_index -= 1;
                return Some(CNFVar::pos(self.queue[self.search_index]))
            }
            self.search_index -= 1;
        }
        None
    }

    fn learn_clause_hook(&mut self, clause: &Clause) {
        for v in &clause.literals {
            self.counters[v.id] += 1;
        }
    }

    fn branch_hook(&mut self) {
        self.branchings += 1;
        if self.branchings == self.resort_period {
            self.branchings = 0;

            for i in 0..self.queue.len() {
                self.scores[i] /= 2;
                self.scores[i] += self.counters[i];
                self.counters[i] = 0;
            }

            let scores = &self.scores;
            self.queue.sort_by_key(|x| scores[*x]);
            for i in 0..self.queue.len() {
                self.coqueue[self.queue[i]] = i;
            }
            self.search_index = self.queue.len() - 1;
        }
    }

    fn assign_var_hook(&mut self, _var: VarId) {}

    fn unassign_var_hook(&mut self, var: VarId) {
        if self.search_index < self.coqueue[var] {
            self.search_index = self.coqueue[var]
        }
    }
}

// http://fmv.jku.at/papers/BiereFroehlich-SAT15.pdf
pub struct VMTF<'a> {
    scores: HashMap<VarId, usize>,
    queue: LinkedList<Variable>,
    last_search: CursorMut<'a, Variable>,
    enqueue_count: u32,
}

impl VMTF<'_> {

}
