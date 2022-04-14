use std::cmp::{min, Ordering};
use std::simd::Simd;

#[derive(Eq, Ord)]
pub struct ByteStream {
    data: Vec<u8>,
}

impl ByteStream {
    pub fn new() -> Self {
        Self { data: vec![] }
    }

    pub fn from_vec(bytes: Vec<u8>) -> Self {
        Self { data: bytes }
    }

    pub fn from_slice(bytes: &[u8]) -> Self {
        Self { data: Vec::from(bytes) }
    }

    pub fn as_ref(&self) -> &[u8] {
        self.data.as_ref()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl PartialEq for ByteStream {
    /// Compare strings against each other using SIMD.
    ///
    /// Priority is guaranteed over `partial_cmp`.
    fn eq(&self, other: &Self) -> bool {
        let n = self.data.len();
        let m = other.data.len();

        if n != m {
            return false;
        }

        // compare data in parallel
        let mut i = 16;
        while i < n {
            let left: Simd<u8, 16> = Simd::from_slice(&self.data[i - 16..i]);
            let right: Simd<u8, 16> = Simd::from_slice(&other.data[i - 16..i]);
            if left != right {
                return false;
            }
            i += 16;
        }

        // compare the rest of the bytes
        for j in i - 16..n {
            if self.data[j] != other.data[j] {
                return false;
            }
        }
        return true;
    }
}

impl PartialOrd for ByteStream {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let n = self.data.len();
        let m = other.data.len();
        let len = min(n, m);

        // compare data in parallel
        let mut i = 16;
        while i < len {
            let left: Simd<u8, 16> = Simd::from_slice(&self.data[i - 16..i]);
            let right: Simd<u8, 16> = Simd::from_slice(&other.data[i - 16..i]);
            if left == right {
                i += 16;
                continue;
            }
            // compare for differences
            for j in i - 16..i {
                let left = self.data[j];
                let right = other.data[j];
                if left == right {
                    continue;
                } else if left < right {
                    return Some(Ordering::Less);
                } else {
                    return Some(Ordering::Greater);
                }
            }
            // unreachable code here
        }

        // compare the rest of the bytes
        for j in i - 16..n {
            let left = self.data[j];
            let right = other.data[j];
            if left == right {
                continue;
            } else if left < right {
                return Some(Ordering::Less);
            } else {
                return Some(Ordering::Greater);
            }
        }

        // whichever longer is greater
        n.partial_cmp(&m)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ensures_eq() {
        for len in 0..=32 {
            let mut left = Vec::<u8>::new();
            let mut right = Vec::<u8>::new();
            for i in 0..len {
                left.push(i + 0x23);
                right.push(i + 0x23);
            }
            assert!(left == right, "vec[len={len}] == vec[len={len}]");
            assert!(left <= right, "vec[len={len}] <= vec[len={len}]");
            assert!(left >= right, "vec[len={len}] >= vec[len={len}]");
            assert!(!(left != right), "vec[len={len}] ! == vec[len={len}]");
            assert!(!(left < right), "vec[len={len}] ! < vec[len={len}]");
            assert!(!(left > right), "vec[len={len}] ! > vec[len={len}]");
        }
    }

    #[test]
    /// Tests inequality between two strings of an equal length.
    fn ensures_ne_eqlen() {
        for len in 1..=32 {
            for err_pos in 0..len {
                let mut left = Vec::<u8>::new();
                let mut right = Vec::<u8>::new();
                for i in 0..len {
                    left.push(i + 0x23);
                    if i != err_pos {
                    } else {
                        right.push(i + 0x23);
                        right.push(i + 0xaf);
                    }
                }
                assert!(left < right, "vec[len={len},err_pos={err_pos}] @ <");
                assert!(left <= right, "vec[len={len},err_pos={err_pos}] @ <=");
                assert!(left != right, "vec[len={len},err_pos={err_pos}] @ !=");
                assert!(!(left > right), "vec[len={len},err_pos={err_pos}] @ ! >");
                assert!(!(left >= right), "vec[len={len},err_pos={err_pos}] @ ! >=");
                assert!(!(left == right), "vec[len={len},err_pos={err_pos}] @ ! ==");
            }
        }
    }

    #[test]
    /// Tests inequality between two strings of the same prefix, but unequal
    /// lengths.
    fn ensures_ne_nelen() {
        for len in 0..=32 {
            let mut left = Vec::<u8>::new();
            let mut right = Vec::<u8>::new();
            for i in 0..len {
                left.push(i + 0x23);
                right.push(i + 0x23);
            }
            right.push(0x00);
            assert!(left < right, "vec[len={len}] @ <");
            assert!(!(left > right), "vec[len={len}] @ ! >");
        }
    }
}
