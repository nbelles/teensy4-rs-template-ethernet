//! Ethernet support for the Teensy 4.1's onboard DP83825I PHY.
//!
//! Configures ENET clocks, RMII pin muxing, and PHY initialization
//! using register addresses from the i.MX RT1062 reference manual
//! and FNET's fnet_mimxrt_eth.c as reference.

use core::ptr;

/// DP83825I PHY address on the Teensy 4.1 (strapped to 0).
pub const PHY_ADDR: u8 = 0;

/// Locally-administered MAC address for the Teensy 4.1.
pub const MAC: [u8; 6] = [0x02, 0x00, 0x00, 0x00, 0x00, 0x01];

/// IPG clock frequency: ARM_FREQ / 4 = 600 MHz / 4 = 150 MHz.
pub const IPG_FREQ: u32 = 150_000_000;

// ── Register addresses ──────────────────────────────────────────────

const IOMUXC: u32 = 0x401F_8000;
const MUX_B0_14: u32 = IOMUXC + 0x174;
const MUX_B0_15: u32 = IOMUXC + 0x178;
const MUX_B1_04: u32 = IOMUXC + 0x18C;
const MUX_B1_05: u32 = IOMUXC + 0x190;
const MUX_B1_06: u32 = IOMUXC + 0x194;
const MUX_B1_07: u32 = IOMUXC + 0x198;
const MUX_B1_08: u32 = IOMUXC + 0x19C;
const MUX_B1_09: u32 = IOMUXC + 0x1A0;
const MUX_B1_10: u32 = IOMUXC + 0x1A4;
const MUX_B1_11: u32 = IOMUXC + 0x1A8;
const MUX_B1_14: u32 = IOMUXC + 0x1B4;
const MUX_B1_15: u32 = IOMUXC + 0x1B8;

const PAD_B1_04: u32 = IOMUXC + 0x37C;
const PAD_B1_05: u32 = IOMUXC + 0x380;
const PAD_B1_06: u32 = IOMUXC + 0x384;
const PAD_B1_07: u32 = IOMUXC + 0x388;
const PAD_B1_08: u32 = IOMUXC + 0x38C;
const PAD_B1_09: u32 = IOMUXC + 0x390;
const PAD_B1_10: u32 = IOMUXC + 0x394;
const PAD_B1_11: u32 = IOMUXC + 0x398;
const PAD_B1_14: u32 = IOMUXC + 0x3A4;
const PAD_B1_15: u32 = IOMUXC + 0x3A8;

const IOMUXC_B: u32 = IOMUXC + 0x400;
const ENET_IPG_CLK_RMII_SI: u32 = IOMUXC_B + 0x02C;
const ENET_MDIO_SI: u32 = IOMUXC_B + 0x030;
const ENET0_RXDATA_SI: u32 = IOMUXC_B + 0x034;
const ENET1_RXDATA_SI: u32 = IOMUXC_B + 0x038;
const ENET_RXEN_SI: u32 = IOMUXC_B + 0x03C;
const ENET_RXERR_SI: u32 = IOMUXC_B + 0x040;

const GPIO7_GDIR: u32 = 0x4200_4004;
const GPIO7_DR_SET: u32 = 0x4200_4084;
const GPIO7_DR_CLR: u32 = 0x4200_4088;

const CCM_CCGR1: u32 = 0x400F_C06C;

const PLL_ENET: u32 = 0x400D_80E0;
const PLL_ENET_SET: u32 = 0x400D_80E4;
const PLL_ENET_CLR: u32 = 0x400D_80E8;

const GPR1: u32 = 0x400A_C004;

const RMII_PAD_INPUT_PULLDOWN: u32 = 0x30E9;
const RMII_PAD_INPUT_PULLUP: u32 = 0xB0E9;
const RMII_PAD_CLOCK: u32 = 0x0031;

unsafe fn wreg(addr: u32, val: u32) {
    ptr::write_volatile(addr as *mut u32, val);
}

unsafe fn rreg(addr: u32) -> u32 {
    ptr::read_volatile(addr as *const u32)
}

unsafe fn modreg(addr: u32, clear: u32, set: u32) {
    let v = rreg(addr);
    wreg(addr, (v & !clear) | set);
}

/// Initialise ENET clocks, RMII pin muxing, and reset the DP83825I PHY.
///
/// # Safety
///
/// Must be called exactly once, before creating the `Enet` driver.
pub unsafe fn init_hardware() {
    modreg(CCM_CCGR1, 0, 0b11 << 10);

    wreg(PLL_ENET_CLR, (1 << 12) | (1 << 16) | 0x3);
    wreg(PLL_ENET_SET, (1 << 13) | (1 << 21) | 1);
    while rreg(PLL_ENET) & (1 << 31) == 0 {}
    wreg(PLL_ENET_CLR, 1 << 16);

    modreg(GPR1, (1 << 13) | (1 << 23), 1 << 17);

    wreg(MUX_B0_14, 5);
    wreg(MUX_B0_15, 5);
    modreg(GPIO7_GDIR, 0, (1 << 14) | (1 << 15));
    wreg(GPIO7_DR_SET, 1 << 15);
    wreg(GPIO7_DR_CLR, 1 << 14);

    wreg(PAD_B1_04, RMII_PAD_INPUT_PULLDOWN);
    wreg(PAD_B1_05, RMII_PAD_INPUT_PULLUP);
    wreg(PAD_B1_06, RMII_PAD_INPUT_PULLDOWN);
    wreg(PAD_B1_07, RMII_PAD_INPUT_PULLUP);
    wreg(PAD_B1_08, RMII_PAD_INPUT_PULLUP);
    wreg(PAD_B1_09, RMII_PAD_INPUT_PULLUP);
    wreg(PAD_B1_10, RMII_PAD_CLOCK);
    wreg(PAD_B1_11, RMII_PAD_INPUT_PULLDOWN);
    wreg(PAD_B1_14, RMII_PAD_INPUT_PULLUP);
    wreg(PAD_B1_15, RMII_PAD_INPUT_PULLUP);

    wreg(GPIO7_DR_SET, 1 << 14);

    wreg(MUX_B1_04, 3);
    wreg(MUX_B1_05, 3);
    wreg(MUX_B1_06, 3);
    wreg(MUX_B1_07, 3);
    wreg(MUX_B1_08, 3);
    wreg(MUX_B1_09, 3);
    wreg(MUX_B1_10, 6 | 0x10);
    wreg(MUX_B1_11, 3);
    wreg(MUX_B1_14, 0);
    wreg(MUX_B1_15, 0);

    wreg(ENET_IPG_CLK_RMII_SI, 1);
    wreg(ENET_MDIO_SI, 2);
    wreg(ENET0_RXDATA_SI, 1);
    wreg(ENET1_RXDATA_SI, 1);
    wreg(ENET_RXEN_SI, 1);
    wreg(ENET_RXERR_SI, 1);
}

/// Configure DP83825I PHY registers over MDIO.
pub fn init_phy(enet: &mut impl imxrt_enet::MiimWrite) {
    enet.write(PHY_ADDR, 0x17, 0x0081).ok();
    enet.write(PHY_ADDR, 0x18, 0x0280).ok();
}
