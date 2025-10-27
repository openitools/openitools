use airplay::config::MacAddr6;
use fastrand::u8 as ru8;

pub fn random_lla_mac() -> MacAddr6 {
    let mut bytes = [0u8; 6];

    for b in &mut bytes {
        *b = ru8(..);
    }

    // Set locally administered bit (bit 1) and ensure unicast (bit 0 = 0)
    bytes[0] = (bytes[0] & 0b1111_1100) | 0b0000_0010;

    MacAddr6::new(bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5])
}
