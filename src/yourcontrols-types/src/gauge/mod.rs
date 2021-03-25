mod fragment;
mod messages;
mod msgpack;

pub use fragment::*;
pub use messages::*;
pub use msgpack::*;

/// Type used for keeping track of vars mapped to values.
pub type DatumKey = u32;
/// Type used for getting/setting values.
pub type DatumValue = f64;
/// Time type used for interpolation.
pub type Time = f64;
/// Type used foor mapping events/vars
pub type VarId = usize;
