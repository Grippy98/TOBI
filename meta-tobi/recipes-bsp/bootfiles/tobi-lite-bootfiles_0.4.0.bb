SUMMARY = "TOBI-lite boot partition configuration"
DESCRIPTION = "Deploys low-memory U-Boot environment overrides for TOBI-lite SD-card images."
LICENSE = "GPL-2.0-only"
LIC_FILES_CHKSUM = "file://${COMMON_LICENSE_DIR}/GPL-2.0-only;md5=801f80980d171dd6425610833a22dbe6"

COMPATIBLE_MACHINE = "am62xxsip-evm"

SRC_URI = "file://uEnv-lite.txt"
S = "${UNPACKDIR}"

inherit deploy

do_deploy() {
    install -Dm0644 ${S}/uEnv-lite.txt ${DEPLOYDIR}/tobi-lite-uEnv.txt
}

addtask deploy after do_install before do_build
