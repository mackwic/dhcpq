use errors::Error;
use nom;

pub mod dhcp_message {
    #[derive(Debug, PartialEq, Eq)]
    pub enum Type {
        BootRequest,
        BootReply,
    }
}

pub struct Parser {}

impl Parser {
    pub fn parse(buffer: &[u8]) -> Result<(), Error> {
        if buffer.len() == 0 { return Err(Error::BadInputDatagram) }
        Ok(())
    }

    pub fn parse_message_type(buffer : u8) -> Result<dhcp_message::Type, Error> {
        if buffer == 1 {
            Ok(dhcp_message::Type::BootRequest)
        } else {
            Ok(dhcp_message::Type::BootReply)
        }   
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_return_error_on_empty_buffer() {
        let buf : [u8; 0] = [0; 0];
        assert!(Parser::parse(&buf).is_err());
    }

    #[test]
    fn parser_data() {
        let _data : [u8; 236] = [
            1, // BOOTREQUEST                       // op
            1, // ETHERNET 10Mb/s                   // htype
            6, // Hardware address length           // hlen
            0, // is it a bootp relay ?             // hops
            0, 0, 0, 1, // xid transaction ID
            0, 0, // second elapsed since client began process
            0b1000_0000, 0, // flags with broadcast set
            0, 0, 0, 0,   // ciaddr empty because client has no addres
            127, 0, 0, 1, // yiaddr our adress
            0, 0, 0, 0,   // siaddr not used because client msg
            0, 0, 0, 0,   // giaddr no relay so 0
            0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, // chaddr client hardware address
            // sname optional server hostname 64B
            0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0,
            0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0,
            0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0,
            0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0,
            // file boot file name 128B
            0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0,
            0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0,
            0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0,
            0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0,
            0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0,
            0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0,
            0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0,
            0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0
        ];
    }

    #[test]
    fn parse_message_type_boot_request() {
        assert_eq!(dhcp_message::Type::BootRequest, Parser::parse_message_type(1u8).unwrap());
    }
    
    #[test]
    fn parse_message_type_boot_reply() {
        assert_eq!(dhcp_message::Type::BootReply, Parser::parse_message_type(2u8).unwrap());
    }
}
