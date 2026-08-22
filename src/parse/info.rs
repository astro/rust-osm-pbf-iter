use std::convert::Into;

use protobuf_iter::*;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Info<'a> {
    pub version: Option<u32>,
    /// Milliseconds since 1970-01-01 UTC, already scaled by the
    /// `PrimitiveBlock`'s `date_granularity`.
    pub timestamp: Option<u64>,
    pub changeset: Option<u64>,
    pub uid: Option<u32>,
    pub user: Option<&'a str>,
    pub visible: Option<bool>,
}

impl<'a> Info<'a> {
    /// `date_granularity` is `PrimitiveBlock.date_granularity` (default
    /// 1000, i.e. milliseconds) -- the wire value for `timestamp` is a
    /// *count of `date_granularity`-sized units* since the epoch, not
    /// directly a millisecond (or second) value; the real millisecond
    /// timestamp is `wire_value * date_granularity`. Found via
    /// `node/687015806`, a real, otherwise perfectly ordinary OSM node,
    /// in Geofabrik's Switzerland extract
    /// (<https://download.geofabrik.de/europe/switzerland-latest.osm.pbf>),
    /// downloaded 2026-08-22.
    pub fn parse(stringtable: &'a [&'a str], date_granularity: u64, data: &'a [u8]) -> Self {
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
                2 => {
                    let timestamp: u64 = m.value.into();
                    info.timestamp = Some(date_granularity * timestamp);
                }
                3 => info.changeset = Some(m.value.into()),
                4 => info.uid = Some(m.value.into()),
                5 => {
                    let user_sid: u32 = m.value.into();
                    info.user = Some(stringtable[user_sid as usize]);
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
