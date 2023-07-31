/*!
 * Sphero Packet
 */
use std::convert::TryInto;

/// Define Sphero packet structure
#[derive(Debug)]
pub struct SpheroPacket {
    sop: u8,
    flags: u8,
    tid: Option<u8>,
    sid: Option<u8>,
    did: u8,
    cid: u8,
    seq: u8,
    err: Option<u8>,
    data: Vec<u8>,
    chk: u8,
    eop: u8,
}

impl SpheroPacket {
    /// Create a new packet
    pub fn new(
        flags: u8,
        tid: Option<u8>,
        sid: Option<u8>,
        did: u8,
        cid: u8,
        seq: u8,
        err: Option<u8>,
        data: Vec<u8>,
    ) -> Self {
        let sop = 0x8D;
        let eop = 0xD8;
        let chk = calculate_checksum(
            &[
                flags,
                tid.unwrap_or(0),
                sid.unwrap_or(0),
                did,
                cid,
                seq,
                err.unwrap_or(0),
            ],
            &data,
        );
        SpheroPacket {
            sop,
            flags,
            tid,
            sid,
            did,
            cid,
            seq,
            err,
            data,
            chk,
            eop,
        }
    }
}

/// SLIP Encoding
pub fn slip_encode(packet: &mut SpheroPacket) {
    packet.data = packet
        .data
        .iter()
        .flat_map(|&x| match x {
            0xAB => vec![0xAB, 0x23],
            0x8D => vec![0xAB, 0x05],
            0xD8 => vec![0xAB, 0x50],
            _ => vec![x],
        })
        .collect();
}

/// SLIP Decoding
pub fn slip_decode(packet: &mut SpheroPacket) {
    packet.data = packet
        .data
        .windows(2)
        .flat_map(|x| match x {
            [0xAB, 0x23] => vec![0xAB],
            [0xAB, 0x05] => vec![0x8D],
            [0xAB, 0x50] => vec![0xD8],
            _ => vec![x[0]],
        })
        .collect();
}

/// Checksum calculation
pub fn calculate_checksum(header: &[u8], data: &[u8]) -> u8 {
    let sum: u8 = header
        .iter()
        .chain(data.iter())
        .fold(0u8, |acc, &x| acc.wrapping_add(x));
    !sum
}

/// Converting Packet to Vec<u8> for sending
pub fn to_bytes(packet: &SpheroPacket) -> Vec<u8> {
    let mut bytes = vec![packet.sop, packet.flags];
    if let Some(tid) = packet.tid {
        bytes.push(tid);
    }
    if let Some(sid) = packet.sid {
        bytes.push(sid);
    }
    bytes.extend_from_slice(&[packet.did, packet.cid, packet.seq]);
    if let Some(err) = packet.err {
        bytes.push(err);
    }
    bytes.extend(&packet.data);
    bytes.push(packet.chk);
    bytes.push(packet.eop);
    bytes
}

/// Parsing a packet from Vec<u8> received from the device
pub fn from_bytes(bytes: &[u8]) -> Result<SpheroPacket, &'static str> {
    if bytes.len() < 5 || bytes[0] != 0x8D || bytes[bytes.len() - 1] != 0xD8 {
        return Err("Invalid packet");
    }

    let sop = bytes[0];
    let flags = bytes[1];
    let (tid, sid) = if bytes.len() > 7 && flags & 0x30 != 0 {
        (Some(bytes[2]), Some(bytes[3]))
    } else {
        (None, None)
    };
    let did = bytes[bytes.len() - 5];
    let cid = bytes[bytes.len() - 4];
    let seq = bytes[bytes.len() - 3];
    let err = if flags & 0x01 != 0 {
        Some(bytes[bytes.len() - 2])
    } else {
        None
    };
    let data = bytes[4..bytes.len() - 6].to_vec();
    let chk = bytes[bytes.len() - 2];
    let eop = bytes[bytes.len() - 1];
    let packet = SpheroPacket {
        sop,
        flags,
        tid,
        sid,
        did,
        cid,
        seq,
        err,
        data,
        chk,
        eop,
    };

    Ok(packet)
}
