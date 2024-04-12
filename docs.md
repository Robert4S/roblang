# Documentation

## Basic syntax
At the moment, declaration seperate from assignment is not supported.

Variables (including functions) are assigned with the following syntax:

let foo: Type = value;

eg. let mynum: Number = 12;

Functions are similarly defined, with their value looking like the following:

(parameter: Type) -> Returntype {\
    &nbsp;function body\
    &nbsp;Return ReturnValue;\
}

An example function would be:

let adder: Function = (foo: Number, bar: Number) -> Number {\
    let added: Number = foo + bar;\
    return added;\
}
