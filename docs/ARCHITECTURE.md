# Architecture

Mousehop is a software KVM. One machine captures mouse and keyboard input and sends it over the network to another machine, which replays it. The long design notes are in [DOC.md](../DOC.md) and [AGENTS.md](../AGENTS.md); this page is the short map.

## Components (Cargo workspace)

| Crate | Job |
|---|---|
| `mousehop` | The binary. Config, the daemon, the network listener, mDNS discovery, DTLS, clipboard and crossing logic. |
| `mousehop-gtk` | GTK4 / libadwaita settings window, Windows tray, macOS menu bar. |
| `mousehop-cli` | `mousehop cli ...` commands that talk to the daemon. |
| `mousehop-ipc` | Local socket protocol between daemon and frontends. |
| `mousehop-proto` | Wire format of events sent between peers. Bump the version when it changes. |
| `mousehop-input-capture` | Backends that capture input: layer-shell (wlroots), libei (GNOME/KDE), X11, macOS, Windows. |
| `mousehop-input-emulation` | Backends that replay input: wlroots, libei, remote desktop portal, X11, macOS, Windows. |
| `mousehop-input-event` | Shared event and scancode types. |

Backends are chosen with Cargo features in the root `Cargo.toml` (see the Development section of the README).

## Data flow

1. The sending machine's capture backend sees an event when the pointer crosses a screen edge that is linked to a peer.
2. The daemon encodes the event (`mousehop-proto`) and sends it as UDP, encrypted with DTLS (WebRTC.rs).
3. Connection requests and pairing use TCP on the same port. Peers pin each other's certificate fingerprints. The listen port is set in the config.
4. The receiving daemon hands the event to its emulation backend. It tracks pressed keys and releases them when the peer disconnects.
5. The settings window and CLI talk to the daemon over a local socket (`mousehop-ipc`).

Peers can find each other with mDNS (`mdns-sd`). There is no central server and no telemetry.

## External services

None at run time. Build and release use GitHub Actions, crates.io, the AUR, Cachix (Nix cache), a Flatpak bundle and Apple notarisation.

## Environment variables (names only)

Runtime: `MOUSEHOP_LOG_LEVEL`, `MOUSEHOP_SOCKET_NAME`, `MOUSEHOP_HIDDEN`, `MOUSEHOP_TRAY`, `MOUSEHOP_DISABLE_TCC_WATCH` (macOS).

Release secrets (GitHub): `RELEASE_PLZ_TOKEN`, `CARGO_REGISTRY_TOKEN`, `AUR_SSH_KEY`, `MACOS_CERTIFICATE_P12_BASE64`, `MACOS_CERTIFICATE_PASSWORD`, `MACOS_NOTARY_APPLE_ID`, `MACOS_NOTARY_TEAM_ID`, `MOUSEHOP_SIGNING_PASSWORD`, `CACHIX_AUTH_TOKEN`, `SITE_DEPLOY_KEY` (used by `release-update-site.yml`).

## Release pipeline

See [RELEASING.md](../RELEASING.md). The workflows `release-plz`, `release`, `release-flatpak`, `aur-publish`, `release-update-metainfo`, `release-update-site` and `workflow-drift-check` are kept the same across several `jondkinney/*` repos on purpose; do not edit them in one repo only.
