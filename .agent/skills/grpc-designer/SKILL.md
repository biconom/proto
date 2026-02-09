---
name: grpc-designer
description: Use this skill when designing or implementing gRPC/Protobuf API services. It ensures high-quality API contracts following strict style guides, including nested IDs, lists, wrapped enums, and project configuration (build.rs, Cargo.toml).
---

# gRPC Designer

This skill guides the creation of high-quality Protobuf API contracts for gRPC services. It is designed to be reusable across projects.

## Core Workflow

1. **Understand Specification**: Read project requirements and specifications.
2. **Apply Style Guide**: Follow strict protobuf rules:
    - **Language**: Use rules defined in the project (default to Russian if not specified).
    - **Wrappers**: Always wrap enums and lists.
    - **IDs**: Use nested `Id` with `oneof` for entities.
3. **Structure Architecture**:
    - `[package]/types/`: Domain models.
    - `[package]/client/[domain]/`: Client-facing services.
    - `[package]/admin/[domain]/`: Admin-facing services.
4. **Project Configuration**:
    - Ensure `Cargo.toml` has necessary dependencies (`tonic`, `prost`, `tonic-build`).
    - Configure `build.rs` to compile all proto files using `tonic-build`.

## Quality Assurance
- **Linting**: Ideally use `buf lint` or `protolint` to enforce style (if available in environment).
- **Breaking Changes**: Check for breaking changes against main branch using `buf breaking`.
- **Formatting**: Keep `.proto` files consistently formatted.

## Essential Patterns

### Domain Model (Type)
Every new entity MUST have:
- `message Id` with `oneof identifier`.
- `message List` with `next_page_token`.
- `google.protobuf.Timestamp created_at/updated_at`.
- Wrapped enums in a sub-message (e.g., `message Status { enum Id { ... } }`).

### Clean Requests
- **Empty Messages**: Avoid creating custom empty messages (e.g., `message MyRequest {}`). Always use `google.protobuf.Empty` instead.

### Field Management (Versioning)
- **Never delete fields**: Change the field name to `deprecated_field_name` and add `[deprecated = true]`.
- **Never reuse field numbers**: Once a field number is used, it is burned forever.

### Idempotency
- Mutating RPCs (Create/Update/Delete) SHOULD support idempotency.
- Add `string idempotency_key = N;` to requests where duplicate execution is harmful (e.g., payments).

### Rust Configuration

#### Cargo.toml
Dependencies required for gRPC/Protobuf:
```toml
[dependencies]
prost = "0.14"
tonic = "0.14"
prost-types = "0.14"

[build-dependencies]
tonic-build = "0.14"
```

#### build.rs
Standard build script structure:
```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true) // or false depending on needs
        .compile(
            &[
                "proto/types/common.proto",
                "proto/service/myservice.proto",
                // Add all proto files here
            ],
            &["proto"], // Root include directory
        )?;
    Ok(())
}
```

## Connected Skills
- **Spec Architect**: MUST be consulted first to understand the "big picture" and domain logic. Use its output (`specs/domain/...`) as the blueprint for `.proto` files.
- **Brainstorming**: MUST use before finalizing any complex model. Ask for a "Brainstorming Session" to explore trade-offs (e.g., "Should we use `oneof` or separate messages here?").
- **Thinking Partner**: Use for deep architectural questions. It will ask clarification questions instead of rushing to code.

## References
- [Style Guide](references/style-guide.md) - Strict formatting rules.
- [Design Patterns](references/patterns.md) - Reusable code patterns.
