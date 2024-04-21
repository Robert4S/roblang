# Documentation

## Variable and function assignment
At the moment, declaration seperate from assignment is not supported.\
\
Variables (including functions) are assigned with the following syntax:\
\
let foo: Type = value;\
\
eg. let mynum: Number = 12;\
\
### Functions are similarly defined, with their value looking like the following:\
\
\
(parameter: Type) -> Returntype {\
    &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;function body\
    &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;Return ReturnValue;\
}

An example function would be:\
\
let adder: Function = (foo: Number, bar: Number) -> Number {\
    &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;let added: Number = foo + bar;\
    &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;return added;\
}

Functions can only return identifiers.\
\
Variables are immutable. If you want to change a variable with control flow, make a function that does it.\

## Control flow

### Boolean expressions can be used directly in an if statement, but it can only compare identifiers.\
Else if does not exist. You must create an if statement inside of an else block.\
\
\
let foo: Number = 10;\
\
let bar: Number = 20;\
\
if foo == bar {\
    &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;showme("10 is 20\n");\
} else {\
    &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;showme("10 is not 20.\n");\
}

## Inline c
### You can add inline c with the inline keyword. Variables and functions defined in roblang will have the same names in C.\
\
\
let x: Number = 10;\
\
inline "printf(\"Ten is %d\n \", x);";
