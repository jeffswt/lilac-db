use crate::bloom::BloomFilter;
use crate::record::{ByteStream, KvPointer};
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
    handle: File,

    _marker: PhantomData<Iter>,
}

impl<Pointer, Iter> SSTableWriter<Pointer, Iter>
where
    Pointer: KvPointer + Sized,
    Iter: Iterator<Item = Pointer>,
{
    pub fn new(handle: File) -> Self {
        Self {
            handle,
            _marker: PhantomData,
        }
    }

    #[allow(unused_assignments)]
    pub fn write(&mut self, iter: Iter) -> Result<()> {
        // reset pointer
        self.handle.seek(SeekFrom::Start(0))?;
        let mut block_indices = Vec::<(MetaBlockType, usize)>::new();

        // assure index distance
        let mut offset = 0_usize;
        let mut prev_block = 0;
        let min_block_size = 50; // save index for every 50 keys
        let mut indices = Vec::<usize>::new();

        // prepare bloom filter
        let mut bloom = BloomFilter::new();

        // index prefix compression
        let the_null_key = ByteStream::from_vec(vec![]);
        let mut last_key = the_null_key.as_ref();

        // start writing keys
        let mut _last_item: Pointer;
        let mut iter = iter.peekable();
        while let Some(item) = iter.next() {
            // fetch values
            let k: &[u8] = unsafe { std::mem::transmute(item.key()) };
            let v: &[u8] = unsafe { std::mem::transmute(item.value()) };
            let is_last_value = if let None = iter.peek() { true } else { false };

            // maintain bloom filter
            bloom.insert(&k);

            // compress index prefixes
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

            // write lengths
            offset += VarUint64::write(k.len() as u64, &mut self.handle)?;
            offset += VarUint64::write(common_len as u64, &mut self.handle)?;
            offset += VarUint64::write(v.len() as u64, &mut self.handle)?;

            // write (compressed) key and value
            offset += self.handle.write(&k.as_ref()[common_len..])?;
            offset += self.handle.write(v.as_ref())?;

            // keep last pointer alive
            _last_item = item;
        }
        // largest key has been saved

        // write empty key marking end of data section
        offset += VarUint64::write(0_u64, &mut self.handle)?;
        offset += VarUint64::write(0_u64, &mut self.handle)?;
        offset += VarUint64::write(0_u64, &mut self.handle)?;

        // write index block
        // starts with 1 counter and [counter] indices, all in varuint64
        offset = self.handle.seek(SeekFrom::Current(0))? as usize;
        block_indices.push((MetaBlockType::Index, offset));

        VarUint64::write(indices.len() as u64, &mut self.handle)?;
        for indice in indices {
            VarUint64::write(indice as u64, &mut self.handle)?;
        }

        // write bloom filter block
        // starts with 1 counter and [counter] subsequent bytes as the filter
        offset = self.handle.seek(SeekFrom::Current(0))? as usize;
        block_indices.push((MetaBlockType::BloomFilter, offset));

        VarUint64::write(bloom.size() as u64, &mut self.handle)?;
        bloom.write(&mut self.handle)?;

        // write header block
        // contains a entry counter for all metablock offsets
        // contains [block type: varuint64, varuint64] for each metablock
        offset = self.handle.seek(SeekFrom::Current(0))? as usize;
        VarUint64::write(block_indices.len() as u64, &mut self.handle)?;
        for (block_type, indice) in block_indices {
            VarUint64::write(block_type as u8 as u64, &mut self.handle)?;
            VarUint64::write(indice as u64, &mut self.handle)?;
        }

        // a header block offset in the end
        self.write_u64(offset as u64)?;

        // a magic in the end
        self.write_u64(0x1145_1419_1981_fee1_u64)?;

        // all done
        Ok(())
    }

    // writes uint64 value
    fn write_u64(&mut self, value: u64) -> Result<usize> {
        self.handle.write(&[
            (value & 0xff) as u8,
            ((value >> 8) & 0xff) as u8,
            ((value >> 16) & 0xff) as u8,
            ((value >> 24) & 0xff) as u8,
            ((value >> 32) & 0xff) as u8,
            ((value >> 40) & 0xff) as u8,
            ((value >> 48) & 0xff) as u8,
            (value >> 56) as u8,
        ])
    }
}
