# Names and UNIs of teammates

- Ifesi Onubogu (io2249)
- Giorgio Cavicchioli (gc3137)

# 1. Code generation algorithm

We wrote a code generation algorithm to turn the parsed AST into output Rust code. The Rust code is then compiled into a Python module using [pyo3](https://pyo3.rs/v0.23.3/) and [maturin](https://www.maturin.rs/).

We provide 6 sample programs (5 correct, 1 error) in `example_input_source_code/`, and their matching outputs are in `example_output_source_code/`. These cover all the language features supported by our language including regions, functions, if and if/else statements, for loops, dynamic typing and return statements.

The strength of using the Rust language as our compiler's output is that we ensure the output code is extremely robust and takes full advantage of all of Rust's safety guarantees, combined with C-like performance. A major challenge was that our language is dynamically typed, while Rust requires very strong type annotations due to its type system. However, our language works great and exceeded our expectations for its performance and ease of use.

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

# 2. Sample input programs

We provide several sample input programs in the `example_input_source_code` directory, along with their matching generated code in `example_output_source_code`.


# Demo Video

https://github.com/user-attachments/assets/3431e766-3fdf-455e-b118-e31078ef663e

