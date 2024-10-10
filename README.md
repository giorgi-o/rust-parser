# Lexical grammar

Here are the different types of tokens in our language, in regex form:

* **Identifier:** `[a-zA-Z][a-zA-Z0-9]*`
* **Number:** `-?[0-9]+(.[0-9]+)?`
* **Keyword:** `region|let|function|return|if|else|for`
* **Operator:** `+|-|*|/|=|<|>|<=|>=`
* **Lcur, Rcur:** `{`, `}`
* **Lpar, Rpar:** `(`, `)`
* **Lbrack, Rbrack:** `[`, `]`
* **Semi:** `;`
* **Comma:** `,`
* **Dot:** `.`

# How to run the tokeniser

Our program is written in Rust. We provide two ways to run it, either using the Rust compiler, or using Docker.

We also provide 6 example programs to try it on in the `examples/` folder, each showcasing different features of our language and how the tokeniser handles them. Additionally, the `error.txt` file showcases a scenario where it will fail to tokenise.

## Using Rust/Cargo

Rust and Cargo (the package manager) can be installed using Rustup (the rust installer). You can find the instructions [here](https://www.rust-lang.org/learn/get-started).

Once installed, simply run `cargo run -- examples/[file].txt`. The command will download all dependencies, compile the program, and run it on the specified file.

## Using Docker

A dockerfile is provided to run our tokeniser in a docker container. To use it, first open the `Dockerfile` file and change which example source code to run it on:

```dockerfile
# Run the specified Rust file
CMD ["cargo", "run", "--", "examples/full.txt"] # <- change this
```

Then, to build and run the container:

```bash
docker build --tag plattr-tokeniser .
docker run plattr-tokeniser
```

These two commands are also provided in the `run.sh` script.

# Example output

Example for a successful tokenisation:

```bash
$ cargo run -- examples/full.txt
# ... a bunch of other output

3. Tokens:
<Keyword, region> <Identifier, DataManagement> <Lcur, {> <Keyword, function> <Identifier, allocate> <Lpar, (> <Identifier, size> <Rpar, )> <Lcur, {> <Keyword, let> <Identifier, buffer> <Operator, => <Identifier, allocateMemory> <Lpar, (> <Identifier, size> <Rpar, )> <Semi, ;> <Keyword, return> <Identifier, buffer> <Semi, ;> <Rcur, }> <Keyword, function> <Identifier, free> <Lpar, (> <Identifier, ptr> <Rpar, )> <Lcur, {> <Identifier, freeMemory> <Lpar, (> <Identifier, ptr> <Rpar, )> <Semi, ;> <Keyword, return> <Number, 0> <Semi, ;> <Rcur, }> <Keyword, function> <Identifier, borrow> <Lpar, (> <Identifier, ptr> <Rpar, )> <Lcur, {> <Keyword, if> <Identifier, isMemoryFreed> <Lpar, (> <Identifier, ptr> <Rpar, )> <Lcur, {> <Keyword, return> <Number, 1> <Semi, ;> <Rcur, }> <Keyword, return> <Identifier, ptr> <Semi, ;> <Rcur, }> <Keyword, function> <Identifier, processStream> <Lpar, (> <Identifier, streamSize> <Comma, ,> <Identifier, blocksize> <Rpar, )> <Lcur, {> <Keyword, let> <Identifier, streamPtr> <Operator, => <Identifier, allocate> <Lpar, (> <Identifier, streamSize> <Rpar, )> <Semi, ;> <Keyword, let> <Identifier, blocks> <Operator, => <Lbrack, [> <Rbrack, ]> <Semi, ;> <Keyword, for> <Lpar, (> <Keyword, let> <Identifier, i> <Operator, => <Number, 0> <Semi, ;> <Identifier, i> <Operator, <> <Identifier, streamSize> <Semi, ;> <Identifier, i> <Operator, => <Identifier, i> <Operator, +> <Identifier, blockSize> <Rpar, )> <Lcur, {> <Keyword, let> <Identifier, blockPtr> <Operator, => <Identifier, borrow> <Lpar, (> <Identifier, streamPtr> <Operator, +> <Identifier, i> <Rpar, )> <Semi, ;> <Identifier, blocks> <Dot, .> <Identifier, push> <Lpar, (> <Identifier, blockPtr> <Rpar, )> <Semi, ;> <Rcur, }> <Keyword, return> <Identifier, blocks> <Semi, ;> <Rcur, }> <Rcur, }>
```

Example for a failed tokenisation:

```bash
$ cargo run -- examples/error.txt
# ... a bunch of other output

Error parsing file examples/error.txt:2:13 while parsing token: 123a
```

# Overview of the code

`src/main.rs` is the program's entry point. It contains the definition of the `Token`, `Keyword` and `Operator` enums. The actual `main()` function (line 126) first does some minor pre-processing (removing comments), then calls `Tokeniser::tokenise()` and prints the output.

`src/token_fsm.rs` contains the FSM logic for the tokeniser. The file is very thoroughly commented and should be easy to follow.
