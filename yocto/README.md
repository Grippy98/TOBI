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

Build or provide an AArch64 Linux `tobi` binary and point Yocto at it:

```sh
echo 'TOBI_PREBUILT = "/absolute/path/to/out/aarch64-linux/tobi"' >> conf/local.conf
MACHINE=am62pxx-evm bitbake tobi-initramfs
```

Set `MACHINE` to any entry from the board matrix to build that board's initramfs or SD-card image.

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

This writes:

```text
out/yocto/tobi-sd-image-<machine>.rootfs.wic.xz
out/yocto/tobi-sd-image-<machine>.rootfs.wic.bmap
```

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
