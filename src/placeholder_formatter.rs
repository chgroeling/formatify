use std::collections::HashMap;

/// Trait used to abstract Formatify from a system.
pub trait PlaceholderFormatter {
    /// Replaces placeholders in the input string with corresponding values from a HashMap.
    ///
    /// This method scans the input string `inp` for placeholders, identified by a specific
    /// syntax, and replaces them with corresponding values from the `key_value` HashMap. The function
    /// supports various types of placeholders, including simple variable substitution, alignment,
    /// and optional truncation.
    ///
    /// For detailed information on supported placeholders, see [Supported Placeholder Types](#supported-placeholder-types).
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
    /// # use formatify::{Formatify, PlaceholderFormatter};
    /// # use std::collections::HashMap;
    /// let mut key_value : HashMap<&str, String> = HashMap::new();
    /// key_value.insert("name", "Alice".into());
    /// key_value.insert("date", "Monday".into());
    /// let formatter = Formatify::new();
    /// let formatted_string = formatter.replace_placeholders(&key_value, "Hello, %(name)! Today is %<(10)%(date).");
    /// assert_eq!(formatted_string, "Hello, Alice! Today is Monday    .");
    /// ```
    ///
    /// This function is essential for dynamic string formatting in the Formatify library. It allows users
    /// to create template strings with various types of placeholders, which can be filled with different values at runtime.
    /// This is particularly useful for generating customized messages, dynamic user interfaces, or any other text-based content
    /// that needs to be generated or modified based on changing data.
    fn replace_placeholders(&self, key_value: &HashMap<&str, String>, inp: &str) -> String;

    /// Measures the length of the entire string and the lengths of valid placeholders within it.
    ///
    /// This method processes the input string `inp`, which is analyzed as if it were to be formatted.
    /// Instead of replacing the placeholders, it calculates the overall length of the string with
    /// placeholders hypothetically replaced, followed by the lengths of each valid placeholder. This
    /// is particularly useful for layout planning and understanding the impact of placeholders on the
    /// total length of the string.
    ///
    /// For detailed information on supported placeholders, see [Supported Placeholder Types](#supported-placeholder-types).
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
    /// # use formatify::{Formatify, PlaceholderFormatter};
    /// # use std::collections::HashMap;
    /// let mut key_value : HashMap<&str, String> = HashMap::new();
    /// key_value.insert("name", "Alice".into());
    /// let formatter = Formatify::new();
    /// let lengths = formatter.measure_lengths(&key_value, "Hello, %(name)! This is a test.");
    /// assert_eq!(lengths, vec![29, 5]); // Total length with "Alice" as the placeholder, length of "Alice"
    /// ```
    fn measure_lengths(&self, key_value: &HashMap<&str, String>, inp: &str) -> Vec<usize>;

    /// Extracts and lists all placeholder keys from a given string.
    ///
    /// This method analyzes the input string `inp` to identify and collect the keys of all
    /// placeholders defined within it. Placeholders are identified by a specific syntax, typically
    /// denoted by `%(key)`. This function is particularly useful for determining which placeholders
    /// are used in a string without modifying the string itself. It helps in preparing or validating
    /// the necessary keys in a key-value map for subsequent processing, like formatting or replacing
    /// placeholders. Single char and formatting placeholders are ignored by this function.
    ///  
    /// For detailed information on supported placeholders, see [Supported Placeholder Types](#supported-placeholder-types).
    ///
    /// # Arguments
    /// * `inp` - The input string to be analyzed for placeholder keys.
    ///
    /// # Returns
    /// A `Vec<String>` containing all placeholder keys found in the input string. If no
    /// valid placeholders are found, an empty vector is returned.
    ///
    /// # Examples
    /// ```
    /// # use formatify::{Formatify, PlaceholderFormatter};
    /// let formatter = Formatify::new();
    /// let placeholder_keys = formatter.extract_placeholder_keys("Hello, %(name)! Today is %(day).");
    /// assert_eq!(placeholder_keys, vec!["name", "day"]);
    /// ```
    fn extract_placeholder_keys(&self, inp: &str) -> Vec<String>;
}
