use btleplug;
use btleplug::api::{Central, WriteType, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};
use std::error::Error;
use std::thread;
use std::time::Duration;
use uuid::Uuid;

use deku::DekuContainerWrite;
use sphero_rs::packet::{DeviceID, SpheroCommandID, SpheroCommandPacketV1};

use std::f32::consts::PI;

/// Convert HSV to RGB
/// Assume s = 1, v = 1
fn hsv_to_rgb(h: f32) -> (u8, u8, u8) {
    let r = ((h + PI / 3.0).sin() * 127.5 + 127.5) as u8;
    let g = ((h).sin() * 127.5 + 127.5) as u8;
    let b = ((h - PI / 3.0).sin() * 127.5 + 127.5) as u8;
    (r, g, b)
}

async fn turn_on_led() -> Result<(), Box<dyn Error>> {
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let adapter = adapters.into_iter().next().expect("No adapters found");
    adapter.start_scan(ScanFilter::default()).await?;

    // Scan for Bluetooth devices for 10 seconds.
    thread::sleep(Duration::from_secs(10));

    // Try to find a Sphero SPRK+ device.
    let device = find_sprk(&adapter).await;

    if let Some(device) = device {
        println!("Found device: {:?}", device);

        // Connect to the device.
        device.connect().await?;
        println!("Connected to device");

        // Wake up the device
        let anti_dos_characteristic_uuid = Uuid::parse_str("22bb746f-2bbd-7554-2d6f-726568705327")?;
        let tx_power_characteristic_uuid = Uuid::parse_str("22bb746f-2bb2-7554-2d6f-726568705327")?;
        let wakeup_characteristic_uuid = Uuid::parse_str("22bb746f-2bbf-7554-2d6f-726568705327")?;

        let characteristics = device.characteristics();

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
        ).await?;
        thread::sleep(Duration::from_millis(100)); // Add a short delay between write operations to make sure each operation is processed in order

        device.write(
            &tx_power_characteristic,
            &[0x07],
            WriteType::WithoutResponse,
        ).await?;
        thread::sleep(Duration::from_millis(100));

        device.write(&wakeup_characteristic, &[0x01], WriteType::WithoutResponse).await?;
        thread::sleep(Duration::from_millis(100));

        let characteristics = device.characteristics();

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

            let did = DeviceID::Sphero; // = device id
            let cid: u8 = SpheroCommandID::SetRGBLEDOutput as u8;
            let seq: u8 = 0x06; // = sequence number

            let deku_bytes = SpheroCommandPacketV1::new(did, cid, seq, vec![r, g, b]);
            let bytes_d = deku_bytes.to_bytes().unwrap();

            // Write to the characteristic.
            device.write(&led_char, &bytes_d, WriteType::WithoutResponse).await?;

            // Increase hue
            hue += 0.05;
            if hue > 2.0 * PI {
                hue -= 2.0 * PI;
            }

            // Wait for 50ms before sending the next packet
            thread::sleep(Duration::from_millis(50));
        }
    } else {
        println!("No Sphero SPRK+ found")
    };

    Ok(())
}

async fn find_sprk(central: &Adapter) -> Option<Peripheral> {
    for p in central.peripherals().await.unwrap() {
        if p.properties()
            .await
            .unwrap()
            .unwrap()
            .local_name
            .iter()
            .any(|name| name.contains("SK-"))
        {
            return Some(p);
        }
    }
    None
}

#[tokio::main]
async fn main() {
    match turn_on_led().await {
        Ok(_) => println!("Finished"),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}
