# Contributing to Temporal in Rust

This project is currently highly experimental and fairly volatile. As
such, while we do welcome contributions, we prefer that you open up an
issue first beforehand to ensure the feature is not being actively
worked on by a maintainer.

If you're interested in helping out but don't see an issue that's for
you, please feel free to contact us on `Boa` Matrix server.

## Contributor Information

The Temporal proposal is a new date/time API that is being developed and proposed
for the ECMAScript specification. This library aims to be a Rust
implementation of that specification.

Due to the current experimental nature of the material and this library,
we would advise potential contributors to familiarize themselves with
the Temporal specification.

## Testing and debugging

For more information on testing and debugging `temporal_rs`. Please see
the [testing overview](./docs/testing.md).

## Diplomat and `temporal_capi`

If changes are made to `temporal_capi` that affect the public API, the
FFI bindings will need to be regenerated / updated.

To update the bindings, run:

```bash
cargo run -p diplomat-gen
```
