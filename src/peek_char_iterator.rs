/// A char iterator with peek, mark, and backtrack functionalities.
///
/// This iterator operates on a `Vec<char>` and uses indices
/// to mark positions and to return to previous states.
pub struct PeekCharIterator {
    // The vector of characters to iterate over.
    chars: Vec<char>,
    // The current index in the vector.
    current_index: usize,
    // An optional index for the peeked character.
    peeked_index: Option<usize>,
    // An optional index marking a saved position in the vector.
    marked_index: Option<usize>,
}

impl PeekCharIterator {
    /// Creates a new `PeekCharIterator` for a given `Vec<char>`.
    ///
    /// # Arguments
    ///
    /// * `chars` - The `Vec<char>` to iterate over.
    pub fn new(chars: Vec<char>) -> Self {
        PeekCharIterator {
            chars,
            current_index: 0,
            peeked_index: None,
            marked_index: None,
        }
    }

    /// Peeks at the next character without changing the iterator's state.
    pub fn peek(&mut self) -> Option<char> {
        if self.peeked_index.is_none() {
            self.peeked_index = Some(self.current_index);
        }

        self.chars.get(self.peeked_index.unwrap()).copied()
    }

    /// Marks the current position in the iterator.
    pub fn mark(&mut self) {
        self.marked_index = Some(self.current_index);
    }

    /// Returns a vector of chars between the mark and the current position
    pub fn get_mark2cur(&self) -> Option<Vec<char>> {
        self.marked_index
            .map(|marked_index| self.chars[marked_index..self.current_index].to_vec())
    }
}

impl Iterator for PeekCharIterator {
    type Item = char;

    /// Returns the next character in the iterator.
    ///
    /// If `peek` was previously called, it returns the peeked character and advances the iterator.
    /// Otherwise, it fetches the next character from the vector.
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.peeked_index.take() {
            self.current_index = index + 1;
            return self.chars.get(index).copied();
        }

        let result = self.chars.get(self.current_index).copied();
        self.current_index += 1;
        result
    }
}
