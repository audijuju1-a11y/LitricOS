# Batch B06: Application & Service Crates

This report indexes all 77 assigned inventory entries. Rust source summaries identify public API and implementation symbols; test fixtures and generated/reference artifacts are catalogued by provenance and purpose.

## `litric-browser` — Layer 6 Application — ADR-0018 Browser and broker

Browser shell model for URL interpretation, per-origin permissions/privacy policy, tabs/history, engine abstraction, JSON/base64/JPEG decoding; optional Unix `litric-browserd` bridges LBP/1 to Chromium CDP. It deliberately reports missing Chromium rather than rendering fake content.

**Security/correctness review:** Security: HTTPS-first navigation, rejects embedded credentials and malformed/confusable hosts, tracker blocking, origin-scoped ask-by-default permissions, bounded JPEG dimensions/pixels and strict JSON parse limits. Browser daemon has a separately gated broker feature; one audited fd-primitive module contains its unsafe.

### `src/litric-browser/Cargo.toml`

- Inventory: `F0627`; configuration / cargo-manifest; 1416 bytes; `TOML`.
- Size check: 40 lines; SHA-256 `e2142e3dc4ebca5dbfadd81f6cd2b706d11db00194973ef36d915ccf24685bb8`.
- Package configuration and dependency surface:
  - L1: `[package]`
  - L2: `name        = "litric-browser"`
  - L3: `description = "Litric Browser: tabs, history, URLs and the privacy policy that governs them"`
  - L4: `version.workspace      = true`
  - L5: `edition.workspace      = true`
  - L6: `rust-version.workspace = true`
  - L7: `license.workspace      = true`
  - L12: `[[bin]]`
  - L13: `name = "litric-browserd"`
  - L14: `path = "src/bin/browserd.rs"`
  - L15: `required-features = ["broker"]`
  - L17: `[lints.rust]`
  - L23: `unsafe_code                 = "deny"`
  - L24: `missing_docs                = "warn"`
  - L25: `unused_must_use             = "deny"`
  - L26: `rust_2018_idioms            = { level = "warn", priority = -1 }`
  - L28: `[lints.clippy]`
  - L29: `unwrap_used          = "deny"`
  - L30: `expect_used          = "deny"`
  - L31: `panic                = "deny"`
  - L32: `indexing_slicing     = "deny"`
  - L33: `integer_division     = "warn"`
  - L35: `[features]`
  - L36: `broker = []`
  - L38: `[dependencies]`
  - L39: `litric-caps = { path = "../litric-caps" }`
  - L40: `litric-core = { path = "../litric-core" }`

### `src/litric-browser/examples/jpeg-bench.rs`

- Inventory: `F0628`; project source / rust-example; 1660 bytes; `Rust`.
- Size check: 55 lines; SHA-256 `292353eb9b9a0e194b1f473cc3a572e8b7ca0cf2b317fd38a36f3d1662509b40`.
- Source-level purpose: Times `litric_browser::jpeg::decode` on one file. `jpeg-bench FILE [ROUNDS]` decodes the file ROUNDS times (default 50) after one untimed warm-up decode and prints the mean and best wall-clock time. Build with `--release` for numbers that mean anything.
- Public API / key implementation declarations (0):
- Test/behavior coverage: 0 named functions in source, primarily module unit tests; representative names: no inline `#[test]` functions detected.

### `src/litric-browser/src/base64.rs`

- Inventory: `F0629`; project source / rust-source; 8705 bytes; `Rust`.
- Size check: 274 lines; SHA-256 `1e618c574ffd1eafb171fbe147839adf4b03962a112e2d77470b103fce62acce`.
- Source-level purpose: Base64 with the RFC 4648 §4 standard alphabet. Chromium delivers screencast frames as base64 text inside JSON. Decoding is strict, following RFC 4648 §3.3 and §3.5: * the length must be a multiple of four; * `=` may appear only as the last one or two characters of the input; * every other character must be in the alphabet (no whitespace, no line breaks, no URL-safe `-`/`_`); * the bits discarded by padding must be zero, so eve
- Public API / key implementation declarations (6):
  - L15: `const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";`
  - L18: `const INVALID: u8 = 0xFF;`
  - L21: `const DECODE: [u8; 256] = {`
  - L39: `pub enum Base64Error {`
  - L78: `pub fn encode(bytes: &[u8]) -> String {`
  - L115: `pub fn decode(text: &[u8]) -> Result<Vec<u8>, Base64Error> {`
- Test/behavior coverage: 5 named functions in source, primarily module unit tests; representative names: `rfc_4648_test_vectors`, `every_byte_value_and_length_round_trips`, `the_whole_alphabet_decodes`, `malformed_input_is_refused`, `nonzero_padding_bits_are_refused`.

### `src/litric-browser/src/bin/browserd.rs`

- Inventory: `F0630`; project source / rust-source; 32128 bytes; `Rust`.
- Size check: 811 lines; SHA-256 `c208426e6ac1fb7e335a3fb867bed211bc73f64862261b3df9e9bff97d17773a`.
- Source-level purpose: litric-browserd — the browser engine broker. ADR-0018: LitricOS integrates a rendering engine, never writes one. The engine is Chromium, and the broker owns Chromium's lifetime. It translates between two protocols: litric-shell ⇄ (LBP/1, Unix socket /run/browserd.sock) ⇄ browserd browserd     ⇄ (CDP, Chromium --remote-debugging-pipe)  ⇄ Chromium One command per line, one reply per command, frames announced by header: → GO <tab
- Public API / key implementation declarations (21):
  - L58: `pub fn pct_decode(v: &str) -> String {`
  - L83: `pub fn pct_encode(s: &str) -> String {`
  - L127: `const DEFAULT_SOCKET: &str = "/run/browserd.sock";`
  - L131: `pub enum Engine {`
  - L158: `const O_CLOEXEC: u32 = 0o2_000_000;`
  - L162: `pub fn pipe_cloexec() -> io::Result<(i32, i32)> {`
  - L175: `pub fn dup2_or_err(fd: i32, target: i32) -> io::Result<()> {`
  - L186: `pub fn close_fd(fd: i32) {`
  - L205: `pub struct ChromiumProc {`
  - L247: `pub fn spawn(binary: &Path) -> io::Result<Self> {`
  - L331: `pub fn call(&mut self, method: &str, params: &str) -> Result<String, String> {`
  - L363: `pub fn load_and_capture(`
  - L443: `pub fn find_chromium() -> Option<PathBuf> {`
  - L466: `pub struct Broker {`
  - L481: `pub fn new() -> Self {`
  - L508: `pub fn missing_reason(&self) -> Option<String> {`
  - L516: `pub fn handle_go(&self, tab: u32, url: &str) -> Result<String, String> {`
  - L550: `pub fn take_frame(&self, tab: u32) -> Option<(Vec<u8>, u32, u32)> {`
  - L559: `pub(crate) fn serve(&self, stream: UnixStream) {`
  - L670: `pub fn run(&self, socket_path: &Path) -> io::Result<()> {`
  - L688: `pub fn socket_path() -> PathBuf {`
- Test/behavior coverage: 4 named functions in source, primarily module unit tests; representative names: `pct_round_trips_and_stays_literal_on_garbage`, `a_broker_without_chromium_refuses_by_name`, `about_pages_answer_without_an_engine`, `lbp_lines_round_trip_through_the_serve_loop`.

### `src/litric-browser/src/engine.rs`

- Inventory: `F0631`; project source / rust-source; 3661 bytes; `Rust`.
- Size check: 110 lines; SHA-256 `d25abb838087fc48633ad3d7b89ea97c4f17c26993af23e1fa5f27c74abb65b7`.
- Source-level purpose: Where a rendering engine plugs in. Nothing behind this trait will ever be written here. A modern web engine is millions of lines, a permanent security commitment, and the single largest attack surface on any desktop. Writing one from scratch would be reckless in a way no amount of care could fix, and pretending to have one would be worse. So the engine is an integration point. What LitricOS owns is everything around it, which 
- Public API / key implementation declarations (3):
  - L28: `pub struct Page {`
  - L42: `pub trait Engine: core::fmt::Debug + Send {`
  - L58: `pub struct NoEngine;`
- Test/behavior coverage: 2 named functions in source, primarily module unit tests; representative names: `the_null_engine_is_honest_about_itself`, `every_navigation_fails_the_same_way_rather_than_sometimes_succeeding`.

### `src/litric-browser/src/jpeg.rs`

- Inventory: `F0632`; project source / rust-source; 52364 bytes; `Rust`.
- Size check: 1480 lines; SHA-256 `932045e0b7033a96ee2ea83af136355a35f17b8a3e9f5831eff08a005bdcf07c`.
- Source-level purpose: Sequential JPEG decoding, written from ITU-T T.81 (09/92). Chromium delivers screencast frames as JPEG, and those bytes are shaped by web content. This decoder exists so the broker can turn them into raw pixels without linking a C library. It implements exactly what a screencast needs and refuses everything else by name: | Supported | Refused with [`JpegError::Unsupported`] | |---|---| | SOF0 baseline and SOF1 extended sequent
- Public API / key implementation declarations (28):
  - L41: `pub const MAX_DIMENSION: u32 = 8192;`
  - L44: `pub const MAX_PIXELS: u32 = 16_777_216;`
  - L48: `pub struct Image {`
  - L60: `pub enum Feature {`
  - L87: `pub enum JpegError {`
  - L139: `const ZIGZAG: [u8; 64] = [`
  - L145: `const SOI: u8 = 0xD8;`
  - L146: `const EOI: u8 = 0xD9;`
  - L147: `const SOS: u8 = 0xDA;`
  - L148: `const DQT: u8 = 0xDB;`
  - L149: `const DNL: u8 = 0xDC;`
  - L150: `const DRI: u8 = 0xDD;`
  - L151: `const DHT: u8 = 0xC4;`
  - L152: `const DAC: u8 = 0xCC;`
  - L153: `const APP14: u8 = 0xEE;`
  - L160: `pub fn decode(data: &[u8]) -> Result<Image, JpegError> {`
  - L238: `struct Decoder {`
  - L247: `struct Frame {`
  - L257: `struct Component {`
  - L677: `struct ScanComponent<'t> {`
  - L685: `const FAST_BITS: u32 = 9;`
  - L688: `struct Huffman {`
  - L749: `struct Bits<'a> {`
  - L984: `struct Idct {`
  - L1046: `struct ColorTables {`
  - L1161: `const MAX_CHANNEL_DIFF: u32 = 3;`
  - L1163: `const MAX_MEAN_DIFF: f64 = 0.25;`
  - L1386: `struct XorShift(u64);`
- Test/behavior coverage: 12 named functions in source, primarily module unit tests; representative names: `huffman_tables`, `output_matches_djpeg_within_the_stated_tolerance`, `refused_encodings_are_named`, `header_level_refusals_are_specific`, `a_scan_without_its_tables_is_refused`, `rgb_encoded_jpegs_are_refused_rather_than_miscoloured`, `every_truncation_is_an_error_and_none_panics`, `a_wrong_restart_marker_is_refused`, `random_bit_flips_never_panic`, `the_zigzag_table_walks_the_antidiagonals`, `a_dc_only_block_is_flat`, `huffman_counts_that_overflow_their_lengths_are_refused`.

### `src/litric-browser/src/json.rs`

- Inventory: `F0633`; project source / rust-source; 34043 bytes; `Rust`.
- Size check: 984 lines; SHA-256 `460b8e8f6a1c2b4782e9233b07e85ea6e650a582a90fd1d3eedd9613c58b18d4`.
- Source-level purpose: Strict JSON (RFC 8259): a parser and a writer. The Chrome DevTools Protocol speaks JSON, and everything Chromium sends is derived from web content an attacker may control (titles, URLs, dialog messages). This parser is therefore strict rather than forgiving: * the input must be UTF-8, with no byte-order mark; * exactly one value, surrounded only by the four JSON whitespace bytes; * no trailing commas, comments, single quotes, 
- Public API / key implementation declarations (22):
  - L27: `pub const MAX_DEPTH: usize = 128;`
  - L34: `pub enum Value {`
  - L53: `pub enum JsonError {`
  - L129: `pub fn parse(input: &[u8]) -> Result<Value, JsonError> {`
  - L147: `struct Parser<'a> {`
  - L441: `const PAIRWISE_LIMIT: usize = 16;`
  - L464: `pub fn object<const N: usize>(members: [(&str, Value); N]) -> Self {`
  - L475: `pub fn get(&self, key: &str) -> Option<&Value> {`
  - L484: `pub fn as_str(&self) -> Option<&str> {`
  - L493: `pub const fn as_f64(&self) -> Option<f64> {`
  - L503: `pub fn as_u64(&self) -> Option<u64> {`
  - L504: `const LIMIT: f64 = 9_007_199_254_740_992.0; // 2^53`
  - L516: `pub fn as_i64(&self) -> Option<i64> {`
  - L517: `const LIMIT: f64 = 9_007_199_254_740_992.0; // 2^53`
  - L529: `pub const fn as_bool(&self) -> Option<bool> {`
  - L538: `pub fn as_array(&self) -> Option<&[Value]> {`
  - L550: `pub fn to_json(&self) -> String {`
  - L557: `pub fn write_to(&self, out: &mut String) {`
  - L639: `pub fn write_string(s: &str, out: &mut String) {`
  - L640: `const HEX: &[u8; 16] = b"0123456789abcdef";`
  - L910: `struct XorShift(u64);`
  - L936: `const SPECIAL: &[u8] = b"{}[]\",:\\/-+.eE0123456789tfnu \t\n\x00\x7f\xc3\xa9\xed\xa0\x80";`
- Test/behavior coverage: 7 named functions in source, primarily module unit tests; representative names: `invalid_documents_are_refused_with_the_right_error`, `nesting_is_limited_to_the_documented_depth`, `a_megabyte_string_parses_and_serialises_in_linear_time`, `the_writer_escapes_exactly_what_must_be_escaped`, `numbers_are_written_inside_the_json_grammar`, `a_document_round_trips_through_the_writer`, `randomized_mutations_never_panic_and_are_deterministic`.

### `src/litric-browser/src/lib.rs`

- Inventory: `F0634`; project source / rust-source; 11994 bytes; `Rust`.
- Size check: 389 lines; SHA-256 `00721f3035d3389b3b4b9eff4f5a575e99c05f8b0ce7c1add23136878e15c383`.
- Source-level purpose: Litric Browser. **Layer 6 (Applications).** It sits on `litric-caps` for permissions and on the shell for its window, and it decides what a browser *is* on this system: how an address is read, what a site is allowed to do, and what happens when it is refused. **There is no rendering engine here, and there is never going to be one.** A modern web engine is millions of lines and a permanent, full-time security commitment; writin
- Public API / key implementation declarations (24):
  - L42: `pub mod base64;`
  - L43: `pub mod engine;`
  - L44: `pub mod jpeg;`
  - L45: `pub mod json;`
  - L46: `pub mod policy;`
  - L47: `pub mod tab;`
  - L48: `pub mod url;`
  - L50: `pub use policy::{Decision, Policy, PrivacyPolicy, Refusal, SitePermission};`
  - L51: `pub use tab::{LoadState, Tab, TabId};`
  - L52: `pub use url::{interpret, Intent, Scheme, Url, UrlError};`
  - L56: `pub struct Session {`
  - L69: `pub const REOPEN_CAPACITY: usize = 16;`
  - L73: `pub fn new(policy: Policy, engine: Box<dyn engine::Engine>) -> Self {`
  - L85: `pub fn open_tab(&mut self) -> TabId {`
  - L95: `pub fn tabs(&self) -> &[Tab] {`
  - L101: `pub fn len(&self) -> usize {`
  - L107: `pub fn is_empty(&self) -> bool {`
  - L113: `pub fn active(&self) -> Option<&Tab> {`
  - L118: `pub fn focus(&mut self, id: TabId) -> bool {`
  - L129: `pub fn close(&mut self, id: TabId) -> bool {`
  - L145: `pub fn reopen(&mut self) -> Option<TabId> {`
  - L157: `pub fn go(&mut self, id: TabId, input: &str) -> Outcome {`
  - L204: `pub fn lines(&self) -> Vec<String> {`
  - L218: `pub enum Outcome {`
- Test/behavior coverage: 9 named functions in source, primarily module unit tests; representative names: `a_session_starts_empty_and_opens_tabs`, `closing_the_active_tab_moves_focus_and_does_not_leave_it_dangling`, `a_closed_tab_can_be_reopened_most_recent_first`, `without_an_engine_a_navigation_fails_and_names_what_is_missing`, `plain_http_is_refused_by_policy_before_the_engine_is_asked`, `a_tracker_is_refused`, `typing_words_reports_that_there_is_no_search_provider`, `navigating_a_tab_that_is_gone_is_reported_not_ignored`, `the_reopen_list_is_bounded`.

### `src/litric-browser/src/policy.rs`

- Inventory: `F0635`; project source / rust-source; 19942 bytes; `Rust`.
- Size check: 600 lines; SHA-256 `9db2a095c773135dbcf0e0463e10577115a30675ee7f0666fb2e0cb7e4830fc1`.
- Source-level purpose: What a site is allowed to do, and who decides. A browser is the largest attack surface on a desktop and the one place where a user routinely runs code written by strangers. LitricOS already has a capability model for exactly this problem, so the browser does not invent a second one: a site asking for the camera is an application asking for the camera, and it goes through [`litric_caps`] like everything else. Every permission b
- Public API / key implementation declarations (25):
  - L30: `pub enum SitePermission {`
  - L55: `pub const ALL: [Self; 9] = [`
  - L69: `pub const fn title(self) -> &'static str {`
  - L90: `pub const fn capability(self) -> Option<CapabilityId> {`
  - L108: `pub const fn needs_prompt(self) -> bool {`
  - L121: `pub const fn allowed_on_insecure(self) -> bool {`
  - L129: `pub enum Decision {`
  - L140: `pub struct PrivacyPolicy {`
  - L177: `pub enum Refusal {`
  - L198: `pub fn message(&self) -> String {`
  - L217: `pub struct SiteSettings {`
  - L227: `pub fn new(origin: &str) -> Self {`
  - L236: `pub fn decision(&self, p: SitePermission) -> Decision {`
  - L244: `pub fn set(&mut self, p: SitePermission, d: Decision) {`
  - L254: `pub fn decided(&self) -> &[(SitePermission, Decision)] {`
  - L261: `pub struct Policy {`
  - L271: `pub fn new(trackers: Vec<String>) -> Self {`
  - L284: `pub fn is_tracker(&self, host: &str) -> bool {`
  - L296: `pub fn check_navigation(&self, url: &Url) -> Result<(), Refusal> {`
  - L319: `pub fn permission(&self, url: &Url, p: SitePermission) -> Decision {`
  - L330: `pub fn decide(&mut self, url: &Url, p: SitePermission, d: Decision) {`
  - L343: `pub fn site(&self, origin: &str) -> Option<&SiteSettings> {`
  - L349: `pub fn sites(&self) -> &[SiteSettings] {`
  - L354: `pub fn forget(&mut self, origin: &str) -> bool {`
  - L362: `pub fn lines(&self) -> Vec<String> {`
- Test/behavior coverage: 13 named functions in source, primarily module unit tests; representative names: `every_permission_starts_at_ask_and_nothing_is_granted_by_default`, `a_decision_applies_to_the_whole_origin_not_the_page`, `a_different_scheme_is_a_different_origin`, `an_insecure_origin_can_never_hold_a_sensitive_permission`, `https_only_refuses_http_and_offers_the_upgrade`, `upgrading_drops_the_old_default_port`, `a_non_default_port_survives_the_upgrade`, `the_defaults_are_the_strict_ones`, `permissions_that_read_the_world_map_to_system_capabilities`, `only_the_physical_world_permissions_prompt`, `forgetting_a_site_removes_everything_about_it`, `changing_a_decision_replaces_it_rather_than_stacking`, `status_lines_report_the_policy_in_force`.

### `src/litric-browser/src/tab.rs`

- Inventory: `F0636`; project source / rust-source; 11873 bytes; `Rust`.
- Size check: 397 lines; SHA-256 `000f3c06797ac9a8e8897e88d1f80fff06be458e54e5c9aa4875d83055b52d99`.
- Source-level purpose: Tabs, history and what a page is doing. The model a browser window shows. It holds no engine and performs no navigation: it records what was asked for and what came back, which is exactly the part that has to be right before anything is rendering. One list with a cursor, not two stacks. The two-stack version is the classic way to write this and it is subtly wrong: navigating somewhere new while part-way back through history ha
- Public API / key implementation declarations (25):
  - L18: `pub type TabId = u32;`
  - L23: `pub enum LoadState {`
  - L48: `pub const fn is_busy(&self) -> bool {`
  - L54: `pub const fn token(&self) -> &'static str {`
  - L67: `pub struct History {`
  - L76: `pub fn len(&self) -> usize {`
  - L82: `pub fn is_empty(&self) -> bool {`
  - L88: `pub fn current(&self) -> Option<&Url> {`
  - L94: `pub fn entries(&self) -> &[Url] {`
  - L100: `pub const fn index(&self) -> usize {`
  - L108: `pub fn push(&mut self, url: Url) {`
  - L126: `pub fn can_go_back(&self) -> bool {`
  - L132: `pub fn can_go_forward(&self) -> bool {`
  - L137: `pub fn back(&mut self) -> Option<&Url> {`
  - L146: `pub fn forward(&mut self) -> Option<&Url> {`
  - L157: `pub struct Tab {`
  - L178: `pub fn new(id: TabId) -> Self {`
  - L192: `pub fn url(&self) -> Option<&Url> {`
  - L201: `pub fn label(&self) -> String {`
  - L216: `pub fn navigate(&mut self, url: Url) {`
  - L227: `pub fn blocked(&mut self, url: Url, reason: &str) {`
  - L236: `pub fn failed(&mut self, reason: &str) {`
  - L243: `pub fn loaded(&mut self, title: Option<&str>) {`
  - L251: `pub fn back(&mut self) -> Option<Url> {`
  - L259: `pub fn forward(&mut self) -> Option<Url> {`
- Test/behavior coverage: 9 named functions in source, primarily module unit tests; representative names: `a_new_tab_is_empty_and_named`, `back_and_forward_walk_the_history`, `navigating_while_part_way_back_discards_the_forward_entries`, `reloading_the_same_address_does_not_grow_the_history`, `a_blocked_page_still_enters_history`, `a_failed_tab_says_which_site_failed`, `a_page_with_no_title_is_labelled_by_its_host`, `history_of_a_single_page_has_no_back_or_forward`, `an_empty_history_is_safe_to_walk`.

### `src/litric-browser/src/url.rs`

- Inventory: `F0637`; project source / rust-source; 19327 bytes; `Rust`.
- Size check: 597 lines; SHA-256 `b837ea6563cc515098936af9cb906b4a09d1adcbc76d6232a66b20c451186b1b`.
- Source-level purpose: URLs, and working out what somebody meant when they typed into the address bar. The address bar is the most security-sensitive text field on a desktop. It is where a user decides whether they are talking to their bank, and it is where every trick for making one site look like another gets played. So the parsing here is deliberately strict and the *display* is deliberately careful: * A host containing non-ASCII is rendered in P
- Public API / key implementation declarations (17):
  - L23: `pub enum UrlError {`
  - L68: `pub enum Scheme {`
  - L82: `pub fn parse(s: &str) -> Option<Self> {`
  - L94: `pub const fn name(self) -> &'static str {`
  - L105: `pub const fn is_secure(self) -> bool {`
  - L111: `pub const fn needs_host(self) -> bool {`
  - L117: `pub const fn default_port(self) -> Option<u16> {`
  - L128: `pub struct Url {`
  - L148: `pub fn parse(raw: &str) -> Result<Self, UrlError> {`
  - L228: `pub fn origin(&self) -> String {`
  - L237: `pub const fn is_secure(&self) -> bool {`
  - L243: `pub fn to_url(&self) -> String {`
  - L273: `pub fn display_host(&self) -> String {`
  - L285: `pub fn tab_label(&self) -> String {`
  - L332: `pub fn is_confusable_host(host: &str) -> bool {`
  - L342: `pub enum Intent {`
  - L357: `pub fn interpret(input: &str) -> Intent {`
- Test/behavior coverage: 18 named functions in source, primarily module unit tests; representative names: `a_plain_https_url_parses`, `a_url_with_no_path_gets_a_root_one`, `the_host_is_lowercased_but_the_path_is_not`, `a_port_is_parsed_and_kept_in_the_origin`, `a_bad_port_is_refused`, `embedded_credentials_are_refused_outright`, `a_nonsense_host_is_refused`, `an_unsupported_scheme_says_which_one`, `the_origin_is_the_security_boundary_not_the_path`, `a_mixed_script_host_is_flagged_rather_than_rendered_convincingly`, `a_whole_script_host_is_not_flagged`, `typing_a_host_navigates_and_typing_words_searches`, `a_bare_host_is_assumed_to_be_https`, `something_that_looks_like_a_scheme_but_is_not_supported_is_searched`, `an_empty_address_bar_searches_for_nothing_rather_than_erroring`, `a_tab_label_is_the_host_for_a_site_and_the_file_name_for_a_file`, `a_scheme_with_no_authority_keeps_its_whole_body_as_the_path`, `every_parsed_url_round_trips_through_its_text`.

### `src/litric-browser/testdata/jpeg/big-gray-q85.jpg`

- Inventory: `F0638`; binary/asset / test-fixture; 1044 bytes; `Binary (JPEG image)`.
- Purpose: JPEG decoder input fixture; paired `.ref.pnm` files are expected reference pixels. Reject fixtures exercise unsupported JPEG coding modes.

### `src/litric-browser/testdata/jpeg/big-gray-q85.ref.pnm`

- Inventory: `F0639`; binary/asset / test-fixture; 3085 bytes; `Binary (PNM image)`.
- Purpose: Reference PNM pixel output used to compare the in-tree JPEG decoder against fixture generation.

### `src/litric-browser/testdata/jpeg/big-s420-q85.jpg`

- Inventory: `F0640`; binary/asset / test-fixture; 1832 bytes; `Binary (JPEG image)`.
- Purpose: JPEG decoder input fixture; paired `.ref.pnm` files are expected reference pixels. Reject fixtures exercise unsupported JPEG coding modes.

### `src/litric-browser/testdata/jpeg/big-s420-q85.ref.pnm`

- Inventory: `F0641`; binary/asset / test-fixture; 9229 bytes; `Binary (PNM image)`.
- Purpose: Reference PNM pixel output used to compare the in-tree JPEG decoder against fixture generation.

### `src/litric-browser/testdata/jpeg/big-s444-q85.jpg`

- Inventory: `F0642`; binary/asset / test-fixture; 2764 bytes; `Binary (JPEG image)`.
- Purpose: JPEG decoder input fixture; paired `.ref.pnm` files are expected reference pixels. Reject fixtures exercise unsupported JPEG coding modes.

### `src/litric-browser/testdata/jpeg/big-s444-q85.ref.pnm`

- Inventory: `F0643`; binary/asset / test-fixture; 9229 bytes; `Binary (PNM image)`.
- Purpose: Reference PNM pixel output used to compare the in-tree JPEG decoder against fixture generation.

### `src/litric-browser/testdata/jpeg/gen.sh`

- Inventory: `F0644`; other / misc; 2936 bytes; `Bash/POSIX Shell`.
- Size check: 79 lines; SHA-256 `29b24dc7de4491172806fb95d28ae992f914d51f3a356189c56063e66131a076`.
- Script: test fixture generation/helper; inspect script before execution, no generation was run during this audit.

### `src/litric-browser/testdata/jpeg/gray-q75.jpg`

- Inventory: `F0645`; binary/asset / test-fixture; 584 bytes; `Binary (JPEG image)`.
- Purpose: JPEG decoder input fixture; paired `.ref.pnm` files are expected reference pixels. Reject fixtures exercise unsupported JPEG coding modes.

### `src/litric-browser/testdata/jpeg/gray-q75.ref.pnm`

- Inventory: `F0646`; binary/asset / test-fixture; 1086 bytes; `Binary (PNM image)`.
- Purpose: Reference PNM pixel output used to compare the in-tree JPEG decoder against fixture generation.

### `src/litric-browser/testdata/jpeg/reject-12bit.jpg`

- Inventory: `F0647`; binary/asset / test-fixture; 1406 bytes; `Binary (JPEG image)`.
- Purpose: JPEG decoder input fixture; paired `.ref.pnm` files are expected reference pixels. Reject fixtures exercise unsupported JPEG coding modes.

### `src/litric-browser/testdata/jpeg/reject-arithmetic.jpg`

- Inventory: `F0648`; binary/asset / test-fixture; 549 bytes; `Binary (JPEG image)`.
- Purpose: JPEG decoder input fixture; paired `.ref.pnm` files are expected reference pixels. Reject fixtures exercise unsupported JPEG coding modes.

### `src/litric-browser/testdata/jpeg/reject-lossless.jpg`

- Inventory: `F0649`; binary/asset / test-fixture; 1349 bytes; `Binary (JPEG image)`.
- Purpose: JPEG decoder input fixture; paired `.ref.pnm` files are expected reference pixels. Reject fixtures exercise unsupported JPEG coding modes.

### `src/litric-browser/testdata/jpeg/reject-progressive.jpg`

- Inventory: `F0650`; binary/asset / test-fixture; 939 bytes; `Binary (JPEG image)`.
- Purpose: JPEG decoder input fixture; paired `.ref.pnm` files are expected reference pixels. Reject fixtures exercise unsupported JPEG coding modes.

### `src/litric-browser/testdata/jpeg/s420-3scans.jpg`

- Inventory: `F0651`; binary/asset / test-fixture; 1098 bytes; `Binary (JPEG image)`.
- Purpose: JPEG decoder input fixture; paired `.ref.pnm` files are expected reference pixels. Reject fixtures exercise unsupported JPEG coding modes.

### `src/litric-browser/testdata/jpeg/s420-3scans.ref.pnm`

- Inventory: `F0652`; binary/asset / test-fixture; 3232 bytes; `Binary (PNM image)`.
- Purpose: Reference PNM pixel output used to compare the in-tree JPEG decoder against fixture generation.

### `src/litric-browser/testdata/jpeg/s420-optimize.jpg`

- Inventory: `F0653`; binary/asset / test-fixture; 710 bytes; `Binary (JPEG image)`.
- Purpose: JPEG decoder input fixture; paired `.ref.pnm` files are expected reference pixels. Reject fixtures exercise unsupported JPEG coding modes.

### `src/litric-browser/testdata/jpeg/s420-optimize.ref.pnm`

- Inventory: `F0654`; binary/asset / test-fixture; 3232 bytes; `Binary (PNM image)`.
- Purpose: Reference PNM pixel output used to compare the in-tree JPEG decoder against fixture generation.

### `src/litric-browser/testdata/jpeg/s420-q10.jpg`

- Inventory: `F0655`; binary/asset / test-fixture; 839 bytes; `Binary (JPEG image)`.
- Purpose: JPEG decoder input fixture; paired `.ref.pnm` files are expected reference pixels. Reject fixtures exercise unsupported JPEG coding modes.

### `src/litric-browser/testdata/jpeg/s420-q10.ref.pnm`

- Inventory: `F0656`; binary/asset / test-fixture; 3232 bytes; `Binary (PNM image)`.
- Purpose: Reference PNM pixel output used to compare the in-tree JPEG decoder against fixture generation.

### `src/litric-browser/testdata/jpeg/s420-q2-sof1.jpg`

- Inventory: `F0657`; binary/asset / test-fixture; 782 bytes; `Binary (JPEG image)`.
- Purpose: JPEG decoder input fixture; paired `.ref.pnm` files are expected reference pixels. Reject fixtures exercise unsupported JPEG coding modes.

### `src/litric-browser/testdata/jpeg/s420-q2-sof1.ref.pnm`

- Inventory: `F0658`; binary/asset / test-fixture; 3232 bytes; `Binary (PNM image)`.
- Purpose: Reference PNM pixel output used to compare the in-tree JPEG decoder against fixture generation.

### `src/litric-browser/testdata/jpeg/s420-q75.jpg`

- Inventory: `F0659`; binary/asset / test-fixture; 1089 bytes; `Binary (JPEG image)`.
- Purpose: JPEG decoder input fixture; paired `.ref.pnm` files are expected reference pixels. Reject fixtures exercise unsupported JPEG coding modes.

### `src/litric-browser/testdata/jpeg/s420-q75.ref.pnm`

- Inventory: `F0660`; binary/asset / test-fixture; 3232 bytes; `Binary (PNM image)`.
- Purpose: Reference PNM pixel output used to compare the in-tree JPEG decoder against fixture generation.

### `src/litric-browser/testdata/jpeg/s420-q95.jpg`

- Inventory: `F0661`; binary/asset / test-fixture; 1576 bytes; `Binary (JPEG image)`.
- Purpose: JPEG decoder input fixture; paired `.ref.pnm` files are expected reference pixels. Reject fixtures exercise unsupported JPEG coding modes.

### `src/litric-browser/testdata/jpeg/s420-q95.ref.pnm`

- Inventory: `F0662`; binary/asset / test-fixture; 3232 bytes; `Binary (PNM image)`.
- Purpose: Reference PNM pixel output used to compare the in-tree JPEG decoder against fixture generation.

### `src/litric-browser/testdata/jpeg/s420-restart1.jpg`

- Inventory: `F0663`; binary/asset / test-fixture; 1098 bytes; `Binary (JPEG image)`.
- Purpose: JPEG decoder input fixture; paired `.ref.pnm` files are expected reference pixels. Reject fixtures exercise unsupported JPEG coding modes.

### `src/litric-browser/testdata/jpeg/s420-restart1.ref.pnm`

- Inventory: `F0664`; binary/asset / test-fixture; 3232 bytes; `Binary (PNM image)`.
- Purpose: Reference PNM pixel output used to compare the in-tree JPEG decoder against fixture generation.

### `src/litric-browser/testdata/jpeg/s420-restart3b.jpg`

- Inventory: `F0665`; binary/asset / test-fixture; 1098 bytes; `Binary (JPEG image)`.
- Purpose: JPEG decoder input fixture; paired `.ref.pnm` files are expected reference pixels. Reject fixtures exercise unsupported JPEG coding modes.

### `src/litric-browser/testdata/jpeg/s420-restart3b.ref.pnm`

- Inventory: `F0666`; binary/asset / test-fixture; 3232 bytes; `Binary (PNM image)`.
- Purpose: Reference PNM pixel output used to compare the in-tree JPEG decoder against fixture generation.

### `src/litric-browser/testdata/jpeg/s422-q75.jpg`

- Inventory: `F0667`; binary/asset / test-fixture; 1222 bytes; `Binary (JPEG image)`.
- Purpose: JPEG decoder input fixture; paired `.ref.pnm` files are expected reference pixels. Reject fixtures exercise unsupported JPEG coding modes.

### `src/litric-browser/testdata/jpeg/s422-q75.ref.pnm`

- Inventory: `F0668`; binary/asset / test-fixture; 3232 bytes; `Binary (PNM image)`.
- Purpose: Reference PNM pixel output used to compare the in-tree JPEG decoder against fixture generation.

### `src/litric-browser/testdata/jpeg/s440-q75.jpg`

- Inventory: `F0669`; binary/asset / test-fixture; 1166 bytes; `Binary (JPEG image)`.
- Purpose: JPEG decoder input fixture; paired `.ref.pnm` files are expected reference pixels. Reject fixtures exercise unsupported JPEG coding modes.

### `src/litric-browser/testdata/jpeg/s440-q75.ref.pnm`

- Inventory: `F0670`; binary/asset / test-fixture; 3232 bytes; `Binary (PNM image)`.
- Purpose: Reference PNM pixel output used to compare the in-tree JPEG decoder against fixture generation.

### `src/litric-browser/testdata/jpeg/s444-3scans.jpg`

- Inventory: `F0671`; binary/asset / test-fixture; 1362 bytes; `Binary (JPEG image)`.
- Purpose: JPEG decoder input fixture; paired `.ref.pnm` files are expected reference pixels. Reject fixtures exercise unsupported JPEG coding modes.

### `src/litric-browser/testdata/jpeg/s444-3scans.ref.pnm`

- Inventory: `F0672`; binary/asset / test-fixture; 3232 bytes; `Binary (PNM image)`.
- Purpose: Reference PNM pixel output used to compare the in-tree JPEG decoder against fixture generation.

### `src/litric-browser/testdata/jpeg/s444-q75.jpg`

- Inventory: `F0673`; binary/asset / test-fixture; 1346 bytes; `Binary (JPEG image)`.
- Purpose: JPEG decoder input fixture; paired `.ref.pnm` files are expected reference pixels. Reject fixtures exercise unsupported JPEG coding modes.

### `src/litric-browser/testdata/jpeg/s444-q75.ref.pnm`

- Inventory: `F0674`; binary/asset / test-fixture; 3232 bytes; `Binary (PNM image)`.
- Purpose: Reference PNM pixel output used to compare the in-tree JPEG decoder against fixture generation.

## `litric-defender` — Layer 3 Security Policy Core — ADR-0022 Phase 0

Pure detection decision engine: streaming in-crate SHA-256, hash denylist, bounded LRU verdict cache, Aho–Corasick-style byte pattern matching, scan observations and strict/permissive verdict policy. Zero third-party dependencies and no I/O/process execution.

**Security/correctness review:** Security: `#![forbid(unsafe_code)]`, denies panic/unwrap/indexing lints, bounds pattern length and cache, fails pattern reload atomically, and separates malicious matches from unknown/unscannable handling.

### `src/litric-defender/Cargo.toml`

- Inventory: `F0685`; configuration / cargo-manifest; 593 bytes; `TOML`.
- Size check: 15 lines; SHA-256 `587a279dae67736c89efa26bf35958a3691c39a72ffc317132412ac1c39f383f`.
- Package configuration and dependency surface:
  - L1: `[package]`
  - L2: `name        = "litric-defender"`
  - L3: `description = "LitricOS Defender policy core: hash verdicts, byte patterns, verdict cache, fail-open policy and quarantine planning (ADR-0022 Phase 0)"`
  - L4: `version.workspace      = true`
  - L5: `edition.workspace      = true`
  - L6: `rust-version.workspace = true`
  - L7: `license.workspace      = true`
  - L9: `[lints]`
  - L10: `workspace = true`
  - L12: `[dependencies]`

### `src/litric-defender/src/cache.rs`

- Inventory: `F0686`; project source / rust-source; 9251 bytes; `Rust`.
- Size check: 257 lines; SHA-256 `08a5b9bac62bc784b6ba71e1ac54761a3e8316499e01a84891ce319f05167bf4`.
- Source-level purpose: Verdict cache: a bounded LRU keyed by (path, size, mtime). Purpose (ADR-0022 §5): an on-access scanner sits on the critical path of every file open on the machine. Re-scanning an unchanged file on every open would tax the whole OS for zero security gain. The cache maps the identity of a file *version* to the last verdict, and drops the oldest entries under pressure so memory is bounded by construction. Key design (ADR-0022 §5)
- Public API / key implementation declarations (15):
  - L28: `pub const DEFAULT_CAPACITY: usize = 65_536;`
  - L32: `pub enum CacheMiss {`
  - L43: `pub enum Verdict {`
  - L52: `pub struct Key {`
  - L62: `pub struct Generation(u64);`
  - L66: `pub struct VerdictCache {`
  - L78: `pub fn new() -> Self {`
  - L84: `pub fn with_capacity(capacity: usize) -> Self {`
  - L95: `pub fn generation(&self) -> Generation {`
  - L101: `pub fn bump_generation(&mut self) {`
  - L109: `pub fn lookup(&mut self, path: &str, size: u64, mtime: u64) -> Result<Verdict, CacheMiss> {`
  - L127: `pub fn store(&mut self, path: &str, size: u64, mtime: u64, verdict: Verdict) {`
  - L159: `pub fn len(&self) -> usize {`
  - L165: `pub fn is_empty(&self) -> bool {`
  - L171: `pub fn clear(&mut self) {`
- Test/behavior coverage: 2 named functions in source, primarily module unit tests; representative names: `hit_and_miss_roundtrip`, `generation_bump_stales_everything`.

### `src/litric-defender/src/denylist.rs`

- Inventory: `F0687`; project source / rust-source; 6379 bytes; `Rust`.
- Size check: 178 lines; SHA-256 `9f5dbef191a3a4027a938db131320aa45ad088475c6c4a6b2c70e6b62df4af1d`.
- Source-level purpose: SHA-256 denylist: a sorted, immutable hash table searched by binary search — the fastest detection tier and the only one with no false positives (ADR-0022 §4). The on-disk format is deliberately line-oriented text so it can be diffed, audited, hand-edited and reviewed without tooling: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  empty-file ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  t
- Public API / key implementation declarations (12):
  - L23: `pub struct Denylist {`
  - L35: `pub fn new() -> Self {`
  - L42: `pub fn parse(text: &str) -> Self {`
  - L75: `pub fn insert(&mut self, digest_hex: &str, label: &str) -> bool {`
  - L85: `pub fn lookup_hex(&self, digest_hex: &str) -> Option<&str> {`
  - L92: `pub fn lookup(&self, digest: &[u8; 32]) -> Option<&str> {`
  - L98: `pub fn len(&self) -> usize {`
  - L105: `pub fn is_empty(&self) -> bool {`
  - L111: `pub fn rejected(&self) -> &[String] {`
  - L118: `pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {`
  - L127: `const EMPTY: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";`
  - L128: `const ABC: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";`
- Test/behavior coverage: 3 named functions in source, primarily module unit tests; representative names: `parses_comments_blanks_and_labels`, `rejects_malformed_lines_without_panicking`, `insert_rejects_short_hashes`.

### `src/litric-defender/src/engine.rs`

- Inventory: `F0688`; project source / rust-source; 10800 bytes; `Rust`.
- Size check: 309 lines; SHA-256 `454721cf573d99130bea0366fe9eca6b567989334820e61445f9b45c57868727`.
- Source-level purpose: The engine: ties denylist, patterns, cache and policy into one scan entry point, with the evidence record the audit log requires (ADR-0022 §10). Pure and synchronous: the engine takes bytes and metadata, returns a decision and the observations behind it. Transport (fanotify, kernel hook) is a later phase's problem; this stays testable on any host.
- Public API / key implementation declarations (9):
  - L17: `pub struct Engine {`
  - L26: `pub struct ScanReport {`
  - L41: `pub fn new() -> Self {`
  - L51: `pub fn set_mode(&mut self, mode: Mode) {`
  - L57: `pub fn load_denylist(&mut self, list: Denylist) {`
  - L68: `pub fn load_patterns<I, P, S>(&mut self, patterns: I) -> Result<(), crate::patterns::BuildError>`
  - L84: `pub fn scan(&mut self, path: &str, size: u64, mtime: u64, content: &[u8]) -> ScanReport {`
  - L162: `pub fn cache_len(&self) -> usize {`
  - L177: `const EICAR: &[u8] = b"X5O!P%@AP[4\\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*";`
- Test/behavior coverage: 2 named functions in source, primarily module unit tests; representative names: `empty_engine_allows_everything`, `clean_files_cache_as_clean`.

### `src/litric-defender/src/lib.rs`

- Inventory: `F0689`; project source / rust-source; 966 bytes; `Rust`.
- Size check: 28 lines; SHA-256 `1555a726151e6d5a91d9779d747413db01dd7ae43177310437ea5b8a680f143b`.
- Source-level purpose: LitricOS Defender — policy core (ADR-0022 Phase 0). This crate is the detection brain without the body: hash verdicts, byte-pattern matching, a bounded verdict cache and the fail-open policy ladder. It contains **no I/O, no process execution and no third-party dependencies** — by the same supply-chain rule as the other core crates, because this code decides what the machine may execute. Phase roadmap (ADR-0022): Phase 0 is thi
- Public API / key implementation declarations (6):
  - L23: `pub mod cache;`
  - L24: `pub mod denylist;`
  - L25: `pub mod engine;`
  - L26: `pub mod patterns;`
  - L27: `pub mod sha256;`
  - L28: `pub mod verdict;`
- Test/behavior coverage: 0 named functions in source, primarily module unit tests; representative names: no inline `#[test]` functions detected.

### `src/litric-defender/src/patterns.rs`

- Inventory: `F0690`; project source / rust-source; 14043 bytes; `Rust`.
- Size check: 385 lines; SHA-256 `93f68e45eb7c21ad8bf019f15b97e6727ef7a531091b91a8db456ff866c50831`.
- Source-level purpose: Aho–Corasick multi-pattern matcher over byte-signature patterns (ADR-0022 §4, detection tier 3). One pass over the scanned bytes reports every pattern that matches, regardless of how many patterns are loaded. The trie is built eagerly with failure links (BFS) at construction, so matching is linear in the input size plus the emitted matches. Memory model: per-node transitions use a hash map rather than a dense 256-way table. Fo
- Public API / key implementation declarations (11):
  - L25: `pub const MAX_PATTERN_LEN: usize = 4096;`
  - L29: `pub enum BuildError {`
  - L64: `pub struct Match {`
  - L74: `struct Node {`
  - L86: `pub struct Matcher {`
  - L101: `pub fn build<I, P, S>(patterns: I) -> Result<Self, BuildError>`
  - L199: `pub fn len(&self) -> usize {`
  - L205: `pub fn is_empty(&self) -> bool {`
  - L211: `pub fn id(&self, index: usize) -> Option<&str> {`
  - L218: `pub fn find(&self, haystack: &[u8]) -> Vec<Match> {`
  - L327: `const EICAR: &[u8] =`
- Test/behavior coverage: 3 named functions in source, primarily module unit tests; representative names: `finds_eicar_in_noise`, `clean_input_yields_nothing`, `build_rejects_degenerate_patterns`.

### `src/litric-defender/src/sha256.rs`

- Inventory: `F0691`; project source / rust-source; 11213 bytes; `Rust`.
- Size check: 295 lines; SHA-256 `26b15684cee13d514723b67e9927794bb84a6fa175fe3b455ad91ae8a9423c4a`.
- Source-level purpose: SHA-256 (FIPS 180-4), implemented here because ADR-0022 §7 forbids third-party dependencies in the policy core. The streaming [`Sha256`] type allows hashing without owning the whole file; the Defender reads chunks into a fixed 8 KiB buffer, so peak memory per file is that buffer, never the file size. Indexing policy: this module is one of the few places the workspace's `indexing_slicing = deny` is intentionally relaxed, and ev
- Public API / key implementation declarations (9):
  - L22: `const BLOCK: usize = 64;`
  - L26: `const H0: [u32; 8] = [`
  - L32: `const K: [u32; 64] = [`
  - L49: `pub struct Sha256 {`
  - L63: `pub const fn new() -> Self {`
  - L79: `pub fn update(&mut self, mut data: &[u8]) {`
  - L115: `pub fn finish(mut self) -> [u8; 32] {`
  - L207: `pub fn digest(data: &[u8]) -> [u8; 32] {`
  - L215: `pub fn hex(digest: &[u8; 32]) -> String {`
- Test/behavior coverage: 1 named functions in source, primarily module unit tests; representative names: `million_a_streamed`.

### `src/litric-defender/src/verdict.rs`

- Inventory: `F0692`; project source / rust-source; 6682 bytes; `Rust`.
- Size check: 191 lines; SHA-256 `4c85f0b3607d6f3885cdbf2f08628441a133ea9327eca0d040e8fe9bafcff809`.
- Source-level purpose: Verdict policy: the fail-open ladder that turns detection results into a single decision (ADR-0022 §6). The rule, and why: Defender runs inside the OS it protects. A bug that makes the scanner unable to scan must degrade to "unknown", never to "the machine cannot open files". But an *actual detection* is the opposite kind of event: it is evidence, not malfunction, and must never be waved through by a policy bug. So the ladder 
- Public API / key implementation declarations (6):
  - L21: `pub enum Mode {`
  - L34: `pub enum Observation {`
  - L56: `pub enum Tier {`
  - L66: `pub enum Decision {`
  - L87: `pub fn resolve(observation: &Observation, mode: Mode) -> Decision {`
  - L115: `pub fn cacheable_verdict(decision: &Decision) -> Option<Verdict> {`
- Test/behavior coverage: 4 named functions in source, primarily module unit tests; representative names: `clean_allows_in_both_modes`, `detection_denies_even_when_permissive`, `unscannable_follows_the_mode`, `every_decision_is_cacheable`.

## `litric-files` — Layer 4 Runtime — files API and LFP/1 service

Performs confined file I/O using `litric-storage` logical paths and `litric-trash` policy; implements LFP/1 request/reply, Unix socket service, freedesktop trash metadata, ZIP/TAR/GZIP, and std-only DEFLATE/INFLATE. Unix-only filesystem operations explicitly refuse symlink traversal; portable codecs and protocol parse on other platforms.

**Security/correctness review:** Security: no-follow root-relative operations, strict logical path validation, archive extraction traversal/type/size limits and typed LFP/1 errors. Tests cover codecs, protocol, path policy, and filesystem boundaries.

### `src/litric-files/Cargo.toml`

- Inventory: `F0702`; configuration / cargo-manifest; 548 bytes; `TOML`.
- Size check: 20 lines; SHA-256 `928c8e361bcf1c9cafc404f6ad69ae6457a2cd6ec2bd7bf01c9f8c3e7f3864c7`.
- Package configuration and dependency surface:
  - L1: `[package]`
  - L2: `name        = "litric-files"`
  - L3: `description = "Litric Files: confined file operations, freedesktop trash and archives, served by litric-filesd"`
  - L4: `version.workspace      = true`
  - L5: `edition.workspace      = true`
  - L6: `rust-version.workspace = true`
  - L7: `license.workspace      = true`
  - L9: `[lints]`
  - L10: `workspace = true`
  - L12: `[dependencies]`
  - L13: `litric-core    = { path = "../litric-core" }`
  - L14: `litric-storage = { path = "../litric-storage" }`
  - L15: `litric-trash   = { path = "../litric-trash" }`
  - L16: `litric-locale  = { path = "../litric-locale" }`
  - L18: `[lib]`
  - L19: `name = "litric_files"`
  - L20: `path = "src/lib.rs"`

### `src/litric-files/src/archive.rs`

- Inventory: `F0703`; project source / rust-source; 20390 bytes; `Rust`.
- Size check: 536 lines; SHA-256 `94e2b03cb57afa04988a511d204af0217db21da59543c263bec2c7cb248320a3`.
- Source-level purpose: `COMPRESS` and `EXTRACT` — archive creation and extraction under the ADR-0019 limits (§ "Extraction limits"). The codecs ([`crate::zip`], [`crate::tar`], [`crate::gzip`]) are pure in-memory transformations with their own bomb checks. This module adds the filesystem half: walking the real tree for compression, streaming validated entries to disk for extraction, and refusing everything the ADR names — absolute names, `..`, drive
- Public API / key implementation declarations (9):
  - L23: `pub const EXTRACT_SUFFIXES: [&str; 5] = [".zip", ".tar.gz", ".tgz", ".tar", ".gz"];`
  - L27: `pub enum ArchiveKind {`
  - L39: `pub fn from_name(name: &str) -> Option<Self> {`
  - L60: `pub fn load_archive_bytes(`
  - L106: `pub fn compress_zip(`
  - L263: `pub fn extract_archive(`
  - L487: `pub fn trim_slash(name: &str) -> &str {`
  - L497: `pub fn planned_extract_total(root: &Root, archive: &LogicalPath) -> Result<u64, Error> {`
  - L527: `pub fn read_exact_sized(r: &mut impl Read, size: usize) -> Result<Vec<u8>, Error> {`
- Test/behavior coverage: 0 named functions in source, primarily module unit tests; representative names: no inline `#[test]` functions detected.

### `src/litric-files/src/crc32.rs`

- Inventory: `F0704`; project source / rust-source; 2260 bytes; `Rust`.
- Size check: 95 lines; SHA-256 `7d61f755a7023c1ef0d34d779c414a43f760c540eed40a9e8731f411bad0d56e`.
- Source-level purpose: CRC-32 as gzip (RFC 1952 section 8) and ZIP (APPNOTE 4.4.7) define it: the reflected polynomial `0xEDB88320`, initial value and final XOR `0xFFFFFFFF`.
- Public API / key implementation declarations (6):
  - L7: `const POLY: u32 = 0xEDB8_8320;`
  - L30: `pub struct Crc32 {`
  - L43: `pub const fn new() -> Self {`
  - L48: `pub fn update(&mut self, data: &[u8]) {`
  - L60: `pub const fn value(&self) -> u32 {`
  - L67: `pub fn crc32(data: &[u8]) -> u32 {`
- Test/behavior coverage: 1 named functions in source, primarily module unit tests; representative names: `the_standard_check_value`.

### `src/litric-files/src/deflate.rs`

- Inventory: `F0705`; project source / rust-source; 18365 bytes; `Rust`.
- Size check: 540 lines; SHA-256 `77f2a5b66ba52322b4a4f1e1f3f74407bd16071944911c3a12d0dc268cfb4738`.
- Source-level purpose: DEFLATE encoding, written from RFC 1951. [`Deflater`] is a streaming encoder: it implements [`Write`], compresses input through LZ77 back-reference matching, and emits legal fixed-Huffman blocks (RFC 1951 §3.2.6). Every block it opens is closed with an end-of-block symbol; `finish` marks the last block final. Correctness contract: the round-trip tests against [`crate::inflate`] are exhaustive over the buffered paths, and every
- Public API / key implementation declarations (12):
  - L26: `const MAX_MATCH: usize = 258;`
  - L28: `const MIN_MATCH: usize = 3;`
  - L30: `const WSIZE: usize = 32_768;`
  - L32: `const HASH_BITS: u32 = 15;`
  - L33: `const HASH_SIZE: usize = 1 << HASH_BITS;`
  - L37: `pub struct Level {`
  - L47: `pub const fn fast() -> Self {`
  - L56: `pub const fn balanced() -> Self {`
  - L72: `pub struct Deflater<W: Write> {`
  - L94: `pub fn new(out: W, level: Level) -> Self {`
  - L114: `pub fn finish(mut self) -> io::Result<W> {`
  - L126: `pub fn pending_len(&self) -> usize {`
- Test/behavior coverage: 8 named functions in source, primarily module unit tests; representative names: `roundtrip`, `roundtrip_empty`, `roundtrip_literals_only`, `roundtrip_highly_repetitive`, `roundtrip_text_like`, `roundtrip_all_bytes_and_far_distances`, `roundtrip_chunked_writes`, `every_write_closes_its_block`.

### `src/litric-files/src/gzip.rs`

- Inventory: `F0706`; project source / rust-source; 19064 bytes; `Rust`.
- Size check: 529 lines; SHA-256 `b8d7aabae1fa404702f31ba7fbef3f5ec70474415453efd58834d5d89e1421e1`.
- Source-level purpose: gzip containers, written from RFC 1952. Compression goes through [`crate::deflate::Deflater`]; decompression through [`crate::inflate`]. Both directions validate the container rules the RFC states rather than assumes: the magic, the compression method byte, reserved flag bits, the optional header fields in the RFC's order, and the CRC32 + ISIZE trailer over the *uncompressed* data. Multi-member files (concatenated gzip streams
- Public API / key implementation declarations (15):
  - L32: `const MAGIC: [u8; 2] = [0x1f, 0x8b];`
  - L34: `const METHOD_DEFLATE: u8 = 8;`
  - L36: `const FHCRC: u8 = 0x02;`
  - L37: `const FEXTRA: u8 = 0x04;`
  - L38: `const FNAME: u8 = 0x08;`
  - L39: `const FCOMMENT: u8 = 0x10;`
  - L41: `const MAX_FIELD: usize = 4096;`
  - L54: `pub fn compress(data: &[u8], level: Level) -> Vec<u8> {`
  - L75: `pub struct MemberInfo {`
  - L92: `pub fn read_member<R: BufRead>(`
  - L184: `pub fn decompress<R: BufRead>(`
  - L210: `pub struct GzipEncoder<W: Write> {`
  - L218: `enum EncoderState<W: Write> {`
  - L230: `pub fn new(inner: W, level: Level) -> Self {`
  - L244: `pub fn finish(mut self) -> io::Result<W> {`
- Test/behavior coverage: 10 named functions in source, primarily module unit tests; representative names: `roundtrip`, `roundtrip_small_and_empty`, `roundtrip_repetitive_and_textual`, `corrupt_trailer_is_refused`, `bad_magic_is_refused`, `truncated_trailer_is_refused`, `garbage_between_members_is_refused`, `streaming_encoder_roundtrips`, `double_finish_is_refused`, `reserved_flags_are_refused`.

### `src/litric-files/src/inflate.rs`

- Inventory: `F0707`; project source / rust-source; 23567 bytes; `Rust`.
- Size check: 676 lines; SHA-256 `a887d3729e3d6ae4bb881854a07f9a094c2a9407ca418c9140c461f33beef89e`.
- Source-level purpose: DEFLATE decoding, written from RFC 1951. [`Inflater`] is a pull decoder: it implements [`Read`] over any [`BufRead`] source, keeps only the 32 KiB window the format allows a back-reference to reach, and refuses to produce more than a caller-set number of bytes. Every rule the RFC states is checked rather than assumed: a stored block's `NLEN` must complement `LEN`, block type 3 is an error, a code-length repeat with nothing to 
- Public API / key implementation declarations (20):
  - L22: `const WSIZE: usize = 32_768;`
  - L24: `const WMASK: usize = 32_767;`
  - L26: `const FAST_BITS: u32 = 9;`
  - L28: `const FAST_SIZE: usize = 512;`
  - L31: `const LEN_BASE: [u16; 29] = [`
  - L36: `const LEN_EXTRA: [u8; 29] = [`
  - L40: `const DIST_BASE: [u16; 30] = [`
  - L45: `const DIST_EXTRA: [u8; 30] = [`
  - L50: `const CL_ORDER: [usize; 19] = [`
  - L72: `pub(crate) const fn reverse_bits(code: u32, len: u32) -> u32 {`
  - L80: `struct BitReader<R> {`
  - L147: `struct Huffman {`
  - L295: `enum State {`
  - L304: `pub struct Inflater<R> {`
  - L330: `pub fn new(inner: R, max_output: u64) -> Self {`
  - L353: `pub const fn total_out(&self) -> u64 {`
  - L359: `pub fn is_done(&self) -> bool {`
  - L366: `pub const fn compressed_len(&self) -> u64 {`
  - L375: `pub fn into_parts(self) -> (Vec<u8>, R) {`
  - L599: `pub fn inflate_slice(data: &[u8], max_output: u64) -> Result<(Vec<u8>, usize), Error> {`
- Test/behavior coverage: 5 named functions in source, primarily module unit tests; representative names: `a_hand_assembled_fixed_block`, `a_stored_block_and_what_follows_it`, `malformed_streams_are_refused_with_corrupt_archive`, `a_distance_before_the_output_start_is_refused`, `the_output_cap_is_a_limit_error`.

### `src/litric-files/src/lib.rs`

- Inventory: `F0708`; project source / rust-source; 13699 bytes; `Rust`.
- Size check: 404 lines; SHA-256 `95aa6d70d15d2ccbb6e26e98749361a80ba408b0967dcf491983b38ee7799b6e`.
- Source-level purpose: Litric Files: confined file operations, the freedesktop.org trash on disk, and archives. **Layer 4 (Runtime).** It performs the I/O that `litric-storage` (names) and `litric-trash` (what deleting means) deliberately do not, and it is served to the shell by `litric-filesd` over LFP/1 (`docs/adr/0019-litric-files.md`). path      logical paths ("/Documents/a.txt"), name-collision rules    any host proto     LFP/1 request parsing 
- Public API / key implementation declarations (32):
  - L34: `pub mod crc32;`
  - L35: `pub mod deflate;`
  - L36: `pub mod gzip;`
  - L37: `pub mod inflate;`
  - L38: `pub mod path;`
  - L39: `pub mod proto;`
  - L40: `pub mod tar;`
  - L41: `pub mod trashinfo;`
  - L42: `pub mod zip;`
  - L45: `pub mod archive;`
  - L47: `pub mod ops;`
  - L49: `pub mod server;`
  - L51: `pub mod trash;`
  - L54: `pub use ops::Root;`
  - L59: `pub enum ErrorKind {`
  - L96: `pub const ALL: [Self; 15] = [`
  - L116: `pub const fn token(self) -> &'static str {`
  - L139: `pub struct Error {`
  - L147: `pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {`
  - L156: `pub const fn kind(&self) -> ErrorKind {`
  - L162: `pub fn message(&self) -> &str {`
  - L172: `pub fn from_io(context: &str, err: &io::Error) -> Self {`
  - L200: `pub fn into_io(self) -> io::Error {`
  - L229: `pub enum Policy {`
  - L241: `pub fn parse(token: &str) -> Option<Self> {`
  - L252: `pub const fn token(self) -> &'static str {`
  - L263: `pub struct Limits {`
  - L281: `pub const ADR_0019: Self = Self {`
  - L292: `pub const fn ratio_exceeded(&self, uncompressed: u64, compressed: u64) -> bool {`
  - L303: `pub enum Root {}`
  - L311: `pub fn open(`
  - L326: `pub use unsupported::Root;`
- Test/behavior coverage: 5 named functions in source, primarily module unit tests; representative names: `every_error_kind_has_the_adr_token`, `an_error_survives_a_trip_through_io_error`, `the_invalid_name_mapping_keys_off_the_raw_os_code`, `policies_round_trip`, `the_ratio_rule_exempts_small_entries_only`.

### `src/litric-files/src/ops.rs`

- Inventory: `F0709`; project source / rust-source; 43836 bytes; `Rust`.
- Size check: 1151 lines; SHA-256 `304725d3160e73a86181789ac55b1021e7a54b696abf4af4837030af09990c44`.
- Source-level purpose: [`Root`]: resolution of logical paths to host paths without ever following a symlink, and the file operations the LFP/1 commands are built on. The confinement rule (ADR-0019 §3) is enforced by construction: every path that reaches the filesystem is produced by walking the logical path one segment at a time, checking each intermediate with `symlink_metadata` and requiring a *real* directory. A symlink anywhere in the chain is r
- Public API / key implementation declarations (33):
  - L27: `pub const LIST_CAP: usize = 10_000;`
  - L32: `pub const WALK_CAP: usize = 50_000;`
  - L37: `const COPY_DEPTH_CAP: usize = 64;`
  - L41: `pub enum EntryKind {`
  - L55: `pub const fn token(self) -> &'static str {`
  - L67: `pub struct StatInfo {`
  - L89: `pub struct Root {`
  - L103: `pub fn open(host: &Path, zone: litric_locale::TimeZone) -> Result<Self, Error> {`
  - L125: `pub fn host_root(&self) -> &Path {`
  - L131: `pub const fn zone(&self) -> &litric_locale::TimeZone {`
  - L141: `pub fn validate_segment(name: &str) -> Result<(), Error> {`
  - L155: `pub fn resolve(&self, logical: &LogicalPath) -> Result<PathBuf, Error> {`
  - L200: `pub fn resolve_parent(&self, logical: &LogicalPath) -> Result<(PathBuf, String), Error> {`
  - L218: `pub fn ensure_dir_chain(&self, host_dir: &Path) -> Result<(), Error> {`
  - L265: `pub fn stat_host(&self, host: &Path, name: &str) -> Result<StatInfo, Error> {`
  - L301: `pub fn stat(&self, logical: &LogicalPath) -> Result<StatInfo, Error> {`
  - L312: `pub fn list_dir(`
  - L358: `pub fn measure(&self, logical: &LogicalPath) -> Result<(u64, u64, u64, bool), Error> {`
  - L410: `pub fn mkdir(&self, parent: &LogicalPath, name: &str) -> Result<PathBuf, Error> {`
  - L424: `pub fn new_file(&self, parent: &LogicalPath, name: &str) -> Result<PathBuf, Error> {`
  - L444: `pub fn rename(&self, logical: &LogicalPath, new_name: &str) -> Result<PathBuf, Error> {`
  - L462: `pub fn is_within(host: &Path, dir: &Path) -> bool {`
  - L473: `pub fn copy_item(`
  - L598: `pub fn move_item(`
  - L742: `pub fn remove_host(&self, host: &Path) -> Result<(), Error> {`
  - L756: `pub fn delete(&self, logical: &LogicalPath) -> Result<(), Error> {`
  - L767: `pub fn stat_with_items(&self, logical: &LogicalPath) -> Result<(StatInfo, u64), Error> {`
  - L789: `pub fn search(`
  - L865: `pub fn recent(&self) -> Result<Vec<(String, u64, u64)>, Error> {`
  - L910: `struct Measure {`
  - L920: `enum NameAction {`
  - L937: `pub fn copy_of_name(base: &str) -> String {`
  - L988: `pub fn unix_time_of(meta: &fs::Metadata) -> u64 {`
- Test/behavior coverage: 4 named functions in source, primarily module unit tests; representative names: `replace_keeps_the_destination_when_the_copy_fails`, `replace_installs_the_new_item_and_removes_the_old`, `move_onto_an_occupied_directory_swaps_instead_of_deleting_first`, `resolve_reports_a_file_standing_in_the_middle`.

### `src/litric-files/src/path.rs`

- Inventory: `F0710`; project source / rust-source; 10089 bytes; `Rust`.
- Size check: 315 lines; SHA-256 `98cd9f9320b600da82e6b6dc40b098aa6529b472d3b7048fc0b2fa4be9c63a48`.
- Source-level purpose: Logical paths (`/Documents/report.odt`), validated against the ADR-0019 segment rules. The shell never sends a host path; every path is relative to the user root. Validation is total: a path is either valid or rejected, never "valid with surprises". The rules exist so that a validated path cannot traverse, cannot smuggle control bytes through the protocol, and cannot exceed filesystem limits.
- Public API / key implementation declarations (11):
  - L14: `pub const MAX_SEGMENT: usize = 255;`
  - L16: `pub const MAX_PATH: usize = 4096;`
  - L20: `pub enum PathError {`
  - L72: `pub struct LogicalPath {`
  - L81: `pub fn root() -> Self {`
  - L92: `pub fn parse(text: &str) -> Result<Self, PathError> {`
  - L124: `pub fn join(&self, segment: &str) -> Result<Self, PathError> {`
  - L135: `pub fn as_str(&self) -> &str {`
  - L143: `pub fn file_name(&self) -> &str {`
  - L151: `pub fn parent(&self) -> Option<Self> {`
  - L165: `pub fn segments(&self) -> impl Iterator<Item = &str> {`
- Test/behavior coverage: 4 named functions in source, primarily module unit tests; representative names: `parses_and_normalizes`, `rejects_dot_segments_and_traversal`, `rejects_control_bytes_and_long_segments`, `rejects_overlong_paths`.

### `src/litric-files/src/proto.rs`

- Inventory: `F0711`; project source / rust-source; 11442 bytes; `Rust`.
- Size check: 349 lines; SHA-256 `fc852877aff7bd499654f96d3398f714d0e86ec65c5c09d832ce03221255931d`.
- Source-level purpose: LFP/1 — the Litric Files Protocol, version 1 (ADR-0019). Request: one UTF-8 line ending in `\n`, at most 16384 bytes, fields separated by exactly one space, free text percent-encoded ([`litric_core::pct`]). Response: `OK <n>` followed by exactly `n` lines, or one line `ERR <kind> <pct-msg>`. Long operations may send any number of `PROGRESS <done> <total> <pct>` lines before the final `OK`/`ERR`. Entry lines are `key=value` pai
- Public API / key implementation declarations (11):
  - L18: `pub const MAX_LINE: usize = 16_384;`
  - L26: `pub struct Request {`
  - L35: `pub enum ParseError {`
  - L96: `pub fn parse_request(line: &str) -> Result<Request, ParseError> {`
  - L139: `pub fn decode_field(field: &str) -> Result<String, Error> {`
  - L146: `pub fn ok(count: usize) -> String {`
  - L153: `pub fn entry(pairs: &[(&str, &str)]) -> String {`
  - L168: `pub fn err(kind: ErrorKind, message: &str) -> String {`
  - L174: `pub fn progress(done: u64, total: u64, current_pct: u64) -> String {`
  - L180: `pub enum Reply {`
  - L206: `pub fn parse_reply(line: &str) -> Result<Reply, Error> {`
- Test/behavior coverage: 3 named functions in source, primarily module unit tests; representative names: `parses_verbs_and_fields`, `rejects_the_documented_shapes`, `replies_round_trip`.

### `src/litric-files/src/server.rs`

- Inventory: `F0712`; project source / rust-source; 53255 bytes; `Rust`.
- Size check: 1391 lines; SHA-256 `87cc75ab06ed2590c2ad369058e66fcbdfcd287651d746f198a3ab162a5dd702`.
- Source-level purpose: The LFP/1 server: request dispatch, one connection at a time (ADR-0019 §1, `Accept=yes` per-connection activation). Framing is over any [`io::BufRead`]/[`io::Write`] pair so the protocol logic runs in tests without a socket; [`serve_unix`] wires the same loop to a `UnixListener`. Every reply is built by [`crate::proto`]; every filesystem action goes through [`crate::ops`], [`crate::trash`] or [`crate::archive`], so the confine
- Public API / key implementation declarations (16):
  - L26: `const TEXT_CAP: usize = 64 * 1024;`
  - L29: `const BINARY_PROBE: usize = 8 * 1024;`
  - L33: `pub enum Outcome {`
  - L44: `pub struct Connection {`
  - L51: `pub fn new(root: Root) -> Self {`
  - L57: `pub const fn root(&self) -> &Root {`
  - L69: `pub fn dispatch(&mut self, line: &str, progress: &mut dyn FnMut(u64, u64)) -> Outcome {`
  - L122: `const PLACES: [(&str, &str, &str); 10] = [`
  - L691: `enum Kind {`
  - L698: `enum Move {`
  - L779: `pub fn serve_stream(input: &mut dyn BufRead, output: &mut dyn Write, root: Root) -> io::Result<()> {`
  - L837: `pub fn serve_unix(socket: &Path, root: Root) -> io::Result<()> {`
  - L855: `pub struct MountEntry {`
  - L866: `pub fn parse_mounts(text: &str) -> Vec<MountEntry> {`
  - L886: `pub fn mount_hosting<'a>(mounts: &'a [MountEntry], path: &str) -> Option<&'a MountEntry> {`
  - L906: `pub fn device_bytes(sysfs_root: &Path, source: &str) -> Option<u64> {`
- Test/behavior coverage: 12 named functions in source, primarily module unit tests; representative names: `places_lists_the_conventional_spots`, `mkdir_stat_and_list_round_trip`, `rename_refuses_to_overwrite`, `trash_restore_and_purge_round_trip`, `restoring_an_entry_that_came_from_another_root_is_refused_not_guessed`, `trashlist_reports_the_original_logical_location_for_a_legacy_entry`, `conflicts_names_only_the_occupied`, `copy_reports_progress_and_keeps_both_into_the_source_dir`, `move_refuses_a_directory_into_itself`, `search_finds_by_name_and_marks_truncation_shape`, `readtext_reports_text_and_binary`, `logical_paths_survive_the_wire`.

### `src/litric-files/src/tar.rs`

- Inventory: `F0713`; project source / rust-source; 17904 bytes; `Rust`.
- Size check: 488 lines; SHA-256 `0f4cf153c3cc8157cc5ca92ae14f3cd7a91499db9dbe287092cb97b4e2e5f82d`.
- Source-level purpose: ustar archives, written from POSIX 1003.1-1988 (the ustar format). Writing produces POSIX ustar headers (typeflags `0` and `5`, ustar magic, octal fields) — enough for the file manager's COMPRESS/EXTRACT, and readable by every tar tool. Reading validates rather than assumes: the checksum, the size/length fields, and the entry-name rules shared with ADR-0019 extraction (`unsafe-archive-entry` for absolute names, `..`, drive let
- Public API / key implementation declarations (7):
  - L21: `const USTAR_MAGIC: [u8; 8] = [b'u', b's', b't', b'a', b'r', 0, b'0', b'0'];`
  - L23: `const TYPE_FILE: u8 = b'0';`
  - L25: `const TYPE_DIR: u8 = b'5';`
  - L27: `const BLOCK: usize = 512;`
  - L43: `pub struct EntrySpec {`
  - L63: `pub fn write_archive<W: Write>(out: &mut W, entries: &[(EntrySpec, Vec<u8>)]) -> io::Result<()> {`
  - L185: `pub fn read_archive<R: Read>(`
- Test/behavior coverage: 5 named functions in source, primarily module unit tests; representative names: `roundtrip_files_and_dirs`, `every_forbidden_entry_shape_is_refused_on_read`, `link_entries_are_refused`, `checksum_mismatch_is_refused`, `octal_fields_round_trip`.

### `src/litric-files/src/trash.rs`

- Inventory: `F0714`; project source / rust-source; 24933 bytes; `Rust`.
- Size check: 574 lines; SHA-256 `ba60ca17e3028bd16518be83b1a5f0e2bb3ab4c57a5d7990600f3d570d8d718b`.
- Source-level purpose: The freedesktop.org home trash **on disk** (ADR-0019 §4). `litric-trash` owns *what deleting means* — retention, capacity, the ledger — and does no I/O. This module is the disk side of the same contract: it maintains `<root>/.local/share/Trash/{files,info}` in the exact layout the freedesktop specification describes, so other tools (`gio trash`, `kioclient`, every Linux file manager) read the same trash Litric writes. The ledg
- Public API / key implementation declarations (14):
  - L22: `pub const TRASH_DIR_NAME: &str = ".local/share/Trash";`
  - L26: `pub struct OnDiskItem {`
  - L53: `pub struct DiskTrash {`
  - L68: `pub fn open(root: &Root) -> Result<Self, Error> {`
  - L96: `pub fn files_dir(&self) -> &std::path::Path {`
  - L116: `pub fn put(`
  - L223: `pub fn list(&self) -> Result<Vec<OnDiskItem>, Error> {`
  - L281: `pub fn get(&self, name: &str) -> Option<OnDiskItem> {`
  - L295: `pub fn restore(&self, name: &str, dest: &std::path::Path) -> Result<std::path::PathBuf, Error> {`
  - L323: `pub fn purge(&self, name: &str) -> Result<u64, Error> {`
  - L348: `pub fn empty(&self) -> Result<(u64, u64), Error> {`
  - L387: `pub fn iso_local(root: &Root, unix: u64) -> String {`
  - L418: `pub fn entry_size(meta: &fs::Metadata) -> u64 {`
  - L428: `pub fn entry_mtime(meta: &fs::Metadata) -> u64 {`
- Test/behavior coverage: 3 named functions in source, primarily module unit tests; representative names: `the_recorded_path_is_the_absolute_location_not_a_uri`, `an_entry_written_in_the_legacy_uri_form_still_lists_and_restores`, `an_entry_whose_path_names_nothing_inside_the_root_is_refused_not_guessed`.

### `src/litric-files/src/trashinfo.rs`

- Inventory: `F0715`; project source / rust-source; 14810 bytes; `Rust`.
- Size check: 368 lines; SHA-256 `48b93c08987d7a6009cfd6a27776f3119c62b0c1b6205c0ccbdcdfafc3931d22`.
- Source-level purpose: The `.trashinfo` record: the `Path=` field, and the body around it. The trash directory on disk has **two** writers in this repository — the `litric-filesd` daemon ([`crate::trash::DiskTrash`]) and the shell's own bridge (`litric_shell::fsbridge`). Before this module existed each one formatted the record itself, and the same defect had to be found and fixed twice (BUG-0011), twice differently. One definition, used by both, is 
- Public API / key implementation declarations (8):
  - L49: `pub fn pct_encode(text: &str) -> String {`
  - L50: `const HEX: &[u8; 16] = b"0123456789ABCDEF";`
  - L75: `pub fn pct_decode(field: &str) -> String {`
  - L106: `pub fn record(host_path: &Path, deleted_iso: &str) -> String {`
  - L119: `pub fn is_not_a_uri(recorded: &str) -> bool {`
  - L129: `pub fn strip_root(host_root: &Path, absolute: &str) -> Option<String> {`
  - L171: `pub fn logical_location(host_root: &Path, recorded: &str) -> String {`
  - L188: `pub fn maps_into_root(host_root: &Path, recorded: &str) -> bool {`
- Test/behavior coverage: 11 named functions in source, primarily module unit tests; representative names: `the_record_carries_the_absolute_location_and_the_date`, `the_record_never_writes_a_uri`, `spaces_and_non_ascii_are_percent_encoded_and_round_trip`, `an_absolute_location_under_the_root_maps_back_to_a_logical_path`, `a_legacy_uri_entry_still_restores_to_the_right_place`, `a_legacy_uri_entry_that_is_still_percent_encoded_is_decoded_first`, `the_root_itself_is_the_logical_root`, `a_host_root_of_slash_keeps_the_leading_separator`, `a_trailing_slash_on_the_root_does_not_change_the_answer`, `a_sibling_that_shares_the_roots_text_prefix_is_not_inside_it`, `an_unrecognised_value_is_returned_unchanged_so_it_can_be_refused`.

### `src/litric-files/src/zip.rs`

- Inventory: `F0716`; project source / rust-source; 23864 bytes; `Rust`.
- Size check: 597 lines; SHA-256 `0be139eeef76bf867485b3aad74cf14a33b6b4337ceabdd4eeb624eacf640e2d`.
- Source-level purpose: ZIP containers, written from the PKWARE APPNOTE (the format specification every zip tool implements). Writing: local file headers + data + central directory + end record, method 8 (deflate) or 0 (store), 32-bit fields, MS-DOS timestamps. Reading: validates the central directory against the local headers, refuses ZIP64 (`too-large`), refuses entries the ADR-0019 extraction rules refuse, refuses overlapping data between entries 
- Public API / key implementation declarations (13):
  - L26: `const SIG_LOCAL: [u8; 4] = [0x50, 0x4b, 0x03, 0x04];`
  - L28: `const SIG_CENTRAL: [u8; 4] = [0x50, 0x4b, 0x01, 0x02];`
  - L30: `const SIG_EOCD: [u8; 4] = [0x50, 0x4b, 0x05, 0x06];`
  - L32: `const METHOD_STORE: u16 = 0;`
  - L34: `const METHOD_DEFLATE: u16 = 8;`
  - L36: `const VERSION_NEEDED: u16 = 20;`
  - L38: `const DOS_EPOCH: u32 = 0x0021;`
  - L69: `pub struct ZipSpec {`
  - L79: `pub struct ZipEntry {`
  - L95: `pub fn write_zip<W: Write>(out: &mut W, entries: &[(ZipSpec, Vec<u8>)]) -> io::Result<()> {`
  - L234: `pub fn read_zip<R: io::Read>(r: &mut R, limits: &Limits) -> Result<Vec<ZipEntry>, io::Error> {`
  - L244: `pub fn read_zip_slice(raw: &[u8], limits: &Limits) -> Result<Vec<ZipEntry>, io::Error> {`
  - L246: `const EOCD_MIN: usize = 22;`
- Test/behavior coverage: 6 named functions in source, primarily module unit tests; representative names: `roundtrip_mixed_methods`, `every_forbidden_name_is_refused`, `ratio_bomb_is_refused`, `overlapping_entries_are_refused`, `crc_mismatch_is_refused`, `zip64_is_refused`.

## `litric-winbridge` — Layer 6 Native Ecosystem / Windows Compatibility

Platform-independent Windows PE inspection, installer-family identification, bottle/runtime state and ordered installation planning. The crate decides a user-visible plan but does not execute, write files, spawn processes, or access the network; execution is delegated to supervised `litric-compatd`.

**Security/correctness review:** Security: unsafe-free and depends only downward on `litric-core`; bounded PE inspection window is 1 MiB; parses architecture/subsystem and rejects unsupported software with user-facing reasons rather than assuming compatibility.

### `src/litric-winbridge/Cargo.toml`

- Inventory: `F0786`; configuration / cargo-manifest; 366 bytes; `TOML`.
- Size check: 13 lines; SHA-256 `694ae5335d145a34c86127d14739064d28cbc7afdc5ab1f9f70748e2bced2384`.
- Package configuration and dependency surface:
  - L1: `[package]`
  - L2: `name        = "litric-winbridge"`
  - L3: `description = "LitricOS Windows compatibility: PE inspection, installer detection, bottle lifecycle, install planning"`
  - L4: `version.workspace      = true`
  - L5: `edition.workspace      = true`
  - L6: `rust-version.workspace = true`
  - L7: `license.workspace      = true`
  - L9: `[lints]`
  - L10: `workspace = true`
  - L12: `[dependencies]`
  - L13: `litric-core = { path = "../litric-core" }`

### `src/litric-winbridge/src/bottle.rs`

- Inventory: `F0787`; project source / rust-source; 18356 bytes; `Rust`.
- Size check: 587 lines; SHA-256 `1e30565dbf07394f0127656ef1766d1080a2afb1bc9e0532a818927e8181e02a`.
- Source-level purpose: Bottles: disposable, per-application Windows environments. ECOSYSTEM section 17.1 requires that a Windows application run inside an environment that can be created, suspended, reset, repaired, migrated and deleted. This module is that model. The traditional Wine model is a single shared `~/.wine` prefix. It is the wrong shape for an operating system, for three reasons: 1. **Blast radius.** Every Windows program shares one regi
- Public API / key implementation declarations (29):
  - L32: `pub enum WindowsVersion {`
  - L45: `pub const fn wine_token(self) -> &'static str {`
  - L57: `pub enum Runtime {`
  - L75: `pub fn token(&self) -> String {`
  - L84: `pub const fn has_d3d_translation(&self) -> bool {`
  - L102: `pub enum Component {`
  - L120: `pub const fn token(self) -> &'static str {`
  - L135: `pub enum BottleState {`
  - L151: `pub const fn can_launch(self) -> bool {`
  - L161: `pub fn allowed_transitions(self) -> &'static [BottleState] {`
  - L179: `pub fn can_transition_to(self, next: Self) -> bool {`
  - L190: `pub struct Isolation {`
  - L238: `pub const fn for_installer() -> Self {`
  - L253: `pub const fn for_game() -> Self {`
  - L271: `pub fn required_capabilities(&self) -> Vec<&'static str> {`
  - L303: `pub struct Bottle {`
  - L321: `pub fn new(id: AppId, runtime: Runtime) -> Self {`
  - L335: `pub fn with_windows_version(mut self, v: WindowsVersion) -> Self {`
  - L342: `pub fn with_isolation(mut self, iso: Isolation) -> Self {`
  - L349: `pub fn with_components(mut self, components: impl IntoIterator<Item = Component>) -> Self {`
  - L358: `pub const fn id(&self) -> &AppId {`
  - L363: `pub const fn runtime(&self) -> &Runtime {`
  - L368: `pub const fn windows_version(&self) -> WindowsVersion {`
  - L373: `pub const fn is_win64(&self) -> bool {`
  - L378: `pub fn components(&self) -> &[Component] {`
  - L383: `pub const fn isolation(&self) -> &Isolation {`
  - L388: `pub const fn state(&self) -> BottleState {`
  - L398: `pub fn transition(&mut self, next: BottleState) -> Result<(), BottleState> {`
  - L412: `pub fn path_component(&self) -> String {`
- Test/behavior coverage: 2 named functions in source, primarily module unit tests; representative names: `default_isolation_denies_network_and_home`, `installer_profile_is_the_most_restrictive`.

### `src/litric-winbridge/src/compat.rs`

- Inventory: `F0788`; project source / rust-source; 12867 bytes; `Rust`.
- Size check: 359 lines; SHA-256 `1386a95749b6dee724176f76fc95741f992667e1e180f16af1baabaa9f7ce0dc`.
- Source-level purpose: What is known about running a particular Windows program here. A translation layer either works for a program or it does not, and for a large class of software the answer is known in advance and is "no" — not because the translation is incomplete, but because the program contains a component whose entire purpose is to detect that it is being translated. Kernel-level anti-cheat is the clearest case: it is *designed* to refuse t
- Public API / key implementation declarations (11):
  - L33: `pub enum Verdict {`
  - L47: `pub const fn should_offer_bottle(self) -> bool {`
  - L53: `pub const fn token(self) -> &'static str {`
  - L66: `pub enum Alternative {`
  - L85: `pub const fn advice(self) -> &'static str {`
  - L105: `pub struct Entry {`
  - L123: `pub const KNOWN: &[Entry] = &[`
  - L182: `pub fn lookup(file_name: &str) -> Option<&'static Entry> {`
  - L189: `pub struct Assessment {`
  - L203: `pub fn of(file_name: &str) -> Self {`
  - L229: `pub fn lines(&self) -> Vec<String> {`
- Test/behavior coverage: 9 named functions in source, primarily module unit tests; representative names: `roblox_is_recorded_as_blocked_with_a_reason_and_a_way_forward`, `matching_is_case_insensitive_and_matches_anywhere_in_the_name`, `an_unknown_program_is_unknown_and_not_optimistic`, `a_known_working_program_offers_no_alternative_it_does_not_need`, `a_caveat_is_neither_a_promise_nor_a_refusal`, `every_entry_carries_a_reason_that_could_be_checked`, `no_two_entries_share_a_key`, `every_blocked_or_caveat_entry_says_what_to_do_instead`, `status_lines_carry_the_verdict`.

### `src/litric-winbridge/src/installer.rs`

- Inventory: `F0789`; project source / rust-source; 11819 bytes; `Rust`.
- Size check: 315 lines; SHA-256 `2bdca7c7343a30e968d2cfeecc21152826e5f58096e44989c657738db2f037d8`.
- Source-level purpose: Installer family detection and unattended-install flags. When a user double-clicks `setup.exe` in LitricOS, the goal is that a progress bar appears and an application shows up in the launcher. That is only possible if we know *which* installer we are looking at, because every installer toolkit has its own switch for "do not ask me anything". The alternative is showing the user a Windows installer wizard rendered by Wine, which
- Public API / key implementation declarations (8):
  - L29: `pub enum InstallerFamily {`
  - L59: `pub fn silent_args(self) -> Option<&'static [&'static str]> {`
  - L82: `pub const fn needs_msiexec(self) -> bool {`
  - L88: `pub const fn token(self) -> &'static str {`
  - L109: `pub const fn describe(self) -> &'static str {`
  - L134: `const SIGNATURES: &[(&[u8], InstallerFamily)] = &[`
  - L150: `const GENERIC_INSTALLER_HINTS: &[&[u8]] = &[`
  - L166: `pub fn detect(head: &[u8], is_pe: bool) -> InstallerFamily {`
- Test/behavior coverage: 4 named functions in source, primarily module unit tests; representative names: `plain_program_is_not_an_installer`, `generic_hints_yield_unknown_not_a_guess`, `every_recognised_family_has_silent_args`, `contains_handles_edges`.

### `src/litric-winbridge/src/lib.rs`

- Inventory: `F0790`; project source / rust-source; 18887 bytes; `Rust`.
- Size check: 511 lines; SHA-256 `4265c50077e07b57add080b626a14c43c70acb8e117026c72c416f77cffd22d2`.
- Source-level purpose: LitricOS Windows compatibility: inspection and install planning. This crate sits at **Layer 6 (Native Ecosystem / Applications)**. It depends downward on `litric-core` and on nothing else. It is subordinate to the native platform by construction: nothing in LitricOS Core references it, and removing it entirely leaves a working operating system. That is the architectural invariant from ECOSYSTEM section 22, rule 15, and it is e
- Public API / key implementation declarations (15):
  - L27: `pub mod bottle;`
  - L28: `pub mod compat;`
  - L29: `pub mod installer;`
  - L30: `pub mod pe;`
  - L32: `pub use bottle::{Bottle, BottleState, Component, Isolation, Runtime, WindowsVersion};`
  - L33: `pub use installer::InstallerFamily;`
  - L34: `pub use pe::{Machine, PeError, PeInfo, Subsystem};`
  - L43: `pub const INSPECT_WINDOW: usize = 1024 * 1024;`
  - L47: `pub enum Rejection {`
  - L63: `pub fn user_message(&self) -> String {`
  - L79: `pub enum Step {`
  - L112: `pub fn label(&self) -> String {`
  - L133: `pub struct InstallPlan {`
  - L152: `pub fn is_unattended(&self) -> bool {`
  - L176: `pub fn plan(head: &[u8], app_id: AppId) -> Result<InstallPlan, Rejection> {`
- Test/behavior coverage: 9 named functions in source, primarily module unit tests; representative names: `plans_an_nsis_install_unattended`, `dotnet_binaries_get_wine_and_the_framework`, `portable_programs_skip_the_installer_step`, `msi_is_planned_via_msiexec`, `installs_request_only_gpu_by_default`, `signature_presence_is_never_reported_as_validity`, `rejects_non_windows_files_distinctly_from_unrunnable_ones`, `every_step_has_a_human_label`, `planning_never_panics_on_arbitrary_input`.

### `src/litric-winbridge/src/pe.rs`

- Inventory: `F0791`; project source / rust-source; 27772 bytes; `Rust`.
- Size check: 765 lines; SHA-256 `8dfc8bbacf80bd61b7eccd718527b558d63bf88a0b10172b742d1cbae76cc46d`.
- Source-level purpose: Portable Executable inspection. Parses just enough of the PE/COFF format to answer the questions LitricOS actually needs to answer before it will run a Windows binary: * Is this even a PE, or is it something wearing an `.exe` extension? * Which architecture? (An ARM64 binary on an x86-64 machine is a clear "no", not a mysterious crash three seconds into a Wine prefix.) * GUI or console? Decides whether the shell opens a termin
- Public API / key implementation declarations (41):
  - L34: `pub enum PeError {`
  - L88: `pub enum Machine {`
  - L123: `pub const fn runnable_on_x86_64(self) -> bool {`
  - L129: `pub const fn token(self) -> &'static str {`
  - L153: `pub enum Subsystem {`
  - L179: `pub const fn token(self) -> &'static str {`
  - L202: `pub struct Hardening {`
  - L221: `pub fn score(self) -> u8 {`
  - L234: `pub struct PeInfo {`
  - L268: `pub const fn is_runnable_on_x86_64(&self) -> bool {`
  - L276: `pub fn blocking_reason(&self) -> Option<&'static str> {`
  - L321: `const MIN_PE_LEN: usize = 0x40;`
  - L324: `const MZ_MAGIC: u16 = 0x5a4d;`
  - L326: `const E_LFANEW_OFF: usize = 0x3c;`
  - L328: `const PE_MAGIC: u32 = 0x0000_4550;`
  - L330: `const COFF_MACHINE_OFF: usize = 4;`
  - L331: `const COFF_NUMSECTIONS_OFF: usize = 6;`
  - L332: `const COFF_SIZEOFOPT_OFF: usize = 20;`
  - L333: `const COFF_CHARACTERISTICS_OFF: usize = 22;`
  - L334: `const COFF_HEADER_LEN: usize = 24; // signature (4) + COFF fields (20)`
  - L336: `const OPT_MAGIC_PE32: u16 = 0x010b;`
  - L337: `const OPT_MAGIC_PE32PLUS: u16 = 0x020b;`
  - L342: `const OPT_MAJOR_OS_OFF: usize = 40;`
  - L343: `const OPT_MINOR_OS_OFF: usize = 42;`
  - L344: `const OPT_SUBSYSTEM_OFF: usize = 68;`
  - L345: `const OPT_DLLCHARACTERISTICS_OFF: usize = 70;`
  - L348: `const OPT_DATADIR_OFF_PE32: usize = 96;`
  - L349: `const OPT_DATADIR_OFF_PE32PLUS: usize = 112;`
  - L350: `const DATADIR_ENTRY_LEN: usize = 8;`
  - L351: `const DATADIR_INDEX_CERTIFICATE: usize = 4;`
  - L352: `const DATADIR_INDEX_CLR: usize = 14;`
  - L354: `const IMAGE_FILE_DLL: u16 = 0x2000;`
  - L356: `const DLLCHAR_HIGH_ENTROPY_VA: u16 = 0x0020;`
  - L357: `const DLLCHAR_DYNAMIC_BASE: u16 = 0x0040;`
  - L358: `const DLLCHAR_FORCE_INTEGRITY: u16 = 0x0080;`
  - L359: `const DLLCHAR_NX_COMPAT: u16 = 0x0100;`
  - L360: `const DLLCHAR_NO_SEH: u16 = 0x0400;`
  - L361: `const DLLCHAR_GUARD_CF: u16 = 0x4000;`
  - L371: `pub fn inspect(buf: &[u8]) -> Result<PeInfo, PeError> {`
  - L495: `pub fn is_msi(buf: &[u8]) -> bool {`
  - L496: `const OLE_MAGIC: [u8; 8] = [0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1];`
- Test/behavior coverage: 8 named functions in source, primarily module unit tests; representative names: `parses_a_64bit_gui_application`, `parses_a_32bit_console_application`, `refuses_things_that_cannot_run_with_a_specific_reason`, `rejects_non_pe_input_without_panicking`, `rejects_out_of_range_pe_offset`, `rejects_dos_stub_without_pe_image`, `rejects_bad_optional_header`, `truncation_at_every_offset_never_panics`.

