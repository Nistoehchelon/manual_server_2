use unicode_segmentation::UnicodeSegmentation;

pub fn count_string_length(s: String) -> usize {
    s.graphemes(true).count()
}