# Unit Tests

Unit tests are embedded in the Rust source code using `#[cfg(test)]` modules,
which is the standard Rust convention. They are compiled and run via `cargo test`.

Test modules are located in:
- `src/auth/password.rs` — Password validation & hashing
- `src/crypto/masking.rs` — Sensitive field masking
- `src/crypto/aes.rs` — AES-256-GCM field encryption
- `src/pos/state_machine.rs` — Order state machine transitions
- `src/pos/idempotency.rs` — Idempotency key helpers (pure logic)
- `src/storage/mod.rs` — File storage, content type, SHA-256
- `src/audit/service.rs` — Audit hash computation
- `src/models/order.rs` — Order status serde roundtrip
- `src/models/dataset.rs` — Dataset type serde roundtrip
- `src/observability/metrics.rs` — Metrics counter logic

Run with: `cargo test` (inside the Docker builder) or via `run_tests.sh`
