# Release notes

## Hash compatibility

v2 hashes are incompatible with v1.x because entry encoding now includes
a NUL byte after the relative path and an entry-type byte. Regenerate stored
hashes when upgrading.

## v2 library API migration

`hash_source` now returns `Result<ArrayString<64>, Error>` and replaces both
previous hashing functions. Replace calls to `try_hash_source` with `hash_source`.
For callers of the previous infallible `hash_source`, use `?` to propagate errors
or `.expect(...)` to panic on failure.

Both migrations require a third `follow_links` argument. Pass `false` to hash
links by their target-path text, or `true` to hash symbolic-link targets and
traverse linked directories. When following links, broken links and cycles
return an error.

The option also applies to the source path itself. Previously, a root directory
symlink was traversed even without following links. In v2, use `follow_links =
true` (or `--follow` on the CLI) to traverse it; otherwise only the link is hashed.
