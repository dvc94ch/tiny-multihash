use crate::hasher::{Digest, Hasher, Size};
use generic_array::GenericArray;

macro_rules! derive_digest {
    ($name:ident) => {
        /// Multihash digest.
        #[derive(Clone, Debug, Default, Eq, PartialEq)]
        pub struct $name<S: Size>(GenericArray<u8, S>);

        impl<S: Size> AsRef<[u8]> for $name<S> {
            fn as_ref(&self) -> &[u8] {
                &self.0
            }
        }

        impl<S: Size> From<GenericArray<u8, S>> for $name<S> {
            fn from(array: GenericArray<u8, S>) -> Self {
                Self(array)
            }
        }

        impl<S: Size> From<$name<S>> for GenericArray<u8, S> {
            fn from(digest: $name<S>) -> Self {
                digest.0
            }
        }

        impl<S: Size> Digest<S> for $name<S> {}
    };
}

#[cfg(any(feature = "blake2b", feature = "blake2s"))]
macro_rules! derive_hasher_blake {
    ($module:ident, $name:ident, $digest:ident) => {
        derive_digest!($digest);

        /// Multihash hasher.
        pub struct $name<S: Size> {
            _marker: PhantomData<S>,
            state: $module::State,
        }

        impl<S: Size> Default for $name<S> {
            fn default() -> Self {
                let mut params = $module::Params::new();
                params.hash_length(S::to_usize());
                Self {
                    _marker: PhantomData,
                    state: params.to_state(),
                }
            }
        }

        impl<S: Size> Hasher for $name<S> {
            type Size = S;
            type Digest = $digest<Self::Size>;

            fn update(&mut self, input: &[u8]) {
                self.state.update(input);
            }

            fn finalize(&self) -> Self::Digest {
                let digest = GenericArray::clone_from_slice(self.state.finalize().as_bytes());
                Self::Digest::from(digest)
            }

            fn reset(&mut self) {
                let Self { state, .. } = Self::default();
                self.state = state;
            }
        }
    };
}

#[cfg(feature = "blake2b")]
pub mod blake2b {
    use super::*;
    use core::marker::PhantomData;
    use generic_array::typenum::{U32, U64};

    derive_hasher_blake!(blake2b_simd, Blake2bHasher, Blake2bDigest);

    /// 256 bit blake2b hasher.
    pub type Blake2b256 = Blake2bHasher<U32>;

    /// 512 bit blake2b hasher.
    pub type Blake2b512 = Blake2bHasher<U64>;
}

#[cfg(feature = "blake2s")]
pub mod blake2s {
    use super::*;
    use core::marker::PhantomData;
    use generic_array::typenum::{U16, U32};

    derive_hasher_blake!(blake2s_simd, Blake2sHasher, Blake2sDigest);

    /// 256 bit blake2b hasher.
    pub type Blake2s128 = Blake2sHasher<U16>;

    /// 512 bit blake2b hasher.
    pub type Blake2s256 = Blake2sHasher<U32>;
}

#[cfg(feature = "digest")]
macro_rules! derive_hasher_sha {
    ($module:ty, $name:ident, $size:ty, $digest:ident) => {
        /// Multihash hasher.
        #[derive(Default)]
        pub struct $name {
            state: $module,
        }

        impl $crate::hasher::Hasher for $name {
            type Size = $size;
            type Digest = $digest<Self::Size>;

            fn update(&mut self, input: &[u8]) {
                use digest::Digest;
                self.state.update(input)
            }

            fn finalize(&self) -> Self::Digest {
                use digest::Digest;
                Self::Digest::from(self.state.clone().finalize())
            }

            fn reset(&mut self) {
                use digest::Digest;
                self.state.reset();
            }
        }
    };
}

#[cfg(feature = "sha1")]
pub mod sha1 {
    use super::*;
    use generic_array::typenum::U20;

    derive_digest!(Sha1Digest);
    derive_hasher_sha!(::sha1::Sha1, Sha1, U20, Sha1Digest);
}

#[cfg(feature = "sha2")]
pub mod sha2 {
    use super::*;
    use generic_array::typenum::{U32, U64};

    derive_digest!(Sha2Digest);
    derive_hasher_sha!(sha_2::Sha256, Sha2_256, U32, Sha2Digest);
    derive_hasher_sha!(sha_2::Sha512, Sha2_512, U64, Sha2Digest);
}

#[cfg(feature = "sha3")]
pub mod sha3 {
    use super::*;
    use generic_array::typenum::{U28, U32, U48, U64};

    derive_digest!(Sha3Digest);
    derive_hasher_sha!(sha_3::Sha3_224, Sha3_224, U28, Sha3Digest);
    derive_hasher_sha!(sha_3::Sha3_256, Sha3_256, U32, Sha3Digest);
    derive_hasher_sha!(sha_3::Sha3_384, Sha3_384, U48, Sha3Digest);
    derive_hasher_sha!(sha_3::Sha3_512, Sha3_512, U64, Sha3Digest);

    derive_digest!(KeccakDigest);
    derive_hasher_sha!(sha_3::Keccak224, Keccak224, U28, KeccakDigest);
    derive_hasher_sha!(sha_3::Keccak256, Keccak256, U32, KeccakDigest);
    derive_hasher_sha!(sha_3::Keccak384, Keccak384, U48, KeccakDigest);
    derive_hasher_sha!(sha_3::Keccak512, Keccak512, U64, KeccakDigest);
}

pub mod identity {
    use super::*;
    use generic_array::typenum::U32;

    derive_digest!(IdentityDigest);

    /// Identity hasher.
    #[derive(Default)]
    pub struct IdentityHasher<S: Size> {
        bytes: GenericArray<u8, S>,
        i: usize,
    }

    impl<S: Size> Hasher for IdentityHasher<S> {
        type Size = S;
        type Digest = IdentityDigest<Self::Size>;

        fn update(&mut self, input: &[u8]) {
            let start = self.i;
            let end = start + input.len();
            self.bytes[start..end].copy_from_slice(input);
            self.i = end;
        }

        fn finalize(&self) -> Self::Digest {
            Self::Digest::from(self.bytes.clone())
        }

        fn reset(&mut self) {
            self.bytes = Default::default();
            self.i = 0;
        }
    }

    /// 256 bit Identity hasher.
    pub type Identity256 = IdentityHasher<U32>;
}

pub mod unknown {
    use super::*;
    derive_digest!(UnknownDigest);
}

#[cfg(feature = "strobe")]
pub mod strobe {
    use super::*;
    use core::marker::PhantomData;
    use generic_array::typenum::{U32, U64};
    use strobe_rs::{SecParam, Strobe};

    derive_digest!(StrobeDigest);

    /// Strobe hasher.
    pub struct StrobeHasher<S: Size> {
        _marker: PhantomData<S>,
        strobe: Strobe,
        initialized: bool,
    }

    impl<S: Size> Default for StrobeHasher<S> {
        fn default() -> Self {
            Self {
                _marker: PhantomData,
                strobe: Strobe::new(b"StrobeHash", SecParam::B128),
                initialized: false,
            }
        }
    }

    impl<S: Size> Hasher for StrobeHasher<S> {
        type Size = S;
        type Digest = StrobeDigest<Self::Size>;

        fn update(&mut self, input: &[u8]) {
            self.strobe.ad(input, self.initialized);
            self.initialized = true;
        }

        fn finalize(&self) -> Self::Digest {
            let mut hash = GenericArray::default();
            self.strobe.clone().prf(&mut hash, false);
            Self::Digest::from(hash)
        }

        fn reset(&mut self) {
            let Self { strobe, .. } = Self::default();
            self.strobe = strobe;
            self.initialized = false;
        }
    }

    /// 256 bit strobe hasher.
    pub type Strobe256 = StrobeHasher<U32>;

    /// 512 bit strobe hasher.
    pub type Strobe512 = StrobeHasher<U64>;
}
