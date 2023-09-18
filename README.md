# 🥧 Hashcalc

[![CI](https://github.com/Antidote1911/hashcalc/actions/workflows/ci.yml/badge.svg)](https://github.com/Antidote1911/hashcalc/actions/workflows/ci.yml)
Collection of file hashing algorithms all bundled in a single tool.

Latest Windows, Linux or other release are [here](https://github.com/Antidote1911/hashcalc/releases/latest).

## Build

Have the latest `rustup`, `rust` toolchain and `cargo` installed and run:

```sh
cargo build
```

## Usage and examples

Run the binary with the `--help` flag to get detailed instructions.

No argument return Blake3 hash by default :
```sh
./hashcalc myfile.mp4
a73d16389457836d4239862102105cf08b31202e0e0fe35da9289fa722b2b66d  myfile.mp4
```
Want sha3-256, add the algo name to output, export result on a txt file:
```sh
./hashcalc -p -a sha3-256 myfile.mp4 >> myfile.txt
sha3-256 6a1a98b457b2dd56893639040bb53bc8fd6edb24a4feedde043bf050854d908f  myfile.mp4
```
Want to check all hash written in a txt file:
```sh
./hashcalc check myfile.txt
myfile.mp4 OK
```
The txt file is in the same folder as the file to check. It contains one hash by line:
```sh
sha3-256 6a1a98b457b2dd56893639040bb53bc8fd6edb24a4feedde043bf050854d908f  myfile.mp4
blake3 a73d16389457836d4239862102105cf08b31202e0e0fe35da9289fa722b2b66d myfile.mp4
```

## License

This project is licensed under the [AGPL-3.0 License](LICENSE) (or
<https://www.gnu.org/licenses/agpl-3.0.html>).
