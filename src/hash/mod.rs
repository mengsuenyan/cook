mod hash;
mod adler32;

pub use hash::{GenericHasher, GenericHasher32, GenericHasher64, GenericHasher128};
pub use adler32::Adler32;
