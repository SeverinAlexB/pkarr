use std::io::Cursor;
use domain::{base::Rtype, zonefile::inplace::{Entry, Zonefile}};
use anyhow::anyhow;
use pkarr::dns::{Name, Packet, ResourceRecord};


#[derive(Debug)]
pub struct PkarrZone {
    pub paket_bytes: Vec<u8>
}

impl PkarrZone {
    pub fn read(simplified_zone: String, pubkey: &str) -> Result<Self, anyhow::Error> {
        let entries = Self::parse_simplified_zone(simplified_zone, pubkey)?;
        let packet = Self::entries_to_simple_dns_packet(entries)?;
        Ok(Self{
            paket_bytes: packet
        })
    }

    pub fn packet(&self) -> Packet {
        Packet::parse(&self.paket_bytes).unwrap()
    }

    /**
     * Generate a fake soa entry to simplify the
     * zone file the user needs to write.
     */
    fn generate_soa(pubkey: &str) -> String {
        let formatted = format!("$ORIGIN {pubkey}. 
$TTL 86400 
@	IN	SOA	127.0.0.1.	hostmaster.example.com. (
			2001062501 ; serial                     
			21600      ; refresh after 6 hours                     
			3600       ; retry after 1 hour                     
			604800     ; expire after 1 week                     
			86400 )    ; minimum TTL of 1 day  
            ");
        formatted
    }

    fn parse_simplified_zone(simplified_zone: String, pubkey: &str) -> Result<Vec<Entry>, anyhow::Error> {
        let raw_soa = PkarrZone::generate_soa(pubkey);
        let zone = format!("{raw_soa}\n{simplified_zone}\n");

        let byte_data = zone.into_bytes();
        let mut cursor = Cursor::new(byte_data);
        let zone = Zonefile::load(&mut cursor)?;

        let mut entries: Vec<Entry> = vec![];
        for entry_res in zone.into_iter() {
            let entry = entry_res?;

            let should_include: bool = match entry.clone() {
                Entry::Record(val) => {
                    val.rtype() != Rtype::SOA
                },
                _ => false
            };
            if should_include {
                entries.push(entry);
            }
        };
        Ok(entries)
    }

    fn entries_to_simple_dns_packet(entries: Vec<Entry>) -> Result<Vec<u8>, anyhow::Error> {
        let mut packets = vec![];
        for entry in entries.iter() {
            let entry = entry.clone();
            let packet = match entry {
                Entry::Include { path, origin } => continue,
                Entry::Record(val) => {
                    let (name, data) = val.clone().into_owner_and_data();
                    let simple_name_str = name.to_string();
                    let simple_name = Name::try_from(simple_name_str.as_str())?;
                    let simple_data = match data {
                        domain::rdata::ZoneRecordData::A(val) => {
                            let rdata: pkarr::dns::rdata::RData = pkarr::dns::rdata::RData::A(
                                pkarr::dns::rdata::A{
                                    address: val.addr().into()
                                }
                            );
                            let rr = ResourceRecord::new(simple_name, pkarr::dns::CLASS::IN, 60*60, rdata);
                            let mut packet = pkarr::dns::Packet::new_reply(0);
                            packet.answers.push(rr);
                            packet.build_bytes_vec_compressed()?
                        },
                        domain::rdata::ZoneRecordData::Aaaa(val) => {
                            let rdata: pkarr::dns::rdata::RData = pkarr::dns::rdata::RData::AAAA(
                                pkarr::dns::rdata::AAAA{
                                    address: val.addr().into()
                                }
                            );
                            let rr = ResourceRecord::new(simple_name, pkarr::dns::CLASS::IN, 60*60, rdata);
                            let mut packet = pkarr::dns::Packet::new_reply(0);
                            packet.answers.push(rr);
                            packet.build_bytes_vec_compressed()?
                        },
                        domain::rdata::ZoneRecordData::Ns(val) => {
                            let ns_name = val.to_string();
                            let rdata: pkarr::dns::rdata::RData = pkarr::dns::rdata::RData::NS(
                                pkarr::dns::rdata::NS(Name::try_from(ns_name.as_str())?)
                            );
                            
                            let rr = ResourceRecord::new(simple_name, pkarr::dns::CLASS::IN, 60*60, rdata);
                            let mut packet = pkarr::dns::Packet::new_reply(0);
                            packet.answers.push(rr);
                            packet.build_bytes_vec_compressed()?
                        },

                        domain::rdata::ZoneRecordData::Txt(val) => {
                            let value = val.to_string();
                            let mut txt = pkarr::dns::rdata::TXT::new();
                            txt.add_string(value.as_str())?;
                            let rdata: pkarr::dns::rdata::RData = pkarr::dns::rdata::RData::TXT(
                                txt
                            );
                            
                            let rr = ResourceRecord::new(simple_name, pkarr::dns::CLASS::IN, 60*60, rdata);
                            let mut packet = pkarr::dns::Packet::new_reply(0);
                            packet.answers.push(rr);
                            packet.build_bytes_vec_compressed()?
                        },
                        domain::rdata::ZoneRecordData::Mx(val) => {
                            let exchange = val.exchange().to_string();
                            let mut mx = pkarr::dns::rdata::MX{
                                preference: val.preference(),
                                exchange: Name::try_from(exchange.as_str())?,
                            };
      
                            let rdata: pkarr::dns::rdata::RData = pkarr::dns::rdata::RData::MX(
                                mx
                            );
                            
                            let rr = ResourceRecord::new(simple_name, pkarr::dns::CLASS::IN, 60*60, rdata);
                            let mut packet = pkarr::dns::Packet::new_reply(0);
                            packet.answers.push(rr);
                            packet.build_bytes_vec_compressed()?
                        },
                        _ => return Err(anyhow!("Not support record type."))
                    };
                    simple_data
                }
            };
            packets.push(packet);
            
        };
        let mut final_packet = Packet::new_reply(0);
        for packet in packets.iter() {
            let parsed = Packet::parse(packet)?;
            for answer in parsed.answers {
                final_packet.answers.push(answer)
            }
        }
        Ok(final_packet.build_bytes_vec_compressed()?)
    }
}





#[cfg(test)]
mod tests {
    
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    fn simplified_zone() -> String {
        String::from(
            "           
@	IN	NS	dns1.example.com.       
@	IN	NS	dns2.example.com.        
	
	
@	IN	MX	10	mail.example.com.       
@	IN	MX	20	mail2.example.com.   

@   IN  A 127.0.0.1
test    IN  A 127.0.0.1

	
dns1	IN	A	10.0.1.1
dns2	IN	A	10.0.1.2

yolo    IN  TXT  testsev
",
        )
    }

    #[test]
    fn test_create_entries() {
        let simplified_zone = simplified_zone();
        let zone = PkarrZone::read(simplified_zone, "123456");
        let zone = zone.unwrap();
        assert_eq!(zone.packet().answers.len(), 9);

        println!("{zone:#?}");
    }

    #[test]
    fn test_transform() {
        let simplified_zone = simplified_zone();
        let zone = PkarrZone::read(simplified_zone, "123456").unwrap();
        let packet = zone.packet();

        println!("{:#?}", packet.answers);
    }
}
