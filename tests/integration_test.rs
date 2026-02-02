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
}
