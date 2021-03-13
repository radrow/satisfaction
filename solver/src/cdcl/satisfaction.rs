use std::{cell::RefCell, collections::VecDeque, marker::PhantomData, ops::Not, rc::Rc};
use crate::{CNFClause, CNFVar, SATSolution, Solver, CNF};
use itertools::Itertools;
use tinyset::SetUsize;
use crate::solvers::{InterruptibleSolver, FlagWaiter};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use super::{
    variable::{VariableId, Variable, Variables, Assignment, AssignmentType},
    clause::{ClauseId, Clause, Clauses},
    util::{PriorityQueue, HashSet, HashMap, IndexMap, BuildHasher},
    update::{Initialisation, Update},
};

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


pub trait BranchingStrategy: Initialisation+Update {
    fn pick_literal(&mut self, clauses: &Clauses, variables: &Variables) -> Option<CNFVar>;
}

pub trait LearningScheme: Initialisation+Update {
    /// Finds the a cut in the implication graph.
    fn find_conflict_clause(&mut self, empty_clause: ClauseId, branching_depth: usize, clauses: &Clauses, variables: &Variables) -> Option<(CNFClause, CNFVar, usize)>;
}

pub trait ClauseDeletionStrategy: Initialisation+Update {
    fn delete_clause(&mut self, clauses: &Clauses, variables: &Variables) -> Vec<ClauseId>;
}


struct ExecutionState<B,L,C,R> {
    clauses: Clauses,
    variables: Variables,
    branching_depth: usize,
    assignment_stack: Vec<VariableId>,
    unit_queue: IndexMap<VariableId, (bool, ClauseId)>,

    branching_strategy: Rc<RefCell<B>>,
    learning_scheme: Rc<RefCell<L>>,
    clause_deletion_strategy: Rc<RefCell<C>>,
    restart_policy: Rc<RefCell<R>>,

    updates: Vec<Rc<RefCell<dyn Update>>>,
}

unsafe impl<B,L,C,R> Sync for ExecutionState<B,L,C,R> {}
unsafe impl<B,L,C,R> Send for ExecutionState<B,L,C,R> {}

impl<B,L,C,R> ExecutionState<B,L,C,R>
where B: 'static+BranchingStrategy,
      L: 'static+LearningScheme,
      C: 'static+ClauseDeletionStrategy,
      R: 'static+RestartPolicy {

    fn new(formula: &CNF) -> ExecutionState<B, L, C, R> {
        // TODO: Avoid cloning
        let ordered_cnf: CNF = order_formula(formula.clone());
        let variables = (1..=ordered_cnf.num_variables)
                .map(|index| Variable::new(&ordered_cnf, index))
                .collect();

        let clauses = ordered_cnf.clauses
                .iter()
                .map(|cnf_clause| Clause::new(cnf_clause))
                .collect(); 

        let unit_queue = IndexMap::with_capacity_and_hasher(formula.num_variables, BuildHasher::default());
        let assignment_stack= Vec::with_capacity(formula.num_variables);

        let branching_strategy = Rc::new(RefCell::new(B::initialise(&clauses, &variables)));
        let learning_scheme = Rc::new(RefCell::new(L::initialise(&clauses, &variables)));
        let clause_deletion_strategy = Rc::new(RefCell::new(C::initialise(&clauses, &variables)));
        let restart_policy = Rc::new(RefCell::new(R::initialise(&clauses, &variables)));

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
                restart_policy.clone(),
            ],

            branching_strategy,
            learning_scheme,
            clause_deletion_strategy,
            restart_policy,
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
                .filter(|_| {
                    let should_restart = self.restart_policy
                        .borrow_mut()
                        .restart();
                    if should_restart { self.restart() }
                    !should_restart
                })
                .map_or(false, |conflict_clause| {
                    self.backtracking(conflict_clause)
                })
            { return SATSolution::Unsatisfiable; }

            let to_delete = self.clause_deletion_strategy.borrow_mut()
                .delete_clause(&self.clauses, &self.variables);

            for clause in to_delete {
                let w = self.clauses.remove(clause);

                self.variables[w.0.id].remove_watched_occ(clause);
                self.variables[w.1.id].remove_watched_occ(clause);
            }
        }

        SATSolution::Satisfiable(
            self.variables
                .into_iter()
                .map(|var| var.assignment.map(|a| a.sign).unwrap_or(false))
                .collect_vec(),
        )
    }

    fn interruptible_cdcl(mut self, flag: Arc<AtomicBool>) -> SATSolution {
        if self.inital_unit_propagation() {
            flag.store(true, Ordering::Relaxed);
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
                .filter(|_| {
                    let should_restart = self.restart_policy
                        .borrow_mut()
                        .restart();
                    if should_restart { self.restart() }
                    !should_restart
                })
                .map_or(false, |conflict_clause| {
                    self.backtracking(conflict_clause)
                })
            { 
                flag.store(true, Ordering::Relaxed);
                return SATSolution::Unsatisfiable;
            }
            if flag.load(Ordering::Relaxed) { return SATSolution::Unknown }

            let to_delete = self.clause_deletion_strategy.borrow_mut()
                .delete_clause(&self.clauses, &self.variables);

            for clause in to_delete {
                let w = self.clauses.remove(clause);

                self.variables[w.0.id].remove_watched_occ(clause);
                self.variables[w.1.id].remove_watched_occ(clause);
            }
        }

        flag.store(true, Ordering::Relaxed);

        SATSolution::Satisfiable(
            self.variables
                .into_iter()
                .map(|var| var.assignment.map(|a| a.sign).unwrap_or(false))
                .collect_vec(),
        )
    }


    fn restart(&mut self) {
        self.branching_depth = 0;
        self.unit_queue.clear();
        while let Some(id) = self.assignment_stack.pop() {
            let assignment = self.variables[id].assignment.take().unwrap();
            let literal = CNFVar::new(id, assignment.sign);
            self.updates.iter().for_each(|up| up.borrow_mut().on_unassign(literal, &self.clauses, &self.variables));
        }
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
        let unit = literals.len() == 1;
        let index = self.clauses.push(literals);

        self.variables[self.clauses[index].get_first_watched().id].add_watched_occ(index);
        if !unit {
            self.variables[self.clauses[index].get_second_watched().id].add_watched_occ(index);
        }

        index
    }

    fn backtracking(
        &mut self,
        empty_clause: ClauseId,
    ) -> bool {
        if empty_clause >= self.clauses.len_formula() {
            self.updates.iter()
                .for_each(|up| up.borrow_mut().on_conflict(empty_clause, &self.clauses, &self.variables));
        }


        let (conflict_clause, assertion_literal, assertion_level) = match {
            let mut ls = self.learning_scheme.borrow_mut();
            ls.find_conflict_clause(empty_clause, self.branching_depth, &self.clauses, &self.variables)
        } {
            Some(t) => t,
            None => return true,
        };

        let index = self.add_clause(conflict_clause);
        self.updates.iter()
            .for_each(|up| up.borrow_mut().on_learn(index, &self.clauses, &self.variables));

        while let Some(id) = self.assignment_stack.pop() {
            match self.variables[id].assignment {
                Some(assign) if assign.branching_level > assertion_level => {
                    let literal = self.variables[id].assignment.take()
                        .map(|assign| CNFVar::new(id, assign.sign))
                        .unwrap();
                    self.updates.iter().for_each(|up| up.borrow_mut().on_unassign(literal, &self.clauses, &self.variables));
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
        let mut visited: SetUsize = literal_queue
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
                        let last_index = clause.len()-1;
                        clause.vars.swap(0, last_index);
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
    fn delete_clause(&mut self, _clauses: &Clauses, _variables: &Variables) -> Vec<ClauseId> { Vec::new() }
}


pub struct CDCLSolver<B,L,C,R> {
    branching_strategy: PhantomData<B>,
    learning_scheme: PhantomData<L>,
    clause_deletion_strategy: PhantomData<C>,
    restart_policy: PhantomData<R>,
}

impl<B,L,C,R> CDCLSolver<B,L,C,R>
where B: 'static+BranchingStrategy,
      L: 'static+LearningScheme,
      C: 'static+ClauseDeletionStrategy,
      R: 'static+RestartPolicy {
    pub fn new() -> CDCLSolver<B,L,C,R> {
        CDCLSolver {
            branching_strategy: PhantomData,
            learning_scheme: PhantomData,
            clause_deletion_strategy: PhantomData,
            restart_policy: PhantomData,
        }
    }
}

impl<B,L,C,R> Solver for CDCLSolver<B,L,C,R>
where B: 'static+BranchingStrategy,
      L: 'static+LearningScheme,
      C: 'static+ClauseDeletionStrategy,
      R: 'static+RestartPolicy {
          fn solve(&self, formula: &CNF) -> SATSolution {
              let execution_state = ExecutionState::<B,L,C,R>::new(formula);
              execution_state.cdcl()
          }
}

#[async_trait]
impl<B,L,C,R> InterruptibleSolver for CDCLSolver<B,L,C,R>
where B: 'static+Send+Sync+BranchingStrategy,
      L: 'static+Send+Sync+LearningScheme,
      C: 'static+Send+Sync+ClauseDeletionStrategy,
      R: 'static+Send+Sync+RestartPolicy {
          async fn solve_interruptible(&self, formula: &CNF) -> SATSolution {
              let execution_state = ExecutionState::<B,L,C,R>::new(formula);
              FlagWaiter::start(move |flag| {
                  execution_state.interruptible_cdcl(flag)
              }).await
          }
}


pub struct BerkMin {
    activity: HashMap<ClauseId, usize>,
    insertion_order: VecDeque<ClauseId>,
    threshold: usize,
}

impl Initialisation for BerkMin {
    fn initialise(_clauses: &Clauses, _variables: &Variables) -> Self where Self: Sized {
        BerkMin {
            activity: HashMap::default(),
            insertion_order: VecDeque::new(),
            threshold: 60,
        }
    }
}

impl Update for BerkMin {
    fn on_learn(&mut self, learned_clause: ClauseId, _clauses: &Clauses, _variables: &Variables) {
        self.insertion_order.push_back(learned_clause);
        self.activity.insert(learned_clause, 0);
    }

    fn on_conflict(&mut self, empty_clause: ClauseId, _clauses: &Clauses, _variables: &Variables) {
        *self.activity.get_mut(&empty_clause)
            .expect("Empty clause was not registered!") +=1;
    }
}

impl ClauseDeletionStrategy for BerkMin {
    fn delete_clause(&mut self, clauses: &Clauses, variables: &Variables) -> Vec<ClauseId> {
        let pct = clauses.len() / 16;
        let unassigned = self.insertion_order.iter()
            .cloned()
            .filter(|c| {
                clauses[*c].watched_literals.iter().all(|i| { variables[clauses[*c].literals[*i].id].assignment.is_none() })
            });

        let young = unassigned.clone().take(pct);
        let old = unassigned.skip(pct);
        
        let to_be_deleted: SetUsize= young.filter(|id| 42 < clauses[*id].literals.len() && self.activity[id] < 7)
            .chain(old.filter(|id| 8 < clauses[*id].literals.len() && self.activity[id] < self.threshold)).collect();

        self.activity.retain(|k,_| !to_be_deleted.contains(*k));
        self.insertion_order.retain(|k| !to_be_deleted.contains(*k));

        self.threshold += 1;

        to_be_deleted.into_iter().collect_vec()
    }
}

pub struct VSIDS {
    resort_period: usize,
    branchings: usize,
    priority_queue: PriorityQueue<*const usize, VariableId>,
    scores: Vec<usize>,
    counters: Vec<usize>
}

unsafe impl Sync for VSIDS {}
unsafe impl Send for VSIDS {}

impl VSIDS {
    #[inline]
    fn literal_to_index(literal: &CNFVar) -> usize {
        let mut index = 2*literal.id;
        if literal.sign { index += 1; }
        index
    }
    
    fn index_to_literal(index: usize) -> CNFVar {
        CNFVar {
            id: index/2,
            sign: index%2==1
        }
    }
}

impl Initialisation for VSIDS {
    fn initialise(clauses: &Clauses, variables: &Variables) -> Self where Self: Sized {
        let mut scores = std::iter::repeat(0)
            .take(2*variables.len())
            .collect_vec();
        let counters = scores.clone();

        for clause in clauses.iter() {
            for lit in clause.literals.iter() {
                scores[VSIDS::literal_to_index(lit)] += 1;
            }
        }


        let priority_queue = scores
            .iter()
            .enumerate()
            .map(|(id, p)| (id, p as *const usize))
            .collect();

        VSIDS {
            resort_period: 255,
            priority_queue,
            branchings: 0,
            scores,
            counters,
        }

    }
}

impl Update for VSIDS {
    fn on_learn(&mut self, learned_clause: ClauseId, clauses: &Clauses, _variables: &Variables) {
        for lit in clauses[learned_clause].literals.iter() {
            self.counters[VSIDS::literal_to_index(lit)] += 1;
        }
    }
    fn on_unassign(&mut self, literal: CNFVar, _clauses: &Clauses, _variables: &Variables) {
        let index = VSIDS::literal_to_index(&literal);
        let p = &self.scores[index];
        self.priority_queue.push(index, p as *const usize);
    }
}

impl BranchingStrategy for VSIDS {
    fn pick_literal(&mut self, _clauses: &Clauses, variables: &Variables) -> Option<CNFVar> {
        self.branchings += 1;

        if self.branchings >= self.resort_period {
            self.branchings = 0;
            self.scores.iter_mut()
                .zip(self.counters.iter_mut())
                .for_each(|(s, r)| {
                    let new = *s/2 + *r;
                    *s = new;
                    *r = 0;
                });

            take_mut::take(&mut self.priority_queue, |pq| {
                pq.into_iter()
                    .map(std::convert::identity)
                    .collect()
            });
        }

        while let Some((index, _)) = self.priority_queue.pop() {
            let lit = VSIDS::index_to_literal(index);
            if variables[lit.id].assignment.is_none() {
                return Some(lit);
            }
        }
        None
    }
}


pub trait RestartPolicy: Initialisation+Update {
    fn restart(&mut self) -> bool;
}

pub struct RestartNever;
impl Initialisation for RestartNever {
    fn initialise(_clauses: &Clauses, _variables: &Variables) -> RestartNever {
        RestartNever
    }
}

impl Update for RestartNever {}
impl RestartPolicy for RestartNever {
    fn restart(&mut self) -> bool {false}
}

pub struct RestartFixed { conflicts: u64, rate: u64 }
impl RestartFixed {
    pub fn new(rate: u64) -> RestartFixed {
        RestartFixed{
            conflicts: 0,
            rate,
        }
    }
}
impl Update for RestartFixed {
    fn on_conflict(&mut self, _empty_clause: ClauseId, _clauses: &Clauses, _variables: &Variables) {
        self.conflicts += 1;
    }
}
impl Initialisation for RestartFixed {
    fn initialise(_clauses: &Clauses, _variables: &Variables) -> RestartFixed {
        RestartFixed::new(550)
    }
}impl RestartPolicy for RestartFixed {
    fn restart(&mut self) -> bool {
        if self.conflicts > self.rate {
            self.conflicts = 0;
            return true
        }
        false
    }
}

pub struct RestartGeom { conflicts: u64, rate: u64, factor_percent: u64 }
impl RestartGeom {
    pub fn new(rate: u64, factor_percent: u64) -> RestartGeom {
        RestartGeom{
            conflicts: 0,
            rate,
            factor_percent,
        }
    }
}
impl Update for RestartGeom {
    fn on_conflict(&mut self, _empty_clause: ClauseId, _clauses: &Clauses, _variables: &Variables) {
        self.conflicts += 1;
    }
}
impl Initialisation for RestartGeom {
    fn initialise(_clauses: &Clauses, _variables: &Variables) -> RestartGeom {
        RestartGeom::new(100, 150)
    }
}
impl RestartPolicy for RestartGeom {
    fn restart(&mut self) -> bool {
        if self.conflicts > self.rate {
            self.rate *= self.factor_percent;
            self.rate /= 100;
            self.conflicts = 0;
            return true
        }
        false
    }
}


pub struct RestartLuby { conflicts: u64, rate: u64, luby_state: (u64, u64, u64) }
impl RestartLuby {
    fn next_luby(&mut self) -> u64 {
        let (u, v, w) = self.luby_state;
        self.luby_state =
            if u == w {
                if u == v {
                    (u + 1, 1, w * 2)
                } else {
                    (u, v + 1, w)
                }
            } else {
                (u + 1, v, w)
            };
        v
    }

    pub fn new() -> RestartLuby {
        RestartLuby {
            conflicts: 0,
            rate: 1,
            luby_state: (2, 1, 2), // first step already made
        }
    }
}
impl Update for RestartLuby {
    fn on_conflict(&mut self, _empty_clause: ClauseId, _clauses: &Clauses, _variables: &Variables) {
        self.conflicts += 1;
    }
}
impl Initialisation for RestartLuby {
    fn initialise(_clauses: &Clauses, _variables: &Variables) -> RestartLuby {
        RestartLuby::new()
    }
}
impl RestartPolicy for RestartLuby {
    fn restart(&mut self) -> bool {
        if self.conflicts > self.rate {
            self.rate = 32 * self.next_luby();
            self.conflicts = 0;
            return true
        }
        false
    }
}
