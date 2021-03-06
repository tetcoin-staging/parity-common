#![cfg_attr(not(feature="std"), no_std)]

extern crate uint;

#[cfg(feature="std")]
extern crate core;

#[cfg(feature="std")]
extern crate rustc_hex;

#[cfg(feature="serialize")]
extern crate serde;

#[cfg(feature="serialize")]
#[macro_use]
extern crate serde_derive;

#[cfg(feature="std")]
use core::{fmt, str};
use core::{ops, cmp};
use core::cmp::{min, Ordering};
use core::ops::{Deref, DerefMut, BitXor, BitAnd, BitOr, IndexMut, Index};
use core::hash::{Hash, Hasher};
use uint::U256;

#[cfg(feature="std")]
use rustc_hex::{FromHex, FromHexError};

#[allow(non_camel_case_types)]
#[repr(u8)]
enum c_void {
    // Two dummy variants so the #[repr] attribute can be used.
    #[doc(hidden)]
    __variant1,
    #[doc(hidden)]
    __variant2,
}

#[allow(non_camel_case_types)]
extern {
    fn memcmp(cx: *const c_void, ct: *const c_void, n: usize) -> i32;
}

/// Return `s` without the `0x` at the beginning of it, if any.
#[cfg(feature="std")]
fn clean_0x(s: &str) -> &str {
    if s.starts_with("0x") {
        &s[2..]
    } else {
        s
    }
}

macro_rules! impl_hash {
    ($from: ident, $size: expr $(, $m:meta)*) => {
        #[repr(C)]
        $(#[$m])*
        /// Unformatted binary data of fixed length.
        pub struct $from (pub [u8; $size]);

        impl From<[u8; $size]> for $from {
            fn from(bytes: [u8; $size]) -> Self {
                $from(bytes)
            }
        }

        impl From<$from> for [u8; $size] {
            fn from(s: $from) -> Self {
                s.0
            }
        }

        impl Deref for $from {
            type Target = [u8];

            #[inline]
            fn deref(&self) -> &[u8] {
                &self.0
            }
        }

        impl AsRef<[u8]> for $from {
            #[inline]
            fn as_ref(&self) -> &[u8] {
                &self.0
            }
        }

        impl AsMut<[u8]> for $from {
            #[inline]
            fn as_mut(&mut self) -> &mut [u8] {
                &mut self.0
            }
        }

        impl DerefMut for $from {
            #[inline]
            fn deref_mut(&mut self) -> &mut [u8] {
                &mut self.0
            }
        }

        impl $from {
            /// Create a new, zero-initialised, instance.
            pub fn new() -> $from {
                $from([0; $size])
            }

            /// Synonym for `new()`. Prefer to new as it's more readable.
            pub fn zero() -> $from {
                $from([0; $size])
            }

            /// Get the size of this object in bytes.
            pub fn len() -> usize {
                $size
            }

            /// Returns a constant raw pointer to the value
            pub fn as_ptr(&self) -> *const u8 {
                self.0.as_ptr()
            }

            /// Returns a mutable raw pointer to the value
            pub fn as_mut_ptr(&mut self) -> *mut u8 {
                (&mut self.0).as_mut_ptr()
            }

            #[inline]
            /// Assign self to be of the same value as a slice of bytes of length `len()`.
            pub fn clone_from_slice(&mut self, src: &[u8]) -> usize {
                let min = cmp::min($size, src.len());
                self.0[..min].copy_from_slice(&src[..min]);
                min
            }

            /// Convert a slice of bytes of length `len()` to an instance of this type.
            pub fn from_slice(src: &[u8]) -> Self {
                let mut r = Self::new();
                r.clone_from_slice(src);
                r
            }

            /// Copy the data of this object into some mutable slice of length `len()`.
            pub fn copy_to(&self, dest: &mut[u8]) {
                let min = cmp::min($size, dest.len());
                dest[..min].copy_from_slice(&self.0[..min]);
            }

            /// Returns `true` if all bits set in `b` are also set in `self`.
            pub fn contains<'a>(&'a self, b: &'a Self) -> bool {
                &(b & self) == b
            }

            /// Returns `true` if no bits are set.
            pub fn is_zero(&self) -> bool {
                self.eq(&Self::new())
            }

            /// Returns the lowest 8 bytes interpreted as a BigEndian integer.
            pub fn low_u64(&self) -> u64 {
                let mut ret = 0u64;
                for i in 0..min($size, 8) {
                    ret |= (self.0[$size - 1 - i] as u64) << (i * 8);
                }
                ret
            }
        }

        #[cfg(feature="std")]
        impl str::FromStr for $from {
            type Err = FromHexError;

            fn from_str(s: &str) -> Result<$from, FromHexError> {
                let a : Vec<u8> = s.from_hex()?;
                if a.len() != $size {
                    return Err(FromHexError::InvalidHexLength);
                }

                let mut ret = [0;$size];
                ret.copy_from_slice(&a);
                Ok($from(ret))
            }
        }

        #[cfg(feature="std")]
        impl fmt::Debug for $from {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                for i in &self.0[..] {
                    write!(f, "{:02x}", i)?;
                }
                Ok(())
            }
        }

        #[cfg(feature="std")]
        impl fmt::Display for $from {
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
				for i in &self.0[0..2] {
					write!(f, "{:02x}", i)?;
				}
				write!(f, "???")?;
				for i in &self.0[$size - 2..$size] {
					write!(f, "{:02x}", i)?;
				}
				Ok(())
			}
		}

        impl Copy for $from {}

        impl Clone for $from {
            fn clone(&self) -> $from {
                let mut ret = $from::new();
                ret.0.copy_from_slice(&self.0);
                ret
            }
        }

        impl Eq for $from {}

        impl PartialEq for $from {
            fn eq(&self, other: &Self) -> bool {
                unsafe { memcmp(self.0.as_ptr() as *const c_void, other.0.as_ptr() as *const c_void, $size) == 0 }
            }
        }

        impl Ord for $from {
            fn cmp(&self, other: &Self) -> Ordering {
                let r = unsafe { memcmp(self.0.as_ptr() as *const c_void, other.0.as_ptr() as *const c_void, $size) };
                if r < 0 { return Ordering::Less }
                if r > 0 { return Ordering::Greater }
                return Ordering::Equal;
            }
        }

        impl PartialOrd for $from {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Hash for $from {
            fn hash<H>(&self, state: &mut H) where H: Hasher {
                state.write(&self.0);
                state.finish();
            }
        }

        impl Index<usize> for $from {
            type Output = u8;

            fn index(&self, index: usize) -> &u8 {
                &self.0[index]
            }
        }
        impl IndexMut<usize> for $from {
            fn index_mut(&mut self, index: usize) -> &mut u8 {
                &mut self.0[index]
            }
        }
        impl Index<ops::Range<usize>> for $from {
            type Output = [u8];

            fn index(&self, index: ops::Range<usize>) -> &[u8] {
                &self.0[index]
            }
        }
        impl IndexMut<ops::Range<usize>> for $from {
            fn index_mut(&mut self, index: ops::Range<usize>) -> &mut [u8] {
                &mut self.0[index]
            }
        }
        impl Index<ops::RangeFull> for $from {
            type Output = [u8];

            fn index(&self, _index: ops::RangeFull) -> &[u8] {
                &self.0
            }
        }
        impl IndexMut<ops::RangeFull> for $from {
            fn index_mut(&mut self, _index: ops::RangeFull) -> &mut [u8] {
                &mut self.0
            }
        }

        /// `BitOr` on references
        impl<'a> BitOr for &'a $from {
            type Output = $from;

            fn bitor(self, rhs: Self) -> Self::Output {
                let mut ret: $from = $from::default();
                for i in 0..$size {
                    ret.0[i] = self.0[i] | rhs.0[i];
                }
                ret
            }
        }

        /// Moving `BitOr`
        impl BitOr for $from {
            type Output = $from;

            fn bitor(self, rhs: Self) -> Self::Output {
                &self | &rhs
            }
        }

        /// `BitAnd` on references
        impl <'a> BitAnd for &'a $from {
            type Output = $from;

            fn bitand(self, rhs: Self) -> Self::Output {
                let mut ret: $from = $from::default();
                for i in 0..$size {
                    ret.0[i] = self.0[i] & rhs.0[i];
                }
                ret
            }
        }

        /// Moving `BitAnd`
        impl BitAnd for $from {
            type Output = $from;

            fn bitand(self, rhs: Self) -> Self::Output {
                &self & &rhs
            }
        }

        /// `BitXor` on references
        impl <'a> BitXor for &'a $from {
            type Output = $from;

            fn bitxor(self, rhs: Self) -> Self::Output {
                let mut ret: $from = $from::default();
                for i in 0..$size {
                    ret.0[i] = self.0[i] ^ rhs.0[i];
                }
                ret
            }
        }

        /// Moving `BitXor`
        impl BitXor for $from {
            type Output = $from;

            fn bitxor(self, rhs: Self) -> Self::Output {
                &self ^ &rhs
            }
        }

        #[cfg(feature="std")]
        impl $from {
            /// Get a hex representation.
            pub fn hex(&self) -> String {
                format!("{:?}", self)
            }
        }

        impl Default for $from {
            fn default() -> Self { $from::new() }
        }

        impl From<u64> for $from {
            fn from(mut value: u64) -> $from {
                let mut ret = $from::new();
                for i in 0..8 {
                    if i < $size {
                        ret.0[$size - i - 1] = (value & 0xff) as u8;
                        value >>= 8;
                    }
                }
                ret
            }
        }
        #[cfg(feature="std")]
        impl From<&'static str> for $from {
            fn from(s: &'static str) -> $from {
                let s = clean_0x(s);
                if s.len() % 2 == 1 {
                    ("0".to_string() + s).parse().unwrap()
                } else {
                    s.parse().unwrap()
                }
            }
        }

        impl<'a> From<&'a [u8]> for $from {
            fn from(s: &'a [u8]) -> $from {
                $from::from_slice(s)
            }
        }
    }
}

impl From<U256> for H256 {
    fn from(value: U256) -> H256 {
        let mut ret = H256::new();
        value.to_big_endian(&mut ret);
        ret
    }
}

impl<'a> From<&'a U256> for H256 {
    fn from(value: &'a U256) -> H256 {
        let mut ret: H256 = H256::new();
        value.to_big_endian(&mut ret);
        ret
    }
}

impl From<H256> for U256 {
    fn from(value: H256) -> U256 {
        U256::from(&value)
    }
}

impl<'a> From<&'a H256> for U256 {
    fn from(value: &'a H256) -> U256 {
        U256::from(value.as_ref() as &[u8])
    }
}

impl From<H256> for H160 {
    fn from(value: H256) -> H160 {
        let mut ret = H160::new();
        ret.0.copy_from_slice(&value[12..32]);
        ret
    }
}

impl From<H256> for H64 {
    fn from(value: H256) -> H64 {
        let mut ret = H64::new();
        ret.0.copy_from_slice(&value[20..28]);
        ret
    }
}

impl From<H160> for H256 {
    fn from(value: H160) -> H256 {
        let mut ret = H256::new();
        ret.0[12..32].copy_from_slice(&value);
        ret
    }
}

impl<'a> From<&'a H160> for H256 {
    fn from(value: &'a H160) -> H256 {
        let mut ret = H256::new();
        ret.0[12..32].copy_from_slice(value);
        ret
    }
}

impl_hash!(H32, 4);
impl_hash!(H64, 8);
impl_hash!(H128, 16);
impl_hash!(H160, 20, cfg_attr(feature = "serialize", derive(Serialize, Deserialize)));
impl_hash!(H256, 32);
impl_hash!(H264, 33);
impl_hash!(H512, 64);
impl_hash!(H520, 65);
impl_hash!(H1024, 128);
impl_hash!(H2048, 256);

pub type Address = H160;


#[cfg(test)]
mod tests {
	use super::*;
	use std::str::FromStr;

	#[test]
	#[cfg_attr(feature="dev", allow(eq_op))]
	fn hash() {
		let h = H64([0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]);
		assert_eq!(H64::from_str("0123456789abcdef").unwrap(), h);
		assert_eq!(format!("{}", h), "0123???cdef");
		assert_eq!(format!("{:?}", h), "0123456789abcdef");
		assert_eq!(h.hex(), "0123456789abcdef");
		assert!(h == h);
		assert!(h != H64([0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xee]));
		assert!(h != H64([0; 8]));
	}

	#[test]
	fn hash_bitor() {
		let a = H64([1; 8]);
		let b = H64([2; 8]);
		let c = H64([3; 8]);

		// borrow
		assert_eq!(&a | &b, c);

		// move
		assert_eq!(a | b, c);
	}

	#[test]
	fn from_and_to_address() {
		let address: H160 = "ef2d6d194084c2de36e0dabfce45d046b37d1106".into();
		let h = H256::from(address.clone());
		let a = H160::from(h);
		assert_eq!(address, a);
	}

	#[test]
	fn from_u64() {
		assert_eq!(H128::from(0x1234567890abcdef), H128::from_str("00000000000000001234567890abcdef").unwrap());
		assert_eq!(H64::from(0x1234567890abcdef), H64::from_str("1234567890abcdef").unwrap());
		assert_eq!(H32::from(0x1234567890abcdef), H32::from_str("90abcdef").unwrap());
	}

	#[test]
	fn from_str() {
		assert_eq!(H64::from(0x1234567890abcdef), H64::from("0x1234567890abcdef"));
		assert_eq!(H64::from(0x1234567890abcdef), H64::from("1234567890abcdef"));
		assert_eq!(H64::from(0x234567890abcdef), H64::from("0x234567890abcdef"));
	}

	#[test]
	fn from_and_to_u256() {
		let u: U256 = 0x123456789abcdef0u64.into();
		let h = H256::from(u);
		assert_eq!(H256::from(u), H256::from("000000000000000000000000000000000000000000000000123456789abcdef0"));
		let h_ref = H256::from(&u);
		assert_eq!(h, h_ref);
		let r_ref: U256 = From::from(&h);
		assert_eq!(r_ref, u);
		let r: U256 = From::from(h);
		assert_eq!(r, u);
	}
}
