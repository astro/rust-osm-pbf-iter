# Test data

Test cases taken from [osm-testdata](https://github.com/osmcode/osm-testdata),
converted with `osmium cat` from XML to PBF format.

The test case `multipolygon.osm.pbf` is much simplified from the
upstream test for multipolygons. Since we do not do any geometric
processing in this library, we don’t care about self-intersections and
similar corner cases.


## License

The files in this directory are in the public domain, just like the upstream
[osm-testdata](https://github.com/osmcode/osm-testdata) from which they
were automatically converted.
