use std::io::{self, Read, Write, Seek, SeekFrom};
use byteorder::{BE, ByteOrder, ReadBytesExt};

const DATA_ARR: &'static str = "vtxData";
const INFO_VAR: &'static str = "vtxInfo";
const INDENT: &'static str = "    ";
const DATA_SIZE: usize = 3; // short arr[3]

#[derive(Debug, Fail)]
pub enum VtxDumpErr {
    #[fail(display = "problem with io when dumping vertices")]
    Io(#[cause] io::Error),
    #[fail(display = "couldn't convert from RAM <{:#x}> to file offset based on start RAM <{:#x}>", _0, _1)]
    Tlb(u32, u32)
}
impl From<io::Error> for VtxDumpErr {
    fn from(e: io::Error) -> Self { VtxDumpErr::Io(e) }
}

struct VtxInfo {
    count: u32,
    kind: i32,
    data_ptr: u32,
}
impl VtxInfo {
    fn from_bytes(b: &[u8; 12]) -> Self {
        let count    = BE::read_u32(&b[0..4]);
        let kind     = BE::read_i32(&b[4..8]);
        let data_ptr = BE::read_u32(&b[8..12]);

        VtxInfo {count, kind, data_ptr}
    }
}

pub fn dump<R, W>(mut rdr: R, mut wtr: W, offset: u64, vram: u32) -> Result<(), VtxDumpErr> 
    where R: Read + Seek, W: Write
{
    let mut info_buf = [0u8; 12];
    rdr.seek(SeekFrom::Start(offset))?;
    rdr.read_exact(&mut info_buf)?;

    let info = VtxInfo::from_bytes(&info_buf);
    let data_offset = match info.data_ptr.checked_sub(vram) {
        Some(o) => o as u64,
        None => return Err(VtxDumpErr::Tlb(info.data_ptr, vram)),
    };
    // abstract into function?
    let mut data = vec![0i16; info.count as usize * DATA_SIZE];   // count of short[3] 
    rdr.seek(SeekFrom::Start(data_offset))?;
    rdr.read_i16_into::<BE>(&mut data)?;

    // Write out the three 16bit arrays in sets of two chunks
    writeln!(wtr, "/* @ {:08X} ({:x}) */", data_offset + vram as u64, data_offset)?;
    writeln!(wtr, "{}[{}] = {{", DATA_ARR, info.count)?;
    for arr in data.chunks(3) {
        writeln!(wtr, "{}{{ {}, {}, {} }},", INDENT, arr[0], arr[1], arr[2])?;
    }
    write!(wtr,"\n")?;

    // Write the info struct
    writeln!(wtr, "/* @ {:08X} ({:x}) */", offset + vram as u64, offset)?;
    writeln!(wtr, "{} = {{ {}, {:#x}, {} }}", INFO_VAR, info.count, info.kind, DATA_ARR)?;

    Ok(())
}
