// To easily test this you can connect GPIO2 and GPIO4
// This way we will receive was we send. (loopback)

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    dma::{DmaRxBuf, DmaTxBuf},
    dma_buffers, main,
    spi::{
        master::{Config, Spi},
        Mode,
    },
    time::Rate,
    rmt::Rmt
};
use esp_println::println;
use esp_hal_smartled::{buffer_size, color_order, RmtSmartLeds, Sk68xxTiming};
use smart_leds::{SmartLedsWrite, RGB8};

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Initialize the RMT peripheral for controlling the RGB LED on GPIO8
    let rmt = Rmt::new(peripherals.RMT, Rate::from_mhz(80)).unwrap();

    // Create LED driver for SK68XXMINI-HS (1 LED)
    let mut led = RmtSmartLeds::<{ buffer_size::<RGB8>(1) }, _, RGB8, color_order::Grb, Sk68xxTiming>::new(
        rmt.channel0,
        peripherals.GPIO8,
    )
    .unwrap();
    // Purple color: Red + Blue (255, 0, 255)
    let purple = RGB8::new(100, 0, 100);
    let off = RGB8::new(0, 0, 0);

    let sclk = peripherals.GPIO0;
    let miso = peripherals.GPIO2;
    let mosi = peripherals.GPIO4;
    let cs = peripherals.GPIO5;

    let dma_channel = peripherals.DMA_CH0;

    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(32000);
    let mut dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let mut dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();

    let mut spi = Spi::new(
        peripherals.SPI2,
        Config::default()
            .with_frequency(Rate::from_khz(100))
            .with_mode(Mode::_0),
    )
    .unwrap()
    .with_sck(sclk)
    .with_mosi(mosi)
    .with_miso(miso)
    .with_cs(cs)
    .with_dma(dma_channel);

    let delay = Delay::new();

    loop {
        // ANCHOR: transfer
        // To transfer much larger amounts of data we can use DMA and
        // the CPU can even do other things while the transfer is in progress
        let data = [0x01u8, 0x02, 0x03, 0x04];
        for chunk in dma_tx_buf.as_mut_slice().chunks_mut(data.len()) {
            chunk.copy_from_slice(&data[..chunk.len()]);
        }

        let transfer = spi
            .transfer(dma_rx_buf.len(), dma_rx_buf, dma_tx_buf.len(), dma_tx_buf)
            .map_err(|e| e.0)
            .unwrap();
        // ANCHOR_END: transfer

        while !transfer.is_done() {
            // Set RGB LED to purple
            led.write([purple]).unwrap();
            delay.delay_millis(100);
            // Turn off the LED
            led.write([off]).unwrap();
            delay.delay_millis(100);
        }

        (spi, (dma_rx_buf, dma_tx_buf)) = transfer.wait();
        println!("{:x?}..{:x?}", 
            &dma_rx_buf.as_slice()[..10], 
            &dma_rx_buf.as_slice().last_chunk::<10>().unwrap());

        delay.delay_millis(2500u32);
    }
}
