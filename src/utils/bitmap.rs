pub struct BitMap {
    data: Vec<u8>,
}

impl BitMap {
    pub fn new(size: usize) -> Self {
        Self { data: vec![0; size / 8 + 1] }
    }

    pub fn set(&mut self, index: usize) {
        let data_index = index / 8;
        let bit_index = index % 8;
        self.data[data_index] |= 1 << bit_index;
    }

    pub fn get(&self, index: usize) -> bool {
        let data_index = index / 8;
        let bit_index = index % 8;
        return self.data[data_index] & (1 << bit_index) != 0;
    }
}