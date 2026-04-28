use tracing::Span;
use unicode_segmentation::UnicodeSegmentation;

pub fn count_string_length(s: String) -> usize {
    count_graphemes(s)
}
pub fn count_graphemes(s: String) -> usize {
    s.graphemes(true).count()
}

#[allow(unused_assignments)]
#[tracing::instrument(fields(bytes_count = tracing::field::Empty, grapheme_count = tracing::field::Empty, max_byte_allowance = tracing::field::Empty, result = tracing::field::Empty))]
pub fn valid_string_length(s: String, min_graphemes: usize, max_graphemes: usize) -> bool {
   // I don't actually assume that most graphemes contain 8 bytes. But feels like a fine number. For example, if user is signing up for an acoount and enters 50 characters in username fields, it seems fine to allow up to 400 bytes but no more
   let mut result = true;
    let an_almost_arbitrary_number = 8;
    {
        let bytes_count = s.len();
        let grapheme_count = count_graphemes(s.clone());
        let max_byte_allowance = max_graphemes * an_almost_arbitrary_number;
        Span::current().record("bytes_count", bytes_count);
        Span::current().record("grapheme_count", grapheme_count);
        Span::current().record("max_byte_allowance", max_byte_allowance);

        if bytes_count > max_byte_allowance  {
            result = false;
        }
        if grapheme_count > max_graphemes {
            result = false;
        }
        if grapheme_count < min_graphemes {
            result = false;
        }
    }
    Span::current().record("result", result);
    tracing::debug!("Debug dump");
    result
}