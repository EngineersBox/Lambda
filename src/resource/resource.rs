use std::io::{self, Cursor, BufReader};
use byteorder::{ByteOrder, ReadBytesExt};

pub trait Resource {
    type T: ByteOrder;
    fn from_reader(reader: &mut BufReader<impl ReadBytesExt>) -> io::Result<Self> where Self: Sized;
}

pub fn read_char_array(reader: &mut BufReader<impl ReadBytesExt>, char_array: &[u8]) -> usize {
    let mut read_count: usize = 0;
    for i in 0..char_array.len() {
        match reader.read_u8() {
            Ok(0) => break,
            Ok(value) => {
                char_array[i] = value;
                read_count += 1;
            },
            Err(error) => panic!("Unable to read char array: {}", error),
        };
    }
    return read_count;
}
