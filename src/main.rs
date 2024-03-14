#![no_std]
#![no_main]

mod nandio;

use alloc::boxed::Box;
use bsp::{
    entry,
    hal::{
        gpio::{FunctionPio0, Pin},
        pio::{PIOBuilder, PIOExt},
    },
};
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use panic_probe as _;

use pio::Assembler;
use rp_pico as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

use bitflags;

/// TC58NVG0S3HTA00
/// NAND ICのコマンド定義
pub struct NandCmdSpec {
    first: u8,
    second: Option<u8>,
    acceptable_while_busy: bool,
}

/// NAND ICのコマンド列挙
pub enum NandCmd {
    SerialDataInput,
    Read,
    ColumnAddressChangeInSerialDataOutput,
    ReadWithDataCache,
    ReadStartForLastPageInReadCycleWithDataCache,
    AutoPageProgram,
    ColumnAddressChangeInSerialDataInput,
    AutoPageProgramWithDataCache,
    ReadForPageCopyWithDataOut,
    AutoPageProgramWithDataCacheDuringPageCopy,
    AutoPageProgramForLastPageDuringPageCopy,
    AutoBlockErase,
    IdRead,
    Statusread,
    Reset,
}

bitflags! {
    /// StatusReadの結果
    pub struct StatusReadResult: u8 {
        /// pass=0, fail=1
        const CHIP_STATUS_1 = 0b_00000001;
        /// pass=0, fail=1
        const CHIP_STATUS_2 = 0b_00000010;
        const RESERVED_1 = 0b_00000100;
        const RESERVED_2 = 0b_00001000;
        const RESERVED_3 = 0b_00010000;
        /// Page Buffer Busy=0, Page Buffer Ready=1
        const PAGE_BUFFER_READY_BUSYB = 0b_00100000;
        /// Data Cache Busy=0, Data Cache Ready=1
        const DATA_CACHE_READY_BUSYB = 0b_01000000;
        /// Write Protect Protected=0, Write Protect Not Protected=1
        const WRITE_PROTECT = 0b_10000000;
    }
}

/// NAND IC Interconnect
/// | 31  | 30  | 29  | 28  | 27  | 26  | 25  | 24  | 23  | 22  | 21  | 20  | 19  | 18  | 17  | 16  | 15  | 14  | 13  | 12  | 11  | 10  | 9    | 8    | 7   | 6   | 5   | 4   | 3   | 2   | 1   | 0   |
/// | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | ---- | ---- | --- | --- | --- | --- | --- | --- | --- | --- |
/// | --  | --  | --  | --  | --  | --  | led | --  | --  | --  | --  | --  | --  | --  | --  | --  | rbb | reb | web | wpb | ale | cle | ceb1 | ceb0 | io7 | io6 | io5 | io4 | io3 | io2 | io1 | io0 |
#[repr(u32)]
pub enum NandPinAssign {
    Io0Pin = 0,
    Io1Pin = 1,
    Io2Pin = 2,
    Io3Pin = 3,
    Io4Pin = 4,
    Io5Pin = 5,
    Io6Pin = 6,
    Io7Pin = 7,
    Ceb0Pin = 8,
    Ceb1Pin = 9,
    ClePin = 10,
    AlePin = 11,
    WpbPin = 12,
    WebPin = 13,
    RebPin = 14,
    RbbPin = 15,
    LedPin = 25,
}

/// PIOのコマンド定義
/// SET Cmdで値を入れるので、コマンドは5bit (0x00~0x1f) の範囲であること
///
/// ## Command Description
///
///
/// ## Example
///
/// - Reset
/// - ID read
/// - StatusRead
/// - READ
/// - Program
/// - Erase
///
#[repr(u8)]
pub enum NandPioCmd {
    Nop = 0x00,
    PinDir = 0x01,
    PinInit = 0x02,
    CmdLatch = 0x03,
    AddrLatch = 0x04,
    WaitRbb = 0x05,
    WriteData = 0x06,
    ReadData = 0x07,
    SendIrq = 0x08,
}

impl Default for NandPioCmd {
    fn default() -> Self {
        Self::Nop
    }
}

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    // setup gpio
    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    // gpio25
    let mut led_pin: Pin<_, FunctionPio0, _> = pins.led.into_function();

    delay.delay_ms(1000);
    info!("== broccoli ==");

    const IRQ_INDEX: u8 = 0;

    let mut assembler = pio::Assembler::<32>::new();
    let mut label_wrap_target = assembler.label();
    let mut label_wrap_source = assembler.label();

    ////////////////////////////////////////////////////
    assembler.bind(&mut label_wrap_target);

    assembler.bind(&mut label_wrap_source);

    let program = assembler.assemble_with_wrap(label_wrap_source, label_wrap_target);

    // run pio0
    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let installed = pio.install(&program).unwrap();
    let (int, frac) = (0, 0);
    let (sm, _, _) = PIOBuilder::from_program(installed)
        .set_pins(led_pin.id().num, 1) // TODO:ピン設定とRBBのInternal Pulldown
        .clock_divisor_fixed_point(int, frac)
        .build(sm0);
    sm.start();

    // 指定したbitだけ1の値
    let bit_on = |bit_pos: u32| (0x01u32 << bit_pos);

    // RBB以外全部Output
    let write_pin_dir = bit_on(NandPinAssign::LedPin as u32)
        | bit_on(NandPinAssign::RebPin as u32)
        | bit_on(NandPinAssign::WebPin as u32)
        | bit_on(NandPinAssign::WpbPin as u32)
        | bit_on(NandPinAssign::AlePin as u32)
        | bit_on(NandPinAssign::ClePin as u32)
        | bit_on(NandPinAssign::Ceb1Pin as u32)
        | bit_on(NandPinAssign::Ceb0Pin as u32)
        | bit_on(NandPinAssign::Io7Pin as u32)
        | bit_on(NandPinAssign::Io6Pin as u32)
        | bit_on(NandPinAssign::Io5Pin as u32)
        | bit_on(NandPinAssign::Io4Pin as u32)
        | bit_on(NandPinAssign::Io3Pin as u32)
        | bit_on(NandPinAssign::Io2Pin as u32)
        | bit_on(NandPinAssign::Io1Pin as u32)
        | bit_on(NandPinAssign::Io0Pin as u32);
    // RBB,IO以外Output
    let read_pin_dir = bit_on(NandPinAssign::LedPin as u32)
        | bit_on(NandPinAssign::RebPin as u32)
        | bit_on(NandPinAssign::WebPin as u32)
        | bit_on(NandPinAssign::WpbPin as u32)
        | bit_on(NandPinAssign::AlePin as u32)
        | bit_on(NandPinAssign::ClePin as u32)
        | bit_on(NandPinAssign::Ceb1Pin as u32)
        | bit_on(NandPinAssign::Ceb0Pin as u32);
    // Assert=/WP, Negate=/CS,/RE,/WE,/CLE,ALE,
    let init_pin_state = bit_on(NandPinAssign::RebPin as u32)
        | bit_on(NandPinAssign::WebPin as u32)
        | bit_on(NandPinAssign::Ceb1Pin as u32)
        | bit_on(NandPinAssign::Ceb0Pin as u32);
    // CS0, CS1設定
    let access_to_cs0 = true;
    let (en_cs_pin, dis_cs_pin) = if access_to_cs0 {
        (NandPinAssign::Ceb0Pin as u32, NandPinAssign::Ceb1Pin as u32)
    } else {
        (NandPinAssign::Ceb1Pin as u32, NandPinAssign::Ceb0Pin as u32)
    };

    loop {
        // pio
        cortex_m::asm::wfi();
    }
}

// End of file
