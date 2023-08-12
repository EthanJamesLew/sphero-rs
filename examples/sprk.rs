use btleplug;
use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter, WriteType};
use btleplug::platform::{Adapter, Manager, Peripheral as PeripheralPlatform};
use futures::stream::StreamExt;
use sphero_rs::command::{Roll, SetDataStreaming, SetRGBLEDOutput, ToCommandPacket};
use sphero_rs::packet::{SpheroAsynchronousPacketV1, SpheroResponsePacketV1};
use std::error::Error;
use std::thread;
use std::time::Duration;
use uuid::Uuid;

use deku::{DekuContainerRead, DekuContainerWrite};

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
        let read_char_uuid = Uuid::parse_str("22bb746f-2ba6-7554-2d6f-726568705327").unwrap();

        // Collect the characteristics we want to interact with
        device.discover_services().await?;
        let characteristics = device.characteristics();

        // print out characteristics
        for ch in characteristics.iter() {
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

        let read_char = characteristics
            .iter()
            .find(|c| c.uuid == read_char_uuid)
            .ok_or("Receive not found")?
            .clone();

        device
            .write(
                &anti_dos_characteristic,
                b"011i3",
                WriteType::WithoutResponse,
            )
            .await?;
        thread::sleep(Duration::from_millis(100)); // Add a short delay between write operations to make sure each operation is processed in order

        device
            .write(
                &tx_power_characteristic,
                &[0x07],
                WriteType::WithoutResponse,
            )
            .await?;
        thread::sleep(Duration::from_millis(100));

        device
            .write(&wakeup_characteristic, &[0x01], WriteType::WithoutResponse)
            .await?;
        thread::sleep(Duration::from_millis(100));

        // Spin up a new thread to continuously read notifications from the characteristic.
        let device_clone = device.clone();
        tokio::spawn(async move {
            loop {
                // Subscribe to the characteristic.
                device_clone.subscribe(&read_char).await.unwrap();

                let mut notification_stream = device_clone.notifications().await.unwrap().take(8);
                // Process while the BLE connection is not broken or stopped.
                while let Some(data) = notification_stream.next().await {
                    let response = SpheroResponsePacketV1::from_bytes((data.value.as_ref(), 0));
                    let response_async =
                        SpheroAsynchronousPacketV1::from_bytes((data.value.as_ref(), 0));
                    match response {
                        Ok((_, response)) => {
                            println!("Received data from [{:?}]: {:?}", data.uuid, response);
                        }
                        Err(e) => match response_async {
                            Ok((_, response_async)) => {
                                println!(
                                    "Received data from [{:?}]: {:?}",
                                    data.uuid, response_async
                                );
                            }
                            Err(e) => {
                                println!(
                                    "Received data from [{:?}]: {:?}, {:?}, {:?}",
                                    data.uuid,
                                    data.value.as_slice(),
                                    data.value,
                                    e
                                );
                            }
                        },
                    }
                }
                // Sleep for a bit before trying to read the next notification.
                thread::sleep(Duration::from_millis(50));
            }
        });

        // Find the characteristic to write to.
        // The actual UUID might differ.
        let led_char_uuid = Uuid::parse_str("22bb746f-2ba1-7554-2d6f-726568705327").unwrap();
        let led_char = characteristics
            .into_iter()
            .find(|c| c.uuid == led_char_uuid)
            .unwrap();

        // Start with a hue of 0
        let mut hue: f32 = 0.0;

        // set the streaming mask
        let bytes_d = SetDataStreaming {
            n: 2,
            m: 1,
            mask1: 0xffffffff,
            pcnt: 0,
            mask2: None,
        }
        .to_packet(0xff)
        .to_bytes()
        .unwrap();
        device
            .write(&led_char, &bytes_d, WriteType::WithoutResponse)
            .await?;

        // Loop to run forever
        loop {
            // Convert hue to RGB
            let (r, g, b) = hsv_to_rgb(hue);

            let bytes_d = SetRGBLEDOutput {
                red: r,
                green: g,
                blue: b,
                flag: false,
            }
            .to_packet(0x07)
            .to_bytes()
            .unwrap();

            // Write to the characteristic.
            // device.write(&led_char, &bytes_d, WriteType::WithoutResponse).await?;

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

async fn find_sprk(central: &Adapter) -> Option<PeripheralPlatform> {
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
