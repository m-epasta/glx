# GLX COMPILER

This crate contains all compiler core features.

## TESTS

There are two types of tests:
- integrations tests, located under tests/ They do *real* tests on compiler features
- unit tests, embedded into source code files, they test the compiler API

## AST

HTML AST is not considered as JS. Instead, it gets treated as HTML with SSR calls that all gets verified at build time (builder/)
