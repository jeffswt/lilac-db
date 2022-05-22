use std::fs::File;
use std::io::{self, Error, ErrorKind, Result, Write};
use std::mem::MaybeUninit;

/// Stores unsigned integer values of up to 64 bits (2^64) in up to 9 bytes.
///
/// The varuint64 format is so designed to meet the following demands:
///
///   * Support for small values, particularly those under 128 (2^7), which
///     occur significantly more frequent in its use cases.
///   * Optimized performance for fast reading, especially for small values.
///     Values under 128 should be read with only 1 seek.
///   * Support for large values must be available, and reading them should
///     take as few seeks as possible.
///   * Security: buffer overflow should be avoided wherever possible. Memory
///     access sanity check is always performed.
///
/// The 9 bytes in varuint64 is ordered in little endian with additional bits
/// considered so as to support variable length:
///
///   * Byte #0: [1-bit flag] [data bits 6 ~ 0]
///     The 1-bit flag is 0 if no data bits follows, and is 1 if 7 bits are not
///         enough to represent the data.
///   * Byte #1: [len4] [len2] [len1] [data bits 11 ~ 7]
///     `len4 * 4 + len2 * 2 + len1` represents the total number of following
///         bytes (byte #1 excluded). It is hence possible to represent up to
///         76 bits with this format, but is not actively supported.
///     For example, a 10-bit varuint64 would have [len4 len2 len1] in the form
///         of [0 0 0], while a 13-bit one would have [0 0 1] instead.
///   * Byte #2: [data bits 19 ~ 12]
///   * Byte #3: [data bits 27 ~ 20]
///   * Byte #4: [data bits 35 ~ 28]
///   * Byte #5: [data bits 43 ~ 36]
///   * Byte #6: [data bits 51 ~ 44]
///   * Byte #7: [data bits 59 ~ 52]
///   * Byte #8: [4 reserved bits] [data bits 63 ~ 60]
///     Reserved bits default to 0, but aren't validated anyway while reading.
///
/// Note: the bits are ordered in big-endian format just to adhere to human
/// readability requirements.
///
/// For performance analysis, values less than 128 would take 1 seek, values
/// less than 4096 would take no more than 2 seeks. The rest would take up to
/// 3 seeks and finish the procedure.
///
/// A write is performed instantly.
pub struct VarUint64;

impl VarUint64 {
    /// Same as `read` but also returns read bytes.
    pub fn read_offset(ptr: &[u8], length: usize) -> Result<(usize, u64)> {
        // read byte #0
        if length == 0 {
            return Err(Error::new(ErrorKind::InvalidData, "out of bounds"));
        }
        let mut result = ptr[0] as u64;
        match result & 0b_10000000 {
            0 => return Ok((1, result)),
            _ => result &= 0b_01111111,
        };

        // read byte #1
        let byte1 = ptr[1] as u64;
        let len = byte1 >> 5;
        result |= (byte1 & 0b_00011111) << 7;

        // read the rest, depending on the case
        // also since a length read often precedes a data read, the overhead
        //     here really isn't a big issue
        if (length as u64) < 2 + len {
            return Err(Error::new(ErrorKind::InvalidData, "out of bounds"));
        }
        match len {
            0 => {}
            1 => {
                let byte2 = (ptr[2] as u64) << 12;
                result |= byte2;
            }
            2 => {
                let byte2 = (ptr[2] as u64) << 12;
                let byte3 = (ptr[3] as u64) << 20;
                result |= byte2 | byte3;
            }
            3 => {
                let byte2 = (ptr[2] as u64) << 12;
                let byte3 = (ptr[3] as u64) << 20;
                let byte4 = (ptr[4] as u64) << 28;
                result |= byte2 | byte3 | byte4;
            }
            4 => {
                let byte2 = (ptr[2] as u64) << 12;
                let byte3 = (ptr[3] as u64) << 20;
                let byte4 = (ptr[4] as u64) << 28;
                let byte5 = (ptr[5] as u64) << 36;
                result |= byte2 | byte3 | byte4 | byte5;
            }
            5 => {
                let byte2 = (ptr[2] as u64) << 12;
                let byte3 = (ptr[3] as u64) << 20;
                let byte4 = (ptr[4] as u64) << 28;
                let byte5 = (ptr[5] as u64) << 36;
                let byte6 = (ptr[6] as u64) << 44;
                result |= byte2 | byte3 | byte4 | byte5 | byte6;
            }
            6 => {
                let byte2 = (ptr[2] as u64) << 12;
                let byte3 = (ptr[3] as u64) << 20;
                let byte4 = (ptr[4] as u64) << 28;
                let byte5 = (ptr[5] as u64) << 36;
                let byte6 = (ptr[6] as u64) << 44;
                let byte7 = (ptr[7] as u64) << 52;
                result |= byte2 | byte3 | byte4 | byte5 | byte6 | byte7;
            }
            7 => {
                let byte2 = (ptr[2] as u64) << 12;
                let byte3 = (ptr[3] as u64) << 20;
                let byte4 = (ptr[4] as u64) << 28;
                let byte5 = (ptr[5] as u64) << 36;
                let byte6 = (ptr[6] as u64) << 44;
                let byte7 = (ptr[7] as u64) << 52;
                let byte8 = (ptr[8] as u64 & 0b_00001111) << 60;
                result |= byte2 | byte3 | byte4 | byte5 | byte6 | byte7 | byte8;
            }
            _ => unreachable!(),
        }

        Ok(((len + 2) as usize, result))
    }

    /// Read a varuint64 value from a given [`ptr`] with up to [`length`] bytes
    /// remaining in buffer. Reading past [`ptr + length`] (inclusive) would
    /// trigger a read error.
    pub fn read(ptr: &[u8], length: usize) -> Result<u64> {
        let (_len, value) = Self::read_offset(ptr, length)?;
        Ok(value)
    }

    /// Same with `read`, but actually adds an offset value for you.
    ///
    /// Also ignores the errors.
    pub fn read_and_seek(ptr: &[u8], offset: &mut usize, length: usize) -> Result<u64> {
        let (len, value) = Self::read_offset(ptr, length)?;
        *offset += len;
        Ok(value)
    }

    /// Converts an unsigned 64-bit integer into a varuint64 char array,
    /// returning the number of written characters.
    pub fn as_slice(value: u64, array: &mut [u8]) -> usize {
        if value < (1 << 7) {
            array[0] = value as u8;
            return 1;
        } else if value < (1 << 12) {
            array[0] = 0b10000000 | (value & 0b01111111) as u8;
            array[1] = 0b00000000 | ((value >> 7) & 0b00011111) as u8;
            return 2;
        } else if value < (1 << 20) {
            array[0] = 0b10000000 | (value & 0b01111111) as u8;
            array[1] = 0b00100000 | ((value >> 7) & 0b00011111) as u8;
            array[2] = (value >> 12) as u8;
            return 3;
        } else if value < (1 << 28) {
            array[0] = 0b10000000 | (value & 0b01111111) as u8;
            array[1] = 0b01000000 | ((value >> 7) & 0b00011111) as u8;
            array[2] = (value >> 12) as u8;
            array[3] = (value >> 20) as u8;
            return 4;
        } else if value < (1 << 36) {
            array[0] = 0b10000000 | (value & 0b01111111) as u8;
            array[1] = 0b01100000 | ((value >> 7) & 0b00011111) as u8;
            array[2] = (value >> 12) as u8;
            array[3] = (value >> 20) as u8;
            array[4] = (value >> 28) as u8;
            return 5;
        } else if value < (1 << 44) {
            array[0] = 0b10000000 | (value & 0b01111111) as u8;
            array[1] = 0b10000000 | ((value >> 7) & 0b00011111) as u8;
            array[2] = (value >> 12) as u8;
            array[3] = (value >> 20) as u8;
            array[4] = (value >> 28) as u8;
            array[5] = (value >> 36) as u8;
            return 6;
        } else if value < (1 << 52) {
            array[0] = 0b10000000 | (value & 0b01111111) as u8;
            array[1] = 0b10100000 | ((value >> 7) & 0b00011111) as u8;
            array[2] = (value >> 12) as u8;
            array[3] = (value >> 20) as u8;
            array[4] = (value >> 28) as u8;
            array[5] = (value >> 36) as u8;
            array[6] = (value >> 44) as u8;
            return 7;
        } else if value < (1 << 60) {
            array[0] = 0b10000000 | (value & 0b01111111) as u8;
            array[1] = 0b11000000 | ((value >> 7) & 0b00011111) as u8;
            array[2] = (value >> 12) as u8;
            array[3] = (value >> 20) as u8;
            array[4] = (value >> 28) as u8;
            array[5] = (value >> 36) as u8;
            array[6] = (value >> 44) as u8;
            array[7] = (value >> 52) as u8;
            return 8;
        } else {
            array[0] = 0b10000000 | (value & 0b01111111) as u8;
            array[1] = 0b11100000 | ((value >> 7) & 0b00011111) as u8;
            array[2] = (value >> 12) as u8;
            array[3] = (value >> 20) as u8;
            array[4] = (value >> 28) as u8;
            array[5] = (value >> 36) as u8;
            array[6] = (value >> 44) as u8;
            array[7] = (value >> 52) as u8;
            array[8] = ((value >> 60) & 0b00001111) as u8;
            return 9;
        }
    }

    /// Writes an unsigned 64-bit integer into a file handle in the format of
    /// varuint64, returning how many bytes were written.
    pub fn write(value: u64, file: &mut File) -> io::Result<usize> {
        let mut buffer: MaybeUninit<[u8; 9]> = MaybeUninit::uninit();
        let array = unsafe { buffer.assume_init_mut() };
        let len = Self::as_slice(value, array);
        file.write(&array[0..len])
    }
}

#[cfg(test)]
pub mod tests {
    use super::VarUint64;

    fn works(value: u64) -> () {
        let mut buffer: [u8; 9] = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        VarUint64::as_slice(value, &mut buffer);

        match VarUint64::read(&buffer, 9) {
            Ok(resolved) => assert_eq!(resolved, value),
            Err(_) => panic!("varuint64 0x{value:x} failed to decode"),
        };
    }

    #[test]
    fn rand_3_bits() -> () {
        for i in 0..64 {
            for j in i..64 {
                for k in j..64 {
                    let value = (1_u64 << i) | (1_u64 << j) | (1_u64 << k);
                    works(value);
                }
            }
        }
    }

    #[test]
    fn small_values() -> () {
        for i in 0..16384 {
            works(i);
        }
    }
}
