use std::io::{self, Cursor, BufReader};
use byteorder::{ByteOrder, ReadBytesExt};

pub trait Resource {
    type T: ByteOrder;
    fn from_reader(reader: &mut BufReader<impl ReadBytesExt>) -> io::Result<Self> where Self: Sized;

}
