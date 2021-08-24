## What is Cythan

Cythan is a turing like abstract machine that aims to be even simpler and generic.
It merges the instruction pointer and the memory. So everything is just memory.

## What is complex when programming in Cythan

Since cythan has no addition, substraction, increment, decrement or even if.
We need to use trick to simulate them.
For instance, increments links a value a the next only memory copies are done.

## CH2L (pronunced as Shell) (Cythan High Level Language)

This repo contains a full CH2L to CythanV3 compiler.
To compile CythanV3 assembly: [https://github.com/Cythan-Project/cythan-compiler](https://github.com/Cythan-Project/cythan-compiler)
Or to use it online: [https://ccgauche.github.io/cythan](https://ccgauche.github.io/cythan)

A simple program:

```rust
exit(
    if0(1, :3, :4)
);
```

> This program will exit with a 4 since 1 isn't a 0

## The ICL (Internal Compiler Library) (hard-coded in CythanV3)

```rust
((Exits the program with the given code))
exit(<number>)
((Set the value of a register from a variable ))
set_reg(<number>, <value>)
((Push the value of a register to a variable ))
get_reg(<&*variable>, <number>)
((Will execute the first block if the value is 0 or else the second one if it exists))
if0(<value>, <block if true>, <OPTIONAL: block if false>)
((Will load the following file in scope))
include(<file to load>)
((Will set or [create a variable if it doesn't exists in scope])))
set(<&*variable>,<value>)
((Will [set in scope only] or [create a variable if it doesn't exists in scope]))
let(<&*variable>,<value>)
((Will create a function in the scope))
fn(<name>, <arguments...>, <code block>)
((Will increment the variable ref))
inc(<&variable>)
((Will decrement the variable ref))
dec(<&variable>)
(( Will execute the code block until break() is called,
will restart the execution when continue() is called ))
loop(<code block>)
(( Will restart the current loop))
continue()
(( Will exit the current loop))
break()
```

### Types

#### Literal

A literal is a String that follows this rule:

```rust
'A'..'Z' | 'a'..'z' | '0'..'9' | "." | "_" | "/" | "$"
| "-" | "!" | "+" | "*" | "#" | "$" | "=" | "[" | "]"
| "@" | "?" | "*" | "%" | "\\" | "^" | "<" | ">" | "~" | "&"
```

Note that they are far more permissive than in other languages. (For instance Java has only `'A'..'Z' | 'a'..'z' | '0'..'9' | "_"`)

#### Number

Depending on your current cythan value max size a value is between 0 and this size. (Default 16)
On the standard Cythan you can't use other numbers than 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15.

#### Variable

A variable is a standard literal

#### Arguments

Arguments are variables but if they start with & or &\* they behave diferently:

- `&` Instead of copying the input value when calling the function will reference the variable
  and allow modifications to it
- `&*` Same as before but if the variable doesn't exist it will be created

#### Files

They are just literal that points toward a file

#### Function calls

Everything in Cythan is a function call and they should either be in another function call or ended with a `;`
Operators are syntaxic sugar for function calls so the same rules apply.

#### Code blocks

Blocks can be defined in two ways:

- The multi-line block that uses `{code}`
  Don't forget to add `;` between function calls in code blocks
- The single expression block that uses `:code`

Code blocks have return value this means:

```
a = {
    b = 1;
    b += 1;
    b
};
```

> This example uses STD
> Here the a variable will be set to the value to b.
