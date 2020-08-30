//! A integer compression library which wraps the c [streamvbyte](https://github.com/lemire/streamvbyte) encoder/decoder
//!
//! Encodes 32 bit integers into variable length byte sequences in `O(n)` time.
//!
//! Input can be at most `u32::MAX` integers.
//!
//! # Examples
//!
//! Encode an u32 slice into a new buf in with [`encode`]:
//!
//! ```
//! use streamvbyte::encode;
//! let out_bytes: Vec<u8> = encode(&[1,2,44,5123,43,534]);
//! ```
//!
//! ...or by using the [`encode_to_buf`] function into an existing buffer:
//!
//! ```
//! use streamvbyte::{max_compressedbytes,encode_to_buf};
//! let input = vec![1,2,44,5123,43,534];
//! let max_bytes = max_compressedbytes(input.len());
//! let mut out_buf = vec![0;max_bytes];
//! let bytes_written = encode_to_buf(&input,&mut out_buf);
//! assert_eq!(bytes_written.unwrap(),10);
//! ```
//!
//! You can use [`encode_delta`] to encode increasing sequences more effectively:
//!
//! ```
//! use streamvbyte::encode_delta;
//! let out_bytes: Vec<u8> = encode_delta(&[1,2,44,64,71,534],0);
//! ```
//!
//! Decoding values works in much the same way:
//!
//! ```
//! use streamvbyte::{decode_delta,encode_delta};
//! let out_bytes: Vec<u8> = encode_delta(&[1,2,44,64,71,534],0);
//! let mut recovered = vec![0;6];
//! let bytes_read = decode_delta(&out_bytes,&mut recovered,0);
//! assert_eq!(out_bytes.len(),bytes_read);
//! assert_eq!(&recovered,&[1,2,44,64,71,534]);
//! ```
//! Note: **length** of the output buf `recovered` needs to match the input length. This information needs to be stored
//! external to compressed output.
//!

use thiserror::Error;

///! Errors that can be emitted from the streamvbyte crate and the underlying -sys crate
#[derive(Error, Debug)]
pub enum StreamVbyteError {
    /// Output buffer might overflow as it is not at least max_compressedbytes long
    #[error("insufficient output buffer len: is {0}, expected {1}")]
    OutbufOverflow(usize, usize),
}

///! Returns the maximum number of bytes required by the compressor to encode `length` u32s
pub fn max_compressedbytes(length: usize) -> usize {
    // number of control bytes:
    let cb = (length + 3) / 4;
    // maximum number of control bytes:
    let db = length * std::mem::size_of::<u32>();
    cb + db
}

/// Encode a sequence of u32 integers into a vbyte encoded byte representation.
/// Internally a buffer of length [`max_compressedbytes`] is allocated to store the compressed result.
///
/// The buffer will be truncated to the correct length before returning.
///
/// # Examples
///
/// ```
/// use streamvbyte::encode;
/// let out_bytes: Vec<u8> = encode(&[1,2,44,5123,43,534]);
/// ```
/// # Return
///
/// Returns the encoded output as a byte buffer
///
pub fn encode(input: &[u32]) -> Vec<u8> {
    let output_bytes_req = max_compressedbytes(input.len()) as usize;
    let mut buf = vec![0; output_bytes_req];
    // SAFETY: unwrap ok as we compute required max bytes beforehand
    let bytes_written = encode_to_buf(input, &mut buf).unwrap();
    buf.truncate(bytes_written);
    buf
}

/// Decode a sequence of u32 integers from a vbyte encoded byte representation into an existing buffer `output`.
///
/// # Arguments
///
/// * `input` - The input sequence of vbyte encoding (u8s)
/// * `output` - The output buf to store the recovered u32 integers. **MUST** be the same size as the original input sequence
///
/// # Examples
///
/// ```
/// use streamvbyte::{max_compressedbytes,encode,decode};
/// let input = vec![1,2,44,5123,43,534];
/// let out_buf = encode(&input);
/// let mut recovered = vec![0;6];
/// let bytes_read = decode(&out_buf,&mut recovered);
/// assert_eq!(bytes_read,out_buf.len());
/// ```
///
/// # Return
///
/// Returns the number of bytes processed from input during decoding
///
pub fn decode(input: &[u8], output: &mut [u32]) -> usize {
    unsafe {
        streamvbyte_sys::streamvbyte_decode(
            input.as_ptr(),
            output.as_mut_ptr(),
            output.len() as u32,
        ) as usize
    }
}

/// Encode a sequence of u32 integers into a vbyte encoded byte representation into an existing buffer `output`.
///
/// Required: output buf is at least [`max_compressedbytes`] long.
///
///
/// # Examples
///
/// ```
/// use streamvbyte::{max_compressedbytes,encode_to_buf};
/// let input = vec![1,2,44,5123,43,534];
/// let max_bytes = max_compressedbytes(input.len());
/// let mut out_buf = vec![0;max_bytes];
/// let bytes_written = encode_to_buf(&input,&mut out_buf);
/// assert_eq!(bytes_written.unwrap(),10);
/// ```
/// # Return
///
/// Returns the number of bytes written to output during encoding
///
pub fn encode_to_buf(input: &[u32], output: &mut [u8]) -> Result<usize, StreamVbyteError> {
    let output_bytes_req = max_compressedbytes(input.len());
    if output.len() < output_bytes_req {
        return Err(StreamVbyteError::OutbufOverflow(
            output.len(),
            output_bytes_req,
        ));
    }
    // SAFETY: output buf is as long as max compressed size
    unsafe {
        Ok(streamvbyte_sys::streamvbyte_encode(
            input.as_ptr(),
            input.len() as u32,
            output.as_mut_ptr(),
        ) as usize)
    }
}

/// Encode a sequence **non decreasing** of u32 integers into a vbyte encoded byte representation.
/// Internally a buffer of length [`max_compressedbytes`] is allocated to store the compressed result.
///
/// The buffer will be truncated to the correct length before returning.
///
/// # Arguments
///
/// * `input` - The input sequence of non decreasing u32 integers
/// * `initial` - The intial value to substract from the all the value in the array to decrease the universe. **MUST** be <= input\[0\]
///
/// # Examples
///
/// ```
/// use streamvbyte::encode_delta;
/// let out_bytes: Vec<u8> = encode_delta(&[1,2,44,5123,43,534],1);
/// ```
/// # Return
///
/// Returns the encoded output as a byte buffer
///
pub fn encode_delta(input: &[u32], intial: u32) -> Vec<u8> {
    let output_bytes_req = max_compressedbytes(input.len());
    let mut buf = vec![0; output_bytes_req];
    // SAFETY: unwrap ok as we compute required max bytes beforehand
    let bytes_written = encode_delta_to_buf(input, &mut buf, intial).unwrap();
    buf.truncate(bytes_written);
    buf
}

/// Encode a sequence **non decreasing** of u32 integers into a vbyte encoded byte representation into an existing buffer `output`.
///
/// # Arguments
///
/// * `input` - The input sequence of non decreasing u32 integers
/// * `output` - The output u8 slice of at least  [`max_compressedbytes`]
/// * `initial` - The intial value to substract from the all the value in the array to decrease the universe. **MUST** be <= input\[0\]
///
/// Required: output buf is at least [`max_compressedbytes`] long.
///
///
/// # Examples
///
/// ```
/// use streamvbyte::{max_compressedbytes,encode_delta_to_buf};
/// let input = vec![1,2,44,54,433,534];
/// let max_bytes = max_compressedbytes(input.len());
/// let mut out_buf = vec![0;max_bytes];
/// let bytes_written = encode_delta_to_buf(&input,&mut out_buf,0);
/// assert_eq!(bytes_written.unwrap(),9);
/// ```
/// # Return
///
/// Returns the number of bytes written to output during encoding
///
pub fn encode_delta_to_buf(
    input: &[u32],
    output: &mut [u8],
    initial: u32,
) -> Result<usize, StreamVbyteError> {
    let output_bytes_req = max_compressedbytes(input.len());
    if output.len() < output_bytes_req {
        return Err(StreamVbyteError::OutbufOverflow(
            output.len(),
            output_bytes_req,
        ));
    }
    unsafe {
        Ok(streamvbyte_sys::streamvbyte_delta_encode(
            input.as_ptr(),
            input.len() as u32,
            output.as_mut_ptr(),
            initial,
        ) as usize)
    }
}

/// Decode a sequence of non decreasing u32 integers from a vbyte encoded byte representation into an existing buffer `output`.
///
/// # Arguments
///
/// * `input` - The input sequence of vbyte encoding (u8s)
/// * `output` - The output buf to store the recovered non decreasing u32 integers. **MUST** be the same size as the original input sequence
/// * `initial` - The intial value thaw was substract from the all the value in the array during encoding.
///
/// # Examples
///
/// ```
/// use streamvbyte::{max_compressedbytes,encode_delta,decode_delta};
/// let input = vec![1,2,44,5123,43,534];
/// let out_buf = encode_delta(&input,1);
/// let mut recovered = vec![0;6];
/// let bytes_read = decode_delta(&out_buf,&mut recovered,1);
/// assert_eq!(bytes_read,out_buf.len());
/// ```
/// # Return
///
/// Returns the number of bytes processed from input during decoding
///
pub fn decode_delta(input: &[u8], output: &mut [u32], initial: u32) -> usize {
    unsafe {
        streamvbyte_sys::streamvbyte_delta_decode(
            input.as_ptr(),
            output.as_mut_ptr(),
            output.len() as u32,
            initial,
        ) as usize
    }
}

#[cfg(test)]
mod tests {

    fn create_input(bits: u32, len: usize) -> Vec<u32> {
        use rand::distributions::{Distribution, Uniform};
        let min = 0;
        let max: u64 = (1 << bits) - 1;
        let between = Uniform::from(min..=max);
        let mut rng = rand::thread_rng();
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(between.sample(&mut rng) as u32);
        }
        vec
    }

    #[test]
    fn encode_decode_roundtrip() {
        let len = 10000;
        for bits in 1..=32 {
            for _ in 0..2 {
                let input = create_input(bits, len);
                let output_buf = super::encode(&input);
                let mut recovered: Vec<u32> = vec![0; len];
                let read_bytes = super::decode(&output_buf, &mut recovered);
                assert_eq!(read_bytes, output_buf.len());
                assert_eq!(recovered, input);
            }
        }
    }

    fn create_delta_input(bits: u32, len: usize) -> Vec<u32> {
        use rand::distributions::{Distribution, Uniform};
        let min = 0;
        let max: u64 = (1 << bits) - 1;
        let between = Uniform::from(min..=max);
        let mut rng = rand::thread_rng();
        let mut vec = Vec::with_capacity(len);
        let mut prev: u32 = 0;
        for _ in 0..len {
            let gap = between.sample(&mut rng) as u32;
            let new = prev + gap;
            prev = new;
            vec.push(new);
        }
        vec
    }

    #[test]
    fn encode_decode_delta_roundtrip() {
        let len = 10000;
        for bits in 1..=16 {
            for _ in 0..2 {
                let input = create_delta_input(bits, len);
                let output_buf = super::encode_delta(&input, 0);
                let mut recovered: Vec<u32> = vec![0; len];
                let read_bytes = super::decode_delta(&output_buf, &mut recovered, 0);
                assert_eq!(read_bytes, output_buf.len());
                assert_eq!(recovered, input);
            }
        }
    }
}
