use murmurhash3::murmurhash3_x86_32;
use crate::utils::bitmap::BitMap;

pub struct BloomFilter {
    bit_map: BitMap,
    sends: [u32; 2],
}

impl BloomFilter {
    pub fn new() -> Self {

        Self { bit_map: BitMap::new(u32::MAX as usize), sends: [0x9747B28C, 0x0747B28D] }
    }

    pub fn add(&mut self, data: &str) -> bool {
        let hash1 = murmurhash3_x86_32(data.as_bytes(), self.sends[0]);
        let hash2 = murmurhash3_x86_32(data.as_bytes(), self.sends[1]);

        if self.bit_map.get(hash1 as usize) && self.bit_map.get(hash2 as usize) {
            return false;
        } else {
            self.bit_map.set(hash1 as usize);
            self.bit_map.set(hash2 as usize);
            return true;
        }
    }
}
