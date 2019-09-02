use crate::ipld;
use crate::ipld::*;
use crate::untyped::Ipld;
use cid::Cid;
use protobuf::Message;
use super::gen;
use std::convert::{TryFrom, TryInto};

#[derive(Clone)]
pub struct PbLink {
    pub cid: Cid,
    pub name: String,
    pub size: u64,
}

impl Into<Ipld> for PbLink {
    fn into(self) -> Ipld {
        ipld!({
            "Hash": self.cid,
            "Name": self.name,
            "Tsize": self.size,
        })
    }
}

fn from_ipld(ipld: Ipld) -> Option<PbLink> {
    match ipld {
        Ipld::Map(IpldMap(mut map)) => {
            let cid: Option<Cid> = map
                .remove("Hash")
                .map(|t| TryInto::try_into(t).ok())
                .unwrap_or_default();
            let name: Option<String> = map
                .remove("Name")
                .map(|t| TryInto::try_into(t).ok())
                .unwrap_or_default();
            let size: Option<u64> = map
                .remove("Tsize")
                .map(|t| TryInto::try_into(t).ok())
                .unwrap_or_default();
            if cid.is_some() && name.is_some() && size.is_some() {
                return Some(PbLink {
                    cid: cid.unwrap(),
                    name: name.unwrap(),
                    size: size.unwrap(),
                })
            }
        }
        _ => {}
    }
    None
}

#[derive(Clone, Default)]
pub struct PbNode {
    pub links: Vec<PbLink>,
    pub data: Vec<u8>,
}

impl Into<Ipld> for PbNode {
    fn into(self) -> Ipld {
        let links: Vec<Ipld> = self.links.into_iter().map(Into::into).collect();
        ipld!({
            "Links": links,
            "Data": self.data,
        })
    }
}

impl From<&Ipld> for PbNode {
    fn from(ipld: &Ipld) -> Self {
        match ipld {
            Ipld::Map(IpldMap(map)) => {
                let links: Vec<Ipld> = map
                    .get("Links")
                    .cloned()
                    .map(|t| TryInto::try_into(t).ok())
                    .unwrap_or_default()
                    .unwrap_or_default();
                let links: Vec<PbLink> = links
                    .into_iter()
                    .filter_map(from_ipld)
                    .collect();
                let data: Vec<u8> = map
                    .get("Data")
                    .cloned()
                    .map(|t| TryInto::try_into(t).ok())
                    .unwrap_or_default()
                    .unwrap_or_default();
                PbNode {
                    links,
                    data,
                }
            }
            _ => Default::default()
        }
    }
}

impl PbNode {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, failure::Error> {
        let proto: gen::PBNode = protobuf::parse_from_bytes(bytes)?;
        let data = proto.get_Data().to_vec();
        let mut links = Vec::new();
        for link in proto.get_Links() {
            let cid = Cid::try_from(link.get_Hash())?.into();
            let name = link.get_Name().to_string();
            let size = link.get_Tsize();
            links.push(PbLink {
                cid,
                name,
                size,
            });
        }
        Ok(PbNode {
            links,
            data,
        })
    }

    pub fn into_bytes(self) -> Vec<u8> {
        let mut proto = gen::PBNode::new();
        proto.set_Data(self.data);
        for link in self.links {
            let mut pb_link = gen::PBLink::new();
            pb_link.set_Hash(link.cid.to_bytes());
            pb_link.set_Name(link.name);
            pb_link.set_Tsize(link.size);
            proto.mut_Links().push(pb_link);
        }
        proto
            .write_to_bytes()
            .expect("there is no situation in which the protobuf message can be invalid")
    }
}