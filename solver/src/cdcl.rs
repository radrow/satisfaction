use std::{collections::{HashSet, VecDeque}, iter::FromIterator};
use crate::{CNF, CNFClause, CNFVar, SATSolution, Solver};

pub struct CDCLSat;
impl Solver for CDCLSat {
    fn solve(&self, formula: &CNF) -> SATSolution {
        let mut data = DataStructures::new(formula);
        data.watched_literals()
    }
}


type VariableId = usize;
type ClauseId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AssignmentType {
    Forced,
    Branching,
}

#[derive(Debug, Clone)]
struct PrevAssignment {
    literal: CNFVar,
    assignment: Assignment
}

#[derive(Debug, Clone, Copy)]
struct Assignment {
    sign: bool,
    branching_level: usize, 
    reason: AssignmentType,
}

#[derive(Debug, Clone)]
struct Variable {
    watched_occ: HashSet<ClauseId>,
    debug_vector: Vec<ClauseId>,
    assignment: Option<Assignment>,
}

impl Variable {
    fn new(cnf: &CNF, var_num: usize) -> Variable {
        // defalut assignment if the variable is not contained in any clause and is empty
        let mut assignment: Option<Assignment> = Some(Assignment {
            sign: false,
            branching_level: 0,
            reason: AssignmentType::Forced
        });
        // if variable is contained in any clause set it to unassigned first
        cnf.clauses.iter().for_each(|clause| {
            for var in &clause.vars {
                if var_num == var.id {
                    assignment = None;
                }
            }
        });
        let variable = Variable {
            watched_occ: cnf.clauses
                .iter()
                .enumerate()
                .filter_map(|(index, clause)| {
                    if clause.vars.first()?.id == var_num {
                        return Some(index);
                    }
                    if clause.vars.last()?.id == var_num {
                        return Some(index);
                    }
                    return None;
                }).collect(),
            debug_vector: cnf.clauses
                .iter()
                .enumerate()
                .filter_map(|(index, clause)| {
                    if clause.vars.first()?.id == var_num {
                        return Some(index);
                    }
                    if clause.vars.last()?.id == var_num {
                        return Some(index);
                    }
                    return None;
                }).collect(),
            assignment 
        };


        variable
    }

    fn add_watched_occ(&mut self, index: ClauseId) {
        self.watched_occ.insert(index);
        self.debug_vector = Vec::from_iter(self.watched_occ.clone());
    }

    fn remove_watched_occ(&mut self, index: ClauseId) {
        self.watched_occ.remove(&index);
        self.debug_vector = Vec::from_iter(self.watched_occ.clone());
    }
}


struct Clause {
    literals: Vec<CNFVar>,
    watched_literals: [usize; 2],
}

impl Clause {
    fn new(cnf_clause: &CNFClause) -> Clause {
        // remove douplicated variables for active_lit, because they count as only 1 active literal
        let mut cnf_variables = cnf_clause.vars.clone();
        cnf_variables.sort();
        cnf_variables.dedup();
        cnf_variables.iter_mut().for_each(|var| var.id -= 1);

        // assign the first and the last literal as watched literals
        let mut watched_literals: [usize; 2] = [0, 0];
        if cnf_variables.len() > 0 {
            watched_literals = [0, cnf_variables.len() - 1];
        }

        Clause {
            literals: cnf_variables,
            watched_literals,
        }
    }

    fn get_watched_lits(&self) -> (CNFVar, CNFVar) {
        (self.literals[self.watched_literals[0]], self.literals[self.watched_literals[1]])
    }
}


type Clauses = Vec<Clause>;
type Variables = Vec<Variable>;

struct DataStructures {
    variables: Vec<Variable>,
    clauses: Vec<Clause>,
    unit_queue: VecDeque<CNFVar>,
    assignment_stack: Vec<PrevAssignment>,
    branching_level: usize
}

impl DataStructures {
    fn new(cnf: &CNF) -> DataStructures {
        DataStructures {
            variables: (1..=cnf.num_variables)
                .map(|index| Variable::new(cnf, index))
                .collect(),
            clauses: cnf.clauses
                .iter()
                .map(|cnf_clause| Clause::new(cnf_clause))
                .collect(),
            unit_queue: VecDeque::with_capacity(cnf.num_variables),
            assignment_stack: Vec::with_capacity(cnf.num_variables),
            branching_level: 0
        }
    }

    fn watched_literals(&mut self) -> SATSolution {
        if !self.inital_unit_propagation() {
            return SATSolution::Unsatisfiable;
        }

        // repeat & choose literal b
        while let Some(literal) = self.pick_branching_variable(&self.variables, &self.clauses)
        {
            // set value b
            let conflict = !(self.set_variable(literal, AssignmentType::Branching)
                // unit propagation
                && self.unit_propagation());

            // If backtracking does not help, formula is unsat.
            if conflict && !self.backtracking() {
                return SATSolution::Unsatisfiable;
            }

            //if self.satisfaction_check() {
            //    break;
            //}
        }

        // output assignment
        self.variables
            .iter()
            .map(|x| match x.assignment {
                Some(assign) => assign.sign,
                None => false,
            })
            .collect()
    }

    fn inital_unit_propagation(&mut self) -> bool {
        for clause in &self.clauses {
            if clause.literals.len() > 0 && (clause.watched_literals[0] == clause.watched_literals[1]) {
                if !self.unit_queue.contains(&clause.get_watched_lits().0) {
                    self.unit_queue.push_back(clause.get_watched_lits().0);
                }
            }
        }
        self.unit_propagation()
    }

    fn unit_propagation(&mut self) -> bool {
        while let Some(var) = self.unit_queue.pop_front() {
            if !self.set_variable(var, AssignmentType::Forced) {
                return false
            }
        }
        true
    }

    fn set_variable(&mut self, lit: CNFVar, assign_type: AssignmentType) -> bool {
        //increase branching level
        if assign_type == AssignmentType::Branching {
            self.branching_level += 1;
        }

        // set the variable and remember the assignment
        let assignment = Assignment {
            sign: lit.sign,
            branching_level: self.branching_level,
            reason: assign_type
        };
        self.variables[lit.id].assignment = Some(assignment);
        self.assignment_stack.push(PrevAssignment{
            assignment,
            literal: lit
        });

        if self.variables[lit.id].watched_occ.len() > 0 {
            // when a set literal is also watched find a new literal to be watched
            return self.find_new_watched(lit.id);
        }
        
        true
    }

    fn find_new_watched(&mut self, var_index: usize) -> bool {
        // needs to be cloned before cause watched_occ will change its size while loop is happening 
        // due to removing and adding new occ
        let watched_occ: HashSet<ClauseId> = self.variables[var_index].watched_occ.clone();

        'clauses: for clause_index in watched_occ {
            let clause: &mut Clause = &mut self.clauses[clause_index];
            let literals: &Vec<CNFVar> = &clause.literals;
            let watched_literals: (CNFVar, CNFVar) = clause.get_watched_lits();
            let watched_index: [usize; 2] = clause.watched_literals;

            // index of the watched variable that has been assigned a value
            let mut move_index: usize = 0;
            if watched_literals.1.id == var_index {
                move_index = 1;
            }

            // todo -> into function
            // check if the clause is satisfied over a watched literal already
            let first = match self.variables[literals[watched_index[0]].id].assignment {
                Some(assign) => assign.sign == literals[watched_index[0]].sign,
                None => false
            };
            let second = match self.variables[literals[watched_index[1]].id].assignment {
                Some(assign) => assign.sign == literals[watched_index[1]].sign,
                None => false
            };
            if first || second {
                continue 'clauses;
            }

            let mut no_unassign: bool = true;
            // find new literal
            for lit_index in 0..literals.len() {
                match self.variables[literals[lit_index].id].assignment {
                    // Variable is assigned to a value already
                    Some(assign) => {
                        // check if clause is satisfied
                        if assign.sign == literals[lit_index].sign {
                            // clause is satisfied there is no need to find a new literal
                            continue 'clauses;
                        } 
                    },

                    // Variable not assigned yet
                    None => {
                        // unassigned variable found
                        no_unassign = false;

                        // check if variable is not one of the watched variables
                        if lit_index != watched_index[0] && lit_index != watched_index[1] {

                            // todo -> into function
                            // change watched variables and clauses to new ones
                            clause.watched_literals[move_index] = lit_index;
                            self.variables[literals[lit_index].id].add_watched_occ(clause_index);
                            self.variables[var_index].remove_watched_occ(clause_index);
                            continue 'clauses;
                        }
                    }
                }
            }
            if no_unassign {
                // report conflict 
                return false;
            } else {
                // unassigned variable is the other watched literal which means it is a unit variable
                self.enqueue_unit(watched_literals);
            } 
        }
        true
    }

    // todo -> use as function (maybe with unsafe)
    /// Method to check if one of the watched literals that has an assinged value also satisfies the 
    /// Clause.
    fn check_watched_lit_satisfied(&self, clause_index: usize) -> bool {
        let watched_index: [usize; 2] = self.clauses[clause_index].watched_literals;
        let literals: &Vec<CNFVar> = &self.clauses[clause_index].literals;

        let first = match self.variables[literals[watched_index[0]].id].assignment {
            Some(assign) => assign.sign == literals[watched_index[0]].sign,
            None => false
        };
        let second = match self.variables[literals[watched_index[1]].id].assignment {
            Some(assign) => assign.sign == literals[watched_index[1]].sign,
            None => false
        };
        first || second
    }

    fn enqueue_unit(&mut self, watched_literals: (CNFVar, CNFVar)) {
        match self.variables[watched_literals.0.id].assignment {
            Some(_) => {
                if !self.unit_queue.contains(&watched_literals.1) {
                    self.unit_queue.push_back(watched_literals.1)
                }
            },
            None => {
                if !self.unit_queue.contains(&watched_literals.0) {
                    self.unit_queue.push_back(watched_literals.0)
                }
            }
        }

    }


    fn backtracking(&mut self) -> bool {
        // clear unit queue cause we are setting state back to last branching
        self.unit_queue.clear();

        while let Some(assign) = self.assignment_stack.pop() {
            self.variables[assign.literal.id].assignment = None;
            if assign.assignment.reason == AssignmentType::Branching {
                self.branching_level -= 1;
                // branch different 
                if self.set_variable(-assign.literal, AssignmentType::Forced) {
                    if self.unit_propagation() == false {
                        return self.backtracking();
                    }
                }
                return true;
            }
        }
        false
    }

    fn satisfaction_check(&mut self) -> bool {
        true
    }

    // todo -> Remove this and work with branching variable trait
    fn pick_branching_variable(&self, variables: &Variables, _clauses: &Clauses) -> Option<CNFVar> {
        variables
            .iter()
            .enumerate()
            // Only consider unset variables
            .filter_map(|(i, v)| match v.assignment {
                Some(_) => None,
                None => Some(CNFVar::new(i, true)),
            })
            .next() // Take the first one
    }
}




trait BranchingStrategy {
    fn pick_literal(&self, clause: &Clauses, variables: &Variables) -> Option<CNFVar>;
}

trait LearningScheme {
    fn find_conflict_clause(&self, clauses: &Clauses, variables: &Variables) -> Option<Clause>;
}

trait ClauseDeletionStrategy {
    fn delete_clause(&self, clauses: &mut Clauses, variables: &mut Variables);
}

struct ExecutionState{
    clauses: Clauses,
    variables: Variables,
    branching_depth: usize,
    unit_queue: VecDeque<CNFVar>,
}

impl ExecutionState {
    fn cdcl(&mut self, branching_strategy: &impl BranchingStrategy, learning_scheme: &impl LearningScheme, clause_deletion_strategy: &impl ClauseDeletionStrategy) -> SATSolution {
        todo!()
    }
}


struct CDCLSolver<B: BranchingStrategy, L: LearningScheme, C: ClauseDeletionStrategy> {
    branching_strategy: B,
    learning_scheme: L,
    clause_deletion_strategy: C,
}

impl<B,L,C> Solver for CDCLSolver<B, L, C>
where B: BranchingStrategy,
      L: LearningScheme,
      C: ClauseDeletionStrategy {
    fn solve(&self, formula: &CNF) -> SATSolution {
       todo!() 
    }
}
