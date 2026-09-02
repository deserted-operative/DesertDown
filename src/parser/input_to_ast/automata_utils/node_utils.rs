use self::NodeType::{EmbeddedLink, Underline};

/*
    Traits:
        debug allows {:?} formatting
        clone gives .clone()
        copy allows values to be copied rather than moved
        partialEq allows equality testing between NodeType values
        eq declares equality is a full equivalence, so reflexivity holds

    repr(u8): one byte per node type
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
/// Types for both block nodes, as well as nodes that are valid within blocks
pub enum NodeType {
    // Root
    Document,
    NoOp,

    // Blocks
    Paragraph,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    ThematicBreak,
    FoldingBreak,
    IndentBlock,
    Callout,
    DashListItem,
    PlusListItem,
    AsteriskListItem,
    NumListItem,
    MathBlock,
    CodeBlock,
    CommentBlock,
    TableRowBlock,

    // Inline categories - NOT for output use, but for internal decisions
    Emphasis,
    Highlighting,

    // Inlines
    Literal,
    Italic,
    Bold,
    BoldItalic,
    HighlightC1,
    HighlightC2,
    Underline,
    Strikethrough,
    InlineCode,
    InlineComment,
    Link,
    EmbeddedLink,
    InlineMath,
    Tag,

    // Special
    EscapeBackslash,

    // H1
    H1Opener,
    H1Content,
    H1Closer,

    // H2
    H2Opener,
    H2Content,
    H2Closer,

    // H3
    H3Opener,
    H3Content,
    H3Closer,

    // H4
    H4Opener,
    H4Content,
    H4Closer,

    // H5
    H5Opener,
    H5Content,
    H5Closer,

    // H6
    H6Opener,
    H6Content,
    H6Closer,

    // Indent block
    IndentOpener,
    IndentContent,
    IndentContinuation,

    // Callout
    CalloutOpener,
    CalloutType,
    CalloutTitle,
    CalloutHeaderTerminator,
    CalloutContent,
    CalloutContinuation,

    // Dash list item
    DashListItemOpener,
    DashListItemContent,
    DashListItemContinuation,

    // Plus list item
    PlusListItemOpener,
    PlusListItemContent,
    PlusListItemContinuation,

    // Asterisk list item
    AsteriskListItemOpener,
    AsteriskListItemContent,
    AsteriskListItemContinuation,

    // Num list item
    NumListItemOpener,
    NumListItemContent,
    NumListItemContinuation,

    // Math block
    MathBlockOpener,
    MathBlockContent,
    MathBlockCloser,

    // Code block
    CodeBlockOpener,
    CodeBlockOpenerTerminator,
    CodeBlockOpenerContent,
    CodeBlockLanguage,
    CodeBlockContent,
    CodeBlockCloser,

    // Comment block
    CommentBlockOpener,
    CommentBlockContent,
    CommentBlockCloser,

    // Table row
    TableWall,
    TableCellContent,
    TableHeaderRow,
    LeftAlignedTableHeaderRow,
    CentredTableHeaderRow,
    RightAlignedTableHeaderRow,

    // Emphasis
    EmphDelimiter,
    DoubleEmphDelimiter,
    TripleEmphDelimeter,

    // Highlighting
    HighlightC1Delimiter,
    HighlightC2Delimiter,

    // Underline
    UnderlineDelimiter,

    // Strikethorugh
    StrikethroughDelimiter,

    // Inline Code
    InlineCodeOpener,
    InlineCodeContent,
    InlineCodeCloser,

    // Inline Comment
    InlineCommentOpener,
    InlineCommentContent,
    InlineCommentCloser,

    // Inline Math
    InlineMathOpener,
    InlineMathContent,
    InlineMathCloser,

    // Tag
    TagOpener,
    TagContent,

    // Link
    LinkOpener,
    LinkContent,
    LinkCloser,

    // Embedded Link
    EmbeddedLinkOpener,
    EmbeddedLinkContent,
    EmbeddedLinkCloser,
}

// methods implemented for NodeType
impl NodeType {
    #[inline]
    /// Takes a node and returns the NodeType for the block it belongs to
    pub const fn block_type(self) -> NodeType {
        match self {
            // Paragraph
            NodeType::Paragraph => NodeType::Paragraph,

            // H1
            NodeType::H1Opener | NodeType::H1Content | NodeType::H1Closer => NodeType::H1,

            // H2
            NodeType::H2Opener | NodeType::H2Content | NodeType::H2Closer => NodeType::H2,

            // H3
            NodeType::H3Opener | NodeType::H3Content | NodeType::H3Closer => NodeType::H3,

            // H4
            NodeType::H4Opener | NodeType::H4Content | NodeType::H4Closer => NodeType::H4,

            // H5
            NodeType::H5Opener | NodeType::H5Content | NodeType::H5Closer => NodeType::H5,

            // H6
            NodeType::H6Opener | NodeType::H6Content | NodeType::H6Closer => NodeType::H6,

            // Thematic Break
            NodeType::ThematicBreak => NodeType::ThematicBreak,

            // Folding Break
            NodeType::FoldingBreak => NodeType::FoldingBreak,

            // Indent Block
            NodeType::IndentOpener | NodeType::IndentContent | NodeType::IndentContinuation => {
                NodeType::IndentBlock
            }

            // Callout
            NodeType::CalloutOpener
            | NodeType::CalloutType
            | NodeType::CalloutTitle
            | NodeType::CalloutHeaderTerminator
            | NodeType::CalloutContent
            | NodeType::CalloutContinuation => NodeType::Callout,

            // Dash List Item
            NodeType::DashListItemOpener
            | NodeType::DashListItemContent
            | NodeType::DashListItemContinuation => NodeType::DashListItem,

            // Plus List Item
            NodeType::PlusListItemOpener
            | NodeType::PlusListItemContent
            | NodeType::PlusListItemContinuation => NodeType::PlusListItem,

            // Asterisk List Item
            NodeType::AsteriskListItemOpener
            | NodeType::AsteriskListItemContent
            | NodeType::AsteriskListItemContinuation => NodeType::AsteriskListItem,

            // Num List Item
            NodeType::NumListItemOpener
            | NodeType::NumListItemContent
            | NodeType::NumListItemContinuation => NodeType::NumListItem,

            // Math Block
            NodeType::MathBlockOpener | NodeType::MathBlockContent | NodeType::MathBlockCloser => {
                NodeType::MathBlock
            }

            // Code Block
            NodeType::CodeBlockOpener
            | NodeType::CodeBlockOpenerTerminator
            | NodeType::CodeBlockOpenerContent
            | NodeType::CodeBlockLanguage
            | NodeType::CodeBlockContent
            | NodeType::CodeBlockCloser => NodeType::CodeBlock,

            // Comment Block
            NodeType::CommentBlockOpener
            | NodeType::CommentBlockContent
            | NodeType::CommentBlockCloser => NodeType::CommentBlock,

            // Table Row Block
            NodeType::TableWall
            | NodeType::TableCellContent
            | NodeType::TableHeaderRow
            | NodeType::LeftAlignedTableHeaderRow
            | NodeType::CentredTableHeaderRow
            | NodeType::RightAlignedTableHeaderRow => NodeType::TableRowBlock,

            // default
            _ => NodeType::NoOp,
        }
    }

    #[inline]
    /// Takes a NodeType, and returns whether its occurrence opens a new block.
    /// Required for the situation where two of the same block follow each other - eg. two separate dash list items on two consecutive input lines
    pub const fn is_block_opener(self) -> bool {
        /*
            All headings (levels 1 to 6), when an opener node is encountered, this means a new heading block should be opened
            For thematic and folding breaks, all characters that belong to a break are written out at the same time, therefore whenever encountering a thematic/folding break write-out, that should open a new block
            For all list items (dash, plus, asterisk, num) an opener means a new list item block should be opened
            For math, code, and comment blocks, an opener means a new block should be opened

            For indent and callout blocks, successive openings are facilitated by the folding break, so not covered here

            For table row blocks, will need to instead rely on when one is closed - which will be a table wall node with a newline in it (a table wall can only have a newline in it if it's the end of the line) - handled by function below
        */
        match self {
            // Heading Blocks
            NodeType::H1Opener
            | NodeType::H2Opener
            | NodeType::H3Opener
            | NodeType::H4Opener
            | NodeType::H5Opener
            | NodeType::H6Opener => true,

            // Themaatic/Folding Break Blocks
            NodeType::ThematicBreak | NodeType::FoldingBreak => true,

            // List Item Blocks
            NodeType::DashListItemOpener
            | NodeType::PlusListItemOpener
            | NodeType::AsteriskListItemOpener
            | NodeType::NumListItemOpener => true,

            // Math, Code, and Comment Blocks
            NodeType::MathBlockOpener
            | NodeType::CodeBlockOpener
            | NodeType::CommentBlockOpener => true,

            // default
            _ => false,
        }
    }

    #[inline]
    /// Takes a NodeType, and returns whether its occurrence opens a new inline.
    /// Required for the situation where two of the same self contained inline follow each other - eg. two inline code spans, such as: `a``b`
    pub const fn is_inline_opener(self) -> bool {
        matches!(
            self,
            NodeType::InlineCodeOpener
                | NodeType::InlineCommentOpener
                | NodeType::InlineMathOpener
                | NodeType::TagOpener
                | NodeType::LinkOpener
                | NodeType::EmbeddedLinkOpener
        )
    }

    #[inline]
    /// Takes a NodeType, and returns whether encountering a newline marks the end of the block
    /// Required for the situation where two of the same block follow each other, but determining the end of the blocks via their treatment of newline chars is more indicative than how they are opened
    pub const fn does_newline_terminate_block(self) -> bool {
        /*
            Table row blocks end with a newline in a TableWall node (which can obviously only happen at the end of the line), thus marking an end of the block
            Thematic and Folding breaks are also blocks terminated by their trailing newline
        */
        matches!(
            self,
            NodeType::TableRowBlock | NodeType::ThematicBreak | NodeType::FoldingBreak
        )
    }

    #[inline]
    /// Takes a NodeType, and returns whether the block has no children.
    /// Applies to Thematic and Folding breaks, where the block type clearly indicates the semantics of the characters they span over, with no further information required
    pub const fn is_self_explanatory_block(self) -> bool {
        matches!(self, NodeType::ThematicBreak | NodeType::FoldingBreak)
    }

    #[inline]
    /// Takes a node and returns the NodeType for the inline structure it belongs to
    pub const fn inline_type(self) -> NodeType {
        match self {
            // Emphasis
            NodeType::EmphDelimiter
            | NodeType::DoubleEmphDelimiter
            | NodeType::TripleEmphDelimeter => NodeType::Emphasis,

            // Highlighting
            NodeType::HighlightC1Delimiter | NodeType::HighlightC2Delimiter => {
                NodeType::Highlighting
            }

            // Literal
            NodeType::Literal => NodeType::Literal,

            // Italic
            NodeType::Italic => NodeType::Italic,

            // Bold
            NodeType::Bold => NodeType::Bold,

            // BoldItalic
            NodeType::BoldItalic => NodeType::BoldItalic,

            // HighlightC1 (ie. Colour 1)
            NodeType::HighlightC1 => NodeType::HighlightC1,

            // HighlightC2 (ie. Colour 2)
            NodeType::HighlightC2 => NodeType::HighlightC2,

            // Underline
            NodeType::Underline | NodeType::UnderlineDelimiter => Underline,

            // Strikethrough
            NodeType::Strikethrough | NodeType::StrikethroughDelimiter => NodeType::Strikethrough,

            // Inline Code
            NodeType::InlineCodeOpener
            | NodeType::InlineCodeContent
            | NodeType::InlineCodeCloser => NodeType::InlineCode,

            // Inline Comment
            NodeType::InlineCommentOpener
            | NodeType::InlineCommentContent
            | NodeType::InlineCommentCloser => NodeType::InlineComment,

            // Inline Math
            NodeType::InlineMathOpener
            | NodeType::InlineMathContent
            | NodeType::InlineMathCloser => NodeType::InlineMath,

            // Tag
            NodeType::TagOpener | NodeType::TagContent => NodeType::Tag,

            // Link
            NodeType::LinkOpener | NodeType::LinkContent | NodeType::LinkCloser => NodeType::Link,

            // Embedded Link
            NodeType::EmbeddedLinkOpener
            | NodeType::EmbeddedLinkContent
            | NodeType::EmbeddedLinkCloser => EmbeddedLink,

            // default
            _ => NodeType::NoOp,
        }
    }
}
