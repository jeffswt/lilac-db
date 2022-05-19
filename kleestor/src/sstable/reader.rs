use crate::bloom::BloomFilter;
use crate::record::ByteStream;
use crate::utils::varint::VarUint64;
use memmap::{Mmap, MmapOptions};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Error, ErrorKind, Result};

use super::MetaBlockType;

pub struct SSTableReader {
    handle: File,

    region: Mmap,
    bloom: BloomFilter,
    keys: Vec<ByteStream>,
}

impl SSTableReader {
    pub fn new(handle: File) -> Result<Self> {
        // unzip file to a memory map
        let region = unsafe { MmapOptions::new().map(&handle)? };

        // validate magic
        let magic = Self::read_u64(&region[region.len() - 8..]);
        if magic != 0x1145_1419_1981_fee1_u64 {
            return Err(Error::new(ErrorKind::InvalidData, "invalid magic"));
        }

        // extract header block
        let mut offset = Self::read_u64(&region[region.len() - 16..]) as usize;
        let header_block_items = Self::read_varu64(&region, &mut offset)?;

        let mut header_block = BTreeMap::<MetaBlockType, usize>::new();
        for _ in 0..header_block_items {
            let block_type = Self::read_varu64(&region, &mut offset)?;
            let indice = Self::read_varu64(&region, &mut offset)?;
            let block_type = match block_type {
                1 => MetaBlockType::Index,
                2 => MetaBlockType::BloomFilter,
                _ => return Err(Error::new(ErrorKind::InvalidData, "invalid metablock type")),
            };
            header_block.insert(block_type, indice as usize);
        }

        // extract index block
        let offset = *match header_block.get(&MetaBlockType::Index) {
            Some(val) => val,
            None => return Err(Error::new(ErrorKind::InvalidData, "missing index")),
        };
        let keys = Self::get_index(&region, offset)?;

        // extract bloom block
        let offset = *match header_block.get(&MetaBlockType::BloomFilter) {
            Some(val) => val,
            None => return Err(Error::new(ErrorKind::InvalidData, "missing bloom filter")),
        };
        let bloom = Self::get_bloom_filter(&region, offset)?;

        Ok(Self {
            handle,
            region,
            bloom,
            keys,
        })
    }

    fn get_index(region: &Mmap, mut offset: usize) -> Result<Vec<ByteStream>> {
        // read key offset values
        let len = Self::read_varu64(region, &mut offset)?;
        let mut indices = Vec::<usize>::new();
        let mut keys = Vec::<ByteStream>::new();

        for _ in 0..len {
            let indice = Self::read_varu64(region, &mut offset)?;
            indices.push(indice as usize);
        }

        // scan database for keys
        for mut indice in indices {
            let k_len = Self::read_varu64(region, &mut indice)? as usize;
            let common_len = Self::read_varu64(region, &mut indice)?;
            let _v_len = Self::read_varu64(region, &mut indice)?;
            let _flags = Self::read_varu64(region, &mut indice)?;

            // you shouldn't index a compressed key
            if common_len != 0 {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "index pointing to compressed key",
                ));
            }

            // read key and send it away
            let key = &region[indice..indice + k_len];
            let key = ByteStream::from_slice(key);
            keys.push(key);
        }
        Ok(keys)
    }

    fn get_bloom_filter(region: &Mmap, mut offset: usize) -> Result<BloomFilter> {
        // validate filter size
        let size = Self::read_varu64(region, &mut offset)? as usize;
        if size != BloomFilter::default_size() {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "improper bloom filter size",
            ));
        }

        // read a lot of bytes
        let bloom = &region[offset..offset + size];
        let bloom = BloomFilter::from_slice(bloom);
        Ok(bloom)
    }

    /// Accesses u64 value in little-endian.
    fn read_u64(region: &[u8]) -> u64 {
        (region[0] as u64)
            | ((region[1] as u64) << 8)
            | ((region[2] as u64) << 16)
            | ((region[3] as u64) << 24)
            | ((region[4] as u64) << 32)
            | ((region[5] as u64) << 40)
            | ((region[6] as u64) << 48)
            | ((region[7] as u64) << 56)
    }

    /// Accesses variable u64 value.
    fn read_varu64(region: &Mmap, offset: &mut usize) -> Result<u64> {
        let offset_val = *offset;
        VarUint64::read_and_seek(&region[offset_val..], offset, region.len() - offset_val)
    }
}
