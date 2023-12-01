//! # Formatify: Dynamic String Formatting Library
//!
//! Formatify is a Rust library designed for dynamic string formatting. It provides flexible and powerful
//! tools for parsing strings with placeholders, similar to how Git formats commit logs (see [Git pretty format](https://git-scm.com/docs/pretty-formats)),
//! and replacing them with corresponding values. The library excels in handling various placeholder
//! formats and alignment options, ideal for applications requiring adaptable text representation.
//!
//! ## Placeholder Formats
//!
//! Formatify supports several types of placeholders, enabling a wide range of formatting options.
//! Placeholders are defined within a string using a specific syntax, typically denoted by `%(key)`.
//! The library processes these placeholders and replaces them with corresponding values at runtime.
//!
//! ### Supported Placeholder Types:
//!
//! 1. **Single-Character Placeholders**:
//!    - **New Line (`%n`)**: Inserts a newline character where `%n` is placed.
//!    - **Percentage (`%%`)**: Escapes and inserts a literal percent sign.
//!
//! 2. **Variable Substitution**:
//!    - **Syntax**: `%(key)`
//!    - **Description**: Replaces this placeholder with the value associated with `key` in the `key_value` HashMap.
//!
//! 3. **Format Placeholders**:
//!    - **Left Alignment**:
//!        - **Syntax**: `%<(width)`
//!        - **Description**: Aligns the subsequent placeholder to the left within a field of `width` characters. The placeholder itself is not displayed.
//!    - **Left Alignment with Truncation**:
//!        - **Syntax**: `%<(width,trunc)`
//!        - **Description**: Similar to left alignment, but truncates the text to fit within the specified `width`. The placeholder itself is not displayed.
//!    - **Right Alignment**:
//!        - **Syntax**: `%>(width)`
//!        - **Description**: Aligns the subsequent placeholder to the right within a field of `width` characters. The placeholder itself is not displayed.
//!    - **Right Alignment with Truncation**:
//!        - **Syntax**: `%>(width,trunc)`
//!        - **Description**: Similar to right alignment, but truncates the text to fit within the specified `width`. The placeholder itself is not displayed.
//!    - **Right Alignment with left Truncation**:
//!        - **Syntax**: `%>(width,ltrunc)`
//!        - **Description**: Similar to right alignment, but left truncates the text to fit within the specified `width`. The placeholder itself is not displayed.

//!
//!
//! Note: In the context of format placeholders, `width` refers to the total number of characters allocated for the value being formatted. For example, `%<(10)` aligns the value within a 10-character wide field.
//!
//! ### Example Usage:
//!
//! ```rust
//! # use formatify::{Formatify, PlaceholderFormatter};
//! # use std::collections::HashMap;
//! let mut key_value = HashMap::new();
//! key_value.insert("name", "Alice".into());
//! let formatter = Formatify::new();
//! let formatted_string = formatter.replace_placeholders(&key_value, "Hello, %(name)!");
//! assert_eq!(formatted_string, "Hello, Alice!");
//! ```
//!
//! ## Public Methods
//!
//! Public methods utilizing these placeholders include:
//! - `replace_placeholders`: Replaces placeholders in a string with values from a HashMap.
//! - `measure_lengths`: Calculates the length of strings and placeholders.
//! - `extract_placeholder_keys`: Extracts and lists all valid placeholder keys from a string.
//!
//! For more details on these methods and their usage, refer to the respective method documentation in this module.
//!
//! ## Integration and Compatibility
//!
//! Formatify is designed to be easily integrated into existing Rust projects and works seamlessly with standard data
//! types and collections.
//!
//! ## Contribution and Feedback
//!
//! Contributions to Formatify are welcome. For bug reports, feature requests, or general feedback, please open an issue
//! on the repository's issue tracker.

mod output_format;
mod parsing_context;
mod parsing_task;
mod parsing_task_extract_placeholder_keys;
mod parsing_task_measure_lengths;
mod parsing_task_replace_placeholders;
mod peek_char_iterator;
mod placeholder_formatter;

use self::output_format::OutputFormat;
use self::parsing_context::ParsingContext;
use self::parsing_task::ParsingTask;
use self::parsing_task_extract_placeholder_keys::ParsingTaskExtractPlaceholderKeys;
use self::parsing_task_measure_lengths::ParsingTaskMeasureLengths;
use self::parsing_task_replace_placeholders::ParsingTaskReplacePlaceholders;
pub use self::placeholder_formatter::PlaceholderFormatter;
use std::collections::HashMap;

/// `consume_expected_chars` checks and consumes the next char in the iterator if it matches the provided pattern(s).
/// - `$context`: The parsing context containing the `PeekCharIterator`.
/// - `$($a:pat)+`: Pattern(s) to match against the next char.
/// If the next char matches, it's consumed and returned as `Some(char)`. Otherwise, returns `None`.
macro_rules! consume_expected_chars{
    ($context:ident, $($a:pat)+) => {
        if let Some(ch) = $context.iter.peek()  {
            match ch {
                $($a)|+ => {
                    $context.iter.next(); // consume
                    Some(ch)
                }
                _ => {
                    None
                }
            }
        } else {
            None
        }
    };
}

macro_rules! consume_digits {
    ($context:ident) => {
        consume_expected_chars!($context, '0'..='9')
    };
}

macro_rules! consume_digits_without_0 {
    ($context:ident) => {
        consume_expected_chars!($context, '1'..='9')
    };
}

macro_rules! gather {
    ($context:ident, $($a:pat)+) => {{
        let mut vec: Vec<char> = Vec::new();
        loop {
            let Some(ch) = $context.iter.peek() else {
                break None;
            };

            match ch {
                $($a)|+ => {
                    vec.push(ch);
                    $context.iter.next();

                }
                _ => {
                    break Some(vec);
                }
            }
        }
    }};
}

macro_rules! gather_str_placeholder {
    ($context:ident) => {
        gather!(
            $context,
            ('0'..='9')
                | ('a'..='z')
                | ('A'..='Z')
                | '_'
                | '+'
                | '*'
                | '/'
                | 'ä'
                | 'ö'
                | 'ü'
                | 'ß'
                | '?'
        )
    };
}

macro_rules! skip_until_neg_char_match {
    ($context:ident, $a:expr) => {
        loop {
            let Some(ch) = $context.iter.peek() else {
                break None;
            };

            if ch != $a {
                break Some(());
            } else {
                $context.iter.next();
            }
        }
    };
}

/// `Formatify`: Main struct for dynamic string formatting.
///
/// This struct is part of the `formatify` library, offering tools to parse strings with
/// placeholders and replace them with values from a `HashMap`. It handles various placeholder
/// formats and alignment options, suitable for adaptive text representation in diverse applications.
/// Key functionalities include replacing placeholders, measuring string lengths with placeholders,
/// and extracting placeholder keys.
///
/// ## Usage
///
/// Import the necessary modules and use `Formatify` for string formatting tasks:
///
/// ```rust
/// use formatify::{Formatify, PlaceholderFormatter};
/// use std::collections::HashMap;
/// ```
///
/// ## Features
///
/// 1. **Placeholder Replacement**: Replace placeholders in strings with values from a `HashMap`.
/// 2. **Length Measurement**: Measure lengths of strings and placeholders.
/// 3. **Placeholder Extraction**: Extract placeholder keys from a string.
///
/// ## Examples
///
/// ### Replacing Placeholders
///
/// ```rust
/// # use formatify::{Formatify, PlaceholderFormatter};
/// # use std::collections::HashMap;
/// let mut key_value: HashMap<&str, String> = HashMap::new();
/// key_value.insert("name", "Alice".into());
/// let formatter = Formatify::new();
/// let formatted_string = formatter.replace_placeholders(&key_value, "Hello, %(name)!");
/// assert_eq!(formatted_string, "Hello, Alice!");
/// ```
///
/// ### Measuring Lengths
///
/// ```rust
/// # use formatify::{Formatify, PlaceholderFormatter};
/// # use std::collections::HashMap;
/// let mut key_value: HashMap<&str, String> = HashMap::new();
/// key_value.insert("name", "Alice".into());
/// let formatter = Formatify::new();
/// let lengths = formatter.measure_lengths(&key_value, "Hello, %(name)! This is a test.");
/// assert_eq!(lengths, vec![29, 5]); // Total length with "Alice" as the placeholder, length of "Alice"
/// ```
///
/// ### Extracting Placeholder Keys
///
/// ```rust
/// # use formatify::{Formatify, PlaceholderFormatter};
/// let formatter = Formatify::new();
/// let placeholder_keys = formatter.extract_placeholder_keys("Hello, %(name)! Today is %(day).");
/// assert_eq!(placeholder_keys, vec!["name", "day"]);
/// ```
pub struct Formatify;

impl Formatify {
    pub fn new() -> Self {
        Self
    }

    fn parse_decimal_number<I>(&self, context: &mut ParsingContext<'_, I>) -> Option<u32> {
        let mut decimal_vec = Vec::<char>::new();

        let Some(first_digit) = consume_digits_without_0!(context) else {
            return None;
        };

        decimal_vec.push(first_digit);
        loop {
            let res_digit = consume_digits!(context);

            let Some(digit) = res_digit else {
                let decimal_str: String = decimal_vec.into_iter().collect();
                let decimal = decimal_str.parse::<u32>().unwrap();
                return Some(decimal);
            };

            decimal_vec.push(digit);
        }
    }

    fn process_str_placeholder<T: ParsingTask>(&self, context: &mut ParsingContext<'_, T::Item>) {
        let opt_literal = gather_str_placeholder!(context);

        let Some(literal) = opt_literal else {
            T::error(context);
            return;
        };
        context.iter.next(); // consume ")"

        T::process_str_placeholder(context, literal.into_iter().collect());

        // Reset format for next Placeholder
        context.format = OutputFormat::None;
    }

    fn process_format_left_placeholder<T: ParsingTask>(
        &self,
        context: &mut ParsingContext<'_, T::Item>,
    ) {
        if consume_expected_chars!(context, '(').is_none() {
            T::error(context);
            return;
        }
        skip_until_neg_char_match!(context, ' '); // consume whitespaces

        let Some(decimal) = self.parse_decimal_number(context) else {
            T::error(context);
            return;
        };

        skip_until_neg_char_match!(context, ' '); // consume whitespaces

        // Check if optional arguments are available
        if consume_expected_chars!(context, ',').is_some() {
            skip_until_neg_char_match!(context, ' '); // consume whitespaces
            let Some(literal) = gather_str_placeholder!(context) else {
                T::error(context);
                return;
            };
            skip_until_neg_char_match!(context, ' '); // consume whitespaces
            context.iter.next(); // consume )
            let arg: String = literal.into_iter().collect();

            if arg.trim() == "trunc" {
                context.format = OutputFormat::LeftAlignTrunc(decimal);
                return;
            }

            T::error(context);
        } else {
            if consume_expected_chars!(context, ')').is_none() {
                T::error(context);
                return;
            }

            context.format = OutputFormat::LeftAlign(decimal);
        }
    }

    fn process_format_right_placeholder<T: ParsingTask>(
        &self,
        context: &mut ParsingContext<'_, T::Item>,
    ) {
        if consume_expected_chars!(context, '(').is_none() {
            T::error(context);
            return;
        }
        skip_until_neg_char_match!(context, ' '); // consume whitespaces

        let Some(decimal) = self.parse_decimal_number(context) else {
            T::error(context);
            return;
        };

        skip_until_neg_char_match!(context, ' '); // consume whitespaces

        // Check if optional arguments are available
        if consume_expected_chars!(context, ',').is_some() {
            skip_until_neg_char_match!(context, ' '); // consume whitespaces
            let Some(literal) = gather_str_placeholder!(context) else {
                T::error(context);
                return;
            };
            skip_until_neg_char_match!(context, ' '); // consume whitespaces
            context.iter.next(); // consume )
            let arg: String = literal.into_iter().collect();

            match arg.trim() {
                "trunc" => {
                    context.format = OutputFormat::RightAlignTrunc(decimal);
                    return;
                }
                "ltrunc" => {
                    context.format = OutputFormat::RightAlignLTrunc(decimal);
                    return;
                }
                _ => {}
            }

            T::error(context);
        } else {
            if consume_expected_chars!(context, ')').is_none() {
                T::error(context);
                return;
            }

            context.format = OutputFormat::RightAlign(decimal);
        }
    }

    fn process_placeholder<T: ParsingTask>(&self, context: &mut ParsingContext<'_, T::Item>) {
        let Some(ch) = context.iter.next() else {
            return;
        };

        match ch {
            '(' => {
                self.process_str_placeholder::<T>(context);
            }
            '<' => {
                self.process_format_left_placeholder::<T>(context);
            }
            '>' => {
                self.process_format_right_placeholder::<T>(context);
            }
            'n' => {
                T::process_char_placeholder(context, '\n');
            }
            '%' => {
                T::process_char_placeholder(context, '%');
            }
            _ => {
                T::error(context);
            }
        }
    }

    fn parse_generic<T: ParsingTask>(
        &self,
        key_value: &HashMap<&str, String>,
        inp: &str,
    ) -> T::Output {
        let mut context = T::init(inp, key_value);
        loop {
            let Some(ch) = context.iter.peek() else {
                break;
            };

            match ch {
                '%' => {
                    context.iter.mark(); // mark position of placeholder start
                    context.iter.next();
                    self.process_placeholder::<T>(&mut context);
                }
                _ => {
                    context.iter.next();
                    T::process_char(&mut context, ch);
                }
            }
        }
        T::done(context)
    }
}

impl PlaceholderFormatter for Formatify {
    fn replace_placeholders(&self, key_value: &HashMap<&str, String>, inp: &str) -> String {
        self.parse_generic::<ParsingTaskReplacePlaceholders>(key_value, inp)
    }

    fn measure_lengths(&self, key_value: &HashMap<&str, String>, inp: &str) -> Vec<usize> {
        self.parse_generic::<ParsingTaskMeasureLengths>(key_value, inp)
    }

    fn extract_placeholder_keys(&self, inp: &str) -> Vec<String> {
        let key_value = HashMap::<&str, String>::new();
        self.parse_generic::<ParsingTaskExtractPlaceholderKeys>(&key_value, inp)
    }
}

impl Default for Formatify {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests_extract_placeholder_keys {
    use crate::*;

    macro_rules! test {
        ($test_name:ident, $inp:expr, $expected_output:expr) => {
            #[test]
            fn $test_name() {
                let parser = Formatify::new();
                let out_str = parser.extract_placeholder_keys($inp);
                assert_eq!(out_str, $expected_output);
            }
        };
    }

    test!(
        test_with_empty_input_returns_empty_vec,
        "",
        Vec::<String>::new()
    );

    test!(
        test_with_plain_string_returns_empty_vec,
        "Conventional string",
        Vec::<String>::new()
    );

    test!(
        test_with_unicode_string_returns_empty_vec,
        "Smiley 😊 Smiley",
        Vec::<String>::new()
    );

    test!(
        test_with_single_placeholder_returns_one_placeholder,
        "Hello %(var1)", // replaces to "Hello world"
        vec!["var1"]
    );

    test!(
        test_with_multiple_placeholders_return_two_placeholders,
        "Hello %(var1). Hallo %(var2).", // "Hello world. Hallo welt."
        vec!["var1", "var2"]
    );

    test!(
        test_with_undefined_second_placeholder_returns_two_placeholders,
        "Hallo %(var1)%(vara)",
        vec!["var1", "vara"]
    );

    test!(
        test_with_incomplete_placeholder_syntax_returns_empty_vec,
        "Hallo %(var1",
        Vec::<String>::new()
    );
}

#[cfg(test)]
mod tests_measure_lengths {
    use std::collections::HashMap;

    use crate::*;

    macro_rules! test {
        ($test_name:ident, $inp:expr, $expected_output:expr) => {
            #[test]
            fn $test_name() {
                let mut key_value = HashMap::<&str, String>::new();
                key_value.insert("var1", "world".into());
                key_value.insert("var2", "welt".into());
                key_value.insert("str4", "1234".into());
                key_value.insert("str10", "1234567890".into());
                key_value.insert("str14", "1234567890ABCD".into());
                key_value.insert("umlaute", "äöü".into());
                key_value.insert("umlaute_bigger", "äöü12345678".into());
                let parser = Formatify::new();
                let out_str = parser.measure_lengths(&key_value, $inp);
                assert_eq!(out_str, $expected_output);
            }
        };
    }

    test!(test_with_empty_input_returns_vec0, "", vec![0usize]);
    test!(
        test_with_plain_string_returns_correct_length,
        "Conventional string",
        vec![19usize]
    );

    test!(
        test_with_unicode_string_returns_correct_length,
        "Smiley 😊 Smiley",
        vec![15usize]
    );

    test!(
        test_with_single_placeholder_measures_correctly,
        "Hello %(var1)", // replaces to "Hello world"
        vec![11usize, 5usize]
    );

    test!(
        test_with_invalid_token_type_counts_length_of_unreplaced_string,
        "Hallo %z", // replaces nothing -> "Hallo %z"
        vec![8usize]
    );

    test!(
        test_with_one_char_token_type_counts_length_of_replaced_char,
        "abcde %%", // replaces nothing -> "abcde %"
        vec![7usize]
    );

    test!(
        test_with_multiple_placeholders_return_correct_length_of_string_and_placeholders,
        "Hello %(var1). Hallo %(var2).", // "Hello world. Hallo welt."
        vec![24usize, 5usize, 4usize]
    );

    test!(
        test_with_undefined_second_placeholder_return_correct_length_of_string_and_placeholders,
        "Hallo %(var1)%(vara)", // "Hallo world%(vara)"
        vec![18usize, 5usize]
    );

    test!(
        test_with_left_alignment_placeholder_and_shorter_value_returns_correct_length,
        "Hallo %<(10)%(str4)xx", // "Hallo 1234      xx"
        vec![18usize, 10usize]
    );

    test!(
        test_with_right_alignment_placeholder_and_shorter_returns_correct_length,
        "Hallo %>(10)%(str4)xx", // "Hallo       1234xx"
        vec![18usize, 10usize]
    );

    test!(
        test_with_left_alignment_placeholder_and_exact_length_value_returns_correct_length,
        "Hallo %<(10)%(str10)xx", // "Hallo 1234567890xx"
        vec![18usize, 10usize]
    );

    test!(
        test_with_right_alignment_placeholder_and_exact_length_value_returns_correct_length,
        "Hallo %>(10)%(str10)xx", // "Hallo 1234567890xx"
        vec![18usize, 10usize]
    );

    test!(
        test_with_left_alignment_placeholder_and_longer_value_returns_correct_length,
        "Hallo %<(10)%(str14)xx", // "Hallo 1234567890ABCDxx"
        vec![22usize, 14usize]
    );

    test!(
        test_with_right_alignment_placeholder_and_longer_value_returns_correct_length,
        "Hallo %>(10)%(str14)xx", // "Hallo 1234567890ABCDxx"
        vec![22usize, 14usize]
    );

    test!(
        test_with_right_align_truncate_placeholder_and_shorter_value_with_umlauts_returns_correct_length,
        "Hallo %>(10,trunc)%(umlaute)xx", // "Hallo äöü       xx"
        vec![18usize, 10usize]
    );

    test!(
        test_with_left_align_truncate_placeholder_and_shorter_value_with_umlauts_returns_correct_length,
        "Hallo %<(10,trunc)%(umlaute)xx", // "Hallo äöü       xx"
        vec![18usize, 10usize]
    );

    test!(
        test_with_left_align_truncate_placeholder_and_exact_length_value_returns_correct_length,
        "Hallo %<(10,trunc)%(str10)xx", // "Hallo 1234567890xx"
        vec![18usize, 10usize]
    );

    test!(
        test_with_left_align_truncate_placeholder_and_longer_value_returns_correct_length,
        "Hallo %<(10,trunc)%(str14)xx", // "Hallo 123456789…xx"
        vec![18usize, 10usize]
    );

    test!(
        test_with_right_align_truncate_placeholder_and_longer_value_returns_correct_length,
        "Hallo %>(10,trunc)%(str14)xx", // "Hallo 123456789…xx"
        vec![18usize, 10usize]
    );
}

#[cfg(test)]
mod tests_replace_placeholders {
    use crate::*;
    use std::collections::HashMap;

    macro_rules! test {
        ($test_name:ident, $inp:expr, $expected_output:expr) => {
            #[test]
            fn $test_name() {
                let mut key_value = HashMap::<&str, String>::new();
                key_value.insert("var1", "world".into());
                key_value.insert("var2", "welt".into());
                key_value.insert("str4", "1234".into());
                key_value.insert("str10", "1234567890".into());
                key_value.insert("str14", "1234567890ABCD".into());
                key_value.insert("umlaute", "äöü".into());
                key_value.insert("umlaute_bigger", "äöü12345678".into());
                let parser = Formatify::new();
                let out_str = parser.replace_placeholders(&key_value, $inp);
                assert_eq!(out_str, $expected_output);
            }
        };
    }

    test!(test_with_empty_input_returns_empty_string, "", "");

    test!(
        test_with_plain_string_returns_same_string,
        "Conventional string",
        "Conventional string"
    );

    test!(
        test_with_unicode_string_returns_same_string,
        "Smiley 😊 Smiley",
        "Smiley 😊 Smiley"
    );

    test!(
        test_with_single_placeholder_replaces_correctly,
        "Hello %(var1)",
        "Hello world"
    );

    test!(
        test_with_single_placeholder_alternative_value_replaces_correctly,
        "Hello %(var2)",
        "Hello welt"
    );

    test!(
        test_with_one_char_token_type_replaces_correctly,
        "abcde %%", // replaces nothing -> "abcde %"
        "abcde %"
    );

    test!(
        test_with_invalid_token_type_leaves_token_unreplaced,
        "Hallo %z",
        "Hallo %z"
    );

    test!(
        test_with_multiple_placeholders_replaces_all_correctly,
        "Hello %(var1). Hallo %(var2).",
        "Hello world. Hallo welt."
    );

    test!(
        test_with_multiple_placeholders_and_delimiters_replaces_correctly,
        "|%(var1)|%(var2)|",
        "|world|welt|"
    );

    test!(
        test_with_undefined_second_placeholder_keeps_it_unreplaced,
        "Hallo %(var1)%(vara)",
        "Hallo world%(vara)"
    );

    test!(
        test_with_undefined_first_placeholder_keeps_it_unreplaced,
        "Hallo %(vara)%(var2)",
        "Hallo %(vara)welt"
    );

    test!(
        test_with_incorrect_placeholder_syntax_keeps_it_unreplaced,
        "Hallo %var1",
        "Hallo %var1"
    );

    test!(
        test_with_incomplete_placeholder_syntax_keeps_it_unreplaced,
        "Hallo %(var1",
        "Hallo %(var1"
    );

    test!(
        test_with_newline_placeholder_inserts_newline,
        "Hallo %nWelt",
        "Hallo \nWelt"
    );

    test!(
        test_with_escaped_percent_sign_keeps_it_unchanged,
        "Hallo %%(var1)",
        "Hallo %(var1)"
    );

    test!(
        test_with_newline_placeholder_at_end_inserts_newline,
        "Hallo Welt %n",
        "Hallo Welt \n"
    );

    test!(
        test_with_left_alignment_placeholder_and_shorter_value_pads_correctly,
        "Hallo %<(10)%(str4)xx",
        "Hallo 1234      xx"
    );

    test!(
        test_with_right_alignment_placeholder_and_shorter_value_pads_correctly,
        "Hallo %>(10)%(str4)xx",
        "Hallo       1234xx"
    );

    test!(
        test_with_left_alignment_placeholder_and_exact_length_value_keeps_it_unchanged,
        "Hallo %<(10)%(str10)xx",
        "Hallo 1234567890xx"
    );

    test!(
        test_with_right_alignment_placeholder_and_exact_length_value_keeps_it_unchanged,
        "Hallo %>(10)%(str10)xx",
        "Hallo 1234567890xx"
    );

    test!(
        test_with_left_alignment_placeholder_and_longer_value_keeps_it_unchanged,
        "Hallo %<(10)%(str14)xx",
        "Hallo 1234567890ABCDxx"
    );

    test!(
        test_with_right_alignment_placeholder_and_longer_value_keeps_it_unchanged,
        "Hallo %>(10)%(str14)xx",
        "Hallo 1234567890ABCDxx"
    );

    test!(
        test_with_left_align_truncate_placeholder_and_exact_length_value_keeps_it_unchanged,
        "Hallo %<(10,trunc)%(str10)xx",
        "Hallo 1234567890xx"
    );

    test!(
        test_with_right_align_truncate_placeholder_and_exact_length_value_keeps_it_unchanged,
        "Hallo %>(10,trunc)%(str10)xx",
        "Hallo 1234567890xx"
    );

    test!(
        test_with_left_align_truncate_placeholder_and_exact_length_value_with_spaces_keeps_it_unchanged,
        "Hallo %<(  10  ,  trunc   )%(str10)xx",
        "Hallo 1234567890xx"
    );

    test!(
        test_with_right_align_truncate_placeholder_and_longer_value_truncates_correctly,
        "Hallo %>(10,trunc)%(str14)xx",
        "Hallo 123456789…xx"
    );

    test!(
        test_with_right_align_left_truncate_placeholder_and_longer_value_truncates_correctly,
        "Hallo %>(10,ltrunc)%(str14)xx",
        "Hallo …67890ABCDxx"
    );

    test!(
        test_with_left_align_truncate_placeholder_and_longer_value_truncates_correctly,
        "Hallo %<(10,trunc)%(str14)xx",
        "Hallo 123456789…xx"
    );

    test!(
        test_with_right_align_truncate_placeholder_and_shorter_value_with_umlauts_pads_correctly,
        "Hallo %>(10,trunc)%(umlaute)xx",
        "Hallo        äöüxx"
    );

    test!(
        test_with_left_align_truncate_placeholder_and_shorter_value_with_umlauts_pads_correctly,
        "Hallo %<(10,trunc)%(umlaute)xx",
        "Hallo äöü       xx"
    );

    test!(
        test_with_left_align_truncate_placeholder_and_longer_value_with_umlauts_truncates_correctly,
        "Hallo %<(10,trunc)%(umlaute_bigger)xx",
        "Hallo äöü123456…xx"
    );

    test!(
        test_with_invalid_left_align_placeholder_keeps_format_specifier_unchanged,
        "Hallo %<(a10)%(str14)xx",
        "Hallo %<(a10)1234567890ABCDxx"
    );
}
