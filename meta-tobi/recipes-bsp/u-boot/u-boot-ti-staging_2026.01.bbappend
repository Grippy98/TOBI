FILESEXTRAPATHS:prepend := "${THISDIR}/${PN}:"

# Start with BeaglePlay while the menu and media mappings are hardware-tested.
SRC_URI:append:beagleplay-ti = " \
    file://0001-board-beagleplay-add-TOBI-recovery-boot-menu.patch \
    file://0002-board-beagleplay-enable-IT66121-HDMI-boot-menu.patch \
"
