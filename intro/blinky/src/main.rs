#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    main,
    rmt::Rmt,
    time::Rate,
};
use esp_hal_smartled::{buffer_size, color_order, RmtSmartLeds, Sk68xxTiming};
use esp_println::println;
use smart_leds::{SmartLedsWrite, RGB8};

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    println!("Hello world!");

    // Initialize the RMT peripheral for controlling the RGB LED on GPIO8
    let rmt = Rmt::new(peripherals.RMT, Rate::from_mhz(80)).unwrap();

    // Create LED driver for SK68XXMINI-HS (1 LED)
    let mut led = RmtSmartLeds::<{ buffer_size::<RGB8>(1) }, _, RGB8, color_order::Grb, Sk68xxTiming>::new(
        rmt.channel0,
        peripherals.GPIO8,
    )
    .unwrap();

    // Initialize the Delay peripheral
    let delay = Delay::new();

    // Purple color: Red + Blue (255, 0, 255)
    let purple = RGB8::new(100, 0, 100);
    let off = RGB8::new(0, 0, 0);

    loop {
        // Set RGB LED to purple
        led.write([purple]).unwrap();
        delay.delay_millis(500);

        // Turn off the LED
        led.write([off]).unwrap();
        delay.delay_millis(500);
    }
}
