# Changelog

Dates are UTC. One line per release, what changed and why, so a reader of the diff has the
sentence beside it.

## Unreleased

- First version.
- The `image` module and `examples/host_directory.rs`: a plan of the test image that a harness
  can stage through any filesystem, and a verifier that reads it back.

## BUGS

- The template's lag check guards the CI pin, not this file; a change that matters to a user of
  the crate belongs in a release line here.
