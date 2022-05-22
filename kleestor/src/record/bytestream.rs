use std::cmp::{min, Ordering};
use std::simd::Simd;

#[derive(Eq, Ord)]
pub struct ByteStream {
    data: Vec<u8>,
}

impl ByteStream {
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

    /// Compare equality of a reference against another reference using SIMD.
    pub fn ref_2_eq(this: &[u8], other: &[u8]) -> bool {
        let n = this.len();
        let m = other.len();

        if n != m {
            return false;
        }

        // // compare the rest of the bytes
        // for j in 0..n {
        //     if this[j] != other[j] {
        //         return false;
        //     }
        // }

        // compare data in parallel
        let mut i = 16;
        while i < n {
            let left: Simd<u8, 16> = Simd::from_slice(&this[i - 16..i]);
            let right: Simd<u8, 16> = Simd::from_slice(&other[i - 16..i]);
            if left != right {
                return false;
            }
            i += 16;
        }

        // compare the rest of the bytes
        for j in i - 16..n {
            if this[j] != other[j] {
                return false;
            }
        }
        return true;
    }

    /// Compare equality against another reference using SIMD.
    pub fn ref_eq(&self, other: &[u8]) -> bool {
        Self::ref_2_eq(&self.data, other)
    }

    /// Compare against another reference using SIMD.
    pub fn ref_2_partial_cmp(this: &[u8], other: &[u8]) -> Option<Ordering> {
        let n = this.len();
        let m = other.len();
        let len = min(n, m);

        // // compare the rest of the bytes
        // for j in 0..len {
        //     let left = this[j];
        //     let right = other[j];
        //     if left == right {
        //         continue;
        //     } else if left < right {
        //         return Some(Ordering::Less);
        //     } else {
        //         return Some(Ordering::Greater);
        //     }
        // }

        // compare data in parallel
        let mut i = 16;
        while i < len {
            let left: Simd<u8, 16> = Simd::from_slice(&this[i - 16..i]);
            let right: Simd<u8, 16> = Simd::from_slice(&other[i - 16..i]);
            if left == right {
                i += 16;
                continue;
            }
            // compare for differences
            for j in i - 16..i {
                let left = this[j];
                let right = other[j];
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
        for j in i - 16..len {
            let left = this[j];
            let right = other[j];
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

    pub fn ref_partial_cmp(&self, other: &[u8]) -> Option<Ordering> {
        Self::ref_2_partial_cmp(self.as_ref(), other)
    }
}

impl PartialEq for ByteStream {
    /// Compare strings against each other using SIMD.
    ///
    /// Priority is guaranteed over `partial_cmp`.
    fn eq(&self, other: &Self) -> bool {
        self.ref_eq(other.as_ref())
    }
}

impl PartialOrd for ByteStream {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.ref_partial_cmp(other.as_ref())
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
