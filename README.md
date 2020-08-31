# Bumpalo-herd

Trying to make bump allocator that is `Sync`, by sharding multiple
[`bumpalo::Bump`](https://docs.rs/bumpalo/*/struct.Bump.html) instances.

Unfortunately, this seems to be slow.
