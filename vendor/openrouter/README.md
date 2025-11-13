# Warning

This project was created due to a personal need for it.
My use-case may not reflect that of everyone else, so features may be missing or could be untested.

Please leave issues if something does not work as expected.
You're also encouraged to contribute code if you can!

# Rationale

While OpenRouter can be used with OpenAI compatible clients,
those do not offer the whole feature spectrum OpenRouter provides.

Compared to those, this library might have a more opinionated structure though,
to make use of compile time features of Rust and generally provide a more ergonomic approach.

# State

- [x] most important endpoints and their response types are defined according to the spec
- [x] generation endpoint is tested on a basic level
- [ ] every feature of the generation endpoint is tested
- [ ] extensive documentation
- [ ] additional compile-time checks for any invariants OpenRouter expects
- [ ] helpers, builders and general convenience functions for the most used functionality.
- [ ] instead of `Option<T>` have the fields the same defaults as the spec and don't send them if they're the default

Basically the given OpenAPI and TypeScript definitions that can be found in OpenRouters docs
have been converted into Rust types, but there are no extensive tests.
There's also a helper type for when you don't have your own HTTP client.

# Documentation

OpenRouter has some usage related documentation, but it isn't perfect yet. I've been informed that
this will change soon-ish. In the meantime some implementation or types *could* be wrong.
Where possible tests have been added to verify we receive from the API what the specs say, but that isn't perfect.

# Testing

All tests that interact with OpenRouter require an `OPENROUTER_API_KEY` variable.
It can also be configured in a `.env` file.

If possible free endpoints are used, but that may not always be possible
in which case the test will fail for cost reasons.
It is recommended to set up a key with a low credit limit.
The `./responses` directory contains response json either directly
found in the docs or returned by the API. It is either a legacy named json,
or is correctly sorted in the respective directory and given a timestamp name.
Usually there should be a unit test somewhere using it to test parsing.

# Affiliation

This library is a private project and not affiliated with OpenRouter, Inc.
