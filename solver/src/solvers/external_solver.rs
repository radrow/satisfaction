use tokio::process::Command;
use tokio::io::{AsyncReadExt,AsyncWriteExt};
use std::process::Stdio;
use std::path::PathBuf;
use crate::{Solver, SATSolution, CNF};
use crate::solvers::InterruptibleSolver;
use async_trait::async_trait;

pub struct ExternalSolver {
    program: PathBuf,
}

impl ExternalSolver {
    pub fn new(program_name: PathBuf) -> ExternalSolver {
        ExternalSolver {
            program: program_name,
        }
    }
}

#[async_trait]
impl InterruptibleSolver for ExternalSolver {
    async fn solve_interruptible(&self, formula: &CNF) -> crate::SATSolution {
        let mut program = Command::new(&self.program);
        program
            .kill_on_drop(true)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped());

        let mut child = program
            .spawn()
            // TODO: Real error handling
            .expect("Could not create process");

        let mut stdin = child.stdin.take().unwrap();
        let input = formula.to_dimacs();
        stdin.write_all(input.as_bytes()).await
            .unwrap();

        // TODO: Handle error
        let output = child.wait_with_output().await
            .unwrap();
        

        SATSolution::Unknown
    }
}
