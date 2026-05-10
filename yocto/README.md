# Building TOBI Board Images

TI documents Processor SDK Linux Yocto builds using `oe-layersetup`. For the current Sitara SDK `12.00.00.07.04` board set, the non-Chromium layer config is:

```text
configs/processor-sdk/processor-sdk-master-12.00.00.07.04-config.txt
```

Use Ubuntu 22.04 or TI's Yocto container for repeatable builds. On Apple silicon, the included ARM64 Ubuntu Docker flow builds natively without Rosetta. TI notes that full SDK image builds can require very large disk space; `tobi-initramfs` should be much smaller, but the BSP checkout and shared state are still substantial.

## Supported Board Matrix

| Board | Yocto `MACHINE` | Notes |
| --- | --- | --- |
| SK-AM62P-LP | `am62pxx-evm` | Initial hardware target |
| SK-AM62-LP | `am62xx-lp-evm` | AM62x low-power starter kit |
| SK-AM62-SIP | `am62xxsip-evm` | AM62x SIP starter kit |
| SK-AM62B | `am62xx-evm` | AM62x starter kit family |
| BeaglePlay | `beagleplay-ti` | BeagleBoard.org AM62x single-board computer |
| SK-AM62A-LP | `am62axx-evm` | AM62A Edge AI starter kit |
| TMDS62LEVM | `am62lxx-evm` | AM62L evaluation module |
| SK-AM64B | `am64xx-evm` | AM64x starter kit |
| TMDS64EVM | `am64xx-evm` | AM64x GP evaluation module; shares the AM64x Yocto machine |
| SK-AM68 | `am68-sk` | AM68 starter kit |
| SK-AM69 | `am69-sk` | AM69 starter kit |

## Layout

```text
tobi/       Rust application repo candidate
meta-tobi/  Yocto layer
yocto/      helper scripts and notes
```

## Build Outline

```sh
git clone https://git.ti.com/git/arago-project/oe-layersetup.git tisdk
cd tisdk
./oe-layertool-setup.sh -f configs/processor-sdk/processor-sdk-master-12.00.00.07.04-config.txt
cd build
. conf/setenv
bitbake-layers add-layer /absolute/path/to/meta-tobi
```

The initramfs defaults to the public catalog hosted by the TOBI repository:

```text
https://raw.githubusercontent.com/Grippy98/TOBI/master/tobi/sample/catalog.json
```

Override it with `TOBI_MANIFEST_URL` in the initramfs environment, or with the kernel argument:

```text
tobi.manifest=https://example.com/catalog.json
```

No downloadable-image catalog is embedded into the initramfs. If the hosted catalog cannot be reached at runtime, TOBI keeps running and offers only the local custom-image flow.

If the network is present but a proxy is required, the TUI recovery flow asks the user to set UTC system time first, then enter the proxy URL before retrying the hosted catalog.

To force that proxy/time path for board testing, add this kernel argument:

```text
tobi.test_proxy_setup=1
```

Build or provide an AArch64 Linux `tobi` binary and point Yocto at it:

```sh
echo 'TOBI_PREBUILT = "/absolute/path/to/out/aarch64-linux/tobi"' >> conf/local.conf
MACHINE=am62pxx-evm bitbake tobi-initramfs
```

Set `MACHINE` to any entry from the board matrix to build that board's initramfs or SD-card image.

On x86_64 Linux hosts, the recommended Docker flow keeps BitBake native to the host and only cross-compiles the standalone `tobi` app to AArch64:

```sh
./yocto/scripts/build-tobi-ubuntu-x86_64.sh
```

This writes:

```text
out/aarch64-linux/tobi
```

Cargo dependencies and build outputs are cached under `out/.cache/ubuntu-x86_64-tobi-cross` so repeat builds are faster. Override `OUT_DIR`, `CACHE_DIR`, `IMAGE`, or `TARGET` if you want different paths, a different local builder tag, or another Rust target.

Use that file as `TOBI_PREBUILT` for Yocto integration.

The full initramfs can be built from an x86_64 Linux host with:

```sh
./yocto/scripts/build-tobi-initramfs-ubuntu-x86_64.sh
```

This writes copied artifacts to:

```text
out/yocto/tobi-initramfs-am62pxx-evm.rootfs.cpio.xz
```

To build a user-flashable two-part SD-card image:

```sh
./yocto/scripts/build-tobi-sd-image-ubuntu-x86_64.sh
```

To build one specific board:

```sh
MACHINE=am62axx-evm ./yocto/scripts/build-tobi-sd-image-ubuntu-x86_64.sh
```

To build every currently defined board image:

```sh
./yocto/scripts/build-tobi-all-sd-images-ubuntu-x86_64.sh
```

The all-board script builds regular `tobi-sd-image` artifacts for every machine except `am62xxsip-evm`; that board is built as `tobi-lite-sd-image`.

The x86_64 helper keeps the TI SDK checkout, downloads, sstate cache, and build home in Docker named volumes prefixed with `tobi-x86_64-yocto-`.

### TOBI-lite For SK-AM62-SIP

`SK-AM62-SIP` / `am62xxsip-evm` can be built with the experimental **TOBI-lite** target:

```sh
./yocto/scripts/build-tobi-lite-sd-image-ubuntu-x86_64.sh
```

TOBI-lite keeps HDMI output enabled and uses `tobi.lite=1`, `tobi.ttys=/dev/tty0,/dev/ttyS2`, and `cma=32M` in `uEnv.txt`. HDMI runs the only TOBI app by default; the serial console prints a lightweight prompt explaining that typing `SERIAL` stops the HDMI instance and starts TOBI on serial instead. It keeps the AM62-SIP DTB (`k3-am6254atl-sk.dtb`) and prunes the initramfs module tree to the eMMC/SD, USB storage/HID, FAT/ext4, CPSW Ethernet, and DRM/TIDSS HDMI modules needed for installer use.

The xz memory guard is intentionally relaxed in TOBI-lite so `.wic.xz` images can be tested on 256 MiB boards. The UI reports the gzip-sized working-set estimate for xz and does not block flashing on that estimate. Treat this as a hardware validation mode; if xz proves unreliable, publish AM62-SIP catalog entries as `.wic.gz` or low-window `.wic.zst`.

On Apple silicon, the Rust app can be built inside native ARM64 Ubuntu Docker without Rosetta:

```sh
./yocto/scripts/build-tobi-ubuntu-arm64.sh
```

This writes:

```text
out/aarch64-linux/tobi
```

Cargo dependencies and build outputs are cached under `out/.cache/ubuntu-arm64-tobi` so repeat builds are faster. Override `OUT_DIR`, `CACHE_DIR`, or `IMAGE` if you want different paths or a different local builder tag.

Use that file as `TOBI_PREBUILT` for the first Yocto integration test.

The full initramfs can also be built from Apple silicon inside native ARM64 Ubuntu Docker:

```sh
./yocto/scripts/build-tobi-initramfs-ubuntu-arm64.sh
```

This writes copied artifacts to:

```text
out/yocto/tobi-initramfs-am62pxx-evm.rootfs.cpio.xz
```

The helper keeps the TI SDK checkout, downloads, sstate cache, and build home in Docker named volumes. That avoids BitBake's `TMPDIR` case-sensitivity check on macOS/APFS and keeps repeat builds incremental.

To build a user-flashable two-part SD-card image:

```sh
./yocto/scripts/build-tobi-sd-image-ubuntu-arm64.sh
```

To build one specific board:

```sh
MACHINE=am62axx-evm ./yocto/scripts/build-tobi-sd-image-ubuntu-arm64.sh
```

To build every currently defined board image:

```sh
./yocto/scripts/build-tobi-all-sd-images-ubuntu-arm64.sh
```

The ARM64 all-board script follows the same rule: regular images for all machines except `am62xxsip-evm`, which is built as TOBI-lite.

This writes:

```text
out/yocto/tobi-sd-image-<machine>.rootfs.wic.xz
out/yocto/tobi-sd-image-<machine>.rootfs.wic.bmap
```

The Apple silicon TOBI-lite wrapper is:

```sh
./yocto/scripts/build-tobi-lite-sd-image-ubuntu-arm64.sh
```

It writes `tobi-lite-*am62xxsip-evm*` artifacts under `out/yocto`.

The first successful SK-AM62P-LP build produced a 24 MiB compressed initramfs, 102 MiB uncompressed:

```sh
xz -l out/yocto/tobi-initramfs-am62pxx-evm.rootfs.cpio.xz
```

The expected output is a compressed initramfs under the TI deploy directory, usually:

```text
deploy-ti/images/am62pxx-evm/tobi-initramfs-am62pxx-evm.cpio.xz
```

## Next Integration Work

1. Build a target `tobi` binary through a Yocto-native Rust recipe or through a cross-build job.
2. Add U-Boot/FIT packaging so supported boards can boot kernel + DTB + TOBI initramfs fully into RAM from eMMC.
3. Add USB/SD automount handling before TOBI starts, so custom local images are visible.

## License

TOBI is licensed under GPL v2 only (`GPL-2.0-only`).
