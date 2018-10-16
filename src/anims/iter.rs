use std::io::{self, Read, Seek, SeekFrom};
use byteorder::{BE, ByteOrder};
use super::types::AnimKind;

#[derive(Debug, Copy, Clone)]
pub struct AnimInfo {
    pub count: i32,
    pub kind: AnimKind,
    pub data_ptr: u32,
}
impl AnimInfo {
    pub const TERMINATOR: Self = AnimInfo { 
        count: -1, 
        kind: AnimKind::Empty, 
        data_ptr: 0
    };
    fn from_bytes(b: &[u8; 12]) -> Self {
        let count    = BE::read_i32(&b[0..4]);
        let kind     = AnimKind::from_i32(BE::read_i32(&b[4..8]));
        let data_ptr = BE::read_u32(&b[8..12]);

        AnimInfo {count, kind, data_ptr}
    }
}

pub struct AnimInfoIter<'r, R: Read + Seek + 'r> {
    buf: [u8; 12],
    rdr: &'r mut R,
    is_done: bool,
}
impl<'r, R: Read + Seek + 'r> AnimInfoIter<'r, R>  {
    pub fn from_rdr(rdr: &'r mut R, offset: u64) -> Result<Self, io::Error> {
        rdr.seek(SeekFrom::Start(offset))?;
        Ok(AnimInfoIter { rdr, buf: [0u8; 12], is_done: false })
    }
}
impl<'r, R: Read + Seek + 'r> Iterator for AnimInfoIter<'r, R> {
    type Item = Result<AnimInfo, io::Error>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.is_done { return None; }
        if let Err(e) = self.rdr.read_exact(&mut self.buf) {
            return Some(Err(e.into()));
        }
        let info = AnimInfo::from_bytes(&self.buf);
        if info.count < 0 { self.is_done = true; return None; }
        Some(Ok(info))
    }
}