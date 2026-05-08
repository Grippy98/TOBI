SUMMARY = "TOBI boot partition configuration"
DESCRIPTION = "Deploys U-Boot environment overrides for TOBI SD-card images."
LICENSE = "GPL-2.0-only"
LIC_FILES_CHKSUM = "file://${COMMON_LICENSE_DIR}/GPL-2.0-only;md5=801f80980d171dd6425610833a22dbe6"

SRC_URI = "file://uEnv.txt"
S = "${UNPACKDIR}"

inherit deploy

do_deploy() {
    install -Dm0644 ${S}/uEnv.txt ${DEPLOYDIR}/tobi-uEnv.txt
}

addtask deploy after do_install before do_build
