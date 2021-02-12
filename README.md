# Rust/SAT-Solving
By Group F: Radosław Rowicki, Alexander Lankheit, Korbinian Federholzner

# Project Discription

The goal of this project is to build a solver for the puzzle Tents,
by encoding it as a SAT problem. 

## Satisfacton Solver

A DPLL based SAT-Solver for solving simple CNF encoded SAT Problems.

## Tents

The game tents is played on an n × m grid, with
a number written next to each row and column. Initially, each cell can either
be empty or contain a tree. Tents should be placed in empty cells, such that:
* No two tents are adjacent in any of the (up to) 8 directions.
* The number of tents in each row/column matches the number specied.
* It is possible to match tents to trees 1:1, such that each tree is orthogonally adjacent to its own tent (but may also be adjacent to other tents).


# Project Structure

The project consists of three parts:

* __solver__:
    The library containing the code for the actual solver.

* __solver-cli__:
    A binary executing the solver via command line.

* __tents__:
    A binary that uses SAT-solving to solve [Tents puzzles](https://brainbashers.com/tents.asp).

* __solver-bench__: 
    A benchmarking tool, for benchmarking the SAT-Solver Heuristics

# Executables

## `solver-cli`

### Usage

Basic CLI for our solver.

Execution:
```
cargo run --release --bin solver-cli -- [ARGS]
```
    
Supported arguments:
    
  * `-i VALUE` or `--input=VALUE` – points to the input file with the
  description of the SAT formula. If not specified, STDIN will be used
  instead.
  
  * `--algorithm=VALUE` – chooses the algorithm. Following options are available:
    - `bruteforce` – plain bruteforce "try, check, backtrack"
    - `cadical` – references to the [cadical implementation](http://fmv.jku.at/cadical/)
    - `satisfaction` – our prioprietary DPLL implementation. Can be adjusted with
    `--branch` option to choose heuristics
    
   Defaults to `satisfaction`.
    
  * `--branch=VALUE` – tweaks the branching heuristics for `satisfaction` solver.
  Possible options:
    - `naive`
    - `DLIS`
    - `DLCS`
    - `MOM`
    - `Jeroslaw-Wang`
    
  Defaults to `DLCS`
  
  * `-r` or `--return-code` – if set, instructs the solver to exit with return code `1`
  if the given formula is satisfied, `0` if not or there was an execution error.

### Return value

Normally the solver returns `0` if the formula was processed successfully and `2` on
execution error. If the `-r` or `--return-code` flag is set, this behaviour is changed [accordingly to the documentation](#Usage).


## `solver-bench`

Benchmarking utility. Will create an SVG plot file with comparison of several solvers.

Execution:
```
cargo run --release --bin solver-bench -- [ARGS]
```

Supported arguments:

  * `-i VALUE` or `--input=VALUE` – directory with test files
  
  * `-o VALUE` or `--output=VALUE` – output file for the plot
  
  * `-t VALUE` or `--time=VALUE` – time limit for a single instance in seconds

## `tents`

An example GUI application that solves the famous [tents puzzle](https://brainbashers.com/tents.asp).

Execution:
```
cargo run --release --bin tents
```