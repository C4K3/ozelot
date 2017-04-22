use clientbound::ClientboundPacket;
use connection::Connection;
use errors::Result;
use json::AuthenticationResponse;
use serverbound::ServerboundPacket;
use {ClientState, PROTOCOL_VERSION, mojang, serverbound, utils};

use std::{thread, time};

/// Represents a single client connection to a Server.
pub struct Client {
    conn: Connection<ClientboundPacket, ServerboundPacket>,
    auto_handle: bool,
    hide_handled: bool,
}
impl Client {
    /// Attempt open the tcp connection to the given host and port, and
    /// nothing more. If you use this you must then send all subsequent
    /// packets manually to authenticate and so on.
    pub fn connect_tcp(host: &str, port: u16) -> Result<Self> {
        Ok(Client {
               conn: Connection::connect_tcp(host, port)?,
               auto_handle: false,
               hide_handled: false,
           })
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
    pub fn connect_unauthenticated(host: &str,
                                   port: u16,
                                   username: &str)
                                   -> Result<Self> {

        let timeout = time::Instant::now();
        let mut client = Client::connect_tcp(host, port)?;
        client.set_auto_handle(true);
        client.set_hide_handled(true);
        let handshake = serverbound::Handshake::new(PROTOCOL_VERSION,
                                                    host.to_string(),
                                                    port,
                                                    2);
        let loginstart = serverbound::LoginStart::new(username.to_string());
        client.send(handshake)?;
        client.set_clientstate(ClientState::Login);
        client.send(loginstart)?;

        /* Now we wait for the PlayerAbilities packet from the server */
        'wait: loop {
            if timeout.elapsed() > time::Duration::new(30, 0) {
                bail!("Timed out waiting for LoginSuccess");
            }
            client.update_inbuf()?;
            match client.read_packet()? {
                Some(ClientboundPacket::LoginDisconnect(ref p)) => {
                    bail!("Got LoginDisconnect, reason: {}", p.get_raw_chat());
                },
                Some(ClientboundPacket::PlayerAbilities(..)) => break 'wait,
                Some(ClientboundPacket::EncryptionRequest(..)) => {
                    bail!("connect_unauthenticated got EncryptionRequest");
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
    /// use ozelot::{mojang, Client};
    /// let auth = mojang::Authenticate::new("my_email@example.com".to_string(),
    ///                                      "my_password".to_string())
    ///     .perform().unwrap();
    /// let mut client = Client::connect_authenticated("minecraft.example.com",
    /// 25565,
    /// &auth).unwrap();
    /// ```
    pub fn connect_authenticated(host: &str,
                                 port: u16,
                                 auth: &AuthenticationResponse)
                                 -> Result<Self> {

        let timeout = time::Instant::now();
        let mut client = Client::connect_tcp(host, port)?;
        client.set_auto_handle(true);
        client.set_hide_handled(true);
        let handshake = serverbound::Handshake::new(PROTOCOL_VERSION,
                                                    host.to_string(),
                                                    port,
                                                    2);
        let loginstart =
            serverbound::LoginStart::new(auth.selectedProfile.name.clone());
        client.send(handshake)?;
        client.set_clientstate(ClientState::Login);
        client.send(loginstart)?;

        /* Here we wait for a LoginSuccess/EncryptionRequest packet */
        'wait: loop {
            if timeout.elapsed() > time::Duration::new(30, 0) {
                bail!("Timed out waiting for LoginSuccess/EncryptionRequest");
            }
            client.update_inbuf()?;
            match client.read_packet()? {
                Some(ClientboundPacket::LoginDisconnect(ref p)) => {
                    bail!("Got LoginDisconnect, reason: {}", p.get_raw_chat());
                },
                Some(ClientboundPacket::LoginSuccess(..)) => bail!("Logged in unauthenticated"),
                Some(ClientboundPacket::EncryptionRequest(ref p)) => {
                    let shared_secret = utils::create_shared_secret();

                    mojang::SessionJoin::new(auth.accessToken.clone(),
                                             auth.selectedProfile.id.clone(),
                                             p.get_server_id(),
                                             &shared_secret,
                                             p.get_public_key())
                            .perform()?;

                    let encryptionresponse
                            = serverbound::EncryptionResponse::new_unencrypted(
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
                bail!("Timed out waiting for PlayerAbilities packet");
            }
            client.update_inbuf()?;
            match client.read_packet()? {
                Some(ClientboundPacket::LoginDisconnect(ref p)) => {
                    bail!("Got LoginDisconnect, reason: {}", p.get_raw_chat());
                },
                Some(ClientboundPacket::PlayerAbilities(..)) => break 'wait2,
                Some(_) => (),
                None => thread::sleep(time::Duration::from_millis(10)),
            }
        }

        Ok(client)
    }

    /// Try to read some packets from the server.
    ///
    /// This function is nonblocking.
    pub fn read(&mut self) -> Result<Vec<ClientboundPacket>> {
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

    /// Send the given packet to the server that we're connected to.
    ///
    /// This function may block.
    pub fn send(&mut self, packet: ServerboundPacket) -> Result<()> {
        self.conn.send(packet)
    }

    /// Whether to automatically handle: KeepAlive, LoginSuccess and
    /// SetCompression packets. Most clients won't need to manually deal with
    /// these.
    pub fn set_auto_handle(&mut self, new: bool) {
        self.auto_handle = new;
    }

    /// Whether or not to hide packets that have been handled by ozelot from the
    /// consumer of the library.
    ///
    /// E.g. KeepAlives that are automatically handled if auto_handle = true.
    pub fn set_hide_handled(&mut self, new: bool) {
        self.hide_handled = new;
    }

    /// Attempt to close this connection, disconnecting from the server.
    ///
    /// All future sends and reads to this connection will fail.
    pub fn close(&mut self) -> Result<()> {
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
    /// I.e. if you're just using client.read(), then you do not need to call
    /// this function.
    ///
    /// This function is nonblocking.
    pub fn update_inbuf(&mut self) -> Result<()> {
        self.conn.update_inbuf()
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
    pub fn read_packet(&mut self) -> Result<Option<ClientboundPacket>> {
        let packet = self.conn.read_packet()?;

        if self.auto_handle {
            match &packet {
                &Some(ClientboundPacket::LoginSuccess(..)) => {
                    self.set_clientstate(ClientState::Play);
                },
                &Some(ClientboundPacket::SetCompression(ref p)) => {
                    self.enable_compression(*p.get_threshold() as usize);
                },
                &Some(ClientboundPacket::KeepAlive(ref p)) => {
                    let keepalive = serverbound::KeepAlive::new(*p.get_id());
                    self.send(keepalive)?;
                },
                _ => (),
            }
        }

        Ok(packet)
    }
}
