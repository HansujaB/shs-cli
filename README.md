# shs-cli

A command-line tool for **Shamir's Secret Sharing** — split any secret into `n` shares such that any `t` of them can reconstruct the original, but `t-1` reveal absolutely nothing.

Built from scratch in Rust with zero unsafe code. No external crypto dependencies — just raw finite field arithmetic over **GF(257)**.

## Why This Exists

Shamir's Secret Sharing (SSS) is the mathematical foundation behind threshold cryptography. This project implements the core primitive that underpins:

- **Key share repair & refresh** in production TSS systems (DKLs23, CGGMP24)
- **Multi-party custody** solutions for digital assets
- **Distributed key management** where no single party holds the full secret

Building it from scratch demonstrates understanding of the polynomial interpolation that makes `t-of-n` threshold schemes work.

## Quick Start

```bash
# Build
cargo build --release

# Split a secret into 5 shares, requiring any 3 to reconstruct
cargo run -- split --secret "my secret password" --threshold 3 --shares 5

# Reconstruct from any 3 shares
cargo run -- reconstruct --threshold 3 \
  --shares "1-00ae0052..." "3-00c70031..." "5-009a0067..."
```

## Usage

### `split` — Break a secret into shares

```
sss-cli split --secret <SECRET> --threshold <T> --shares <N>
```

| Flag | Description |
|------|-------------|
| `-s, --secret` | The secret string to split |
| `-t, --threshold` | Minimum shares needed to reconstruct (1 ≤ t ≤ n) |
| `-n, --shares` | Total number of shares to generate (max 255) |

**Output:** Each share is printed as `{index}-{hex}`, e.g. `1-00480065006c006c006f`.

### `reconstruct` — Recover the secret

```
sss-cli reconstruct --threshold <T> --shares <SHARE1> <SHARE2> ...
```

| Flag | Description |
|------|-------------|
| `-t, --threshold` | The threshold used during splitting |
| `-s, --shares` | One or more share strings in `index-hex` format |

## How It Works

### The Math

Given a secret byte `s`, we construct a random polynomial of degree `t-1`:

```
f(x) = s + a₁x + a₂x² + ... + aₜ₋₁xᵗ⁻¹   (mod 257)
```

where `a₁, ..., aₜ₋₁` are random coefficients. Each share `i` gets the evaluation `f(i)`.

To reconstruct, we use **Lagrange interpolation** to recover `f(0) = s` from any `t` points:

```
f(0) = Σᵢ yᵢ · ∏_{j≠i} (0 - xⱼ) / (xᵢ - xⱼ)   (mod 257)
```

The prime **257** was chosen because it's the smallest prime larger than 255, meaning every byte value fits cleanly into the field.

### Architecture

```
src/
├── main.rs       CLI entry point — clap-based argument parsing
├── shamir.rs     Core math — polynomial evaluation, Lagrange interpolation
├── encoding.rs   Share serialization — hex encoding with index embedding
├── errors.rs     Typed errors via thiserror
└── lib.rs        Module re-exports
```

**Design rule:** `shamir.rs` has zero CLI dependencies — it's pure math that could be extracted into a standalone library.

### Share Format

Shares are serialized as `{index}-{hex_data}`:

```
3-00480065006c006c006f
│ └─── each u16 y-value as 2 big-endian bytes, hex-encoded
└───── 1-based share index
```

## Security Properties

| Property | Guarantee |
|----------|-----------|
| **Information-theoretic security** | `t-1` shares reveal zero information about the secret |
| **Correctness** | Any `t` shares always reconstruct the original secret |
| **No secret persistence** | The raw secret is never stored or logged beyond the terminal |

> ⚠️ **Note:** This is an educational implementation. For production use, consider additional protections like share authentication (MACs), secure memory handling, and side-channel resistance.

## Testing

```bash
cargo test
```

The test suite covers:

- **Roundtrip correctness** — split → reconstruct returns the original
- **Any-t-of-n** — multiple combinations of `t` shares all work
- **Insufficient shares** — `t-1` shares correctly fail
- **Edge cases** — `t=1` (trivial), `t=n` (all required), single byte, 1KB+ secrets
- **Invalid inputs** — empty secrets, bad thresholds, malformed share strings
- **Math primitives** — modular inverse, polynomial evaluation

## Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing (derive API) |
| `rand` | Cryptographically secure random coefficients |
| `hex` | Share hex encoding/decoding |
| `thiserror` | Typed library errors |
| `anyhow` | Ergonomic CLI error handling |

## License

MIT
