SUMMARY = "TOBI boot partition configuration"
DESCRIPTION = "Deploys U-Boot environment overrides for TOBI SD-card images."
LICENSE = "MIT"
LIC_FILES_CHKSUM = "file://${COMMON_LICENSE_DIR}/MIT;md5=0835ade698e0bcf8506ecda2f7b4f302"

SRC_URI = "file://uEnv.txt"
S = "${UNPACKDIR}"

inherit deploy

do_deploy() {
    install -Dm0644 ${S}/uEnv.txt ${DEPLOYDIR}/tobi-uEnv.txt
}

addtask deploy after do_install before do_build
