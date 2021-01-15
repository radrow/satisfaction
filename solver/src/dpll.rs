use crate::cnf;

use cnf::CNF;
use cnf::CNFClause;
use cnf::CNFVar;
use core::panic;
use std::fmt;
use std::collections::VecDeque;
use crate::{Solver, Assignment};

pub trait BranchingStrategy: Clone {
    /// Funtion that picks the next variable to be chosen for branching.
    /// Returns the index of the next variable, or None if there is no Variable to be picked
    fn pick_branching_variable(&mut self, variables: &Variables, clauses: &Clauses) -> Option<usize>;
}

#[derive(Clone)]
pub struct NaiveBranching;

impl BranchingStrategy for NaiveBranching {
    fn pick_branching_variable(&mut self, variables: &Variables, _clauses: &Clauses) -> Option<usize> {
        // TODO -> add heuristics to chose Variables
        variables.iter()
            .enumerate()
            .filter_map(|(i,v)| match v.value {
                VarValue::Free  => Some(i),
                _               => None,
            }).next()
    }
}

pub struct SatisfactionSolver<B: BranchingStrategy> {
    strategy: B,
}

impl<B: BranchingStrategy> SatisfactionSolver<B> {
    pub fn new(strategy: B) -> SatisfactionSolver<B> {
        SatisfactionSolver {
            strategy
        }
    }
}

impl<B: BranchingStrategy> Solver for SatisfactionSolver<B> {
    fn solve(&self, formula: CNF, num_variables: usize) -> Assignment {
        let mut data = DataStructures::new(formula, num_variables);
        data.dpll(self.strategy.clone())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AssignmentType {
    Forced, Branching, Empty
}


/// Used to store assignments made in the past, for undoing them with backtracking
struct PrevAssignment {
    variable: usize,
    assignment_type: AssignmentType
}


#[derive(Clone, Debug, PartialEq, Eq)]
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
    pos_occ: Vec<usize>,
    neg_occ: Vec<usize>
}

pub struct Clause {
    active_lits: usize,
    satisfied: Option<usize>,
    literals: Vec<isize>
}

pub type Variables = Vec<Variable>;
pub type Clauses = Vec<Clause>;

impl Variable {
    fn new(cnf: &CNF, var_num: usize) -> Variable {
        let mut v = Variable {
            value: VarValue::Free,
            neg_occ: cnf.clauses.iter().enumerate().filter_map(|(index, clause)| {
                    if clause.vars.contains(&CNFVar {id: var_num, sign: false}) {
                        Some(index)
                    } else {
                        None
                    }
                }).collect(),
            pos_occ: cnf.clauses.iter().enumerate().filter_map(|(index, clause)| {
                    if clause.vars.contains(&CNFVar {id: var_num, sign: true}) {
                        Some(index)
                    } else {
                        None
                    }
                }).collect()
        };
        // if variable is not used set it to false
        if v.neg_occ.len() == 0 && v.pos_occ.len() == 0 {
            v.value = VarValue::Neg;
        }
        v
    }
}

impl fmt::Display for Clause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "act: {}, sat: {:?}, lit: ", self.active_lits, self.satisfied)?;
        self.literals.iter().for_each(|lit| {if let Err(e) = write!(f, "{} ", lit){println!("{}", e)}});
        write!(f, "\n")
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "val: {:?}, pos: ", self.value)?;
        self.pos_occ.iter().for_each(|var| {if let Err(e) = write!(f, "{} ", var){println!("{}", e)}});
        write!(f, "| neg: ")?;
        self.neg_occ.iter().for_each(|var| {if let Err(e) = write!(f, "{} ", var){println!("{}", e)}});
        write!(f, "\n")
    }
}

impl Clause {
    fn new(cnf_clause: &CNFClause) -> Clause {
        // remove douplicated variables for active_lit, because they count as only 1 active literal
        let mut cnf_variables = cnf_clause.vars.clone();
        cnf_variables.sort();
        cnf_variables.dedup();

        Clause {
            active_lits: cnf_variables.len(),
            satisfied: None,
            literals: cnf_clause.vars.iter().map(|var| {
                if var.sign {
                    var.id as isize
                } else {
                    -1 * (var.id as isize)
                }
            }).collect()

        }
    }
}

struct DataStructures {
    variables: Vec<Variable>,
    clauses: Vec<Clause>,
    unit_queue: VecDeque<isize>,
    assignment_stack: Vec<PrevAssignment>,
}

impl DataStructures {
    fn new(cnf: CNF, num_variables: usize) -> DataStructures {
        let clauses: Vec<Clause> = cnf.clauses.iter().map(|cnf_clause| Clause::new(&cnf_clause)).collect();
        let variables = (1..=num_variables).map(|i| Variable::new(&cnf, i)).collect();
        let unit_queue = VecDeque::with_capacity(num_variables);
        let assignment_stack = Vec::with_capacity(num_variables);

        DataStructures {
            variables,
            clauses,
            unit_queue,
            assignment_stack,
        }
    }

    fn dpll(&mut self, mut branching: impl BranchingStrategy) -> Assignment {

        if !self.inital_unit_propagation() {
            return Assignment::Unsatisfiable;
        }


        // As long as there are variable left pick one of them.
        while let Some(i) = branching.pick_branching_variable(&self.variables, &self.clauses) {
            if self.satisfaction_check() {
                break;
            }

            if self.set_literal(i, AssignmentType::Branching, VarValue::Pos).is_none() {
                return Assignment::Unsatisfiable;
            }

            if self.process_unit_queue() == false {
                return Assignment::Unsatisfiable;
            }
        }

        self.variables.iter().map(|x| match x.value {
            VarValue::Pos => true,
            VarValue::Neg => false,
            _ => false
        }).collect()
    }

    fn satisfaction_check(&mut self) -> bool {
        let mut satisfied = true;
        self.clauses.iter().for_each(|clause| {
            if clause.satisfied == None {
                satisfied = false;
            }
        });
        satisfied
    }

    fn set_literal(&mut self, i: usize, assign_type: AssignmentType, sign: VarValue) -> Option<()> {
        self.variables[i].value = sign;
        self.assignment_stack.push(PrevAssignment {variable: i, assignment_type: assign_type});
        let assignment = self.unit_propagation(i)?;
        if assignment.assignment_type == AssignmentType::Branching {
            self.variables[assignment.variable].value = VarValue::Neg;
            self.assignment_stack.push(PrevAssignment {variable: assignment.variable, assignment_type: AssignmentType::Forced});
            self.unit_propagation(assignment.variable)?;
        }
        Some(())
    }

    fn process_unit_queue(&mut self) -> bool{
        loop {
            match self.unit_queue.pop_front() {
                Some(var) => {
                    let mut sign = VarValue::Pos;
                    if var < 0 {
                        sign = VarValue::Neg;
                    }
                    if self.set_literal((var.abs() - 1) as usize, AssignmentType::Forced, sign).is_none() {
                        return false;
                    }
                },
                None => break
            }
        }
        true
    }

    fn inital_unit_propagation(&mut self) -> bool {
        for i in 0..self.clauses.len() {
            if self.clauses[i].active_lits == 1 {
                let unit_var_index: isize = self.find_unit_variable(&self.clauses[i]);
                self.unit_queue.push_back(unit_var_index);
            }
        }
        
        self.process_unit_queue()
    }

    fn unit_propagation(&mut self, i: usize) -> Option<PrevAssignment> {
        let mut pos_occ: &Vec<usize> = &self.variables[i].pos_occ;
        let mut neg_occ: &Vec<usize> = &self.variables[i].neg_occ;
        let clauses = &mut self.clauses;
        if self.variables[i].value == VarValue::Neg {
            neg_occ = &self.variables[i].pos_occ;
            pos_occ = &self.variables[i].neg_occ;
        }

        pos_occ.iter().for_each(|p_occ| {
            if clauses[*p_occ].satisfied == None {
                clauses[*p_occ].satisfied = Some(i)
            }
        });
        for u in 0..neg_occ.len() {
            let n_occ = neg_occ[u];
            if self.clauses[n_occ].satisfied == None {
                self.clauses[n_occ].active_lits -= 1;

                if self.clauses[n_occ].active_lits == 1 {
                    let unit_var_index: isize = self.find_unit_variable(&self.clauses[n_occ]);
                    self.unit_queue.push_back(unit_var_index);
                } else if self.clauses[n_occ].active_lits <= 0 {
                    self.unit_queue.clear();
                    return self.backtracking();
                }
            }
        };
        Some(PrevAssignment {variable: i, assignment_type: AssignmentType::Empty})
    }

    fn backtracking(&mut self) -> Option<PrevAssignment> {
        while let Some(assign) = self.assignment_stack.pop() {
            self.variables[assign.variable as usize].value = VarValue::Free;
            for i in 0..self.variables[assign.variable as usize].neg_occ.len() {
                let n_occ = self.variables[assign.variable as usize].neg_occ[i];
                self.clauses[n_occ].active_lits += 1;
            }
            for i in 0..self.variables[assign.variable as usize].pos_occ.len() {
                let p_occ = self.variables[assign.variable as usize].pos_occ[i];
                if let Some(cl) = self.clauses[p_occ].satisfied {
                    if cl == assign.variable {
                        self.clauses[p_occ].satisfied = None
                    }
                }
            }
            if assign.assignment_type == AssignmentType::Branching {
                return Some(assign)
            }
        }
        // unsatisfied
        None
    }

    fn find_unit_variable(&self, clause: &Clause) -> isize {
        let mut variable: isize = 0;
        for lit in &clause.literals {
            if self.variables[(lit.abs()-1) as usize].value == VarValue::Free {
                variable = *lit;
                break;
            }
        }
        if variable == 0 {
            panic!("no unit variable could be found!");
        }
        variable
    }
    
    #[allow(dead_code)]
    fn print_data_structures(&self) {
        for var in self.variables.iter() {
            print!("{}", var);
        }
        for cl in self.clauses.iter() {
            print!("{}", cl);
        }
        println!("");
    }
}

