use std::cmp::Ordering;
use crate::{cnf, dpll};
use cnf::CNFVar;
use dpll::{Variables, Clauses, VarValue};
use auto_impl::auto_impl;

#[auto_impl(&, Box)]
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

#[inline]
fn count_number_of_clauses(occ: &Vec<usize>, clauses: &Clauses) -> usize {
    occ.iter()
        .filter(|clause| clauses[**clause].satisfied.is_none())
        .count()
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
                let h = pos+neg;
                if h > max {
                    max = h;
                    cnf_var = Some(CNFVar::new(i, pos > neg));
                }
            }
        }
        cnf_var
    }
}

pub struct JeroslawWang;

impl BranchingStrategy for JeroslawWang {
    fn pick_branching_variable(&self, variables: &Variables, clauses: &Clauses) -> Option<CNFVar> {
        let values: Vec<_> = clauses.iter()
            .map(|clause| if clause.satisfied.is_none() {
                Some(JeroslawWang::literal_measure(clause.active_lits))
            } else {
                None
            })
            .collect();

        variables.iter()
            .enumerate()
            .filter(|(_, var)| var.value == VarValue::Free)
            .map(|(id, var)| {
                let pos = JeroslawWang::compute(&var.pos_occ, &values);
                let neg = JeroslawWang::compute(&var.neg_occ, &values);
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
    fn compute(occ: &Vec<usize>, values: &[Option<f32>]) -> f32 {
        occ.iter()
            .filter_map(|clause_index| {
                unsafe { *values.get_unchecked(*clause_index) }
            }).sum()
    }
}

pub struct MOM;

impl BranchingStrategy for MOM {
    fn pick_branching_variable(&self, variables: &Variables, clauses: &Clauses) -> Option<CNFVar> {
        let min_clause_width = usize::max(2,
            clauses.iter()
            .map(|c| c.active_lits)
            .min()?);

        variables.iter()
            .enumerate()
            .filter(|(_, var)| var.value == VarValue::Free)
            .max_by_key(
                |(_, v)| {
                    let hp =
                        v.pos_occ.iter()
                            .filter(|c| clauses[**c].active_lits == min_clause_width)
                            .count();
                    let hn =
                        v.neg_occ.iter()
                            .filter(|c| clauses[**c].active_lits == min_clause_width)
                            .count();

                    (hp + hn) * clauses.len()^2 + hp * hn
                }
            ).map(|(i, _)| CNFVar::pos(i))
    }
}
