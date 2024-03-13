#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    gpio::{connect_high_to_peripheral, connect_low_to_peripheral, InputSignal},
    otg_fs::{UsbBus, USB},
    peripherals::Peripherals,
    prelude::*,
    Delay, IO,
};
use esp_println::println;
use usb_device::endpoint::{EndpointType, In};
pub mod usb;

static mut EP_MEMORY: [u32; 1024] = [0; 1024];

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();

    let clocks = ClockControl::max(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    peripherals.USB_WRAP.otg_conf().modify(|_, w| {
        w.usb_pad_enable()
            .set_bit()
            .phy_sel()
            .clear_bit()
            .clk_en()
            .set_bit()
            .ahb_clk_force_on()
            .set_bit()
            .phy_clk_force_on()
            .set_bit()
    });

    // Connect USB OTG peripheral to internal TXvr
    peripherals
        .LPWR
        .usb_conf()
        .modify(|_, w| w.sw_hw_usb_phy_sel().set_bit().sw_usb_phy_sel().set_bit());

    connect_low_to_peripheral(InputSignal::USB_OTG_IDDIG); // TODO: connected connector is mini-A side
    connect_high_to_peripheral(InputSignal::USB_OTG_VBUSVALID); // receiving a valid Vbus from device
    connect_high_to_peripheral(InputSignal::USB_OTG_AVALID); // HIGH to force USB Host mode
    connect_low_to_peripheral(InputSignal::USB_SRP_BVALID);

    let usb = USB::new(peripherals.USB0, io.pins.gpio19, io.pins.gpio20);

    let usbbus = UsbBus::new(usb, unsafe { &mut EP_MEMORY });

    let mut hid_ep = usbbus
        .alloc::<In>(Some(0x81.into()), EndpointType::Interrupt, 64, 10)
        .expect("Could not create USBHID EP");

    // setup logger
    // To change the log_level change the env section in .cargo/config.toml
    // or remove it and set ESP_LOGLEVEL manually before running cargo run
    // this requires a clean rebuild because of https://github.com/rust-lang/cargo/issues/10358
    esp_println::logger::init_logger_from_env();
    log::info!("Logger is setup");
    println!("Hello world!");
    loop {
        println!("Loop...");
        delay.delay_ms(500u32);
    }
}
