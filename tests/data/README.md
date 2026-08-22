# Test data

Test cases taken from [osm-testdata](https://github.com/osmcode/osm-testdata),
converted with `osmium cat` from XML to PBF format.

The test case `multipolygon.osm.pbf` is much simplified from the
upstream test for multipolygons. Since we do not do any geometric
processing in this library, we don’t care about self-intersections and
similar corner cases.

`non_default_date_granularity.osm.pbf` is the one exception: it's
built by hand directly against the OSM PBF protobuf schema, not
converted from `osm-testdata` via `osmium cat` like the others.
`osmium`'s PBF writer has no option to set `PrimitiveBlock`'s
`date_granularity` to anything but the default (1000ms) -- real-world
files essentially always use that default, so no producer needs to
override it, but this crate's own `Info::parse` needs a fixture that
doesn't, to test that it's actually honored. See
`tests/integration_test.rs`'s `test_non_default_date_granularity` for
the full story.


## License

The files in this directory are in the public domain. Most are
converted from [osm-testdata](https://github.com/osmcode/osm-testdata),
just like their upstream source; `non_default_date_granularity.osm.pbf`
isn't derived from any upstream source, but is released into the
public domain the same way.
