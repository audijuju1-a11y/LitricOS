# DeepSeek task prompts — how these are used in this project

## What is true about invocation

These files are **prompts for a human to carry to DeepSeek**. Nothing in this
repository, and nothing in the agent environment this pass ran in, can invoke a
model directly: there is no DeepSeek API key configured here, no model-call tool
available, and no route from the agent to such a service. An answer that arrived
from DeepSeek was carried back by hand, and that is the only claim made about
provenance.

Two consequences, both deliberate:

1. **A returned patch is a proposal, not evidence.** Every symbol it names is
   checked against the actual tree, the diff is inspected for scope creep, the
   tests are run here, and the evidence ledger is updated by us. None of that is
   waived because a model wrote the code.
2. **The prompt must contain everything needed to be correct.** DeepSeek cannot
   see this checkout. A ticket that says "the settings registry" without quoting
   the real file is a ticket that produces invented APIs.

## Rules for every ticket in this directory

- One bounded objective. No ticket asks for a subsystem, and no ticket starts
  before its prerequisites exist in the tree.
- Every claim in "Verified repository facts" was produced by a command in this
  checkout and can be reproduced by the reader.
- Files allowed to change are listed, and files forbidden are listed.
- Every ticket names a **negative test** — a check that must fail when the
  behaviour is deliberately broken, so that a green result means something.
- Workspace constraints are repeated in every ticket, because a patch that
  violates them is rejected regardless of what it achieves: `unsafe_code =
  "forbid"`, `clippy::unwrap_used`/`expect_used`/`panic`/`indexing_slicing` denied,
  zero third-party dependencies in the core crates, `rust-version = 1.85`,
  edition 2021.
- The "Output required" section is not optional: diagnosis, minimal design,
  patch, tests, **exact commands actually executed**, risks, and integration
  notes. A ticket answer that claims a test ran without a command and a result is
  returned, not merged.

## Tickets

| Ticket | Objective | Prerequisites |
|---|---|---|
| [TASK-0001](TASK-0001-dead-systemd-units.md) | Delete the systemd units whose binaries do not exist; make the gate fail if one reappears | none |
| [TASK-0002](TASK-0002-doc-contradictions.md) | Correct the three documentation contradictions found in the Stage A audit | none |
| [TASK-0003](TASK-0003-dependency-policy-check.md) | Add a dependency-policy check with a negative control | TASK-0001 (it touches the same gate script) |

No further ticket is written ahead of need. The next real objectives — a settings
store with persistence, a network provider contract, a PAM-backed authentication
helper — do not exist in the tree yet; a prompt for them today would be full of
invented names, which is the failure mode these rules exist to prevent. Each
becomes a ticket when its design note lands.
