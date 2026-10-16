# Technical changelog

Framework, build, CI and packaging changes, from Git history. Feature and bug-fix history for users is in [CHANGELOG.md](../CHANGELOG.md) (written by release-plz). There is no website, so there are no my-seo annotations.

| Date | Change |
|---|---|
| 2026-04 | Forked from `feschber/lan-mouse` (via `jondkinney/mousehop`). |
| 2026-05-17 | Rebranded to Mousehop; input crates renamed to the `mousehop-*` namespace. |
| 2026-05-19 | AUR variants added: `mousehop-bin` and `mousehop-git`. |
| 2026-05-20 | Root crate moved into its own subdirectory (#17); later renamed to `mousehop/`. |
| 2026-05-28 | release-plz adopted: release PRs, crates.io publishing, tags (v0.11.x onward). |
| 2026-05-29 | `aur-publish` can ship an install scriptlet. |
| 2026-05-30 | release-plz release PR waits for the publish job (serialised). |
| 2026-06-04 | macOS bundle fixes (icon, menu-bar template, LSUIElement plist). |
| 2026-07-24 | CI: Windows GTK build fixed for the VS2026 runner image (#67). |
| 2026-08-14 | macOS bundle includes transitive dylibs; Windows CI unblocked; release retry procedure documented. |
| 2026-08-16 | Shared release workflows synced with the sibling `jondkinney/*` repos; `workflow-drift-check` added. |
| 2026-08-24 | Windows build drops the console window; GTK tray icon with close-to-tray. |
| 2026-09-01 | Nix build fetches crates from the static CDN. |
| 2026-09-05 | v0.17.1 released (layer-shell edge fix). Last upstream commit as of this audit. |
| 2026-10-16 | Audit: `rustls` 0.23.37→0.23.46, `rustls-webpki` 0.103.10→0.103.15, `quick-xml` 0.39.2→0.39.4 (lockfile only). Docs added. |
