#![no_std]
#![no_main]

use core::cell::RefCell;
use critical_section::Mutex;
use esp_backtrace as _;
use esp_hal::{
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
enum LedColor {
    Purple,
    Off,
}

impl LedColor {
    fn toggle(self) -> Self {
        match self {
            LedColor::Purple => LedColor::Off,
            LedColor::Off => LedColor::Purple,
        }
    }

    fn to_rgb(self) -> RGB8 {
        match self {
            LedColor::Purple => RGB8::new(50, 0, 50),
            LedColor::Off => RGB8::new(0, 0, 0),
        }
    }
}

// 状态直接初始化，无 Option，无非法状态
static LED_COLOR: Mutex<RefCell<LedColor>> = Mutex::new(RefCell::new(LedColor::Purple));
static BUTTON: Mutex<RefCell<Option<Input>>> = Mutex::new(RefCell::new(None));
static LED: Mutex<RefCell<Option<RmtSmartLeds<{ buffer_size::<RGB8>(1) }, Blocking, RGB8, color_order::Grb, Sk68xxTiming>>>> = Mutex::new(RefCell::new(None));

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    println!("Hello world!");

    let mut io = Io::new(peripherals.IO_MUX);
    io.set_interrupt_handler(handler);

    // 初始化 LED
    let rmt = Rmt::new(peripherals.RMT, Rate::from_mhz(80)).unwrap();
    let mut led = RmtSmartLeds::<{ buffer_size::<RGB8>(1) }, _, RGB8, color_order::Grb, Sk68xxTiming>::new(
        rmt.channel0,
        peripherals.GPIO8,
    ).unwrap();
    
    // 初始化按键
    let mut button = Input::new(peripherals.GPIO9, InputConfig::default());
    button.listen(Event::FallingEdge);

    // LED 初始状态与 LED_COLOR 保持一致
    led.write([LedColor::Purple.to_rgb()]).unwrap();

    critical_section::with(|cs| {
        LED.borrow_ref_mut(cs).replace(led);
        BUTTON.borrow_ref_mut(cs).replace(button);
    });

    loop {}
}

#[handler]
fn handler() {
    critical_section::with(|cs| {
        println!("GPIO interrupt");
        
        // 切换颜色状态
        let new_color = LED_COLOR.borrow_ref(cs).toggle();
        *LED_COLOR.borrow_ref_mut(cs) = new_color;
        
        // 应用新颜色
        LED.borrow_ref_mut(cs).as_mut().unwrap().write([new_color.to_rgb()]).unwrap();
        
        BUTTON.borrow_ref_mut(cs).as_mut().unwrap().clear_interrupt();
    });
}

