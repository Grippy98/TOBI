# TOBI Hardware Image Workspace

This workspace keeps the Rust application and Yocto integration separate:

```text
tobi/       standalone Rust TUI application; can become its own git repo
meta-tobi/  Yocto layer for packaging TOBI into a RAM installer image
yocto/      build notes and helper scripts for TI Processor SDK Linux
```

The first hardware target is **SK-AM62P-LP** / Yocto machine `am62pxx-evm`.

The Rust app can be split out later with:

```sh
git subtree split --prefix=tobi -b tobi-app
```

On Apple silicon, both the Rust app and the first TI Yocto initramfs can be built in native ARM64 Ubuntu Docker without Rosetta:

```sh
./yocto/scripts/build-tobi-ubuntu-arm64.sh
./yocto/scripts/build-tobi-initramfs-ubuntu-arm64.sh
```

The Yocto helper uses Docker named volumes for the TI SDK checkout, downloads, sstate cache, and build home so BitBake runs on a Linux filesystem instead of macOS' default case-insensitive filesystem.
