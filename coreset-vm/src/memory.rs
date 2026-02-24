pub struct Memory(Vec<u64>);

impl Memory {
    pub fn new(size: usize) -> Self {
        Memory(vec![0; size])
    }

    pub fn small() -> Self {
        Memory(vec![0; 64])
    }

    pub fn medium() -> Self {
        Memory(vec![0; 256])
    }

    pub fn large() -> Self {
        Memory(vec![0; 1024])
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Expose raw cell data for display/debugging.
    pub fn data(&self) -> &[u64] {
        &self.0
    }

    pub fn read(&self, addr: u64) -> u64 {
        self.0[addr as usize]
    }

    pub fn write(&mut self, addr: u64, value: u64) {
        self.0[addr as usize] = value;
    }

    /// Zero all cells.
    pub fn reset(&mut self) {
        self.0.iter_mut().for_each(|v| *v = 0);
    }
}
