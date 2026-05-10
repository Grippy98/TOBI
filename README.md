# TOBI Hardware Image Workspace

**TOBI** is the **TI Out of Box Installer** for Texas Instruments Sitara starter kit evaluation modules. It boots a small RAM-resident Linux environment, presents a terminal UI, downloads a selected OS image, streams/decompresses it directly to target media, and reboots into the installed image.

The first hardware target was **SK-AM62P-LP** / Yocto machine `am62pxx-evm`.
TOBI now carries board definitions and catalog entries for these starter kits and EVMs:

| Board | Yocto `MACHINE` | Catalog SDK |
| --- | --- | --- |
| SK-AM62P-LP | `am62pxx-evm` | `PROCESSOR-SDK-LINUX-AM62P` |
| SK-AM62-LP | `am62xx-lp-evm` | `PROCESSOR-SDK-LINUX-AM62X` |
| SK-AM62-SIP | `am62xxsip-evm` | `PROCESSOR-SDK-LINUX-AM62X` |
| SK-AM62B | `am62xx-evm` | `PROCESSOR-SDK-LINUX-AM62X` |
| BeaglePlay | `beagleplay-ti` | `PROCESSOR-SDK-LINUX-AM62X` |
| SK-AM62A-LP | `am62axx-evm` | `PROCESSOR-SDK-LINUX-AM62A` |
| TMDS62LEVM | `am62lxx-evm` | `AM62L-LINUX-SDK` |
| SK-AM64B | `am64xx-evm` | `PROCESSOR-SDK-LINUX-AM64X` |
| TMDS64EVM | `am64xx-evm` | `PROCESSOR-SDK-LINUX-AM64X` |
| SK-AM68 | `am68-sk` | `PROCESSOR-SDK-LINUX-AM68` |
| SK-AM69 | `am69-sk` | `PROCESSOR-SDK-LINUX-AM69` |

The catalog also includes Armbian community downloads for supported boards that have matching Armbian board pages as a separate Community section.

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

The production image does not embed a downloadable-image catalog. If the board cannot reach the hosted catalog, TOBI falls back to local-image flashing only and asks the user to attach FAT32 media with a compatible image file.

When a proxy is needed, TOBI prompts for UTC system time before the proxy URL so TLS validation can succeed even if automatic time sync failed.

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

Running TOBI with no arguments starts the production path: `--mode live` with write permissions enabled. Use `--mode mock` for local UI testing; mock mode never writes to a real block device. Use `--no-allow-write` to inspect live device detection without allowing writes.

## Build With Docker

The TUI app can be built and run in Docker:

```sh
cd tobi
docker build -t tobi .
docker run --rm -it tobi
```

For quick local UI testing from this workspace, use the helper script. It runs
the app in mock mode, attaches it to your host terminal, and uses the same
x86_64 Docker builder/cache used by the Yocto helper scripts:

```sh
./scripts/run-tobi-local.sh tui
./scripts/run-tobi-local.sh serial
./scripts/run-tobi-local.sh tui --test-proxy-setup
```

The `tui` mode exercises the HDMI-style crossterm UI. The `serial` mode
exercises the line-oriented UART UI that the initramfs starts with
`--serial-ui`. The `--test-proxy-setup` flag simulates a board with DHCP and
a valid local IP where the online catalog is unreachable, then opens the UTC
time and proxy setup screen.

On an x86_64 Linux host, build natively and let Yocto cross-compile the ARM image:

```sh
./yocto/scripts/build-tobi-ubuntu-x86_64.sh
./yocto/scripts/build-tobi-sd-image-ubuntu-x86_64.sh
```

Build a specific board image by setting `MACHINE`:

```sh
MACHINE=am64xx-evm ./yocto/scripts/build-tobi-sd-image-ubuntu-x86_64.sh
```

Build all currently defined board images:

```sh
./yocto/scripts/build-tobi-all-sd-images-ubuntu-x86_64.sh
```

The all-board build produces regular TOBI images for every supported board except `am62xxsip-evm`, which is built as TOBI-lite.

The x86_64 flow cross-compiles the standalone `tobi` binary to AArch64, then runs BitBake as a native x86_64 process. This is the preferred Docker flow for x86_64 hosts.

On Apple silicon, both the Rust app and the TI Yocto image can be built in native ARM64 Ubuntu Docker without Rosetta:

```sh
./yocto/scripts/build-tobi-ubuntu-arm64.sh
./yocto/scripts/build-tobi-sd-image-ubuntu-arm64.sh
```

Build a specific board image by setting `MACHINE`:

```sh
MACHINE=am64xx-evm ./yocto/scripts/build-tobi-sd-image-ubuntu-arm64.sh
```

Build all currently defined board images:

```sh
./yocto/scripts/build-tobi-all-sd-images-ubuntu-arm64.sh
```

The all-board ARM64 build uses the same rule: regular TOBI for each board except `am62xxsip-evm`, which gets TOBI-lite.

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

Replace `am62pxx-evm` with another supported machine to generate that board's TOBI image.

The SD image is user-flashable. It boots TOBI into RAM and leaves the target eMMC free to be overwritten by the installer.

## TOBI-lite For AM62-SIP Low Memory Testing

`SK-AM62-SIP` / `am62xxsip-evm` has a 256 MiB RAM configuration, so this branch also carries **TOBI-lite** image targets for that board only. TOBI-lite still drives the HDMI framebuffer UI while trimming the rest of the installer environment before flashing eMMC.

The low-memory boot path uses:

```text
console=ttyS2,115200n8
console=tty0
tobi.ttys=/dev/tty0,/dev/ttyS2
tobi.lite=1
cma=32M
```

The initramfs keeps AM62 essentials for eMMC/SD, USB mass storage/HID, FAT/ext4 local media, CPSW Ethernet, and the DRM/TIDSS display path needed for HDMI. The lite recipe starts from TI `kernel-modules` and prunes the initramfs module tree to the AM62-SIP allowlist plus dependencies from `modules.dep`.

Only one TOBI app runs by default. HDMI owns the active installer; the serial console prints a lightweight prompt. If the user types `SERIAL`, the initramfs stops the HDMI TOBI process and starts TOBI on the serial console instead. HDMI mode is not restarted after that handoff.

For hardware testing, TOBI-lite deliberately relaxes the `.wic.xz` RAM guard. Normal TOBI budgets xz images conservatively, but TOBI-lite reports an xz working set at the gzip-sized estimate and does not block flashing solely on that estimate. This is to verify whether the existing TI `.wic.xz` images can actually stream on 256 MiB hardware; if they are unstable, the catalog should publish AM62-SIP images as `.wic.gz` or low-window `.wic.zst`.

Build the TOBI-lite SD image with:

```sh
./yocto/scripts/build-tobi-lite-sd-image-ubuntu-x86_64.sh
```

On Apple silicon:

```sh
./yocto/scripts/build-tobi-lite-sd-image-ubuntu-arm64.sh
```

Expected copied artifacts use the `tobi-lite-*` basename, for example:

```text
out/yocto/tobi-lite-initramfs-am62xxsip-evm.rootfs.cpio.xz
out/yocto/tobi-lite-sd-image-am62xxsip-evm.rootfs.wic.xz
out/yocto/tobi-lite-sd-image-am62xxsip-evm.rootfs.wic.bmap
```
