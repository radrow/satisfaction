use super::{
    BranchingStrategy,
    super::{
        util::PriorityQueue,
        variable::{VariableId, Variables},
        clause::{ClauseId, Clauses},
        update::{Initialisation, Update},
    },
};
use crate::CNFVar;
use itertools::Itertools;

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

