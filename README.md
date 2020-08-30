streamvbyte
===========

Rust FFI bindings to streamvbyte: https://github.com/lemire/streamvbyte

# Examples

Encode an u32 slice into a new buf in with `encode`:

```
use streamvbyte::encode;
let out_bytes: Vec<u8> = encode(&[1,2,44,5123,43,534]);
```

...or by using the `encode_to_buf` function into an existing buffer:

```
use streamvbyte::{max_compressedbytes,encode_to_buf};
let input = vec![1,2,44,5123,43,534];
let max_bytes = max_compressedbytes(input.len());
let mut out_buf = vec![0;max_bytes];
let bytes_written = encode_to_buf(&input,&mut out_buf);
assert_eq!(bytes_written.unwrap(),10);
```

You can use `encode_delta` to encode increasing sequences more effectively:

```
use streamvbyte::encode_delta;
let out_bytes: Vec<u8> = encode_delta(&[1,2,44,64,71,534],0);
```

Decoding values works in much the same way:

```
use streamvbyte::{decode_delta,encode_delta};
let out_bytes: Vec<u8> = encode_delta(&[1,2,44,64,71,534],0);
let mut recovered = vec![0;6];
let bytes_read = decode_delta(&out_bytes,&mut recovered,0);
assert_eq!(out_bytes.len(),bytes_read);
assert_eq!(&recovered,&[1,2,44,64,71,534]);
```
