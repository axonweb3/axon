use std::{io, str};

use tokio_util::codec::Decoder;

use protocol::types::BytesMut;

/// Separator for enveloping messages in streaming codecs
#[derive(Debug, Clone)]
pub enum Separator {
    /// No envelope is expected between messages. Decoder will try to figure out
    /// message boundaries by accumulating incoming bytes until valid JSON is
    /// formed. Encoder will send messages without any boundaries between
    /// requests.
    Empty,
    /// Byte is used as an sentitel between messages
    Byte(u8),
}

impl Default for Separator {
    fn default() -> Self {
        Separator::Byte(b'\n')
    }
}

/// Stream codec for streaming protocols (ipc, tcp)
#[derive(Debug, Default)]
pub struct StreamCodec {
    incoming_separator: Separator,
    outgoing_separator: Separator,
}

impl StreamCodec {
    /// Default codec with streaming input data. Input can be both enveloped and
    /// not.
    pub fn stream_incoming() -> Self {
        StreamCodec::new(Separator::Empty, Default::default())
    }

    /// New custom stream codec
    pub fn new(incoming_separator: Separator, outgoing_separator: Separator) -> Self {
        StreamCodec {
            incoming_separator,
            outgoing_separator,
        }
    }
}

fn is_whitespace(byte: u8) -> bool {
    matches!(byte, 0x0D | 0x0A | 0x20 | 0x09)
}

impl Decoder for StreamCodec {
    type Error = io::Error;
    type Item = BytesMut;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Self::Item>> {
        if let Separator::Byte(separator) = self.incoming_separator {
            if let Some(i) = buf.as_ref().iter().position(|&b| b == separator) {
                let line = buf.split_to(i);
                let _ = buf.split_to(1);

                match str::from_utf8(&line.as_ref()) {
                    Ok(_) => Ok(Some(line)),
                    Err(_) => Err(io::Error::new(io::ErrorKind::Other, "invalid UTF-8")),
                }
            } else {
                Ok(None)
            }
        } else {
            let mut depth = 0;
            let mut in_str = false;
            let mut is_escaped = false;
            let mut start_idx = 0;
            let mut whitespaces = 0;

            for idx in 0..buf.as_ref().len() {
                let byte = buf.as_ref()[idx];

                if (byte == b'{' || byte == b'[') && !in_str {
                    if depth == 0 {
                        start_idx = idx;
                    }
                    depth += 1;
                } else if (byte == b'}' || byte == b']') && !in_str {
                    depth -= 1;
                } else if byte == b'"' && !is_escaped {
                    in_str = !in_str;
                } else if is_whitespace(byte) {
                    whitespaces += 1;
                }
                if byte == b'\\' && !is_escaped && in_str {
                    is_escaped = true;
                } else {
                    is_escaped = false;
                }

                if depth == 0 && idx != start_idx && idx - start_idx + 1 > whitespaces {
                    let bts = buf.split_to(idx + 1);
                    match str::from_utf8(bts.as_ref()) {
                        Ok(_) => return Ok(Some(bts)),
                        Err(_) => {
                            return Ok(None);
                        } // skip non-utf requests (TODO: log error?)
                    };
                }
            }
            Ok(None)
        }
    }
}

impl tokio_util::codec::Encoder<String> for StreamCodec {
    type Error = io::Error;

    fn encode(&mut self, msg: String, buf: &mut BytesMut) -> io::Result<()> {
        let mut payload = msg.into_bytes();
        if let Separator::Byte(separator) = self.outgoing_separator {
            payload.push(separator);
        }
        buf.extend_from_slice(&payload);
        Ok(())
    }
}
