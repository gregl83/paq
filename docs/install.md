# Installing paq

Install paq with the shell installer, a manual download, Cargo, or Nix.

## Quick Install

The shell installer supports the Linux x86/x64 and macOS Intel/Apple Silicon ZIP assets
published by this repository. Windows users can use the manual downloads.
On Apple Silicon, the installer selects ARM64 even from a Rosetta terminal.
On Linux, an x86-64 kernel with 32-bit userspace selects the x86 binary.

The installer requires `curl`, `unzip`, and either `sha256sum` or `shasum`.
It verifies the release's SHA-256 checksum before installing.

```bash
curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/gregl83/paq/main/install.sh | sh
```

Installs to `~/.local/bin`; add it to your `PATH` if needed.

### Installer options

| Variable          | Default            | Purpose                                          |
| :---------------- | :----------------- | :----------------------------------------------- |
| `PAQ_VERSION`     | `latest`           | Release version, with or without the `v` prefix. |
| `PAQ_INSTALL_DIR` | `$HOME/.local/bin` | Absolute destination directory.                  |

To select a version or installation directory, set variables on the `sh` side of
the pipeline:

```bash
curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/gregl83/paq/main/install.sh | env PAQ_VERSION=2.0.0 PAQ_INSTALL_DIR="$HOME/.local/bin" sh
```

The installer needs write access to the destination. It does not invoke `sudo`
or edit shell profiles.

To inspect the script before running it:

```bash
curl --proto '=https' --tlsv1.2 -fsSL -o install.sh https://raw.githubusercontent.com/gregl83/paq/main/install.sh
less install.sh
sh install.sh --help
sh install.sh
```

For a reproducible setup, use an immutable commit in the script URL in place of
`main` and set `PAQ_VERSION` to the desired release.

### Updating and uninstalling

Run the installer again to upgrade, or set `PAQ_VERSION` to select another release.
Remove the installed `paq` executable to uninstall.

## Manual Download

Windows, macOS, and Ubuntu are supported.

1. **Download:** Go to the [Latest Release](https://github.com/gregl83/paq/releases) page and download the `.zip` archive matching your OS and Architecture.
2. **Extract:** Unzip the `.zip` archive to retrieve the `paq` binary.
3. **Install:** Make the `paq` binary executable (e.g., `chmod +x`) and move it to a directory in your system PATH.
4. **Verify:** Confirm installation by running `paq --version` from the Command Line Interface.

## Cargo Install

Requires the [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) package manager.

```bash
cargo install paq
```

### Install From Repository Clone (Unstable)

Not recommended due to instability of `main` branch in-between tagged releases.

1. Clone this repository.
2. Run `cargo install --path .` from repository root.

## Nix Flakes

Requires [nix](https://nix.dev/) and the `nix-command` [experimental feature](https://nixos.wiki/wiki/Flakes#Enable_flakes_temporarily) to be enabled.

```bash
nix profile add github:gregl83/paq
```

## Troubleshooting

- **Checksum download fails:** the release must include a matching `.zip.sha256`
  asset. Older releases, including `v1.5.0`, do not include these files; use manual
  downloads or Cargo for those versions.
- **Unsupported platform:** Linux ARM and musl systems currently have no matching
  release assets. Use Cargo to build for your system.
- **Binary cannot run:** Linux binaries depend on glibc and the runtime libraries
  of the release build environment. The installer runs `paq --version` before
  replacing an existing installation. On an incompatible system, build with Cargo.
- **Command not found after installation:** add the reported installation directory
  to your shell's PATH. For the default directory in Bash or Zsh, use
  `export PATH="$HOME/.local/bin:$PATH"` and persist it in your shell configuration.

For release publishing and installer tests, see the [release guide](release.md).
