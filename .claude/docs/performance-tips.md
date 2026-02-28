# Performance Tips

## Deferred Drop (Offload Cleanup to Another Thread)

When a function holds a large heap structure and the **response time matters more than
total memory usage**, move the value into a throwaway thread instead of dropping it inline.

```rust
fn process(big: BigThing) -> Output {
    let out = big.compute();
    std::thread::spawn(move || drop(big)); // cleanup happens off the hot path
    out
}
```

**When to use:**
- Interactive CLIs / real-time handlers where latency is visible to the user
- Functions whose computation finishes fast but whose cleanup dominates wall time

**Trade-offs:**
- Memory stays allocated until the cleanup thread runs — do not use when memory pressure is tight
- Cleanup timing becomes non-deterministic

**Reference**: https://abrams.cc/rust-dropping-things-in-another-thread
