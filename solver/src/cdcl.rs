use std::{cell::RefCell, collections::{HashSet, VecDeque}, marker::PhantomData, ops::Not, rc::Rc, hash::BuildHasherDefault, iter::FromIterator};
use crate::{CNFClause, CNFVar, SATSolution, Solver, CNF};
use itertools::Itertools;

type IndexSet<V> = indexmap::IndexSet<V, BuildHasherDefault<rustc_hash::FxHasher>>;
type IndexMap<K, V> = indexmap::IndexMap<K, V, BuildHasherDefault<rustc_hash::FxHasher>>;

pub type VariableId = usize;
pub type ClauseId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignmentType {
    Forced(ClauseId),
    Branching,
    Known,
}

#[derive(Debug, Clone, Copy)]
pub struct Assignment {
    pub sign: bool,
    pub branching_level: usize,
    pub reason: AssignmentType,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub watched_occ: HashSet<ClauseId>,
    pub assignment: Option<Assignment>,
}

fn order_formula(cnf: CNF) -> CNF {
    let mut order_cnf: CNF = CNF {clauses: Vec::new(), num_variables: cnf.num_variables};
    for cnf_clause in cnf.clauses {
        let mut cnf_variables = cnf_clause.vars.clone();
        cnf_variables.sort();
        cnf_variables.dedup();
        order_cnf.clauses.push(CNFClause {vars: cnf_variables});
    }
    order_cnf
}

impl Variable {
    fn new(cnf: &CNF, var_num: usize) -> Variable {
        // default assignment if the variable is not contained in any clause and is empty
        let mut assignment: Option<Assignment> = Some(Assignment {
            sign: false,
            branching_level: 0,
            reason: AssignmentType::Known,
        });
        // if variable is contained in any clause set it to unassigned first
        cnf.clauses.iter().for_each(|clause| {
            for var in &clause.vars {
                if var_num == var.id {
                    assignment = None;
                }
            }
        });

        Variable {
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
            assignment 
        }
    }

    fn add_watched_occ(&mut self, index: ClauseId) {
        self.watched_occ.insert(index);
    }

    fn remove_watched_occ(&mut self, index: ClauseId) {
        self.watched_occ.remove(&index);
    }
}

#[derive(Debug)]
pub struct Clause {
    literals: Vec<CNFVar>,
    watched_literals: [usize; 2],
}

impl Clause {
    fn new(cnf_clause: &CNFClause) -> Clause {
        // decrement the variables by 1 to get a 0 offset
        let mut cnf_variables = cnf_clause.vars.clone();
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

    fn get_first_watched(&self) -> CNFVar {
        self.literals[self.watched_literals[0]]
    }

    #[allow(dead_code)]
    fn get_second_watched(&self) -> CNFVar {
        self.literals[self.watched_literals[1]]
    }
}

pub type Clauses = Vec<Clause>;
pub type Variables = Vec<Variable>;

pub trait Update {
    fn on_assign(&mut self, variable: VariableId, clauses: &Clauses, variables: &Variables) {}
    fn on_unassign(&mut self, variable: VariableId, clauses: &Clauses, variables: &Variables) {}
    fn on_learn(&mut self, learned_clause: ClauseId, clauses: &Clauses, variables: &Variables) {}
    fn on_conflict(&mut self, empty_clause: ClauseId, clauses: &Clauses, variables: &Variables) {}
    fn on_deletion(&mut self, deleted_clause: &Clause) {}
}

pub trait Initialisation {
    fn initialise(clauses: &Clauses, variables: &Variables) -> Self where Self: Sized;
}

pub trait BranchingStrategy: Initialisation+Update {
    fn pick_literal(&mut self, clauses: &Clauses, variables: &Variables) -> Option<CNFVar>;
}

pub trait LearningScheme: Initialisation+Update {
    fn find_conflict_clause(&mut self, empty_clause: ClauseId, branching_depth: usize, clauses: &Clauses, variables: &Variables) -> Option<(CNFClause, CNFVar, usize)>;
}

pub trait ClauseDeletionStrategy: Initialisation+Update {
    fn delete_clause(&mut self, clauses: &mut Clauses, variables: &mut Variables);
}


struct ExecutionState<B,L,C>
where B: 'static+BranchingStrategy,
      L: 'static+LearningScheme,
      C: 'static+ClauseDeletionStrategy {

    clauses: Clauses,
    variables: Variables,
    branching_depth: usize,
    assignment_stack: Vec<VariableId>,
    unit_queue: IndexMap<VariableId, (bool, ClauseId)>,

    branching_strategy: Rc<RefCell<B>>,
    learning_scheme: Rc<RefCell<L>>,
    clause_deletion_strategy: Rc<RefCell<C>>,

    updates: Vec<Rc<RefCell<dyn Update>>>,
}

impl<B,L,C> ExecutionState<B,L,C>
where B: BranchingStrategy,
      L: LearningScheme,
      C: ClauseDeletionStrategy {

    fn new(formula: &CNF) -> ExecutionState<B, L, C> {
        // TODO: Avoid cloning
        let ordered_cnf: CNF = order_formula(formula.clone());
        let variables = (1..=ordered_cnf.num_variables)
                .map(|index| Variable::new(&ordered_cnf, index))
                .collect();

        let clauses = ordered_cnf.clauses
                .iter()
                .map(|cnf_clause| Clause::new(cnf_clause))
                .collect(); 

        let unit_queue = IndexMap::with_capacity_and_hasher(formula.num_variables, BuildHasherDefault::default());
        let assignment_stack= Vec::with_capacity(formula.num_variables);

        let branching_strategy = Rc::new(RefCell::new(B::initialise(&clauses, &variables)));
        let learning_scheme = Rc::new(RefCell::new(L::initialise(&clauses, &variables)));
        let clause_deletion_strategy = Rc::new(RefCell::new(C::initialise(&clauses, &variables)));

        ExecutionState {
            clauses,
            variables,
            branching_depth: 0,
            unit_queue,
            assignment_stack,

            updates: vec![
                branching_strategy.clone(),
                learning_scheme.clone(),
                clause_deletion_strategy.clone(),
            ],

            branching_strategy,
            learning_scheme,
            clause_deletion_strategy,
        }
    }

    fn cdcl(mut self) -> SATSolution {
        if self.inital_unit_propagation() {
            return SATSolution::Unsatisfiable;
        }

        while let Some(literal) = { 
            let mut bs = self.branching_strategy.borrow_mut();
            bs.pick_literal(&self.clauses, &self.variables)
        } {
            self.branching_depth += 1;
            // Try to set a variable or receive the conflict clause it was not possible
            if self
                .set_variable(literal, AssignmentType::Branching)
                // If there was no conflict, eliminate unit clauses
                .or(self.unit_propagation())
                // If there was a conflict backtrack; return true if backtracking failed
                .map_or(false, |conflict_clause| {
                    self.backtracking(conflict_clause)
                })
            {
                return SATSolution::Unsatisfiable;

            // TODO: Probably redundant
            } else if self.is_satisfied() {
                break;
            }
        }

        SATSolution::Satisfiable(
            self.variables
                .into_iter()
                .map(|var| var.assignment.map(|a| a.sign).unwrap_or(false))
                .collect_vec(),
        )
    }

    fn set_variable(&mut self, literal: CNFVar, assign_type: AssignmentType) -> Option<ClauseId> {
        // set the variable and remember the assignment
        let assignment = Assignment {
            sign: literal.sign,
            branching_level: self.branching_depth,
            reason: assign_type
        };

        self.variables[literal.id].assignment = Some(assignment);
        self.assignment_stack.push(literal.id);


        if self.variables[literal.id].watched_occ.len() > 0 {
            // when a set literal is also watched find a new literal to be watched
            return self.find_new_watched(literal.id);
        }
        
        None
    }

    fn unit_propagation(&mut self) -> Option<ClauseId> {
        while let Some((id, (sign, reason))) = self.unit_queue.pop() {
            let empty_clause = self.set_variable(CNFVar::new(id, sign), AssignmentType::Forced(reason));
            if empty_clause.is_some() {
                return empty_clause;
            }
        }
        None
    }

    fn inital_unit_propagation(&mut self) -> bool {
        for i in 0..self.clauses.len() {
            if self.clauses[i].literals.len() > 0 
                && self.clauses[i].watched_literals[0] == self.clauses[i].watched_literals[1] {
                let literal = self.clauses[i].get_first_watched();
                if self.add_to_unit_queue(literal, i)
                    || self.unit_propagation().is_some() {
                        return true;
                }
            }
        }
        false
    }

    fn add_to_unit_queue(&mut self, literal: CNFVar, reason: ClauseId) -> bool {
        self.unit_queue.insert(literal.id, (literal.sign, reason))
            .map_or(false, |(sign, _)| sign != literal.sign)
    }

    fn add_clause(&mut self, literals: CNFClause) -> usize {
        let index = self.clauses.len();
        let watched_literals = match literals.len() {
            0 => unreachable!(),
            1 => [0,0],
            _ => [0,1],
        };
        for lit in literals.vars.iter() {
            self.variables[lit.id].add_watched_occ(index);
        }

        let clause = Clause {
            literals: literals.vars,
            watched_literals,
        };

        self.clauses.push(clause);
        index
    }

    fn backtracking(
        &mut self,
        empty_clause: ClauseId,
    ) -> bool {
        self.updates.iter()
            .for_each(|up| up.as_ref().borrow_mut().on_conflict(empty_clause, &self.clauses, &self.variables));


        let (conflict_clause, assertion_literal, assertion_level) = match {
            let mut ls = self.learning_scheme.borrow_mut();
            ls.find_conflict_clause(empty_clause, self.branching_depth, &self.clauses, &self.variables)
        } {
            Some((_,_,l)) if l == self.branching_depth => return true,
            Some(t) => t,
            None => return true,
        };

        let index = self.add_clause(conflict_clause);
        self.updates.iter()
            .for_each(|up| up.borrow_mut().on_learn(index, &self.clauses, &self.variables));

        while let Some(id) = self.assignment_stack.pop() {
            match self.variables[id].assignment {
                Some(assign) if assign.branching_level > assertion_level => {
                    self.variables[id].assignment = None;
                    self.updates.iter().for_each(|up| up.borrow_mut().on_unassign(id, &self.clauses, &self.variables));
                },
                _ => {
                    self.assignment_stack.push(id);
                    break;
                },
            }
        }

        self.branching_depth = assertion_level;
        self.unit_queue.clear();
        self.unit_queue.insert(assertion_literal.id, (assertion_literal.sign, index));

        self.unit_propagation()
            .map_or(false, |empty_clause| self.backtracking(empty_clause))
    }

   fn find_new_watched(&mut self, var_index: usize) -> Option<ClauseId> {
        // needs to be cloned before cause watched_occ will change its size while loop is happening 
        // due to removing and adding new occ
        let watched_occ: HashSet<ClauseId> = self.variables[var_index].watched_occ.clone();

        'clauses: for clause_index in watched_occ {
            let watched_literals: (CNFVar, CNFVar) = self.clauses[clause_index].get_watched_lits();

            // index of the watched variable that has been assigned a value
            let mut move_index: usize = 0;
            if watched_literals.1.id == var_index {
                move_index = 1;
            }

            if self.check_watched_lit_satisfied(clause_index) {
                continue 'clauses;
            }

            let mut no_unassign: bool = true;
            // find new literal
            for lit_index in 0..self.clauses[clause_index].literals.len() {
                match self.variables[self.clauses[clause_index].literals[lit_index].id].assignment {
                    // Variable is assigned to a value already
                    Some(assign) => {
                        // check if clause is satisfied
                        if assign.sign == self.clauses[clause_index].literals[lit_index].sign {
                            // clause is satisfied there is no need to find a new literal
                            continue 'clauses;
                        } 
                    },

                    // Variable not assigned yet
                    None => {
                        // unassigned variable found
                        no_unassign = false;

                        if self.change_watched_lists(lit_index, clause_index, move_index, var_index) {
                            continue 'clauses;
                        }
                    }
                }
            }

            if no_unassign {
                // report conflict 
                return Some(clause_index);
            } else {
                // unassigned variable is the other watched literal which means it is a unit variable
                let literal = self.variables[watched_literals.0.id].assignment
                    .map_or(watched_literals.0, |_| watched_literals.1);

                if self.add_to_unit_queue(literal, clause_index) { return Some(clause_index) }
            } 
        }
        None
    }

    fn change_watched_lists(&mut self, lit_index: usize, clause_index: usize, move_index: usize, var_index: usize) -> bool {
        let clause: &mut Clause = &mut self.clauses[clause_index];
        let literals: &Vec<CNFVar> = &clause.literals;
        let watched_index: [usize; 2] = clause.watched_literals;


        if lit_index != watched_index[0] && lit_index != watched_index[1] {
            clause.watched_literals[move_index] = lit_index;
            self.variables[literals[lit_index].id].add_watched_occ(clause_index);
            self.variables[var_index].remove_watched_occ(clause_index);
            return true
        }
        false
    }

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

    fn is_satisfied(&mut self) -> bool {
        self.variables.iter()
            .all(|var| var.assignment.is_some() || var.watched_occ.is_empty())
    }
}


pub struct RelSAT;

impl Initialisation for RelSAT {
    fn initialise(_clauses: &Clauses, _variables: &Variables) -> Self where Self: Sized {
        RelSAT
    }
}
impl Update for RelSAT {}


impl LearningScheme for RelSAT {
    fn find_conflict_clause(&mut self, empty_clause: ClauseId, branching_depth: usize, clauses: &Clauses, variables: &Variables) -> Option<(CNFClause, CNFVar, usize)> {
        // Start with vertices that are connected to the conflict clause
        let mut literal_queue: VecDeque<VariableId> = clauses[empty_clause].literals
            .iter()
            .map(|lit| lit.id)
            .collect();
        let mut visited: HashSet<VariableId> = literal_queue
            .iter()
            .cloned()
            .collect();


        let mut clause = CNFClause::with_capacity(literal_queue.len());
        let mut assertion_literal = None;
        let mut assertion_level = 0;

        while let Some(id) = literal_queue.pop_front() {
            let variable = &variables[id];
            match variable.assignment {
                // For each forced literal of current branching_level
                // append connected vertices to literal queue
                Some(Assignment{branching_level, reason: AssignmentType::Forced(reason), ..}) if branching_level == branching_depth => { 
                    for lit in clauses[reason].literals.iter() {
                        if visited.insert(lit.id) {
                            literal_queue.push_back(lit.id);
                        }
                    }
                },
                Some(Assignment{sign, branching_level, ..}) => {
                    let literal = CNFVar::new(id, sign.not());
                    clause.push(literal);
                    if branching_level != branching_depth {
                        assertion_level = std::cmp::max(assertion_level, branching_level);
                    } else {
                        assertion_literal = Some(literal);
                    }
                }
                _ => {},
            }
        }
        assertion_literal.map(|literal| (clause, literal, assertion_level))
    }
}

pub struct IdentityDeletionStrategy;
impl Initialisation for IdentityDeletionStrategy {
    fn initialise(_clauses: &Clauses, _variables: &Variables) -> Self where Self: Sized { IdentityDeletionStrategy }
}
impl Update for IdentityDeletionStrategy {}
impl ClauseDeletionStrategy for IdentityDeletionStrategy {
    fn delete_clause(&mut self, _clauses: &mut Clauses, _variables: &mut Variables) {}
}


pub struct CDCLSolver<B,L,C>
where B: BranchingStrategy,
      L: LearningScheme,
      C: ClauseDeletionStrategy {

    branching_strategy: PhantomData<B>,
    learning_scheme: PhantomData<L>,
    clause_deletion_strategy: PhantomData<C>,
}

impl<B,L,C> CDCLSolver<B,L,C>
where B: 'static+BranchingStrategy,
      L: 'static+LearningScheme,
      C: 'static+ClauseDeletionStrategy {
    pub fn new() -> CDCLSolver<B,L,C> {
        CDCLSolver {
            branching_strategy: PhantomData,
            learning_scheme: PhantomData,
            clause_deletion_strategy: PhantomData,
        }
    }
}


impl<B,L,C> Solver for CDCLSolver<B,L,C>
where B: 'static+BranchingStrategy,
      L: 'static+LearningScheme,
      C: 'static+ClauseDeletionStrategy {
          fn solve(&self, formula: &CNF) -> SATSolution {
              let execution_state = ExecutionState::<B,L,C>::new(formula);
              execution_state.cdcl()
          }
}
