//! embedded-sdmmc-rs - FAT file system
//!
//! Implements the File Allocation Table file system

use super::{Block, BlockDevice, BlockIdx, Controller, Error};
use byteorder::{ByteOrder, LittleEndian};

pub const PARTITION_ID_FAT32_LBA: u8 = 0x0C;

/// Identifies a FAT32 Volume on the disk.
#[derive(PartialEq, Eq)]
pub struct Volume {
    pub(crate) lba_start: BlockIdx,
    pub(crate) num_blocks: BlockIdx,
    pub(crate) name: [u8; 11],
}

impl core::fmt::Debug for Volume {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(fmt, "Volume(name={:?}, ", core::str::from_utf8(&self.name))?;
        write!(fmt, "lba_start=0x{:08x}, ", self.lba_start.0)?;
        write!(fmt, "num_blocks=0x{:08x})", self.num_blocks.0)?;
        Ok(())
    }
}

pub fn parse_volume<D>(
    controller: &mut Controller<D>,
    lba_start: BlockIdx,
    num_blocks: BlockIdx,
) -> Result<Volume, Error<D::Error>>
where
    D: BlockDevice,
    D::Error: core::fmt::Debug,
{
    const FOOTER_START: usize = 510;
    const FOOTER_VALUE: u16 = 0xAA55;

    let mut blocks = [Block::new()];
    controller
        .block_device
        .read(&mut blocks, lba_start)
        .map_err(|e| Error::DeviceError(e))?;
    let block = &blocks[0];
    if LittleEndian::read_u16(&block[FOOTER_START..FOOTER_START + 2]) != FOOTER_VALUE {
        return Err(Error::FormatError("Invalid partition signature."));
    }
    let mut volume = Volume {
        lba_start,
        num_blocks,
        name: [0u8; 11],
    };
    volume.name[..].copy_from_slice(&block[71..82]);
    Ok(volume)
}
