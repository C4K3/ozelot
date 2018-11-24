use ClientState;
use errors::{Result, ResultExt};
use read::read_varint;
use write::write_varint;

use std::io::{Cursor, Read, Write};
use std::marker::PhantomData;
use std::net::Shutdown;
use std::net::TcpStream;
use std::{io, time};

use netbuf::Buf;

use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;
use flate2::Compression;

use openssl::symm;

/// Trait for the two enums ClientboundPacket and ServerboundPacket
pub trait Packet: Sized {
    /// Deserializes a Read type into a packet. You usually won't need to use this.
    fn deserialize<R: Read>(r: &mut R, state: &ClientState) -> Result<Self>;
    /// Returns the packet's name
    fn get_packet_name(&self) -> &str;
    /// Returns the connection state in which the packet can be sent
    fn get_clientstate(&self) -> ClientState;
    /// Returns the ID of the packet
    fn get_id(&self) -> i32;
    /// Serializes the packet into Vec<u8>. You usually won't need to use this.
    fn to_u8(&self) -> Result<Vec<u8>>;
}

/// Represents a single MC connection, either as client or server
pub struct Connection<I: Packet, O: Packet> {
    stream: TcpStream,
    clientstate: ClientState,
    /* The buffer for incoming packets */
    buf: Buf,
    /* This tracks the length the next packet in the buffer.
     * If None, then we haven't received enough bytes to completely figure out
     * the packet length (usually means we haven't received anything, but MIGHT
     * also mean we haven't received the full header yet)
     *
     * If Some(x) then we need to read x bytes from the buf to get the complete
     * packet (excluding packet length header, including encryption/compression/
     * packet id header) */
    packet_len: Option<usize>,
    compression: Option<usize>,
    /// Buffer for outgoing data
    out_buf: Buf,
    /* Incoming encryption cipher */
    in_encryption: Option<symm::Crypter>,
    /* Outgoing encryption cipher */
    out_encryption: Option<symm::Crypter>,
    /* When we last read something from the server. Use this to timeout the
     * connection if the connection is lost */
    last_read: time::Instant,
    in_type: PhantomData<I>,
    out_type: PhantomData<O>,
}
impl<I: Packet, O: Packet> Connection<I, O> {
    pub fn from_tcpstream(stream: TcpStream) -> Result<Self> {
        let conn = Connection {
            stream: stream,
            clientstate: ClientState::Handshake,
            buf: Buf::new(),
            packet_len: None,
            compression: None,
            out_buf: Buf::new(),
            in_encryption: None,
            out_encryption: None,
            last_read: time::Instant::now(),
            in_type: PhantomData,
            out_type: PhantomData,
        };
        /* Set 30 second timeout */
        conn.stream.set_read_timeout(Some(time::Duration::new(30, 0)))?;
        conn.stream.set_write_timeout(Some(time::Duration::new(30, 0)))?;
        conn.stream.set_nonblocking(true)?;
        conn.stream.set_nodelay(true)?;
        Ok(conn)
    }

    pub fn connect_tcp(host: &str, port: u16) -> Result<Self> {
        let stream = TcpStream::connect(&format!("{}:{}", host, port))?;
        Ok(Connection::from_tcpstream(stream)?)
    }

    /// Send the given packet
    ///
    /// This adds the packet to the outgoing buffer, and sends as much as is
    /// possible. Returns the length of the outgoing buffer. If this is greater
    /// than 0, you will need to call write() to send the remaining data.
    pub fn send(&mut self, packet: &O) -> Result<usize> {
        let tmp = packet.to_u8()?;
        let uncompressed_length = tmp.len();
        let mut out = Vec::with_capacity(uncompressed_length);

        match self.compression {
            /* Compression is enabled, and the packet length is over the
             * threshold, thus compress the packet */
            Some(threshold) if uncompressed_length >= threshold => {
                /* We have to copy all the data again, because we need
                 * to prefix the packet with length of the compressed data */
                let mut compressed = Vec::new();
                write_varint(&(uncompressed_length as i32), &mut compressed)?;
                let mut compressor = ZlibEncoder::new(compressed, Compression::default());
                compressor.write_all(&tmp)?;
                let compressed = compressor.finish()?;

                write_varint(&(compressed.len() as i32), &mut out)?;
                out.write_all(&compressed)?;
            },
            /* Compression is enabled, but the packet length is not over the
             * threshold, thus we don't compress the packet */
            Some(_) => {
                /* Add 1 to the uncompressed length for the 1 byte it takes
                 * to specify no compression */
                write_varint(&((uncompressed_length + 1) as i32), &mut out)?;
                write_varint(&0, &mut out)?;
                out.write_all(&tmp)?;
            },
            /* Compression is not enabled */
            None => {
                write_varint(&(uncompressed_length as i32), &mut out)?;
                out.write_all(&tmp)?;
            },
        }

        if let Some(ref mut enc) = self.out_encryption {
            let mut tmp = vec![0; out.len() + 16];
            let n = enc.update(&out, &mut tmp).chain_err(|| "connection::send error writing encrypted data")?;
            let mut i = 0;
            while i < n {
                i += self.out_buf.write(&tmp[i..n])?;
            }
        } else {
            let mut i = 0;
            while i < out.len() {
                i += self.out_buf.write(&out[i..])?;
            }
        }

        match self.out_buf.write_to(&mut self.stream) {
            Ok(_) => (),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => (),
            Err(e) => bail!(e),
        }
        Ok(self.out_buf.len())
    }

    /// Write from the outgoing buffer to the TcpStream
    ///
    /// Returns the amount of bytes written.
    pub fn write(&mut self) -> Result<usize> {
        return Ok(self.out_buf.write_to(&mut self.stream)?);
    }

    /// Attempt to close this connection.
    ///
    /// All future sends and reads to this connection will fail
    pub fn close(&mut self) -> Result<()> {
        Ok(self.stream.shutdown(Shutdown::Both)?)
    }

    /// Change the client state of this connection
    pub fn set_clientstate(&mut self, new_state: ClientState) {
        self.clientstate = new_state;
    }

    /// Enable encryption with the given key.
    ///
    /// It is an error to enable encryption if encryption has already been
    /// enabled.
    pub fn enable_encryption(&mut self, key: &[u8; 16]) {
        let out_cipher =
            symm::Crypter::new(symm::Cipher::aes_128_cfb8(),
                               symm::Mode::Encrypt,
                               key,
                               Some(key))
                    .expect("client::enable_encryption error creating cipher");
        let in_cipher =
            symm::Crypter::new(symm::Cipher::aes_128_cfb8(),
                               symm::Mode::Decrypt,
                               key,
                               Some(key))
                    .expect("client::enable_encryption error creating cipher");

        self.out_encryption = Some(out_cipher);
        self.in_encryption = Some(in_cipher);
    }

    /// Enable compression.
    ///
    /// It is generally an error to enable compression if compression has
    /// already been enabled.
    pub fn enable_compression(&mut self, threshold: usize) {
        self.compression = Some(threshold);
    }

    /// Read from the TcpStream and update the incoming buffer.
    ///
    /// This is the only way to actually read from the TcpStream. Unless you
    /// know for sure you need to call this, then you do not need to call this.
    /// I.e. if you're just using client.read(), then you do not need to call
    /// this function.
    pub fn update_inbuf(&mut self) -> Result<()> {
        if let Some(ref mut enc) = self.in_encryption {
            let mut enc_buf = Buf::new();
            let n = match enc_buf.read_from(&mut self.stream) {
                Ok(n) => n,
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => 0,
                Err(e) => bail!(e),
            };
            let mut tmp = vec![0; n + 16];
            let n = enc.update(&enc_buf[..], &mut tmp).chain_err(|| "connection::update_inbuf error reading encrypted data")?;
            self.buf.extend(&tmp[..n]);
        } else {
            match self.buf.read_from(&mut self.stream) {
                Ok(_) => (),
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => (),
                Err(e) => bail!(e),
            };
        }
        Ok(())
    }

    /// Read a single packet from the internal buffer.
    ///
    /// This is only really useful if you want finegrained control over the
    /// processing of packets, or if you want to manually authenticate with
    /// the server. In most cases, you'll want to just call client.read().
    ///
    /// You MUST be sure that client.update_inbuf() has been called before this,
    /// this function will not attempt to read from the TcpStream, only from the
    /// internal buffer.
    pub(crate) fn read_packet(&mut self) -> Result<Option<I>> {
        if let None = self.packet_len {
            self.read_length()?;
        }

        let len = match self.packet_len {
            Some(x) => x,
            None => {
                if self.last_read.elapsed() > time::Duration::new(30, 0) {
                    /* If we haven't read anything for 30 seconds, timeout */
                    self.close()?;
                    bail!("Read timeout");
                } else {
                    return Ok(None);
                }
            },
        };

        if self.buf.len() < len {
            /* We haven't received enough yet to read the whole packet */
            if self.last_read.elapsed() > time::Duration::new(30, 0) {
                /* If we haven't read anything for 30 seconds, timeout */
                self.close()?;
                bail!("Read timeout");
            } else {
                return Ok(None);
            }
        } else {
            self.last_read = time::Instant::now();
        }

        let packet = {
            let data = &self.buf[..len];
            let mut r = Cursor::new(data);

            match self.compression {
                Some(_) => {
                    let compressed_length = read_varint(&mut r)?;
                    if compressed_length == 0 {
                        /* Compression is enabled, but the given packet
                         * is not compressed */
                        I::deserialize(&mut r, &self.clientstate)?
                    } else {
                        /* Compression is enabled, and the given packet
                         * is compressed */
                        let mut r = ZlibDecoder::new(r);
                        I::deserialize(&mut r, &self.clientstate)?
                    }
                },
                /* Compression is not enabled */
                None => I::deserialize(&mut r, &self.clientstate)?,
            }
        };

        self.buf.consume(len);
        self.packet_len = None;

        Ok(Some(packet))
    }

    /** Tries to read the length of the next packet in the buf, and sets
     * self.packet_len accordingly. It will return Ok(()) as long as it doesn't
     * encounter any io errors, even if it doesn't read the whole length
     * (for example if the buffer is empty.) It will only consume the length
     * header from the buf if it successfully reads the entire length header */
    fn read_length(&mut self) -> Result<()> {
        let msb: u8 = 128; /* Only the MSB set */
        let mut i: usize = 0;

        /* The result */
        let res: usize = {
            let mut tmp = match self.buf.get(i) {
                Some(x) => x,
                None => return Ok(()),
            };

            let mut res = (tmp & (!msb)) as usize;

            /* While the previous byte had the MSB set */
            while (tmp & msb) != 0 {
                i += 1;

                /* A varint can be at most 5 bytes, remember it's nullindexed */
                if i >= 5 {
                    bail!("Received varint that was too long");
                }

                tmp = match self.buf.get(i) {
                    Some(x) => x,
                    None => return Ok(()),
                };

                res += ((tmp & (!msb)) as usize) << (7 * i);
            }

            res
        };

        self.packet_len = Some(res);
        /* Consume the length header as we no longer need it */
        self.buf.consume(i + 1);
        Ok(())
    }
}
