pub struct Xorshift128pState {
    a: u64,
    b: u64,
}

// random number generator
// based on https://en.wikipedia.org/wiki/Xorshift
impl Xorshift128pState {
    pub fn new(seed: u64) -> Xorshift128pState {
        let b = seed * 34;
        let mut res = Xorshift128pState { a: seed, b: b };
        // drop some samples
        for _i in 0..6 {
            res.gen();
        }
        res
    }

    pub fn gen(&mut self) -> u64 {
        let mut t: u64 = self.a;
        let s: u64 = self.b;
        self.a = s;
        t ^= t << 23; // a
        t ^= t >> 17; // b
        t ^= s ^ (s >> 26); // c
        self.b = t;
        return t.wrapping_add(s);
    }

    pub fn gen_min_max(&mut self, min: u64, max: u64) -> u64 {
        let n = self.gen();
        n / (u64::MAX / (max - min)) + min
    }
}