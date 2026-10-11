# ADR-0005: Compositor
**Status:** Accepted | **Date:** 2026-08-26

## Context
Iltric needs a compositor that is memory-safe, modern, and supports DRM syncobj, dmabuf, and HDR.

## Decision
Use Smithay (Rust) as the compositor foundation.

## Consequences
+ Memory-safe (Rust)
+ Supports DRM syncobj, dmabuf, DRM leasing
+ Proven at desktop scale (COSMIC)
+ Active development, modern protocols
- Rust learning curve
- Less mature than wlroots (C) in some areas
- Smithay's API evolves rapidly

## Alternatives considered
- wlroots: most mature but C (unsafe language), violates Iltric's memory safety goals
- Fully custom: months of effort for basics, unjustified at this stage

## Revisit-if
- Smithay cannot implement required protocols
- Smithay's memory overhead exceeds budget
- A Rust compositor framework surpasses Smithay
