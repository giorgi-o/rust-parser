# Names and UNIs of teammates

- Ifesi Onubogu (io2249)
- Giorgio Cavicchioli (gc3137)

# 1. Code optimisation techniques

We implemented 4 of the optimisation techniques described in class, and an additionnal one that is specific to our program. For each one, we provide an example program in `example_input_source_code/` and `example_output_source_code`. When relevant, we also include an edge case to show that our compiler only optimises the code in cases where the optimisation is valid.

All our optimisations are implemented in `src/clean_ast.rs`.

**Note:** Throughout our examples, we make heavy use of the [`blackbox()`](https://doc.rust-lang.org/std/hint/fn.black_box.html) function. This function, for demonstration purposes, prevents our compiler from over-optimising our sample programs, and allows us to showcase only one optimisation per example. At runtime, this function is the identity function and just returns the input value.

## a. Algebraic simplification

Our code automatically detects expressions such as `x + 0` and `y * 1`, and removes the unecessary computations, leaving only `x` and `y` respectively.

An example of this optimisation can be found in `algebraic_simpl.txt`.

## b. Common subexpression elimination

Our code detects common subexpressions in the code, and replaces them with a variable that stores the result of the computation. This way, the computation is only done once, and the result is reused. Our code also checks that the variables used in the common subexpression have not changed between the two computations.

An example of this optimisation can be found in `cse.txt`.

## c. Loop invariant motion

Our code detects expressions that are computed inside a loop, but whose value does not change during the loop. It then moves the computation outside of the loop, to avoid recomputing the same value multiple times.

An example of this optimisation can be found in `loop_invariant.txt`.

## d. Unreachable code elimination

Our code detects code that is never reached (due to all possible paths leading to it being blocked by a return statement), and removes it from the generated code. This helps optimise the size of the generated code.

An example of this optimisation can be found in `unreachable_code.txt`.

## e. Smart type conversions

This optimisation is specific to our language. Since our language is weakly typed, we allow any variable to be of any type, and to change types. We also do not require the programmer to indicate the type of declared variables.

However, our generated code is in Rust, a strongly typed language. This causes issues as we need to know the type of each variable at compile time, in order to properly annotate their types in the strongly-typed generated code.

The workaround we used was to cast all variables, including simple numbers, into `PyAny`, which defers the type checking to runtime. However this caused poor performance as all variables needed to be turned into `PyAny` when they were assigned, then downcasted to their actual type when they were used, even for simple expressions such as `1 + 2`.

To solve this issue, we implemented a smart type conversion system. Our system keeps track of the type of each variable, and only casts them to `PyAny` when they are used in a context where their type is unknown. This way, we avoid unnecessary type conversions, and only cast variables when necessary.

This also opened the door to more advanced optimisations. For example, when converting a `Vec<T>` (vector of Ts) into a `PyList`, we can now check if all elements of the vector are of the same type, and if so, cast the whole vector into a `PyList` of that type. This allows the reuse of the memory allocated by the vector, and avoids the need to cast each element individually.

An example of this optimisation can be found in `full.txt`.

# Running our compiler

We provide 2 ways of running our compiler: using Rust/Cargo, and using Docker. Both methods are described below.

## Using Rust/Cargo

Rust and Cargo (the package manager) can be installed using Rustup (the rust installer). You can find the instructions [here](https://www.rust-lang.org/learn/get-started).

Once installed, simply run `cargo run -- example_input_source_code/[file].txt`. The command will download all dependencies, compile the program, and run it on the specified file.

## Using Docker

A dockerfile is provided to run our parser in a docker container. To use it, first open the `Dockerfile` file and change which example source code to run it on:

```dockerfile
# Run the specified Rust file
CMD ["cargo", "run", "--", "example_input_source_code/full.txt"] # <- change this
```

Then, to build and run the container:

```bash
docker build --tag plattr-parser .
docker run plattr-parser
```

These two commands are also provided in the `run.sh` script.

