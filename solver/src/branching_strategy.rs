use std::cmp::Ordering;
use crate::{cnf, dpll};
use cnf::CNFVar;
use dpll::{Variables, Clauses, VarValue};

pub trait BranchingStrategy {
    /// Funtion that picks the next variable to be chosen for branching.
    /// Returns the index of the next variable, or None if there is no Variable to be picked
    fn pick_branching_variable(&self, variables: &Variables, clauses: &Clauses) -> Option<CNFVar>;
}

pub struct NaiveBranching;

impl BranchingStrategy for NaiveBranching {
    fn pick_branching_variable(&self, variables: &Variables, _clauses: &Clauses) -> Option<CNFVar> {
        // TODO -> add heuristics to chose Variables
        variables.iter()
            .enumerate()
            .filter_map(|(i,v)| match v.value {
                VarValue::Free  => Some(CNFVar::new(i, true)),
                _               => None,
            }).next()
    }
}

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
                let pos = v.pos_occ.iter().filter(|clause| clauses[**clause].satisfied.is_none()).count();
                let neg = v.neg_occ.iter().filter(|clause| clauses[**clause].satisfied.is_none()).count();
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

pub struct DLCS;

impl BranchingStrategy for DLCS {
    fn pick_branching_variable(&self, variables: &Variables, _clauses: &Clauses) -> Option<CNFVar> {
        let mut max = 0;
        let mut cnf_var: Option<CNFVar> = None;
        for (i, v) in variables.iter().enumerate() {
            if v.value == VarValue::Free {
                let h = v.neg_occ.len() + v.pos_occ.len();
                let local_cnf_var = CNFVar {id: i, sign: v.pos_occ.len() > v.neg_occ.len()};
                if h > max {
                    max = h;
                    cnf_var = Some(local_cnf_var);
                }
            }
        }
        cnf_var
    }
}

pub struct JeroslawWang;

impl BranchingStrategy for JeroslawWang {
    fn pick_branching_variable(&self, variables: &Variables, clauses: &Clauses) -> Option<CNFVar> {
        variables.iter()
            .enumerate()
            .filter(|(_, var)| var.value == VarValue::Free)
            .map(|(id, var)| {
                let pos = JeroslawWang::compute(&var.pos_occ, clauses);
                let neg = JeroslawWang::compute(&var.neg_occ, clauses);
                if pos < neg { (neg, false, id) }
                else { (pos, true, id) }
            }).max_by(|(v1, _, _), (v2, _, _)| v1.partial_cmp(v2).unwrap_or(Ordering::Equal))
                .map(|(_, sign, id)| CNFVar::new(id, sign))
    }
}

impl JeroslawWang {
    #[inline]
    fn literal_measure(w: usize) -> f32 {
        2f32.powf(-(w as f32))
    }

    #[inline]
    fn compute(occ: &Vec<usize>, clauses: &Clauses) -> f32 {
        occ.iter()
            /*.map(|clause_index| {
                JeroslawWang::literal_measure(clauses[*clause_index].literals.len())*/
            .filter_map(|clause_index| {
                let clause = &clauses[*clause_index];
                if clause.satisfied.is_none() {
                    Some(JeroslawWang::literal_measure(clause.active_lits))
                } else {
                    None
                }
            }).sum()
    }
}

pub struct MOM;
impl BranchingStrategy for MOM {
    fn pick_branching_variable(&self, variables: &Variables, clauses: &Clauses) -> Option<CNFVar> {
        let min_clause_width =
            clauses.iter().map(|c| (*c).literals.len()).min()?;
        variables.iter()
            .enumerate()
            .max_by_key(
                |(_, v)| {
                    let hp =
                        v.pos_occ.iter().
                        filter(|c| clauses[**c].literals.len() == min_clause_width)
                        .count();
                    let hn =
                        v.neg_occ.iter().
                        filter(|c| clauses[**c].literals.len() == min_clause_width)
                        .count();

                    ((hp + hn) << 32) + hp * hn
                }
            ).map(|(i, _)| CNFVar::pos(i))
    }
}

/// A variation of MOM where we measure width counting only active literals
pub struct ActiveMOM;
impl BranchingStrategy for ActiveMOM {
    fn pick_branching_variable(&self, variables: &Variables, clauses: &Clauses) -> Option<CNFVar> {
        let min_clause_width =
            clauses.iter().map(|c| (*c).active_lits).min()?;
        variables.iter()
            .enumerate()
            .max_by_key(
                |(_, v)| {
                    let hp =
                        v.pos_occ.iter().
                        filter(|c| clauses[**c].literals.len() == min_clause_width)
                        .count();
                    let hn =
                        v.neg_occ.iter().
                        filter(|c| clauses[**c].literals.len() == min_clause_width)
                        .count();

                    ((hp + hn) << 32) + hp * hn
                }
            ).map(|(i, _)| CNFVar::pos(i))
    }
}
