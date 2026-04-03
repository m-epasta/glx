#!/usr/bin/bash
#
# Small script that verifies that:
# - gleam is installed
# - at least one of JS runtimes is installed

# Check if gleam is installed
if ! which gleam &>/dev/null; then
  echo "gleam not installed"
  exit 1
fi

# Check for JS runtimes (Node.js, Deno, or Bun)
if ! which node &>/dev/null && ! which deno &>/dev/null && ! which bun &>/dev/null; then
  echo "No JS runtime installed. Please install Node.js, Deno or Bun"
  exit 1
fi
