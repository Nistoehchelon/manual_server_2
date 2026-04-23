use unicode_segmentation::UnicodeSegmentation;

pub fn count_string_length(s: String) -> usize {
    count_graphemes(s)
}
pub fn count_graphemes(s: String) -> usize {
    s.graphemes(true).count()
}
pub fn validate_string_length(s: String, min_graphemes: usize, max_graphemes: usize) -> bool {
   // I don't actually assume that most graphemes contain 8 bytes. But feels like a fine number. For example, if user is signing up for an acoount and enters 50 characters in username fields, it seems fine to allow up to 400 bytes but no more
    let an_almost_arbitrary_number = 8;
    {
        let bytes_count = s.len();
        let grapheme_count = count_graphemes(s.clone());
        let max_byte_allowance = max_graphemes * an_almost_arbitrary_number;
        if bytes_count > max_byte_allowance  {
            return false;
        }
        if grapheme_count > max_graphemes {
            return false;
        }
        if grapheme_count < min_graphemes {
            return false;
        }
    }
    true
}