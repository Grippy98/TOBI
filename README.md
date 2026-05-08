# TOBI Hardware Image Workspace

**TOBI** is the **TI Out of Box Installer** for Texas Instruments Sitara starter kit evaluation modules. It boots a small RAM-resident Linux environment, presents a terminal UI, downloads a selected OS image, streams/decompresses it directly to target media, and reboots into the installed image.

The first hardware target is **SK-AM62P-LP** / Yocto machine `am62pxx-evm`.

## Layout

```text
tobi/       standalone Rust TUI application; can become its own git repo
meta-tobi/  Yocto layer for packaging TOBI into a RAM installer image
yocto/      build notes and helper scripts for TI Processor SDK Linux
```

The Rust app can be split out later with:

```sh
git subtree split --prefix=tobi -b tobi-app
```

## Hosted Catalog

The public OS catalog is hosted from this GitHub repository:

```text
https://raw.githubusercontent.com/Grippy98/TOBI/master/tobi/sample/catalog.json
```

TOBI uses that URL by default. For local testing or private catalogs, pass another source:

```sh
cargo run --manifest-path tobi/Cargo.toml -- --manifest /path/to/catalog.json --mode mock
cargo run --manifest-path tobi/Cargo.toml -- --manifest https://example.com/catalog.json --mode mock
```

The Yocto initramfs also uses the hosted catalog by default. It can be overridden at boot with:

```text
tobi.manifest=https://example.com/catalog.json
```

or by setting `TOBI_MANIFEST_URL` in the initramfs environment.

## License

TOBI is licensed under GPL v2 only (`GPL-2.0-only`). See [LICENSE](LICENSE).

## Build The TUI On Linux

Ubuntu 22.04 is the recommended Linux host baseline for this project.

```sh
sudo apt-get update
sudo apt-get install -y build-essential ca-certificates curl git pkg-config libssl-dev liblzma-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
. "$HOME/.cargo/env"
cd tobi
cargo test
cargo build --release
cargo run -- --mode mock
```

`mock` mode never writes to a real block device. Live writes are guarded by `--mode live --allow-write`.

## Build With Docker

The TUI app can be built and run in Docker:

```sh
cd tobi
docker build -t tobi .
docker run --rm -it tobi
```

On Apple silicon, both the Rust app and the TI Yocto image can be built in native ARM64 Ubuntu Docker without Rosetta:

```sh
./yocto/scripts/build-tobi-ubuntu-arm64.sh
./yocto/scripts/build-tobi-sd-image-ubuntu-arm64.sh
```

The Yocto helper uses Docker named volumes for the TI SDK checkout, downloads, sstate cache, and build home so BitBake runs on a Linux filesystem instead of macOS' default case-insensitive filesystem.

## Build The Yocto Image On Linux

Build or provide an AArch64 Linux `tobi` binary first. On an ARM64 Linux host:

```sh
cd tobi
cargo build --release
mkdir -p ../out/aarch64-linux
cp target/release/tobi ../out/aarch64-linux/tobi
```

Then set up TI Processor SDK Linux and add `meta-tobi`:

```sh
git clone https://git.ti.com/git/arago-project/oe-layersetup.git tisdk
cd tisdk
./oe-layertool-setup.sh -f configs/processor-sdk/processor-sdk-master-12.00.00.07.04-config.txt
cd build
. conf/setenv
bitbake-layers add-layer /absolute/path/to/meta-tobi
echo 'TOBI_PREBUILT = "/absolute/path/to/out/aarch64-linux/tobi"' >> conf/local.conf
MACHINE=am62pxx-evm bitbake tobi-sd-image
```

The expected deploy artifacts are:

```text
tobi-initramfs-am62pxx-evm.rootfs.cpio.xz
tobi-sd-image-am62pxx-evm.rootfs.wic.xz
tobi-sd-image-am62pxx-evm.rootfs.wic.bmap
```

The SD image is user-flashable. It boots TOBI into RAM and leaves the target eMMC free to be overwritten by the installer.
