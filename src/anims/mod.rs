use std::io::{self, Read, Write, Seek, SeekFrom};
use byteorder::{BE, ReadBytesExt};
use std::fmt::{Write as FmtWrite};

mod iter;
mod types;
use self::iter::{AnimInfoIter, AnimInfo};
use self::types::{Mtx, AnimKind};

const DATA_ARR: &'static str = "animdata";
const INFO_VAR: &'static str = "anim";
const INDENT: &'static str = "    ";
const SEPARATOR: &'static str = " ";

#[derive(Debug, Fail)]
pub enum AnimDumpErr {
    #[fail(display = "problem with io when dumping vertices")]
    Io(#[cause] io::Error),
    #[fail(display = "couldn't convert from RAM <{:#x}> to file offset based on start RAM <{:#x}>", _0, _1)]
    Tlb(u32, u64)
}
impl From<io::Error> for AnimDumpErr {
    fn from(e: io::Error) -> Self { AnimDumpErr::Io(e) }
}

pub fn dump<R, W>(mut rdr: R, mut wtr: W, offset: u64, vram: u32, width: usize) 
-> Result<(), AnimDumpErr> 
    where R: Read + Seek, W: Write
{
    use self::AnimKind::*;
    let vram = vram as u64;
    let info_addr = offset + vram;
    let anim_infos = AnimInfoIter::from_rdr(&mut rdr, offset)?
        .collect::<Result<Vec<_>,_>>()?;
    let mut info_str = String::new();

    /* Write the data themselves */
    for info in anim_infos.iter() {
        match info.kind {
            Empty | Stub => (),
            Matrix => write_mtx(&mut wtr, &mut rdr, &info, vram, width)?,
            Triangle_2 | Triangle_4 => unimplemented!("Vec out"),
            Short3_6 | Short3_7 => write_short_three(&mut wtr, &mut rdr, &info, vram, width)?,
            Short6_8 | Short6_11 => write_short_six(&mut wtr, &mut rdr, &info, vram, width)?,
            Short9 => unimplemented!("Short[9] out"),
            MatVec => unimplemented!("Matrix+Vec out"),
        };
        /* Write into the info array for the data */
        if info.kind == Empty {
            writeln!(&mut info_str, "{}{{ {}, {}, NULL }},",
                INDENT, info.count, info.kind
            );
        } else {
            writeln!(&mut info_str, "{}{{ {}, {}, {}_{:08X} }},",
                INDENT, info.count, info.kind, DATA_ARR, info.data_ptr
            );
        }
    }
    writeln!(wtr, "/* @ {:08X} */", info_addr)?;
    writeln!(wtr, "{}_{:08X}[{}] = {{", INFO_VAR, info_addr, anim_infos.len())?;
    write!(wtr, "{}", info_str)?;
    /* Write terminator */
    let term = AnimInfo::TERMINATOR;
    writeln!(wtr, "{}{{ {}, {}, NULL }},", INDENT, term.count, term.kind as i32)?;
    writeln!(wtr, "}};")?;
    Ok(())
}

fn get_ind_end(i: usize, size: usize, last: usize) -> (&'static str, &'static str) {
    let lnpos = i % size;
    let indent = if lnpos == 0 {INDENT} else {""};
    let ending = if lnpos == size - 1 || i == last {"\n"} else {SEPARATOR};
    (indent, ending)
}

#[inline]
fn write_short_three<R, W>(wtr: &mut W, rdr: &mut R, info: &AnimInfo, ram: u64, width: usize) 
    -> Result<(), AnimDumpErr>
    where R: Read + Seek, W: Write
{
    const LINE_SZ: usize = 4;
    const DATA_SZ: usize = 3;

    let data_offset = (info.data_ptr as u64).checked_sub(ram).ok_or(AnimDumpErr::Tlb(info.data_ptr, ram))?;
    let mut data = vec![0i16; info.count as usize * DATA_SZ];
    rdr.seek(SeekFrom::Start(data_offset))?;
    rdr.read_i16_into::<BE>(&mut data)?;
    writeln!(wtr, "/* @ {:08X} */", info.data_ptr)?;
    writeln!(wtr, "{}_{:08X}[{}][{}] = {{", DATA_ARR, info.data_ptr, info.count, DATA_SZ)?;
    for (i, arr) in data.chunks(DATA_SZ).enumerate() {
        let (indent, ending) = get_ind_end(i, LINE_SZ, info.count as usize - 1);
        write!(wtr, "{}{{ {:w$}, {:w$}, {:w$} }},{}", 
            indent, arr[0], arr[1], arr[2], ending, 
            w = width
        )?;
    }
    writeln!(wtr,"}};\n")?;
    Ok(())
}

#[inline]
fn write_short_six<R, W>(wtr: &mut W, rdr: &mut R, info: &AnimInfo, ram: u64, width: usize) 
    -> Result<(), AnimDumpErr>
    where R: Read + Seek, W: Write
{
    const LINE_SZ: usize = 2;
    const DATA_SZ: usize = 6;

    let data_offset = (info.data_ptr as u64).checked_sub(ram).ok_or(AnimDumpErr::Tlb(info.data_ptr, ram))?;
    let mut data = vec![0i16; info.count as usize * DATA_SZ];
    rdr.seek(SeekFrom::Start(data_offset))?;
    rdr.read_i16_into::<BE>(&mut data)?;
    writeln!(wtr, "/* @ {:08X} */", info.data_ptr)?;
    writeln!(wtr, "{}_{:08X}[{}][{}] = {{", DATA_ARR, info.data_ptr, info.count, DATA_SZ)?;
    for (i, arr) in data.chunks(DATA_SZ).enumerate() {
        let (indent, ending) = get_ind_end(i, LINE_SZ, info.count as usize - 1);
        write!(wtr, "{}{{ {:w$}, {:w$}, {:w$}, {:w$}, {:w$}, {:w$} }},{}", 
            indent, arr[0], arr[1], arr[2], arr[3], arr[4], arr[5], ending, 
            w = width
        )?;
    }
    writeln!(wtr,"}};\n")?;
    Ok(())
}

#[inline]
fn write_mtx<R, W>(wtr: &mut W, rdr: &mut R, info: &AnimInfo, ram: u64, width: usize) 
    -> Result<(), AnimDumpErr>
    where R: Read + Seek, W: Write
{
    const LINE_SZ: usize = 1;
    const DATA_SZ: usize = Mtx::SIZE;
    
    let data_offset = (info.data_ptr as u64).checked_sub(ram).ok_or(AnimDumpErr::Tlb(info.data_ptr, ram))?;
    let mut data = vec![0.0f32; info.count as usize * DATA_SZ];
    rdr.seek(SeekFrom::Start(data_offset))?;
    rdr.read_f32_into::<BE>(&mut data)?;

    writeln!(wtr, "/* @ {:08X} */", info.data_ptr)?;
    writeln!(wtr, "{}_{:08X}[{}] = {{", DATA_ARR, info.data_ptr, info.count)?;
    for (i, mtx) in data.chunks(DATA_SZ).enumerate() {
        let mtx: Mtx = mtx.into();
        let (indent, ending) = get_ind_end(i, LINE_SZ, info.count as usize - 1);
        writeln!(wtr, "{}{:w$},{}", indent, mtx, ending, w = width)?;
    }
    writeln!(wtr,"}};\n")?;
    Ok(())
}


