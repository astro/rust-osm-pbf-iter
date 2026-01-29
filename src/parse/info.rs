use std::convert::Into;

use protobuf_iter::*;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Info<'a> {
    pub version: Option<u32>,
    pub timestamp: Option<u64>,
    pub changeset: Option<u64>,
    pub uid: Option<u32>,
    pub user: Option<&'a str>,
    pub visible: Option<bool>,
}

impl<'a> Info<'a> {
    pub fn parse(stringtable: &'a [&'a str], data: &'a [u8]) -> Self {
        let mut info = Info {
            version: None,
            timestamp: None,
            changeset: None,
            uid: None,
            user: None,
            visible: None,
        };

        let iter = MessageIter::new(data);
        for m in iter {
            match m.tag {
                1 => info.version = Some(m.value.into()),
                2 => info.timestamp = Some(m.value.into()),
                3 => info.changeset = Some(m.value.into()),
                4 => info.uid = Some(m.value.into()),
                5 => {
                    let user_sid: u32 = m.value.into();
                    info.user = Some(&stringtable[user_sid as usize]);
                }
                6 => {
                    let visible: u32 = m.value.into();
                    info.visible = Some(visible != 0);
                }
                _ => (),
            }
        }

        info
    }
}
