# Rust Style Guide

This guide defines reusable conventions for organizing Rust applications and libraries. It emphasizes explicit construction, typed domain values, clear module boundaries, and predictable error handling. These are project conventions, not requirements of the Rust language.

## 1. Organize modules by responsibility

Group code by domain or feature. Keep a module's public entry points close to its implementation, and separate data definitions from orchestration when the distinction improves readability.

```text
src/
  feature/
    mod.rs          Public interface, state, initialization, trait implementations
    types.rs        Requests, responses, domain enums, conversions, validation
    operation/
      mod.rs        A substantial operation or workflow
      types.rs      Types owned by that operation
```

Create files only when they have a clear responsibility. A small feature does not need every layer shown above.

Keep feature-specific helpers within their owning feature. Move code into a shared module only when multiple features need the same behavior. Prefer descriptive shared modules such as `http`, `email`, or `time` over an unrelated collection of utilities.

## 2. Construct application types with `::new()`

Use constructors instead of struct literals at application call sites. Keep fields private by default, and place initialization rules in the type's implementation.

```rust
pub struct BatchRequest {
    item_ids: Vec<u64>,
}

impl BatchRequest {
    pub fn new(item_ids: Vec<u64>) -> Self {
        Self { item_ids }
    }
}

let request = BatchRequest::new(vec![1, 2]);
```

Apply the same rule to nested objects. Internal types may have private constructors. Struct literals remain appropriate inside constructors and other implementations that own the type's invariants.

When construction requires validation, return `Result<Self, Error>` rather than panicking. For constructors with many unrelated arguments, introduce a focused parameter type or builder rather than an increasingly long positional argument list.

Use purpose-specific entry points where appropriate:

- `new(...)` constructs an object from explicit inputs.
- `Default::default()` supplies a complete, meaningful default configuration.
- `from_env()` loads configuration from the environment and should return a result when loading can fail.
- `From` represents an infallible conversion; `TryFrom` represents a fallible conversion.
- Deserialization constructs external response data. Do not add unused constructors solely for symmetry.

If both a constructor and a conversion trait are provided, delegate to one implementation of the initialization rules. Prefer `Type::new(...)` when explicitly assembling a request at a call site.

Follow the construction APIs provided by third-party libraries. Enum variants with named fields are not subject to the struct-constructor rule.

## 3. Model fixed sets of values as enums in `types`

Represent statuses, modes, categories, and policies with enums rather than strings or unrelated boolean flags. Define an enum in the `types` module of the narrowest feature that owns it. Promote it to a shared domain module only when several features use the same concept.

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
}

pub struct JobQuery {
    status: JobStatus,
}

impl JobQuery {
    pub fn new(status: JobStatus) -> Self {
        Self { status }
    }
}

let query = JobQuery::new(JobStatus::Pending);
```

Accept the enum in function parameters as well as storing it in fields. A typed field offers little protection if callers must still supply arbitrary strings.

Use Serde's `rename` or `rename_all` attributes to preserve external spelling, case, and field names. Derive only the traits the type needs. Do not invent unsupported variants to make an enum appear complete.

Choose an explicit policy for unknown external values:

- Reject unknown values when accepting them would violate a contract or make processing unsafe.
- Use `#[serde(other)] Unknown` for a compatible tagged or unit-enum representation when unknown values may be ignored safely. This discards the original value.
- Use a representation such as `Other(String)` with suitable serialization and deserialization when callers need the original value. Adding a payload variant alone does not make it a Serde fallback.

Never map an unknown state to success or eligibility for a state-changing operation. Do not turn arbitrary text, URLs, or regular expressions into enums unless they actually represent a finite domain choice.

## 4. Separate orchestration from data handling

Use public methods and trait implementations as clear entry points. Extract substantial workflows into private functions or methods when that makes their steps easier to understand and test.

Keep parsing, serialization, domain conversion, and local invariant checks with their types. Keep network calls, sequencing, persistence, and coordination in operation modules.

Avoid wrappers that only add indirection. Do not force distinct authentication mechanisms or workflows into the same implementation merely to make their source code look alike.

## 5. Make ownership and visibility intentional

Prefer borrowed inputs such as `&str` and `&[T]` when a function only reads data. Accept owned values such as `String` and `Vec<T>` when an object stores or consumes them. Clone only when ownership or lifetime requirements justify it.

Keep fields private unless direct access is part of the intended public API. Use the narrowest useful visibility: private, `pub(super)`, `pub(crate)`, or `pub`. Expose accessors for meaningful operations or observations rather than automatically creating a getter and setter for every field.

Use `Arc` for genuinely shared ownership across threads, not as a substitute for deciding who owns data. Avoid adding synchronization when ordinary ownership is sufficient.

## 6. Use names that communicate purpose

Use `UpperCamelCase` for types and enum variants, `snake_case` for functions and fields, and `SCREAMING_SNAKE_CASE` for constants.

Choose role-specific names such as `CreateJobRequest`, `JobResponse`, `JobPage`, `JobStatus`, and `RetryPolicy`. Add a domain or provider prefix when it prevents ambiguity. Avoid redundant prefixes inside a well-defined module.

Follow familiar conversion naming where possible: `as_*` for borrowed views, `to_*` for producing a converted value without consuming the source, and `into_*` for consuming conversions. Preserve established public names when renaming would cause unnecessary compatibility changes.

Private helpers should describe the operation they perform. An underscore prefix is not required for private visibility; use one only when an existing project's conventions explicitly call for it.

## 7. Return errors and preserve their meaning

Return `Result<T, Error>` for recoverable failures. A domain-specific result alias is useful when it consistently represents the same error boundary. Use `?` to propagate errors and add context where it clarifies the failed operation.

Do not use `unwrap()` or `expect()` for environment variables, user input, external responses, or network operations. They can be reasonable in tests or for invariants whose validity is evident from the code.

Distinguish configuration errors, authentication failures, invalid data, business failures, and transport failures when callers need different recovery behavior. Do not replace every failure with a generic authentication or retry error. Preserve underlying causes where useful without exposing secrets in user-facing messages.

For state-changing requests, a timeout or lost response may mean the operation succeeded remotely. Represent an unknown outcome explicitly. Retry only when idempotency or reconciliation makes doing so safe. Validate batch contents before sending the first mutation, and do not count missing, duplicated, or unrecognized results as successful.

## 8. Keep external representations at boundaries

Use typed request and response models for stable external contracts. Keep wire-format details in serialization attributes and boundary conversions rather than scattering string keys throughout workflows.

Dynamic JSON is appropriate for genuinely variable data or a deliberately raw API. Convert it into typed data before relying on specific fields for business decisions.

Centralize repeated protocol behavior such as transport setup, authentication headers, pagination rules, or message parsing when it is actually shared. Keep domain-specific filters and policies with the feature that owns them.

Do not log passwords, access tokens, session cookies, one-time codes, or sensitive payloads. Logs should identify the operation and useful diagnostic context without copying private data.

## 9. Format consistently and document decisions

Use `rustfmt` with the project's Rust edition and formatter configuration. Follow existing import organization, and separate declarations and implementations with clear whitespace.

Place unit tests after the implementation they exercise. Use documentation comments for public contracts, failure conditions, side effects, and ownership expectations. Use ordinary comments to explain decisions or constraints that the code cannot express directly.

Keep examples aligned with the public API. When moving a public type, consider a re-export to preserve its import path. Document intentional breaking changes, including string parameters replaced by enums.

## 10. Verify behavior, contracts, and compatibility

Test meaningful behavior rather than reproducing implementation details. For boundary types, check exact serialized field names, enum values, numeric versus string representations, required fields, and unknown-value handling. For workflows, check failure paths, partial results, and mutation outcomes.

Use the checks appropriate to the project's feature model and environment. Common commands include:

```sh
cargo fmt --check
cargo check --all-targets
cargo test
cargo clippy --all-targets
```

Check additional supported feature combinations when relevant. Use `--all-features` only when the feature set supports enabling everything together. Keep tests that require live services, credentials, or real state changes separate from routine local tests, and state which checks were actually performed.
