# Release Process

This document describes how to create a new release of LazyJira.

## Prerequisites

- Push access to the repository
- All CI checks passing on `master` branch

## Release Steps

### 1. Prepare the Release

1. Ensure all changes for the release are merged to `master`
2. Verify CI is passing: check the [CI workflow](https://github.com/jonbito/lazyjira/actions/workflows/ci.yml)
3. Update the version in `Cargo.toml`:

   ```toml
   [package]
   version = "X.Y.Z"
   ```

4. Commit the version bump:

   ```bash
   git add Cargo.toml
   git commit -m "chore: bump version to vX.Y.Z"
   git push origin master
   ```

### 2. Create and Push the Tag

Create a version tag and push it to trigger the release workflow:

```bash
git tag vX.Y.Z
git push origin vX.Y.Z
```

### 3. Monitor the Release

1. Watch the [Release workflow](https://github.com/jonbito/lazyjira/actions/workflows/release.yml)
2. The workflow will:
   - Build binaries for all target platforms
   - Generate shell and PowerShell installers
   - Create SHA256 checksums for all artifacts
   - Create a GitHub Release with all artifacts
   - Update the Homebrew formula in [jonbito/homebrew-tap](https://github.com/jonbito/homebrew-tap)

### 4. Verify the Release

After the workflow completes:

- [ ] Check [GitHub Releases](https://github.com/jonbito/lazyjira/releases) for the new release
- [ ] Verify all platform binaries are attached:
  - `lazyjira-aarch64-apple-darwin.tar.gz` (macOS ARM64)
  - `lazyjira-x86_64-apple-darwin.tar.gz` (macOS Intel)
  - `lazyjira-x86_64-unknown-linux-gnu.tar.gz` (Linux)
  - `lazyjira-x86_64-pc-windows-msvc.zip` (Windows)
- [ ] Verify installer scripts are attached:
  - `lazyjira-installer.sh` (Shell installer)
  - `lazyjira-installer.ps1` (PowerShell installer)
- [ ] Verify Homebrew installation works:

  ```bash
  brew update
  brew install jonbito/tap/lazyjira
  lazyjira --version
  ```

## Target Platforms

| Platform | Architecture | Target Triple |
|----------|--------------|---------------|
| macOS | ARM64 (Apple Silicon) | `aarch64-apple-darwin` |
| macOS | Intel (x86_64) | `x86_64-apple-darwin` |
| Linux | x86_64 | `x86_64-unknown-linux-gnu` |
| Windows | x86_64 | `x86_64-pc-windows-msvc` |

## Troubleshooting

### Release workflow failed

1. Check the [workflow logs](https://github.com/jonbito/lazyjira/actions/workflows/release.yml)
2. Common issues:
   - Build failures: Check for platform-specific compilation errors
   - Homebrew publish failed: Verify the `HOMEBREW_TAP_DEPLOY_KEY` secret is set

### Homebrew formula not updated

1. Check the `publish-homebrew-formula` job in the release workflow
2. Verify the [homebrew-tap repository](https://github.com/jonbito/homebrew-tap) has the new formula
3. Ensure the deploy key has write access to the tap repository

### Users report installation issues

1. Verify the release artifacts are downloadable
2. Check the SHA256 checksums match
3. Test the installation commands from a clean environment

## Version Numbering

LazyJira follows [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking changes to CLI interface or configuration
- **MINOR**: New features, backward compatible
- **PATCH**: Bug fixes, backward compatible

## cargo-dist

This project uses [cargo-dist](https://opensource.axo.dev/cargo-dist/) for release automation. The configuration is in `Cargo.toml` under `[workspace.metadata.dist]`.

To update cargo-dist configuration:

```bash
cargo dist init
```

This will regenerate the release workflow based on current settings.
