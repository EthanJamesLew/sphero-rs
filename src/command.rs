/*!
 * Sphero Commands
 */
use crate::packet::{DeviceID, SpheroCommandID, SpheroCommandPacketV1, CoreCommandID};

/// Sphero Command Conversion (requires seq)
pub trait ToCommandPacket {
    /// Convert to a Sphero Command Packet
    fn to_packet(&self, seq: u8) -> SpheroCommandPacketV1;
}

/// Sphero Ping Command
#[derive(Debug, Default)]
pub struct Ping {}

/// Sphero Get Versioning Command
#[derive(Debug, Default)]
pub struct GetVersioning {}

/// Sphero Get Bluetooth Info Command
#[derive(Debug, Default)]
pub struct GetBluetoothInfo {}

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

/// Sphero Set Streaming Data
#[derive(Debug, Default)]
pub struct SetDataStreaming {
    /// Divisor of the maximum sensor sampling rate
    /// the control system runs at 400Hz and because it's pretty unlikely
    /// you will want to see data at that rate, N allows you to divide
    /// that down. N = 2 yields data samples at 200Hz, N = 10, 40Hz, etc.
    pub n: u16,
    /// Number of sample frames emitted per packet
    /// M value defines how many frames to collect in memory before the
    /// packet is emitted
    pub m: u16,
    /// Bitwise selector of data sources to stream
    pub mask1: u32,
    /// Packet count (set to 0 for unlimited streaming)
    pub pcnt: u8,
    /// Bitwise selector of more data sources to stream (optional)
    pub mask2: Option<u32>,
}

impl ToCommandPacket for Ping {
    fn to_packet(&self, seq: u8) -> SpheroCommandPacketV1 {
        let did = DeviceID::Core; // = device id
        let cid: u8 = CoreCommandID::Ping as u8;
        let seq: u8 = seq; // = sequence number

        let deku_bytes = SpheroCommandPacketV1::new(did, cid, seq, vec![]);
        deku_bytes
    }
}

impl ToCommandPacket for GetVersioning {
    fn to_packet(&self, seq: u8) -> SpheroCommandPacketV1 {
        let did = DeviceID::Core; // = device id
        let cid: u8 = CoreCommandID::GetVersioningInformation as u8;
        let seq: u8 = seq; // = sequence number

        let deku_bytes = SpheroCommandPacketV1::new(did, cid, seq, vec![]);
        deku_bytes
    }
}

impl ToCommandPacket for GetBluetoothInfo {
    fn to_packet(&self, seq: u8) -> SpheroCommandPacketV1 {
        let did = DeviceID::Core; // = device id
        let cid: u8 = CoreCommandID::GetBluetoothInfo as u8;
        let seq: u8 = seq; // = sequence number

        let deku_bytes = SpheroCommandPacketV1::new(did, cid, seq, vec![]);
        deku_bytes
    }
}

impl ToCommandPacket for SetRGBLEDOutput {
    fn to_packet(&self, seq: u8) -> SpheroCommandPacketV1 {
        let did = DeviceID::Sphero; // = device id
        let cid: u8 = SpheroCommandID::SetRGBLEDOutput as u8;
        let seq: u8 = seq; // = sequence number

        let deku_bytes = SpheroCommandPacketV1::new(
            did,
            cid,
            seq,
            vec![self.red, self.green, self.blue, self.flag as u8],
        );
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

        let deku_bytes = SpheroCommandPacketV1::new(
            did,
            cid,
            seq,
            vec![
                self.speed,
                (self.heading >> 8) as u8,
                self.heading as u8,
                self.state as u8,
            ],
        );
        deku_bytes
    }
}

impl ToCommandPacket for SetDataStreaming {
    fn to_packet(&self, seq: u8) -> SpheroCommandPacketV1 {
        let did = DeviceID::Sphero; // = device id
        let cid: u8 = SpheroCommandID::SetDataStreaming as u8;
        let seq: u8 = seq; // = sequence number

        // if mask2: data = [0xfh [N bid endian] [M big endian] [mask1 big endian] [pcnt] [mask2 big endian]]
        // else: data = [0xfh [N bid endian] [M big endian] [mask1 big endian] [pcnt]]
        let nbs = self.n.to_be_bytes();
        let mbs = self.m.to_be_bytes();
        let m1bs = self.mask1.to_be_bytes();

        match self.mask2 {
            Some(mask2) => {
                let m2bs = mask2.to_be_bytes();
                SpheroCommandPacketV1::new(
                    did,
                    cid,
                    seq,
                    vec![
                        nbs[0], nbs[1], mbs[0], mbs[1], m1bs[0], m1bs[1], m1bs[2], m1bs[3],
                        self.pcnt, m2bs[0], m2bs[1], m2bs[2], m2bs[3],
                    ],
                )
            }
            None => SpheroCommandPacketV1::new(
                did,
                cid,
                seq,
                vec![
                    nbs[0], nbs[1], mbs[0], mbs[1], m1bs[0], m1bs[1], m1bs[2], m1bs[3], self.pcnt,
                ],
            ),
        }
    }
}
