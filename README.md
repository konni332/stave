# stave

stave is a Rust library that utilizes compile-time typestate validation
to generate type-safe builder code for structs. By leveraging the
Rust type system, stave ensures that specific builder methods cannot
be invoked until all requisite fields are explicitly initialized.
This architecture eliminates a common class of runtime panics
resulting from uninitialized configuration fields, moving validation
errors from runtime to compile time.

The crate provides two primary attribute macros: `#[builder]`
and `#[methods]`. Annotating a struct generates the underlying
typestate machinery, allowing developers to define custom setters
and accessors while delegating the boilerplate generation to stave.

## Quick Start

```rust
#[builder]
struct Server {
    #[stave(required)]
    host: String,
    #[stave(required)]
    port: u16,
    timeout: Duration, // Unannotated fields default to optional
    #[stave(optional)] // Explicit annotation is supported for clarity
    note: String,
}

#[methods]
impl Server {
    #[sets(host)]
    fn set_host(self, value: impl Into<String>) -> String {
        value.into()
    }

    #[requires(host, port)]
    fn finish(self) -> Config {
        Config {
            host: self.host().clone(),
            port: *self.port(),
            timeout: self.timeout().clone(),
        }
    }
}
```

```rust
let config = Server::new()
    .set_host("localhost")
    .set_port(8080)
    .finish();
```

In the execution chain above, `set_port` was generated
automatically by stave. Omitting `.set_port(8080)` results in
a compilation failure rather than a runtime error:

```
error[E0599]: no method named `finish` found for struct `Server<__HostSet, __PortUnset>`
```

This guarantees structural and configuration integrity prior
to executing arbitrary, user-defined state dependent methods.

---

## Attributes Reference

### `#[builder]`

This attribute is applied directly to a struct definition.
Fields are evaluated based on their attributes:

- `#[stave(required)]`: Enforces initialization before allowing access
  to dependent methods.
- `#[stave(optional)]`: Wraps the inner type in an `Option<T>`.
  Unannotated fields default to this state.

For each required field, stave generates internal marker types
(e.g., `__HostUnset` and `__HostSet`) and appends a generic
parameter to the struct to track initialization state at compile time.

The macro generates a `new()` constructor where all required fields
are bound to their unset marker types and optional fields are
initialized to `None`. User-defined generic parameters, lifetimes,
and where-clauses are fully preserved; stave appends its internal
tracking parameters to the existing generic signature.

### `#[methods]`

This attribute is applied to the `impl` block of a struct previously
annotated with `#[builder]`. Within this block, developers can
implement arbitrary logic alongside two specialized attributes:

#### `#[sets(field)]`

Designates a method as the setter for a specified field.
The method body computes the underlying value and must return
the inner field type rather than the parent struct. stave rewrites
the method signature and return type to execute the typestate transition,
rebuilding `Self` with the updated state marker.

```rust
#[sets(host)]
fn set_host(self, value: impl Into<String>) -> String {
    value.into()
}
```

Setters for required fields consume `self` by value (accepting
`self` or `mut self`, but prohibiting `&self` or `&mut self`)
because the underlying type representation changes upon
field initialization.

If a field lacks an explicit `#[sets(...)]` method within the
`impl` block, stave automatically synthesizes a default
public setter: `pub fn set_foo(self, value: FooTy) -> ...`.
Custom setters are therefore only necessary when input
transformations are required.

#### `#[requires(a, b, ...)]`

Restricts method availability until the specified required fields
are initialized. Within the scope of this method, stave guarantees
the fields are populated, rendering data access safe. This is not
limited to finalization steps; any arbitrary user-defined function
can enforce these checks.

```rust
#[requires(host, port)]
fn finish(self) -> Config {
    // ...
}
```

Attempting to invoke a `#[requires(...)]` method on an incomplete
builder configuration triggers a compile-time error.

Developers can combine `#[sets(...)]` and `#[requires(...)]` on
a single method to express complex state dependencies, such as
requiring field `host` to be initialized before allowing a method
to execute and set field `note`.

### Getters

stave automatically generates a getter method named after each field.

- For a required field `host: String`, a `pub fn host(&self) -> &String`
  method is exposed, but it is only accessible once the field
  transitions to a set state.
- For an optional field `timeout: Duration`, a
  `pub fn timeout(&self) -> &Option<Duration>` method is exposed
  and remains accessible throughout the entire lifecycle
  of the builder.

---

## Generics Support

The `#[builder]` macro natively resolves structs containing lifetimes,
type parameters, and const generics:

```rust
#[builder]
struct Cache<'a, T: Clone, const N: usize> {
    #[stave(required)]
    items: [T; N],
    #[stave(required)]
    name: &'a str,
    note: String,
}
```

Each generated marker type is parameterized exclusively over the
subset of generics required by that specific field. For example,
`__ItemsSet` retains parameters for `T` and `N` because it stores
`[T; N]`, whereas `__NameSet` is parameterized strictly over
the lifetime `'a`. This model ensures composition across
`#[methods]` blocks, enabling user-defined methods to introduce
independent generic constraints.

---

## Internal Architecture and State Management

This section details the underlying mechanisms of the macro
synchronization for contributors and maintainers.

Because `#[builder]` and `#[methods]` process code as distinct
procedural macro invocations, they cannot directly share state
via token streams. To facilitate communication, `#[builder]`
evaluates the struct schema and writes the metadata (field
statuses, types, and generic mappings) to an in-process global
registry implemented as a `static` `HashMap` protected by a `Mutex`.

The `#[methods]` macro subsequently queries this registry by struct
identifier. This design introduces a structural requirement:
**the `#[builder]` macro must be evaluated prior to the corresponding
`#[methods]` block in the compilation stream**. Standard source file
layouts, where an `impl` block follows the struct definition, satisfy
this requirement naturally.

If `#[methods]` fails to locate the corresponding registry metadata
due to missing declarations or improper ordering, it generates an
explicit compile-time error.

### State Representation via Marker Types

For a required field `host: String`, the `#[builder]` macro
generates the following internal structures:

```rust
struct __HostUnset;
struct __HostSet(String);
```

stave introduces the generic parameter `__HostState` to the
struct signature, which alternates between `__HostUnset` and
`__HostSet`. The `#[methods]` macro then generates specific
`impl` block permutations corresponding to valid state
combinations. Setter methods consume the unset variant and
return a new instance of the struct containing the set variant
wrapper. These marker types are declared with `pub(crate)`
visibility to keep the public API clean.

---

## Current Limitations

- **Single Implementation Block Restriction**: The builder logic must
  reside within a single `#[methods]` block per struct. Splitting
  setters across multiple blocks prevents the macro from verifying
  existing setters, which can cause duplicate definition errors.
- **Aliasing (Virtual Flags)**: The `as` aliasing feature, which
  permits setting a field typestate while satisfying a named virtual
  flag requirement, is not currently supported.
- **Module Context**: The system operates under the assumption that
  `#[builder]` and `#[methods]` are invoked within the same module
  scope to ensure registry resolution.

## Testing Suite

The project includes an integration testing framework powered by
`trybuild` located in the `tests/` directory. The suite contains
positive test cases (`tests/pass/`) validating correct structural
behavior, and negative test cases (`tests/fail/`) verifying that
typestate violations properly trigger compile-time diagnostic errors.

## License

This project is licensed under the MIT License. See the
[LICENSE](LICENSE) file in the root directory of this repository
for complete details.
