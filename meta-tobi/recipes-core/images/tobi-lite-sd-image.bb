SUMMARY = "TOBI-lite bootable SD-card image"
DESCRIPTION = "AM62-SIP focused SD-card image that boots the serial-only TOBI-lite initramfs into RAM."
LICENSE = "GPL-2.0-only"
LIC_FILES_CHKSUM = "file://${COMMON_LICENSE_DIR}/GPL-2.0-only;md5=801f80980d171dd6425610833a22dbe6"

COMPATIBLE_MACHINE = "am62xxsip-evm"

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

IMAGE_ROOTFS_SIZE = "32768"

TOBI_INITRAMFS_IMAGE = "tobi-lite-initramfs-${MACHINE}.rootfs.cpio.xz"
TOBI_BOOT_DTB_FILES = "k3-am6254atl-sk.dtb"

IMAGE_BOOT_FILES += "\
    Image \
    ${@' '.join(['%s;dtb/ti/%s' % (dtb, dtb) for dtb in d.getVar('TOBI_BOOT_DTB_FILES').split()])} \
    ${TOBI_INITRAMFS_IMAGE};uInitrd \
    tobi-lite-uEnv.txt;uEnv.txt \
"

do_image_wic[depends] += "\
    tobi-lite-bootfiles:do_deploy \
    tobi-lite-initramfs:do_image_complete \
    virtual/kernel:do_deploy \
"

export IMAGE_BASENAME = "tobi-lite-sd-image"
