<!--
This page mirrors `apps/desktop-app/src/app/content/license.ts`, which is the canonical text and also
drives the License dialog in Settings → About inside the app (credits list included). A third, condensed
copy lives in `apps/desktop-app/src-tauri/license/PitCrew-EULA.rtf` — the one screen Windows' MSI
installer can show — and is not meant to be a line-for-line duplicate of this page, only a faithful
summary of the same terms plus a short privacy notice. Keep this page in sync with `license.ts` by
convention, the same way `docs/analytics.md` stays in sync with `what-pitcrew-sends.md`.

Convention is no longer the whole of it for the *Voice models* section: `scripts/check-license-copies.mjs`
fails the PR when the four copies disagree about which voices are credited. That gate exists because this
page was two attributions ahead of the two copies a paying driver actually reads (GOT-475).
-->

# License

> **Draft for legal review.** This reflects the terms Igor has confirmed in principle — free to use,
> as-is, no redistribution — but has not been reviewed by a lawyer and is not yet a finished legal
> document. The bracketed `[governing law]` section is deliberately unfilled until that review answers
> it.

Effective date: *[effective date — set when this leaves draft]*

PitCrew is **proprietary freeware**, not open source. It is free to use; it is not free to redistribute,
resell, modify, or reverse-engineer. `src-tauri/Cargo.toml`'s `license = "MIT"` field describes the Rust
crate's own metadata, left over from `cargo init` — it is not this agreement, and does not describe the
terms the shipped application is used under.

## 1. License grant

Subject to the terms of this agreement, Igor Gottschalg grants you a personal, non-exclusive,
non-transferable, revocable license to install and use PitCrew on your own computer(s), free of charge,
for as long as this license remains in effect.

## 2. Ownership

PitCrew — its source code, its compiled binary, its name, its icons and its phrase catalog — is and
remains the property of Igor Gottschalg. This license grants you the right to run the software; it does
not transfer any ownership of it to you.

Third-party components distributed with PitCrew remain the property of their respective owners and are
licensed under their own terms — see the credits list below for what is included and under which
license.

## 3. What you may not do

Without separate written permission from Igor Gottschalg, you may not:

- Redistribute, sell, rent, lease, or sublicense the PitCrew installer or binary, in whole or in part.
- Modify, adapt, translate, or create derivative works of PitCrew.
- Reverse-engineer, decompile, or disassemble PitCrew, except to the extent that applicable law expressly
  permits this despite this restriction.
- Remove or obscure any copyright, trademark, or other proprietary notice from PitCrew.
- Use PitCrew's name, icons, or branding to represent a modified or redistributed copy as official.

## 4. "As-is", no warranty

PitCrew is provided "as-is" and "as-available", without warranty of any kind, express or implied,
including without limitation the implied warranties of merchantability, fitness for a particular
purpose, and non-infringement. PitCrew reads live telemetry from a race simulator and speaks generated
audio during a live session — Igor Gottschalg does not warrant that it will be error-free, uninterrupted,
or fit for competitive use, and you use it at your own risk.

## 5. Limitation of liability

To the maximum extent permitted by applicable law, in no event will Igor Gottschalg be liable for any
indirect, incidental, special, consequential, or punitive damages, or any loss of data, arising out of or
related to your use of, or inability to use, PitCrew — including any outcome in a race or session where
PitCrew was running.

## 6. Third-party components

PitCrew is built on open-source and third-party software, each under its own license. The list below is
the set of direct dependencies whose license requires PitCrew to keep an attribution notice — not every
direct dependency shipped in the application, only the ones with something to credit (also reachable from
Settings → About → License inside the app). Nothing in this agreement overrides a more permissive or more
restrictive term that a listed component's own license imposes on that component specifically.

Voice models are separate from all of the above and are not distributed with PitCrew. When you choose a
voice, PitCrew downloads that model from its publisher to your own computer; it is licensed to you by
whoever published it, under that model's own terms, and Igor Gottschalg grants you no rights in it. The
[voice models](#voice-models) list below credits the ones whose license requires an attribution — the rest
ask for nothing.

### JavaScript / npm

| Component | Version | License |
| --- | --- | --- |
| @fontsource/commit-mono | 5.3.0 | OFL-1.1 |
| @fontsource/inter | 5.3.0 | OFL-1.1 |
| @fontsource/jetbrains-mono | 5.3.0 | OFL-1.1 |
| @fontsource/martian-mono | 5.3.0 | OFL-1.1 |
| @fontsource/space-grotesk | 5.3.0 | OFL-1.1 |
| @radix-ui/react-dialog | 1.1.23 | MIT |
| @radix-ui/react-label | 2.1.15 | MIT |
| @radix-ui/react-select | 2.3.7 | MIT |
| @radix-ui/react-slider | 1.4.7 | MIT |
| @radix-ui/react-slot | 1.3.3 | MIT |
| @radix-ui/react-switch | 1.3.7 | MIT |
| @sentry/react | 8.55.2 | MIT |
| @tauri-apps/api | 2.11.1 | Apache-2.0 OR MIT |
| @tauri-apps/plugin-opener | 2.5.4 | MIT OR Apache-2.0 |
| class-variance-authority | 0.7.1 | Apache-2.0 |
| clsx | 2.1.1 | MIT |
| jotai | 2.20.2 | MIT |
| lucide-react | 1.27.0 | ISC |
| react | 19.2.8 | MIT |
| react-dom | 19.2.8 | MIT |
| react-router-dom | 7.18.1 | MIT |
| tailwind-merge | 3.6.0 | MIT |

### Rust

Direct dependencies only — not the full transitive tree, which is hundreds of crates. `clap` and
`indicatif` are declared in `Cargo.toml` but excluded here: both back the offline `pregen` developer tool
behind a feature flag, and neither is compiled into the shipped application.

Narrowed further to licenses that actually require attribution — `blake3` is shipped but not listed: its
license is `CC0-1.0 OR Apache-2.0 OR Apache-2.0 WITH LLVM-exception`, and PitCrew relies on the CC0-1.0
option, which is public domain and requires nothing kept. Nothing else here offers an attribution-free
option.

| Component | Version | License |
| --- | --- | --- |
| thiserror | 2.0.19 | MIT OR Apache-2.0 |
| serde | 1.0.229 | MIT OR Apache-2.0 |
| serde_json | 1.0.151 | MIT OR Apache-2.0 |
| serde_yaml | 0.9.34 | MIT OR Apache-2.0 |
| tauri | 2.11.5 | Apache-2.0 OR MIT |
| ort | 2.0.0-rc.12 | MIT OR Apache-2.0 |
| espeak-rs | 0.2.0 | MIT |
| audiopus | 0.2.0 | ISC |
| rodio | 0.21.1 | MIT OR Apache-2.0 |
| tauri-plugin-global-shortcut | 2.3.2 | Apache-2.0 OR MIT |
| tauri-plugin-autostart | 2.5.1 | Apache-2.0 OR MIT |
| ureq | 2.12.1 | MIT OR Apache-2.0 |
| native-tls | 0.2.18 | MIT OR Apache-2.0 |
| sentry | 0.32.3 | Apache-2.0 |
| tauri-plugin-opener | 2.5.4 | Apache-2.0 OR MIT |
| axum | 0.8.9 | MIT |
| tokio | 1.53.1 | MIT |
| rust-embed | 8.12.0 | MIT |
| if-addrs | 0.15.0 | MIT OR BSD-3-Clause |
| qrcode | 0.14.1 | MIT OR Apache-2.0 |
| mdns-sd | 0.20.3 | Apache-2.0 OR MIT |
| gethostname | 1.1.0 | Apache-2.0 |
| tokenizers | 0.23.1 | Apache-2.0 |
| sha2 | 0.10.9 | MIT OR Apache-2.0 |
| tauri-plugin-dialog | 2.7.2 | Apache-2.0 OR MIT |

### Voice models

Voice models are **not part of the installer** — PitCrew downloads the ones you choose, and until you
choose one, nothing here is on your PC. They carry their own licenses, separate from PitCrew's and from
the libraries above.

Only the ones that ask for something are listed. Most are CC0 or public domain, which require nothing
kept; these are the ones that do:

| Voice | Where it comes from | License |
| --- | --- | --- |
| Siwis (French) | [SIWIS French Speech Synthesis Database](https://datashare.is.ed.ac.uk/handle/10283/2353), University of Edinburgh | CC BY 4.0 — attribution required |
| Sam (English) | [Non-Binary Voice Files](https://github.com/Sam-Accenture-Non-Binary-Voice/non-binary-voice-files), Accenture | Apache-2.0 — attribution required |
| Parakeet TDT 0.6B v3 (speech recognition) | [NVIDIA Parakeet TDT 0.6B v3](https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3), ONNX export by [sherpa-onnx (csukuangfj)](https://huggingface.co/csukuangfj/sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8) | CC BY 4.0 — attribution required |

## 7. Privacy

PitCrew collects usage data as described in the [Privacy Policy](privacy-policy.md), reachable from
Settings → About → Privacy inside the app. That policy is incorporated into this agreement by reference:
using PitCrew means you have had the opportunity to read it.

## 8. Termination

This license is effective until terminated. It terminates automatically, without notice, if you fail to
comply with any of its terms. Upon termination you must stop using PitCrew and delete any copies of it in
your possession.

## 9. Governing law

This agreement is governed by the laws of *[governing law / jurisdiction]*, without regard to its
conflict-of-law provisions.

## 10. Changes to this license

Igor Gottschalg may update this license in a future release. Continuing to use a version of PitCrew means
you accept the license shipped with that version; a materially different license will be called out in
that release's notes.

## 11. Contact

Questions about this license: [open an issue](reporting-issues.md) on the public repository.
