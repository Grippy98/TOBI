SUMMARY = "TOBI RAM-resident installer initramfs"
DESCRIPTION = "Minimal initramfs image that runs TOBI entirely from RAM before flashing target media."
LICENSE = "GPL-2.0-only"
LIC_FILES_CHKSUM = "file://${COMMON_LICENSE_DIR}/GPL-2.0-only;md5=801f80980d171dd6425610833a22dbe6"

inherit core-image

IMAGE_FEATURES = ""
IMAGE_LINGUAS = ""
IMAGE_FSTYPES = "cpio.xz"

PACKAGE_INSTALL = "\
    base-files \
    base-passwd \
    busybox \
    ca-certificates \
    dosfstools \
    e2fsprogs \
    iproute2 \
    kmod \
    kernel-modules \
    mmc-utils \
    parted \
    tobi-initramfs-init \
    tobi-prebuilt \
    util-linux \
    xz \
    zstd \
"

IMAGE_ROOTFS_SIZE = "65536"

export IMAGE_BASENAME = "tobi-initramfs"
