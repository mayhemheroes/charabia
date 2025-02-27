use deunicode::deunicode_char;
use fst::Set;

use crate::segmenter::SegmentedTokenIter;
use crate::token::SeparatorKind;
use crate::{Token, TokenKind};

/// Iterator over classified [`Token`]s.
pub struct ClassifiedTokenIter<'o, 'al, 'sw, A> {
    inner: SegmentedTokenIter<'o, 'al>,
    classifier: TokenClassifier<'sw, A>,
}

impl<'o, A: AsRef<[u8]>> Iterator for ClassifiedTokenIter<'o, '_, '_, A> {
    type Item = Token<'o>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.classifier.classify(self.inner.next()?))
    }
}

impl<'o, 'al> SegmentedTokenIter<'o, 'al> {
    /// Assign to each [`Token`]s a [`TokenKind`].
    ///
    /// [`TokenKind`]: crate::TokenKind
    pub fn classify(self) -> ClassifiedTokenIter<'o, 'al, 'o, Vec<u8>> {
        self.classify_with_stop_words(None)
    }

    /// Assign to each [`Token`]s a [`TokenKind`] using provided stop words.
    ///
    /// [`TokenKind`]: crate::TokenKind
    ///
    /// Any `Token` that is in the stop words [`Set`] is assigned to [`TokenKind::StopWord`].
    ///
    /// [`TokenKind::StopWord`]: crate::TokenKind#StopWord
    pub fn classify_with_stop_words<'sw, A: AsRef<[u8]>>(
        self,
        stop_words: Option<&'sw Set<A>>,
    ) -> ClassifiedTokenIter<'o, 'al, 'sw, A> {
        ClassifiedTokenIter { inner: self, classifier: TokenClassifier::new(stop_words) }
    }
}

#[derive(Clone)]
struct TokenClassifier<'sw, A> {
    stop_words: Option<&'sw Set<A>>,
}

impl Default for TokenClassifier<'_, Vec<u8>> {
    fn default() -> Self {
        Self { stop_words: None }
    }
}

impl<'sw, A> TokenClassifier<'sw, A> {
    pub fn new(stop_words: Option<&'sw Set<A>>) -> Self {
        Self { stop_words }
    }
}

impl<A> TokenClassifier<'_, A>
where
    A: AsRef<[u8]>,
{
    pub fn classify<'o>(&self, mut token: Token<'o>) -> Token<'o> {
        let lemma = token.lemma();
        let mut is_hard_separator = false;
        if self.stop_words.map(|stop_words| stop_words.contains(lemma)).unwrap_or(false) {
            token.kind = TokenKind::StopWord;
            token
        } else if lemma.chars().all(|c| match classify_separator(c) {
            Some(SeparatorKind::Hard) => {
                is_hard_separator = true;
                true
            }
            Some(SeparatorKind::Soft) => true,

            None => false,
        }) {
            if is_hard_separator {
                token.kind = TokenKind::Separator(SeparatorKind::Hard);
            } else {
                token.kind = TokenKind::Separator(SeparatorKind::Soft);
            }
            token
        } else {
            token.kind = TokenKind::Word;
            token
        }
    }
}

fn classify_separator(c: char) -> Option<SeparatorKind> {
    match deunicode_char(c)?.chars().next()? {
        // Prevent deunicoding cyrillic chars (e.g. ь -> ' is incorrect)
        _ if ('\u{0410}'..='\u{044f}').contains(&c) => None, // russian cyrillic letters [а-яА-Я]
        c if c.is_whitespace() => Some(SeparatorKind::Soft), // whitespaces
        '-' | '_' | '\'' | ':' | '/' | '\\' | '@' | '"' | '+' | '~' | '=' | '^' | '*' | '#' => {
            Some(SeparatorKind::Soft)
        }
        '.' | ';' | ',' | '!' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '|' => {
            Some(SeparatorKind::Hard)
        }
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use std::borrow::Cow;

    use super::*;
    use crate::Token;

    #[test]
    fn separators() {
        let classifier = TokenClassifier::default();

        let token =
            classifier.classify(Token { lemma: Cow::Borrowed("   "), ..Default::default() });
        assert_eq!(token.separator_kind(), Some(SeparatorKind::Soft));

        let token =
            classifier.classify(Token { lemma: Cow::Borrowed("\" "), ..Default::default() });
        assert_eq!(token.separator_kind(), Some(SeparatorKind::Soft));

        let token =
            classifier.classify(Token { lemma: Cow::Borrowed("@   "), ..Default::default() });
        assert_eq!(token.separator_kind(), Some(SeparatorKind::Soft));

        let token = classifier.classify(Token { lemma: Cow::Borrowed("."), ..Default::default() });
        assert_eq!(token.separator_kind(), Some(SeparatorKind::Hard));

        let token =
            classifier.classify(Token { lemma: Cow::Borrowed("   ."), ..Default::default() });
        assert_eq!(token.separator_kind(), Some(SeparatorKind::Hard));

        let token =
            classifier.classify(Token { lemma: Cow::Borrowed("  。"), ..Default::default() });
        assert_eq!(token.separator_kind(), Some(SeparatorKind::Hard));

        let token =
            classifier.classify(Token { lemma: Cow::Borrowed("S.O.S"), ..Default::default() });
        assert!(token.is_word());

        let token = classifier.classify(Token { lemma: Cow::Borrowed("ь"), ..Default::default() });
        assert!(token.is_word());
    }

    #[test]
    fn stop_words() {
        let stop_words = Set::from_iter(["the"].iter()).unwrap();
        let classifier = TokenClassifier::new(Some(&stop_words));

        let token =
            classifier.classify(Token { lemma: Cow::Borrowed("the"), ..Default::default() });
        assert!(token.is_stopword());

        let token =
            classifier.classify(Token { lemma: Cow::Borrowed("The"), ..Default::default() });
        assert!(token.is_word());

        let token =
            classifier.classify(Token { lemma: Cow::Borrowed("foobar"), ..Default::default() });
        assert!(token.is_word());
    }
}
