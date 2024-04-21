# Roblang compiler and language

I am an IB computer sciences student, and this compiler is a pet project of mine. It is being developed as learning exercise, and not as a useful tool, as I do not have the time nor expertise to create a language or compiler that provides any genuine advantage over any already established languages. Feel free to raise any issues or suggestions if you do decide to try it out.

Note:
The language is very limited. Don't expect it to be able to do anything useful. Also, there are some horrifying snippets in the source code from when I got lazy and wanted to implement something quickly.

All your RAM are belong to me.

## Installation requirements
Linux (maybe windows support sometime, probably not mac.)\
GCC (support for other C compilers coming soon)\
[Cargo](https://www.rust-lang.org/tools/install) (rusts build system)

## Installation
1. Clone the repo
2. Run "cargo build --release"
3. Create the static library files for the libraries in ctests. vec.c must be libvec.a (if you are using ubuntu, you do not have to do this.)
4. Ddd the roblang/target/release directory to your path
5. Create an environment variable ROBSTD linking to the roblang/ctests directory
6. Create an environment variable CVEC linking to the roblang/ctests/c-vector directory
7. all good

## Examples
I am not even really sure what it can or cant do, but the prog.rob file is a decent showcase of the current functionality.

## [Documentation](docs.md)
