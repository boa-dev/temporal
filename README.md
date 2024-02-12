# Temporal in Rust

:warning: This crate is highly experimental and NOT stable. Do not use in production without extreme caution :warning:

`Temporal` is a calendar and timezone aware date/time library that is currently being designed and proposed as a new
builtin to the `ECMAScript` specification.

This crate is an implementation of `Temporal` in Rust. While initially developed for the `Boa`, the crate has been externalized
as we intended to make an engine agnostic and general usage implementation of `Temporal` and its algorithms.

IMPORTANT NOTE: The Temporal Proposal is still a Stage 3 proposal. As such, regardless of current state of this repository,
please refrain from using without flags.

## Temporal Proposal

Relevent links regarding Temporal can be found below.

-[Temporal Documentation](https://tc39.es/proposal-temporal/docs/)
-[Temporal Specification](https://tc39.es/proposal-temporal/)
-[Temporal Repository](https://github.com/tc39/proposal-temporal)

## Disclaimer / Warning

This crate should be viewed as highly experimental and unstable. As such, we can make no API guarantees
until a stable `0.1.0` version of the crate has been released.

## Contributing

This project is open source and welcomes anyone interested to participate. Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for more information.

## Communication

Feel free to contact us on [Discord](https://discord.gg/tUFFk9Y).

## License

This project is licensed under the [Apache](./LICENSE-Apache) or [MIT](./LICENSE-MIT) licenses, at your option.
