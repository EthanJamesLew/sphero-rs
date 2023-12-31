/*!
 * Sphero Packet
 * 
 * Implements the packet structures defined in the v1.20 API
 * 
 * A lightweight command and control protocol which you can use 
 * to build up applications offering a higher level of functionality. 
 * 
 * Multi-byte numbers are sent MSB first in both directions
 */
use deku::prelude::*;

/// Sphero Command Packet V1
/// <https://docs.gosphero.com/api/Sphero_API_1.20.pdf> (Page 7)
#[derive(Default, Debug, PartialEq, DekuRead, DekuWrite)]
pub struct SpheroCommandPacketV1 {
    sop1: SOP1Field,
    sop2: SOP2Field,
    did: DeviceID,
    cid: u8,
    seq: u8,
    #[deku(update = "self.data.len() + 1")]
    dlen: u8,
    #[deku(count = "dlen - 1")]
    data: Vec<u8>,
    #[deku(
        update = "calculate_checksum(&[self.did as u8, self.cid, self.seq, self.dlen], &self.data)"
    )]
    chk: u8,
}

/// Sphero Response Packet V1
/// <https://docs.gosphero.com/api/Sphero_API_1.20.pdf> (Page 7)
#[derive(Default, Debug, PartialEq, DekuRead, DekuWrite)]
pub struct SpheroResponsePacketV1 {
    sop1: SOP1Field,
    sop2: SOP2Field,
    mrsp: MRSPField,
    seq: u8,
    #[deku(update = "self.data.len() + 1")]
    dlen: u8,
    #[deku(count = "dlen - 1")]
    data: Vec<u8>,
    #[deku(update = "calculate_checksum(&[self.mrsp as u8, self.seq, self.dlen], &self.data)")]
    chk: u8,
}

/// Sphero Asynchronous Packet V1
/// <https://docs.gosphero.com/api/Sphero_API_1.20.pdf> (Page 9)
#[derive(Default, Debug, PartialEq, DekuRead, DekuWrite)]
pub struct SpheroAsynchronousPacketV1 {
    sop1: SOP1Field,
    sop2: SOP2Field,
    idcode: u8,
    #[deku(update = "self.data.len() + 1")]
    dlen: u16,
    #[deku(count = "dlen - 1")]
    data: Vec<u8>,
    #[deku(
        update = "calculate_checksum(&[self.idcode, (self.dlen >> 8) as u8, self.dlen as u8], &self.data)"
    )]
    chk: u8,
}

impl SpheroCommandPacketV1 {
    /// Create a new packet
    pub fn new(did: DeviceID, sid: u8, seq: u8, data: Vec<u8>) -> Self {
        let chk = calculate_checksum(&[did as u8, sid, seq, data.len() as u8 + 1], &data);
        Self {
            sop1: SOP1Field::All,
            sop2: SOP2Field::Response,
            did: did,
            cid: sid,
            seq: seq,
            dlen: data.len() as u8 + 1,
            data: data,
            chk: chk,
        }
    }
}

/// Checksum calculation
/// modulo 256 sum of all the bytes from the DID through the end of the data payload,
/// bit inverted (1's complement)
pub fn calculate_checksum(fields: &[u8], data: &[u8]) -> u8 {
    let sum: u8 = fields
        .iter()
        .chain(data.iter())
        .fold(0u8, |acc, &x| acc.wrapping_add(x));
    !sum
}

/// Sphero Packet SOP1 Values
#[derive(Default, Debug, PartialEq, Clone, Copy, DekuRead, DekuWrite)]
#[deku(type = "u8", endian = "big")]
pub enum SOP1Field {
    /// Acknowledgement Required (Command) or Acknowledgement (Response)
    #[default]
    #[deku(id = "0xff")]
    All = 0xff,
}

/// Sphero Packet SOP2 Values
#[derive(Default, Debug, PartialEq, Clone, Copy, DekuRead, DekuWrite)]
#[deku(type = "u8", endian = "big")]
pub enum SOP2Field {
    /// Acknowledgement Required (Command) or Acknowledgement (Response)
    #[default]
    #[deku(id = "0xff")]
    Response = 0xff,
    /// Asynchronous Message
    #[deku(id = "0xfe")]
    Async = 0xfe,
}

/// Sphero Message Response Codes
/// <https://docs.gosphero.com/api/Sphero_API_1.20.pdf> (Page 44)
#[derive(Default, Debug, PartialEq, Clone, Copy, DekuRead, DekuWrite)]
#[deku(type = "u8", endian = "big")]
pub enum MRSPField {
    /// Command succeeded
    #[default]
    #[deku(id = "0x00")]
    Ok = 0x00,
    /// General, non-specific error
    #[deku(id = "0x01")]
    GeneralError = 0x01,
    /// Received a bad checksum
    #[deku(id = "0x02")]
    ChecksumError = 0x02,
    /// Received command fragment
    #[deku(id = "0x03")]
    FragmentError = 0x03,
    /// Unknown command ID
    #[deku(id = "0x04")]
    UnknownCommandError = 0x04,
    /// Command currently unsupported
    #[deku(id = "0x05")]
    UnsupportedCommandError = 0x05,
    /// Bad message format
    #[deku(id = "0x06")]
    BadMessageFormatError = 0x06,
    /// Parameter value(s) invalid
    #[deku(id = "0x07")]
    InvalidParameterError = 0x07,
    /// Failed to execute command
    #[deku(id = "0x08")]
    ExecuteError = 0x08,
    /// Unknown device ID
    #[deku(id = "0x09")]
    UnknownDeviceError = 0x09,
    /// voltage too low for reflash operation
    #[deku(id = "0x31")]
    LowVoltageError = 0x31,
    /// Illegal page number provided
    #[deku(id = "0x32")]
    IllegalPageError = 0x32,
    /// Page did not reprogram correctly
    #[deku(id = "0x33")]
    FlashFailError = 0x33,
    /// Main application corrupt
    #[deku(id = "0x34")]
    MainAppCorruptError = 0x34,
    /// Msg state machine timed out
    #[deku(id = "0x35")]
    MsgTimeoutError = 0x35,
}

/// Sphero Device ID
/// From the spec: "Sphero is an actual device (obviously) but in his 
/// core software, many virtual devices are implemented."
#[derive(Default, Debug, PartialEq, Clone, Copy, DekuRead, DekuWrite)]
#[deku(type = "u8", endian = "big")]
pub enum DeviceID {
    /// The Core
    #[deku(id = "0x00")]
    Core = 0x00,
    /// Bootloader
    #[deku(id = "0x01")]
    Bootloader = 0x01,
    /// Sphero
    #[default]
    #[deku(id = "0x02")]
    Sphero = 0x02,
}

/// Device ID 00h – The Core
/// <https://docs.gosphero.com/api/Sphero_API_1.20.pdf> (Page 11)
/// The Core Device encapsulates actions that are fundamental to all Orbotix devices.
#[derive(Debug, PartialEq, Clone, Copy, DekuRead, DekuWrite)]
#[deku(type = "u8", endian = "big")]
pub enum CoreCommandID {
    /// Ping
    #[deku(id = "0x01")]
    Ping = 0x01,
    /// Get Versioning Information
    #[deku(id = "0x02")]
    GetVersioningInformation = 0x02,
    /// Set Device Name
    #[deku(id = "0x10")]
    SetDeviceName = 0x10,
    /// Get Bluetooth Info
    #[deku(id = "0x11")]
    GetBluetoothInfo = 0x11,
    /// Set Auto Reconnect
    #[deku(id = "0x12")]
    GetAutoReconnect = 0x12,
    /// Get Auto Reconnect
    #[deku(id = "0x13")]
    SetAutoReconnect = 0x13,
    /// Get Power State
    #[deku(id = "0x20")]
    GetPowerState = 0x20,
    /// Set Power Notification
    #[deku(id = "0x21")]
    SetPowerNotification = 0x21,
    /// Sleep
    #[deku(id = "0x22")]
    Sleep = 0x22,
    /// Get Voltage Trip Points
    #[deku(id = "0x23")]
    GetVoltageTripPoints = 0x23,
    /// Set Voltage Trip Points
    #[deku(id = "0x24")]
    SetVoltageTripPoints = 0x24,
    /// Set Inactivity Timeout
    #[deku(id = "0x25")]
    SetInactivityTimeout = 0x25,
    /// Jump To Bootloader
    #[deku(id = "0x30")]
    JumpToBootloader = 0x30,
    /// Perform Level 1 Diagnostics
    #[deku(id = "0x40")]
    PerformLevel1Diagnostics = 0x40,
    /// Perform Level 2 Diagnostics
    #[deku(id = "0x41")]
    PerformLevel2Diagnostics = 0x41,
    /// Clear Counters
    #[deku(id = "0x42")]
    ClearCounters = 0x42,
    /// Assign Time Value
    #[deku(id = "0x50")]
    AssignTimeValue = 0x50,
    /// Poll Packet Times
    #[deku(id = "0x51")]
    PollPacketTimes = 0x51,
}

/// Device ID 01h – Bootloader
/// <https://docs.gosphero.com/api/Sphero_API_1.20.pdf> (Page 22)
#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8", endian = "big")]
pub enum BootloaderCommandID {
    /// Reflash
    #[deku(id = "0x02")]
    Reflash = 0x02,
    /// Here is Page
    #[deku(id = "0x03")]
    HereIsPage = 0x03,
    /// Leave Bootloader
    #[deku(id = "0x04")]
    LeaveBootloader = 0x04,
    /// Is Page Blank
    #[deku(id = "0x05")]
    IsPageBlank = 0x05,
    /// Erase User Config
    #[deku(id = "0x06")]
    EraseUserConfig = 0x06,
}

/// Device ID 02h – Sphero
/// <https://docs.gosphero.com/api/Sphero_API_1.20.pdf> (Page 23)
#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8", endian = "big")]
pub enum SpheroCommandID {
    /// Set Heading
    #[deku(id = "0x01")]
    SetHeading = 0x01,
    /// Set Stabilization
    #[deku(id = "0x02")]
    SetStabilization = 0x02,
    /// Set Rotation Rate
    #[deku(id = "0x03")]
    SetRotationRate = 0x03,
    /// Set Application Configuration Block
    #[deku(id = "0x04")]
    SetApplicationConfigurationBlock = 0x04,
    /// Get Application Configuration Block
    #[deku(id = "0x05")]
    GetApplicationConfigurationBlock = 0x05,
    /// Re-Enable Demo
    #[deku(id = "0x06")]
    ReEnableDemo = 0x06,
    /// Get Chassis ID
    #[deku(id = "0x07")]
    GetChassisID = 0x07,
    /// Set Chassis ID
    #[deku(id = "0x08")]
    SetChassisID = 0x08,
    /// Self Level
    #[deku(id = "0x09")]
    SelfLevel = 0x09,
    /// Set Data Streaming
    #[deku(id = "0x11")]
    SetDataStreaming = 0x11,
    /// Configure Collision Detection
    #[deku(id = "0x12")]
    ConfigureCollisionDetection = 0x12,
    /// Set RGB LED Output
    #[deku(id = "0x20")]
    SetRGBLEDOutput = 0x20,
    /// Set Back LED Output
    #[deku(id = "0x21")]
    SetBackLEDOutput = 0x21,
    /// Get RGB LED Output
    #[deku(id = "0x22")]
    GetRGBLEDOutput = 0x22,
    /// Roll
    #[deku(id = "0x30")]
    Roll = 0x30,
    /// Set Boost With Time
    #[deku(id = "0x31")]
    SetBoostWithTime = 0x31,
    /// Set Raw Motor Values
    #[deku(id = "0x33")]
    SetRawMotorValues = 0x33,
    /// Set Motion Timeout
    #[deku(id = "0x34")]
    SetMotionTimeout = 0x34,
    /// Set Options Flags
    #[deku(id = "0x35")]
    SetOptionsFlags = 0x35,
    /// Get Options Flags
    #[deku(id = "0x36")]
    GetOptionsFlags = 0x36,
    /// Get Configuration Block
    #[deku(id = "0x40")]
    GetConfigurationBlock = 0x40,
    /// Set Device Mode
    #[deku(id = "0x42")]
    SetDeviceMode = 0x42,
    /// Set Configuration Block
    #[deku(id = "0x43")]
    SetConfigurationBlock = 0x43,
    /// Get Device Mode
    #[deku(id = "0x44")]
    GetDeviceMode = 0x44,
    /// Run Macro
    #[deku(id = "0x50")]
    RunMacro = 0x50,
    /// Save Temporary Macro
    #[deku(id = "0x51")]
    SaveTemporaryMacro = 0x51,
    /// Save Macro
    #[deku(id = "0x52")]
    SaveMacro = 0x52,
    /// Reinit Macro Executive
    #[deku(id = "0x54")]
    ReinitMacroExecutive = 0x54,
    /// Abort Macro
    #[deku(id = "0x55")]
    AbortMacro = 0x55,
    /// Get Macro Status
    #[deku(id = "0x56")]
    GetMacroStatus = 0x56,
    /// Set Macro Parameter
    #[deku(id = "0x57")]
    SetMacroParameter = 0x57,
    /// Append Macro Chunk
    #[deku(id = "0x58")]
    AppendMacroChunk = 0x58,
    /// Erase Orbbasic Storage
    #[deku(id = "0x60")]
    EraseOrbbasicStorage = 0x60,
    /// Append Orbbasic Fragment
    #[deku(id = "0x61")]
    AppendOrbbasicFragment = 0x61,
    /// Execute Orbbasic Program
    #[deku(id = "0x62")]
    ExecuteOrbbasicProgram = 0x62,
    /// Abort Orbbasic Program
    #[deku(id = "0x63")]
    AbortOrbbasicProgram = 0x63,
}
