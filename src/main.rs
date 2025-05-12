// +---------------------------------------------------------------------------+
// |                             PM/MA lab skel                                |
// +---------------------------------------------------------------------------+

//! By default, this app prints a "Hello world" message with `defmt`.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_net::StackResources;
use embassy_time::{Duration, Timer};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

// Use the logging macros provided by defmt.
use defmt::*;
use fixed::traits::ToFixed;
use embassy_rp::pwm::Pwm;
use embassy_rp::pwm::Config as ConfigPwm;
use cyw43::JoinOptions;

use embassy_net::{IpAddress, IpEndpoint};
use embassy_net::tcp::TcpSocket;
use embedded_io_async::Write;

use embassy_rp::gpio::*;

// Import interrupts definition module
mod irqs;

const SOCK: usize = 4;
static RESOURCES: StaticCell<StackResources<SOCK>> = StaticCell::<StackResources<SOCK>>::new();

const WIFI_NETWORK: &str = "DIGI-dskU";
const WIFI_PASSWORD: &str = "uXyuNYuEut";

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Get a handle to the RP's peripherals.
    let peripherals = embassy_rp::init(Default::default());

    // Init WiFi driver
    let (net_device, mut _control) = embassy_lab_utils::init_wifi!(&spawner, peripherals).await;

    // Default config for dynamic IP address
    let config = embassy_net::Config::dhcpv4(Default::default());

    // Init network stack
    let _stack = embassy_lab_utils::init_network_stack(&spawner, net_device, &RESOURCES, config);


    loop {
        match _control.join(WIFI_NETWORK, JoinOptions::new(WIFI_PASSWORD.as_bytes())).await {
            Ok(_) => break,
            Err(err) => {
                info!("join failed with status={}", err.status);
            }
        }
    }

    // Wait for DHCP, not necessary when using static IP
    info!("waiting for DHCP...");
    while !_stack.is_config_up() {
        Timer::after_millis(100).await;
    }
    info!("DHCP is now up!");

    let stack = _stack; // extrage stack-ul
    let mut rx_buffer = [0; 1024];
    let mut tx_buffer = [0; 1024];

    // Creare socket TCP
    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(Duration::from_secs(5)));

    // IP-ul laptopului tău din rețea
    let server_ip = IpAddress::v4(192, 168, 1, 130); // IP-ul laptopului tău
    let server_port = 1236; // Portul serverului

    loop {
        info!("Conectare la server...");

        // Încearcă să te conectezi la server
        match socket.connect(IpEndpoint::new(server_ip, server_port)).await {
            Ok(()) => {
                info!("Conectat la server!");

                // Trimitere mesaj către server
                let message = b"Salut de la Pico!\n";
                match socket.write_all(message).await {
                    Ok(()) => info!("Mesaj trimis."),
                    Err(e) => warn!("Eroare la trimitere: {:?}", e),
                }

                // Așteaptă răspunsul de la server
                let mut buf = [0; 128];
                match socket.read(&mut buf).await {
                    Ok(n) => {
                        let s = core::str::from_utf8(&buf[..n]).unwrap_or("???");
                        info!("Răspuns primit: {}", s);
                    }
                    Err(e) => warn!("Eroare la citire: {:?}", e),
                }

                break;
            }
            Err(e) => {
                warn!("Eroare la conectare: {:?}", e);
                Timer::after_secs(2).await;
            }
        }
    }

    // TO DO: LED
    // let mut led = Output::new(peripherals.PIN_15, Level::Low);

    // loop {
    //     led.set_high();
    //     Timer::after(Duration::from_millis(500)).await;

    //     info!("LED ON");

    //     led.set_low();
    //     Timer::after(Duration::from_millis(500)).await;

    //     info!("LED OFF");
    // }

    // TO DO: servo

    // let mut servo_config: ConfigPwm = Default::default();

    // servo_config.top = 0xB71A;

    // servo_config.divider = 64_i32.to_fixed();

    // // Servo timing constants
    // const PERIOD_US: usize = 20_000;
    // const MIN_PULSE_US: usize = 500;
    // const MAX_PULSE_US: usize = 2500;

    // let min_pulse = (MIN_PULSE_US * servo_config.top as usize) / PERIOD_US;
    // let max_pulse = (MAX_PULSE_US * servo_config.top as usize) / PERIOD_US;

    // let mut servo = Pwm::new_output_a(
    //     peripherals.PWM_SLICE2,
    //     peripherals.PIN_4,
    //     servo_config.clone()
    // );

    //     servo.set_config(&servo_config);

    // // Main loop to move the servo back and forth
    // let delay = Duration::from_secs(1);
    // loop {
    //     // Move servo to maximum position (180 degrees)
    //     servo_config.compare_a = max_pulse as u16; // Cast to u16
    //     servo.set_config(&servo_config);          // Update PWM configuration
    //     Timer::after(delay).await;                // Wait 1 second

    //     // Move servo to minimum position (0 degrees)
    //     servo_config.compare_a = min_pulse as u16; // Cast to u16
    //     servo.set_config(&servo_config);          // Update PWM configuration
    //     Timer::after(delay).await;                // Wait 1 second
    // }
}
