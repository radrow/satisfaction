use super::{
    BranchingStrategy,
    super::{
        util::PriorityQueue,
        variable::{VariableId, Variables},
        clause::{ClauseId, Clauses},
        update::Update,
        abstract_factory::AbstractFactory,
    },
};
use crate::CNFVar;
use itertools::Itertools;

pub struct VSIDSInstance {
    resort_period: usize,
    branchings: usize,
    priority_queue: PriorityQueue<usize, VariableId>,
    scores: Vec<usize>,
    counters: Vec<usize>
}

unsafe impl Sync for VSIDSInstance {}
unsafe impl Send for VSIDSInstance {}

impl VSIDSInstance {
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

impl Update for VSIDSInstance {
    fn on_learn(&mut self, learned_clause: ClauseId, clauses: &Clauses, _variables: &Variables) {
        for lit in clauses[learned_clause].literals.iter() {
            self.counters[VSIDSInstance::literal_to_index(lit)] += 1;
        }
    }
    fn on_unassign(&mut self, literal: CNFVar, _clauses: &Clauses, _variables: &Variables) {
        let index = VSIDSInstance::literal_to_index(&literal);
        self.priority_queue.push(index, self.scores[index]);
    }
}

impl BranchingStrategy for VSIDSInstance {
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

            let scores = &self.scores;

            take_mut::take(&mut self.priority_queue, |pq| {
                pq.into_iter()
                    .map(|(id,_)| (id, scores[id]))
                    .collect()
            });
        }

        while let Some((index, _)) = self.priority_queue.pop() {
            let lit = VSIDSInstance::index_to_literal(index);
            if variables[lit.id].assignment.is_none() {
                return Some(lit);
            }
        }
        None
    }
}

pub struct VSIDS;

impl AbstractFactory for VSIDS {
    type Product = VSIDSInstance;

    fn create(&self, clauses: &Clauses, variables: &Variables) -> Self::Product {
        let mut scores = std::iter::repeat(0)
            .take(2*variables.len())
            .collect_vec();
        let counters = scores.clone();

        for clause in clauses.iter() {
            for lit in clause.literals.iter() {
                scores[VSIDSInstance::literal_to_index(lit)] += 1;
            }
        }

        let priority_queue: PriorityQueue<usize, VariableId> = scores
            .iter()
            .enumerate()
            .map(|(id, p)| (id, *p))
            .collect();

        VSIDSInstance {
            resort_period: 255,
            priority_queue,
            branchings: 0,
            scores,
            counters,
        }
    }
}
