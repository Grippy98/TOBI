# TOBI

**TI Out of Box Installer**: terminal OS installer prototype for Texas Instruments Sitara starter kit evaluation modules.

The first target board is **SK-AM62P-LP** using TI's Yocto machine name `am62pxx-evm`.

## Run Locally

```sh
cargo run -- --manifest sample/catalog.json --mode mock
```

`mock` mode is the default and never writes to a real block device.

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

If the online catalog cannot be reached, TOBI stays open, warns the user, and still allows flashing a custom local image. Press `p` from the warning to enter a proxy URL and retry the catalog.

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
  --manifest https://example.com/ti-os-catalog.json \
  --mode live \
  --proxy http://proxy.example.com:8080 \
  --target /dev/mmcblk0 \
  --allow-write
```

The production Yocto image should run fully from initramfs before this mode is used.
