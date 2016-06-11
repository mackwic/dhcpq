use std::io;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Ipv6Unsupported {
            description("IPv6 addresses are not supported")
        }
        InnerIo(err: io::Error) {
            from()
        }
    }
}

