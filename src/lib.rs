//! Formatify is a rust library designed for dynamic string formatting. It offers flexible
//! and powerful tools for parsing strings with placeholders and replacing them with
//! corresponding values. The library's key feature is its ability to handle various placeholder
//! formats and alignment options.

mod output_format;
mod parsing_context;
mod parsing_task;
mod parsing_task_extract_placeholder_keys;
mod parsing_task_measure_lengths;
mod parsing_task_replace_placeholders;
mod peek_char_iterator;

use self::output_format::OutputFormat;
use self::parsing_context::ParsingContext;
use self::parsing_task::ParsingTask;
use self::parsing_task_extract_placeholder_keys::ParsingTaskExtractPlaceholderKeys;
use self::parsing_task_measure_lengths::ParsingTaskMeasureLengths;
use self::parsing_task_replace_placeholders::ParsingTaskReplacePlaceholders;
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

/// `Formatify` is a struct in the formatify library, a versatile string formatting tool in Rust.
/// It provides methods for replacing placeholders in a string with values from a `HashMap`,
/// measuring the lengths of strings with placeholders, and extracting placeholder keys from a string.
/// This struct is the core of the library, enabling users to perform various string formatting tasks
/// efficiently. With methods like `replace_placeholders`, `measure_lengths`, and `extract_placeholder_keys`,
/// Formatify simplifies the process of dynamic string manipulation, catering to use cases where
/// template-based string formatting is essential.
///
/// ## Usage
///
/// Import the necessary modules and use `Formatify` for string formatting tasks:
///
/// ```rust
/// use formatify::Formatify;
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
/// # use formatify::Formatify;
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
/// # use formatify::Formatify;
/// # use std::collections::HashMap;
/// let mut key_value: HashMap<&str, String> = HashMap::new();
/// key_value.insert("name", "Alice".into());
/// let formatter = Formatify::new();
/// let segment_lengths = formatter.measure_lengths(&key_value, "Hello, %(name)! This is a test.");
/// assert_eq!(segment_lengths, vec![29, 5]); // Total length with "Alice" as the placeholder, length of "Alice"
/// ```
///
/// ### Extracting Placeholder Keys
///
/// ```rust
/// # use formatify::Formatify;
/// # use std::collections::HashMap;
/// let mut key_value: HashMap<&str, String> = HashMap::new();
/// key_value.insert("name", "Alice".into());
/// let formatter = Formatify::new();
/// let placeholder_keys = formatter.extract_placeholder_keys(&key_value, "Hello, %(name)! Today is %(day).");
/// assert_eq!(placeholder_keys, vec!["name"]);
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

    /// Replaces placeholders in the input string with corresponding values from a HashMap.
    ///
    /// This method scans the input string (`inp`) for placeholders, identified by a specific
    /// syntax (e.g., "%(var1)"), and replaces them with corresponding values from the
    /// `key_value` HashMap. The replacement process is versatile, accommodating various placeholder
    /// formats, including those requiring left alignment and truncation. It's ideal for dynamically
    /// generating strings where placeholders are replaced by context-specific values.
    ///
    /// # Arguments
    /// * `key_value` - A reference to a HashMap where keys correspond to placeholder identifiers in the input string and values are their replacements.
    /// * `inp` - The input string containing placeholders.
    ///
    /// # Returns
    /// A new `String` with placeholders replaced by their respective values from the `key_value` HashMap.
    /// If a placeholder has no corresponding value in the map, it remains unchanged in the output string.
    ///
    /// # Examples
    /// ```
    /// # use std::collections::HashMap;
    /// # use formatify::Formatify;
    /// let mut key_value : HashMap<&str, String> = HashMap::new();
    /// key_value.insert("name", "Alice".into());
    /// let formatter = Formatify::new();
    /// let formatted_string = formatter.replace_placeholders(&key_value, "Hello, %(name)!");
    /// assert_eq!(formatted_string, "Hello, Alice!");
    /// ```
    pub fn replace_placeholders(&self, key_value: &HashMap<&str, String>, inp: &str) -> String {
        self.parse_generic::<ParsingTaskReplacePlaceholders>(key_value, inp)
    }

    /// Measures the length of the entire string and the lengths of valid placeholders within it.
    ///
    /// This method processes the input string (`inp`), which is analyzed as if it were to be formatted.
    /// Instead of replacing the placeholders, it calculates the overall length of the string with
    /// placeholders hypothetically replaced, followed by the lengths of each valid placeholder. This
    /// is particularly useful for layout planning and understanding the impact of placeholders on the
    /// total length of the string.
    ///
    /// # Arguments
    /// * `key_value` - A reference to a HashMap containing key-value pairs. The keys represent placeholders in the input string, and the values are their potential replacements.
    /// * `inp` - The input string with placeholders to be measured.
    ///
    /// # Returns
    /// A `Vec<usize>` where the first element represents the length of the entire string with placeholders
    /// replaced, and subsequent elements represent the lengths of each valid placeholder. Placeholders
    /// are replaced with their corresponding values from the `key_value` HashMap for these calculations.
    ///
    /// # Examples
    /// ```
    /// # use std::collections::HashMap;
    /// # use formatify::Formatify;
    /// let mut key_value : HashMap<&str, String> = HashMap::new();
    /// key_value.insert("name", "Alice".into());
    /// let formatter = Formatify::new();
    /// let segment_lengths = formatter.measure_lengths(&key_value, "Hello, %(name)! This is a test.");
    /// assert_eq!(segment_lengths, vec![29, 5]); // Total length with "Alice" as the placeholder, length of "Alice"
    /// ```
    pub fn measure_lengths(&self, key_value: &HashMap<&str, String>, inp: &str) -> Vec<usize> {
        self.parse_generic::<ParsingTaskMeasureLengths>(key_value, inp)
    }

    /// Extracts and lists all valid placeholder keys from a given string.
    ///
    /// This method analyzes the input string (`inp`) to identify and collect the keys of all
    /// placeholders defined within it. Placeholders are identified by a specific syntax, typically
    /// denoted by "%(key)". This function is particularly useful for determining which placeholders
    /// are used in a string without modifying the string itself. It helps in preparing or validating
    /// the necessary keys in a key-value map for subsequent processing, like formatting or replacing
    /// placeholders.
    ///
    /// # Arguments
    /// * `key_value` - A reference to a HashMap containing key-value pairs. These pairs may be used within the placeholder syntax in the input string.
    /// * `inp` - The input string to be analyzed for placeholder keys.
    ///
    /// # Returns
    /// A `Vec<String>` containing all distinct placeholder keys found in the input string. If no
    /// placeholders are found, an empty vector is returned.
    ///
    /// # Examples
    /// ```
    /// # use std::collections::HashMap;
    /// # use formatify::Formatify;
    /// let mut key_value : HashMap<&str, String> = HashMap::new();
    /// key_value.insert("name", "Alice".into());
    /// let formatter = Formatify::new();
    /// let placeholder_keys = formatter.extract_placeholder_keys(&key_value, "Hello, %(name)! Today is %(day).");
    /// assert_eq!(placeholder_keys, vec!["name"]);
    /// ```
    pub fn extract_placeholder_keys(
        &self,
        key_value: &HashMap<&str, String>,
        inp: &str,
    ) -> Vec<String> {
        self.parse_generic::<ParsingTaskExtractPlaceholderKeys>(key_value, inp)
    }
}

#[cfg(test)]
mod tests_extract_placeholder_keys {
    use std::collections::HashMap;

    use crate::Formatify;

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
                let out_str = parser.extract_placeholder_keys(&key_value, $inp);
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
        vec!["var1"]
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

    use crate::Formatify;

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
        "Hallo %z", // replaces nothing
        vec![8usize]
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
        test_with_left_alignment_placeholder_and_exact_length_value_returns_correct_length,
        "Hallo %<(10)%(str10)xx", // "Hallo 1234567890xx"
        vec![18usize, 10usize]
    );

    test!(
        test_with_left_alignment_placeholder_and_longer_value_returns_correct_length,
        "Hallo %<(10)%(str14)xx", // "Hallo 1234567890ABCDxx"
        vec![22usize, 14usize]
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
}

#[cfg(test)]
mod tests_replace_placeholders {
    use crate::Formatify;
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
        test_with_left_alignment_placeholder_and_exact_length_value_keeps_it_unchanged,
        "Hallo %<(10)%(str10)xx",
        "Hallo 1234567890xx"
    );

    test!(
        test_with_left_alignment_placeholder_and_longer_value_keeps_it_unchanged,
        "Hallo %<(10)%(str14)xx",
        "Hallo 1234567890ABCDxx"
    );

    test!(
        test_with_left_align_truncate_placeholder_and_exact_length_value_keeps_it_unchanged,
        "Hallo %<(10,trunc)%(str10)xx",
        "Hallo 1234567890xx"
    );

    test!(
        test_with_left_align_truncate_placeholder_and_exact_length_value_with_spaces_keeps_it_unchanged,
        "Hallo %<(  10  ,  trunc   )%(str10)xx",
        "Hallo 1234567890xx"
    );

    test!(
        test_with_left_align_truncate_placeholder_and_longer_value_truncates_correctly,
        "Hallo %<(10,trunc)%(str14)xx",
        "Hallo 123456789…xx"
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
