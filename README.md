# Formatify: Dynamic String Formatting Library for Rust

Formatify is a Rust library tailored for flexible string formatting. It takes inspiration from Git's commit log formatting system (see [here](https://git-scm.com/docs/pretty-formats)). This library provides robust tools to parse strings containing placeholders and replace them with appropriate values. It's an excellent choice for applications needing dynamic text representations.

Explore more in our [documentation](https://docs.rs/formatify).

## Key Features

- **Dynamic String Formatting:** Simplify your coding by replacing placeholders in strings with matching values from a HashMap.
- **Alignment Varieties:** Manage various text alignments like left alignment, and easily handle text truncation.
- **String and Placeholder Lengths:** Efficiently measure the lengths of both strings and individual placeholders.
- **Extracting Placeholders:** Quickly identify and extract all the placeholder keys present in a string.

## How to Use Formatify

To get started with Formatify, here's a simple example:

```rust
use formatify::{Formatify, FormatifiyFormatter};
use std::collections::HashMap;

let mut key_value_pairs = HashMap::new();
key_value_pairs.insert("name", "Alice".into());
let formatter = Formatify::new();
let formatted_string = formatter.replace_placeholders(&key_value_pairs, "Hello, %(name)!");
assert_eq!(formatted_string, "Hello, Alice!");
```

In this example, we create a HashMap with key-value pairs, initialize Formatify, and replace the placeholder `%(name)` with the corresponding value from the HashMap.

## Library Methods

Formatify includes several helpful methods:
- `replace_placeholders`: Substitutes placeholders in a string with their corresponding values from a HashMap.
- `measure_lengths`: Calculates the length of a string and the lengths of each placeholder it contains.
- `extract_placeholder_keys`: Identifies and lists all the placeholders in a string.

For detailed information on these methods, check out our [module documentation](https://docs.rs/formatify).

## Easy Integration

Formatify is built to integrate smoothly into your existing Rust projects. It is compatible with standard Rust data types and collection frameworks.

## Contributing

Contributions are welcome! For bug reports, feature requests, or general feedback, please open an issue on the repository's issue tracker.
