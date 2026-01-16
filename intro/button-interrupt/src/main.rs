#![no_std]
#![no_main]

use core::cell::{Cell, RefCell};
use critical_section::Mutex;
use esp_backtrace as _;
use esp_hal::{
    // delay::Delay,
    Blocking, 
    gpio::{Event, Input, InputConfig, Io}, 
    handler, main, 
    rmt::Rmt, time::Rate
};
use esp_println::println;
use smart_leds::{SmartLedsWrite, RGB8};
use esp_hal_smartled::{buffer_size, color_order, RmtSmartLeds, Sk68xxTiming};

esp_bootloader_esp_idf::esp_app_desc!();

#[derive(Copy, Clone)]
enum LedColorNow {
    Purple,
    Off
}

static BUTTON: Mutex<RefCell<Option<Input>>> = Mutex::new(RefCell::new(None));
static LED: Mutex<RefCell<Option<RmtSmartLeds<{ buffer_size::<RGB8>(1) }, Blocking, RGB8, color_order::Grb, Sk68xxTiming>>>> = Mutex::new(RefCell::new(None));
// LED 的影子状态：由于 LED 硬件只能写入无法读取，需要软件追踪当前颜色。
// 初始化为 Off 是因为 SK6812 LED 上电时默认熄灭。
static COLORFLAG: Mutex<Cell<LedColorNow>> = Mutex::new(Cell::new(LedColorNow::Off));
static PURPLE: RGB8 = RGB8::new(50, 0, 50);
static OFF: RGB8 = RGB8::new(0, 0, 0);

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
    let led = RmtSmartLeds::<{ buffer_size::<RGB8>(1) }, _, RGB8, color_order::Grb, Sk68xxTiming>::new(
        rmt.channel0,
        peripherals.GPIO8,
    )
    .unwrap();
    critical_section::with(|cs| {
        LED.borrow_ref_mut(cs).replace(led);
    });

    // Set GPIO9 as an input
    let mut button = Input::new(peripherals.GPIO9, InputConfig::default());

    critical_section::with(|cs| {
        button.listen(Event::FallingEdge);
        BUTTON.borrow_ref_mut(cs).replace(button);
    });
    critical_section::with(|cs| {
        LED.borrow_ref_mut(cs).as_mut().unwrap().write([PURPLE]).unwrap();
        COLORFLAG.borrow(cs).set(LedColorNow::Purple);
    });
    // let delay = Delay::new();
    loop {
        // led.write([purple]).unwrap();
        // delay.delay_millis(500);
        // led.write([off]).unwrap();
        // delay.delay_millis(500);
    }
}

#[handler]
fn handler() {
    critical_section::with(|cs| {
        println!("GPIO interrupt");
        let color = COLORFLAG.borrow(cs).get();
        match color {
            LedColorNow::Off => {
                LED.borrow_ref_mut(cs).as_mut().expect("LED driver not initialized") // check initialization (Option::unwrap)
                    .write([PURPLE]).unwrap(); // check write success (Result::unwrap)
                COLORFLAG.borrow(cs).set(LedColorNow::Purple);
            }
            LedColorNow::Purple => {
                LED.borrow_ref_mut(cs).as_mut().expect("LED driver not initialized")
                    .write([OFF]).unwrap();
                COLORFLAG.borrow(cs).set(LedColorNow::Off);
            }
        }
        
        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt();
    });
}
