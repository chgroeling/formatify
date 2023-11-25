use super::parsing_context::ParsingContext;

use std::collections::HashMap;

pub trait ParsingTask {
    type Item;
    type Output;

    /// Initializes the parsing context at the start of parsing.
    fn init<'a>(
        inp: &'a str,
        key_value: &'a HashMap<&'a str, String>,
    ) -> ParsingContext<'a, Self::Item>;

    /// Finalizes the parsing process.
    fn done(context: ParsingContext<'_, Self::Item>) -> Self::Output;

    /// Handles errors encountered during parsing.
    fn error(context: &mut ParsingContext<'_, Self::Item>);

    /// Copies a character from the input to the output as is.
    fn process_char(context: &mut ParsingContext<'_, Self::Item>, ch: char);

    /// Processes a single character placeholder.
    fn process_char_placeholder(context: &mut ParsingContext<'_, Self::Item>, ch: char);

    /// Processes a placeholder represented by a string.
    fn process_str_placeholder(context: &mut ParsingContext<'_, Self::Item>, arg: String);
}
