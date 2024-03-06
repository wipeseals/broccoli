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
    // ...
    // base + 25: LED
    //
    // -- datain --
    // wait_rbb                     : u32   : scratchYで保持. 1:RBB Highを待つ, 0: RBB Highを待たない
    // pin_dir                      : u32   : bit15~0をpin_dirsに設定.bit15はRBBなのでLow(Input)固定. bit31~16は実質reserved
    // transfer_count               : u32   : scratchXで保持. ループカウント-1を設定。 (post-decrent由来)
    // pinout0, pinout1 ...         : [u32] : transfer_count数分だけ出力pinに流し込む。ceb0~rbb含む(入力ピンは実質無効)
    //                              :       : 全シーケンス完了後、現在の状態を保持したまま先頭に戻るので継続動作可
    const IO0_PIN: u32 = 0;
    const IO1_PIN: u32 = 1;
    const IO2_PIN: u32 = 2;
    const IO3_PIN: u32 = 3;
    const IO4_PIN: u32 = 4;
    const IO5_PIN: u32 = 5;
    const IO6_PIN: u32 = 6;
    const IO7_PIN: u32 = 7;
    const CEB0_PIN: u32 = 8;
    const CEB1_PIN: u32 = 9;
    const CLE_PIN: u32 = 10;
    const ALE_PIN: u32 = 11;
    const WPB_PIN: u32 = 12;
    const WEB_PIN: u32 = 13;
    const REB_PIN: u32 = 14;
    const RBB_PIN: u32 = 15;
    const LED_PIN: u32 = 25;
    const IRQ_INDEX: u8 = 0;
    let mut assembler = pio::Assembler::<32>::new();
    let mut label_wait_rbb = assembler.label();
    let mut label_start_transfer = assembler.label();
    let mut label_loop_transfer = assembler.label();
    let mut label_end_transfer = assembler.label();
    let mut label_wrap_target = assembler.label();
    let mut label_wrap_source = assembler.label();

    ////////////////////////////////////////////////////
    // start
    assembler.bind(&mut label_wrap_target);

    ////////////////////////////////////////////////////
    // wait rbb

    // RBB待たない場合のフラグをFIFOから持ってきて初回判定とする
    // TX FIFO -> OSR (Output Shift Register): wait_rbb
    assembler.pull(true, true);
    // OSR -> Y: wait_rbb
    assembler.mov(
        pio::MovDestination::Y,
        pio::MovOperation::None,
        pio::MovSource::OSR,
    );

    // 初回Skip, 2回目以後のRBB監視ループ
    assembler.bind(&mut label_wait_rbb);
    // check (1st=wait_rbb, 2nd/3rd/...=~pins[15])
    assembler.jmp(pio::JmpCondition::YIsZero, &mut label_start_transfer);
    // clear flag
    assembler.mov(
        pio::MovDestination::Y,
        pio::MovOperation::None,
        pio::MovSource::NULL,
    );
    // pins -> OSR
    assembler.mov(
        pio::MovDestination::OSR,
        pio::MovOperation::Invert, // RBB=1でReadyだが、jmpはXisZero判定で抜けるので反転入力
        pio::MovSource::PINS,
    );
    // (OSR >> 15) & 0x1をXに代入。 OSR >> 15 したあとに1bit転送
    assembler.out(pio::OutDestination::NULL, (RBB_PIN as u8) - 1);
    assembler.out(pio::OutDestination::X, 1);

    ////////////////////////////////////////////////////
    // transfer data
    assembler.bind(&mut label_start_transfer);

    // 転送数設定
    // TX FIFO -> OSR: transfer_count
    assembler.pull(true, true);
    // OSR -> X: transfer_count
    assembler.mov(
        pio::MovDestination::X,
        pio::MovOperation::None,
        pio::MovSource::OSR,
    );

    // pin設定
    // TX FIFO -> OSR: pin_dir
    assembler.pull(true, true);
    // OSR -> PINDIRS: pin_dir
    assembler.out(pio::OutDestination::PINDIRS, 32); // movだとPINDIRS選べない、setは使える即値のbitwidth足りない

    // 送受信ループ
    assembler.bind(&mut label_loop_transfer);
    // TX FIFO -> OSR: out_data[N]
    assembler.pull(true, true);
    // OSR -> pins: out_data[N]
    assembler.mov(
        pio::MovDestination::PINS,
        pio::MovOperation::None,
        pio::MovSource::OSR,
    );
    // pins -> ISR: in_data[N]
    assembler.mov(
        pio::MovDestination::ISR,
        pio::MovOperation::None,
        pio::MovSource::PINS,
    );
    // ISR -> RX FIFO: in_data[N]
    assembler.push(true, true);
    // loop count & loop
    assembler.jmp(pio::JmpCondition::XDecNonZero, &mut label_loop_transfer);

    // 終了処理
    assembler.bind(&mut label_end_transfer);
    // 割り込み通知
    assembler.irq(false, false, IRQ_INDEX, false);
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

    // | 31  | 30  | 29  | 28  | 27  | 26  | 25  | 24  | 23  | 22  | 21  | 20  | 19  | 18  | 17  | 16  | 15  | 14  | 13  | 12  | 11  | 10  | 9    | 8    | 7   | 6   | 5   | 4   | 3   | 2   | 1   | 0   |
    // | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | ---- | ---- | --- | --- | --- | --- | --- | --- | --- | --- |
    // | --  | --  | --  | --  | --  | --  | led | --  | --  | --  | --  | --  | --  | --  | --  | --  | rbb | reb | web | wpb | ale | cle | ceb1 | ceb0 | io7 | io6 | io5 | io4 | io3 | io2 | io1 | io0 |

    // 指定したbitだけ1の値
    let bit_on = |bit_pos: u32| (0x01u32 << bit_pos);

    // RBB以外全部Output
    let write_pin_dir = bit_on(LED_PIN)
        | bit_on(REB_PIN)
        | bit_on(WEB_PIN)
        | bit_on(WPB_PIN)
        | bit_on(ALE_PIN)
        | bit_on(CLE_PIN)
        | bit_on(CEB1_PIN)
        | bit_on(CEB0_PIN)
        | bit_on(IO7_PIN)
        | bit_on(IO6_PIN)
        | bit_on(IO5_PIN)
        | bit_on(IO4_PIN)
        | bit_on(IO3_PIN)
        | bit_on(IO2_PIN)
        | bit_on(IO1_PIN)
        | bit_on(IO0_PIN);
    // RBB,IO以外Output
    let read_pin_dir = bit_on(LED_PIN)
        | bit_on(REB_PIN)
        | bit_on(WEB_PIN)
        | bit_on(WPB_PIN)
        | bit_on(ALE_PIN)
        | bit_on(CLE_PIN)
        | bit_on(CEB1_PIN)
        | bit_on(CEB0_PIN);
    // Assert=/WP, Negate=/CS,/RE,/WE,/CLE,ALE,
    let init_pin_state = bit_on(REB_PIN) | bit_on(WEB_PIN) | bit_on(CEB1_PIN) | bit_on(CEB0_PIN);
    // CS0, CS1設定
    let access_to_cs0 = true;
    let (en_cs_pin, dis_cs_pin) = if access_to_cs0 {
        (CEB0_PIN, CEB1_PIN)
    } else {
        (CEB1_PIN, CEB0_PIN)
    };
    const READ_ID_CMD: u32 = 0x90;
    const READ_ID_ADDRESS: u32 = 0x00;

    let read_id_seq = [
        // send cmd & address
        0x00000000u32,  // wait_rbb
        write_pin_dir,  // pin_dir
        5u32,           // transfer_count
        init_pin_state, // data00: init
        bit_on(dis_cs_pin) | bit_on(REB_PIN) | bit_on(CLE_PIN) | bit_on(LED_PIN) | READ_ID_CMD, //data01: set cmd
        bit_on(dis_cs_pin) | bit_on(REB_PIN) | bit_on(CLE_PIN) | bit_on(WEB_PIN) | READ_ID_CMD, //data02: posedge /WE with CLE
        bit_on(dis_cs_pin) | bit_on(REB_PIN) | bit_on(ALE_PIN) | bit_on(LED_PIN) | READ_ID_ADDRESS, //data03: set address
        bit_on(dis_cs_pin) | bit_on(REB_PIN) | bit_on(ALE_PIN) | bit_on(WEB_PIN) | READ_ID_ADDRESS, //data04: posedge /WE with ALE
        // read data
        0x00000000u32,                                          // wait_rbb
        read_pin_dir,                                           // pin_dir
        13u32,                                                  // transfer_count
        bit_on(dis_cs_pin) | bit_on(WEB_PIN) | bit_on(LED_PIN), //data00: /RE=0
        bit_on(dis_cs_pin) | bit_on(WEB_PIN) | bit_on(REB_PIN), //data01: posedge /RE for d0
        bit_on(dis_cs_pin) | bit_on(WEB_PIN) | bit_on(LED_PIN), //data02: /RE=0
        bit_on(dis_cs_pin) | bit_on(WEB_PIN) | bit_on(REB_PIN), //data03: posedge /RE for d1
        bit_on(dis_cs_pin) | bit_on(WEB_PIN) | bit_on(LED_PIN), //data04: /RE=0
        bit_on(dis_cs_pin) | bit_on(WEB_PIN) | bit_on(REB_PIN), //data05: posedge /RE for d2
        bit_on(dis_cs_pin) | bit_on(WEB_PIN) | bit_on(LED_PIN), //data06: /RE=0
        bit_on(dis_cs_pin) | bit_on(WEB_PIN) | bit_on(REB_PIN), //data07: posedge /RE for d3
        bit_on(dis_cs_pin) | bit_on(WEB_PIN) | bit_on(LED_PIN), //data08: /RE=0
        bit_on(dis_cs_pin) | bit_on(WEB_PIN) | bit_on(REB_PIN), //data09: posedge /RE for d4
        bit_on(dis_cs_pin) | bit_on(WEB_PIN) | bit_on(LED_PIN), //data10: /RE=0
        bit_on(dis_cs_pin) | bit_on(WEB_PIN) | bit_on(REB_PIN), //data11: posedge /RE for d5
        init_pin_state,                                         // data12: finalize
    ];

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
    }
}

// End of file
