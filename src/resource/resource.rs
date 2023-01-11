use std::io::{self, BufReader};
use byteorder::{ByteOrder, ReadBytesExt};

pub trait Resource {
    type T: ByteOrder;
    fn from_reader(reader: &mut BufReader<impl ReadBytesExt>) -> io::Result<Self> where Self: Sized;
}

pub fn read_char_array(arr: &mut [u8], reader: &mut BufReader<impl ReadBytesExt>) -> io::Result<()> {
    let mut null_byte_encountered: bool = false;
    for i in 0..arr.len() {
        if null_byte_encountered {
            reader.read_u8()?;
            continue;
        }
        arr[i] = reader.read_u8()?;
        if arr[i] == 0 {
            null_byte_encountered = true;
        }
    }
    return Ok(());
}
