

/// AC Characteristicsの要素定義
/// TODO: ns以外の考慮が必要ならu32固定やめるマクロ作る
#[derive(Clone, Copy, Debug)]
struct Timing {
    pub min_ns: Option<u32>,
    pub max_ns: Option<u32>,
}

/// AC Characteristics定義
#[derive(Clone, Copy, Debug)]
struct TimingSpec {
    /// t_CLS
    pub cle_setup: Timing,
    /// t_CLH
    pub cle_hold: Timing,
    /// t_CS
    pub ceb_setup: Timing,
    /// t_CH
    pub ceb_hold: Timing,
    /// t_WP
    pub wp_pulse_width: Timing,
    /// t_ALS
    pub ale_setup: Timing,
    /// t_ALH
    pub ale_hold: Timing,
    /// t_DS
    pub io_setup: Timing,
    /// t_DH
    pub io_hold: Timing,
    /// t_WC
    pub write_cycle_time: Timing,
    /// t_WH
    pub web_high_hold: Timing,
    /// t_WW
    pub wpb_high_to_web_low: Timing,
    /// t_RR
    pub reb_fall_edge: Timing,
    /// t_RW
    pub web_fall_edge: Timing,
    /// t_RP
    pub read_pulse_width: Timing,
    /// t_RC
    pub read_cycle_time: Timing,
    /// t_REA
    pub reb_access_time: Timing,
    /// t_CEA
    pub ceb_access_time: Timing,
    /// t_CLR
    pub cle_low_to_reb_low: Timing,
    /// t_AR
    pub ale_low_to_reb_low: Timing,
    /// t_RHOH
    pub reb_high_to_output_hold: Timing,
    /// t_RLOH
    pub reb_low_to_output_hold: Timing,
    /// t_RHZ
    pub reb_high_to_output_impedance: Timing,
    /// t_CHZ
    pub ceb_high_to_output_impedance: Timing,
    /// t_CSD
    pub ceb_high_to_ale_or_cle: Timing,
    /// t_REH
    pub reb_high_hold: Timing,
    /// t_IR
    pub output_high_impedance_to_reb: Timing,
    /// t_RHW
    pub reb_high_to_web_low: Timing,
    /// t_WHC
    pub web_high_to_ceb_low: Timing,
    /// t_WHR
    pub web_high_to_reb_low: Timing,
    /// t_WB
    pub web_high_to_busy: Timing,
    /// t_RST
    pub device_reset: Timing,
}

// /// IO pin定義
// struct IoPins {
//     /// IO: I/O Port 0~7
//     pub data: PinGroup,
//     /// /CE: Chip Enable 0
//     pub ceb0: AnyPin,
//     /// /CE: Chip Enable 1
//     pub ceb1: AnyPin,
//     /// CLE: Command Latch Enable
//     pub cle: AnyPin,
//     /// ALE: Address Latch Enable
//     pub ale: AnyPin,
//     /// /WP: Write Protect
//     pub wpb: AnyPin,
//     /// /WE: Write Enable
//     pub web: AnyPin,
//     /// /RE: Read Enable
//     pub reb: AnyPin,
//     /// RY / /BY: Read/Busy
//     pub rbb: AnyPin,
// }

// struct NandDevice {
//     pub timing_spec: TimingSpec,
//     pub pins: IoPins,
// }

// impl NandDevice {
//     pub fn init(&self) {
//         let push_pull_output = self.pins.ceb0.into_push_pull_output();
//     }
// }
