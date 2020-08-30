#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub type __uint8_t = ::std::os::raw::c_uchar;
pub type __uint32_t = ::std::os::raw::c_uint;
pub type size_t = ::std::os::raw::c_ulong;
extern "C" {
    pub fn streamvbyte_encode(in_: *const u32, length: u32, out: *mut u8) -> size_t;
}
extern "C" {
    pub fn streamvbyte_decode(in_: *const u8, out: *mut u32, length: u32) -> size_t;
}
extern "C" {
    pub fn streamvbyte_delta_encode(
        in_: *const u32,
        length: u32,
        out: *mut u8,
        prev: u32,
    ) -> size_t;
}
extern "C" {
    pub fn streamvbyte_delta_decode(
        in_: *const u8,
        out: *mut u32,
        length: u32,
        prev: u32,
    ) -> size_t;
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
                let mut output_buf: Vec<u8> = vec![0; 5 * len];
                let written_bytes = unsafe {
                    super::streamvbyte_encode(
                        input.as_ptr(),
                        input.len() as u32,
                        output_buf.as_mut_ptr(),
                    )
                };
                let mut recovered: Vec<u32> = vec![0; len];
                let read_bytes = unsafe {
                    super::streamvbyte_decode(
                        output_buf.as_ptr(),
                        recovered.as_mut_ptr(),
                        len as u32,
                    )
                };

                assert_eq!(read_bytes, written_bytes);
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
                let mut output_buf: Vec<u8> = vec![0; 5 * len];
                let written_bytes = unsafe {
                    super::streamvbyte_delta_encode(
                        input.as_ptr(),
                        input.len() as u32,
                        output_buf.as_mut_ptr(),
                        0,
                    )
                };
                let mut recovered: Vec<u32> = vec![0; len];
                let read_bytes = unsafe {
                    super::streamvbyte_delta_decode(
                        output_buf.as_ptr(),
                        recovered.as_mut_ptr(),
                        len as u32,
                        0,
                    )
                };

                assert_eq!(read_bytes, written_bytes);
                assert_eq!(recovered, input);
            }
        }
    }
}
