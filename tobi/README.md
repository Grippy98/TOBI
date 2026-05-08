# TOBI

**TI Out of Box Installer**: terminal OS installer prototype for Texas Instruments Sitara starter kit evaluation modules.

The first target board was **SK-AM62P-LP** using TI's Yocto machine name `am62pxx-evm`.
The catalog now includes SK-AM62P-LP, SK-AM62-LP, SK-AM62-SIP, SK-AM62B, SK-AM62A-LP, TMDS62LEVM, SK-AM64B, SK-AM68, and SK-AM69 entries.

## Catalog

TOBI uses the public GitHub-hosted catalog by default:

```text
https://raw.githubusercontent.com/Grippy98/TOBI/master/tobi/sample/catalog.json
```

Use `--manifest` to test a local or alternate catalog:

```sh
cargo run -- --manifest sample/catalog.json --mode mock
```

Mock mode defaults to SK-AM62P-LP. To preview another board's filtered OS list:

```sh
TOBI_MOCK_BOARD=sk-am64b cargo run -- --mode mock
```

## Run Locally

```sh
cargo run -- --mode mock
```

`mock` mode is the default and never writes to a real block device.

## Build On Linux

Ubuntu 22.04 host example:

```sh
sudo apt-get update
sudo apt-get install -y build-essential ca-certificates curl git pkg-config libssl-dev liblzma-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
. "$HOME/.cargo/env"
cargo test
cargo build --release
```

## Custom Images

TOBI includes a built-in **Custom image from attached media** option. It scans mounted media for:

```text
.wic.xz, .img.xz, .wic.zst, .img.zst, .wic.gz, .img.gz, .wic, .img, .raw, .bin
```

Default scan roots:

```text
macOS: /Volumes
Linux: /run/media, /media, /mnt, /var/run/media
```

For local testing or unusual mount layouts:

```sh
TOBI_CUSTOM_IMAGE_ROOTS="/path/to/images:/another/root" cargo run -- --mode mock
```

If the online catalog cannot be reached, TOBI stays open, warns the user, and still allows flashing a custom local image. Press `P` from the warning to enter a proxy URL and retry the catalog.

TOBI streams images directly to the target media. The full downloaded or local image does not need to fit into RAM; only the installer runtime, decompressor, and write buffers do. Before installing, TOBI checks the available RAM against an estimated working set and blocks the install if that working set cannot fit.

## Run In Docker

```sh
docker build -t tobi .
docker run --rm -it tobi
```

## Live Write Mode

Live mode is intentionally guarded:

```sh
sudo tobi \
  --manifest https://raw.githubusercontent.com/Grippy98/TOBI/master/tobi/sample/catalog.json \
  --mode live \
  --proxy http://proxy.example.com:8080 \
  --target /dev/mmcblk0 \
  --allow-write
```

The production Yocto image should run fully from initramfs before this mode is used.

## License

TOBI is licensed under GPL v2 only (`GPL-2.0-only`). See [LICENSE](LICENSE).
