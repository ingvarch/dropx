# Rust Best Practices for DropX Project

## Critical Rules

- Never use emoji in code, comments, or documentation
- **Follow DRY (Don't Repeat Yourself) principle strictly:**
  - Extract repeated code into functions or constants
  - Replace magic numbers with named constants immediately
  - Replace magic strings with enums or constants
  - Avoid duplicating logic across multiple places
  - If you see repeated values or patterns, refactor them right away
- **Follow YAGNI (You Aren't Gonna Need It) principle:**
  - Don't add functionality until it's actually needed
  - Don't over-engineer solutions for hypothetical future requirements
  - Implement only what's required by current specifications
  - Refactor when new requirements emerge, not before
- Use meaningful variable and function names that reflect their purpose
- **ALWAYS check worker-rs version and documentation before writing Cloudflare Workers code:**
  1. First, check Cargo.toml to see current worker version in the project
  2. Search for latest worker-rs version on crates.io or GitHub
  3. Read official documentation at https://docs.rs/worker/latest/worker/ for the version being used
  4. Verify API signatures, module paths, and patterns in the docs before implementing
  5. Never assume API based on older versions - always verify current implementation

## Rust-Specific Best Practices

### Error Handling

- Use `Result<T, E>` for all operations that can fail
- Prefer `?` operator over explicit error handling when possible
- Create custom error types using `thiserror` or enum-based errors
- Never use `unwrap()` or `expect()` in production code paths
- Use `anyhow` for application-level errors, `thiserror` for library errors

### Memory Safety and Ownership

- Prefer borrowing (`&T`) over ownership transfer when possible
- Use `Cow<str>` for strings that might be borrowed or owned
- Avoid unnecessary clones; use references instead
- Use `Arc<T>` for shared ownership across threads
- Use `Rc<T>` for single-threaded reference counting

### Type System

- Leverage strong typing; avoid `String` for everything
- Use newtypes for domain-specific types (e.g., `FileId(String)`)
- Implement `From`/`Into` traits for type conversions
- Use `#[derive]` for common traits when possible
- Mark types as `#[non_exhaustive]` if they might grow

### Async Programming

- Use `async`/`await` consistently
- Prefer `tokio` or native async runtime features
- Don't block async executors with synchronous code
- Use `spawn` for concurrent tasks
- Handle cancellation and timeouts properly

### Code Organization

- One module per file in complex projects
- Use `mod.rs` to expose module API
- Keep functions small and focused (max 50-100 lines)
- Separate pure logic from I/O operations
- Use builder pattern for complex constructors

### Performance

- Profile before optimizing
- Use `&str` over `String` when ownership not needed
- Prefer iterators over collecting to Vec unnecessarily
- Use `SmallVec` for small fixed-size collections
- Enable LTO and optimize for size in release builds

### Testing

- Write unit tests for all business logic
- Use `#[cfg(test)]` modules
- Mock external dependencies
- Test error cases, not just happy paths
- Use property-based testing for complex logic

### Documentation

- Document all public APIs with `///` doc comments
- Include examples in doc comments
- Explain "why" not just "what"
- Use `#[doc(hidden)]` for internal implementation details

### Dependencies

- Minimize dependencies; evaluate before adding
- Pin major versions in Cargo.toml
- Regularly update dependencies for security
- Use workspace for multi-crate projects

### Cloudflare Workers Specific

- Keep cold start time low (optimize binary size)
- Always verify current worker-rs version and API from official documentation
- Handle bindings (R2, D1, Durable Objects) gracefully
- Implement proper CORS handling
- Use structured logging with context

### Security

- Validate all user inputs
- Hash sensitive data (IPs, passwords)
- Use constant-time comparisons for secrets
- Implement rate limiting at multiple layers
- Never log sensitive information

### Patterns to Follow

1. **Result propagation**: Use `?` operator chains
2. **Builder pattern**: For complex struct initialization
3. **Type state pattern**: Encode state in type system
4. **Newtype pattern**: Wrap primitives for type safety
5. **Strategy pattern**: Use traits for polymorphism

### Patterns to Avoid

1. **String typing**: Don't use `String` for everything
2. **Panic in libraries**: Return `Result` instead
3. **Unnecessary allocations**: Profile and optimize
4. **Deep nesting**: Extract functions to flatten
5. **Magic numbers**: Use named constants

## Project-Specific Guidelines

### File Structure

```
src/
├── lib.rs              # Main router and entry point
├── handlers/           # HTTP request handlers
│   ├── mod.rs
│   ├── upload.rs
│   ├── download.rs
│   ├── info.rs
│   └── admin.rs
├── services/           # Business logic layer
│   ├── mod.rs
│   ├── storage.rs      # R2 operations
│   ├── database.rs     # D1 operations
│   └── rate_limiter.rs # Durable Objects client
├── models/             # Data structures
│   ├── mod.rs
│   ├── file.rs
│   └── api_key.rs
├── utils/              # Helper functions
│   ├── mod.rs
│   ├── id_generator.rs
│   ├── mime.rs
│   └── auth.rs
├── durable_objects/    # Durable Object implementations
│   ├── mod.rs
│   └── rate_limiter.rs
└── cleanup.rs          # Cron job handlers
```

### Naming Conventions

- Use `snake_case` for functions, variables, modules
- Use `PascalCase` for types, structs, enums, traits
- Use `SCREAMING_SNAKE_CASE` for constants
- Prefix private items with underscore if intentionally unused
- Use descriptive names: `file_id` not `fid`, `expires_at` not `exp`

### Error Messages

- Be specific and actionable
- Include context about what failed
- Suggest solutions when possible
- Don't expose internal implementation details to users

### Git Commit Messages

- Follow conventional commits format
- Use imperative mood: "add feature" not "added feature"
- Scope examples: `api`, `storage`, `rate-limit`, `auth`, `admin`
- Types: `feat`, `fix`, `chore`, `refactor`, `docs`, `test`, `perf`

Example: `feat(storage): implement R2 upload with retry logic`

## Code Review Checklist

- [ ] No `unwrap()` or `expect()` in production paths
- [ ] All public APIs documented
- [ ] Error types are descriptive
- [ ] No unnecessary allocations or clones
- [ ] Tests cover edge cases
- [ ] No hardcoded secrets or magic numbers
- [ ] Proper error propagation with `?`
- [ ] Input validation implemented
- [ ] Logging includes context
- [ ] Binary size optimized for cold starts

## Performance Targets

- Cold start: < 50ms
- Upload response: < 200ms (excluding transfer time)
- Download response: < 100ms (excluding transfer time)
- Binary size: < 1MB (optimized)
- Memory usage: < 128MB per request

## References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Effective Rust](https://www.lurklurk.org/effective-rust/)
- [Cloudflare Workers Rust](https://developers.cloudflare.com/workers/languages/rust/)
- [worker-rs Documentation](https://github.com/cloudflare/workers-rs)
