# ADR-0003: Image Builder
**Status:** Accepted | **Date:** 2026-08-26

## Context
Litric needs reproducible UEFI images with ESP, dm-verity, and UKI in one pipeline.

## Decision
Use mkosi as the image build tool.

## Consequences
+ Reproducible builds (same inputs → same hash)
+ Built-in support for erofs, dm-verity, UKI
+ Integrates with systemd ecosystem
+ Supports GPT partition layout with ESP
- mkosi is systemd-ecosystem specific
- Less flexible than custom scripts for exotic layouts

## Revisit-if
- mkosi cannot produce the exact partition layout needed
- Reproducibility is broken by upstream changes
