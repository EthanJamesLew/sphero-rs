use btleplug;
use btleplug::api::{Central, CentralEvent, Characteristic, Peripheral, WriteType};
use btleplug::bluez::{adapter::Adapter, manager::Manager};
use sphero_rs::packet::{slip_encode, to_bytes, SpheroPacket};
use std::error::Error;
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

use std::f32::consts::PI;

/// Convert HSV to RGB
/// Assume s = 1, v = 1
fn hsv_to_rgb(h: f32) -> (u8, u8, u8) {
    let r = ((h + PI / 3.0).sin() * 127.5 + 127.5) as u8;
    let g = ((h).sin() * 127.5 + 127.5) as u8;
    let b = ((h - PI / 3.0).sin() * 127.5 + 127.5) as u8;
    (r, g, b)
}

fn turn_on_led() -> Result<(), Box<dyn Error>> {
    let manager = Manager::new()?;
    let adapters = manager.adapters()?;
    let adapter = adapters.into_iter().next().expect("No adapters found");
    adapter.start_scan()?;

    // Scan for Bluetooth devices for 10 seconds.
    thread::sleep(Duration::from_secs(10));

    // Try to find a Sphero SPRK+ device.
    let device = adapter.peripherals().into_iter().find(|p| {
        p.properties()
            .local_name
            .as_ref()
            .map(|name| name.contains("SK-"))
            .unwrap_or(false)
    });

    if let Some(device) = device {
        println!("Found device: {:?}", device);

        // Connect to the device.
        device.connect()?;
        println!("Connected to device");

        // Wake up the device
        let anti_dos_characteristic_uuid = Uuid::parse_str("22bb746f-2bbd-7554-2d6f-726568705327")?;
        let tx_power_characteristic_uuid = Uuid::parse_str("22bb746f-2bb2-7554-2d6f-726568705327")?;
        let wakeup_characteristic_uuid = Uuid::parse_str("22bb746f-2bbf-7554-2d6f-726568705327")?;

        let characteristics = device.discover_characteristics()?;

        // print out characteristics
        for ch in &characteristics {
            println!("{:?}", ch);
        }

        let anti_dos_characteristic = characteristics
            .iter()
            .find(|c| c.uuid == anti_dos_characteristic_uuid)
            .ok_or("Anti DOS characteristic not found")?;

        let tx_power_characteristic = characteristics
            .iter()
            .find(|c| c.uuid == tx_power_characteristic_uuid)
            .ok_or("TX power characteristic not found")?;

        let wakeup_characteristic = characteristics
            .iter()
            .find(|c| c.uuid == wakeup_characteristic_uuid)
            .ok_or("Wakeup characteristic not found")?;

        device.write(
            &anti_dos_characteristic,
            b"011i3",
            WriteType::WithoutResponse,
        )?;
        thread::sleep(Duration::from_millis(100)); // Add a short delay between write operations to make sure each operation is processed in order

        device.write(
            &tx_power_characteristic,
            &[0x07],
            WriteType::WithoutResponse,
        )?;
        thread::sleep(Duration::from_millis(100));

        device.write(&wakeup_characteristic, &[0x01], WriteType::WithoutResponse)?;
        thread::sleep(Duration::from_millis(100));

        let characteristics = device.discover_characteristics()?;

        // Find the characteristic to write to.
        // The actual UUID might differ.
        let led_char_uuid = Uuid::parse_str("22bb746f-2ba1-7554-2d6f-726568705327").unwrap();
        let led_char = characteristics
            .into_iter()
            .find(|c| c.uuid == led_char_uuid)
            .unwrap();

        // Start with a hue of 0
        let mut hue: f32 = 0.0;

        // Loop to run forever
        loop {
            // Convert hue to RGB
            let (r, g, b) = hsv_to_rgb(hue);

            // Build packet to turn on LEDs with RGB color
            // let mut packet = SpheroPacket::new(0x01, None, None, 0x02, 0x20, 0x01, None, vec![r, g, b]);
            let mut packet = SpheroPacket::new(
                0b00010000,
                None,
                None,
                0x01,
                0x20,
                0x00,
                None,
                vec![0xFF, 0x00, 0x00],
            );

            let mut bytes = vec![0xff, 0x02, 0x20, 0x06, 0x04, r, g, b];
            let chk = !bytes.iter().fold(0u8, |acc, &x| acc.wrapping_add(x)) - 1;
            bytes.push(chk);
            bytes.insert(0, 0xff);

            //slip_encode(&mut packet);

            // Write to the characteristic.
            device.write(&led_char, &bytes, WriteType::WithoutResponse)?;

            // Increase hue
            hue += 0.05;
            if hue > 2.0 * PI {
                hue -= 2.0 * PI;
            }

            // Wait for 200ms before sending the next packet
            thread::sleep(Duration::from_millis(50));
        }

        // Disconnect from the device.
        // device.disconnect()?;
        // println!("Disconnected from device");
    } else {
        println!("No Sphero SPRK+ found")
    };

    Ok(())
}

fn main() {
    match turn_on_led() {
        Ok(_) => println!("Finished"),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}
