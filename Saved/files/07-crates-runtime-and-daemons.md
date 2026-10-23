# Batch B07: Runtime, WM, Input, System Daemon & Bench Crates

This report indexes all 41 assigned inventory entries. Rust source summaries identify public API and implementation symbols; test fixtures and generated/reference artifacts are catalogued by provenance and purpose.

## `litric-bench` — Validation & Security Benchmark Harness — MASTER §26

Adversarial-but-nonweaponizable testing framework and `litricos-test` CLI: deterministic corpus, defender scanning, seeded chaos/stress schedules, safety gates, forensic JSONL SHA-256 chain, recovery/baseline comparison and report generation.

**Security/correctness review:** Safety: corpus uses benign synthetic bytes/EICAR public test string, no execution/network/encryption or persistent effects; higher risk levels require disposable-root and snapshot markers; VM rollback contract is reported rather than falsely simulated.

### `src/litric-bench/Cargo.toml`

- Inventory: `F0615`; configuration / cargo-manifest; 652 bytes; `TOML`.
- Size check: 16 lines; SHA-256 `863e50989fa224113dce2bbcc08344cba5fc472876458b4218d086fbb071407a`.
- Package configuration and dependency surface:
  - L1: `[package]`
  - L2: `name        = "litric-bench"`
  - L3: `description = "LitricOS maximum-depth adversarial security benchmark: destructive-equivalent corpus, deterministic chaos, forensic hash-chained event log (safety architecture per MASTER prompt §SAFETY)"`
  - L4: `version.workspace      = true`
  - L5: `edition.workspace      = true`
  - L6: `rust-version.workspace = true`
  - L7: `license.workspace      = true`
  - L9: `[lints]`
  - L10: `workspace = true`
  - L12: `[dependencies]`
  - L16: `litric-defender = { path = "../litric-defender" }`

### `src/litric-bench/src/bin/litricos-test.rs`

- Inventory: `F0616`; tests / rust-test; 24346 bytes; `Rust`.
- Size check: 716 lines; SHA-256 `c5e30678aa111edb05b2f8d7e321aad3e541ba7bea8d64e2f2c6f51242943b8c`.
- Source-level purpose: litricos-test — the adversarial benchmark CLI (MASTER §26). Every subcommand funnels through the safety gate before it touches anything: severity above the environment's clearance is refused with the exact unmet condition, and LEVEL ≥ 4 demands the snapshot markers only a disposable harness provides. The file-backed commands run anywhere; the VM-rolling commands (`snapshot`, full LEVEL 5) print the exact outer tooling contract
- Public API / key implementation declarations (1):
  - L36: `struct Opts {`
- Test/behavior coverage: 1 named functions in source, primarily module unit tests; representative names: `test_root`.

### `src/litric-bench/src/chaos.rs`

- Inventory: `F0617`; project source / rust-source; 8235 bytes; `Rust`.
- Size check: 243 lines; SHA-256 `2c77eb860b1c90f2b4a893d9933da7dad74e41ba5376008e231e022ed44687bf`.
- Source-level purpose: The chaos engine: seeded, reproducible event storms (MASTER §14). Chaos here is a *sequence* of synthetic events drawn from every class the OS must survive — file, process, thread, memory, I/O, network, security, telemetry, crash, resource. The sequence is a pure function of the seed: seed 928374 replays byte-identically on any machine, which is what turns a chaos run from an anecdote into a regression test. Nothing in this mo
- Public API / key implementation declarations (8):
  - L21: `pub enum EventClass {`
  - L47: `pub fn all() -> &'static [EventClass] {`
  - L64: `pub fn as_str(self) -> &'static str {`
  - L82: `pub struct ChaosStep {`
  - L95: `pub struct ChaosSchedule {`
  - L106: `struct Rng(u64);`
  - L124: `pub fn generate(seed: u64, count: usize, level: Level) -> ChaosSchedule {`
  - L179: `pub fn class_histogram(schedule: &ChaosSchedule) -> Vec<(EventClass, usize)> {`
- Test/behavior coverage: 0 named functions in source, primarily module unit tests; representative names: no inline `#[test]` functions detected.

### `src/litric-bench/src/corpus.rs`

- Inventory: `F0618`; project source / rust-source; 25290 bytes; `Rust`.
- Size check: 681 lines; SHA-256 `17adfa376925ae685d92a71b2b450cd06fafa9c80acb68e32e19349a3bce5e74`.
- Source-level purpose: The synthetic corpus generator: >=100,000 deterministic samples. Hostile samples here are: the public EICAR test string, high-entropy random bytes, bytes that *look* like structure (headers, section names, encoded blocks), or benign bytes with a threatening *shape*. Nothing generated parses as an executable for any real loader, nothing carries a real payload, and nothing here is a known malware sample — hostile truth comes fro
- Public API / key implementation declarations (12):
  - L28: `pub enum Category {`
  - L133: `pub fn all_categories() -> &'static [Category] {`
  - L191: `pub fn slug(self) -> &'static str {`
  - L249: `pub fn default_truth(self) -> Truth {`
  - L259: `pub fn level(self) -> Level {`
  - L317: `pub struct Sample {`
  - L335: `struct Rng(u64);`
  - L367: `pub const EICAR: &[u8] = b"X5O!P%@AP[4\\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*";`
  - L371: `pub const MARKERS: &[&[u8]] = &[`
  - L384: `const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789+/=";`
  - L564: `pub fn samples_for_category(seed: u64, cat: Category, count: usize) -> Vec<Sample> {`
  - L611: `pub fn generate(seed: u64, total: usize) -> Vec<Sample> {`
- Test/behavior coverage: 3 named functions in source, primarily module unit tests; representative names: `the_categories_are_exactly_fifty`, `hostile_samples_never_contain_real_executable_headers`, `every_sample_digest_matches_its_content`.

### `src/litric-bench/src/forensics.rs`

- Inventory: `F0619`; project source / rust-source; 15635 bytes; `Rust`.
- Size check: 445 lines; SHA-256 `5b6c723233c8a4db9af03e92fe03e78cb79b2b2958c66a00ee0c51095952ba56`.
- Source-level purpose: Event-by-event forensic logging with a tamper-evident hash chain. MASTER §15/§16/§17: every step of every test is an event; events chain (each carries the previous event's hash); the chain is verifiable end-to-end so a modified log is *detected*, not assumed away. The log is append-only by discipline: the writer never rewrites a line, and the verifier catches any line that does. The in-memory log is the source of truth for the
- Public API / key implementation declarations (17):
  - L23: `pub struct Event {`
  - L65: `pub fn canonical(&self) -> String {`
  - L93: `pub struct EventLog {`
  - L104: `pub fn new(run_id: &str) -> Self {`
  - L115: `pub fn run_id(&self) -> &str {`
  - L121: `pub fn push(&mut self, mut ev: Event) -> u64 {`
  - L138: `pub fn events(&self) -> &[Event] {`
  - L144: `pub fn len(&self) -> usize {`
  - L150: `pub fn is_empty(&self) -> bool {`
  - L155: `pub fn verify(&self) -> Result<(), String> {`
  - L176: `pub fn chain_broken(&self) -> bool {`
  - L183: `pub fn timeline_for(&self, sample_id: &str) -> Vec<&Event> {`
  - L194: `pub fn to_jsonl(&self) -> String {`
  - L224: `pub fn write_jsonl(&self, path: &std::path::Path) -> Result<u64, String> {`
  - L239: `pub fn verify_jsonl(text: &str) -> Result<usize, String> {`
  - L331: `pub fn monotonic_ns() -> u64 {`
  - L342: `pub fn now_ns() -> u64 {`
- Test/behavior coverage: 3 named functions in source, primarily module unit tests; representative names: `the_chain_verifies_when_untouched`, `a_modified_event_is_detected`, `jsonl_round_trips_the_essential_fields`.

### `src/litric-bench/src/lib.rs`

- Inventory: `F0620`; project source / rust-source; 2609 bytes; `Rust`.
- Size check: 67 lines; SHA-256 `04d4c1efae617dbe1ddd10ea64d0306b979158f20f3731dd64d9b2c20838a0a4`.
- Source-level purpose: litric-bench — LitricOS maximum-depth adversarial security benchmark. A destructive-equivalent test framework: it produces inputs and event sequences whose *impact profile* equals a catastrophic security event — parser pressure, memory and CPU exhaustion, event floods, tampering attempts, quarantine abuse, crash and recovery loops — without ever constructing a weaponizable artifact. Every "malicious" byte sequence in this crat
- Public API / key implementation declarations (10):
  - L31: `pub mod chaos;`
  - L32: `pub mod corpus;`
  - L33: `pub mod forensics;`
  - L34: `pub mod recovery;`
  - L35: `pub mod report;`
  - L36: `pub mod safety;`
  - L37: `pub mod scan;`
  - L38: `pub mod severity;`
  - L39: `pub mod stress;`
  - L42: `pub const VERSION: &str = env!("CARGO_PKG_VERSION");`
- Test/behavior coverage: 1 named functions in source, primarily module unit tests; representative names: `the_version_is_stamped_from_the_crate`.

### `src/litric-bench/src/recovery.rs`

- Inventory: `F0621`; project source / rust-source; 12329 bytes; `Rust`.
- Size check: 352 lines; SHA-256 `9b44cd149b4c00f1d1989b75ca86c5255188cd3cc14e7f1f4a33895a1862cc44`.
- Source-level purpose: Automatic recovery and baseline regression (MASTER §19, §20, §22). A catastrophic-equivalent test is not finished when the storm ends; it is finished when the environment is *proven* restored: markers verified, log chain intact, volume back to baseline, and the diff against the baseline recorded. A recovery that cannot prove itself is FAILED_RECOVERY — an incident, never a shrug. Recovery counters and depth caps saturate; oper
- Public API / key implementation declarations (9):
  - L21: `pub enum Step {`
  - L41: `pub fn as_str(self) -> &'static str {`
  - L56: `pub enum RecoveryVerdict {`
  - L71: `pub fn outcome(self) -> Outcome {`
  - L84: `pub struct EnvironmentCheck {`
  - L100: `pub fn check_environment(root: &Path) -> EnvironmentCheck {`
  - L145: `pub fn recover(`
  - L210: `pub struct Regression {`
  - L233: `pub fn classify(&self) -> Vec<&'static str> {`
- Test/behavior coverage: 2 named functions in source, primarily module unit tests; representative names: `clean_root_recovers`, `broken_sandbox_report_fails_first_step`.

### `src/litric-bench/src/report.rs`

- Inventory: `F0622`; project source / rust-source; 21702 bytes; `Rust`.
- Size check: 624 lines; SHA-256 `1ae3a2fb84013ac04f6c76cd0d8f1e71b6c1ec72d87cc306de0f5d38ef2cae6c`.
- Source-level purpose: The report layer: every artifact MASTER §25 demands, generated from the forensic event log alone (MASTER §28: if the log cannot answer it, the run did not record it). All CSVs are RFC-4180-escaped; all JSON is hand-built with the same `jstr` discipline as the forensic JSONL — this crate still has zero third-party dependencies. CSV/JSONL row emission and rate math on counters; saturating by design.
- Public API / key implementation declarations (14):
  - L38: `pub struct RunReport {`
  - L72: `pub fn summary_json(&self) -> String {`
  - L139: `pub fn summary_csv(&self) -> String {`
  - L197: `pub fn detections_csv(&self) -> String {`
  - L219: `pub fn misses_csv(&self) -> String {`
  - L243: `pub fn rule_gaps_csv(&self) -> String {`
  - L262: `pub fn resource_usage_csv(&self) -> String {`
  - L278: `pub fn regressions_csv(&self) -> String {`
  - L290: `pub fn crashes_csv(&self) -> String {`
  - L296: `pub fn recovery_csv(&self) -> String {`
  - L306: `pub fn report_md(&self) -> String {`
  - L414: `pub fn manifest_json(artifacts: &[(&str, u64)]) -> String {`
  - L463: `pub fn count_environment_limitations(results: &[ScanResult]) -> u64 {`
  - L472: `pub fn write_all(run: &RunReport, log: &EventLog, dir: &std::path::Path) -> Result<u64, String> {`
- Test/behavior coverage: 2 named functions in source, primarily module unit tests; representative names: `csv_escaping_handles_commas_quotes_newlines`, `write_all_produces_every_file`.

### `src/litric-bench/src/safety.rs`

- Inventory: `F0623`; project source / rust-source; 7138 bytes; `Rust`.
- Size check: 214 lines; SHA-256 `38bcc6e646def740ec96d8ebc4695f806cb96750bc00a7130f8985afee5b3c23`.
- Source-level purpose: The safety architecture: nothing destructive-equivalent runs ungated. MASTER §SAFETY: the framework must produce catastrophic *equivalents* without weaponizable artifacts, and must refuse to run outside a disposable environment. This module is the enforcement, not the prose: every entry point in the CLI funnels through [`SafetyGate::enter`], and the gate refuses when the target directory is not the dedicated test volume layout
- Public API / key implementation declarations (16):
  - L23: `pub const TEST_VOLUME_MARKER: &str = "LITRIC_BENCH_VOLUME";`
  - L25: `pub const SNAPSHOT_MARKER: &str = "LITRIC_BENCH_SNAPSHOT";`
  - L27: `pub const RECOVERY_MARKER: &str = "LITRIC_BENCH_RECOVERY_OK";`
  - L31: `pub enum Clearance {`
  - L43: `pub fn max_level(self) -> u8 {`
  - L54: `pub fn inspect(root: &Path) -> Clearance {`
  - L72: `pub fn provision(root: &Path) -> Result<(), String> {`
  - L86: `pub fn now_unix() -> u64 {`
  - L95: `pub struct SafetyGate {`
  - L107: `pub fn enter(root: &Path, level: u8) -> Result<Self, String> {`
  - L128: `pub fn root(&self) -> &Path {`
  - L134: `pub fn clearance(&self) -> Clearance {`
  - L140: `pub fn max_level(&self) -> u8 {`
  - L147: `pub fn ladder(&self, target: u8) -> Vec<u8> {`
  - L155: `pub fn critical_stop(&self, reason: &str) -> Incident {`
  - L166: `pub struct Incident {`
- Test/behavior coverage: 3 named functions in source, primarily module unit tests; representative names: `an_unmarked_directory_is_host_clearance`, `a_provisioned_volume_unlocks_levels`, `a_critical_stop_carries_what_a_forensic_needs`.

### `src/litric-bench/src/scan.rs`

- Inventory: `F0624`; project source / rust-source; 15927 bytes; `Rust`.
- Size check: 411 lines; SHA-256 `6e31e842e58703b6a34cab11c5533c564ea09525c759ef584d92460cde54153b`.
- Source-level purpose: The scan runner: the real Defender engine over the whole corpus. MASTER §14's contract lives here in its most testable form: the same engine binary that ships in the OS is driven over 100k+ deterministic samples, every scan is one forensic event, and the aggregate truth table (MASTER §2's outcome vocabulary) is a [`Scoreboard`], not a pass/fail. Corpus categories carry their own [`Truth`]: hostile categories are hostile becaus
- Public API / key implementation declarations (10):
  - L29: `pub struct ScanResult {`
  - L55: `pub fn outcome_of(decision: &Decision) -> (Outcome, String) {`
  - L78: `pub fn denylist_from_corpus(samples: &[Sample]) -> Denylist {`
  - L93: `pub struct LatencyProfile {`
  - L110: `pub fn percentile(&self, p: f64) -> u64 {`
  - L133: `pub fn mean(&self) -> u64 {`
  - L144: `pub fn samples_per_sec(&self) -> u64 {`
  - L154: `pub fn run_scan(`
  - L242: `pub struct CategoryGap {`
  - L255: `pub fn category_gaps(results: &[ScanResult]) -> Vec<CategoryGap> {`
- Test/behavior coverage: 5 named functions in source, primarily module unit tests; representative names: `benign_never_counts_as_detection_in_a_clean_run`, `every_sample_produces_exactly_one_forensic_event`, `percentile_is_not_clamped_to_the_u32_ceiling`, `an_out_of_range_percentile_reports_nothing`, `level_zero_benign_only_corpus_has_zero_tps`.

### `src/litric-bench/src/severity.rs`

- Inventory: `F0625`; project source / rust-source; 12192 bytes; `Rust`.
- Size check: 338 lines; SHA-256 `d46a6e5bfdcce7446f3d47ca258d7c009e75f771b95cdc486a1dbe1eb3a7bd52`.
- Source-level purpose: Severity levels and the outcome vocabulary every result reports. MASTER §2: six levels, ten outcomes. The outcomes are the honest part — a test that crashed the harness reports `CRASHED`, not `DETECTED`, and an environment that cannot run a class at all reports [`Outcome::EnvironmentLimitation`] rather than silently skipping. A benchmark that blurs these lies in its headline numbers. Scoreboard counters and rate math saturate 
- Public API / key implementation declarations (15):
  - L15: `pub enum Level {`
  - L33: `pub fn number(self) -> u8 {`
  - L46: `pub fn from_number(n: u8) -> Option<Self> {`
  - L61: `pub enum Outcome {`
  - L87: `pub fn as_str(self) -> &'static str {`
  - L104: `pub fn parse(s: &str) -> Option<Self> {`
  - L122: `pub fn is_detection(self) -> bool {`
  - L129: `pub enum Truth {`
  - L141: `pub fn as_str(self) -> &'static str {`
  - L151: `pub struct Scoreboard {`
  - L168: `pub fn record(&mut self, truth: Truth, outcome: Outcome) {`
  - L188: `pub fn detection_rate(&self) -> f64 {`
  - L199: `pub fn false_positive_rate(&self) -> f64 {`
  - L211: `pub const fn recorded(&self) -> u64 {`
  - L223: `pub const fn classified_hostile(&self) -> u64 {`
- Test/behavior coverage: 4 named functions in source, primarily module unit tests; representative names: `outcomes_round_trip_through_the_machine_names`, `the_scoreboard_counts_each_quadrant_once`, `every_truth_and_outcome_lands_in_exactly_one_counter`, `crashed_is_never_a_detection_nor_a_clean_pass`.

### `src/litric-bench/src/stress.rs`

- Inventory: `F0626`; project source / rust-source; 13393 bytes; `Rust`.
- Size check: 413 lines; SHA-256 `281954325b42eb77850e10c2752f027f1d2c583ef943acb90263265d7c1516c1`.
- Source-level purpose: Bounded stress: catastrophic pressure with hard ceilings (MASTER §5–§7). "Catastrophic-equivalent" does not mean unbounded. Every generator here takes an explicit budget and refuses to exceed it — the framework must never become the outage it is trying to simulate. Filesystem stress shapes (deep trees, long names, many entries) are produced as *sample content* plus the plan to materialize them inside the TEST_VOLUME only; the 
- Public API / key implementation declarations (14):
  - L20: `pub struct Rng(u64);`
  - L22: `pub fn new(seed: u64) -> Self {`
  - L25: `pub fn next(&mut self) -> u64 {`
  - L39: `pub struct Budget {`
  - L51: `pub const fn conservative() -> Self {`
  - L61: `pub const fn allows(self, files: u64, bytes_each: u64) -> bool {`
  - L69: `pub enum FsAction {`
  - L109: `pub enum MemAction {`
  - L133: `pub enum CpuAction {`
  - L162: `pub enum FloodAction {`
  - L184: `pub struct StressPlan {`
  - L203: `pub fn plan(seed: u64, level: Level, budget: Budget) -> StressPlan {`
  - L287: `pub fn planned_bytes(plan: &StressPlan) -> u64 {`
  - L317: `pub fn planned_files(plan: &StressPlan) -> u64 {`
- Test/behavior coverage: 2 named functions in source, primarily module unit tests; representative names: `plan_is_deterministic_per_seed`, `plan_never_exceeds_budget`.

## `litric-input` — Layer 4 Runtime — device-independent input model

Converts Linux evdev/sysfs devices into normalized input events; includes key/layout/modifier mapping, pointer/button state, per-device source threads, backend trait and scripted backend for tests. Above `InputEvent`, device/backend details are abstracted.

**Security/correctness review:** Security/correctness: strict deny unsafe by default; narrowly documented EVIOCGABS ioctl site for real pointer range; validates record sizes and device classification; scripted source tests stack with no hardware.

### `src/litric-input/Cargo.toml`

- Inventory: `F0717`; configuration / cargo-manifest; 1066 bytes; `TOML`.
- Size check: 28 lines; SHA-256 `26d923269c9d93fd5db2052827fe8c6b88729f0088c7c3e64aeec8660dbb4a54`.
- Package configuration and dependency surface:
  - L1: `[package]`
  - L2: `name        = "litric-input"`
  - L3: `description = "LitricOS input: devices, events, keymaps and pointer state, independent of any renderer"`
  - L4: `version.workspace      = true`
  - L5: `edition.workspace      = true`
  - L6: `rust-version.workspace = true`
  - L7: `license.workspace      = true`
  - L9: `[lints.rust]`
  - L16: `unsafe_code                 = "deny"`
  - L17: `missing_docs                = "warn"`
  - L18: `unused_must_use             = "deny"`
  - L19: `rust_2018_idioms            = { level = "warn", priority = -1 }`
  - L21: `[lints.clippy]`
  - L22: `unwrap_used          = "deny"`
  - L23: `expect_used          = "deny"`
  - L24: `panic                = "deny"`
  - L25: `indexing_slicing     = "deny"`
  - L26: `integer_division     = "warn"`
  - L28: `[dependencies]`

### `src/litric-input/src/backend.rs`

- Inventory: `F0718`; project source / rust-source; 20792 bytes; `Rust`.
- Size check: 601 lines; SHA-256 `a6c48ce5d66cf440ff3c82e41dd941e489355310db68d125d0c0ff28653d8f73`.
- Source-level purpose: Getting bytes off the devices and events into the desktop. One thread per device, each doing a blocking `read()`, feeding a channel the main loop waits on. That shape is chosen deliberately over the alternatives: * **Polling** with `O_NONBLOCK` would burn a wake-up per device per frame even when nobody touches the machine, which is the opposite of what Adaptive Idle is for. * **`epoll`** would be one thread and no polling, and
- Public API / key implementation declarations (23):
  - L33: `const READ_BUFFER: usize = 64 * 24;`
  - L37: `pub struct BackendReport {`
  - L52: `pub fn has_keyboard(&self) -> bool {`
  - L58: `pub fn has_pointer(&self) -> bool {`
  - L64: `pub fn lines(&self) -> Vec<String> {`
  - L94: `pub trait InputSource {`
  - L113: `pub const EV_WAKE: u16 = 0xFFFF;`
  - L123: `pub struct Waker {`
  - L130: `pub fn wake(&self) -> bool {`
  - L145: `pub struct EvdevSource {`
  - L164: `pub fn open(root: &Path, width: f32, height: f32) -> Self {`
  - L214: `pub fn waker(&self) -> Waker {`
  - L222: `pub const fn report(&self) -> &BackendReport {`
  - L227: `pub fn set_layout(&mut self, layout: Layout) {`
  - L260: `const EVIOCGABS_X: u64 = 0x8018_4540;`
  - L261: `const EVIOCGABS_Y: u64 = 0x8018_4541;`
  - L272: `pub fn query(file: &File) -> (f32, f32) {`
  - L298: `pub fn query(_file: &File) -> (f32, f32) {`
  - L374: `pub struct ScriptedSource {`
  - L382: `pub fn new(width: f32, height: f32, events: Vec<RawEvent>) -> Self {`
  - L393: `pub fn empty(width: f32, height: f32) -> Self {`
  - L398: `pub fn push(&mut self, events: &[RawEvent]) {`
  - L406: `pub fn is_empty(&self) -> bool {`
- Test/behavior coverage: 8 named functions in source, primarily module unit tests; representative names: `a_scripted_source_replays_a_packet_at_a_time`, `an_empty_source_polls_empty_forever`, `opening_a_root_with_no_devices_reports_that_rather_than_failing`, `a_device_that_cannot_be_opened_is_reported_with_a_reason`, `a_waker_ends_a_blocked_poll_without_producing_input`, `a_wake_between_real_records_is_dropped_and_the_records_survive`, `the_report_lists_what_opened`, `a_learned_range_is_reported_in_the_devices_own_words`.

### `src/litric-input/src/device.rs`

- Inventory: `F0719`; project source / rust-source; 13660 bytes; `Rust`.
- Size check: 404 lines; SHA-256 `da68c650e3a8fb3deee621613aac42465a1796388dacf9bb45a336008f606e54`.
- Source-level purpose: Finding input devices and working out what they are. `/dev/input/event*` gives no hint what a device is: a keyboard, a mouse, a lid switch and a power button all look alike until you ask what they can report. The kernel publishes that as text under `/sys/class/input/eventN/device/capabilities/`, so classification is file reads and bit tests — no `ioctl`, no `unsafe`, and testable from fixtures. /sys/class/input/event3/device/n
- Public API / key implementation declarations (22):
  - L32: `const BITS_PER_WORD: usize = 64;`
  - L36: `pub struct Bitmap {`
  - L44: `pub fn parse(text: &str) -> Self {`
  - L55: `pub fn has(&self, n: u16) -> bool {`
  - L68: `pub fn has_all(&self, bits: &[u16]) -> bool {`
  - L75: `pub struct DeviceKinds {`
  - L88: `pub const fn is_useful(self) -> bool {`
  - L94: `pub fn label(self) -> String {`
  - L120: `pub fn classify(ev: &Bitmap, key: &Bitmap, rel: &Bitmap, abs: &Bitmap) -> DeviceKinds {`
  - L123: `const KEYBOARD_PROOF: &[u16] = &[30, 44, 57, 28];`
  - L143: `pub struct DeviceInfo {`
  - L157: `pub fn summary(&self) -> String {`
  - L177: `pub fn describe(sysfs: &Path, dev_dir: &Path) -> Option<DeviceInfo> {`
  - L206: `pub fn discover(root: &Path) -> Vec<DeviceInfo> {`
  - L222: `pub fn usable(root: &Path) -> Vec<DeviceInfo> {`
  - L235: `const KEYBOARD_KEY: &str = "402000000 3803078f800d001 feffffdfffefffff fffffffffffffffe";`
  - L236: `const KEYBOARD_EV: &str = "120013";`
  - L239: `const MOUSE_EV: &str = "17";`
  - L240: `const MOUSE_KEY: &str = "70000 0 0 0 0";`
  - L241: `const MOUSE_REL: &str = "1943";`
  - L244: `const POWER_EV: &str = "3";`
  - L245: `const POWER_KEY: &str = "10000000000000 0";`
- Test/behavior coverage: 11 named functions in source, primarily module unit tests; representative names: `a_bitmap_indexes_words_from_the_right`, `a_single_word_bitmap_reads_its_low_bits`, `a_bit_past_the_end_is_absent_rather_than_a_panic`, `junk_in_a_bitmap_reads_as_zero_rather_than_failing`, `a_keyboard_is_recognised`, `a_mouse_is_recognised`, `a_power_button_is_not_a_keyboard`, `a_touchscreen_is_recognised_and_a_tablet_without_touch_is_not`, `discovery_reads_a_fixture_tree_and_sorts_it`, `a_root_with_no_input_class_yields_nothing_rather_than_failing`, `a_device_with_no_name_file_still_appears`.

### `src/litric-input/src/evdev.rs`

- Inventory: `F0720`; project source / rust-source; 11320 bytes; `Rust`.
- Size check: 332 lines; SHA-256 `fd38d083e43cacba724754dbeb12f3959eba2c9b66615027cf060ae483f8164b`.
- Source-level purpose: The Linux evdev wire format. `/dev/input/event*` delivers a stream of fixed-size records. That is all this module knows: it turns bytes into [`RawEvent`] and does not care where the bytes came from, which is what lets the whole input stack be tested from a byte array on a machine with no input devices. struct input_event {          64-bit ABI       32-bit ABI struct timeval time;      16 bytes          8 bytes __u16 type;     
- Public API / key implementation declarations (34):
  - L23: `pub const EV_SYN: u16 = 0x00;`
  - L25: `pub const EV_KEY: u16 = 0x01;`
  - L27: `pub const EV_REL: u16 = 0x02;`
  - L29: `pub const EV_ABS: u16 = 0x03;`
  - L31: `pub const EV_MSC: u16 = 0x04;`
  - L33: `pub const EV_LED: u16 = 0x11;`
  - L36: `pub const SYN_REPORT: u16 = 0;`
  - L39: `pub const REL_X: u16 = 0x00;`
  - L41: `pub const REL_Y: u16 = 0x01;`
  - L43: `pub const REL_WHEEL: u16 = 0x08;`
  - L45: `pub const REL_HWHEEL: u16 = 0x06;`
  - L47: `pub const REL_WHEEL_HI_RES: u16 = 0x0B;`
  - L49: `pub const REL_HWHEEL_HI_RES: u16 = 0x0C;`
  - L52: `pub const ABS_X: u16 = 0x00;`
  - L54: `pub const ABS_Y: u16 = 0x01;`
  - L57: `pub const BTN_LEFT: u16 = 0x110;`
  - L59: `pub const BTN_RIGHT: u16 = 0x111;`
  - L61: `pub const BTN_MIDDLE: u16 = 0x112;`
  - L63: `pub const BTN_SIDE: u16 = 0x113;`
  - L65: `pub const BTN_EXTRA: u16 = 0x114;`
  - L67: `pub const BTN_TOUCH: u16 = 0x14A;`
  - L70: `pub const VALUE_UP: i32 = 0;`
  - L72: `pub const VALUE_DOWN: i32 = 1;`
  - L74: `pub const VALUE_REPEAT: i32 = 2;`
  - L77: `pub const RECORD_SIZE_64: usize = 24;`
  - L79: `pub const RECORD_SIZE_32: usize = 16;`
  - L83: `pub enum Abi {`
  - L97: `pub const fn native() -> Self {`
  - L107: `pub const fn record_size(self) -> usize {`
  - L117: `pub struct RawEvent {`
  - L138: `pub fn parse(abi: Abi, bytes: &[u8]) -> Option<Self> {`
  - L195: `pub const fn millis(&self) -> i64 {`
  - L203: `pub const fn is_sync(&self) -> bool {`
  - L216: `pub fn parse_stream(abi: Abi, bytes: &[u8]) -> (Vec<RawEvent>, usize) {`
- Test/behavior coverage: 8 named functions in source, primarily module unit tests; representative names: `a_key_press_parses`, `a_negative_relative_motion_keeps_its_sign`, `a_short_buffer_is_refused_rather_than_padded`, `the_record_size_is_the_kernel_abi_not_a_guess`, `a_stream_yields_whole_records_and_leaves_a_partial_one`, `an_empty_stream_consumes_nothing`, `a_32_bit_record_parses_with_the_32_bit_abi`, `a_broken_clock_cannot_wrap_the_millisecond_conversion`.

### `src/litric-input/src/event.rs`

- Inventory: `F0721`; project source / rust-source; 7767 bytes; `Rust`.
- Size check: 263 lines; SHA-256 `b0fd836085c6854bff5f22ebf35ccad554df0703934b0d41956a2a826b4061ca`.
- Source-level purpose: The events the desktop actually consumes. Nothing above this module knows what evdev is. That is the point: the desktop reacts to "the pointer moved to (412, 88)" and "the user typed é", never to `EV_REL/REL_X/-7`, so replacing the backend — libinput, a Wayland seat, a remote session, a test harness — changes one crate and nothing else.
- Public API / key implementation declarations (8):
  - L13: `pub enum PointerButton {`
  - L29: `pub const fn name(self) -> &'static str {`
  - L42: `pub struct Point {`
  - L52: `pub const fn new(x: f32, y: f32) -> Self {`
  - L58: `pub fn distance(self, other: Self) -> f32 {`
  - L68: `pub enum InputEvent {`
  - L150: `pub const fn is_user_activity(&self) -> bool {`
  - L164: `pub fn describe(&self) -> String {`
- Test/behavior coverage: 3 named functions in source, primarily module unit tests; representative names: `user_activity_is_what_a_person_did`, `descriptions_include_the_modifiers_that_were_held`, `a_double_click_says_so_and_a_release_does_not`.

### `src/litric-input/src/keys.rs`

- Inventory: `F0722`; project source / rust-source; 19619 bytes; `Rust`.
- Size check: 654 lines; SHA-256 `4a3683abb56b8de21340bf69094d5c2acdd33a576dfe4fae152c73b2f9d6d73f`.
- Source-level purpose: Keys, modifiers and keyboard layouts. The kernel reports a **scancode**: a number for a physical key, the same number whichever alphabet is printed on it. Turning that into "the user typed é" is the layout's job, and the layout is data, not code, so that adding Thai or German is adding a table rather than editing a renderer. Two different questions are being answered here and they must not be confused, because confusing them i
- Public API / key implementation declarations (19):
  - L22: `pub enum Side {`
  - L32: `pub enum Key {`
  - L107: `pub const fn is_modifier(self) -> bool {`
  - L116: `pub fn label(self) -> String {`
  - L163: `pub struct Modifiers {`
  - L184: `pub const NONE: Self = Self {`
  - L199: `pub const fn any_action_modifier(self) -> bool {`
  - L206: `pub const fn normalised(self) -> Self {`
  - L220: `pub fn prefix(self) -> String {`
  - L240: `pub struct KeyDef {`
  - L254: `pub struct Layout {`
  - L275: `pub const US_QWERTY: Layout = Layout {`
  - L331: `pub const UK_QWERTY: Layout = Layout {`
  - L388: `pub const LAYOUTS: &[Layout] = &[US_QWERTY, UK_QWERTY];`
  - L396: `pub fn layout_by_name(name: &str) -> Option<Layout> {`
  - L403: `pub fn plain(&self, code: u16) -> Option<char> {`
  - L414: `pub fn text(&self, code: u16, mods: Modifiers) -> Option<char> {`
  - L432: `pub fn key(&self, code: u16) -> Key {`
  - L442: `pub fn named_key(code: u16) -> Key {`
- Test/behavior coverage: 11 named functions in source, primarily module unit tests; representative names: `letters_resolve_through_the_layout`, `shift_produces_the_shifted_character`, `caps_lock_affects_letters_and_leaves_the_number_row_alone`, `a_command_is_not_text`, `a_key_with_no_character_produces_no_text`, `the_uk_layout_differs_where_it_should_and_agrees_elsewhere`, `a_shortcut_key_is_the_same_on_both_layouts_where_the_key_is_the_same`, `every_layout_covers_the_whole_alphabet_and_the_digits`, `normalising_drops_lock_state_but_keeps_the_real_modifiers`, `shortcut_labels_read_the_way_they_are_written`, `an_unknown_scancode_is_carried_not_dropped`.

### `src/litric-input/src/lib.rs`

- Inventory: `F0723`; project source / rust-source; 1947 bytes; `Rust`.
- Size check: 48 lines; SHA-256 `71f347d7a1fb2dd114a6bdde55d18aaae52de038fc79a4bc919d4b42af9f48c9`.
- Source-level purpose: LitricOS input. **Layer 4 (Runtime).** It depends on nothing above the kernel's own interfaces and knows nothing about rendering, which is the whole design: the desktop reacts to *what the user did*, never to what a particular device reported. /dev/input/event*  ─►  evdev   raw records state   modifiers, pointer, clicks keys    layout: scancode -> key and text event   what the desktop consumes /sys/class/input   ─►  device  di
- Public API / key implementation declarations (10):
  - L38: `pub mod backend;`
  - L39: `pub mod device;`
  - L40: `pub mod evdev;`
  - L41: `pub mod event;`
  - L42: `pub mod keys;`
  - L43: `pub mod state;`
  - L45: `pub use backend::{BackendReport, EvdevSource, InputSource, ScriptedSource, Waker};`
  - L46: `pub use event::{InputEvent, Point, PointerButton};`
  - L47: `pub use keys::{Key, Layout, Modifiers, Side};`
  - L48: `pub use state::InputState;`
- Test/behavior coverage: 0 named functions in source, primarily module unit tests; representative names: no inline `#[test]` functions detected.

### `src/litric-input/src/state.rs`

- Inventory: `F0724`; project source / rust-source; 27737 bytes; `Rust`.
- Size check: 780 lines; SHA-256 `f17b7433aa3c281eff7ce435e344b158334c32d352b55a353d0a297fc88f3bea`.
- Source-level purpose: Turning raw device records into desktop events. This is where the accumulated state lives: which modifiers are held, where the pointer is, when the last click was and where. It is a pure state machine over [`RawEvent`], with no clock, no files and no threads, so every behaviour below — double-click timing, edge clamping, wheel accumulation — is reachable from a test that runs in microseconds. A mouse reports `REL_X`, then `REL
- Public API / key implementation declarations (14):
  - L31: `pub const DOUBLE_CLICK_MS: i64 = 400;`
  - L35: `pub const DOUBLE_CLICK_SLOP: f32 = 6.0;`
  - L38: `const HI_RES_PER_DETENT: f32 = 120.0;`
  - L53: `pub struct InputState {`
  - L77: `pub fn new(width: f32, height: f32) -> Self {`
  - L115: `pub fn set_abs_max(&mut self, x: f32, y: f32) {`
  - L125: `pub const fn abs_max(&self) -> (f32, f32) {`
  - L130: `pub fn set_layout(&mut self, layout: Layout) {`
  - L136: `pub const fn layout(&self) -> &Layout {`
  - L141: `pub fn set_bounds(&mut self, width: f32, height: f32) {`
  - L148: `pub const fn pointer(&self) -> Point {`
  - L154: `pub const fn modifiers(&self) -> Modifiers {`
  - L159: `pub fn set_pointer(&mut self, at: Point) {`
  - L185: `pub fn feed(&mut self, raw: RawEvent) -> Vec<InputEvent> {`
- Test/behavior coverage: 19 named functions in source, primarily module unit tests; representative names: `a_keypress_produces_a_key_event_and_the_text_it_typed`, `holding_shift_changes_the_text_but_not_the_key`, `a_modifier_release_clears_it`, `a_command_key_combination_types_nothing`, `an_absolute_tablet_position_maps_the_full_device_range_to_the_screen`, `the_default_range_matches_what_the_automation_harness_sends`, `motion_is_clamped_to_the_display_but_the_delta_is_not`, `a_click_lands_at_the_position_the_same_packet_moved_to`, `two_quick_clicks_in_the_same_place_are_a_double_click`, `a_slow_second_click_is_a_fresh_click`, `a_second_click_somewhere_else_is_a_fresh_click`, `a_triple_click_stops_at_three`, `the_wheel_scrolls_content_down_when_it_turns_forward`, `a_high_resolution_wheel_reports_fractions_of_a_detent`, `nothing_is_emitted_without_a_sync`, `an_empty_packet_produces_nothing`, `resizing_the_display_brings_the_pointer_back_on_screen`, `changing_the_layout_changes_what_is_typed`, `a_whole_recorded_packet_parses_and_replays`.

## `litric-sysd` — Layer 4 Runtime — first LitricOS daemon and system control protocol

System facts, platform probes, provider action plans, capability-gated state machine, LSP-like line protocol and Unix socket server. CLI modes report, serve or query. Fact readers use injected roots/fixtures; action providers validate fixed arguments and use bounded subprocess runner. Some advertised state intentionally remains modeled only: trash is in memory and environment switch does not start a VM.

**Security/correctness review:** Security: strict line and arity limits, percent-encoded fields, capability checks and consent for destructive/provider actions; passwords are sent to helper stdin and omitted from replies; fixed absolute tool paths, validated interface/account/SSID values, deadlines and audit log.

### `src/litric-sysd/Cargo.toml`

- Inventory: `F0766`; configuration / cargo-manifest; 615 bytes; `TOML`.
- Size check: 25 lines; SHA-256 `30236c6d6e7f2ae7fc23b8f16ce8963f0eca4e24522b33f0575f1cec3b08c251`.
- Package configuration and dependency surface:
  - L1: `[package]`
  - L2: `name        = "litric-sysd"`
  - L3: `description = "LitricOS system daemon: the first LitricOS code that runs inside the OS"`
  - L4: `version.workspace      = true`
  - L5: `edition.workspace      = true`
  - L6: `rust-version.workspace = true`
  - L7: `license.workspace      = true`
  - L9: `[lints]`
  - L10: `workspace = true`
  - L12: `[dependencies]`
  - L13: `litric-core    = { path = "../litric-core" }`
  - L14: `litric-caps    = { path = "../litric-caps" }`
  - L15: `litric-env     = { path = "../litric-env" }`
  - L16: `litric-storage = { path = "../litric-storage" }`
  - L17: `litric-trash   = { path = "../litric-trash" }`
  - L19: `[lib]`
  - L20: `name = "litric_sysd"`
  - L21: `path = "src/lib.rs"`
  - L23: `[[bin]]`
  - L24: `name = "litric-sysd"`
  - L25: `path = "src/main.rs"`

### `src/litric-sysd/src/action.rs`

- Inventory: `F0767`; project source / rust-source; 22705 bytes; `Rust`.
- Size check: 561 lines; SHA-256 `59cc548d2df8f30cc2568dca997b531ad79d3589930cecdffaa414e442551a16`.
- Source-level purpose: The write half of the wave-1 providers: what the session *does* to the machine, as opposed to what [`crate::platform`] reads from it. The runner seam, and the power transition that was the first action to execute. [`crate::providers`] is the rest of the wave-1 write half — audio, network, Bluetooth and authentication — and it is a separate module because this one is about *how a program is run* and that one is about *which pro
- Public API / key implementation declarations (23):
  - L47: `pub const SYSTEMCTL: &str = "/usr/bin/systemctl";`
  - L50: `pub const POWER_TIMEOUT_SECS: u32 = 30;`
  - L54: `pub enum PowerAction {`
  - L65: `pub const ALL: [Self; 3] = [Self::Suspend, Self::Reboot, Self::PowerOff];`
  - L69: `pub const fn token(self) -> &'static str {`
  - L80: `pub const fn verb(self) -> &'static str {`
  - L86: `pub fn parse(s: &str) -> Option<Self> {`
  - L104: `pub fn capability(self) -> Option<Capability> {`
  - L112: `pub const fn prompt(self) -> &'static str {`
  - L122: `pub const fn consequence(self) -> &'static str {`
  - L145: `pub trait Runner: fmt::Debug + Send + Sync {`
  - L179: `pub struct SystemRunner;`
  - L247: `pub struct Outcome {`
  - L288: `pub fn power(action: PowerAction, runner: &dyn Runner) -> Outcome {`
  - L312: `pub(crate) mod testing {`
  - L317: `type Call = (String, Vec<String>);`
  - L320: `type Calls = Arc<Mutex<Vec<Call>>>;`
  - L329: `pub struct RecordingRunner {`
  - L341: `pub fn succeeding(code: i32) -> Self {`
  - L352: `pub fn failing(kind: io::ErrorKind, what: &str) -> Self {`
  - L363: `pub fn calls(&self) -> Vec<(String, Vec<String>)> {`
  - L372: `pub fn stdin_len(&self) -> Option<usize> {`
  - L378: `pub fn timeout_secs(&self) -> Option<u32> {`
- Test/behavior coverage: 9 named functions in source, primarily module unit tests; representative names: `every_action_parses_its_own_token_and_has_a_capability`, `the_three_verbs_do_not_authorise_each_other`, `a_transition_runs_the_base_tool_with_a_fixed_argv`, `the_argv_is_never_built_from_anything_a_caller_said`, `a_nonzero_exit_is_not_success_and_says_why`, `a_missing_systemctl_is_reported_rather_than_hidden`, `the_real_runner_reports_a_program_that_is_not_there_rather_than_panicking`, `the_real_runner_feeds_a_program_its_standard_input`, `a_tool_that_never_exits_is_killed_and_reported_as_a_timeout`.

### `src/litric-sysd/src/facts.rs`

- Inventory: `F0768`; project source / rust-source; 48922 bytes; `Rust`.
- Size check: 1367 lines; SHA-256 `3ef2fb7b4fada0eb4fb0755c25972c5a09b6304ae669e89a653869f906cdb1d6`.
- Source-level purpose: Reading what the machine is actually doing. Every claim LitricOS makes about being light has to come from a measurement taken on the running system, not from a design document. This module is where those measurements are read. The parsers take `&str` and the readers take a root directory, so the whole module is testable on a host that has no `/proc` at all. That is deliberate: a parser that can only be exercised on the target 
- Public API / key implementation declarations (46):
  - L17: `pub struct MemInfo {`
  - L42: `pub fn parse(text: &str) -> Option<Self> {`
  - L82: `pub const fn used_kb(&self) -> u64 {`
  - L88: `pub const fn swap_used_kb(&self) -> u64 {`
  - L95: `pub struct Uptime {`
  - L105: `pub fn parse(text: &str) -> Option<Self> {`
  - L118: `pub struct CpuTimes {`
  - L159: `pub fn parse(text: &str) -> Option<Self> {`
  - L173: `pub fn parse_all(text: &str) -> Vec<Self> {`
  - L188: `pub const fn total(&self) -> u64 {`
  - L202: `pub const fn busy(&self) -> u64 {`
  - L215: `pub fn busy_fraction_since_boot(&self) -> Option<f64> {`
  - L229: `pub fn utilisation(earlier: Self, later: Self) -> Option<f64> {`
  - L243: `pub struct LoadAvg {`
  - L259: `pub fn parse(text: &str) -> Option<Self> {`
  - L282: `pub fn parse_vm_rss_kb(status: &str) -> Option<u64> {`
  - L303: `pub fn parse_cpu_model(text: &str) -> Option<String> {`
  - L304: `const KEYS: [&str; 3] = ["model name", "Hardware", "Model"];`
  - L331: `pub struct MountSpec {`
  - L346: `pub fn unescape_mount(path: &str) -> String {`
  - L376: `pub fn parse_mounts(text: &str) -> Vec<MountSpec> {`
  - L399: `pub fn parse_mglru(text: &str) -> Option<bool> {`
  - L413: `pub struct ProcStat {`
  - L430: `pub fn parse_proc_stat(text: &str) -> Option<ProcStat> {`
  - L471: `pub fn parse_uid(status: &str) -> Option<u32> {`
  - L483: `pub struct ProcessFacts {`
  - L514: `pub const fn ticks(&self) -> u64 {`
  - L520: `pub const fn state_word(&self) -> &'static str {`
  - L537: `pub fn display_name(&self) -> &str {`
  - L559: `pub fn read_process(root: &Path, pid: u32) -> Option<ProcessFacts> {`
  - L594: `pub fn read_processes(root: &Path, limit: usize) -> Vec<ProcessFacts> {`
  - L623: `pub fn cpu_shares(`
  - L648: `pub struct Facts {`
  - L694: `pub fn escape_field(s: &str) -> String {`
  - L709: `pub fn parse_cpu_online(text: &str) -> Option<u32> {`
  - L741: `pub fn read_facts(root: &Path) -> Facts {`
  - L782: `pub const MAX_PROCESSES: usize = 512;`
  - L791: `pub fn cmdline_param(&self, key: &str) -> Option<&str> {`
  - L805: `pub fn root_verity(&self) -> Option<bool> {`
  - L815: `pub fn to_lines(&self) -> Vec<String> {`
  - L912: `const MEMINFO: &str = "\`
  - L1078: `const CPUINFO_X86: &str = "\`
  - L1086: `const CPUINFO_ARM: &str = "\`
  - L1106: `const STAT_PER_CORE: &str = "\`
  - L1143: `const MOUNTS: &str = "\`
  - L1196: `const ODD_STAT: &str = "4242 (my (odd) name) S 1 4242 4242 0 -1 4194560 100 0 0 0 \`
- Test/behavior coverage: 26 named functions in source, primarily module unit tests; representative names: `meminfo_parses_the_fields_that_matter`, `meminfo_without_memtotal_is_rejected_rather_than_defaulted`, `uptime_parses`, `cpu_times_split_busy_from_idle_with_iowait_on_the_idle_side`, `cpu_line_must_be_the_aggregate_not_a_per_core_line`, `busy_since_boot_is_an_average_and_says_so_when_it_cannot_be_computed`, `utilisation_rejects_a_counter_that_went_backwards`, `loadavg_parses_including_the_task_fraction`, `vm_rss_parses`, `mglru_is_a_bitmask_not_a_boolean`, `cpu_online_handles_ranges_and_lists`, `reading_a_root_that_has_no_proc_yields_nothing_rather_than_zeroes`, `cmdline_parameters_are_read_with_the_last_occurrence_winning`, `a_root_without_verity_reports_off_rather_than_nothing`, `the_cpu_model_comes_from_the_kernel_not_from_a_table`, `per_cpu_counters_are_read_in_order_and_never_from_the_aggregate`, `utilisation_is_the_ratio_of_the_difference_between_two_samples`, `mounts_report_read_only_and_unescape_the_mount_point`, `facts_read_from_a_fixture_root_and_report_stable_lines`, `a_process_name_with_spaces_and_parens_does_not_shift_the_fields`, `a_process_with_no_name_in_parens_is_refused_rather_than_guessed`, `the_real_uid_is_the_first_of_the_four_on_the_uid_line`, `a_process_walks_the_real_layout_of_a_proc_directory`, `cpu_share_is_a_fraction_of_the_whole_machine_and_fabricates_nothing`.

### `src/litric-sysd/src/lib.rs`

- Inventory: `F0769`; project source / rust-source; 2014 bytes; `Rust`.
- Size check: 45 lines; SHA-256 `19e4963d2a88ae8b391faee95125c2236fece11ca1d55e822dc14b38bba6b641`.
- Source-level purpose: LitricOS system daemon, as a library. **Layer 4 (Runtime).** Everything below — the capability engine, the storage broker, the trash policy, the environment manager — is real code with passing tests and no way to run. This is what runs it, and the binary in `main.rs` is a thin shell around it. It is a library and not only a binary for two reasons. The client half of the protocol is what the shell and the settings application w
- Public API / key implementation declarations (7):
  - L37: `pub mod action;`
  - L38: `pub mod facts;`
  - L39: `pub mod platform;`
  - L40: `pub mod proto;`
  - L41: `pub mod providers;`
  - L42: `pub mod state;`
  - L45: `pub mod server;`
- Test/behavior coverage: 0 named functions in source, primarily module unit tests; representative names: no inline `#[test]` functions detected.

### `src/litric-sysd/src/main.rs`

- Inventory: `F0770`; project source / rust-source; 8987 bytes; `Rust`.
- Size check: 289 lines; SHA-256 `6e46fa97eec464266e85a2b2d59c5a0735143dd95dd18c69d8320c0810eab25d`.
- Source-level purpose: `litric-sysd` — the LitricOS system daemon. **Layer 4 (Runtime).** The first LitricOS binary that runs *inside* the operating system rather than beside it. Everything below it — the capability engine, the storage broker, the trash policy, the environment manager — was already real code with passing tests and no way to run. This is what runs it. Nothing above Layer 4 can be honest until this exists: a shell that draws a trash i
- Public API / key implementation declarations (5):
  - L40: `const VERSION: &str = env!("CARGO_PKG_VERSION");`
  - L46: `const DEFAULT_SOCKET: &str = "/run/litric/sysd.sock";`
  - L50: `enum Mode {`
  - L64: `struct Args {`
  - L120: `const USAGE: &str = "\`
- Test/behavior coverage: 2 named functions in source, primarily module unit tests; representative names: `an_option_without_its_value_is_an_error_not_a_default`, `an_unknown_argument_is_refused_rather_than_ignored`.

### `src/litric-sysd/src/platform/audio.rs`

- Inventory: `F0771`; project source / rust-source; 13897 bytes; `Rust`.
- Size check: 381 lines; SHA-256 `cc72ed5c78a6175524abf0626e67b72a36ec38d9d3f21a50eb3fd613d6d727d3`.
- Source-level purpose: Audio facts: cards, PCM devices, and whether a sound server is running. PipeWire is the sound server and WirePlumber is its policy; neither has a file interface. Their protocol is the PipeWire native protocol over `/run/user/<uid>/pipewire-0`, and speaking it in-tree is a subsystem rather than a task — and taking `libpipewire` as a dependency breaks the dependency policy this tree is built on. So this module reports what the *
- Public API / key implementation declarations (9):
  - L44: `pub struct Card {`
  - L57: `pub struct Pcm {`
  - L72: `pub struct Server {`
  - L83: `pub struct AudioFacts {`
  - L102: `pub fn to_lines(&self) -> Vec<String> {`
  - L138: `pub fn read(root: &Path) -> AudioFacts {`
  - L170: `pub fn parse_cards(text: &str) -> Vec<Card> {`
  - L212: `pub fn parse_pcm(text: &str) -> Vec<Pcm> {`
  - L257: `pub fn find_server(root: &Path) -> Option<Server> {`
- Test/behavior coverage: 3 named functions in source, primarily module unit tests; representative names: `cards_are_parsed_and_the_detail_line_is_not_a_card`, `a_card_line_with_no_name_still_reports_the_driver`, `a_machine_with_no_sound_hardware_is_reported_as_having_none`.

### `src/litric-sysd/src/platform/bluetooth.rs`

- Inventory: `F0772`; project source / rust-source; 11494 bytes; `Rust`.
- Size check: 327 lines; SHA-256 `f565ee1561ac81920edd530b79ee0b4dc91880422ac8c6c655e43f30c0e0e86f`.
- Source-level purpose: Bluetooth facts: adapters, radio blocks, and whether a daemon is present. Bluetooth is the one subsystem in this wave with **no userspace daemon in the image**: `bluez` is in no profile, so there is no `bluetoothd` to pair through and pairing, trust and connect are not implementable here yet (`docs/design/wave1/01-bluetooth.md` §2 — a product decision, not an oversight). What is readable anyway, and what this module therefore 
- Public API / key implementation declarations (9):
  - L33: `pub struct Adapter {`
  - L45: `pub struct Radio {`
  - L61: `pub const fn blocked(&self) -> bool {`
  - L67: `pub const fn blocked_by(&self) -> &'static str {`
  - L82: `pub struct BtFacts {`
  - L102: `pub fn state(&self) -> &'static str {`
  - L122: `pub fn to_lines(&self) -> Vec<String> {`
  - L163: `pub fn read(root: &Path) -> BtFacts {`
  - L205: `pub fn parse_bool(text: &str) -> bool {`
- Test/behavior coverage: 4 named functions in source, primarily module unit tests; representative names: `a_machine_with_no_radio_says_so`, `a_radio_with_no_daemon_is_not_reported_as_working`, `only_bluetooth_radios_decide_the_bluetooth_verdict`, `a_host_fixture_is_read_end_to_end`.

### `src/litric-sysd/src/platform/input.rs`

- Inventory: `F0773`; project source / rust-source; 11813 bytes; `Rust`.
- Size check: 315 lines; SHA-256 `4ee35c5aa4f6a51f4e80e10b88c092cfa92ebaff093800b45fca9ec822a57803`.
- Source-level purpose: Input device facts: every keyboard, pointer and tablet that is attached. The shell reads `/dev/input/event*` itself — `litric-input` opens the character devices and parses the evdev stream. What it cannot answer from a character device before opening it is *what kind of thing it is*, and that is exactly the question Settings has to answer: a machine with no keyboard attached and a machine whose keyboard failed to probe both de
- Public API / key implementation declarations (13):
  - L34: `pub enum Kind {`
  - L49: `pub const fn as_str(self) -> &'static str {`
  - L61: `pub struct Device {`
  - L72: `pub struct InputFacts {`
  - L80: `pub fn keyboards(&self) -> usize {`
  - L90: `pub fn pointers(&self) -> usize {`
  - L102: `pub fn to_lines(&self) -> Vec<String> {`
  - L122: `const KEY_A: u32 = 30;`
  - L124: `const REL_X: u32 = 0;`
  - L125: `const REL_Y: u32 = 1;`
  - L127: `const ABS_X: u32 = 0;`
  - L128: `const ABS_Y: u32 = 1;`
  - L165: `pub fn read(root: &Path) -> InputFacts {`
- Test/behavior coverage: 6 named functions in source, primarily module unit tests; representative names: `a_machine_with_no_input_reports_zero_and_not_none`, `a_keyboard_is_recognised_by_the_letter_a_and_not_by_its_buttons`, `a_tablet_is_a_pointer_even_though_it_reports_no_deltas`, `a_mouse_is_recognised_by_its_two_relative_axes`, `a_node_whose_name_cannot_be_read_is_listed_rather_than_dropped`, `a_bitmap_is_read_word_by_word_not_as_one_number`.

### `src/litric-sysd/src/platform/mod.rs`

- Inventory: `F0774`; project source / rust-source; 9029 bytes; `Rust`.
- Size check: 211 lines; SHA-256 `ad2a745c2c738ee635b64f93a1157c323031b253e7357cb9824683570c498df2`.
- Source-level purpose: Reading what the machine's subsystems are doing. [`crate::facts`] answers "how busy is this machine". This module answers "what is this machine": which interfaces exist and which are up, what the sound cards are, what the battery is doing, whether there is a Bluetooth radio and whether firmware blocked it, who is logged in. Every line here is read from a file the kernel maintains, or from a file a provider maintains in a docum
- Public API / key implementation declarations (12):
  - L30: `pub mod audio;`
  - L31: `pub mod bluetooth;`
  - L32: `pub mod input;`
  - L33: `pub mod net;`
  - L34: `pub mod power;`
  - L35: `pub mod session;`
  - L48: `pub struct Platform {`
  - L82: `pub fn to_lines(&self) -> Vec<String> {`
  - L105: `pub fn read(root: &Path) -> Platform {`
  - L127: `pub(crate) fn read_file(root: &Path, rel: &str) -> Option<String> {`
  - L135: `pub(crate) fn read_attr(root: &Path, rel: &str) -> Option<String> {`
  - L143: `pub(crate) fn read_dir(root: &Path, rel: &str) -> Vec<String> {`
- Test/behavior coverage: 3 named functions in source, primarily module unit tests; representative names: `a_root_that_does_not_exist_produces_no_claims_at_all`, `an_empty_but_readable_machine_reports_zeroes_that_are_true`, `reading_a_directory_that_is_a_file_is_not_an_error`.

### `src/litric-sysd/src/platform/net.rs`

- Inventory: `F0775`; project source / rust-source; 30985 bytes; `Rust`.
- Size check: 838 lines; SHA-256 `a20c4d38dc01c66b768c9eb9d1ed5df34dd2f8c22abda1dba6e72716636af3e4`.
- Source-level purpose: Network facts: interfaces, addresses, routes, DNS. `systemd-networkd` is the manager and `iwd` owns Wi-Fi (ADR-0023 decision 2). Neither is spoken to here: this module only reports what the *kernel* says, which is the same information both of them act on, and which is available whether or not either daemon is running. Three surfaces, each chosen because it is the kernel's own: | Fact | Surface | Why this one | |---|---|---| | 
- Public API / key implementation declarations (18):
  - L48: `pub struct Iface {`
  - L71: `pub struct Addr {`
  - L86: `pub struct Route {`
  - L99: `pub struct Dns {`
  - L111: `pub struct NetFacts {`
  - L128: `pub fn to_lines(&self) -> Vec<String> {`
  - L184: `pub fn read(root: &Path) -> NetFacts {`
  - L250: `pub fn parse_operstate(text: &str) -> String {`
  - L260: `pub fn parse_flag(text: &str) -> bool {`
  - L271: `pub fn parse_net_dev(text: &str) -> Vec<(String, (u64, u64))> {`
  - L305: `pub fn parse_route(text: &str) -> Vec<Route> {`
  - L336: `pub fn hex_v4(field: &str) -> Option<std::net::Ipv4Addr> {`
  - L349: `pub fn netmask_to_prefix(mask: &str) -> Option<u8> {`
  - L373: `pub fn parse_if_inet6(text: &str) -> Vec<Addr> {`
  - L405: `pub fn hex_v6(field: &str) -> Option<String> {`
  - L424: `pub fn format_v6(groups: &[u16; 8]) -> String {`
  - L490: `pub fn parse_ip_addr(text: &str) -> Vec<Addr> {`
  - L543: `pub fn parse_resolv_conf(text: &str) -> Dns {`
- Test/behavior coverage: 10 named functions in source, primarily module unit tests; representative names: `operstate_keeps_the_three_answers_apart`, `a_short_net_dev_row_is_skipped_not_padded`, `a_route_with_no_next_hop_says_so_instead_of_naming_address_zero`, `a_route_whose_mask_is_not_a_netmask_is_dropped_rather_than_invented`, `a_mask_that_is_not_a_run_of_ones_has_no_prefix_length`, `ipv6_addresses_are_compressed_the_way_every_other_tool_prints_them`, `a_single_zero_group_is_not_compressed`, `ip_addr_garbage_yields_nothing_rather_than_a_guess`, `resolv_conf_reports_servers_search_and_whether_it_was_there`, `the_reply_says_why_there_are_no_ipv4_addresses`.

### `src/litric-sysd/src/platform/power.rs`

- Inventory: `F0776`; project source / rust-source; 18195 bytes; `Rust`.
- Size check: 503 lines; SHA-256 `ccc514ac362abeff8b124209d9915aa709f875ec2a98aeff49be7dfc9f5ef993`.
- Source-level purpose: Power facts: supplies, thermal zones, backlights. The shell already drew a battery indicator before this module existed, from a reader inside the shell itself. That reader is folded in here so there is one answer to "what is the battery doing" rather than two that can disagree, and so the parsers are exercised by tests instead of only on a laptop. Two rules are lifted from that reader and kept, because both were learned from a
- Public API / key implementation declarations (14):
  - L28: `pub enum SupplyKind {`
  - L43: `pub fn parse(text: &str) -> Self {`
  - L54: `pub const fn token(self) -> &'static str {`
  - L66: `pub struct Supply {`
  - L95: `pub const fn is_battery(&self) -> bool {`
  - L105: `pub fn percent(&self) -> Option<u8> {`
  - L127: `pub struct Zone {`
  - L139: `pub struct Cooling {`
  - L150: `pub struct Backlight {`
  - L161: `pub struct PowerFacts {`
  - L175: `pub fn to_lines(&self) -> Vec<String> {`
  - L247: `pub fn read(root: &Path) -> PowerFacts {`
  - L325: `pub fn parse_status(text: &str) -> &'static str {`
  - L342: `pub fn parse_capacity(text: &str) -> Option<u8> {`
- Test/behavior coverage: 4 named functions in source, primarily module unit tests; representative names: `a_desktop_reports_no_battery_rather_than_a_flat_one`, `percentage_can_come_from_energy_when_there_is_no_capacity_file`, `a_capacity_the_kernel_does_not_know_is_not_a_number`, `a_laptop_fixture_is_read_end_to_end`.

### `src/litric-sysd/src/platform/session.rs`

- Inventory: `F0777`; project source / rust-source; 19125 bytes; `Rust`.
- Size check: 487 lines; SHA-256 `2b230d1e717309ca7cbe8641eb07bfa05cafb4294b25cab7653672f839840a42`.
- Source-level purpose: Session facts, and the honest state of authentication. `systemd-logind` already models sessions, seats and users, and writes each one as a small text record in `/run/systemd/sessions/`. This module reads that record. Inventing a parallel session model — the thing `docs/design/wave1/04-authentication.md` §2.2 argues against — would give the shell a second opinion about who is logged in, and only one of the two can be right. Two
- Public API / key implementation declarations (13):
  - L40: `pub enum Verifier {`
  - L60: `pub const fn token(self) -> &'static str {`
  - L70: `pub fn parse(text: &str) -> Self {`
  - L83: `pub const fn can_verify(self) -> bool {`
  - L96: `pub const fn can_lock(self) -> bool {`
  - L107: `pub const fn refusal(self) -> &'static str {`
  - L124: `pub struct Session {`
  - L149: `pub struct SessionFacts {`
  - L177: `pub fn active(&self) -> Option<&Session> {`
  - L188: `pub fn to_lines(&self) -> Vec<String> {`
  - L230: `pub fn read(root: &Path) -> SessionFacts {`
  - L275: `pub fn parse_session_record(id: &str, text: &str) -> Session {`
  - L316: `const RECORD: &str = "\`
- Test/behavior coverage: 6 named functions in source, primarily module unit tests; representative names: `a_logind_record_is_parsed_field_by_field`, `a_session_that_is_not_in_the_foreground_is_not_called_active`, `a_record_missing_everything_but_an_id_still_parses`, `a_lock_is_offered_exactly_when_a_password_can_be_verified`, `a_machine_with_two_active_sessions_has_no_single_active_one`, `the_verifier_is_found_on_the_filesystem_not_assumed`.

### `src/litric-sysd/src/proto.rs`

- Inventory: `F0778`; project source / rust-source; 32408 bytes; `Rust`.
- Size check: 913 lines; SHA-256 `bc5fb192e095b5f9efb175663110132e1d90805648841be9990d54de74404dd9`.
- Source-level purpose: The line protocol `litric-sysd` speaks on its socket. Deliberately text, deliberately line-oriented, and deliberately boring. The shell, the settings app and the selftest all talk to the daemon; a binary protocol would mean none of them could be debugged with `socat` at three in the morning, and the daemon is not on a path where the encoding is the bottleneck. -> ENV PLAN windows <- OK 2 <- outcome=restoring <- latency=tens-of
- Public API / key implementation declarations (13):
  - L32: `pub const MAX_LINE: usize = 4096;`
  - L36: `pub enum AccessWord {`
  - L48: `pub fn parse(s: &str) -> Option<Self> {`
  - L61: `pub enum Request {`
  - L121: `pub enum ProtoError {`
  - L155: `pub const fn kind(&self) -> &'static str {`
  - L189: `pub fn parse(line: &str) -> Result<Self, ProtoError> {`
  - L513: `pub struct Response {`
  - L523: `pub fn ok(lines: Vec<String>) -> Self {`
  - L534: `pub fn err(kind: &str, detail: impl Into<String>) -> Self {`
  - L545: `pub const fn is_ok(&self) -> bool {`
  - L551: `pub fn lines(&self) -> &[String] {`
  - L562: `pub fn encode(&self) -> String {`
- Test/behavior coverage: 14 named functions in source, primarily module unit tests; representative names: `a_link_takes_up_and_down_and_not_on_and_off`, `the_radio_takes_on_and_off`, `audio_parses_a_percentage_and_a_mute_word`, `a_wifi_join_decodes_hex_and_never_echoes_a_secret`, `a_verification_carries_its_confirmation_or_it_does_not_carry_one`, `unknown_provider_subcommands_are_refused_by_name`, `resolve_parses_all_three_access_words`, `a_power_request_that_is_not_understood_is_refused_rather_than_guessed`, `unknown_verbs_and_subcommands_are_named_in_the_error`, `an_empty_line_is_an_error_not_a_ping`, `an_oversized_line_is_refused_before_it_is_parsed`, `a_response_declares_its_line_count`, `newlines_inside_a_payload_cannot_desynchronise_the_count`, `an_error_reply_carries_a_stable_kind`.

### `src/litric-sysd/src/providers.rs`

- Inventory: `F0779`; project source / rust-source; 61164 bytes; `Rust`.
- Size check: 1572 lines; SHA-256 `082fcdb367150ac5ad2e0bf58ee8c7ef6a74a8d7c0ce537fe541a88aed707114`.
- Source-level purpose: The write half of four wave-1 providers: audio, network, Bluetooth and authentication. [`crate::action`] owns the runner seam and the one transition that existed before this module — a power request. This module is the rest of `docs/design/wave1/`, and it exists because "the facts are already read" was the state of all four of them: a Settings pane could say what the machine *is* and nothing else. ADR-0023 decision 2: the base
- Public API / key implementation declarations (44):
  - L59: `pub const WPCTL: &str = "/usr/bin/wpctl";`
  - L63: `pub const IP: &str = "/usr/bin/ip";`
  - L66: `pub const NETWORKCTL: &str = "/usr/bin/networkctl";`
  - L69: `pub const IWCTL: &str = "/usr/bin/iwctl";`
  - L72: `pub const BLUETOOTHCTL: &str = "/usr/bin/bluetoothctl";`
  - L80: `pub const UNIX_CHKPWD: &str = "/usr/bin/unix_chkpwd";`
  - L84: `const IWD_STATE_DIR: &str = "var/lib/iwd";`
  - L87: `pub const SOCKET_ROUND_TRIP_SECS: u32 = 10;`
  - L91: `pub const RADIO_OPERATION_SECS: u32 = 30;`
  - L95: `pub const LOCAL_CHANGE_SECS: u32 = 15;`
  - L98: `const MAX_PASSPHRASE: usize = 63;`
  - L101: `const MIN_PASSPHRASE: usize = 8;`
  - L104: `const MAX_SSID: usize = 32;`
  - L114: `pub struct Refusal {`
  - L148: `pub fn valid_iface(name: &str) -> bool {`
  - L161: `pub fn valid_user(name: &str) -> bool {`
  - L178: `pub fn valid_bdaddr(addr: &str) -> bool {`
  - L205: `pub fn valid_ssid(bytes: &[u8]) -> bool {`
  - L224: `pub fn valid_passphrase(bytes: &[u8]) -> bool {`
  - L242: `pub fn unhex(text: &str) -> Option<Vec<u8>> {`
  - L264: `pub enum AudioAction {`
  - L274: `pub const fn token(self) -> &'static str {`
  - L283: `pub fn capability(self) -> Option<Capability> {`
  - L291: `pub enum NetAction {`
  - L316: `pub const fn token(&self) -> &'static str {`
  - L332: `pub fn capability(&self) -> Option<Capability> {`
  - L343: `pub fn iwd_dir(root: &Path) -> PathBuf {`
  - L354: `pub fn credential_path(root: &Path, ssid: &str) -> Option<PathBuf> {`
  - L364: `pub enum BtAction {`
  - L380: `pub const fn token(&self) -> &'static str {`
  - L396: `pub fn capability(&self) -> Option<Capability> {`
  - L410: `pub enum AuthAction {`
  - L423: `pub const fn token(&self) -> &'static str {`
  - L434: `pub fn capability(&self) -> Option<Capability> {`
  - L443: `pub enum SystemAction {`
  - L461: `pub fn capability(&self) -> Option<Capability> {`
  - L472: `pub const fn family(&self) -> &'static str {`
  - L483: `pub fn token(&self) -> &'static str {`
  - L501: `pub const fn requires_consent(&self) -> bool {`
  - L516: `pub enum FileChange {`
  - L539: `pub struct Plan {`
  - L573: `pub fn command_line(&self) -> String {`
  - L589: `pub fn plan(&self, root: &Path) -> Result<Plan, Refusal> {`
  - L945: `pub fn perform(action: &SystemAction, runner: &dyn Runner, root: &Path) -> Outcome {`
- Test/behavior coverage: 29 named functions in source, primarily module unit tests; representative names: `interface_names_are_checked_against_the_kernels_rules`, `account_names_follow_the_distributions_rule`, `network_names_are_refused_rather_than_escaped`, `passphrases_are_the_wpa_range_and_ascii_only`, `hex_decoding_round_trips_and_refuses_anything_else`, `every_action_carries_a_capability_for_its_own_scope`, `a_grant_for_one_interface_does_not_cover_another`, `only_a_verification_demands_consent`, `volume_becomes_a_fraction_with_a_ceiling`, `mute_is_a_number_because_that_is_what_the_tool_takes`, `a_link_change_runs_ip_with_a_fixed_argv`, `an_illegal_interface_never_becomes_a_plan`, `joining_a_wifi_network_writes_a_private_credential_and_then_connects`, `the_wireless_interface_is_found_in_the_roots_sysfs`, `forgetting_a_network_removes_the_credential`, `a_pairing_plan_does_not_claim_the_device_is_paired`, `a_radio_grant_covers_every_device_and_a_device_grant_covers_its_own`, `a_password_travels_on_standard_input_and_never_on_the_command_line`, `an_empty_password_is_refused_rather_than_verified`, `performing_an_audio_change_runs_the_tool_once`, `a_missing_tool_is_reported_as_a_failure_and_not_as_success`, `a_refused_action_runs_nothing_at_all`, `a_failed_file_write_stops_before_the_tool_runs`, `the_credential_file_is_written_before_the_tool_is_asked_to_connect`.

### `src/litric-sysd/src/server.rs`

- Inventory: `F0780`; project source / rust-source; 6833 bytes; `Rust`.
- Size check: 183 lines; SHA-256 `914cb159265d47bdb8d1ee9d214e7a2f91b01ec883d6af31e57b0b14d2d63bae`.
- Source-level purpose: The Unix-socket transport. Everything interesting happens in [`crate::state`]; this file only moves bytes. It is compiled on Unix only, which is why the rest of the crate is carefully free of platform assumptions: the daemon's whole behaviour stays testable on the development host, and only the transport needs the target. Two properties are worth stating because they are the ones that bite: * **A request line is length-limited
- Public API / key implementation declarations (3):
  - L44: `pub fn bind(path: &Path) -> io::Result<UnixListener> {`
  - L70: `pub fn serve(listener: &UnixListener, state: Sysd) -> io::Result<()> {`
  - L157: `pub fn query(path: &Path, request: &str) -> io::Result<String> {`
- Test/behavior coverage: 0 named functions in source, primarily module unit tests; representative names: no inline `#[test]` functions detected.

### `src/litric-sysd/src/state.rs`

- Inventory: `F0781`; project source / rust-source; 38903 bytes; `Rust`.
- Size check: 1012 lines; SHA-256 `9f1ec7d9676d878c4cc3f1e200cd61393e770f90dbef8e086fcc9aa90a98c32e`.
- Source-level purpose: The daemon's live state, and what every request does to it. This is the join between the Layer 2 and Layer 3 crates and the running system. Those crates decide *what is true*; this holds one instance of each of them, for the whole machine, and answers questions about it. It contains no socket code and no clock of its own: the caller supplies the time. That keeps the entire behaviour of the daemon reachable from a test that run
- Public API / key implementation declarations (14):
  - L46: `pub const LOCAL_CLIENT: &str = "os.litric.shell";`
  - L63: `pub type Timestamp = u64;`
  - L73: `pub const PROVIDER_GRANTS: [(CapabilityId, &str); 5] = [`
  - L83: `pub struct Sysd {`
  - L103: `pub fn new(total_cpus: u32, total_memory_mb: u32, home: &str, root: &Path) -> Self {`
  - L125: `pub fn with_runner(mut self, runner: Box<dyn Runner>) -> Self {`
  - L132: `pub const fn client(&self) -> Option<&AppId> {`
  - L142: `pub fn from_system(home: &str, root: &Path) -> Self {`
  - L160: `pub const fn environments(&self) -> &EnvironmentManager {`
  - L166: `pub const fn trash(&self) -> &Trash {`
  - L172: `pub fn caps_mut(&mut self) -> &mut Broker {`
  - L178: `pub fn facts(&self) -> Facts {`
  - L186: `pub fn handle(&mut self, req: &Request, now: Timestamp) -> Response {`
  - L521: `const NOW: Timestamp = 1_800_000_000;`
- Test/behavior coverage: 25 named functions in source, primarily module unit tests; representative names: `trash_plan_empty`, `env_list_reports_every_environment_and_the_host_reservation`, `planning_a_switch_changes_nothing`, `a_switch_says_it_only_moved_the_state_machine`, `an_unknown_environment_is_refused_by_name`, `an_empty_trash_reports_zero_rather_than_failing`, `resolving_without_a_grant_is_denied`, `resolving_with_a_grant_returns_a_real_path`, `a_grant_that_has_expired_no_longer_resolves`, `a_malformed_app_id_is_rejected_before_anything_else_happens`, `the_local_client_is_the_platform_shell_and_holds_only_implemented_actions`, `a_power_request_without_consent_never_runs_anything`, `a_confirmed_power_request_runs_the_base_tool_once`, `a_transition_that_did_not_happen_is_an_error_not_a_success`, `a_revoked_grant_cannot_power_the_machine_off`, `every_power_request_appears_in_the_audit_log`, `an_authorised_volume_change_runs_the_audio_tool_once`, `a_provider_action_without_a_grant_runs_nothing`, `a_verification_without_the_confirmation_word_never_reaches_the_helper`, `a_confirmed_verification_reaches_the_helper_with_the_password_on_stdin`, `a_provider_action_that_failed_is_an_error_and_says_which_kind`, `a_provider_action_with_an_illegal_argument_is_the_callers_error`, `every_provider_request_appears_in_the_audit_log`, `facts_from_an_unreadable_root_are_absent_rather_than_zero`.

## `litric-wm` — Layer 5 Native Ecosystem — display-independent window model

Pure geometry and window manager: frame/chrome geometry, window state/content, focus/z-order/MRU switcher, workspaces, snap/maximize/minimize, hit-testing, drag/resize constraints and damage rectangles. No rendering, devices, clock, I/O or dependencies.

**Security/correctness review:** Correctness: deterministic unit coverage for geometry edges, non-dangling focus, minimum sizes, title-bar reachability, workspace visibility, snap/restore and damage reporting.

### `src/litric-wm/Cargo.toml`

- Inventory: `F0792`; configuration / cargo-manifest; 316 bytes; `TOML`.
- Size check: 12 lines; SHA-256 `3ec22e33ca97fb27b0aa7992fc952139416aace196f11cb517e45f045a33de0f`.
- Package configuration and dependency surface:
  - L1: `[package]`
  - L2: `name        = "litric-wm"`
  - L3: `description = "LitricOS window manager: windows, focus, z-order, workspaces and interaction, with no rendering in it"`
  - L4: `version.workspace      = true`
  - L5: `edition.workspace      = true`
  - L6: `rust-version.workspace = true`
  - L7: `license.workspace      = true`
  - L9: `[lints]`
  - L10: `workspace = true`
  - L12: `[dependencies]`

### `src/litric-wm/src/geom.rs`

- Inventory: `F0793`; project source / rust-source; 8262 bytes; `Rust`.
- Size check: 259 lines; SHA-256 `4c7083c96b32754c161bedad68d6a458dbde11a110748802784ea174943ece91`.
- Source-level purpose: Rectangles and the chrome metrics that hit-testing and drawing must share. Everything here is in **logical units**. A logical unit is a pixel at 100% scale and half a pixel at 200%, so a layout written once works on a 1280x800 console and a 4K panel. The renderer multiplies by the scale factor exactly once, at the edge; nothing in the window manager knows what a physical pixel is.
- Public API / key implementation declarations (18):
  - L11: `pub struct Rect {`
  - L25: `pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {`
  - L30: `pub const ZERO: Self = Self::new(0.0, 0.0, 0.0, 0.0);`
  - L34: `pub fn right(&self) -> f32 {`
  - L39: `pub fn bottom(&self) -> f32 {`
  - L44: `pub fn center(&self) -> (f32, f32) {`
  - L49: `pub fn contains(&self, x: f32, y: f32) -> bool {`
  - L54: `pub fn intersects(&self, other: &Self) -> bool {`
  - L62: `pub fn inflate(&self, d: f32) -> Self {`
  - L67: `pub fn offset(&self, dx: f32, dy: f32) -> Self {`
  - L72: `pub fn union(&self, other: &Self) -> Self {`
  - L90: `pub fn intersection(&self, other: &Self) -> Option<Self> {`
  - L103: `pub fn constrained_to(&self, bounds: &Self) -> Self {`
  - L112: `pub fn is_empty(&self) -> bool {`
  - L124: `pub struct Chrome {`
  - L155: `pub fn button_rect(&self, frame: &Rect, n: u32) -> Rect {`
  - L164: `pub fn title_rect(&self, frame: &Rect) -> Rect {`
  - L170: `pub fn content_rect(&self, frame: &Rect) -> Rect {`
- Test/behavior coverage: 2 named functions in source, primarily module unit tests; representative names: `containment_excludes_the_far_edges`, `a_window_shorter_than_its_title_bar_does_not_produce_negative_content`.

### `src/litric-wm/src/lib.rs`

- Inventory: `F0794`; project source / rust-source; 1608 bytes; `Rust`.
- Size check: 39 lines; SHA-256 `2bf23ff7cf0f1239ee227363e1f480d40c82f2301c0254c57f6b41562ecf58a7`.
- Source-level purpose: LitricOS window manager. **Layer 5 (Native ecosystem).** It sits between input and rendering and touches neither: no devices, no framebuffer, no clock, no I/O of any kind. It is told where the pointer is and what happened, and it decides what that means for the windows. input  ──►  WindowManager  ──►  scene  ──►  compositor  ──►  renderer │ take_damage() ──► only these rectangles are repainted ADR-0009 puts the compositor and 
- Public API / key implementation declarations (6):
  - L33: `pub mod geom;`
  - L34: `pub mod manager;`
  - L35: `pub mod window;`
  - L37: `pub use geom::{Chrome, Rect};`
  - L38: `pub use manager::{Hit, HitZone, ResizeEdge, WindowManager};`
  - L39: `pub use window::{Content, SnapEdge, Window, WindowId, WindowState};`
- Test/behavior coverage: 0 named functions in source, primarily module unit tests; representative names: no inline `#[test]` functions detected.

### `src/litric-wm/src/manager.rs`

- Inventory: `F0795`; project source / rust-source; 45062 bytes; `Rust`.
- Size check: 1311 lines; SHA-256 `fc679a1e2038b5e05cb25a883ee7b3e8545497225c2008d8d89b2ea1324c5d1c`.
- Source-level purpose: The window manager: z-order, focus, workspaces and interaction. No rendering, no input devices, no clock. It is handed positions and told what happened, and it decides what that means. Everything below is therefore reachable from a test, including the parts that are normally only reachable by a person with a mouse: dragging, resizing against a minimum size, snapping to an edge, and what happens to focus when the focused window
- Public API / key implementation declarations (46):
  - L22: `pub enum HitZone {`
  - L40: `pub enum ResizeEdge {`
  - L76: `pub const fn cursor(self) -> &'static str {`
  - L88: `pub struct Hit {`
  - L97: `enum DragKind {`
  - L103: `struct Drag {`
  - L113: `pub struct WindowManager {`
  - L137: `pub fn new(width: f32, height: f32, top_inset: f32) -> Self {`
  - L163: `pub const fn chrome(&self) -> &Chrome {`
  - L169: `pub const fn screen(&self) -> Rect {`
  - L175: `pub const fn work_area(&self) -> Rect {`
  - L180: `pub fn set_display(&mut self, width: f32, height: f32, top_inset: f32) {`
  - L207: `pub fn workspaces(&self) -> &[String] {`
  - L213: `pub const fn active_workspace(&self) -> usize {`
  - L219: `pub fn active_workspace_name(&self) -> &str {`
  - L226: `pub fn set_workspace(&mut self, index: usize) -> bool {`
  - L243: `pub fn cycle_workspace(&mut self, forward: bool) -> bool {`
  - L262: `pub fn move_to_workspace(&mut self, id: WindowId, index: usize) -> bool {`
  - L281: `pub fn open(&mut self, app_id: &str, title: &str, size: (f32, f32)) -> WindowId {`
  - L308: `pub fn open_at(&mut self, app_id: &str, title: &str, frame: Rect) -> WindowId {`
  - L321: `pub fn close(&mut self, id: WindowId) -> bool {`
  - L352: `pub fn all(&self) -> &[Window] {`
  - L358: `pub fn window(&self, id: WindowId) -> Option<&Window> {`
  - L364: `pub fn visible(&self) -> Vec<&Window> {`
  - L374: `pub fn switcher_order(&self) -> Vec<&Window> {`
  - L392: `pub const fn focused(&self) -> Option<WindowId> {`
  - L397: `pub fn focus_window(&mut self, id: WindowId) -> bool {`
  - L415: `pub fn raise(&mut self, id: WindowId) -> bool {`
  - L430: `pub fn cycle_focus(&mut self, forward: bool) -> Option<WindowId> {`
  - L472: `pub fn toggle_maximize(&mut self, id: WindowId) -> bool {`
  - L492: `pub fn snap(&mut self, id: WindowId, edge: SnapEdge) -> bool {`
  - L506: `pub fn restore(&mut self, id: WindowId) -> bool {`
  - L520: `pub fn minimize(&mut self, id: WindowId) -> bool {`
  - L542: `pub fn unminimize(&mut self, id: WindowId) -> bool {`
  - L558: `pub fn center(&mut self, id: WindowId) -> bool {`
  - L577: `pub fn hit_test(&self, x: f32, y: f32) -> Option<Hit> {`
  - L653: `pub fn begin_drag(&mut self, hit: Hit, x: f32, y: f32) -> bool {`
  - L676: `pub fn update_drag(&mut self, x: f32, y: f32) -> Option<SnapEdge> {`
  - L791: `pub fn end_drag(&mut self, x: f32, y: f32) -> Option<SnapEdge> {`
  - L814: `pub const fn is_dragging(&self) -> bool {`
  - L819: `pub fn cancel_drag(&mut self) -> bool {`
  - L830: `pub fn damage(&mut self, r: Rect) {`
  - L835: `pub fn take_damage(&mut self) -> Vec<Rect> {`
  - L841: `pub fn is_dirty(&self) -> bool {`
  - L847: `pub fn diagnostics(&self) -> Vec<String> {`
  - L871: `pub fn is_placeholder(&self, id: WindowId) -> bool {`
- Test/behavior coverage: 26 named functions in source, primarily module unit tests; representative names: `the_work_area_starts_below_the_top_bar`, `a_new_window_takes_focus_and_the_top_of_the_stack`, `closing_the_focused_window_moves_focus_to_the_next_most_recent`, `closing_the_last_window_leaves_no_focus_rather_than_a_dangling_one`, `window_ids_are_never_reused`, `maximize_fills_the_work_area_and_restore_puts_it_back`, `snapping_left_then_restoring_returns_to_the_original_size`, `minimizing_hides_a_window_but_keeps_it_in_the_switcher`, `the_title_bar_is_hit_where_it_is_drawn`, `the_close_button_is_hit_where_it_is_drawn`, `corners_resize_and_the_middle_does_not`, `hit_testing_finds_the_topmost_window`, `hit_testing_empty_desktop_finds_nothing`, `a_window_on_another_workspace_is_neither_drawn_nor_hit`, `switching_to_the_workspace_already_showing_changes_nothing`, `dragging_a_title_bar_moves_the_window_by_the_pointer_delta`, `a_drag_can_never_put_the_title_bar_above_the_top_bar`, `dragging_to_the_left_edge_snaps_to_the_left_half`, `resizing_from_the_south_east_grows_the_window`, `resizing_stops_at_the_minimum_size`, `resizing_from_the_west_moves_the_left_edge_and_keeps_the_right_one`, `a_cancelled_drag_puts_the_window_back`, `a_maximized_window_cannot_be_resized_but_can_be_dragged_free`, `every_change_records_damage_and_taking_it_clears_it`.

### `src/litric-wm/src/window.rs`

- Inventory: `F0796`; project source / rust-source; 8633 bytes; `Rust`.
- Size check: 276 lines; SHA-256 `214f54d3469e5b0cd8e5e07201aeff1afedf84857635fada6ca5bbd8471f222b`.
- Source-level purpose: What a window *is*. A window here is an object with an identity, not a rectangle the shell remembered to draw again. That distinction is the whole point of this crate: a rectangle cannot be focused, cannot be raised above another rectangle, cannot be moved to a workspace and cannot tell you whether the thing that owns it is still alive.
- Public API / key implementation declarations (14):
  - L13: `pub type WindowId = u32;`
  - L18: `pub enum SnapEdge {`
  - L36: `pub fn bounds(self, work: Rect) -> Rect {`
  - L51: `pub const fn name(self) -> &'static str {`
  - L66: `pub enum WindowState {`
  - L82: `pub const fn is_visible(self) -> bool {`
  - L91: `pub const fn is_free(self) -> bool {`
  - L97: `pub const fn name(self) -> &'static str {`
  - L116: `pub enum Content {`
  - L133: `pub struct Window {`
  - L163: `pub fn new(id: WindowId, app_id: &str, title: &str, frame: Rect) -> Self {`
  - L183: `pub const fn is_visible(&self) -> bool {`
  - L189: `pub fn summary(&self) -> String {`
  - L210: `const WORK: Rect = Rect::new(0.0, 34.0, 1280.0, 766.0);`
- Test/behavior coverage: 5 named functions in source, primarily module unit tests; representative names: `halves_tile_the_work_area_exactly`, `quarters_tile_the_work_area_exactly`, `a_minimized_window_is_not_visible_but_is_still_a_window`, `only_a_normal_window_can_be_dragged`, `a_new_window_remembers_where_it_started`.

