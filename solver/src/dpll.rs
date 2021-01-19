use crate::cnf;

use cnf::CNF;
use cnf::CNFClause;
use cnf::CNFVar;
use std::fmt;
use std::collections::VecDeque;
use crate::{Solver, SATSolution};

pub trait BranchingStrategy: Clone {
    /// Funtion that picks the next variable to be chosen for branching.
    /// Returns the index of the next variable, or None if there is no Variable to be picked
    fn pick_branching_variable(&mut self, variables: &Variables, clauses: &Clauses) -> Option<CNFVar>;
}

#[derive(Clone)]
pub struct NaiveBranching;

impl BranchingStrategy for NaiveBranching {
    fn pick_branching_variable(&mut self, variables: &Variables, _clauses: &Clauses) -> Option<CNFVar> {
        // TODO -> add heuristics to chose Variables
        variables.iter()
            .enumerate()
            .filter_map(|(i,v)| match v.value {
                VarValue::Free  => Some(CNFVar::new(i, true)),
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
    fn solve(&self, formula: CNF) -> SATSolution {
        let mut data = DataStructures::new(formula);
        data.dpll(self.strategy.clone())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AssignmentType {
    Forced, Branching
}


/// Used to store assignments made in the past, for undoing them with backtracking
struct PrevAssignment {
    literal: CNFVar,
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
    pos_occ: Vec<usize>,
    neg_occ: Vec<usize>,
}

pub struct Clause {
    active_lits: usize,
    satisfied: Option<usize>,
    literals: Vec<CNFVar>,
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
        cnf_variables.iter_mut()
            .for_each(|var| var.id -= 1);

        Clause {
            active_lits: cnf_variables.len(),
            satisfied: None,
            literals: cnf_variables,
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

    fn dpll(&mut self, mut branching: impl BranchingStrategy) -> SATSolution {
        // unit propagation
        if !self.inital_unit_propagation() {
            return SATSolution::Unsatisfiable;
        }

        // repeat & choose literal b 
        while let Some(literal) = branching.pick_branching_variable(&self.variables, &self.clauses) {
            // set value b
            let conflict = !(self.set_variable(literal, AssignmentType::Branching)
                // unit propagation
                && self.unit_propagation()
                && self.pure_literal_elimination());
            
            // If backtracking does not help, formula is unsat.
            if conflict && !self.backtracking(){
                return SATSolution::Unsatisfiable;
            }

            if self.satisfaction_check() {
                break;
            }
        }

        // output assignment
        self.variables.iter().map(|x| match x.value {
            VarValue::Pos => true,
            VarValue::Neg => false,
            _ => false
        }).collect()
    }

    fn inital_unit_propagation(&mut self) -> bool {
        // find all unit clauses and enqueue the variables in the queue
        for i in 0..self.clauses.len() {
            if self.clauses[i].active_lits == 1 {
                let unit_literal = self.find_unit_variable(i);
                if !self.unit_queue.contains(&unit_literal) {
                    self.unit_queue.push_back(unit_literal);
                }
            }
        }
        
        self.unit_propagation()
    }

    

    fn set_variable(&mut self, lit: CNFVar, assign_type: AssignmentType) -> bool {
        self.variables[lit.id].value = lit.sign.into();
        self.assignment_stack.push(PrevAssignment { literal: lit, assignment_type: assign_type});

        let mut pos_occ: &Vec<usize> = &self.variables[lit.id].pos_occ;
        let mut neg_occ: &Vec<usize> = &self.variables[lit.id].neg_occ;
        let clauses = &mut self.clauses;

        if self.variables[lit.id].value == VarValue::Neg {
            neg_occ = &self.variables[lit.id].pos_occ;
            pos_occ = &self.variables[lit.id].neg_occ;
        }

        pos_occ.iter().for_each(|p_occ| {
            if clauses[*p_occ].satisfied == None {
                clauses[*p_occ].satisfied = Some(lit.id)
            }
        });

        let mut no_conflict = true;
        for u in 0..neg_occ.len() {
            let clause = &mut self.clauses[neg_occ[u]];
            clause.active_lits -= 1;
            if clause.satisfied.is_none() {
                if clause.active_lits == 1 {
                    // unit literal detected
                    let unit_literal = self.find_unit_variable(neg_occ[u]);
                    if !self.unit_queue.contains(&unit_literal) {
                        self.unit_queue.push_back(unit_literal);
                    }
                } else if clause.active_lits <= 0 {
                    // conflict
                    no_conflict =  false;
                }
            }
        };
        no_conflict
    }

    fn unit_propagation(&mut self) -> bool {
        while let Some(var) = self.unit_queue.pop_front() {
            if !self.set_variable(var, AssignmentType::Forced) {
                return false;
            }
        }
        true
    }

    fn pure_literal_elimination(&mut self) -> bool {
        let pure_literals = self.variables.iter()
            .enumerate()
            .filter_map(|(id, var)| {
                match var.value {
                    VarValue::Free if var.pos_occ.is_empty() =>
                        Some(CNFVar::new(id, false)),
                    VarValue::Free if var.neg_occ.is_empty() => 
                        Some(CNFVar::new(id, true)),
                    _ => None
                }
            }).collect::<Vec<_>>();
        pure_literals.into_iter()
            .all(|literal| self.set_variable(literal, AssignmentType::Branching))
    }


    fn backtracking(&mut self) -> bool {
        while let Some(assign) = self.assignment_stack.pop() {
            let mut pos_occ: &Vec<usize> = &self.variables[assign.literal.id].pos_occ;
            let mut neg_occ: &Vec<usize> = &self.variables[assign.literal.id].neg_occ;
            if self.variables[assign.literal.id].value == VarValue::Neg {
                neg_occ = &self.variables[assign.literal.id].pos_occ;
                pos_occ = &self.variables[assign.literal.id].neg_occ;
            }

            for i in 0..neg_occ.len() {
                let n_occ = neg_occ[i];
                self.clauses[n_occ].active_lits += 1;
            }
            for i in 0..pos_occ.len() {
                let p_occ = pos_occ[i];
                if let Some(cl) = self.clauses[p_occ].satisfied {
                    if cl == assign.literal.id {
                        self.clauses[p_occ].satisfied = None
                    }
                }
            }

            // empty queue
            self.unit_queue.clear();

            if assign.assignment_type == AssignmentType::Branching {
                if self.set_variable(-assign.literal, AssignmentType::Forced) {
                    // goto unit propagation
                    if self.unit_propagation() == false {
                        return self.backtracking();
                    }
                }
                return true
            }
            self.variables[assign.literal.id].value = VarValue::Free;
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

    fn satisfaction_check(&mut self) -> bool {
        let mut satisfied = true;
        self.clauses.iter().for_each(|clause| {
            if clause.satisfied == None {
                satisfied = false;
            }
        });
        satisfied
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

