use clientbound::ClientboundPacket;
use serverbound::ServerboundPacket;
use connection::Connection;
use ClientState;

use std::io;
use std::net::TcpStream;

/// Represents a single client connection, from the point of view of a server
pub struct Server {
    conn: Connection<ServerboundPacket, ClientboundPacket>,
}
impl Server {
    /// Create a new connection from an existing TcpStream
    pub fn from_tcpstream(stream: TcpStream) -> io::Result<Self> {
        Ok(Server {
               conn: Connection::from_tcpstream(stream)?,
           })
    }

    /// Try to read some packets from the client.
    ///
    /// This function is nonblocking.
    pub fn read(&mut self) -> io::Result<Vec<ServerboundPacket>> {
        self.update_inbuf()?;

        let mut ret = Vec::new();
        loop {
            if let Some(packet) = self.read_packet()? {
                ret.push(packet);
            } else {
                break;
            }
        }
        Ok(ret)
    }

    /// Send the given packet to the client
    ///
    /// This function may block.
    pub fn send(&mut self, packet: ClientboundPacket) -> io::Result<()> {
        self.conn.send(packet)
    }

    /// Attempt to close this connection, disconnecting the client
    ///
    /// All future sends and reads to this connection will fail.
    pub fn close(&mut self) -> io::Result<()> {
        self.conn.close()
    }

    /// Change the client state of this connection
    pub fn set_clientstate(&mut self, new_state: ClientState) {
        self.conn.set_clientstate(new_state)
    }

    /// Enable encryption with the given key.
    ///
    /// It is an error to enable encryption if encryption has already been
    /// enabled.
    pub fn enable_encryption(&mut self, key: &[u8; 16]) {
        self.conn.enable_encryption(key)
    }

    /// Enable compression.
    ///
    /// It is generally an error to enable compression if compression has
    /// already been enabled.
    pub fn enable_compression(&mut self, threshold: usize) {
        self.conn.enable_compression(threshold)
    }

    /// Read from the TcpStream and update the incoming buffer.
    ///
    /// This is the only way to actually read from the TcpStream. Unless you
    /// know for sure you need to call this, then you do not need to call this.
    /// I.e. if you're just using server.read(), then you do not need to call
    /// this function.
    ///
    /// This function is nonblocking.
    pub fn update_inbuf(&mut self) -> io::Result<()> {
        self.conn.update_inbuf()
    }

    /// Read a single packet from the internal buffer.
    ///
    /// This is only really useful if you want finegrained control over the
    /// processing of packets, e.g. if you want to authenticate clients. In
    /// most cases you'll just
    /// want to use server.read()
    ///
    /// You MUST be sure that server.update_inbuf() has been called before this,
    /// this function will not attempt to read from the TcpStream, only from the
    /// internal buffer.
    pub fn read_packet(&mut self) -> io::Result<Option<ServerboundPacket>> {
        self.conn.read_packet()
    }
}
