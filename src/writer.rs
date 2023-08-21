use cid::Cid;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::{error::Error, header::CarHeader, util::write_varint_usize};

#[derive(Debug)]
pub struct CarWriter<W> {
    header: CarHeader,
    writer: W,
    cid_buffer: Vec<u8>,
    is_header_written: bool,
}

impl<W> CarWriter<W>
where
    W: AsyncWrite + Send + Unpin,
{
    pub fn new(header: CarHeader, writer: W) -> Self {
        CarWriter {
            header,
            writer,
            cid_buffer: Vec::new(),
            is_header_written: false,
        }
    }

    /// Forces the header to be written. Also called implicitly by `write`.
    ///
    /// Returns the bytes written in this operation.
    pub async fn write_header(&mut self) -> Result<usize, Error> {
        let mut written = 0;

        if !self.is_header_written {
            // Write header bytes
            let header_bytes = self.header.encode()?;
            written += write_varint_usize(header_bytes.len(), &mut self.writer).await?;
            self.writer.write_all(&header_bytes).await?;
            written += header_bytes.len();
            self.is_header_written = true;
        }

        Ok(written)
    }

    /// Writes header and stream of data to writer in Car format.
    ///
    /// Returns the bytes written in this operation.
    pub async fn write<T>(&mut self, cid: Cid, data: T) -> Result<usize, Error>
    where
        T: AsRef<[u8]>,
    {
        let mut written = 0;
        written += self.write_header().await?;

        // Write the given block.
        self.cid_buffer.clear();
        cid.write_bytes(&mut self.cid_buffer).expect("vec write");

        let data = data.as_ref();
        let len = self.cid_buffer.len() + data.len();

        written += write_varint_usize(len, &mut self.writer).await?;
        self.writer.write_all(&self.cid_buffer).await?;
        self.writer.write_all(data).await?;
        written += self.cid_buffer.len();
        written += data.len();

        Ok(written)
    }

    /// Finishes writing, including flushing and returns the writer.
    pub async fn finish(mut self) -> Result<W, Error> {
        self.flush().await?;
        Ok(self.writer)
    }

    /// Flushes the underlying writer.
    pub async fn flush(&mut self) -> Result<(), Error> {
        self.writer.flush().await?;
        Ok(())
    }

    /// Consumes the [`CarWriter`] and returns the underlying writer.
    pub fn into_inner(self) -> W {
        self.writer
    }
}
