# ILTRIC — PHASE 09 PROMPT: AI SUBSYSTEM (SYSTEM SERVICE, NOT A CHATBOT)

**Prerequisite:** GATE 08 passed.
**Deliverable:** `iltric.ai` as a permissioned OS service with a capability/model router, local inference, and speech — on-device by default.

---

## OBJECTIVE
AI as a system layer that respects OS permissions and never silently exfiltrates user data.

---

## TASKS

1. **Runtime** — `llama.cpp` as the local inference backend (GGUF, widest hardware coverage, simplest deploy). `Ollama` may wrap it for UX. `vLLM` is server-scale and out of scope. `ONNX Runtime` / `OpenVINO` only as accelerator execution providers.

2. **Model policy** — `/ai/models/`
   - default quantization **Q4_K_M** (~75% size cut vs FP16, minimal quality loss; clearly better than legacy Q4_0)
   - default assistant model class: **1B–3B** (3B Q4_K_M ≈ 2–3 GB resident)
   - 7–8B (≈ 4.3–5 GB weights + KV cache) only as an opt-in "large local model"
   - models are content-addressed, signed, and verity-checked like any other system artifact

3. **Capability / model router** — `/ai/router/`
   Route on: model size vs free RAM/VRAM, latency budget, energy/battery state, **privacy class of the request**, network availability, user setting.
   ```
   simple/deterministic task     -> CPU
   parallel / large batch        -> GPU
   privacy-sensitive             -> local only, never cloud
   too large for local           -> cloud ONLY with explicit consent scope
   ```
   The router logs every decision with the reason. The user can see it.

4. **NPU — honest scoping** — `/ai/npu/`
   - **Intel NPU** (`intel_vpu`, kernel 6.6+) via **OpenVINO** = the only broadly usable path today; ship it opt-in.
   - **AMD XDNA** (`amdxdna`, `CONFIG_DRM_ACCEL_AMDXDNA`) — driver upstream, runtime stack immature; mark **experimental**, no user-facing claim.
   - OpenVINO is validated on Intel hardware only — do not advertise it elsewhere.
   - Rule: `iltric.ai` must fall back silently to CPU/GPU if the NPU path fails. No user-visible breakage.

5. **Permission model** — AI gets **no ambient access**.
   - `ai.local` and `ai.cloud` are separate capabilities.
   - Reading user files for context requires the same `fs.*` capability an app would need, granted per scope, time-boxed.
   - **No silent screen capture. Ever.** (Learn from the Recall backlash: on-by-default screen capture drew heavy criticism and had to become opt-in, encrypted, and identity-gated.)
   - Design target: on-device-first with attested/consented cloud, in the spirit of Private Cloud Compute — state clearly what ILTRIC does and does not implement.

6. **Speech** — `/ai/speech/`
   - STT: `whisper.cpp` (quantized, CPU-capable)
   - TTS: `piper`
   - wake word: `openWakeWord`, **off by default**, with a hardware-independent visible indicator whenever the mic is live

7. **Audit** — `/ai/audit/`
   Local, user-readable log: what was asked, which model, which device, what data was in scope, whether it left the machine. Retention configurable, default 30 days, one-click purge.

8. **Resource governance** — `ai.slice` with `MemoryMax=` and `CPUWeight=` below the shell. Model unloads after an idle timeout so idle RAM stays within budget.

---

## EXIT GATE — GATE 09
- [ ] **Idle RAM with AI service loaded but model unloaded: total system still ≤ 600 MB.** Model residency is on-demand only.
- [ ] 3B Q4_K_M: **TTFT < 500 ms** and **> 15 tok/s** on the reference machine. Record hardware + numbers.
- [ ] Router decision log shows correct routing for 5 scripted cases (small/large/private/offline/battery-low).
- [ ] Cloud call **cannot** happen without `ai.cloud` — verified by attempting it with the capability revoked, with a packet capture showing zero egress.
- [ ] Mic indicator is on whenever audio capture is active, including wake word. Verified.
- [ ] NPU path either works on Intel test hardware or is cleanly disabled with a logged reason — no silent failure.
- [ ] Privacy audit: a scripted session produces a complete, accurate audit log.

## FAILURE MODES & ROLLBACK
- OOM during inference → router must pre-check free memory and refuse gracefully, never let oomd kill the shell.
- Model file corrupt → verity/signature check refuses to load; fall back to the previous model version.
- NPU driver regression after kernel update → automatic fallback to CPU/GPU + a single non-nagging notification.

## NEXT STEP
`11-PHASE-10-APPS.md`
