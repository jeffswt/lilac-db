use crate::bloom::BloomFilter;
use crate::record::{ByteStream, KvData, KvDataRef, KvEntry, KvPointer};
use crate::utils;
use crate::utils::futures::MutexSync;
use crate::utils::varint::VarUint64;
use lru::LruCache;
use memmap::{Mmap, MmapOptions};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Error, ErrorKind, Result as IoResult};
use std::iter::Peekable;
use std::rc::Rc;

use super::MetaBlockType;

pub struct SSTableReader {
    /// Reference to file, just to keep it open.
    _handle: File,

    /// Mapped memory region.
    region: Mmap,

    /// Internal bloom filter to check for missing keys.
    bloom: BloomFilter,

    /// Indexed breaking points that are saved every ?? keys.
    keys: Vec<(ByteStream, usize)>,

    /// LRU cache locks.
    ///
    /// TODO: this might cause issues on an async workload.
    cache_lock: MutexSync<()>,

    /// LRU cache for recently *accessed* items.
    cache_read: LruCache<ByteStream, KvData>,

    /// LRU cache as a look-aside buffer. This can be really useful when
    /// continuously accessing discrete items sequentially (and for this
    /// purpose is way smaller than the read buffer).
    cache_lookaside: LruCache<ByteStream, KvData>,

    /// Sequential read indicator, when consistent seqread are performed, we
    /// will rely on [`cache_lookaside`] more.
    cache_seq_ind: f64,
}

impl SSTableReader {
    pub fn new(handle: File) -> IoResult<Self> {
        // unzip file to a memory map
        let region = unsafe { MmapOptions::new().map(&handle)? };

        // validate magic
        let magic = Self::read_u64(&region[region.len() - 8..]);
        if magic != 0x1145_1419_1981_fee1_u64 {
            return Err(Error::new(ErrorKind::InvalidData, "invalid magic"));
        }

        // extract header block
        let mut offset = Self::read_u64(&region[region.len() - 16..]) as usize;
        let header_block_items = Self::read_varu64(&region, &mut offset);

        let mut header_block = BTreeMap::<MetaBlockType, usize>::new();
        for _ in 0..header_block_items {
            let block_type = Self::read_varu64(&region, &mut offset);
            let indice = Self::read_varu64(&region, &mut offset);
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
            _handle: handle,
            region,
            bloom,
            keys,
            cache_lock: MutexSync::new(()),
            cache_read: LruCache::new(2048),
            cache_lookaside: LruCache::new(256),
            cache_seq_ind: 0.0,
        })
    }

    fn get_index(region: &Mmap, mut offset: usize) -> IoResult<Vec<(ByteStream, usize)>> {
        // read key offset values
        let len = Self::read_varu64(region, &mut offset);
        let mut indices = Vec::<usize>::new();
        let mut keys = Vec::<(ByteStream, usize)>::new();

        for _ in 0..len {
            let indice = Self::read_varu64(region, &mut offset);
            indices.push(indice as usize);
        }

        // scan database for keys
        for indice in indices {
            let mut ptr = indice;

            let k_len = Self::read_varu64(region, &mut ptr) as usize;
            let common_len = Self::read_varu64(region, &mut ptr);
            let _v_len = Self::read_varu64(region, &mut ptr);
            let _flags = Self::read_varu64(region, &mut ptr);

            // you shouldn't index a compressed key
            if common_len != 0 {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "index pointing to compressed key",
                ));
            }

            // read key and send it away
            let key = &region[ptr..ptr + k_len];
            let key = ByteStream::from_slice(key);
            keys.push((key, indice));
        }
        Ok(keys)
    }

    fn get_bloom_filter(region: &Mmap, mut offset: usize) -> IoResult<BloomFilter> {
        // validate filter size
        let size = Self::read_varu64(region, &mut offset) as usize;
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
    fn read_varu64(region: &Mmap, offset: &mut usize) -> u64 {
        let (len, result) = VarUint64::read_offset_unchecked(&region[*offset..]);
        *offset += len;
        return result;
    }

    /// Access item from table.
    pub fn get(&mut self, key: &[u8]) -> Option<KvData> {
        // check for lru cache(s)
        let key_bs = ByteStream::from(key);
        '_chk_cache: {
            let _lock = self.cache_lock.lock();
            match self.cache_read.get(&key_bs) {
                Some(entry) => return Some(entry.clone()),
                None => (),
            };
            match self.cache_lookaside.get(&key_bs) {
                Some(entry) => return Some(entry.clone()),
                None => (),
            };
        }

        // just to circumvent the blasted lifetime checker
        unsafe {
            let ptr = self as *mut Self;

            // locate item and clone
            let (index, mut iter) = match (*ptr).get_iter_internal(key) {
                None => return None,
                Some(it) => it,
            };
            let item = match iter.next() {
                None => return None,
                Some(it) => it,
            };
            let result = KvData::from(&item.value());

            // update lru cache, flushing entire region into lru
            let mut max_items = 8;
            let term_offset = if index + 1 < (*ptr).keys.len() {
                (*ptr).keys[index + 1].1
            } else {
                (*ptr).keys[index].1
            };
            '_upd_cache: {
                let _lock = (*ptr).cache_lock.lock();
                (*ptr).cache_read.put(key_bs, result.clone());
                for item in iter {
                    (*ptr)
                        .cache_lookaside
                        .put(ByteStream::from(item.key()), KvData::from(&item.value()));
                    if item._offset >= term_offset {
                        break;
                    }
                    // ensure we don't look-ahead too much
                    max_items -= 1;
                    if max_items <= 0 {
                        break;
                    }
                }
            }
            Some(result)
        }
    }

    /// Access item from table, returning a partial-scan iterator from that
    /// location.
    pub fn get_iter(&mut self, key: &[u8]) -> Option<Peekable<SSTableReaderIterator>> {
        match self.get_iter_internal(key) {
            None => None,
            Some((_index, iter)) => Some(iter),
        }
    }

    /// Internal implementation of [`get_iter`].
    fn get_iter_internal(
        &mut self,
        key: &[u8],
    ) -> Option<(usize, Peekable<SSTableReaderIterator>)> {
        // check if key might be non-existent in bloom filter
        if !self.bloom.query(key) {
            return None;
        }

        // resolve index of lower-bound key and go through it
        let index = match self.get_iter_lower_bound(key) {
            None => return None,
            Some(idx) => idx,
        };
        let mut iter = self.iter_from_offset(self.keys[index].1).peekable();
        loop {
            match ByteStream::ref_2_partial_cmp(iter.peek().unwrap().key(), key) {
                Some(Ordering::Greater) => return None,
                Some(Ordering::Equal) => return Some((index, iter)),
                Some(Ordering::Less) => _ = iter.next(),
                _ => panic!("nothing compared"),
            }
        }
    }

    /// Get lower-bound key reference index.
    fn get_iter_lower_bound(&self, key: &[u8]) -> Option<usize> {
        // binary search for some
        let len = self.keys.len();
        if len == 0 {
            return None;
        }
        let mut left = 0_usize;
        let mut right = len;

        while left < right {
            let mid = left + ((right - left + 1) >> 1);
            if mid >= len {
                return None; // key > max(all_keys)
            }
            match self.keys[mid].0.ref_partial_cmp(key) {
                Some(Ordering::Less) => left = mid,
                Some(Ordering::Greater) => right = mid - 1,
                Some(Ordering::Equal) => return Some(mid),
                _ => panic!("expect ordering to return comparison"),
            }
        }

        match left {
            0 => match self.keys[0].0.ref_partial_cmp(key) {
                Some(Ordering::Greater) => None,
                _ => Some(0),
            },
            rest => Some(rest),
        }
    }

    /// Create full-scan iterator.
    pub fn iter(&self) -> SSTableReaderIterator {
        // the sstable file should contain some keys
        // otherwise we should run into a (0, 0, 0, 0) termination file
        let offset = match self.keys.len() {
            0 => 0_usize,
            _ => self.keys[0].1,
        };
        self.iter_from_offset(offset)
    }

    /// Create iterator from given offset.
    fn iter_from_offset(&self, offset: usize) -> SSTableReaderIterator {
        SSTableReaderIterator {
            region: &self.region,
            offset,
            last_key: Rc::from(vec![]),
        }
    }
}

/// SSTable reader iterator manager.
pub struct SSTableReaderIterator<'a> {
    /// Reference to file as a memory region.
    region: &'a Mmap,

    /// Current iterator offset.
    offset: usize,

    /// Holds previous key (index compression).
    last_key: Rc<Vec<u8>>,
}

impl<'a> Iterator for SSTableReaderIterator<'a> {
    type Item = SSTableReaderPointer<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let the_offset = self.offset;

        // read headers of k-v pair
        let key_len = self.read_varu64() as usize;
        let key_common_len = self.read_varu64() as usize;
        let value_len = self.read_varu64() as usize;
        let flags = self.read_varu64() as u8;

        // terminated
        if key_len == 0 && key_common_len == 0 && value_len == 0 {
            return None;
        }

        // deflate new key
        let mut key = Vec::<u8>::with_capacity(key_len);
        let key_stored_len = key_len - key_common_len;
        key.resize(key_len, 0_u8);
        if key_common_len > 0 {
            // assuring last_key won't be accessed on first iteration
            key[0..key_common_len].copy_from_slice(&self.last_key[0..key_common_len]);
        }
        key[key_common_len..key_len]
            .copy_from_slice(&self.region[self.offset..self.offset + key_stored_len]);
        self.offset += key_stored_len;
        let key = Rc::new(key);
        self.last_key = key.clone();

        // get reference to value
        let value = &self.region[self.offset..self.offset + value_len];
        self.offset += value_len;

        // construct pointer
        Some(Self::Item {
            _key: key,
            _value: value,
            _flags: flags,
            _offset: the_offset,
        })
    }
}

impl<'a> SSTableReaderIterator<'a> {
    /// Access VarUint64 from memory region.
    fn read_varu64(&mut self) -> u64 {
        let offset = self.offset;
        VarUint64::read_and_seek(
            &self.region[offset..],
            &mut self.offset,
            self.region.len() - offset,
        )
        .unwrap()
    }
}

/// Reader iterator (pointer) interface.
pub struct SSTableReaderPointer<'a> {
    /// Reference to key.
    _key: Rc<Vec<u8>>,

    /// Reference to value.
    _value: &'a [u8],

    /// Item flags.
    _flags: u8,

    /// Offset from file begin.
    _offset: usize,
}

impl<'a> KvPointer for SSTableReaderPointer<'a> {
    /// Get key where iterator points to.
    fn key(&self) -> &[u8] {
        &self._key
    }

    fn value(&self) -> KvDataRef {
        match self._flags {
            0b00000001_u8 => KvDataRef::Tombstone { cached: true },
            0b00000000_u8 => KvDataRef::Value {
                cached: true,
                value: unsafe { utils::reborrow_slice(self._value) },
            },
            rest => panic!("unrecognized flag {rest}"),
        }
    }

    fn value_mut(&self) -> &mut KvEntry {
        unimplemented!("sstable cannot be opened as read-write");
    }
}
