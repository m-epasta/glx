# ARCHITECTURE
<sub>This file contains all specifications/links' specifications of the global architecture of the project </sub>

## PRELUDE

The architecture is a bit different from gleam source code or even some others JS frontend frameworks. Like for example, in gleam and Vue, compiler is divided in compiler-cli/ and compiler-core/ while glx has compiler/ (IO and non-IO stuff) and cli/ which is just the caller of the compiler (btw, anyone that use compiler as a crate, can do its own CLI). 
