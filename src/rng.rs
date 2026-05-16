pub fn hash_seed(input: &str) -> u64 {
    let mut h: u64 = 14695981039346656037;
    for b in input.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    h
}

pub struct XorShift64(pub u64);

impl XorShift64 {
    pub fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

pub fn shuffle<T>(v: &mut [T], rng: &mut XorShift64) {
    let n = v.len();
    for i in (1..n).rev() {
        let j = (rng.next() as usize) % (i + 1);
        v.swap(i, j);
    }
}
