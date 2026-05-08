SUMMARY = "TOBI - TI Out of Box Installer"
DESCRIPTION = "Terminal OS installer for Texas Instruments Sitara starter kit evaluation modules."
HOMEPAGE = "https://github.com/your-org/tobi"
LICENSE = "Apache-2.0"
LIC_FILES_CHKSUM = "file://${COMMON_LICENSE_DIR}/Apache-2.0;md5=89aea4e17d99a7cacdbeed46a0096b10"

TOBI_PREBUILT ?= ""

SRC_URI += "file://catalog.json"
S = "${UNPACKDIR}"

python () {
    if not d.getVar("TOBI_PREBUILT"):
        bb.fatal("TOBI_PREBUILT must point to a target-architecture tobi binary")
}

do_configure[noexec] = "1"
do_compile[noexec] = "1"

do_install() {
    install -d ${D}${bindir}
    install -m 0755 ${TOBI_PREBUILT} ${D}${bindir}/tobi
    install -d ${D}${sysconfdir}/tobi
    install -m 0644 ${UNPACKDIR}/catalog.json ${D}${sysconfdir}/tobi/catalog.json
}

FILES:${PN} += "${bindir}/tobi ${sysconfdir}/tobi/catalog.json"
RDEPENDS:${PN} += "liblzma"
INSANE_SKIP:${PN} += "already-stripped"
