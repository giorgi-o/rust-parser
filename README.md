# Names and UNIs of teammates

- Ifesi Onubogu (io2249)
- Giorgio Cavicchioli (gc3137)

# 1. Language Grammar

Here is the CFG for our language:

```plaintext
Program ::= Region | Program Region

Region ::= "region" Identifier "{" RegionBody "}"

RegionBody ::= RegionItem*
RegionItem ::= Function | Stmt

Function ::= "function" Identifier "(" Parameters ")" "{" StmtList "}"

Parameters ::= ε | Parameter | Parameters "," Parameter
Parameter ::= Identifier

StmtList ::= Stmt*

Stmt ::= "if" Expr "{" StmtList "}" "else" "{" StmtList "}"
       | "if" Expr "{" StmtList "}"
       | "for" "(" "let" Identifier "=" Expr ";" Expr ";" Identifier "=" Expr ")" "{" StmtList "}"
       | "return" Expr ";"
       | "let" Identifier ";"
       | "let" Identifier "=" Expr ";"
       | Identifier "=" Expr ";"
       | Expr ";"

Expr ::= AddExpr
AddExpr ::= AddExpr "+" CmpExpr | CmpExpr
CmpExpr ::= CmpExpr "<" Term | Term
Term ::= DotExpr
DotExpr ::= DotExpr "." Identifier "(" ExprList ")" | Factor

Factor ::= Number 
        | StringLiteral 
        | "[" "]" 
        | "[" ArrayElements "]"
        | Identifier "(" ExprList ")"
        | Identifier
        | "(" Expr ")"

ArrayElements ::= Expr | ArrayElements "," Expr
ExprList ::= ε | Expr | ExprList "," Expr

Identifier ::= [a-zA-Z_][a-zA-Z0-9_]*
Number ::= -?[0-9]+
StringLiteral ::= "[^"]*"
```

# 2. Parsing algorithm

We use the [Larlpop](https://github.com/nikomatsakis/lalrpop) library (pronounced "lollipop"), which is a library implementation of an LR(1) parser, using the CFG above as input (defined in `src/grammar.larlpop`), and emitting Rust code as output.

We provide 2 ways of running our parser: using Rust/Cargo, and using Docker. Both methods are described below.

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

# 3. Sample input programs

We provide several sample input programs in the `example_input_source_code` directory, along with their matching AST outputs in `example_output_source_code`.


# 6. Demo Video

https://github.com/user-attachments/assets/e08b7f1e-6681-47a9-82f0-68f72ffcae95
