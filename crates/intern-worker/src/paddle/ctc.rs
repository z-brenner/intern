//! Text from the recognizer's per-column character probabilities.
//!
//! The recognizer reads a line as a sequence of narrow columns and gives,
//! for each, a probability for every character it knows plus a "blank" for
//! no character at all. Greedy CTC decoding takes the likeliest class in
//! each column, merges a class repeated across neighbouring columns into
//! one character, and drops the blanks: "--HH-ee-ll-ll-oo" reads "Hello".
//! What it keeps for each character is the probability it was read with,
//! which is what decides whether a line is worth reading again.

/// The class list the recognizer's output columns index: the blank, then
/// the dictionary in order, then a space.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dictionary {
    symbols: Vec<String>,
}

impl Dictionary {
    /// One symbol per line, in class order, as the model's configuration
    /// lists them. Empty lines are ignored; a trailing newline is not a
    /// symbol.
    pub fn from_lines(text: &str) -> Self {
        let symbols = text
            .split('\n')
            .map(|line| line.strip_suffix('\r').unwrap_or(line))
            .filter(|line| !line.is_empty())
            .map(str::to_owned)
            .collect();
        Self { symbols }
    }

    /// Classes the recognizer must output: the blank, every symbol, and
    /// the space.
    pub fn classes(&self) -> usize {
        self.symbols.len() + 2
    }

    /// The text class `index` stands for. The blank is `None`.
    fn symbol(&self, index: usize) -> Option<&str> {
        match index {
            0 => None,
            index if index <= self.symbols.len() => Some(&self.symbols[index - 1]),
            _ => Some(" "),
        }
    }
}

/// One line as decoded.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DecodedLine {
    pub text: String,
    /// The probability each character of `text` was read with, in order.
    /// A space between words counts as a character, as it is one.
    pub char_probabilities: Vec<f32>,
}

impl DecodedLine {
    /// The mean probability over the line's characters, which is the
    /// reference recognizer's line score. Nothing read is no confidence.
    pub fn mean_probability(&self) -> f32 {
        if self.char_probabilities.is_empty() {
            0.0
        } else {
            self.char_probabilities.iter().sum::<f32>() / self.char_probabilities.len() as f32
        }
    }

    /// The least certain character, which is where a misread hides.
    pub fn min_probability(&self) -> f32 {
        self.char_probabilities
            .iter()
            .copied()
            .fold(f32::INFINITY, f32::min)
            .min(1.0)
    }

    /// Leading and trailing spaces dropped, with their probabilities.
    pub fn trimmed(mut self) -> Self {
        let chars: Vec<char> = self.text.chars().collect();
        if chars.len() != self.char_probabilities.len() {
            self.text = self.text.trim().to_owned();
            return self;
        }
        let start = chars
            .iter()
            .position(|c| !c.is_whitespace())
            .unwrap_or(chars.len());
        let end = chars
            .iter()
            .rposition(|c| !c.is_whitespace())
            .map_or(start, |index| index + 1);
        self.text = chars[start..end].iter().collect();
        self.char_probabilities = self.char_probabilities[start..end].to_vec();
        self
    }
}

/// Greedy decoding of one line's `steps` x `classes` probabilities, row
/// major.
///
/// A class repeated in consecutive columns is one character, read with
/// the probability of the first column of the run. A blank between two
/// equal classes separates them, which is how "ll" survives.
pub fn greedy_decode(
    probabilities: &[f32],
    steps: usize,
    classes: usize,
    dictionary: &Dictionary,
) -> DecodedLine {
    let mut line = DecodedLine::default();
    let mut previous: Option<usize> = None;
    for step in 0..steps {
        let row = &probabilities[step * classes..(step + 1) * classes];
        let (best, probability) =
            row.iter()
                .copied()
                .enumerate()
                .fold((0, f32::MIN), |(best, top), (index, value)| {
                    if value > top {
                        (index, value)
                    } else {
                        (best, top)
                    }
                });
        if previous != Some(best)
            && let Some(symbol) = dictionary.symbol(best)
        {
            for character in symbol.chars() {
                line.text.push(character);
                line.char_probabilities.push(probability);
            }
        }
        previous = Some(best);
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dictionary() -> Dictionary {
        Dictionary::from_lines("H\ne\nl\no\n0\n1\n")
    }

    /// One column whose likeliest class is `class`, at `probability`.
    fn column(class: usize, probability: f32, classes: usize) -> Vec<f32> {
        let rest = (1.0 - probability) / (classes - 1) as f32;
        (0..classes)
            .map(|index| if index == class { probability } else { rest })
            .collect()
    }

    #[test]
    fn the_dictionary_is_bracketed_by_the_blank_and_the_space() {
        let dictionary = dictionary();
        assert_eq!(dictionary.classes(), 8);
        assert_eq!(dictionary.symbol(0), None);
        assert_eq!(dictionary.symbol(1), Some("H"));
        assert_eq!(dictionary.symbol(6), Some("1"));
        assert_eq!(dictionary.symbol(7), Some(" "));
        // A Windows line ending is not part of a symbol.
        assert_eq!(Dictionary::from_lines("a\r\nb\r\n").classes(), 4);
    }

    #[test]
    fn repeats_merge_blanks_separate_and_spaces_are_kept() {
        let dictionary = dictionary();
        let classes = dictionary.classes();
        // - H H e - l l - l o o (space) 1 1 -
        let sequence = [0, 1, 1, 2, 0, 3, 3, 0, 3, 4, 4, 7, 6, 6, 0];
        let probabilities: Vec<f32> = sequence
            .iter()
            .enumerate()
            .flat_map(|(step, &class)| column(class, 0.5 + step as f32 / 100.0, classes))
            .collect();

        let line = greedy_decode(&probabilities, sequence.len(), classes, &dictionary);

        assert_eq!(line.text, "Hello 1");
        // Each character keeps the probability of the first column of its run.
        assert_eq!(line.char_probabilities.len(), 7);
        assert!((line.char_probabilities[0] - 0.51).abs() < 1e-6);
        assert!((line.char_probabilities[2] - 0.55).abs() < 1e-6);
        assert!((line.char_probabilities[3] - 0.58).abs() < 1e-6);
        assert!((line.min_probability() - 0.51).abs() < 1e-6);
    }

    #[test]
    fn nothing_read_is_empty_and_unconfident() {
        let dictionary = dictionary();
        let classes = dictionary.classes();
        let probabilities: Vec<f32> = (0..5).flat_map(|_| column(0, 0.99, classes)).collect();
        let line = greedy_decode(&probabilities, 5, classes, &dictionary);
        assert_eq!(line.text, "");
        assert_eq!(line.mean_probability(), 0.0);
    }

    #[test]
    fn trimming_drops_edge_spaces_with_their_probabilities() {
        let line = DecodedLine {
            text: " Hi ".to_owned(),
            char_probabilities: vec![0.1, 0.9, 0.8, 0.2],
        }
        .trimmed();
        assert_eq!(line.text, "Hi");
        assert_eq!(line.char_probabilities, vec![0.9, 0.8]);
    }
}
