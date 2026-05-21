use std::{
    fmt::{self, Display},
    fs::File,
    io::{BufReader, Read, Result},
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};

use digest::Digest;
use strum::{Display, EnumIter, EnumString};

pub mod hashfile;

// Dispatches to a generic function for each algorithm variant, avoiding match duplication.
macro_rules! dispatch {
    ($algo:expr, $fn:ident) => {
        match $algo {
            Algorithm::Blake3         => $fn::<blake3::Hasher>,
            Algorithm::Blake2s        => $fn::<blake2::Blake2s256>,
            Algorithm::Blake2b        => $fn::<blake2::Blake2b512>,
            Algorithm::Sha3_512       => $fn::<sha3::Sha3_512>,
            Algorithm::Sha3_384       => $fn::<sha3::Sha3_384>,
            Algorithm::Sha3_256       => $fn::<sha3::Sha3_256>,
            Algorithm::Sha3_224       => $fn::<sha3::Sha3_224>,
            Algorithm::Keccak512      => $fn::<sha3::Keccak512>,
            Algorithm::Keccak384      => $fn::<sha3::Keccak384>,
            Algorithm::Keccak256      => $fn::<sha3::Keccak256>,
            Algorithm::Keccak224      => $fn::<sha3::Keccak224>,
            Algorithm::Sha2_512       => $fn::<sha2::Sha512>,
            Algorithm::Sha2_512_256   => $fn::<sha2::Sha512_256>,
            Algorithm::Sha2_512_224   => $fn::<sha2::Sha512_224>,
            Algorithm::Sha2_384       => $fn::<sha2::Sha384>,
            Algorithm::Sha2_256       => $fn::<sha2::Sha256>,
            Algorithm::Sha2_224       => $fn::<sha2::Sha224>,
            Algorithm::Sha1           => $fn::<sha1::Sha1>,
            Algorithm::Md5            => $fn::<md5::Md5>,
            Algorithm::Fsb512         => $fn::<fsb::Fsb512>,
            Algorithm::Fsb384         => $fn::<fsb::Fsb384>,
            Algorithm::Fsb256         => $fn::<fsb::Fsb256>,
            Algorithm::Fsb224         => $fn::<fsb::Fsb224>,
            Algorithm::Fsb160         => $fn::<fsb::Fsb160>,
            Algorithm::Gost94Test     => $fn::<gost94::Gost94Test>,
            Algorithm::Gost94CryptoPro=> $fn::<gost94::Gost94CryptoPro>,
            Algorithm::Gost94s2015    => $fn::<gost94::Gost94s2015>,
            Algorithm::Gost94UA       => $fn::<gost94::Gost94UA>,
            Algorithm::Groestl512     => $fn::<groestl::Groestl512>,
            Algorithm::Groestl384     => $fn::<groestl::Groestl384>,
            Algorithm::Groestl256     => $fn::<groestl::Groestl256>,
            Algorithm::Groestl224     => $fn::<groestl::Groestl224>,
            Algorithm::Md4            => $fn::<md4::Md4>,
            Algorithm::Md2            => $fn::<md2::Md2>,
            Algorithm::Ripemd320      => $fn::<ripemd::Ripemd320>,
            Algorithm::Ripemd256      => $fn::<ripemd::Ripemd256>,
            Algorithm::Ripemd160      => $fn::<ripemd::Ripemd160>,
            Algorithm::Ripemd128      => $fn::<ripemd::Ripemd128>,
            Algorithm::Shabal512      => $fn::<shabal::Shabal512>,
            Algorithm::Shabal384      => $fn::<shabal::Shabal384>,
            Algorithm::Shabal256      => $fn::<shabal::Shabal256>,
            Algorithm::Shabal224      => $fn::<shabal::Shabal224>,
            Algorithm::Shabal192      => $fn::<shabal::Shabal192>,
            Algorithm::Sm3            => $fn::<sm3::Sm3>,
            Algorithm::Streebog512    => $fn::<streebog::Streebog512>,
            Algorithm::Streebog256    => $fn::<streebog::Streebog256>,
            Algorithm::Tiger          => $fn::<tiger::Tiger>,
            Algorithm::Tiger2         => $fn::<tiger::Tiger2>,
            Algorithm::Whirlpool      => $fn::<whirlpool::Whirlpool>,
        }
    };
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, EnumString)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
#[strum(serialize_all = "kebab-case")]
pub enum Mode {
    #[default]
    Text,
    Binary,
}

impl Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::fmt::Write;

        f.write_char(match *self {
            Self::Text => ' ',
            Self::Binary => '*',
        })
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Display, EnumString, EnumIter)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
#[strum(serialize_all = "kebab-case")]
pub enum Algorithm {
    #[default]
    Blake3,
    Blake2s,
    Blake2b,
    Sha3_512,
    Sha3_384,
    Sha3_256,
    Sha3_224,
    Keccak512,
    Keccak384,
    Keccak256,
    Keccak224,
    Sha2_512,
    #[strum(serialize = "sha2-512-256")]
    Sha2_512_256,
    #[strum(serialize = "sha2-512-224")]
    Sha2_512_224,
    Sha2_384,
    Sha2_256,
    Sha2_224,
    Sha1,
    Md5,
    #[strum(serialize = "fsb-512")]
    Fsb512,
    #[strum(serialize = "fsb-384")]
    Fsb384,
    #[strum(serialize = "fsb-256")]
    Fsb256,
    #[strum(serialize = "fsb-224")]
    Fsb224,
    #[strum(serialize = "fsb-160")]
    Fsb160,
    #[strum(serialize = "gost94-test")]
    Gost94Test,
    #[strum(serialize = "gost94-cryptopro")]
    Gost94CryptoPro,
    #[strum(serialize = "gost94-s2015")]
    Gost94s2015,
    #[strum(serialize = "gost94-ua")]
    Gost94UA,
    #[strum(serialize = "groestl-512")]
    Groestl512,
    #[strum(serialize = "groestl-384")]
    Groestl384,
    #[strum(serialize = "groestl-256")]
    Groestl256,
    #[strum(serialize = "groestl-224")]
    Groestl224,
    Md4,
    Md2,
    Ripemd320,
    Ripemd256,
    Ripemd160,
    Ripemd128,
    #[strum(serialize = "shabal-512")]
    Shabal512,
    #[strum(serialize = "shabal-384")]
    Shabal384,
    #[strum(serialize = "shabal-256")]
    Shabal256,
    #[strum(serialize = "shabal-224")]
    Shabal224,
    #[strum(serialize = "shabal-192")]
    Shabal192,
    Sm3,
    #[strum(serialize = "streebog-512")]
    Streebog512,
    #[strum(serialize = "streebog-256")]
    Streebog256,
    Tiger,
    Tiger2,
    Whirlpool,
}

impl Algorithm {
    /// Returns a plain hasher function (used by the CLI with rayon).
    pub fn into_hasher(self) -> fn(&Path) -> Result<String> {
        dispatch!(self, hash)
    }

    /// Returns a progress-aware hasher function for use in the GUI.
    /// The `progress` counter is incremented by the number of bytes read,
    /// allowing the caller to compute a fraction against the file size.
    pub fn into_hasher_with_progress(self) -> fn(&Path, &AtomicU64) -> Result<String> {
        dispatch!(self, hash_with_progress)
    }
}

fn hash<H: SimpleHasher>(input: &Path) -> Result<String> {
    let mut reader = BufReader::new(File::open(input)?);
    let mut hasher = H::new();
    let mut buf = vec![0u8; 1 << 16];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize())
}

fn hash_with_progress<H: SimpleHasher>(input: &Path, progress: &AtomicU64) -> Result<String> {
    let mut reader = BufReader::new(File::open(input)?);
    let mut hasher = H::new();
    let mut buf = vec![0u8; 1 << 16];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        progress.fetch_add(n as u64, Ordering::Relaxed);
    }
    Ok(hasher.finalize())
}

trait SimpleHasher {
    fn new() -> Self;
    fn update(&mut self, data: &[u8]);
    fn finalize(self) -> String;
}

impl<D: Digest> SimpleHasher for D {
    fn new() -> Self {
        D::new()
    }

    fn update(&mut self, data: &[u8]) {
        D::update(self, data);
    }

    fn finalize(self) -> String {
        D::finalize(self).iter().map(|b| format!("{:02x}", b)).collect()
    }
}

