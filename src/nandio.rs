use rp_pico::hal::gpio::{Pin, PinGroup};

/// AC Characteristicsの要素定義
/// TODO: ns以外の考慮が必要ならu32固定やめるマクロ作る
struct TimingSpec {
    pub min_ns: Option<u32>,
    pub max_ns: Option<u32>,
}

/// AC Characteristics定義
struct NandTimingSpec {
    /// t_CLS
    pub cle_setup: TimingSpec,
    /// t_CLH
    pub cle_hold: TimingSpec,
    /// t_CS
    pub ceb_setup: TimingSpec,
    /// t_CH
    pub ceb_hold: TimingSpec,
    /// t_WP
    pub wp_pulse_width: TimingSpec,
    /// t_ALS
    pub ale_setup: TimingSpec,
    /// t_ALH
    pub ale_hold: TimingSpec,
    /// t_DS
    pub io_setup: TimingSpec,
    /// t_DH
    pub io_hold: TimingSpec,
    /// t_WC
    pub write_cycle_time: TimingSpec,
    /// t_WH
    pub web_high_hold: TimingSpec,
    /// t_WW
    pub wpb_high_to_web_low: TimingSpec,
    /// t_RR
    pub reb_fall_edge: TimingSpec,
    /// t_RW
    pub web_fall_edge: TimingSpec,
    /// t_RP
    pub read_pulse_width: TimingSpec,
    /// t_RC
    pub read_cycle_time: TimingSpec,
    /// t_REA
    pub reb_access_time: TimingSpec,
    /// t_CEA
    pub ceb_access_time: TimingSpec,
    /// t_CLR
    pub cle_low_to_reb_low: TimingSpec,
    /// t_AR
    pub ale_low_to_reb_low: TimingSpec,
    /// t_RHOH
    pub reb_high_to_output_hold: TimingSpec,
    /// t_RLOH
    pub reb_low_to_output_hold: TimingSpec,
    /// t_RHZ
    pub reb_high_to_output_impedance: TimingSpec,
    /// t_CHZ
    pub ceb_high_to_output_impedance: TimingSpec,
    /// t_CSD
    pub ceb_high_to_ale_or_cle: TimingSpec,
    /// t_REH
    pub reb_high_hold: TimingSpec,
    /// t_IR
    pub output_high_impedance_to_reb: TimingSpec,
    /// t_RHW
    pub reb_high_to_web_low: TimingSpec,
    /// t_WHC
    pub web_high_to_ceb_low: TimingSpec,
    /// t_WHR
    pub web_high_to_reb_low: TimingSpec,
    /// t_WB
    pub web_high_to_busy: TimingSpec,
    /// t_RST
    pub device_reset: TimingSpec,
}

/// IO pin定義
struct NandIo {
    /// IO: I/O Port 0~7
    pub io: PinGroup,
    /// /CE: Chip Enable 0
    pub ceb0: Pin,
    /// /CE: Chip Enable 1
    pub ceb1: Pin,
    /// CLE: Command Latch Enable
    pub cle: Pin,
    /// ALE: Address Latch Enable
    pub ale: Pin,
    /// /WP: Write Protect
    pub wpb: Pin,
    /// /WE: Write Enable
    pub web: Pin,
    /// /RE: Read Enable
    pub reb: Pin,
    /// RY / /BY: Read/Busy
    pub rbb: Pin,
}
