use {ClientState, Sendable, client_send, yggdrasil, PROTOCOL_VERSION};
use client_recv::ClientboundPacket;
use write::write_varint;
use read::read_varint;

use std::{thread, time, io};
use std::io::{Write, Cursor};
use std::net::Shutdown;
use std::net::TcpStream;
use std::ops::Deref;

use netbuf::Buf;

use flate2;
use flate2::Compress;
use flate2::read::ZlibDecoder;

use openssl::symm;

/// Represents a single client connection to a Server.
pub struct Client {
    stream: TcpStream,
    clientstate: ClientState,
    /* The buffer for clientbound packets */
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
    /* Whether to respond to pings */
    auto_handle: bool,
    /* Whether to forward packets that have been handled by ozelot, i.e.
     * KeepAlive's if ping_respond = true */
    hide_handled: bool,
    /* Incoming encryption cipher */
    in_encryption: Option<symm::Crypter>,
    /* Outgoing encryption cipher */
    out_encryption: Option<symm::Crypter>,
    /* When we last read something from the server. Use this to timeout the
     * connection if the connection is lost */
    last_read: time::Instant,
}
impl Client {
    /// Attempt open the tcp connection to the given host and port, and
    /// nothing more. If you use this you must then send all subsequent
    /// packets manually to authenticate and so on.
    pub fn connect_tcp(host: &str, port: u16) -> io::Result<Self> {
        let client = Client {
            stream: TcpStream::connect(format!("{}:{}", host, port).deref())?,
            clientstate: ClientState::Handshake,
            buf: Buf::new(),
            packet_len: None,
            compression: None,
            auto_handle: false,
            hide_handled: false,
            in_encryption: None,
            out_encryption: None,
            last_read: time::Instant::now(),
        };
        /* Set 30 second timeout */
        client.stream.set_read_timeout(Some(time::Duration::new(30, 0)))?;
        client.stream.set_write_timeout(Some(time::Duration::new(30, 0)))?;
        Ok(client)
    }

    /// Attempt to connect to the server at the given host and port,
    /// completing the usual steps for an unauthenticated login.
    ///
    /// In most cases this is what you will want for offline/local servers
    /// and singleplayer. This will establish an unencrypted connection
    /// to the server.
    ///
    /// The username is the Minecraft username, and not the Mojang username,
    /// (i.e. NOT the email address for migrated accounts.)
    ///
    /// This will set auto_handle and hide_handled to true.
    ///
    /// This will return Ok(Client) on receival of a PlayerAbilities packet,
    /// but note that the PlayerAbilities packet and all packets received before
    /// it will not be available for consumers. If you need any of those
    /// packets, you will want to do the authentication manually.
    ///
    /// This function will time out after 30 seconds (theoretical absolute worst
    /// case 60 seconds.)
    pub fn connect_unauthenticated(host: &str, port: u16, username: &str)
        -> io::Result<Self> {

        let timeout = time::Instant::now();
        let mut client = Client::connect_tcp(host, port)?;
        client.set_auto_handle(true);
        client.set_hide_handled(true);
        let handshake = client_send::Handshake::new(PROTOCOL_VERSION,
                                                    host.to_string(),
                                                    port,
                                                    2);
        let loginstart = client_send::LoginStart::new(username.to_string());
        client.send(handshake)?;
        client.set_clientstate(ClientState::Login);
        client.send(loginstart)?;

        /* Now we wait for the PlayerAbilities packet from the server */
        'wait: loop {
            if timeout.elapsed() > time::Duration::new(30, 0) {
                return io_error!(
                    "Timed out waiting for LoginSuccess/EncryptionRequest");
            }
            client.update_inbuf()?;
            match client.read_packet()? {
                Some(ClientboundPacket::LoginDisconnect(ref p)) => {
                    return io_error!("Got LoginDisconnect, reason: {}",
                                     p.get_raw_chat());
                },
                Some(ClientboundPacket::PlayerAbilities(..)) => break 'wait,
                Some(ClientboundPacket::EncryptionRequest(..)) => {
                    return io_error!(
                        "connect_unauthenticated got EncryptionRequest");
                },
                Some(_) => (),
                None => thread::sleep(time::Duration::from_millis(10)),
            }
        }

        Ok(client)
    }

    /// Authenticate with Mojang, and then connect to the server at the
    /// given host and port. In most cases this is what you will want to use
    /// to connect to online servers.
    ///
    /// This will establish an encrypted connection to the server.
    ///
    /// Requires that you've already authenticated with Mojang, e.g. by calling
    /// yggdrasil::authenticate
    ///
    /// This will set auto_handle and hide_handled to true.
    ///
    /// This will return Ok(Client) on receival of a PlayerAbilities packet,
    /// but note that the PlayerAbilities packet and all packets received before
    /// it will not be available for consumers. If you need any of those
    /// packets, you will want to do the authentication manually.
    ///
    /// This function will time out after 30 seconds (theoretical absolute worst
    /// case 60 seconds.)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ozelot::{yggdrasil, Client};
    /// let (access_token, _, username, uuid) = yggdrasil::authenticate("my_email@example.com",
    ///                                                  "my_password").unwrap();
    /// let mut client = Client::connect_authenticated("minecraft.example.com", 25565, &access_token, &username, &uuid).unwrap();
    /// ```
    pub fn connect_authenticated(host: &str,
                                 port: u16,
                                 access_token: &str,
                                 username: &str,
                                 uuid: &str) -> io::Result<Self> {

        let timeout = time::Instant::now();
        let mut client = Client::connect_tcp(host, port)?;
        client.set_auto_handle(true);
        client.set_hide_handled(true);
        let handshake = client_send::Handshake::new(
            PROTOCOL_VERSION,
            host.to_string(),
            port,
            2);
        let loginstart = client_send::LoginStart::new(username.to_string());
        client.send(handshake)?;
        client.set_clientstate(ClientState::Login);
        client.send(loginstart)?;

        /* Here we wait for a LoginSuccess/EncryptionRequest packet */
        'wait: loop {
            if timeout.elapsed() > time::Duration::new(30, 0) {
                return io_error!(
                    "Timed out waiting for LoginSuccess/EncryptionRequest");
            }
            client.update_inbuf()?;
            match client.read_packet()? {
                Some(ClientboundPacket::LoginDisconnect(ref p)) => {
                    return io_error!("Got LoginDisconnect, reason: {}",
                                     p.get_raw_chat());
                },
                Some(ClientboundPacket::LoginSuccess(..)) =>
                    return io_error!("Logged in unauthenticated"),
                Some(ClientboundPacket::EncryptionRequest(ref p)) => {
                    let shared_secret = yggdrasil::create_shared_secret();

                    yggdrasil::session_join(&access_token,
                                            &uuid,
                                            p.get_server_id(),
                                            &shared_secret,
                                            p.get_public_key())?;

                    let encryptionresponse
                        = client_send::EncryptionResponse::new_unencrypted(
                            &p.get_public_key(),
                            &shared_secret,
                            &p.get_verify_token())?;
                    client.send(encryptionresponse)?;
                    client.enable_encryption(&shared_secret);

                    break 'wait;
                },
                Some(_) => (),
                None => thread::sleep(time::Duration::from_millis(10)),
            }
        }

        /* Now we wait for the PlayerAbilities packet from the server */
        'wait2: loop {
            if timeout.elapsed() > time::Duration::new(30, 0) {
                return io_error!(
                    "Timed out waiting for LoginSuccess/EncryptionRequest");
            }
            client.update_inbuf()?;
            match client.read_packet()? {
                Some(ClientboundPacket::LoginDisconnect(ref p)) => {
                    return io_error!("Got LoginDisconnect, reason: {}",
                                     p.get_raw_chat());
                },
                Some(ClientboundPacket::PlayerAbilities(..)) => break 'wait2,
                Some(_) => (),
                None => thread::sleep(time::Duration::from_millis(10)),
            }
        }

        Ok(client)
    }

    /// Send the given packet to the server that we're connected to.
    pub fn send<P: Sendable>(&mut self, packet: P) -> io::Result<()> {
        let tmp = packet.to_u8()?;
        let uncompressed_length = tmp.len();
        let mut out = Vec::with_capacity(uncompressed_length);

        match self.compression {
            Some(threshold) if uncompressed_length >= threshold => {
                /* We have to copy all the data again, because we need
                 * to prefix the packet with length of the compressed data */
                let mut output = Vec::new();
                let mut compressor = Compress::new(::COMPRESSION_LEVEL, false);
                match compressor.compress(&tmp, &mut output,
                                          flate2::Flush::Sync) {
                    flate2::Status::Ok => {
                        return io_error!("Got a Status::Ok when trying to compress outgoing packet");
                    },
                    flate2::Status::BufError => {
                        return io_error!("Got a Status::BufError when trying to compress outgoing packet");
                    },
                    flate2::Status::StreamEnd => (),
                }

                write_varint(&(output.len() as i32), &mut out)?;
                write_varint(&(uncompressed_length as i32), &mut out)?;
                out.write_all(&output)?;
            },
            Some(_) => {
                /* Add 1 to the uncompressed length for the 1 byte it takes
                 * to specify no compression */
                write_varint(&((uncompressed_length + 1) as i32), &mut out)?;
                write_varint(&0, &mut out)?;
                out.write_all(&tmp)?;
            },
            None => {
                write_varint(&(uncompressed_length as i32), &mut out)?;
                out.write_all(&tmp)?;
            }
        }

        if let Some(ref mut enc) = self.out_encryption {
            let mut tmp = vec![0; out.len() + 16];
            let n = match enc.update(&out, &mut tmp) {
                Ok(x) => x,
                Err(_) => return io_error!(
                    "client::send error writing encrypted data"),
            };
            self.stream.write_all(&tmp[..n])?;
        } else {
            self.stream.write_all(&out)?;
        }

        Ok(())
    }

    /// Try to read some packets from the server.
    ///
    /// This function may in rare circumstances block for up to 30 seconds,
    /// specifically the timeout for reading the TcpStream is 30 seconds,
    /// so if the tcp connection is lost, it will block until the timeout.
    /// During normal back and forth with a server it should return as fast as
    /// possible.
    pub fn read(&mut self) -> io::Result<Vec<ClientboundPacket>> {
        self.update_inbuf()?;

        let mut ret = Vec::new();
        loop {
            let packet = self.read_packet()?;
            /* push = whether to push the packet to ret */
            let mut push = match &packet {
                &Some(ClientboundPacket::LoginSuccess(_)) => false,
                &Some(ClientboundPacket::SetCompression(_)) => false,
                &Some(ClientboundPacket::KeepAlive(_)) => false,
                &Some(_) => true,
                &None => break,
            };
            if self.hide_handled == false || self.auto_handle == false {
                push = true;
            }

            if push {
                ret.push(packet
                         .expect("unreachable packet = None in client.read()"));
            }
        }

        Ok(ret)
    }

    /// Attempt to close this connection, disconnecting from the server.
    ///
    /// All future sends and reads to this connection will fail
    pub fn close(&mut self) -> io::Result<()> {
        self.stream.shutdown(Shutdown::Both)
    }

    /// Change the client state of this connection
    pub fn set_clientstate(&mut self, new_state: ClientState) {
        self.clientstate = new_state;
    }

    /// Whether to automatically handle: KeepAlive, LoginSuccess and
    /// SetCompression packets. Most clients won't need to manually deal with
    /// these.
    pub fn set_auto_handle(&mut self, new: bool) {
        self.auto_handle= new;
    }

    /// Whether or not to hide packets that have been handled by ozelot from the
    /// consumer of the library.
    ///
    /// E.g. KeepAlives that are automatically handled if auto_handle = true.
    pub fn set_hide_handled(&mut self, new: bool) {
        self.hide_handled = new;
    }

    /// Enable encryption with the given key.
    ///
    /// It is an error to enable encryption if encryption has already been
    /// enabled.
    pub fn enable_encryption(&mut self, secret: &[u8; 16]) {
        let out = symm::Crypter::new(
            symm::Cipher::aes_128_cfb8(),
            symm::Mode::Encrypt,
            secret,
            Some(secret))
            .expect("client::enable_encryption error creating out cipher");

        let enc_in = symm::Crypter::new(
            symm::Cipher::aes_128_cfb8(),
            symm::Mode::Decrypt,
            secret,
            Some(secret))
            .expect("client::enable_encryption error creating in cipher");

        self.out_encryption = Some(out);
        self.in_encryption = Some(enc_in);
    }

    /// Read from the TcpStream and update the incoming buffer.
    ///
    /// This is the only way to actually read from the TcpStream. Unless you
    /// know for sure you need to call this, then you do not need to call this.
    /// I.e. if you're just using client.read(), then you do not need to call
    /// this function.
    pub fn update_inbuf(&mut self) -> io::Result<()> {
        if let Some(ref mut enc) = self.in_encryption {
            let mut enc_buf = Buf::new();
            let n = enc_buf.read_from(&mut self.stream)?;
            let mut tmp = vec![0; n + 16];
            let n = match enc.update(&enc_buf[..], &mut tmp) {
                Ok(x) => x,
                Err(_) => return io_error!(
                    "client::read error reading encrypted data"),
            };
            self.buf.extend(&tmp[..n]);
        } else {
            let _: usize = self.buf.read_from(&mut self.stream)?;
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
    pub fn read_packet(&mut self) -> io::Result<Option<ClientboundPacket>> {
        if let None = self.packet_len {
            self.read_length()?;
        }

        let len = match self.packet_len {
            Some(x) => x,
            None => {
                if self.last_read.elapsed() > time::Duration::new(30, 0) {
                    /* If we haven't read anything for 30 seconds, timeout */
                    self.close()?;
                    return io_error!("Read timeout");
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
                return io_error!("Read timeout");
            } else {
                return Ok(None);
            }
        } else {
            self.last_read = time::Instant::now();
        }

        let packet = {
            let data = &self.buf[..len];
            let mut r = Cursor::new(data);

            /* This is where we decompress compressed packets and decrypt
             * encrypted packets */
            match self.compression {
                Some(_) => {
                    let compressed_length = read_varint(&mut r)?;

                    if compressed_length == 0 {
                        ClientboundPacket::parse(&mut r, &self.clientstate)?
                    } else {
                        let mut r = ZlibDecoder::new(r);
                        ClientboundPacket::parse(&mut r, &self.clientstate)?
                    }
                },
                None => {
                    ClientboundPacket::parse(&mut r, &self.clientstate)?
                },
            }
        };

        self.buf.consume(len);
        self.packet_len = None;

        if self.auto_handle {
            match &packet {
                &ClientboundPacket::LoginSuccess(_) => {
                    self.set_clientstate(ClientState::Play);
                },
                &ClientboundPacket::SetCompression(ref p) => {
                    self.compression = Some(*p.get_threshold() as usize);
                },
                &ClientboundPacket::KeepAlive(ref p) => {
                    let keepalive = client_send::KeepAlive::new(*p.get_id());
                    self.send(keepalive)?;
                },
                _ => (),
            }
        }

        Ok(Some(packet))

    }

    /** Tries to read the length of the next packet in the buf, and sets
     * self.packet_len accordingly. It will return Ok(()) as long as it doesn't
     * encounter any io errors, even if it doesn't read the whole length
     * (for example if the buffer is empty.) It will only consume the length
     * header from the buf if it successfully reads the entire length header */
    fn read_length(&mut self) -> io::Result<()> {
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
                    return io_error!("Received varint that was too long");
                }

                tmp = match self.buf.get(i) {
                    Some(x) => x,
                    None => return Ok(()),
                };

                res += ((tmp & (!msb)) as usize) << (7 * i);
            }

            res
        };

        //println!("Parsed length of desired packet to be {}", res);
        self.packet_len = Some(res);
        /* Consume the length header as we no longer need it */
        self.buf.consume(i + 1);
        Ok(())
    }

}

