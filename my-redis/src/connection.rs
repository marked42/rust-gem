use crate::frame::{Error, Frame};
use bytes::{BufMut, BytesMut};
use mini_redis::Result;
use std::io::Cursor;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }

    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                return if self.buffer.is_empty() {
                    Ok(None)
                } else {
                    Err("connection closed".into())
                };
            }
        }
    }

    pub async fn write_frame(&mut self, frame: &Frame) -> io::Result<()> {
        let mut frames_to_write = vec![frame];

        while let Some(frame) = frames_to_write.pop() {
            match frame {
                Frame::Simple(val) => {
                    self.stream.write_u8(b'+').await?;
                    self.stream.write_all(val.as_bytes()).await?;
                    self.stream.write_all(b"\r\n").await?;
                }
                Frame::Error(val) => {
                    self.stream.write_u8(b'-').await?;
                    self.stream.write_all(val.as_bytes()).await?;
                    self.stream.write_all(b"\r\n").await?;
                }
                Frame::Integer(val) => {
                    self.stream.write_u8(b':').await?;
                    self.stream.write_u64(*val).await?;
                }
                Frame::Bulk(val) => {
                    let len = val.len();

                    self.stream.write_u8(b'$').await?;
                    self.stream.write_u64(len as u64).await?;
                    self.stream.write_all(val).await?;
                    self.stream.write_all(b"\r\n").await?;
                }
                Frame::Null => {
                    self.stream.write_all(b"$-1\r\n").await?;
                }
                Frame::Array(val) => {
                    self.stream.write_u8(b'*').await?;
                    self.stream.write_u64(val.len() as u64).await?;

                    // 将子帧逆序压入栈，保持正序写入
                    for f in val.iter().rev() {
                        frames_to_write.push(f);
                    }
                    continue; // 继续处理栈中的下一个帧
                }
            }
        }

        // 一个frame一次flush比较低效, connection 可以提供flush函数将多个frame积累一起flush
        self.stream.flush().await?;

        Ok(())
    }

    pub fn parse_frame(&mut self) -> Result<Option<Frame>> {
        // Cursor 提供 seek 操作，可以回退
        let mut buf = Cursor::new(&self.buffer[..]);

        match Frame::check(&mut buf) {
            Ok(_) => {
                let len = buf.position() as usize;
                // 回退
                buf.set_position(0);

                let frame = Frame::parse(&mut buf)?;
                unsafe {
                    // 真正消耗，将解析得到的字节移出缓冲区
                    self.buffer.advance_mut(len);
                }

                Ok(Some(frame))
            }
            Err(Error::Incomplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
