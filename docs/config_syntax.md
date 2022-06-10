# Configuration file syntax

Configuration file is a YAML file whose top element is a dictionary that defines
individual configuration options.

The supported options and their syntax is:

- Name of the source file (mandatory):
```yaml
source: file.c
```

- Rules to select solutions - by default all sub-directories of the project
  directory are selected, this option allows to exclude specific directories.
```yaml
solutions:
    exclude-dirs: [ dir1, dir2, ... ]
```

- Compiler information - compiler name, compilation flags, linker flags. The
  supported options respect standard Makefile variable names. All fields are
  optional, the default compiler is GCC without any additional flags.
```yaml
compiler:
    CC: gcc
    CFLAGS: -Wall -Wextra
    LDFLAGS: -lm
```

- List of tests - the only mandatory field for each test is `score`, however at
  least some input (`args` or `stdin`) and output (`stdout`) should be specified
  so that the test can be reasonably evaluated.
```yaml
tests:
    - name: first test
      score: 1.0
      args: --some-argument
      stdin: some text
      stdout: expected text
    - name: second test
      score: 2.0
      stdin: |
        some
        multiline
        text
      stdout: |
        multiline
        output
    - name: third test
      score: 1.0
      stdin: </path/to/file    # content of the file is passed to stdin
      stdout: </path/to/file   # stdout will be compared to content of the file
    - name: test with multiple cases
      score: 1.0
      test-cases:
        - args: --arg1
          stdin: input1
          stdout: output1
        - args: --arg2
          stdin: input2
          stdout: output2
      require: all            # how many cases must pass to get the points
                              # possible values: "any", "all"
    - name: test with shell input
      score: 1.0
      test-cases:
        - stdin: $(echo err)  # passes "err" to stdin
          stderr: "*"         # matches any string at stderr
```

- Configuration of tests execution. Supports the following settings:
  - Timeout - specifies the time in milliseconds after which the solution
    execution on a test case is killed. The default value is 5 seconds.
```yaml
test-config:
    timeout: 1000 # 1 second
```

- Lists of source code analyses. Each analyser has its own fields, however an
  analysis should specify the analyser name and the penalty to give to the
  solution (if the analyser passes).
```yaml
analyses:
    - analyser: no-call
      funs: [ fopen, fclose ]
      penalty: -0.5
    - analyser: no-header
      header: string.h
      penalty: -1.0
    - analyser: no-globals
      penalty: -1.0
      except: [ .*err.* ]     # allows globals containing the "err" substring
```

You can find examples of project configurations in [integrations
tests](/tests/projects).

