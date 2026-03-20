use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use serde_json::Value;

use super::error::ProtocolError;
use super::rc4::Rc4;
use super::request::Request;
use super::response::Response;

const TIMEOUT: Duration = Duration::from_secs(3);
const MAX_RETRIES: u32 = 3;

/// A connection to a SpiceAPI server.
///
/// Handles TCP communication, RC4 encryption, null-byte framing,
/// session refresh, and retry logic.
pub struct Connection {
    host: String,
    port: u16,
    password: String,
    stream: TcpStream,
    cipher: Option<Rc4>,
}

impl Connection {
    /// Connect to a SpiceAPI server and perform session refresh.
    pub fn new(host: &str, port: u16, password: &str) -> Result<Self, ProtocolError> {
        let stream = Self::tcp_connect(host, port)?;
        let cipher = Self::make_cipher(password);

        let mut conn = Self {
            host: host.to_string(),
            port,
            password: password.to_string(),
            stream,
            cipher,
        };

        conn.session_refresh()?;
        Ok(conn)
    }

    /// Send a request and return the response data array.
    ///
    /// Retries up to 3 times on transient connection failures.
    /// API-level errors and ID mismatches are NOT retried.
    pub fn request(
        &mut self,
        module: &str,
        function: &str,
        params: Vec<Value>,
    ) -> Result<Vec<Value>, ProtocolError> {
        let mut last_err = None;

        for _ in 0..MAX_RETRIES {
            let mut req = Request::new(module, function);
            for p in &params {
                req.add_param(p.clone());
            }

            match self.send_request(&req) {
                Ok(resp) => return Ok(resp.data().to_vec()),
                Err(e @ (ProtocolError::Api(_) | ProtocolError::IdMismatch { .. })) => {
                    return Err(e);
                }
                Err(e) => {
                    last_err = Some(e);
                    let _ = self.reconnect();
                }
            }
        }

        Err(ProtocolError::RetriesExhausted {
            attempts: MAX_RETRIES,
            source: Box::new(last_err.unwrap_or(ProtocolError::MalformedResponse)),
        })
    }

    fn send_request(&mut self, req: &Request) -> Result<Response, ProtocolError> {
        let mut data = req.to_bytes()?;
        data.push(0x00); // null-byte terminator

        if let Some(cipher) = &mut self.cipher {
            cipher.crypt(&mut data);
        }

        self.stream.write_all(&data)?;

        let response_bytes = self.receive()?;
        if response_bytes.is_empty() {
            return Err(ProtocolError::MalformedResponse);
        }

        Response::parse(&response_bytes, req.id())
    }

    fn receive(&mut self) -> Result<Vec<u8>, ProtocolError> {
        let mut buf = [0u8; 4096];
        let mut result = Vec::new();

        loop {
            let n = self.stream.read(&mut buf)?;
            if n == 0 {
                return Err(ProtocolError::Connection(std::io::Error::new(
                    std::io::ErrorKind::ConnectionReset,
                    "connection closed by server",
                )));
            }

            let chunk = &mut buf[..n];
            if let Some(cipher) = &mut self.cipher {
                cipher.crypt(chunk);
            }

            // Check if the last decrypted byte is the null terminator
            if let Some(&last) = chunk.last() {
                if last == 0x00 {
                    result.extend_from_slice(&chunk[..chunk.len() - 1]);
                    return Ok(result);
                }
            }

            result.extend_from_slice(chunk);
        }
    }

    fn session_refresh(&mut self) -> Result<(), ProtocolError> {
        let req = Request::with_id(rand_u64(), "control", "session_refresh");
        let resp = self.send_request(&req)?;

        if let Some(Value::String(new_password)) = resp.data().first() {
            self.password = new_password.clone();
            self.cipher = Self::make_cipher(&self.password);
        }

        Ok(())
    }

    fn reconnect(&mut self) -> Result<(), ProtocolError> {
        self.stream = Self::tcp_connect(&self.host, self.port)?;
        self.cipher = Self::make_cipher(&self.password);
        self.session_refresh()
    }

    fn tcp_connect(host: &str, port: u16) -> Result<TcpStream, ProtocolError> {
        let stream = TcpStream::connect((host, port))?;
        stream.set_read_timeout(Some(TIMEOUT))?;
        stream.set_write_timeout(Some(TIMEOUT))?;
        stream.set_nodelay(true)?;
        Ok(stream)
    }

    fn make_cipher(password: &str) -> Option<Rc4> {
        if password.is_empty() {
            None
        } else {
            Some(Rc4::new(password.as_bytes()))
        }
    }
}

/// Generate a random u64 for session refresh request IDs.
/// Uses a simple approach without pulling in the `rand` crate.
fn rand_u64() -> u64 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    RandomState::new().build_hasher().finish()
}
