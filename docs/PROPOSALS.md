# Proposals

Nothing here is done yet. Each item: gain, effort, risk.

1. **Clear the last two RustSec advisories (quick-xml ≥ 0.41).** `quick-xml` 0.39 is pulled in by `plist` (via `netdev`, macOS only) and by `wayland-scanner` (build time, reads trusted protocol XML). Gain: `cargo audit` shows 0 vulnerabilities. Effort: wait for `netdev` and `wayland-scanner` releases; re-run `cargo update`. Risk: none; real exposure is low.
2. **Full `cargo update`, plus one lint fix.** About 166 crates are behind in the lockfile. A full update makes `clippy -D warnings` fail with `result_large_err` in `mousehop/src/main.rs` (the `MousehopError` enum carries 128-byte variants) and a deprecation of `tokio::io::unix::AsyncFd::new` (tokio ≥ 1.53; use `AsyncFd::register`). Gain: current dependencies, fewer "unsound" warnings (`git2`, `rand`, `event-listener`). Effort: about 1 hour (box the large error variants, switch to `AsyncFd::register`). Risk: low; needs the four-OS CI run.
3. **Add `cargo audit` (or `cargo deny`) to CI.** `deny.toml` exists but is the unedited template and nothing runs it. Gain: new advisories show up on their own. Effort: 30 minutes. Risk: noisy until item 1 is done.
4. **Turn on Dependabot (alerts and updates).** Alerts are disabled on `rusya13/mousehop`. Gain: automatic notice and PRs. Effort: a repo setting plus a small `dependabot.yml`; actions are already at current majors except `actions/cache@v4` and `actions/setup-python@v5`. Risk: PR noise.
5. **README links still point at `jondkinney/mousehop`** (badges, releases, `cargo install --git`). Open PR "FL-622" on this repo handles the owner change; decide whether `rusya13` stays a fork or becomes the main repo.
6. **Toolchain mismatch.** `rust-toolchain.toml` pins 1.95.0 but CI uses `dtolnay/rust-toolchain@stable`. The comment in the pin file says CI is deterministic, which is not true. Pick one.
7. **Stack speed: no change.** It is already Rust. No server, so no traffic, cost or memory question for a faster stack.
