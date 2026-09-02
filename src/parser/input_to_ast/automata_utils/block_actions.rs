use super::action_utils::{Action, RangeExpression};
use super::{ACTION_LOOKUP_WIDTH, BLOCK_STATE_COUNT, NodeType};

/// Array of actions which states can take upon being reached.
/// The [`BLOCK_ACTION_LOOKUP`] array is used to map state indices to the action(s) those states perform.
/// Actions that are part of a chain are stored in sequential order - a properpty exploited by [`BLOCK_ACTION_LOOKUP`]
pub const BLOCK_ACTIONS: &[Action] = &[
    //start
    Action {
        range_expression: RangeExpression::Set(0),
        output_label: NodeType::NoOp,
    },
    //internal
    Action {
        range_expression: RangeExpression::Set(0),
        output_label: NodeType::NoOp,
    },
    //paragraph-content
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::Paragraph,
    },
    //escape-sequence:PART_1
    Action {
        range_expression: RangeExpression::Set(1),
        output_label: NodeType::EscapeBackslash,
    },
    //escape-sequence:PART_2
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::Literal,
    },
    //escape-sequence-1s:PART_1
    Action {
        range_expression: RangeExpression::Set(1),
        output_label: NodeType::Literal,
    },
    //escape-sequence-1s:PART_2
    Action {
        range_expression: RangeExpression::Set(1),
        output_label: NodeType::EscapeBackslash,
    },
    //escape-sequence-1s:PART_3
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::Literal,
    },
    //escape-sequence-2s:PART_1
    Action {
        range_expression: RangeExpression::Set(2),
        output_label: NodeType::Literal,
    },
    //escape-sequence-2s:PART_2
    Action {
        range_expression: RangeExpression::Set(1),
        output_label: NodeType::EscapeBackslash,
    },
    //escape-sequence-2s:PART_3
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::Literal,
    },
    //escape-sequence-3s:PART_1
    Action {
        range_expression: RangeExpression::Set(3),
        output_label: NodeType::Literal,
    },
    //escape-sequence-3s:PART_2
    Action {
        range_expression: RangeExpression::Set(1),
        output_label: NodeType::EscapeBackslash,
    },
    //escape-sequence-3s:PART_3
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::Literal,
    },
    //h1-opener-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::H1Opener,
    },
    //h1-content-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::H1Content,
    },
    //h1-content-e
    Action {
        range_expression: RangeExpression::Sub(1),
        output_label: NodeType::H1Content,
    },
    //h1-opener-t:PART_1
    Action {
        range_expression: RangeExpression::Sub(1),
        output_label: NodeType::H1Opener,
    },
    //h1-opener-t:PART_2
    Action {
        range_expression: RangeExpression::Set(1),
        output_label: NodeType::H1Closer,
    },
    //h1-closer-t
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::H1Closer,
    },
    //h2-opener-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::H2Opener,
    },
    //h2-content-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::H2Content,
    },
    //h2-content-e
    Action {
        range_expression: RangeExpression::Sub(1),
        output_label: NodeType::H2Content,
    },
    //h2-opener-t:PART_1
    Action {
        range_expression: RangeExpression::Sub(1),
        output_label: NodeType::H2Opener,
    },
    //h2-opener-t:PART_2
    Action {
        range_expression: RangeExpression::Set(1),
        output_label: NodeType::H2Closer,
    },
    //h2-closer-t
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::H2Closer,
    },
    //h3-opener-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::H3Opener,
    },
    //h3-content-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::H3Content,
    },
    //h3-content-e
    Action {
        range_expression: RangeExpression::Sub(1),
        output_label: NodeType::H3Content,
    },
    //h3-opener-t:PART_1
    Action {
        range_expression: RangeExpression::Sub(1),
        output_label: NodeType::H3Opener,
    },
    //h3-opener-t:PART_2
    Action {
        range_expression: RangeExpression::Set(1),
        output_label: NodeType::H3Closer,
    },
    //h3-closer-t
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::H3Closer,
    },
    //h4-opener-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::H4Opener,
    },
    //h4-content-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::H4Content,
    },
    //h4-content-e
    Action {
        range_expression: RangeExpression::Sub(1),
        output_label: NodeType::H4Content,
    },
    //h4-opener-t:PART_1
    Action {
        range_expression: RangeExpression::Sub(1),
        output_label: NodeType::H4Opener,
    },
    //h4-opener-t:PART_2
    Action {
        range_expression: RangeExpression::Set(1),
        output_label: NodeType::H4Closer,
    },
    //h4-closer-t
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::H4Closer,
    },
    //h5-opener-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::H5Opener,
    },
    //h5-content-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::H5Content,
    },
    //h5-content-e
    Action {
        range_expression: RangeExpression::Sub(1),
        output_label: NodeType::H5Content,
    },
    //h5-opener-t:PART_1
    Action {
        range_expression: RangeExpression::Sub(1),
        output_label: NodeType::H5Opener,
    },
    //h5-opener-t:PART_2
    Action {
        range_expression: RangeExpression::Set(1),
        output_label: NodeType::H5Closer,
    },
    //h5-closer-t
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::H5Closer,
    },
    //h6-opener-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::H6Opener,
    },
    //h6-content-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::H6Content,
    },
    //h6-content-e
    Action {
        range_expression: RangeExpression::Sub(1),
        output_label: NodeType::H6Content,
    },
    //h6-opener-t:PART_1
    Action {
        range_expression: RangeExpression::Sub(1),
        output_label: NodeType::H6Opener,
    },
    //h6-opener-t:PART_2
    Action {
        range_expression: RangeExpression::Set(1),
        output_label: NodeType::H6Closer,
    },
    //h6-closer-t
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::H6Closer,
    },
    //indent-opener-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::IndentOpener,
    },
    //indent-content-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::IndentContent,
    },
    //indent-continuation-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::IndentContinuation,
    },
    //indent-opener-content:PART_1
    Action {
        range_expression: RangeExpression::Set(4),
        output_label: NodeType::IndentOpener,
    },
    //indent-opener-content:PART_2
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::IndentContent,
    },
    //plus-list-opener-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::PlusListItemOpener,
    },
    //plus-list-content-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::PlusListItemContent,
    },
    //plus-list-continuation-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::PlusListItemContinuation,
    },
    //callout-opener-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::CalloutOpener,
    },
    //callout-type-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::CalloutType,
    },
    //callout-title-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::CalloutTitle,
    },
    //callout-header-terminator-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::CalloutHeaderTerminator,
    },
    //callout-content-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::CalloutContent,
    },
    //callout-continuation-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::CalloutContinuation,
    },
    //thematic-break-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::ThematicBreak,
    },
    //folding-break-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::FoldingBreak,
    },
    //dash-list-opener-content:PART_1
    Action {
        range_expression: RangeExpression::Set(2),
        output_label: NodeType::DashListItemOpener,
    },
    //dash-list-opener-content:PART_2
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::DashListItemContent,
    },
    //dash-list-opener-content-1s:PART_1
    Action {
        range_expression: RangeExpression::Set(3),
        output_label: NodeType::DashListItemOpener,
    },
    //dash-list-opener-content-1s:PART_2
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::DashListItemContent,
    },
    //dash-list-opener-content-2s:PART_1
    Action {
        range_expression: RangeExpression::Set(4),
        output_label: NodeType::DashListItemOpener,
    },
    //dash-list-opener-content-2s:PART_2
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::DashListItemContent,
    },
    //dash-list-opener-content-3s:PART_1
    Action {
        range_expression: RangeExpression::Set(5),
        output_label: NodeType::DashListItemOpener,
    },
    //dash-list-opener-content-3s:PART_2
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::DashListItemContent,
    },
    //dash-list-opener-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::DashListItemOpener,
    },
    //dash-list-content-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::DashListItemContent,
    },
    //dash-list-continuation-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::DashListItemContinuation,
    },
    //asterisk-list-opener-content:PART_1
    Action {
        range_expression: RangeExpression::Set(2),
        output_label: NodeType::AsteriskListItemOpener,
    },
    //asterisk-list-opener-content:PART_2
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::AsteriskListItemContent,
    },
    //asterisk-list-opener-content-1s:PART_1
    Action {
        range_expression: RangeExpression::Set(3),
        output_label: NodeType::AsteriskListItemOpener,
    },
    //asterisk-list-opener-content-1s:PART_2
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::AsteriskListItemContent,
    },
    //asterisk-list-opener-content-2s:PART_1
    Action {
        range_expression: RangeExpression::Set(4),
        output_label: NodeType::AsteriskListItemOpener,
    },
    //asterisk-list-opener-content-2s:PART_2
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::AsteriskListItemContent,
    },
    //asterisk-list-opener-content-3s:PART_1
    Action {
        range_expression: RangeExpression::Set(5),
        output_label: NodeType::AsteriskListItemOpener,
    },
    //asterisk-list-opener-content-3s:PART_2
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::AsteriskListItemContent,
    },
    //asterisk-list-opener-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::AsteriskListItemOpener,
    },
    //asterisk-list-content-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::AsteriskListItemContent,
    },
    //asterisk-list-continuation-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::AsteriskListItemContinuation,
    },
    //num-list-opener-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::NumListItemOpener,
    },
    //num-list-content-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::NumListItemContent,
    },
    //num-list-continuation-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::NumListItemContinuation,
    },
    //math-block-opener-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::MathBlockOpener,
    },
    //math-block-content-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::MathBlockContent,
    },
    //math-block-closer-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::MathBlockCloser,
    },
    //code-block-opener-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::CodeBlockOpener,
    },
    //code-block-opener-end-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::CodeBlockOpenerTerminator,
    },
    //code-block-content-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::CodeBlockContent,
    },
    //code-block-closer-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::CodeBlockCloser,
    },
    //code-block-language-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::CodeBlockLanguage,
    },
    //code-block-escaped-backtick:PART_1
    Action {
        range_expression: RangeExpression::Set(1),
        output_label: NodeType::EscapeBackslash,
    },
    //code-block-escaped-backtick:PART_2
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::CodeBlockContent,
    },
    //code-block-content-then-escaped-backtick:PART_1
    Action {
        range_expression: RangeExpression::Sub(2),
        output_label: NodeType::CodeBlockContent,
    },
    //code-block-content-then-escaped-backtick:PART_2
    Action {
        range_expression: RangeExpression::Set(1),
        output_label: NodeType::EscapeBackslash,
    },
    //code-block-content-then-escaped-backtick:PART_3
    Action {
        range_expression: RangeExpression::Set(1),
        output_label: NodeType::CodeBlockContent,
    },
    //comment-block-opener-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::CommentBlockOpener,
    },
    //comment-block-content-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::CommentBlockContent,
    },
    //comment-block-closer-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::CommentBlockCloser,
    },
    //table-wall-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::TableWall,
    },
    //table-cell-content-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::TableCellContent,
    },
    //table-cell-content-e
    Action {
        range_expression: RangeExpression::Sub(1),
        output_label: NodeType::TableCellContent,
    },
    //table-wall-content-wall:PART_1
    Action {
        range_expression: RangeExpression::Set(1),
        output_label: NodeType::TableWall,
    },
    //table-wall-content-wall:PART_2
    Action {
        range_expression: RangeExpression::Sub(2),
        output_label: NodeType::TableCellContent,
    },
    //table-wall-content-wall:PART_3
    Action {
        range_expression: RangeExpression::Set(2),
        output_label: NodeType::TableWall,
    },
    //table-header-row-wall:PART_1
    Action {
        range_expression: RangeExpression::Sub(2),
        output_label: NodeType::TableHeaderRow,
    },
    //table-header-row-wall:PART_2
    Action {
        range_expression: RangeExpression::Set(2),
        output_label: NodeType::TableWall,
    },
    //table-header-row-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::TableHeaderRow,
    },
    //table-header-row-right-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::RightAlignedTableHeaderRow,
    },
    //table-header-row-left-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::LeftAlignedTableHeaderRow,
    },
    //table-header-row-centre-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::CentredTableHeaderRow,
    },
    //table-header-row-right-wall:PART_1
    Action {
        range_expression: RangeExpression::Sub(2),
        output_label: NodeType::RightAlignedTableHeaderRow,
    },
    //table-header-row-right-wall:PART_2
    Action {
        range_expression: RangeExpression::Set(2),
        output_label: NodeType::TableWall,
    },
    //table-header-row-left-wall:PART_1
    Action {
        range_expression: RangeExpression::Sub(2),
        output_label: NodeType::LeftAlignedTableHeaderRow,
    },
    //table-header-row-left-wall:PART_2
    Action {
        range_expression: RangeExpression::Set(2),
        output_label: NodeType::TableWall,
    },
    //table-header-row-centre-wall:PART_1
    Action {
        range_expression: RangeExpression::Sub(2),
        output_label: NodeType::CentredTableHeaderRow,
    },
    //table-header-row-centre-wall:PART_2
    Action {
        range_expression: RangeExpression::Set(2),
        output_label: NodeType::TableWall,
    },
];

/// Lookup array for identifying which action(s) in the [`BLOCK_ACTIONS`] array should be executed for each state.
/// Each state has two contiguous elements in this array - the first being the index in [`BLOCK_ACTIONS`] of the first action performed by the state, the second being the number of actions executed by the state.
#[rustfmt::skip]
pub static BLOCK_ACTION_LOOKUP: [u32; BLOCK_STATE_COUNT * ACTION_LOOKUP_WIDTH] = [
      0,   1,    // Start
      1,   1,    // I1
     14,   1,    // IP2
     15,   1,    // IP3
     16,   1,    // IE1
      1,   1,    // I7
      1,   1,    // I8
     19,   1,    // TC2-O-Start
     19,   1,    // TC3-O-Start
      1,   1,    // I2
     20,   1,    // IP4
     21,   1,    // IP5
     22,   1,    // IE2
      1,   1,    // I9
      1,   1,    // I10
     25,   1,    // TC5-O-Start
     25,   1,    // TC6-O-Start
      1,   1,    // I3
     26,   1,    // IP6
     27,   1,    // IP7
     28,   1,    // IE3
      1,   1,    // I11
      1,   1,    // I12
     31,   1,    // TC8-O-Start
     31,   1,    // TC9-O-Start
      1,   1,    // I4
     32,   1,    // IP8
     33,   1,    // IP9
     34,   1,    // IE4
      1,   1,    // I13
      1,   1,    // I14
     37,   1,    // TC11-O-Start
     37,   1,    // TC12-O-Start
      1,   1,    // I5
     38,   1,    // IP10
     39,   1,    // IP11
     40,   1,    // IE5
      1,   1,    // I15
      1,   1,    // I16
     43,   1,    // TC14-O-Start
     43,   1,    // TC15-O-Start
      1,   1,    // I6
     44,   1,    // IP12
     45,   1,    // IP13
     46,   1,    // IE6
      1,   1,    // I17
      1,   1,    // I18
     49,   1,    // TC17-O-Start
     49,   1,    // TC18-O-Start
      1,   1,    // I19
      1,   1,    // I20
      1,   1,    // I21
      2,   1,    // IP1
      2,   1,    // TP2-O-Start
      1,   1,    // I161
      3,   2,    // IP108-O-IP1
      1,   1,    // I162
      5,   3,    // IP109-O-IP1
      1,   1,    // I163
      8,   3,    // IP110-O-IP1
      1,   1,    // I164
     11,   3,    // IP111-O-IP1
     50,   1,    // IP14
     51,   1,    // IP15
     51,   1,    // IP213-O-Start
      1,   1,    // I406-O-I19
      1,   1,    // I407-O-I20
      1,   1,    // I408-O-I21
     52,   1,    // IP208
     50,   1,    // IP209
     51,   1,    // IP210
     51,   1,    // IP211-O-Start
     51,   1,    // IP212-O-IP209
      1,   1,    // I36
     55,   1,    // IP32
     56,   1,    // TP7-O-Start
      1,   1,    // I457
      1,   1,    // I458
      1,   1,    // IP33
     56,   1,    // IP34-O-Start
      1,   1,    // I37-O-I19
     57,   1,    // IP35
      1,   1,    // I459
     55,   1,    // IP230
     56,   1,    // IP231
     56,   1,    // IP232-O-Start
      1,   1,    // I460-O-I19
      1,   1,    // I461-O-I20
      1,   1,    // I462-O-I21
      1,   1,    // I463
      1,   1,    // I464
     53,   2,    // IC1-O-IP15
     57,   1,    // IP233
      1,   1,    // I38
     55,   1,    // IP36
     56,   1,    // TP8-O-Start
      1,   1,    // I465
      1,   1,    // I466
      1,   1,    // IP37
     56,   1,    // IP38-O-Start
      1,   1,    // I39-O-I19
      1,   1,    // I40-O-I20
     57,   1,    // IP39
      1,   1,    // I467
     55,   1,    // IP234
     56,   1,    // IP235
     56,   1,    // IP236-O-Start
      1,   1,    // I468-O-I19
      1,   1,    // I469-O-I20
      1,   1,    // I470-O-I21
      1,   1,    // I471
      1,   1,    // I472
     53,   2,    // IC2-O-IP15
      1,   1,    // I473
     57,   1,    // IP237
      1,   1,    // I41
     55,   1,    // IP40
     56,   1,    // TP9-O-Start
      1,   1,    // I474
      1,   1,    // I475
      1,   1,    // IP41
     56,   1,    // IP42-O-Start
      1,   1,    // I42-O-I19
      1,   1,    // I43-O-I20
      1,   1,    // I44-O-I21
     57,   1,    // IP43
      1,   1,    // I476
     55,   1,    // IP238
     56,   1,    // IP239
     56,   1,    // IP240-O-Start
      1,   1,    // I477-O-I19
      1,   1,    // I478-O-I20
      1,   1,    // I479-O-I21
      1,   1,    // I480
      1,   1,    // I481
      1,   1,    // I482
     53,   2,    // IC3-O-IP15
      1,   1,    // I483
     57,   1,    // IP241
      1,   1,    // I45
     55,   1,    // IP44
     56,   1,    // TP10-O-Start
      1,   1,    // I484
      1,   1,    // I485
      1,   1,    // IP45
     56,   1,    // IP46-O-Start
      1,   1,    // I46-O-I19
      1,   1,    // I47-O-I20
      1,   1,    // I48-O-I21
     53,   2,    // IC4-O-IP15
      1,   1,    // I49
     57,   1,    // IP47
      1,   1,    // I486
     55,   1,    // IP242
     56,   1,    // IP243
     56,   1,    // IP244-O-Start
      1,   1,    // I487-O-I19
      1,   1,    // I488-O-I20
      1,   1,    // I489-O-I21
      1,   1,    // I490
      1,   1,    // I491
      1,   1,    // I492
      1,   1,    // I493
     53,   2,    // IC5-O-IP15
      1,   1,    // I494
     57,   1,    // IP245
     58,   1,    // IP262
     62,   1,    // IP267
     58,   1,    // IP263
      1,   1,    // I533
     59,   1,    // IP264
     62,   1,    // IP268-O-Start
     63,   1,    // IP269
     63,   1,    // IP270
     59,   1,    // IP265
     60,   1,    // IP266
     61,   1,    // IP800-O-Start
     58,   1,    // IP271
     62,   1,    // IP276
     58,   1,    // IP272
      1,   1,    // I534
     63,   1,    // IP278
     59,   1,    // IP273
     62,   1,    // IP277-O-Start
      1,   1,    // I535-O-I19
     63,   1,    // IP279
     59,   1,    // IP274
     60,   1,    // IP275
     61,   1,    // IP801-O-Start
      1,   1,    // I543-O-I19
     58,   1,    // IP280
     62,   1,    // IP285
     58,   1,    // IP281
     63,   1,    // IP288
      1,   1,    // I536
     63,   1,    // IP287
     59,   1,    // IP282
     62,   1,    // IP286-O-Start
      1,   1,    // I537-O-I19
      1,   1,    // I538-O-I20
     59,   1,    // IP283
     60,   1,    // IP284
     61,   1,    // IP802-O-Start
      1,   1,    // I544-O-I19
      1,   1,    // I545-O-I20
     58,   1,    // IP289
     62,   1,    // IP294
     58,   1,    // IP290
     63,   1,    // IP297
      1,   1,    // I539
     63,   1,    // IP296
     59,   1,    // IP291
     62,   1,    // IP295-O-Start
      1,   1,    // I540-O-I19
      1,   1,    // I541-O-I20
      1,   1,    // I542-O-I21
     59,   1,    // IP292
     60,   1,    // IP293
     61,   1,    // IP803-O-Start
      1,   1,    // I546-O-I19
      1,   1,    // I547-O-I20
      1,   1,    // I548-O-I21
      1,   1,    // I551
      1,   1,    // I22
      1,   1,    // I550
      1,   1,    // I552
     64,   1,    // TP51-O-Start
     66,   2,    // TC23-O-Start
      1,   1,    // I549
      1,   1,    // I553
      1,   1,    // I555
      1,   1,    // I409
      1,   1,    // I554
     66,   2,    // TC19-O-Start
     66,   2,    // IC6-O-IP17
     66,   2,    // TC20-O-Start
      1,   1,    // I410
     66,   2,    // IC7-O-IP17
      1,   1,    // I411
     75,   1,    // IP17
     75,   1,    // IP31-O-Start
      1,   1,    // I27-O-I19
     76,   1,    // IP18
     75,   1,    // TP3-O-Start
     74,   1,    // IP214
     53,   2,    // IC46-O-IP15
     75,   1,    // IP215
     76,   1,    // IP217
      1,   1,    // I416
      1,   1,    // I415
     75,   1,    // IP216-O-Start
      1,   1,    // I412-O-I19
      1,   1,    // I413-O-I20
      1,   1,    // I414-O-I21
      1,   1,    // I23
     68,   2,    // TC21-O-Start
      1,   1,    // I556
      1,   1,    // I558
      1,   1,    // I559
      1,   1,    // I417
      1,   1,    // I557
     68,   2,    // TC22-O-Start
     68,   2,    // IC8-O-IP20
     68,   2,    // TC24-O-Start
      1,   1,    // I418
     68,   2,    // IC9-O-IP20
      1,   1,    // I419
     75,   1,    // IP20
     75,   1,    // IP28-O-Start
      1,   1,    // I28-O-I19
      1,   1,    // I29-O-I20
     76,   1,    // IP21
     75,   1,    // TP4-O-Start
     74,   1,    // IP218
     53,   2,    // IC47-O-IP15
     75,   1,    // IP219
     76,   1,    // IP220
      1,   1,    // I425
      1,   1,    // I424
      1,   1,    // I423
     75,   1,    // IP225-O-Start
      1,   1,    // I420-O-I19
      1,   1,    // I421-O-I20
      1,   1,    // I422-O-I21
      1,   1,    // I24
     70,   2,    // TC25-O-Start
      1,   1,    // I560
      1,   1,    // I562
      1,   1,    // I563
      1,   1,    // I426
      1,   1,    // I561
     70,   2,    // TC26-O-Start
     70,   2,    // IC10-O-IP23
     70,   2,    // TC27-O-Start
      1,   1,    // I427
     70,   2,    // IC11-O-IP23
      1,   1,    // I428
     75,   1,    // IP23
     75,   1,    // IP29-O-Start
      1,   1,    // I30-O-I19
      1,   1,    // I31-O-I20
      1,   1,    // I32-O-I21
     76,   1,    // IP24
     75,   1,    // TP5-O-Start
     74,   1,    // IP221
     53,   2,    // IC48-O-IP15
     75,   1,    // IP222
     76,   1,    // IP224
      1,   1,    // I435
      1,   1,    // I434
      1,   1,    // I433
      1,   1,    // I432
     75,   1,    // IP223-O-Start
      1,   1,    // I429-O-I19
      1,   1,    // I430-O-I20
      1,   1,    // I431-O-I21
      1,   1,    // I25
     72,   2,    // TC28-O-Start
      1,   1,    // I564
      1,   1,    // I566
      1,   1,    // I567
      1,   1,    // I446
      1,   1,    // I565
     72,   2,    // TC29-O-Start
     72,   2,    // IC12-O-IP26
     72,   2,    // TC30-O-Start
      1,   1,    // I447
     72,   2,    // IC13-O-IP26
      1,   1,    // I448
     75,   1,    // IP26
     75,   1,    // IP30-O-Start
      1,   1,    // I33-O-I19
      1,   1,    // I34-O-I20
      1,   1,    // I35-O-I21
      1,   1,    // I26
     76,   1,    // IP27
     53,   2,    // IC43-O-IP15
     75,   1,    // TP6-O-Start
     74,   1,    // IP229
     53,   2,    // IC36-O-IP15
     75,   1,    // IP226
     76,   1,    // IP228
      1,   1,    // I456
      1,   1,    // I455
      1,   1,    // I454
      1,   1,    // I453
      1,   1,    // I452
     75,   1,    // IP227-O-Start
      1,   1,    // I449-O-I19
      1,   1,    // I450-O-I20
      1,   1,    // I451-O-I21
      1,   1,    // I572
      1,   1,    // I50
      1,   1,    // I573
      1,   1,    // I574
     77,   2,    // TC31-O-Start
      1,   1,    // I568
      1,   1,    // I570
      1,   1,    // I571
      1,   1,    // I495
      1,   1,    // I569
     77,   2,    // TC32-O-Start
     77,   2,    // IC14-O-IP49
     77,   2,    // TC33-O-Start
      1,   1,    // I496
     77,   2,    // IC15-O-IP49
      1,   1,    // I497
     86,   1,    // IP49
     86,   1,    // IP50-O-Start
      1,   1,    // I51-O-I19
     87,   1,    // IP51
     86,   1,    // TP11-O-Start
     85,   1,    // IP246
     53,   2,    // IC49-O-IP15
     86,   1,    // IP247
     87,   1,    // IP249
      1,   1,    // I502
      1,   1,    // I501
     86,   1,    // IP248-O-Start
      1,   1,    // I498-O-I19
      1,   1,    // I499-O-I20
      1,   1,    // I500-O-I21
      1,   1,    // I52
     79,   2,    // TC34-O-Start
      1,   1,    // I575
      1,   1,    // I577
      1,   1,    // I578
      1,   1,    // I503
      1,   1,    // I576
     79,   2,    // TC35-O-Start
     79,   2,    // IC16-O-IP53
     79,   2,    // TC36-O-Start
      1,   1,    // I504
     79,   2,    // IC17-O-IP53
      1,   1,    // I505
     86,   1,    // IP53
     86,   1,    // IP54-O-Start
      1,   1,    // I53-O-I19
      1,   1,    // I54-O-I20
     87,   1,    // IP55
     86,   1,    // TP12-O-Start
     85,   1,    // IP250
     53,   2,    // IC50-O-IP15
     86,   1,    // IP251
     87,   1,    // IP253
      1,   1,    // I511
      1,   1,    // I510
      1,   1,    // I509
     86,   1,    // IP252-O-Start
      1,   1,    // I506-O-I19
      1,   1,    // I507-O-I20
      1,   1,    // I508-O-I21
      1,   1,    // I55
     81,   2,    // TC37-O-Start
      1,   1,    // I579
      1,   1,    // I581
      1,   1,    // I582
      1,   1,    // I512
      1,   1,    // I580
     81,   2,    // TC38-O-Start
     81,   2,    // IC18-O-IP57
     81,   2,    // TC39-O-Start
      1,   1,    // I513
     81,   2,    // IC19-O-IP57
      1,   1,    // I514
     86,   1,    // IP57
     86,   1,    // IP58-O-Start
      1,   1,    // I56-O-I19
      1,   1,    // I57-O-I20
      1,   1,    // I58-O-I21
     87,   1,    // IP59
     86,   1,    // TP13-O-Start
     85,   1,    // IP254
     53,   2,    // IC51-O-IP15
     86,   1,    // IP255
     87,   1,    // IP257
      1,   1,    // I521
      1,   1,    // I520
      1,   1,    // I519
      1,   1,    // I518
     86,   1,    // IP256-O-Start
      1,   1,    // I515-O-I19
      1,   1,    // I516-O-I20
      1,   1,    // I517-O-I21
      1,   1,    // I59
     83,   2,    // TC40-O-Start
      1,   1,    // I583
      1,   1,    // I585
      1,   1,    // I586
      1,   1,    // I522
      1,   1,    // I584
     83,   2,    // TC41-O-Start
     83,   2,    // IC20-O-IP61
     83,   2,    // TC42-O-Start
      1,   1,    // I523
     83,   2,    // IC21-O-IP61
      1,   1,    // I524
     86,   1,    // IP61
     86,   1,    // IP62-O-Start
      1,   1,    // I60-O-I19
      1,   1,    // I61-O-I20
      1,   1,    // I62-O-I21
      1,   1,    // I63
     87,   1,    // IP63
     53,   2,    // IC52-O-IP15
     86,   1,    // TP14-O-Start
     85,   1,    // IP258
     53,   2,    // IC37-O-IP15
     86,   1,    // IP259
     87,   1,    // IP261
      1,   1,    // I532
      1,   1,    // I531
      1,   1,    // I530
      1,   1,    // I529
      1,   1,    // I528
     86,   1,    // IP260-O-Start
      1,   1,    // I525-O-I19
      1,   1,    // I526-O-I20
      1,   1,    // I527-O-I21
      1,   1,    // I64
      1,   1,    // I73
     88,   1,    // IP64
     89,   1,    // TP15-O-Start
     89,   1,    // IP65
     90,   1,    // IP67
     89,   1,    // IP66-O-Start
      1,   1,    // I74-O-I19
      1,   1,    // I75-O-I20
      1,   1,    // I65
      1,   1,    // I76
     88,   1,    // IP68
     89,   1,    // TP16-O-Start
     89,   1,    // IP69
     90,   1,    // IP71
     89,   1,    // IP70-O-Start
      1,   1,    // I77-O-I19
      1,   1,    // I78-O-I20
      1,   1,    // I79-O-I21
      1,   1,    // I66
      1,   1,    // I80
     88,   1,    // IP72
     89,   1,    // TP17-O-Start
     89,   1,    // IP73
     90,   1,    // IP75
     89,   1,    // IP74-O-Start
      1,   1,    // I81-O-I19
      1,   1,    // I82-O-I20
      1,   1,    // I83-O-I21
      1,   1,    // I84
     53,   2,    // IC22-O-IP15
      1,   1,    // I67
      1,   1,    // I85
     88,   1,    // IP76
     89,   1,    // TP18-O-Start
     89,   1,    // IP77
     90,   1,    // IP79
     89,   1,    // IP78-O-Start
      1,   1,    // I86-O-I19
      1,   1,    // I87-O-I20
      1,   1,    // I88-O-I21
      1,   1,    // I89
      1,   1,    // I90
     53,   2,    // IC23-O-IP15
      1,   1,    // I68
      1,   1,    // I91
     88,   1,    // IP80
     89,   1,    // TP19-O-Start
     89,   1,    // IP81
     90,   1,    // IP83
     89,   1,    // IP82-O-Start
      1,   1,    // I92-O-I19
      1,   1,    // I93-O-I20
      1,   1,    // I94-O-I21
      1,   1,    // I95
      1,   1,    // I96
      1,   1,    // I97
     53,   2,    // IC24-O-IP15
      1,   1,    // I69
      1,   1,    // I98
     88,   1,    // IP84
     89,   1,    // TP20-O-Start
     89,   1,    // IP85
     90,   1,    // IP87
     89,   1,    // IP86-O-Start
      1,   1,    // I99-O-I19
      1,   1,    // I100-O-I20
      1,   1,    // I101-O-I21
      1,   1,    // I102
      1,   1,    // I103
      1,   1,    // I104
      1,   1,    // I105
     53,   2,    // IC25-O-IP15
      1,   1,    // I70
      1,   1,    // I106
     88,   1,    // IP88
     89,   1,    // TP21-O-Start
     89,   1,    // IP89
     90,   1,    // IP91
     89,   1,    // IP90-O-Start
      1,   1,    // I107-O-I19
      1,   1,    // I108-O-I20
      1,   1,    // I109-O-I21
      1,   1,    // I110
      1,   1,    // I111
      1,   1,    // I112
      1,   1,    // I113
      1,   1,    // I114
     53,   2,    // IC26-O-IP15
      1,   1,    // I71
      1,   1,    // I115
     88,   1,    // IP92
     89,   1,    // TP22-O-Start
     89,   1,    // IP93
     90,   1,    // IP95
     89,   1,    // IP94-O-Start
      1,   1,    // I116-O-I19
      1,   1,    // I117-O-I20
      1,   1,    // I118-O-I21
      1,   1,    // I119
      1,   1,    // I120
      1,   1,    // I121
      1,   1,    // I122
      1,   1,    // I123
      1,   1,    // I124
     53,   2,    // IC27-O-IP15
      1,   1,    // I72
      1,   1,    // I125
     88,   1,    // IP96
     89,   1,    // TP23-O-Start
     89,   1,    // IP97
     90,   1,    // IP99
     89,   1,    // IP98-O-Start
      1,   1,    // I126-O-I19
      1,   1,    // I127-O-I20
      1,   1,    // I128-O-I21
      1,   1,    // I129
      1,   1,    // I130
      1,   1,    // I131
      1,   1,    // I132
      1,   1,    // I133
      1,   1,    // I134
      1,   1,    // I135
     53,   2,    // IC28-O-IP15
      1,   1,    // I313
      1,   1,    // I367
     88,   1,    // IP196
     89,   1,    // TP48-O-Start
     89,   1,    // IP197
     90,   1,    // IP199
     89,   1,    // IP198-O-Start
      1,   1,    // I368-O-I19
      1,   1,    // I369-O-I20
      1,   1,    // I370-O-I21
      1,   1,    // I371
      1,   1,    // I372
      1,   1,    // I373
      1,   1,    // I374
      1,   1,    // I375
      1,   1,    // I376
      1,   1,    // I377
      1,   1,    // I378
     53,   2,    // IC29-O-IP15
      1,   1,    // I314
      1,   1,    // I379
     88,   1,    // IP200
     89,   1,    // TP49-O-Start
     89,   1,    // IP201
     90,   1,    // IP203
     89,   1,    // IP202-O-Start
      1,   1,    // I380-O-I19
      1,   1,    // I381-O-I20
      1,   1,    // I382-O-I21
      1,   1,    // I383
      1,   1,    // I384
      1,   1,    // I385
      1,   1,    // I386
      1,   1,    // I387
      1,   1,    // I388
      1,   1,    // I389
      1,   1,    // I390
      1,   1,    // I391
     53,   2,    // IC30-O-IP15
      1,   1,    // I315
      1,   1,    // I392
     88,   1,    // IP204
     89,   1,    // TP50-O-Start
     89,   1,    // IP205
     90,   1,    // IP207
     89,   1,    // IP206-O-Start
      1,   1,    // I393-O-I19
      1,   1,    // I394-O-I20
      1,   1,    // I395-O-I21
      1,   1,    // I396
      1,   1,    // I397
      1,   1,    // I398
      1,   1,    // I399
      1,   1,    // I400
      1,   1,    // I401
      1,   1,    // I402
      1,   1,    // I403
      1,   1,    // I404
      1,   1,    // I405
     53,   2,    // IC31-O-IP15
      1,   1,    // I587
      1,   1,    // I588
      1,   1,    // I589
      1,   1,    // I590
      1,   1,    // I591
      1,   1,    // I592
     65,   1,    // TP52-O-Start
      1,   1,    // I136
     91,   1,    // IP16
      1,   1,    // I137
     92,   1,    // IP25
     93,   1,    // TP24-O-Start
     92,   1,    // IP19
     92,   1,    // IP22
      1,   1,    // I138
      1,   1,    // I139
     94,   1,    // IP48
      1,   1,    // I140
     99,   2,    // IC32-O-IP52
     96,   1,    // IP52
      1,   1,    // I141
      1,   1,    // I142
      1,   1,    // I143
    101,   3,    // IC33-O-IP52
     97,   1,    // TP25-O-Start
     98,   1,    // IP60
     94,   1,    // IP56
     95,   1,    // IP100-O-IP52
      1,   1,    // I144
    104,   1,    // IP101
      1,   1,    // I145
    105,   1,    // IP104
    106,   1,    // TP26-O-Start
    105,   1,    // IP102
    105,   1,    // IP103
      1,   1,    // I146
    107,   1,    // IP105
    108,   1,    // IP106
    109,   1,    // IE7
    107,   1,    // IP107
      1,   1,    // I147
      1,   1,    // I148
      1,   1,    // I149
    110,   3,    // IC34-O-I146
    107,   1,    // TP27-O-Start
      1,   1,    // I150
      1,   1,    // I151
      1,   1,    // I152
    113,   2,    // IC35-O-I146
    115,   1,    // TP29-O-Start
    116,   1,    // TP30-O-Start
      1,   1,    // I153
      1,   1,    // I154
    119,   2,    // IC38-O-I146
      1,   1,    // I155
      1,   1,    // I156
      1,   1,    // I157
      1,   1,    // I158
    121,   2,    // IC39-O-I146
    117,   1,    // TP31-O-Start
      1,   1,    // I159
      1,   1,    // I160
    123,   2,    // IC40-O-I146
    118,   1,    // TP32-O-Start
];
