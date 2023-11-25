# Formatify

Formatify is a rust library designed for dynamic string formatting. It offers flexible and powerful tools for parsing strings with placeholders and replacing them with corresponding values. The library's key feature is its ability to handle various placeholder formats and alignment options.

## Usage

Import the necessary modules and use `Formatify` for string formatting tasks:

```rust
use formatify::Formatify;
use std::collections::HashMap;
```

## Features

1. **Placeholder Replacement**: Replace placeholders in strings with values from a `HashMap`.
2. **Length Measurement**: Measure lengths of strings and placeholders.
3. **Placeholder Extraction**: Extract placeholder keys from a string.

## Examples

### Replacing Placeholders

```rust
let mut key_value: HashMap<&str, String> = HashMap::new();
key_value.insert("name", "Alice".into());
let formatter = Formatify::new();
let formatted_string = formatter.replace_placeholders(&key_value, "Hello, %(name)!");
assert_eq!(formatted_string, "Hello, Alice!");
```

### Measuring Lengths

```rust
let segment_lengths = formatter.measure_lengths(&key_value, "Hello, %(name)! This is a test.");
assert_eq!(segment_lengths, vec![29, 5]); // Total length with "Alice" as the placeholder, length of "Alice"
```

### Extracting Placeholder Keys

```rust
let placeholder_keys = formatter.extract_placeholder_keys(&key_value, "Hello, %(name)! Today is %(day).");
assert_eq!(placeholder_keys, vec!["name"]);
```

## Testing

The library includes comprehensive tests for each functionality, ensuring reliability and correctness.

## Conclusion

Formatify offers a versatile solution for dynamic string formatting in Rust, suitable for various applications requiring template-based string manipulation.
