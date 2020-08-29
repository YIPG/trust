use std::io;

pub enum State {
    Closed,
    Listen,
    // SynRcvd,
    // Estab,
}

impl Default for State {
    fn default() -> Self {
        State::Listen
    }
}

impl State {
    pub fn on_packet<'a>(
        &mut self,
        nic: &mut tun_tap::Iface,
        iph: etherparse::Ipv4HeaderSlice<'a>,
        tcph: etherparse::TcpHeaderSlice<'a>,
        data: &'a [u8],
    ) -> io::Result<usize> {
        let mut buf = [0u8; 1500];
        match *self {
            State::Closed => {
                return Ok(0);
            }
            State::Listen => {
                if !tcph.syn() {
                    // not expect other than syn packet
                    return Ok(0);
                }
                let mut syn_ack = etherparse::TcpHeader::new(
                    tcph.destination_port(),
                    tcph.source_port(),
                    unimplemented!(),
                    unimplemented!(),
                );
                syn_ack.syn = true;
                syn_ack.ack = true;
                let mut ip = etherparse::Ipv4Header::new(
                    syn_ack.header_len(),
                    64,
                    etherparse::IpTrafficClass::Tcp,
                    [
                        iph.destination()[0],
                        iph.destination()[1],
                        iph.destination()[2],
                        iph.destination()[3],
                    ],
                    [
                        iph.source()[0],
                        iph.source()[1],
                        iph.source()[2],
                        iph.source()[3],
                    ],
                );

                let unwritten = {
                    let mut unwriten = &mut buf[..];
                    ip.write(&mut unwriten);
                    syn_ack.write(&mut unwriten);
                    unwriten.len()
                };
                nic.send(&buf[..unwritten])
            }
        }
    }
}
