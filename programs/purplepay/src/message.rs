use anchor_lang::{ prelude::Pubkey, AnchorDeserialize, AnchorSerialize };
use std::io;

const PAYLOAD_ID_ALIVE: u8 = 0;
const PAYLOAD_ID_HELLO: u8 = 1;

pub const HELLO_MESSAGE_MAX_LENGTH: usize = 512;

#[derive(Clone)]
/// Expected message types for this program. Only valid payloads are:
/// * `Alive`: Payload ID == 0. Emitted when [`initialize`](crate::initialize)
///  is called).
/// * `Hello`: Payload ID == 1. Emitted when
/// [`send_message`](crate::send_message) is called).
///
/// Payload IDs are encoded as u8.
pub enum HelloWorldMessage {
    Alive {
        program_id: Pubkey,
    },
    Hello {
        message: CrosschainID,
    },
}

impl AnchorSerialize for HelloWorldMessage {
    fn serialize<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            HelloWorldMessage::Alive { program_id } => {
                PAYLOAD_ID_ALIVE.serialize(writer)?;
                program_id.serialize(writer)
            }
            HelloWorldMessage::Hello { message } => {
                if message.len() > HELLO_MESSAGE_MAX_LENGTH {
                    Err(
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            format!("message exceeds {HELLO_MESSAGE_MAX_LENGTH} bytes")
                        )
                    )
                } else {

                    //TODO : Edit, what and how ?
                    PAYLOAD_ID_HELLO.serialize(writer)?;
                    (message.len() as u16).to_be_bytes().serialize(writer)?;
                    for item in message {
                        item.serialize(writer)?;
                    }
                    Ok(())
                }
            }
        }
    }
}

impl AnchorDeserialize for HelloWorldMessage {
    fn deserialize(buf: &mut &[u8]) -> io::Result<Self> {
        match buf[0] {
            PAYLOAD_ID_ALIVE =>
                Ok(HelloWorldMessage::Alive {
                    program_id: Pubkey::try_from(&buf[1..33]).unwrap(),
                }),
            PAYLOAD_ID_HELLO => {

                //TODO : Edit what and how?
                let length = {
                    let mut out = [0u8; 2];
                    out.copy_from_slice(&buf[1..3]);
                    u16::from_be_bytes(out) as usize
                };
                if length > HELLO_MESSAGE_MAX_LENGTH {
                    Err(
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            format!("message exceeds {HELLO_MESSAGE_MAX_LENGTH} bytes")
                        )
                    )
                } else {
                    Ok(HelloWorldMessage::Hello {
                        message: buf[3..3 + length].to_vec(),
                    })
                }
            }
            _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid payload ID")),
        }
    }
}
