/*!
 * Sphero Commands 
 */
use crate::packet::{SpheroCommandPacketV1, DeviceID, SpheroCommandID};

/// Sphero Command Conversion (requires seq)
pub trait ToCommandPacket {
    /// Convert to a Sphero Command Packet
    fn to_packet(&self, seq: u8) -> SpheroCommandPacketV1;
}

/// Sphero Set RGB LED Output Command
#[derive(Debug, Default)]
pub struct SetRGBLEDOutput {
    /// Red
    pub red: u8,
    /// Green
    pub green: u8,
    /// Blue
    pub blue: u8,
    /// Flag - persists across power cycles
    pub flag: bool,
}

/// Sphero Set Back LED Output Command
#[derive(Debug, Default)]
pub struct SetBackLEDOutput {
    /// Brightness of Fixed Color LED
    pub brightness: u8,
}

/// Sphero Roll Command
#[derive(Debug, Default)]
pub struct Roll {
    /// Speed
    pub speed: u8,
    /// Heading - 0..359 degrees
    pub heading: u16,
    /// (CES firmware) State - true = roll, false = stop
    pub state: bool,
}

impl ToCommandPacket for SetRGBLEDOutput {
    fn to_packet(&self, seq: u8) -> SpheroCommandPacketV1 {
        let did = DeviceID::Sphero; // = device id
        let cid: u8 = SpheroCommandID::SetRGBLEDOutput as u8;
        let seq: u8 = seq; // = sequence number

        let deku_bytes = SpheroCommandPacketV1::new(did, cid, seq, vec![self.red, self.green, self.blue, self.flag as u8]);
        deku_bytes
    }
}

impl ToCommandPacket for SetBackLEDOutput {
    fn to_packet(&self, seq: u8) -> SpheroCommandPacketV1 {
        let did = DeviceID::Sphero; // = device id
        let cid: u8 = SpheroCommandID::SetBackLEDOutput as u8;
        let seq: u8 = seq; // = sequence number

        let deku_bytes = SpheroCommandPacketV1::new(did, cid, seq, vec![self.brightness]);
        deku_bytes
    }
}

impl ToCommandPacket for Roll {
    fn to_packet(&self, seq: u8) -> SpheroCommandPacketV1 {
        let did = DeviceID::Sphero; // = device id
        let cid: u8 = SpheroCommandID::Roll as u8;
        let seq: u8 = seq; // = sequence number

        let deku_bytes = SpheroCommandPacketV1::new(did, cid, seq, vec![self.speed, (self.heading >> 8) as u8, self.heading as u8, self.state as u8]);
        deku_bytes
    }
}
