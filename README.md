# CompiledFiles

[![Actions Status](https://github.com/schultetwin1/compiledfiles/workflows/CI/badge.svg)](https://github.com/schultetwin1/compiledfiles/actions)

A rust library to return a list of all the source files listed in the symbols
of a native compiled file.

For example, a simple main.c such as the following

```c
include <stdio.h>

int main(int argc, const char* argv[]) {
    printf("Hello, World\n");
    return 0;
}
```

compiled with GCC, would return 

* `/home/matt/dev/examples/simple_c/main.c`
* `/usr/include/stdio.h`
* `/usr/include/x86_64-linux-gnu/bits/types/FILE.h`
* `/usr/include/x86_64-linux-gnu/bits/types/struct_FILE.h`

# Supported Systems

This library is cross platform, and can be used on a Windows, Linux, or Mac
host. However, there are many tools that generate different symbols files and
not all are currently supported.

## Supported Compilers

The following compilers are currently supported:

* GCC
* MSVC
* Clang

No versioning check has been done yet to ensure the symbol files they
generate are compatiable across all versions.

## Supported Languages

The only supported languages currently are C/C++ though other languages may
just work. Due to the nature of this project, Rust is next on the list for
support.

## Supported Formats

The following symbol formats are currently supported

* Elf
* PDB

Mach-O files are next in line. Also, split dwarfs have not yet been tested.