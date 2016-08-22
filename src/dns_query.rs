use trust_dns::rr::domain::Name;
use trust_dns::rr::dns_class::DNSClass;
use trust_dns::rr::record_type::RecordType;
use trust_dns::op::{MessageType, OpCode, Query};
use trust_dns::serialize::binary::{BinEncoder, BinDecoder, BinSerializable};

pub use trust_dns::op::Message;

pub fn a_query(host: &str) -> Query {
    let mut query = Query::new();

    let root = Name::root();
    let name = Name::parse(host, Some(&root)).unwrap();    
    query.name(name).query_class(DNSClass::IN).query_type(RecordType::A);
    query
}

pub fn any_query(host: &str) -> Query {
    let mut query = Query::new();

    let root = Name::root();
    let name = Name::parse(host, Some(&root)).unwrap();    
    query.name(name).query_class(DNSClass::IN).query_type(RecordType::A);
    query
}

pub fn build_query_message(query: Query) -> Message {
    let mut msg: Message = Message::new();
    msg.message_type(MessageType::Query).op_code(OpCode::Query).recursion_desired(true);
    msg.add_query(query);
    msg    
}

pub fn encode_message(buf: &mut Vec<u8>, msg: &Message) {
    let mut encoder = BinEncoder::new(buf);
    msg.emit(&mut encoder).unwrap();
}

pub fn decode_message(buf: &mut [u8]) -> Message {
    let mut decoder = BinDecoder::new(&buf);
    Message::read(&mut decoder).unwrap()
}
