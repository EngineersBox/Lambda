use std::io::{self, Cursor};
use byteorder::{ByteOrder, ReadBytesExt};

pub trait Resource {
    type T: ByteOrder;
    fn from_reader(reader: &Cursor<impl ReadBytesExt>) -> io::Result<Self> where Self: Sized;

}
