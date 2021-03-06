use std::io::{self, Write};

use byteorder::WriteBytesExt;

#[derive(Debug)]
pub struct BitWriter<W> {
    inner: W,
    buf: u8,
    i: usize,
}

impl<W> BitWriter<W>
where
    W: Write,
{
    pub fn new(inner: W) -> Self {
        Self {
            inner,
            buf: 0,
            i: 0,
        }
    }

    pub fn get_ref(&self) -> &W {
        &self.inner
    }

    pub fn try_finish(&mut self) -> io::Result<()> {
        if self.i > 0 {
            self.write_u32(0, 8 - self.i)
        } else {
            Ok(())
        }
    }

    pub fn finish(mut self) -> io::Result<W> {
        self.try_finish()?;
        Ok(self.inner)
    }

    pub fn write_u32(&mut self, value: u32, len: usize) -> io::Result<()> {
        if len == 0 {
            return Ok(());
        } else if len >= 32 {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }

        let mut mask = 0x01 << (len - 1);

        for _ in 0..len {
            self.write_bit(value & mask != 0)?;
            mask >>= 1;
        }

        Ok(())
    }

    fn write_bit(&mut self, is_set: bool) -> io::Result<()> {
        if is_set {
            self.buf |= 0x01 << (8 - self.i - 1);
        }

        self.i += 1;

        if self.i == 8 {
            self.inner.write_u8(self.buf)?;
            self.buf = 0;
            self.i = 0;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_u32() -> io::Result<()> {
        let mut writer = BitWriter::new(Vec::new());

        writer.write_u32(0x0c, 4)?;
        writer.write_u32(0x03, 2)?;
        writer.write_u32(0x34, 6)?;
        writer.try_finish()?;

        let expected = [0b11001111, 0b01000000];
        assert_eq!(writer.get_ref(), &expected);

        Ok(())
    }

    #[test]
    fn test_write_u32_with_0_len() -> io::Result<()> {
        let mut writer = BitWriter::new(Vec::new());
        writer.write_u32(0xff, 0)?;
        assert!(writer.get_ref().is_empty());
        Ok(())
    }

    #[test]
    fn test_write_u32_with_length_greater_than_32_bits() {
        let mut writer = BitWriter::new(Vec::new());
        assert!(writer.write_u32(0xff, 33).is_err());
    }
}
