# Rust/SAT-Solving

The project consists of three parts:

* __solver__:
    The library containing the code for the actual solver.

* __solver-cli__:
    A binary executing the solver via command line.

* __tents__:
    A binary that uses SAT-solving to solve _Tents_-puzzles.

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
    - `cadical` – references to the cadical implementation
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
execution error. If the `-r` or `--return-code` flag is set, this behaviour is changed [accordingly to the documentation](###Usage).


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

An example GUI application that solves the famous "tents" puzzle.

Execution:
```
cargo run --release --bin tents
```
