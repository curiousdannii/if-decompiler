Glulxtoc - Glulx to C decompiler
================================

Glulxtoc will decompile your Glulx storyfile into C code which you can then compile against any Glk library.

To get it, first [install Rust](https://rustup.rs/) and then install glulxtoc with cargo:

```
cargo install glulxtoc
```

Usage
-----

```
glulxtoc <path> [FLAGS] [OPTIONS]
```

Required option:

- path to storyfile

Flags:

- `-d`, `--disassemble`: Disassembler mode - only disassemble, do not optimise or generate structured code

Options:

- `--debug-file`: path to an Inform debug file for the storyfile
- `--out-dir`: Output folder. If not given will make a folder based on the storyfile's name with `.decompiled` added to the end
- `--stack-size`: Stack size in MB (default 8), for the glulxtoc app (not the stack of the Glulx file being decompiled.) Very large storyfiles may cause the glulxtoc app to have a stack overflow, in which case pass this option.
- `--safe-function-overrides`: An array of function addresses to forcibly set as safe, overriding the decompiler's heuristics. Example, `--safe-function-overrides=1234,5678`
- `--unsafe-function-overrides`: An array of function addresses to forcibly set as unsafe, overriding the decompiler's heuristics.

Compiling the output code
-------------------------

Glulxtoc produces several C files and provides a CMake CMakeLists.txt. You must pass in the Glk library's path to CMake. For example:

```
glulxtoc advent.ulx
cd advent.ulx.decompiled
mkdir remglk
cmake -DGlkLibPath=../../remglk . -B remglk
cd remglk
make
```

Limitations
-----------

In general Glulxtoc is likely to have problems with any Glulx files that weren't compiled with Inform.

- No functions in RAM
- Functions and strings can't be interleaved - will stop decoding once the first string is found
- No 1 and 2 byte locals
- Computed branch and jump offsets are only supported when you supply an Inform debug file
- Inter-function branches are only supported when you manually set the target function as unsafe
- State changing opcodes (save, restart, etc) within functions called by strings

Troubleshooting
---------------

- If you get an error in the decompilation stage (such as an unknown opcode), try passing in an Inform debug file. If you provide one, consider using the [reduce-debug-xml.sh](https://github.com/curiousdannii/if-decompiler/blob/master/tools/reduce-debug-xml.sh) tool to cut back the debug data to only what Glulxtoc makes use of. This is not required, but will make Glulxtoc run faster.
- If the `make` stage of compilation is very slow, try Clang. GCC has [a bug](https://gcc.gnu.org/bugzilla/show_bug.cgi?id=100393) which makes Glulxtoc's output compile very slowly.
- If it compiles without error, but does not run properly, see if switching Glulxtoc to the disassembler mode (`-d`) fixes things. If it does then that indicates a bug in Glulxtoc's decompilation optimisation code.

If you do get an error, please post a [bug report](https://github.com/curiousdannii/if-decompiler/issues) with as much detail as you can provide, and ideally with your storyfile.