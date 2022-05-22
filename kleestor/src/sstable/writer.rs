use crate::bloom::BloomFilter;
use crate::record::{ByteStream, KvDataRef, KvPointer};
use crate::utils::varint::VarUint64;
use std::fs::File;
use std::io::{Result, Seek, SeekFrom, Write};
use std::marker::PhantomData;

use super::MetaBlockType;

pub struct SSTableWriter<Pointer, Iter>
where
    Pointer: KvPointer + Sized,
    Iter: Iterator<Item = Pointer>,
{
    /// File handle to write to.
    handle: File,
    /// Actual pointer in file.
    handle_pointer: usize,

    /// Write buffer, which holds twice as much as the [`flush_interval`].
    buffer: Vec<u8>,
    /// Buffer pointer.
    buffer_pointer: usize,
    /// Buffer should flush writes to handle after this many bytes.
    flush_interval: usize,

    _marker: PhantomData<Iter>,
}

impl<Pointer, Iter> SSTableWriter<Pointer, Iter>
where
    Pointer: KvPointer + Sized,
    Iter: Iterator<Item = Pointer>,
{
    pub fn new(handle: File) -> Self {
        let flush_interval = 4194304_usize;
        let mut buffer = Vec::<u8>::new();
        buffer.resize(flush_interval * 2, 0_u8);

        Self {
            handle,
            handle_pointer: 0_usize,
            buffer,
            buffer_pointer: 0_usize,
            flush_interval,
            _marker: PhantomData,
        }
    }

    #[allow(unused_assignments)]
    pub fn write(&mut self, iter: Iter) -> Result<()> {
        // reset pointer
        self.handle.seek(SeekFrom::Start(0))?;
        let mut block_indices = Vec::<(MetaBlockType, usize)>::new();

        // assure index distance
        let mut prev_block = 0;
        let min_block_size = 50; // save index for every 50 keys
        let mut indices = Vec::<usize>::new();

        // prepare bloom filter
        let mut bloom = BloomFilter::new();

        // index prefix compression
        let the_null_key = ByteStream::from_vec(vec![]);
        let mut last_key = the_null_key.as_ref();

        // start writing keys
        let mut _last_item: Pointer; // hold lifetime
        let mut iter = iter.peekable();
        while let Some(item) = iter.next() {
            // fetch values
            let k: &[u8] = unsafe { std::mem::transmute(item.key()) };
            let v: KvDataRef = unsafe { std::mem::transmute(item.value()) };

            // skip cached values
            match &v {
                KvDataRef::Tombstone { cached: true, .. } => continue,
                KvDataRef::Value { cached: true, .. } => continue,
                _ => (),
            };

            // maintain bloom filter
            let is_last_value = if let None = iter.peek() { true } else { false };
            bloom.insert(&k);

            // compress index prefixes
            let offset = self.tell();
            let mut common_len = 0_usize;

            if offset == 0 || prev_block >= min_block_size || is_last_value {
                // when offset is 0 (start block) or block is big enough
                // or is the last block
                // don't perform compression and save index position
                indices.push(offset);
                prev_block = 0;
            } else {
                // perform compression
                let l_ref = last_key;
                let r_ref = k;
                let min_len = std::cmp::min(l_ref.len(), r_ref.len());
                while common_len < min_len && l_ref[common_len] == r_ref[common_len] {
                    common_len += 1;
                }
                prev_block += 1;
            }
            last_key = k;

            // write key
            self.write_kv_pair(k, common_len, &v)?;

            // keep last pointer alive
            _last_item = item;
        }
        // largest key has been saved

        // write empty key marking end of data section
        self.write_varu64(0_u64);
        self.write_varu64(0_u64);
        self.write_varu64(0_u64);
        self.write_varu64(0_u8 as u64);

        // write index block
        // starts with 1 counter and [counter] indices, all in varuint64
        let offset = self.tell();
        block_indices.push((MetaBlockType::Index, offset));

        self.write_varu64(indices.len() as u64);
        for indice in indices {
            self.write_varu64(indice as u64);
            self.flush_buffer_lazy()?;
        }
        self.flush_buffer()?;

        // write bloom filter block
        // starts with 1 counter and [counter] subsequent bytes as the filter
        let offset = self.tell();
        block_indices.push((MetaBlockType::BloomFilter, offset));

        self.write_varu64(bloom.size() as u64);
        self.flush_buffer()?;
        self.handle_pointer += bloom.write(&mut self.handle)?;

        // write header block
        // contains a entry counter for all metablock offsets
        // contains [block type: varuint64, varuint64] for each metablock
        let offset = self.tell();
        self.write_varu64(block_indices.len() as u64);
        for (block_type, indice) in block_indices {
            self.write_varu64(block_type as u8 as u64);
            self.write_varu64(indice as u64);
        }

        // a header block offset in the end
        self.write_u64(offset as u64);

        // a magic in the end
        self.write_u64(0x1145_1419_1981_fee1_u64);

        // all done
        self.flush_buffer()?;
        Ok(())
    }

    // writes key-value pair
    fn write_kv_pair(
        &mut self,
        k: &[u8],
        k_common_len: usize,
        v: &KvDataRef,
    ) -> Result<()> {
        match &v {
            KvDataRef::Tombstone { .. } => {
                // write lengths
                self.write_varu64(k.len() as u64);
                self.write_varu64(k_common_len as u64);
                self.write_varu64(0_u64);

                // write flag
                self.write_varu64(0b00000001_u8 as u64);

                // write (compressed) key
                self.write_slice(&k[k_common_len..])?;
            }
            KvDataRef::Value { value, .. } => {
                // write lengths
                self.write_varu64(k.len() as u64);
                self.write_varu64(k_common_len as u64);
                self.write_varu64(value.len() as u64);

                // write flag
                self.write_varu64(0b00000000_u8 as u64);

                // write (compressed) key and value
                self.write_slice(&k[k_common_len..])?;
                self.write_slice(value)?;
            }
        };

        Ok(())
    }

    /// Writes VarUint64 value. We're assuming that this will not overflow the
    /// buffer.
    fn write_varu64(&mut self, value: u64) -> () {
        self.buffer_pointer += VarUint64::as_slice(value, &mut self.buffer[self.buffer_pointer..]);
    }

    /// Writes uint64 value in exactly 8 bytes.
    fn write_u64(&mut self, value: u64) -> () {
        let ptr = &mut self.buffer[self.buffer_pointer..];

        ptr[0] = (value & 0xff) as u8;
        ptr[1] = ((value >> 8) & 0xff) as u8;
        ptr[2] = ((value >> 16) & 0xff) as u8;
        ptr[3] = ((value >> 24) & 0xff) as u8;
        ptr[4] = ((value >> 32) & 0xff) as u8;
        ptr[5] = ((value >> 40) & 0xff) as u8;
        ptr[6] = ((value >> 48) & 0xff) as u8;
        ptr[7] = (value >> 56) as u8;

        self.buffer_pointer += 8;
    }

    /// Writes a large string slice. Will try and fit it into buffer. If buffer
    /// is insufficiently large to hold it, forced flush will be executed.
    fn write_slice(&mut self, slice: &[u8]) -> Result<()> {
        let len = slice.len();

        if self.buffer_pointer + len >= self.flush_interval * 2 - 1 {
            self.flush_buffer()?;
            self.handle.write(slice)?;
            self.handle_pointer += len;
        } else if self.buffer_pointer + len >= self.flush_interval {
            self.buffer[self.buffer_pointer..self.buffer_pointer + len].copy_from_slice(slice);
            self.buffer_pointer += len;
            self.flush_buffer()?;
        } else {
            self.buffer[self.buffer_pointer..self.buffer_pointer + len].copy_from_slice(slice);
            self.buffer_pointer += len;
        }
        Ok(())
    }

    /// Informs current position.
    fn tell(&mut self) -> usize {
        self.handle_pointer + self.buffer_pointer
    }

    /// Forcefully flush buffer.
    fn flush_buffer(&mut self) -> Result<()> {
        if self.buffer_pointer > 0 {
            self.handle.write(&self.buffer[0..self.buffer_pointer])?;
            self.handle_pointer += self.buffer_pointer;
            self.buffer_pointer = 0;
        }
        Ok(())
    }

    /// Flush buffer only if it'd exceeded size limit.
    fn flush_buffer_lazy(&mut self) -> Result<()> {
        if self.buffer_pointer > self.flush_interval {
            self.flush_buffer()?;
        }
        Ok(())
    }
}
