use trust_dns::udp::UdpClientConnection;
use trust_dns::client::{ClientConnection};
use trust_dns::rr::domain::Name;
use trust_dns::rr::dns_class::DNSClass;
use trust_dns::rr::record_type::RecordType;
use trust_dns::op::{Message, MessageType, OpCode, Query};
use trust_dns::serialize::binary::{BinEncoder, BinDecoder, BinSerializable};




pub fn build_query(id: u16, host: &str) -> Vec<u8> {
    let root = Name::root();
    let name = Name::parse(host,Some(&root)).unwrap();

    let mut msg: Message = Message::new();
    msg.id(id).message_type(MessageType::Query).op_code(OpCode::Query).recursion_desired(true);
    let mut query = Query::new();
    query.name(name.clone()).query_class(DNSClass::IN).query_type(RecordType::A);
    msg.add_query(query);

    let mut buffer: Vec<u8> = Vec::with_capacity(512);
    {
      let mut encoder = BinEncoder::new(&mut buffer);
      msg.emit(&mut encoder).unwrap();
    }
    buffer
}

pub fn parse_response(buf: &mut [u8]) -> Message {
    let mut decoder = BinDecoder::new(&buf);
    Message::read(&mut decoder).unwrap()
}

pub fn query(udp: &mut UdpClientConnection, host: &str) -> Message {
    let buffer = build_query(1234, host);
    println!("message: {:?}", buffer);
    let resp_buffer = udp.send(buffer).unwrap();
    let mut decoder = BinDecoder::new(&resp_buffer);
    Message::read(&mut decoder).unwrap()
}
