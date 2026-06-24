use bincode_next::{BorrowDecode, Encode};

pub enum DelayKind {
    None,
    Short,
    Medium,
    Long,
}
#[derive(Encode, BorrowDecode, Debug, Clone, PartialEq, Eq)]
pub enum Affix {
    None,
    Comma,
    Period,
    Question,
    Exclamation,
    Colon,
    Semicolon,
    Ellipsis,
}
impl From<Affix> for DelayKind {
    fn from(affix: Affix) -> Self {
        match affix {
            Affix::None => DelayKind::None,
            Affix::Comma | Affix::Colon | Affix::Semicolon => DelayKind::Short,
            Affix::Period | Affix::Question | Affix::Exclamation => DelayKind::Medium,
            Affix::Ellipsis => DelayKind::Long,
        }
    }
}

#[derive(Encode, BorrowDecode, Debug, Clone, PartialEq, Eq)]
pub struct WordToken<'a> {
    pub prefix: Affix,
    pub text: &'a str,
    pub suffix: Affix,
}

#[derive(Encode, BorrowDecode, Debug, Clone, PartialEq, Eq)]
pub enum TokenKind<'a> {
    Word(WordToken<'a>),
    ParagraphBreak,
    LineBreak,
    SectionBreak,
}
