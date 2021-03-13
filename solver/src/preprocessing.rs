use crate::{CNF, CNFClause, CNFVar};
use core::num;
use std::{collections::{HashSet}};

pub fn preprocessing(cnf: &mut CNF) {
    non_incr_var_elim_resolution(cnf);
    //equivalent_substitution(cnf);

}

/// function to remove the case (a v !b) and (!a v b)
pub fn equivalent_substitution(cnf: &mut CNF) {
    // only look at clauses that are relevant and have size of 2
    let small_clauses: Vec<(usize, &CNFClause)> = cnf.clauses.iter().enumerate().filter(|(i, clause)| clause.len() == 2).collect();
    let mut remove_indices: HashSet<usize> = HashSet::new();
    
    // compare each small clause with the other small clauses if they can be removed
    for orign_clause in &small_clauses {
        for comp_clause in &small_clauses {
            if (orign_clause.1.vars[0].id == comp_clause.1.vars[0].id && orign_clause.1.vars[0].sign != comp_clause.1.vars[0].sign)
                || (orign_clause.1.vars[0].id == comp_clause.1.vars[1].id && orign_clause.1.vars[0].sign != comp_clause.1.vars[1].sign) {

                if (orign_clause.1.vars[1].id == comp_clause.1.vars[0].id && orign_clause.1.vars[1].sign != comp_clause.1.vars[0].sign)
                    || (orign_clause.1.vars[1].id == comp_clause.1.vars[1].id && orign_clause.1.vars[1].sign != comp_clause.1.vars[1].sign){

                    // add to the clauses that can be removed
                    remove_indices.insert(orign_clause.0);
                    remove_indices.insert(comp_clause.0);
                }
            }
        }
    }
    // dont remove clauses if all the clauses would get lost
    if remove_indices.len() != cnf.clauses.len() {

        // remove the unnessersary clauses
        cnf.clauses = cnf.clauses.iter().enumerate().filter_map(|(index, clause)| {
            if remove_indices.contains(&index) {
                return None;
            }
            return Some(clause.clone());
        }).collect();
    }
}

fn sig(clause: CNFClause, num_vars: usize) -> usize {
    let mut mask = 1;
    let mut result = 0;
    for i in 0..num_vars {
        if clause.vars.contains(&CNFVar {id: i, sign: true}) || clause.vars.contains(&CNFVar {id: i, sign: false}) {
            result = result | mask;
        }
        mask = mask << 1;
    }
    result
}

fn subsumtion_test(clause1: CNFClause, clause2: CNFClause, num_vars: usize) -> bool {
    if sig(clause1, num_vars) & !sig(clause2, num_vars) != 0 {
        return false;
    }
    true
}

fn resolution(clause1: CNFClause, clause2: CNFClause, resolution_variable: usize) -> CNFClause {
    let mut new_clause1: Vec<CNFVar> = clause1.vars.into_iter().filter(|var| {var.id != resolution_variable}).collect();
    let mut new_clause2: Vec<CNFVar> = clause2.vars.into_iter().filter(|var| {var.id != resolution_variable}).collect();
    new_clause1.append(&mut new_clause2);
    CNFClause{vars: new_clause1}
}

fn is_tautology(clause: &CNFClause) -> bool {
    for variable in &clause.vars {
        if clause.vars.contains(&-(*variable)) {
            return true;
        } 
    }
    false
}

fn remove_dupl_variables(clauses: &mut Vec<CNFClause>) {
    for clause in clauses {
        clause.vars.sort();
        clause.vars.dedup();
    }
}

fn non_incr_var_elim_resolution(cnf: &mut CNF) {
    let mut pos_occ: Vec<CNFClause> = Vec::new();
    let mut neg_occ: Vec<CNFClause> = Vec::new();
    let mut no_occ: Vec<CNFClause> = Vec::new();
    let mut new_combined: Vec<CNFClause> = Vec::new();
    let mut old_clauses: Vec<CNFClause> = cnf.clauses.clone();

    let mut change: bool = true;

    // iterate until no variables can be changed anymore
    // (a v x) and (b v !x)
    while change {
        change = false;
        for variable_number in 1..=cnf.num_variables {
            // decomposite the formula in 3 parts
            pos_occ.clear();
            neg_occ.clear();
            no_occ.clear();
            for clause in &old_clauses {
                if clause.vars.contains(&CNFVar{id: variable_number, sign: true}) {
                    pos_occ.push(clause.clone());
                } else if clause.vars.contains(&CNFVar{id: variable_number, sign: false}) {
                    neg_occ.push(clause.clone());
                } else {
                    no_occ.push(clause.clone());
                }
            }

            new_combined = Vec::new();

            // all possbile combinations
            if pos_occ.len() > 0 && neg_occ.len() > 0 {
                for pos_clause in &pos_occ {
                    for neg_clause in &neg_occ {
                        new_combined.push(
                            resolution(pos_clause.clone(), neg_clause.clone(), variable_number)
                        );
                    }
                }
            
                // filter tautologies
                new_combined = new_combined.into_iter().filter(|clause| {!is_tautology(clause)}).collect();
                new_combined.append(&mut no_occ);
                remove_dupl_variables(&mut new_combined);
            }

            if new_combined.len() != 0 && new_combined.len() < old_clauses.len() {
                old_clauses = new_combined;
                change = true;
            } 
        }
    }

    cnf.clauses = old_clauses;
}