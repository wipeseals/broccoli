#![no_std]
#![no_main]

mod nandio;

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

use rp_pico as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

use pio_proc::pio_asm;

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

    // == pio program ==
    //
    // -- pins <-> gpio --
    // base + 00: io0
    // base + 01: io1
    // base + 02: io2
    // base + 03: io3
    // base + 04: io4
    // base + 05: io5
    // base + 06: io6
    // base + 07: io7
    // base + 08: ceb0
    // base + 09: ceb1
    // base + 10: cle
    // base + 11: ale
    // base + 12: wpb
    // base + 13: web
    // base + 14: reb
    // base + 15: rbb
    //
    // -- datain --
    // config                       : u32   : scratchXで保持. {bit31~bit4=reserved, bit3=完了時IRQ発生, bit2=transfer_count分完了したらNOP loopにする, bit1=pin入力値をFIFO出力する, bit0=RBB Highを待つ}
    // transfer_count               : u32   : scratchYで保持. ループカウントにする
    // pindir                       : u32   : bit15~0をpindirsに設定. bit31~16は実質reserved
    // pinout0, pinout1 ...         : [u32] : stage0_count数分だけ出力pinに流し込む。ceb0~rbb含む(rbbは使わないと思う)
    //                              :       : 全シーケンス完了後、現在の状態を保持したままstage_count入力状態に戻るので継続動作可
    const MAX_DELAY: u8 = 31;
    let mut assembler = pio::Assembler::<32>::new();
    let mut label_configure = assembler.label();
    let mut label_wrap_target = assembler.label();
    let mut label_wrap_source = assembler.label();

    assembler.bind(&mut label_configure);
    // TX FIFO -> OSR (Output Shift Register): config
    assembler.pull(true, true);
    // OSR -> X: config
    assembler.mov(
        pio::MovDestination::X,
        pio::MovOperation::None,
        pio::MovSource::OSR,
    );
    // TX FIFO -> OSR: transfer_count
    assembler.pull(true, true);
    // OSR -> Y: transfer_count
    assembler.mov(
        pio::MovDestination::Y,
        pio::MovOperation::None,
        pio::MovSource::OSR,
    );
    // TX FIFO -> OSR: pindir
    assembler.pull(true, true);
    assembler.out(pio::OutDestination::PINDIRS, 32); // movだとPINDIRS選べない、setは使える即値のbitwidth足りない

    // TODO: ループしてbitbang設定しつつデータ読み出したり

    // TODO: 終了処理

    assembler.set(pio::SetDestination::PINDIRS, 0xff);
    assembler.set_with_delay(pio::SetDestination::PINS, 0, MAX_DELAY);
    assembler.set_with_delay(pio::SetDestination::PINS, 1, MAX_DELAY);
    // read data
    // NOP
    assembler.bind(&mut label_wrap_target);
    assembler.bind(&mut label_wrap_source);
    let program = assembler.assemble_with_wrap(label_wrap_source, label_wrap_target);

    // run pio0
    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let installed = pio.install(&program).unwrap();
    let (int, frac) = (0, 0);
    let (sm, _, _) = PIOBuilder::from_program(installed)
        .set_pins(led_pin.id().num, 1)
        .clock_divisor_fixed_point(int, frac)
        .build(sm0);
    sm.start();
    // //////////////////////////
    // // pin assign & init
    // // IO: I/O Port 0~7
    // let mut io0_pin = pins
    //     .gpio0
    //     .into_push_pull_output_in_state(bsp::hal::gpio::PinState::Low);
    // let mut io1_pin = pins
    //     .gpio1
    //     .into_push_pull_output_in_state(bsp::hal::gpio::PinState::Low);
    // let mut io2_pin = pins
    //     .gpio2
    //     .into_push_pull_output_in_state(bsp::hal::gpio::PinState::Low);
    // let mut io3_pin = pins
    //     .gpio3
    //     .into_push_pull_output_in_state(bsp::hal::gpio::PinState::Low);
    // let mut io4_pin = pins
    //     .gpio4
    //     .into_push_pull_output_in_state(bsp::hal::gpio::PinState::Low);
    // let mut io5_pin = pins
    //     .gpio5
    //     .into_push_pull_output_in_state(bsp::hal::gpio::PinState::Low);
    // let mut io6_pin = pins
    //     .gpio6
    //     .into_push_pull_output_in_state(bsp::hal::gpio::PinState::Low);
    // let mut io7_pin = pins
    //     .gpio7
    //     .into_push_pull_output_in_state(bsp::hal::gpio::PinState::Low);
    // // /CE: Chip Enable 0
    // let mut ceb0_pin = pins
    //     .gpio8
    //     .into_push_pull_output_in_state(bsp::hal::gpio::PinState::High);
    // // /CE: Chip Enable 1
    // let mut ceb1_pin = pins
    //     .gpio9
    //     .into_push_pull_output_in_state(bsp::hal::gpio::PinState::High);
    // // CLE: Command Latch Enable
    // let mut cle_pin = pins
    //     .gpio10
    //     .into_push_pull_output_in_state(bsp::hal::gpio::PinState::Low);
    // // ALE: Address Latch Enable
    // let mut ale_pin = pins
    //     .gpio11
    //     .into_push_pull_output_in_state(bsp::hal::gpio::PinState::Low);
    // // /WP: Write Protect
    // let _wpb_pin = pins
    //     .gpio12
    //     .into_push_pull_output_in_state(bsp::hal::gpio::PinState::Low);
    // // /WE: Write Enable
    // let mut web_pin = pins
    //     .gpio13
    //     .into_push_pull_output_in_state(bsp::hal::gpio::PinState::High);
    // // /RE: Read Enable
    // let mut reb_pin = pins
    //     .gpio14
    //     .into_push_pull_output_in_state(bsp::hal::gpio::PinState::High);
    // // RY / /BY: Read/Busy
    // let _rbb_pin = pins.gpio15.into_pull_up_input();
    // delay.delay_ms(1);

    // ////////////////////////////////
    // // command latch 0x90 (ID Read)
    // io0_pin.set_state(bsp::hal::gpio::PinState::Low).unwrap();
    // io1_pin.set_state(bsp::hal::gpio::PinState::Low).unwrap();
    // io2_pin.set_state(bsp::hal::gpio::PinState::Low).unwrap();
    // io3_pin.set_state(bsp::hal::gpio::PinState::Low).unwrap();
    // io4_pin.set_state(bsp::hal::gpio::PinState::High).unwrap();
    // io5_pin.set_state(bsp::hal::gpio::PinState::Low).unwrap();
    // io6_pin.set_state(bsp::hal::gpio::PinState::Low).unwrap();
    // io7_pin.set_state(bsp::hal::gpio::PinState::High).unwrap();

    // cle_pin.set_state(bsp::hal::gpio::PinState::High).unwrap();

    // ceb0_pin.set_state(bsp::hal::gpio::PinState::Low).unwrap();
    // ceb1_pin.set_state(bsp::hal::gpio::PinState::High).unwrap();

    // web_pin.set_state(bsp::hal::gpio::PinState::Low).unwrap();
    // ale_pin.set_state(bsp::hal::gpio::PinState::Low).unwrap();
    // reb_pin.set_state(bsp::hal::gpio::PinState::High).unwrap();
    // delay.delay_ms(1);

    // web_pin.set_state(bsp::hal::gpio::PinState::High).unwrap();
    // delay.delay_ms(1);

    // ////////////////////////////////
    // // Address latch 0x00
    // io0_pin.set_state(bsp::hal::gpio::PinState::Low).unwrap();
    // io1_pin.set_state(bsp::hal::gpio::PinState::Low).unwrap();
    // io2_pin.set_state(bsp::hal::gpio::PinState::Low).unwrap();
    // io3_pin.set_state(bsp::hal::gpio::PinState::Low).unwrap();
    // io4_pin.set_state(bsp::hal::gpio::PinState::Low).unwrap();
    // io5_pin.set_state(bsp::hal::gpio::PinState::Low).unwrap();
    // io6_pin.set_state(bsp::hal::gpio::PinState::Low).unwrap();
    // io7_pin.set_state(bsp::hal::gpio::PinState::Low).unwrap();

    // cle_pin.set_state(bsp::hal::gpio::PinState::Low).unwrap();

    // ceb0_pin.set_state(bsp::hal::gpio::PinState::Low).unwrap();
    // ceb1_pin.set_state(bsp::hal::gpio::PinState::High).unwrap();

    // web_pin.set_state(bsp::hal::gpio::PinState::Low).unwrap();
    // ale_pin.set_state(bsp::hal::gpio::PinState::High).unwrap();
    // reb_pin.set_state(bsp::hal::gpio::PinState::High).unwrap();
    // delay.delay_ms(1);

    // web_pin.set_state(bsp::hal::gpio::PinState::High).unwrap();
    // delay.delay_ms(1); // TODO: wait t_AR

    // ////////////////////////////////
    // // ready for dataread (/RE = Low & IO dir = Input)
    // let io0_pin = io0_pin.into_pull_down_input();
    // let io1_pin = io1_pin.into_pull_down_input();
    // let io2_pin = io2_pin.into_pull_down_input();
    // let io3_pin = io3_pin.into_pull_down_input();
    // let io4_pin = io4_pin.into_pull_down_input();
    // let io5_pin = io5_pin.into_pull_down_input();
    // let io6_pin = io6_pin.into_pull_down_input();
    // let io7_pin = io7_pin.into_pull_down_input();

    // cle_pin.set_state(bsp::hal::gpio::PinState::Low).unwrap();

    // ceb0_pin.set_state(bsp::hal::gpio::PinState::Low).unwrap();
    // ceb1_pin.set_state(bsp::hal::gpio::PinState::High).unwrap();

    // web_pin.set_state(bsp::hal::gpio::PinState::High).unwrap();
    // ale_pin.set_state(bsp::hal::gpio::PinState::Low).unwrap();
    // reb_pin.set_state(bsp::hal::gpio::PinState::Low).unwrap();
    // delay.delay_ms(1); // TODO: wait t_REA

    // //////////////////////////////////////
    // // data read
    // for read_index in 0..5 {
    //     let read_data: u8 = {
    //         (if io0_pin.is_high().unwrap() {
    //             0x01
    //         } else {
    //             0x00
    //         }) | (if io1_pin.is_high().unwrap() {
    //             0x02
    //         } else {
    //             0x00
    //         }) | (if io2_pin.is_high().unwrap() {
    //             0x04
    //         } else {
    //             0x00
    //         }) | (if io3_pin.is_high().unwrap() {
    //             0x08
    //         } else {
    //             0x00
    //         }) | (if io4_pin.is_high().unwrap() {
    //             0x10
    //         } else {
    //             0x00
    //         }) | (if io5_pin.is_high().unwrap() {
    //             0x20
    //         } else {
    //             0x00
    //         }) | (if io6_pin.is_high().unwrap() {
    //             0x40
    //         } else {
    //             0x00
    //         }) | (if io7_pin.is_high().unwrap() {
    //             0x80
    //         } else {
    //             0x00
    //         })
    //     };
    //     info!("data[{}] = {:#02x}", read_index, read_data);

    //     // next cyc
    //     reb_pin.set_state(bsp::hal::gpio::PinState::High).unwrap();
    //     delay.delay_ms(1);
    //     reb_pin.set_state(bsp::hal::gpio::PinState::Low).unwrap();
    //     delay.delay_ms(1); // TODO: wait t_REA
    // }

    loop {
        // pio
        cortex_m::asm::wfi();

        // cpu
        // led_pin.set_high().unwrap();
        // delay.delay_ms(1000);
        // led_pin.set_low().unwrap();
        // delay.delay_ms(1000);
    }
}

// End of file
