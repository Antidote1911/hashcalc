# Hashcalc

[![Release](https://github.com/Antidote1911/hashcalc/actions/workflows/release.yml/badge.svg)](https://github.com/Antidote1911/hashcalc/actions/workflows/release.yml)

Collection of file hashing algorithms bundled in a single tool, available as a CLI and a graphical interface.

Latest releases for Windows, Linux and macOS are [here](https://github.com/Antidote1911/hashcalc/releases/latest).

## Workspace structure

```
hashcalc/
├── core/   # shared hashing library
├── cli/    # hashcalc — command-line tool
└── gui/    # hashcalc-gui — egui graphical interface
```

## Supported algorithms

| Family | Algorithms |
|---|---|
| Blake | `blake3` (default), `blake2s`, `blake2b` |
| SHA-3 / Keccak | `sha3-512`, `sha3-384`, `sha3-256`, `sha3-224`, `keccak512`, `keccak384`, `keccak256`, `keccak224` |
| SHA-2 | `sha2-512`, `sha2-512-256`, `sha2-512-224`, `sha2-384`, `sha2-256`, `sha2-224` |
| Legacy | `sha1`, `md5`, `md4`, `md2` |
| FSB | `fsb-512`, `fsb-384`, `fsb-256`, `fsb-224`, `fsb-160` |
| GOST 94 | `gost94-test`, `gost94-cryptopro`, `gost94-s2015`, `gost94-ua` |
| Groestl | `groestl-512`, `groestl-384`, `groestl-256`, `groestl-224` |
| RIPEMD | `ripemd320`, `ripemd256`, `ripemd160`, `ripemd128` |
| Shabal | `shabal-512`, `shabal-384`, `shabal-256`, `shabal-224`, `shabal-192` |
| Other | `sm3`, `streebog-512`, `streebog-256`, `tiger`, `tiger2`, `whirlpool` |

## GUI

`hashcalc-gui` is a native desktop application built with [egui](https://github.com/emilk/egui).

- Drag-and-drop a file onto the window or click to browse
- Select any algorithm from the full list via a dropdown
- Real-time progress bar during computation (threaded, non-blocking)
- One-click copy of the hash result to the clipboard
- Dark / light mode toggle

## CLI usage

Run the binary with `--help` to get the full list of options.

Hash a file with the default algorithm (blake3):
```sh
hashcalc myfile.mp4
a73d16389457836d4239862102105cf08b31202e0e0fe35da9289fa722b2b66d  myfile.mp4
```

Choose an algorithm and prepend its name to the output:
```sh
hashcalc -p -a sha3-256 myfile.mp4
sha3-256 6a1a98b457b2dd56893639040bb53bc8fd6edb24a4feedde043bf050854d908f  myfile.mp4
```

Hash multiple files at once (processed in parallel):
```sh
hashcalc -a blake2b file1.iso file2.iso
```

Redirect output to a hash file for later verification:
```sh
hashcalc -p -a sha3-256 myfile.mp4 >> myfile.txt
```

### Verifying hashes

Check all hashes listed in a text file:
```sh
hashcalc check myfile.txt
myfile.mp4 OK
```

The hash file must be in the same directory as the files it references. Each line contains one entry:
```
sha3-256 6a1a98b457b2dd56893639040bb53bc8fd6edb24a4feedde043bf050854d908f  myfile.mp4
blake3 a73d16389457836d4239862102105cf08b31202e0e0fe35da9289fa722b2b66d myfile.mp4
```

Lines without an algorithm prefix are read using blake3 by default, making the format compatible with standard tools like `sha256sum`.

### Shell completions

Generate completions for your shell and source them:
```sh
hashcalc completions bash > /etc/bash_completion.d/hashcalc
hashcalc completions zsh  > /usr/share/zsh/site-functions/_hashcalc
hashcalc completions fish > ~/.config/fish/completions/hashcalc.fish
```

### Man pages

Generate man pages into a directory:
```sh
hashcalc manpages /usr/local/share/man/man1/
```

## Build

Install the latest stable [Rust toolchain](https://rustup.rs/) and run:

```sh
cargo build --release
```

This produces two binaries under `target/release/`:
- `hashcalc` — CLI
- `hashcalc-gui` — GUI

### Linux system dependencies

The GUI requires the following libraries on Linux (Debian/Ubuntu):

```sh
sudo apt-get install \
  libgtk-3-dev \
  libxcb-render0-dev \
  libxcb-shape0-dev \
  libxcb-xfixes0-dev \
  libxkbcommon-dev \
  libssl-dev \
  libglib2.0-dev
```

To build only the CLI without these dependencies:
```sh
cargo build --release --bin hashcalc
```

## License

This project is licensed under the [AGPL-3.0 License](LICENSE) (or
<https://www.gnu.org/licenses/agpl-3.0.html>).
