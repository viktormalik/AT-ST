# AT-ST: Automatic Testing of Student Tasks

![Build](https://github.com/viktormalik/at-st/actions/workflows/ci.yml/badge.svg?branch=master)

AT-ST is a tool for automated execution and evaluation of programming tasks,
mainly designed for high school and university teachers. It allows to easily
specify a list of test cases and code analysers to run.

Currently, the project is in an early stage of development and it only supports
CLI projects written in C.

## Usage

AT-ST is a CLI application written in Rust that can be easily executed using
Cargo:
```
$ cargo run <path-to-project> <config-file>
```

`path-to-project` must contain `config-file` and a number of sub-directories
that contain individual students' solutions. `config-file` is a YAML file that
contains configuration of the evaluation (most importantly the test cases to
run).

## Supported project configuration

Currently, AT-ST allows the following configuration:
- name of the main source file (projects consisting of multiple source files are
  not supported, yet),
- rules to select the student solutions,
- compiler options,
- test execution configuration,
- list of **test cases** to execute each solution on (see below for supported
  features of test cases).
- list of **source code analysers** to run (see below for the list of supported
  analysers),

See [configuration file syntax](docs/config_syntax.md) for a detailed
description of the configuration format.

### Test cases

In general, each test should specify input, expected output, and score. A
solution is executed with the given input and if the produced output matches the
expected one, the solution is awarded the provided score.

Supported input options are:
- the list of arguments to pass to the program,
- the text to pass to program's standard input.

Supported expected output options are:
- the contents of the program's standard output.

### Source code analysers

Code analysers are mainly designed to give a solution a penalty if the code
does not match some rules.

Currently supported analysers are:
  - *no call* - checks that the code does not contain calls to certain
    (external) functions,
  - *no header* - checks that the program does not include certain headers,
  - *no globals* - checks that the program does not use any global
    variables.

## Tests

The project features unit and integration tests that can be executed by running:
```
$ cargo test
```
