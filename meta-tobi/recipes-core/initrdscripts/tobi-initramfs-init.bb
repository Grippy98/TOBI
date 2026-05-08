SUMMARY = "TOBI initramfs init script"
LICENSE = "GPL-2.0-only"
LIC_FILES_CHKSUM = "file://${COMMON_LICENSE_DIR}/GPL-2.0-only;md5=801f80980d171dd6425610833a22dbe6"

SRC_URI = "file://init"

S = "${UNPACKDIR}"

do_install() {
    install -d ${D}/
    install -m 0755 ${S}/init ${D}/init
}

FILES:${PN} += "/init"
