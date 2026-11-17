pub mod bit_set;
pub mod codegen;
pub mod convenience;
pub mod cst;
pub mod formatter;
pub mod lexer;
pub mod machine;
pub mod ntt;
pub mod oset;
pub mod preflight;
pub mod source_text;
pub mod table;
pub mod takeoff;
pub mod token;

mod internal_prelude;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RustCode {
    pub raw: String,
}
