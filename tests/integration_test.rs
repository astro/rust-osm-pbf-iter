#[cfg(test)]
mod tests {
    use chrono::{DateTime, SecondsFormat, TimeZone, Utc};
    use osm_pbf_iter::{BlobReader, Primitive, PrimitiveBlock, RelationMemberType, info::Info};
    use std::fs::{File, read_to_string};
    use std::io::{BufReader, Read};
    use std::path::PathBuf;

    #[test]
    fn test_64bit_ids() {
        assert_eq!(
            dump(new_blob_reader("64bit_ids.osm.pbf")),
            read_to_string(test_data_path("64bit_ids.xml")).unwrap()
        );
    }

    #[test]
    fn test_multipolygon() {
        assert_eq!(
            dump(new_blob_reader("multipolygon.osm.pbf")),
            read_to_string(test_data_path("multipolygon.xml")).unwrap()
        );
    }

    #[test]
    fn test_tag_lengths() {
        assert_eq!(
            dump(new_blob_reader("tag_lengths.osm.pbf")),
            read_to_string(test_data_path("tag_lengths.xml")).unwrap()
        );
    }

    #[test]
    fn test_two_primitive_groups() {
        // The test file contains a single uncompressed OSMData blob
        // with two PrimitiveGroups. The first group consists of node 11,
        // the second group consists of nodes 21 and 22.
        //
        // TODO: At the moment, the implementation fails to process
        // the second PrimitiveGroup, so we get only the single node 11
        // in the result for this test. This is wrong, and we should fix
        // the implementation to find the primitives in all PrimitiveGroups.
        //
        // https://github.com/astro/rust-osm-pbf-iter/issues/10
        assert_eq!(
            dump(new_blob_reader("two_primitive_groups.osm.pbf")),
            read_to_string(test_data_path("two_primitive_groups.xml")).unwrap()
        );
    }

    /// Regression test: a `PrimitiveBlock` with a non-default
    /// `date_granularity` (60000ms, i.e. one minute -- real-world files
    /// essentially always use the default of 1000ms/one second, which
    /// is exactly why this bug went unnoticed: `Info::parse` used to
    /// return `Info.timestamp` as the raw, un-scaled wire value, which
    /// happens to equal true seconds-since-epoch when
    /// `date_granularity` is 1000 -- silently wrong for any other
    /// granularity, no error anywhere to flag it). Built by hand
    /// against the OSM PBF protobuf schema, not via `osmium` like the
    /// other fixtures here -- `osmium`'s PBF writer has no option to
    /// set `date_granularity` to anything but the default, since no
    /// real-world producer needs to. Contains one `Way` (id 1) with
    /// `Info.timestamp` wire value `1` -- one `date_granularity` unit
    /// since the epoch, i.e. `1 * 60000ms = 60000ms` =
    /// `1970-01-01T00:01:00Z`. Independently confirmed against
    /// `osmium cat -f opl`, which reports the same timestamp. Before
    /// this crate's fix, `Info::parse` returned the raw wire value `1`
    /// unscaled, i.e. `1970-01-01T00:00:01Z` -- one minute off.
    #[test]
    fn test_non_default_date_granularity() {
        let mut reader = new_blob_reader("non_default_date_granularity.osm.pbf");
        let blob = reader.find(|_| true).expect("one OSMData blob");
        let data = blob.into_data();
        let block = PrimitiveBlock::parse(&data);
        assert_eq!(block.date_granularity, 60_000);

        let ways: Vec<_> = block
            .primitives()
            .filter_map(|p| match p {
                Primitive::Way(way) => Some(way),
                _ => None,
            })
            .collect();
        assert_eq!(ways.len(), 1);
        assert_eq!(
            ways[0].info.as_ref().and_then(|i| i.timestamp),
            Some(60_000)
        );
    }

    fn new_blob_reader(filename: &str) -> BlobReader<BufReader<File>> {
        let path = test_data_path(filename);
        let file = File::open(&path).expect(&format!("cannot open {:?}", path));
        BlobReader::new(BufReader::new(file))
    }

    fn test_data_path(filename: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests");
        path.push("data");
        path.push(filename);
        path
    }

    fn dump<R: Read>(reader: BlobReader<R>) -> String {
        let mut buf = String::new();
        buf.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        buf.push_str("<osm generator=\"integration_test\" upload=\"false\">\n");
        for blob in reader {
            let data = blob.into_data();
            let primitive_block = PrimitiveBlock::parse(&data);
            for primitive in primitive_block.primitives() {
                let element: &str;
                let mut attrs = Vec::<String>::new();
                let mut nodes = Vec::<i64>::new();
                let mut members = Vec::<(String, RelationMemberType, u64)>::new();
                let mut tags = Vec::<(String, String)>::new();
                match primitive {
                    Primitive::Node(node) => {
                        element = "node";
                        attrs.push(format!("id=\"{}\"", node.id));
                        dump_info(node.info, &mut attrs);
                        attrs.push(format!("lat=\"{:.7?}\"", node.lat));
                        attrs.push(format!("lon=\"{:.7?}\"", node.lon));
                        tags.extend(
                            node.tags
                                .iter()
                                .map(|(k, v)| (escape_xml(k), escape_xml(v))),
                        );
                    }
                    Primitive::Way(way) => {
                        element = "way";
                        attrs.push(format!("id=\"{}\"", way.id));
                        nodes.extend(way.refs());
                        tags.extend(way.tags().map(|(k, v)| (escape_xml(k), escape_xml(v))));
                        dump_info(way.info, &mut attrs);
                    }
                    Primitive::Relation(rel) => {
                        element = "relation";
                        attrs.push(format!("id=\"{}\"", rel.id));
                        members
                            .extend(rel.members().map(|(role, id, member_type)| {
                                (escape_xml(role), member_type, id)
                            }));
                        tags.extend(rel.tags().map(|(k, v)| (escape_xml(k), escape_xml(v))));
                    }
                };
                buf.push_str("  <");
                buf.push_str(element);
                let has_children = !nodes.is_empty() || !members.is_empty() || !tags.is_empty();
                if !attrs.is_empty() {
                    buf.push(' ');
                    buf.push_str(&attrs.join(" "));
                    if !has_children {
                        buf.push('/');
                    }
                    buf.push_str(">\n");
                }
                if has_children {
                    for n in nodes {
                        buf.push_str(&format!("    <nd ref=\"{}\"/>\n", n));
                    }
                    for (role, member_type, id) in members {
                        buf.push_str(&format!(
                            "    <member type=\"{}\" ref=\"{}\" role=\"{}\"/>\n",
                            relation_member_type(member_type),
                            id,
                            role
                        ));
                    }
                    for (k, v) in tags {
                        buf.push_str(&format!("    <tag k=\"{}\" v=\"{}\"/>\n", k, v));
                    }
                    buf.push_str("  </");
                    buf.push_str(element);
                    buf.push_str(">\n");
                }
            }
        }
        buf.push_str("</osm>\n");
        buf
    }

    fn relation_member_type(t: RelationMemberType) -> &'static str {
        match t {
            RelationMemberType::Node => "node",
            RelationMemberType::Way => "way",
            RelationMemberType::Relation => "relation",
        }
    }

    fn dump_info(info: Option<Info>, attrs: &mut Vec<String>) {
        let Some(info) = info else {
            return;
        };
        if let Some(version) = info.version {
            attrs.push(format!("version=\"{}\"", version));
        }
        if let Some(timestamp) = info.timestamp {
            let seconds: i64 = (timestamp / 1000).try_into().unwrap();
            let datetime: DateTime<Utc> = Utc.timestamp_opt(seconds, 0).unwrap();
            let timestamp_str =
                datetime.to_rfc3339_opts(SecondsFormat::Secs, /* use_z */ true);
            attrs.push(format!("timestamp=\"{}\"", timestamp_str));
        }
        if let Some(changeset) = info.changeset {
            attrs.push(format!("changeset=\"{}\"", changeset));
        }
        if let Some(uid) = info.uid {
            attrs.push(format!("uid=\"{}\"", uid));
        }
        if let Some(user) = info.user {
            attrs.push(format!("user=\"{}\"", escape_xml(user)));
        }
        if let Some(visible) = info.visible {
            attrs.push(format!("visible=\"{}\"", visible));
        }
    }

    fn escape_xml(input: &str) -> String {
        let mut output = String::with_capacity(input.len());
        for c in input.chars() {
            match c {
                '<' => output.push_str("&lt;"),
                '>' => output.push_str("&gt;"),
                '&' => output.push_str("&amp;"),
                '"' => output.push_str("&quot;"),
                _ => output.push(c),
            }
        }
        output
    }

    #[test]
    #[ignore = "run by hand to regenerate tests/data/*.xml golden files after a parser change"]
    fn regenerate_golden_files() {
        for name in [
            "multipolygon",
            "64bit_ids",
            "tag_lengths",
            "two_primitive_groups",
        ] {
            let xml = dump(new_blob_reader(&format!("{name}.osm.pbf")));
            std::fs::write(test_data_path(&format!("{name}.xml")), xml).unwrap();
        }
    }
}
