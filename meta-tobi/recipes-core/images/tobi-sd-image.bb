SUMMARY = "TOBI bootable SD-card image"
DESCRIPTION = "Flashable two-part SD-card image that boots the TOBI initramfs into RAM."
LICENSE = "GPL-2.0-only"
LIC_FILES_CHKSUM = "file://${COMMON_LICENSE_DIR}/GPL-2.0-only;md5=801f80980d171dd6425610833a22dbe6"

inherit core-image

IMAGE_FEATURES = ""
IMAGE_LINGUAS = ""

IMAGE_FSTYPES = "wic.xz wic.bmap"
WKS_FILE = "sdimage-2part.wks"
WKS_FILES = "${WKS_FILE}"

PACKAGE_INSTALL = "\
    base-files \
    base-passwd \
    busybox \
"

IMAGE_ROOTFS_SIZE = "65536"

TOBI_INITRAMFS_IMAGE = "tobi-initramfs-${MACHINE}.rootfs.cpio.xz"

IMAGE_BOOT_FILES += "\
    Image \
    k3-am62p5-sk.dtb;dtb/ti/k3-am62p5-sk.dtb \
    ${TOBI_INITRAMFS_IMAGE};uInitrd \
    tobi-uEnv.txt;uEnv.txt \
"

do_image_wic[depends] += "\
    tobi-bootfiles:do_deploy \
    tobi-initramfs:do_image_complete \
    virtual/kernel:do_deploy \
"

export IMAGE_BASENAME = "tobi-sd-image"
