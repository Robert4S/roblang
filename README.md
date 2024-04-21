# Roblang compiler and language

I am an IB computer sciences student, and this compiler is a pet project of mine. It is being developed as learning exercise, and not as a useful tool, as I do not have the time nor expertise to create a language or compiler that provides any genuine advantage over any already established languages. Feel free to raise any issues or suggestions if you do decide to try it out.

Note:
The language is very limited. Don't expect it to be able to do anything useful. Also, there are some horrifying snippets in the source code from when I got lazy and wanted to implement something quickly.

All your RAM are belong to me (egregious use of .clone() because lifetimes scare me).

(I'm not following a compiler book, and I'm lazy. That is why the code is shitty.)

## Installation requirements
Linux (maybe windows support sometime, probably not mac.)\
GCC (support for other C compilers coming soon)\
Cargo (rusts build system)

## Installation
1. Clone the repo
2. run cargo build --release
3. create the static library files for the libraries in ctests. vec.c must be libvec.a
4. add the roblang/target/release directory to path
5. link an environment variable ROBSTD to the roblang/ctests directory
6. link an environment variable CVEC to the roblang/ctests/c-vector directory
7. all good

[Documentation](docs.md)
