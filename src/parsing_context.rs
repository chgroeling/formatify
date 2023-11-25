use super::output_format::OutputFormat;
use super::peek_char_iterator::PeekCharIterator;

use std::collections::HashMap;

pub struct ParsingContext<'a, T> {
    pub key_value: &'a HashMap<&'a str, String>,
    pub iter: PeekCharIterator,
    pub vout: Vec<T>,
    pub format: OutputFormat,
}
