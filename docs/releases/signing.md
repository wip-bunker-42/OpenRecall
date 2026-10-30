# Release signing and packaging

OpenRecall uses Tauri's official updater with an embedded Minisign public key and
`requireSignedVersion: true`. Package bytes and their announced semantic version
must both verify. An unsigned manifest cannot substitute an older signed package
under a newer version number. Release notes are rendered as plain text.

Updating is explicit: checking, downloading, and installing are separate actions
in Settings. Startup does not check the network. Failed checks or downloads do not
disable offline study. Downloaded bytes stay in native memory and are installable
only after verification. Closing the app discards the download; it can be fetched
again. There is no production command accepting an update URL or arbitrary path.

Before installation the native layer holds the database mutex, rejects unsaved
edits and active/resumable review sessions, polls completed training workers into
durable records, blocks if training is still running, and checkpoints SQLite's
WAL. New study writes are blocked once installation starts. Windows' official
updater launches the installer and exits; macOS/Linux explicitly restart after
successful installation. Installation requires a separate confirmation.

## Credentials

The initial encrypted private key and password were created outside the worktree
in `%USERPROFILE%\.codex\credentials\openrecall\`, with inheritance removed and
access granted to the current Windows user. Files are `updater.key`,
`updater.key.pub`, and `updater-password.txt`. Only the public key belongs in Git.
The password and encrypted key are locally kept together; the directory ACL is
the local access boundary. Keep a separate secure backup, ideally in an encrypted
credential vault. The portable study backup deliberately excludes release keys.

GitHub release jobs require repository secrets `TAURI_SIGNING_PRIVATE_KEY` (key
contents) and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. Missing credentials fail the
release. Pull request quality jobs do not receive these secrets and disable
updater artifact generation for their unsigned test packages.

Never put private values in command examples, logs, issue bodies, or source files.
Load them from the protected credential files or secret environment variables.
The public cryptographic fixture in `tests/fixtures/updates/` is inert text,
signed for version `0.0.0`; it contains no private key or executable update.

## Build and verify

Use the pinned Node, pnpm, Rust and Tauri versions documented in
`../development/toolchain.md`. Compile the production binary with no signing
credentials in its environment. Supply the two protected secrets only to each
subsequent `tauri bundle` invocation, then remove them in `finally` before staging
or verification. The release workflow uses the same split.

```powershell
Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY -ErrorAction SilentlyContinue
Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD -ErrorAction SilentlyContinue
pnpm tauri build --ci --target x86_64-pc-windows-msvc --no-bundle
```

After successful compilation, load the existing encrypted key and password from
the protected directory only around bundling. Bundle the normal NSIS package,
preserve it and its signature, then bundle with the offline configuration from
the same existing production binary:

```powershell
pnpm tauri bundle --ci --target x86_64-pc-windows-msvc --bundles nsis
# Preserve the normal installer and signature before the following command.
pnpm tauri bundle --ci --target x86_64-pc-windows-msvc --config src-tauri/tauri.offline.conf.json --bundles nsis
```

Both bundle invocations need the two signing variables; staging and signature
verification do not. Always clear the variables on success and failure. No new
key or repository secret is needed for 0.2.0.

Preserve the normal installer and its `.sig` before building the offline variant,
because both use the same default output basename. Rename the offline copy for
distribution. Its signature still covers identical bytes and the signed version.
The offline configuration embeds the Evergreen WebView2 installer. The normal
configuration downloads WebView2 during setup only if it is missing. Normal study
uses the locally installed WebView2 runtime and bundled assets.

Windows builds explicitly use Tauri's static Visual C++ runtime setting. Verify
the final executable imports rather than assuming the development environment
represents a clean Windows installation. Release builds omit `automation` and all
test-only data overrides. The application's own identifier stays the same for
normal/offline installers so updates retain the same study data.

Verify an artifact without installing or accessing the network:

```powershell
$releaseVersion = (Get-Content package.json -Raw | ConvertFrom-Json).version
cargo run -p openrecall-desktop --example verify_update -- PACKAGE PACKAGE.sig $releaseVersion windows-x86_64
```

Release compilation always checks out the immutable, validated tag. Publisher
policy uses the trusted workflow revision, so a manual dispatch from reviewed
main can repair publication behavior without moving an existing tag or changing
its application source. The publisher discovers owned drafts in the authenticated
release list when the published-release-by-tag endpoint returns 404; ambiguous,
unrelated or already published releases fail closed. Official artifact signature
and version verification remains required before publication.

The helper uses Tauri's official `Update::verify`, the public key embedded in the
checked-out configuration, and signed-version enforcement. Release automation
verifies every updater package before creating `latest.json` with
`scripts/release.mjs`. The manifest requires all four supported updater targets:
Windows x64, macOS ARM64, macOS x64, and Linux x64. Linux `.deb` and macOS `.dmg`
are additional install media; updater entries use the supported updater package.

## Publisher identity and rotation

Updater signatures do not provide a Windows Authenticode identity or Apple
notarization. Without paid/organization signing credentials, Windows installers
have no verified publisher certificate and macOS uses ad-hoc signing. Report this
distribution status with the release; do not describe it as OS notarization.

To rotate a key, publish a transition build containing the new public key while
signing that transition package with the previous private key. Back up both key
generations until migration is complete. The current single `latest.json`
endpoint does not guarantee that users who skip the transition can update
automatically; provide the official installer for those users. If a private key
is lost, existing clients cannot verify packages signed by a replacement key;
manual installation of a new trusted build is required. Study data and portable
backups are independent of release credentials.

Sources: [Tauri updater](https://v2.tauri.app/plugin/updater/),
[Windows packaging](https://v2.tauri.app/distribute/windows-installer/),
[build configuration](https://v2.tauri.app/reference/config/), and the pinned
`tauri-plugin-updater` 2.14.0 source/API, including signed-version validation.
