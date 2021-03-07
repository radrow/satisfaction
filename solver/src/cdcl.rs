use itertools::Itertools;
use std::{cell::RefCell, collections::{HashSet, HashMap, VecDeque}, marker::PhantomData, ops::Not, rc::Rc};

type IndexSet<V> = indexmap::IndexSet<V, std::hash::BuildHasherDefault<rustc_hash::FxHasher>>;

use crate::{CNFClause, CNFVar, SATSolution, Solver, CNF};

type VariableId = usize;
type ClauseId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AssignmentType {
    Forced(ClauseId),
    Branching,
}

#[derive(Debug, Clone, Copy)]
struct Assignment {
    sign: bool,
    branching_level: usize,
    reason: AssignmentType,
}

#[derive(Debug, Clone)]
struct Variable {
    pos_watched_occ: HashSet<ClauseId>,
    neg_watched_occ: HashSet<ClauseId>,
    assignment: Option<Assignment>,
}

struct Clause {
    literals: Vec<CNFVar>,
    watched_literals: [CNFVar; 2],
}

impl Clause {
    /// Creates a new clause given an iterator of literals
    /// that is assumed to contain at least two elements.
    fn new(iter: impl Iterator<Item=CNFVar>, variables: &mut Variables) -> Clause {
        todo!()
    }
}

type Variables = Vec<Variable>;
type Clauses = Vec<Clause>;


trait Update {
    fn on_assign(&mut self, variable: VariableId, clauses: &Clauses, variables: &Variables) {}
    fn on_unassign(&mut self, variable: VariableId, clauses: &Clauses, variables: &Variables) {}
    fn on_learn(&mut self, learned_clause: &Clause, clauses: &Clauses, variables: &Variables) {}
    fn on_conflict(&mut self, empty_clause: ClauseId, clauses: &Clauses, variables: &Variables) {}
    fn on_deletion(&mut self, deleted_clause: &Clause) {}
}

trait Initialisation {
    fn initialise(clauses: &Clauses, variables: &Variables) -> Self where Self: Sized;
}

trait BranchingStrategy: Initialisation+Update {
    fn pick_literal(&mut self, clauses: &Clauses, variables: &Variables) -> Option<CNFVar>;
}

trait LearningScheme: Initialisation+Update {
    fn find_conflict_clause(&mut self, empty_clause: ClauseId, branching_depth: usize, clauses: &Clauses, variables: &Variables) -> (CNFClause, usize);
}

trait ClauseDeletionStrategy: Initialisation+Update {
    fn delete_clause(&mut self, clauses: &mut Clauses, variables: &mut Variables) -> Vec<ClauseId>;
}


struct ExecutionState<B,L,C>
where B: 'static+BranchingStrategy,
      L: 'static+LearningScheme,
      C: 'static+ClauseDeletionStrategy {

    clauses: Clauses,
    variables: Variables,
    branching_depth: usize,
    unit_queue: VecDeque<CNFVar>,

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
        let unit_queue = VecDeque::new();

        let clauses = Clauses::new();
        let variables = Variables::new();

        let branching_strategy = Rc::new(RefCell::new(B::initialise(&clauses, &variables)));
        let learning_scheme = Rc::new(RefCell::new(L::initialise(&clauses, &variables)));
        let clause_deletion_strategy = Rc::new(RefCell::new(C::initialise(&clauses, &variables)));

        ExecutionState {
            clauses,
            variables,
            branching_depth: 0,
            unit_queue,

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
        if self.unit_propagation().is_some() {
            return SATSolution::Unsatisfiable;
        }

        while let Some(literal) = { 
            let mut bs = self.branching_strategy.as_ref().borrow_mut();
            bs.pick_literal(&self.clauses, &self.variables)
        } {
            // Try to set a variable or receive the conflict clause it was not possible
            if self
                .set_variable(literal, false)
                // If there was no conflict, eliminate unit clauses
                .or(self.unit_propagation())
                // If there was a conflict backtrack; return true if backtracking failed
                .map_or(false, |conflict_clause| {
                    self.backtracking(conflict_clause)
                })
            {
                return SATSolution::Unsatisfiable;
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

    fn set_variable(&mut self, literal: CNFVar, is_forced: bool) -> Option<ClauseId> {
        None
    }

    fn unit_propagation(&mut self) -> Option<ClauseId> {
        while let Some(literal) = self.unit_queue.pop_back() {
            let empty_clause = self.set_variable(literal, true);
            if empty_clause.is_some() {
                return empty_clause;
            }
        }
        None
    }

    fn backtracking(
        &mut self,
        empty_clause: ClauseId,
    ) -> bool {
        self.updates.iter()
            .for_each(|up| up.as_ref().borrow_mut().on_conflict(empty_clause, &self.clauses, &self.variables));

        let (conflict_clause, assertion_level) = {
            let mut ls = self.learning_scheme.as_ref().borrow_mut();
            ls.find_conflict_clause(empty_clause, self.branching_depth, &self.clauses, &self.variables)
        };

        let conflict_clause = Clause::new(conflict_clause.into_iter(), &mut self.variables);
        self.updates.iter()
            .for_each(|up| up.as_ref().borrow_mut().on_learn(&conflict_clause, &self.clauses, &self.variables));
        self.clauses.push(conflict_clause);

        // TODO: More efficient: e.g. Stack + dropWhile
        for id in 0..self.variables.len() {
            match self.variables[id].assignment {
                Some(assign) if assign.branching_level > assertion_level => {
                    self.variables[id].assignment = None;

                    // 
                    self.updates.iter().for_each(|up| up.as_ref().borrow_mut().on_unassign(id, &self.clauses, &self.variables));
                },
                _ => {},
            }
        }

        self.branching_depth = assertion_level;
        self.unit_queue.clear();

        let literal = self.clauses.last()
            .expect("There are no clauses!")
            .watched_literals
            .iter()
            .find_map(|lit| 
                if self.variables[lit.id].assignment.is_none() { Some(lit.clone()) }
                else { None }
            ).expect("Conflict clause was not unit");
        self.unit_queue.push_back(literal);
        

        self.unit_propagation()
            .map_or(false, |empty_clause| self.backtracking(empty_clause))
    }

    fn is_satisfied(&self) -> bool {
        unimplemented!()
    }
}


struct RealSAT;

impl Initialisation for RealSAT {
    fn initialise(_clauses: &Clauses, _variables: &Variables) -> Self where Self: Sized {
        RealSAT
    }
}
impl Update for RealSAT {}


impl LearningScheme for RealSAT {
    fn find_conflict_clause(&mut self, empty_clause: ClauseId, branching_depth: usize, clauses: &Clauses, variables: &Variables) -> (CNFClause, usize) {
        // TODO: Optimize preallocation
        // Start with vertices that are connected to the conflict clause
        let mut literal_queue: IndexSet<VariableId> = clauses[empty_clause].literals
            .iter()
            .map(|lit| lit.id)
            .collect();

        let mut clause = CNFClause::with_capacity(literal_queue.len());

        let mut assertion_level = 0;
        while let Some(id) = literal_queue.pop() {
            let variable = &variables[id];
            match variable.assignment {
                // For each forced literal of current branching_level
                // append connected vertices to literal queue
                Some(Assignment{branching_level, reason: AssignmentType::Forced(reason), ..}) if branching_level == branching_depth => { 
                    literal_queue.extend(clauses[reason].literals.iter().map(|lit| lit.id))
                },
                Some(Assignment{sign, branching_level, ..}) => {
                    clause.push(CNFVar::new(id, sign.not()));
                    if branching_level != branching_depth {
                        assertion_level = std::cmp::max(assertion_level, branching_level);
                    }
                }
                _ => {},
            }
        }

        (clause, assertion_level)
    }
}


struct CDCLSolver<B,L,C>
where B: BranchingStrategy,
      L: LearningScheme,
      C: ClauseDeletionStrategy {

    branching_strategy: PhantomData<B>,
    learning_scheme: PhantomData<L>,
    clause_deletion_strategy: PhantomData<C>,
}

impl<B, L, C> Solver for CDCLSolver<B, L, C>
where
    B: 'static+BranchingStrategy,
    L: 'static+LearningScheme,
    C: 'static+ClauseDeletionStrategy,
{
    fn solve(&self, formula: &CNF) -> SATSolution {
        let execution_state = ExecutionState::<B, L, C>::new(formula);
        execution_state.cdcl()
    }
}


struct BerkMin {
    activity: HashMap<ClauseId, usize>,
    threshold: usize,
}

impl Initialisation for BerkMin {
    fn initialise(_clauses: &Clauses, _variables: &Variables) -> Self where Self: Sized {
        BerkMin {
            activity: HashMap::new(),
            threshold: 60,
        }
    }
}

impl Update for BerkMin {
    fn on_learn(&mut self, _learned_clause: &Clause, _clauses: &Clauses, _variables: &Variables) {
        self.activity.entry(self.activity.len()).or_insert(0);
    }

    fn on_conflict(&mut self, empty_clause: ClauseId, _clauses: &Clauses, _variables: &Variables) {
        self.activity.entry(empty_clause).and_modify(|x| *x += 1).or_insert(1);
    }
}

impl ClauseDeletionStrategy for BerkMin {
    fn delete_clause(&mut self, clauses: &mut Clauses, _variables: &mut Variables) -> Vec<ClauseId> {
        let young = &clauses[0..&clauses.len() / 16];
        let old = &clauses[&clauses.len() / 16..clauses.len()];

        let (young_out, young_in) : (Vec<(usize, &Clause)>, Vec<(usize, &Clause)>) = young
            .iter()
            .enumerate()
            .partition(|(i, c)| c.literals.len() > 42 && self.activity[i] < 7);
        let (old_out, old_in) : (Vec<(usize, &Clause)>, Vec<(usize, &Clause)>) = old
            .iter()
            .enumerate()
            .partition(|(i, c)| c.literals.len() > 8 && self.activity[i] < self.threshold);

        self.threshold += 1;

        self.activity = old_in.iter().chain(young_in.iter())
            .enumerate()
            .map(|(new_i, (i, _))| (new_i, self.activity[i]))
            .collect();

        old_out.iter().chain(young_out.iter()).map(|(i, _)| *i).collect()
    }
}
