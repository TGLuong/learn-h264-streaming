#[derive(Debug, PartialEq, Eq)]
pub struct RtpPacket {
    pub sequence_number: u16,
    pub timestamp: u32,
    pub marker: bool,
    pub payload_type: u8,
    pub ssrc: u32,
    pub payload: Vec<u8>,
}

pub fn reconstruct_fu_a_payloads(payload: &Vec<Vec<u8>>) -> Vec<u8> {
    let mut res = vec![];
    for packet in payload.iter() {
        if let (Some(fu_indicator), Some(fu_header), Some(nal_payload)) =
            (packet.get(0), packet.get(1), packet.get(2..))
        {
            let fu_type = fu_indicator & 0x1f;
            let start = (fu_header & 0x80) >> 7;
            let end = (fu_header & 0x40) >> 6;

            println!("{fu_indicator:02x} {fu_header:02x} {fu_type} {start} {end}");
            if fu_type == 28 {
                let nal_header = (fu_indicator & 0xe0) | (fu_header & 0x1f);
                if start == 1 {
                    res.push(nal_header);
                }
                res.extend(nal_payload);
                if end == 1 {
                    return res;
                }
            }
        }
    }
    res
}

pub fn packetize_nal_as_rtp(
    nal: &[u8],
    mtu: usize,
    sequence_number: u16,
    timestamp: u32,
    payload_type: u8,
    ssrc: u32,
    marker: bool,
) -> Vec<RtpPacket> {
    if nal.len() <= mtu {
        return vec![RtpPacket {
            sequence_number,
            timestamp,
            marker,
            payload_type,
            ssrc,
            payload: nal.to_vec(),
        }];
    }
    assert!(mtu >= 3, "mtu must be at least 3 for FU-A fragmentation");

    let mut res = vec![];
    let mut sequence_number = sequence_number;
    let nal_header = nal[0];
    let mut index = 1;
    let chunk_size = mtu - 2;
    while index < nal.len() {
        let fu_indicator = (nal_header & 0xe0) | 28;
        if index == 1 {
            // first FU
            let fu_header = 0x80 | nal_header & 0x1f;
            let mut payload = vec![fu_indicator, fu_header];
            payload.extend(nal[index..index + chunk_size].to_vec());
            res.push(RtpPacket {
                sequence_number,
                timestamp,
                marker: false,
                payload_type,
                ssrc,
                payload,
            });
            sequence_number += 1;
            index += chunk_size;
        } else if index + chunk_size >= nal.len() {
            // last FU
            let fu_header = 0x40 | nal_header & 0x1f;
            let mut payload = vec![fu_indicator, fu_header];
            payload.extend(nal[index..].to_vec());
            res.push(RtpPacket {
                sequence_number,
                timestamp,
                marker,
                payload_type,
                ssrc,
                payload,
            });
            index = nal.len();
        } else {
            // middle FU
            let fu_header = nal_header & 0x1f;
            let mut payload = vec![fu_indicator, fu_header];
            payload.extend(nal[index..index + chunk_size].to_vec());
            res.push(RtpPacket {
                sequence_number,
                timestamp,
                marker: false,
                payload_type,
                ssrc,
                payload,
            });
            index += chunk_size;
            sequence_number += 1;
        }
    }
    res
}

fn next_nal_index(bytes: &[u8], mut index: usize) -> Option<usize> {
    while index + 3 <= bytes.len() {
        if let Some(slice) = bytes.get(index..index + 4) {
            if slice == [0x00, 0x00, 0x00, 0x01] {
                return Some(index);
            }
        }
        if let Some(slice) = bytes.get(index..index + 3) {
            if slice == [0x00, 0x00, 0x01] {
                return Some(index);
            }
        }
        index += 1;
    }
    None
}

pub fn find_nal_units_annex_b(bytes: &[u8]) -> Vec<&[u8]> {
    let mut res = vec![];
    let mut index = 0;
    while index + 3 <= bytes.len() {
        if let Some(annex_b) = bytes.get(index..index + 4) {
            if annex_b == [0x00, 0x00, 0x00, 0x01] {
                index += 4;
                match next_nal_index(bytes, index) {
                    Some(next) => {
                        if let Some(nal_slice) = bytes.get(index..next) {
                            res.push(nal_slice);
                        }
                        index = next;
                        continue;
                    }
                    None => {
                        if let Some(nal_slice) = bytes.get(index..) {
                            res.push(nal_slice);
                        }
                        index = bytes.len();
                        continue;
                    }
                }
            }
        }
        if let Some(annex_b) = bytes.get(index..index + 3) {
            if annex_b == [0x00, 0x00, 0x01] {
                index += 3;
                match next_nal_index(bytes, index) {
                    Some(next) => {
                        if let Some(nal_slice) = bytes.get(index..next) {
                            res.push(nal_slice);
                        }
                        index = next;
                        continue;
                    }
                    None => {
                        if let Some(nal_slice) = bytes.get(index..) {
                            res.push(nal_slice);
                        }
                        index = bytes.len();
                        continue;
                    }
                }
            }
        }
        index += 1;
    }
    res
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn finds_annex_b_nal_unit_byte_ranges_without_start_codes() {
        let bytes = [
            0x00, 0x00, 0x00, 0x01, 0x67, 0xaa, 0xbb, 0x00, 0x00, 0x01, 0x68, 0xcc, 0x00, 0x00,
            0x01, 0x65, 0xdd, 0xee,
        ];

        let nal_units = find_nal_units_annex_b(&bytes);

        assert_eq!(
            nal_units,
            vec![
                &[0x67, 0xaa, 0xbb][..],
                &[0x68, 0xcc][..],
                &[0x65, 0xdd, 0xee][..],
            ]
        );
    }

    #[test]
    fn finds_three_byte_start_code_at_end_boundary() {
        let bytes = [0x00, 0x00, 0x00, 0x01, 0x67, 0x00, 0x00, 0x01];

        assert_eq!(next_nal_index(&bytes, 5), Some(5));
    }

    #[test]
    fn packetizes_small_nal_as_single_rtp_packet() {
        let nal = [0x67, 0xaa, 0xbb];

        let packets = packetize_nal_as_rtp(&nal, 1200, 100, 3000, 96, 0x11223344, true);

        assert_eq!(
            packets,
            vec![RtpPacket {
                sequence_number: 100,
                timestamp: 3000,
                marker: true,
                payload_type: 96,
                ssrc: 0x11223344,
                payload: vec![0x67, 0xaa, 0xbb],
            }]
        );
    }

    #[test]
    fn fragments_large_nal_as_fu_a_packets() {
        let nal = [0x65, 0xaa, 0xbb, 0xcc, 0xdd, 0xee];

        let packets = packetize_nal_as_rtp(&nal, 5, 100, 3000, 96, 0x11223344, true);
        println!("{packets:?}");

        assert_eq!(packets.len(), 2);

        assert_eq!(
            packets[0],
            RtpPacket {
                sequence_number: 100,
                timestamp: 3000,
                marker: false,
                payload_type: 96,
                ssrc: 0x11223344,
                payload: vec![0x7c, 0x85, 0xaa, 0xbb, 0xcc],
            }
        );

        assert_eq!(
            packets[1],
            RtpPacket {
                sequence_number: 101,
                timestamp: 3000,
                marker: true,
                payload_type: 96,
                ssrc: 0x11223344,
                payload: vec![0x7c, 0x45, 0xdd, 0xee],
            }
        );
    }

    #[test]
    fn marks_exact_final_fu_a_chunk_as_end() {
        let nal = [0x65, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff];

        let packets = packetize_nal_as_rtp(&nal, 5, 100, 3000, 96, 0x11223344, true);

        assert_eq!(packets.len(), 2);

        assert_eq!(packets[0].payload, vec![0x7c, 0x85, 0xaa, 0xbb, 0xcc]);
        assert_eq!(packets[0].marker, false);

        assert_eq!(packets[1].payload, vec![0x7c, 0x45, 0xdd, 0xee, 0xff]);
        assert_eq!(packets[1].marker, true);
    }

    #[test]
    fn reconstructs_fu_a_payloads_as_original_nal_unit() {
        let fu_a_payloads = vec![
            vec![0x7c, 0x85, 0xaa, 0xbb, 0xcc],
            vec![0x7c, 0x45, 0xdd, 0xee],
        ];

        let nal = reconstruct_fu_a_payloads(&fu_a_payloads);

        assert_eq!(nal, vec![0x65, 0xaa, 0xbb, 0xcc, 0xdd, 0xee]);
    }
}
