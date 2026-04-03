# COMPILER BUILDER

This the result of a design note (TODO). Since it doesnt have compiler-core and compiler-cli, we use builder to assemble our scripts and to build the project. This like the bundler of or compiler, but the compiler does not really compiles. We have to do that since the builder really interact with gleam CLI to build and structure correctly the artifacts. The runtime has it's own folder and is not related to this module (atleast now).
