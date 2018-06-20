#[cfg(test)]
#[macro_use]
extern crate approx;
extern crate num;

use num::{Integer, PrimInt, Unsigned};

use std::mem::size_of;
use std::ops::{Shl, Shr};

/// Compress integer input so that higher resolution is preserved for
/// values near zero, while reducing systematic bias between input and
/// compressed output
///
/// The motivating use case is to reduce the total number of integers used for
/// a map key, where the integer represents count data, such that high resolution
/// should be preserved near zero
///
/// The compression scheme is: The highest "bit_count" bits from the input integer
/// are preserved, followed by a suffix. The suffix is fixed for all input with the
/// same prefix, and is set in such a way that systematic bias between the input
/// and compressed output is reduced.
///
/// This function deterministically switches between two suffix values, one
/// has only the high bit set, the other is the complement of the first. The suffix
/// is chosen based on the low bit of the prefix (or shift value if bitCount is one).
///
/// Example: For bit_count=3 and input=67:
///
/// input  is 67 or 0b1000011
/// output is 72 or 0b1001000
///
/// Here the top three bits are preserved "0b100XXXX", and the suffix "0bXXX1000" is
/// replaced on the lower bits to create the compressed output.
///
/// T must be of an unsigned integral type
///
pub fn compress_int<T>(input: T, bit_count: u32) -> T
where
    T: Integer + PrimInt + Unsigned + Shl<u32, Output = T> + Shr<u32, Output = T> + Into<u32>,
{
    assert!(bit_count > 0);

    // find last bit (should match POSIX fls() function)
    let input_bit_count = (size_of::<T>() * 8) as u32;
    let high_bit_index = input_bit_count - input.leading_zeros();

    if high_bit_index <= bit_count {
        return input;
    }

    let shift = high_bit_index - bit_count;
    let prefix = input >> shift;

    // switch off between two different suffix schemes to reduce bias
    // scheme 1: suffix is 0b10000...
    // scheme 2: suffix is 0b01111...
    let mut suffix = T::one() << (shift - 1);

    if ((if bit_count == 1 {
        shift
    } else {
        prefix.clone().into()
    }) & 0b1) == 1
    {
        suffix = suffix - T::one();
    }

    (prefix << shift) | suffix
}


#[cfg(test)]
mod tests {
    use compress_int;

    #[test]
    fn test_compression() {
        let bit_count : u32 = 3;
        for i in 0u32..8u32 {
            assert_eq!(compress_int(i, bit_count), i);
        }
        for i in 8u32..10u32 {
            assert_eq!(compress_int(i, bit_count), 9);
        }
        for i in 10u32..12u32 {
            assert_eq!(compress_int(i, bit_count), 10);
        }
        for i in 16u32..20u32 {
            assert_eq!(compress_int(i, bit_count), 18);
        }

        //let test_val : u64 = 123_039_843_249;
        //let expect : u64  = 128_849_018_879;
        //assert_eq!(compress_int(test_val, bit_count), expect);

        // example in function doc:
        assert_eq!(compress_int(67u32,bit_count), 72u32);
    }

    #[test]
    fn test_bias() {
        let bit_count : u32 = 2;
        let n : u32 = 1024;
        let eps : f64 = 0.000_001;

        let mut sum : f64 = 0.;
        for i in 0..n {
           sum += compress_int(i, bit_count) as f64;
        }
        sum /= n as f64;
        let expect : f64 = (n-1) as f64/2.;

        assert_relative_eq!(sum, expect, epsilon = eps);
    }
}
