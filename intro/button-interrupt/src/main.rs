#![no_std]
#![no_main]

use core::cell::RefCell;
use critical_section::Mutex;
use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Event, Input, InputConfig, Io},
    rmt::Rmt,
    time::Rate,
    handler, main,
};
use esp_println::println;
use smart_leds::{SmartLedsWrite, RGB8};
use esp_hal_smartled::{buffer_size, color_order, RmtSmartLeds, Sk68xxTiming};

esp_bootloader_esp_idf::esp_app_desc!();

static BUTTON: Mutex<RefCell<Option<Input>>> = Mutex::new(RefCell::new(None));

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    println!("Hello world!");

    let mut io = Io::new(peripherals.IO_MUX);
    // Set the interrupt handler for GPIO interrupts.
    io.set_interrupt_handler(handler);

    // Set GPIO8 as an output, and set its state high initially.
    let rmt = Rmt::new(peripherals.RMT, Rate::from_mhz(80)).unwrap();
    // Create LED driver for SK68XXMINI-HS (1 LED)
    let mut led = RmtSmartLeds::<{ buffer_size::<RGB8>(1) }, _, RGB8, color_order::Grb, Sk68xxTiming>::new(
        rmt.channel0,
        peripherals.GPIO8,
    )
    .unwrap();
    // Purple color: Red + Blue (255, 0, 255)
    let purple = RGB8::new(50, 0, 50);
    let off = RGB8::new(0, 0, 0);

    // Set GPIO9 as an input
    let mut button = Input::new(peripherals.GPIO9, InputConfig::default());

    critical_section::with(|cs| {
        button.listen(Event::FallingEdge);
        BUTTON.borrow_ref_mut(cs).replace(button);
    });

    let delay = Delay::new();
    loop {
        led.write([purple]).unwrap();
        delay.delay_millis(500);
        led.write([off]).unwrap();
        delay.delay_millis(500);
    }
}

#[handler]
fn handler() {
    critical_section::with(|cs| {
        println!("GPIO interrupt");
        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt();
    });
}
