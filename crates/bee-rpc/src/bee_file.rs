//! The `.bee` file format (`client.write_to_file`, `reader.read_bee_file`).
//!
//! A sequence of records. Each record is a 4-byte big-endian length, then that
//! many bytes of one serialized Buffer. The file ends after the last record.

use std::io::{Read, Write};

use crate::buffer::Buffer;
use crate::Error;

/// Write `buffers` as .bee records.
pub fn write_bee<W: Write>(
    writer: &mut W,
    buffers: impl IntoIterator<Item = Buffer>,
) -> Result<(), Error> {
    let _ = (writer, buffers.into_iter());
    todo!("step 3: bee file")
}

/// Read .bee records. A cut length or a cut record is [`Error::Decode`]. A record
/// that is not a Buffer is [`Error::Decode`].
pub fn read_bee<R: Read>(reader: R) -> BeeReader<R> {
    BeeReader { reader }
}

/// Iterator over the Buffers of a .bee file.
pub struct BeeReader<R> {
    reader: R,
}

impl<R: Read> Iterator for BeeReader<R> {
    type Item = Result<Buffer, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let _ = &mut self.reader;
        todo!("step 3: bee file")
    }
}
