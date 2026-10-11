# ADR-0006: Audio Stack
**Status:** Accepted | **Date:** 2026-08-26

## Context
Litric needs low-latency audio with per-app volume and spatial audio.

## Decision
Use PipeWire + WirePlumber.

## Consequences
+ Low-latency capable (quantum 64-128 at 48 kHz)
+ Per-app volume built-in
+ Modern replacement for PulseAudio + JACK
+ Spatial audio support
+ Active development
- PipeWire is C (not memory-safe)
- Higher memory than minimal PulseAudio (~30-50 MB)

## Revisit-if
- PipeWire latency does not meet pro audio requirements
- PipeWire memory overhead exceeds budget
