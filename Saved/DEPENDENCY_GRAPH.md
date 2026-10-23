# Workspace Dependency Graph

## Source of truth

Derived from `src/Cargo.toml` and each member crate's `Cargo.toml` (19 workspace members). Arrows point from a crate to its direct dependency. `→` edges shown are workspace path dependencies, except where noted.

```text
litric-caps ───────────────→ litric-core
litric-env ─────────────────→ litric-core
litric-storage ─────────────→ litric-core, litric-caps
litric-trash ────────────────→ litric-storage
litric-files ────────────────→ litric-core, litric-storage, litric-trash, litric-locale
litric-manifest ─────────────→ litric-core, litric-caps
litric-winbridge ────────────→ litric-core
litric-sysd ─────────────────→ litric-core, litric-caps, litric-env,
                               litric-storage, litric-trash
litric-envd ─────────────────→ litric-core, litric-env
litric-browser ──────────────→ litric-caps, litric-core
litric-shell ────────────────→ litric-design, litric-sysd, litric-input,
                               litric-wm, litric-locale, litric-files,
                               litric-defender, litric-core, litric-text
litric-bench ────────────────→ litric-defender

No direct workspace dependencies:
  litric-core, litric-design, litric-input, litric-wm, litric-locale,
  litric-defender, litric-text
```

### External dependencies

- `litric-text` directly declares `fontdue = 0.9.4` and `harfrust = 0.14.0`.
- Other member manifests currently declare only in-workspace path dependencies or no dependencies.
- The crate source documents in-tree Rust workspace / supply-chain policies. Do not overstate this as “the entire Rust workspace has zero third-party crates”; text shaping is an explicit exception and the root lockfile resolves transitive packages.

## Architectural interpretation

- `litric-core` is a bottom-level shared contract. `litric-caps` and `litric-env` depend on it; `litric-storage` composes identity and capability vocabulary; `litric-trash` builds on storage URI semantics.
- `litric-files` is an I/O runtime depending on storage/trash policy plus locale; `litric-sysd` composes service decisions and state; `litric-envd` is a supervisor process using the environment model.
- `litric-shell` is the composition root in Cargo: it links UI tokens, daemon/model crates, input, WM, locale, files, defender, core identity and text. It should remain an edge/consumer, not a dependency of lower policy crates.
- `litric-winbridge` is deliberately subordinate, depending only on `litric-core`; removing Windows compatibility should not invalidate the core OS.
- `litric-bench` depends on Defender to exercise its real policy engine, rather than duplicating scan behavior.
- `litric-design`, `litric-input`, `litric-wm`, `litric-locale`, and `litric-defender` are independently testable at crate boundary due to no direct workspace dependencies (locale does use std filesystem for zone/config reads; “dependency-free” means no crate dependency, not no I/O).

## Cargo targets and feature gates

| Package | Target note |
|---|---|
| `litric-sysd` | library `litric_sysd` plus binary `litric-sysd` |
| `litric-envd` | binary `litric-envd` plus library crate |
| `litric-browser` | library plus `litric-browserd` binary requiring feature `broker`; broker feature is declared empty and gates the target |
| `litric-shell` | binary `litric-shell` plus library |
| `litric-bench` | library plus `litricos-test` CLI |

## Dependency governance and review checks

- `src/Cargo.lock` is the resolved graph and must be reviewed with manifest edits.
- `ci/lint-deps.py`, `ci/dep-policy-allow.txt`, `ci/verify.sh`, and the CI/test documentation govern policy checks. Confirm the precise allowlist and negative-control behavior before changing dependency policy.
- New edges should be checked against layer direction and runtime responsibilities. In particular, keep untrusted document/archive decoding out of `litric-shell`; keep `litric-winbridge` out of core; and avoid a reverse edge from model/policy crates to daemon/UI crates.
- Feature-gated targets can differ from the default cross-platform build. A library compile does not establish that the broker binary is built or runnable on the target image.