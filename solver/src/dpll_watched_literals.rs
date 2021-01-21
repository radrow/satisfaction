use crate::cnf;

use cnf::CNF;
use cnf::CNFClause;
use cnf::CNFVar;
use std::{collections::VecDeque, env::var};
use crate::{Solver, SATSolution};

pub struct SatisfactionSolverV2; 

impl Solver for SatisfactionSolverV2 {
    fn solve(&self, formula: CNF) -> SATSolution {
        let mut data = DataStructures::new(formula);
        data.dpll()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AssignmentType {
    Forced, Branching
}


/// Used to store assignments made in the past, for undoing them with backtracking
struct PrevAssignment {
    variable: usize,
    assignment_type: AssignmentType
}


#[derive(Clone, Debug, PartialEq, Eq, Copy)]
enum VarValue {
    Pos, Neg, Free
}

impl std::ops::Neg for VarValue {
    type Output = VarValue;

    fn neg(self) -> Self::Output {
        match self {
            VarValue::Pos => VarValue::Neg,
            VarValue::Neg => VarValue::Pos,
            VarValue::Free => VarValue::Free,
        }
    }
}

impl From<bool> for VarValue {
    fn from(sign: bool) -> Self {
        match sign {
            true => VarValue::Pos,
            false => VarValue::Neg,
        }
    }
}

pub struct Variable {
    value: VarValue, 
    pos_watched_occ: Vec<usize>,
    neg_watched_occ: Vec<usize>,
}

pub struct Clause {
    literals: Vec<CNFVar>,
    watched1: usize,
    watched2: usize
}

pub type Variables = Vec<Variable>;
pub type Clauses = Vec<Clause>;

impl Variable {
    fn new(cnf: &CNF, var_num: usize) -> Variable {
        let mut v = Variable {
            value: VarValue::Neg,
            pos_watched_occ: cnf.clauses.iter()
                .enumerate()
                .filter_map(|(i, c)| { 
                    if (c.vars[0].id == var_num && c.vars[0].sign) || (c.vars[c.vars.len()-1].id == var_num && c.vars[c.vars.len()-1].sign) {
                        return Some(i);
                    } else {
                        None
                    }
                }).collect(),
            neg_watched_occ: cnf.clauses.iter()
                .enumerate()
                .filter_map(|(i, c)| { 
                    if (c.vars[0].id == var_num && !c.vars[0].sign) || (c.vars[c.vars.len()-1].id == var_num && !c.vars[c.vars.len()-1].sign) {
                        return Some(i);
                    } else {
                        None
                    }
                }).collect()
        };
        // check if variable is used
        for i in 0..cnf.clauses.len() {
            for u in 0..cnf.clauses[i].vars.len() {
                if cnf.clauses[i].vars[u].id == var_num {
                    v.value = VarValue::Free;
                }
            }
        }
        v
    }
}

impl Clause {
    fn new(cnf_clause: &CNFClause) -> Clause {
        // remove douplicated variables for active_lit, because they count as only 1 active literal
        let mut cnf_variables = cnf_clause.vars.clone();
        cnf_variables.sort();
        cnf_variables.dedup();
        cnf_variables.iter_mut()
            .for_each(|var| var.id -= 1);

        let last_elem = cnf_variables.len() - 1;
        Clause {
            literals: cnf_variables,
            watched1: 0,
            watched2: last_elem
        }
    }
}

struct DataStructures {
    variables: Vec<Variable>,
    clauses: Vec<Clause>,
    unit_queue: VecDeque<CNFVar>,
    assignment_stack: Vec<PrevAssignment>,
}

impl DataStructures {
    fn new(cnf: CNF) -> DataStructures {
        let clauses: Vec<Clause> = cnf.clauses.iter().map(|cnf_clause| Clause::new(&cnf_clause)).collect();
        let variables = (1..=cnf.num_variables).map(|i| Variable::new(&cnf, i)).collect();
        let unit_queue = VecDeque::with_capacity(cnf.num_variables);
        let assignment_stack = Vec::with_capacity(cnf.num_variables);

        DataStructures {
            variables,
            clauses,
            unit_queue,
            assignment_stack,
        }
    }

    fn dpll(&mut self) -> SATSolution {
        // unit propagation
        if !self.inital_unit_propagation() {
            return SATSolution::Unsatisfiable;
        }

        // repeat & choose literal b 
        while let Some(i) = self.pick_branching_variable() {
            // set value b
            let mut conflict = !self.set_variable(i.id, AssignmentType::Branching, i.sign.into());

            // unit propagation
            if !conflict {
                conflict = !self.unit_propagation();
            }

            if conflict == true {
                if self.backtracking() == false {
                    return SATSolution::Unsatisfiable;
                }
            }
        }

        // output assignment
        self.variables.iter().map(|x| match x.value {
            VarValue::Pos => true,
            VarValue::Neg => false,
            _ => false
        }).collect()
    }

    fn pick_branching_variable(&mut self) -> Option<CNFVar> {
        // TODO -> add heuristics to chose Variables
        self.variables.iter()
            .enumerate()
            .filter_map(|(i,v)| match v.value {
                VarValue::Free  => Some(CNFVar::new(i, true)),
                _               => None,
            }).next()
    }

    fn inital_unit_propagation(&mut self) -> bool {
        // find all unit clauses and enqueue the variables in the queue
        for i in 0..self.clauses.len() {
            if self.clauses[i].literals.len() == 1 {
                let unit_literal = self.find_unit_variable(i);
                if !self.unit_queue.contains(&unit_literal) {
                    self.unit_queue.push_back(unit_literal);
                }
            }
        }
        
        self.unit_propagation() 
    }

    
    fn watch_literal_check(&mut self, i: usize, sign: VarValue, watched_occ: Vec<usize>) -> (bool, VecDeque<usize>) {
        let mut remove_queue: VecDeque<usize> = VecDeque::new();
        for index_watched_clauses in 0..watched_occ.len() {
            let clause_index = watched_occ[index_watched_clauses];
            let mut found = false;
            let watched1_index = self.clauses[clause_index].watched1;
            let watched2_index = self.clauses[clause_index].watched2;
            let watched1_var = self.clauses[clause_index].literals[watched1_index];
            let watched2_var = self.clauses[clause_index].literals[watched2_index];

            for lit_index in 0..self.clauses[clause_index].literals.len() {
                let lit: CNFVar = self.clauses[clause_index].literals[lit_index];
                if (self.variables[lit.id].value == VarValue::Pos && lit.sign) || (self.variables[lit.id].value == VarValue::Neg && !lit.sign) {
                    // set as sat
                    found = true;
                    break;
                } else if self.variables[lit.id].value == VarValue::Free && self.clauses[clause_index].literals[watched1_index] != lit && self.clauses[clause_index].literals[watched2_index] != lit {
                    // check if found variable is already watched
                    found = true;
                    
                    // add variable as watched in clause
                    if self.clauses[clause_index].literals[self.clauses[clause_index].watched1] == (CNFVar {id: lit_index, sign: sign == VarValue::Pos}) {
                        self.clauses[clause_index].watched2 = lit_index;
                    } else {
                        self.clauses[clause_index].watched1 = lit_index;
                    }
                    // remove clause from watched occ
                    remove_queue.push_back(index_watched_clauses);
                    // add new variable to watched occ
                    if lit.sign {
                        self.variables[lit.id].pos_watched_occ.push(clause_index);
                    } else {
                        self.variables[lit.id].neg_watched_occ.push(clause_index);
                    }
                    break;
                }
            }
            if !found {
                // check if the only free variable is a watched one
                if watched1_var.id != i {
                    if self.variables[watched1_var.id].value == VarValue::Free  {
                        if !self.unit_queue.contains(&watched1_var) {
                            self.unit_queue.push_back(watched1_var);
                        }
                        return (true, remove_queue);
                    }
                } 
                if watched2_var.id != i {
                    if self.variables[watched2_var.id].value == VarValue::Free  {
                        if !self.unit_queue.contains(&watched2_var) {
                            self.unit_queue.push_back(watched2_var);
                        }
                        return (true, remove_queue);
                    }
                }
                // conflict
                return (false, remove_queue);
            } 
                
        }
        
        (true, remove_queue)
    }
    

    fn set_variable(&mut self, i: usize, assign_type: AssignmentType, sign: VarValue) -> bool {
        self.variables[i].value = sign;
        self.assignment_stack.push(PrevAssignment {variable: i, assignment_type: assign_type});

        if sign == VarValue::Pos {
            let (ret_val, mut remove_queue) = self.watch_literal_check(i, sign, self.variables[i].neg_watched_occ.clone());
            loop {match remove_queue.pop_back() {
                Some(elem) => {
                    self.variables[i].neg_watched_occ.remove(elem);
                }, 
                None => break
            }};
            return ret_val;
        } else {
            let (ret_val, mut remove_queue) = self.watch_literal_check(i, sign, self.variables[i].pos_watched_occ.clone());
            loop {match remove_queue.pop_back() {
                Some(elem) => {self.variables[i].pos_watched_occ.remove(elem);}, 
                None => break
            }};
            return ret_val;
        }
    }

    fn unit_propagation(&mut self) -> bool {
        loop {
            match self.unit_queue.pop_front() {
                Some(var) => {
                    if self.set_variable(var.id, AssignmentType::Forced, var.sign.into()) == false {
                        return false;
                    }
                },
                None => break
            }
        }
        true
    }

    fn backtracking(&mut self) -> bool {
        // empty queue
        self.unit_queue.clear();

        while let Some(assign) = self.assignment_stack.pop() {
            if assign.assignment_type == AssignmentType::Branching {
                if self.set_variable(assign.variable, AssignmentType::Forced, -self.variables[assign.variable].value) {
                    // goto unit propagation
                    if self.unit_propagation() == false {
                        return self.backtracking();
                    }
                    return true
                } else {
                    self.unit_queue.clear();
                    self.variables[assign.variable as usize].value = VarValue::Free;
                }
            }
            self.variables[assign.variable as usize].value = VarValue::Free;
        }
        // unsatisfied
        false
    }

    fn find_unit_variable(&self, clause: usize) -> CNFVar {
        self.clauses[clause].literals.iter()
            .filter(|lit| self.variables[lit.id].value == VarValue::Free)
            .next()
            .expect("The only left literal cound not be found!")
            .clone()
    }
}
