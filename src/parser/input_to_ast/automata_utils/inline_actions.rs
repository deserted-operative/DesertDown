use super::action_utils::{Action, RangeExpression};
use super::{ACTION_LOOKUP_WIDTH, INLINE_STATE_COUNT, NodeType};

/// Array of actions which states can take upon being reached.
/// The [`INLINE_ACTION_LOOKUP`] array is used to map state indices to the action(s) those states perform.
/// Actions that are part of a chain are stored in sequential order - a properpty exploited by [`INLINE_ACTION_LOOKUP`]
pub const INLINE_ACTIONS: &[Action] = &[
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
    //literal-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::Literal,
    },
    //literal-e
    Action {
        range_expression: RangeExpression::Sub(1),
        output_label: NodeType::Literal,
    },
    //literal-s
    Action {
        range_expression: RangeExpression::Set(1),
        output_label: NodeType::Literal,
    },
    //literal-f
    Action {
        range_expression: RangeExpression::Set(4),
        output_label: NodeType::Literal,
    },
    //emph-delim-s
    Action {
        range_expression: RangeExpression::Set(1),
        output_label: NodeType::EmphDelimiter,
    },
    //emph-delim-1-literal:PART_1
    Action {
        range_expression: RangeExpression::Set(1),
        output_label: NodeType::EmphDelimiter,
    },
    //emph-delim-1-literal:PART_2
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::Literal,
    },
    //double-emph-delim-2-literal:PART_1
    Action {
        range_expression: RangeExpression::Set(2),
        output_label: NodeType::DoubleEmphDelimiter,
    },
    //double-emph-delim-2-literal:PART_2
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::Literal,
    },
    //triple-emph-delim-3-literal:PART_1
    Action {
        range_expression: RangeExpression::Set(3),
        output_label: NodeType::TripleEmphDelimeter,
    },
    //triple-emph-delim-3-literal:PART_2
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::Literal,
    },
    //double-emph-delim-d
    Action {
        range_expression: RangeExpression::Set(2),
        output_label: NodeType::DoubleEmphDelimiter,
    },
    //triple-emph-delim-t
    Action {
        range_expression: RangeExpression::Set(3),
        output_label: NodeType::TripleEmphDelimeter,
    },
    //highlight-1-delim-literal:PART_1
    Action {
        range_expression: RangeExpression::Set(2),
        output_label: NodeType::HighlightC1Delimiter,
    },
    //highlight-1-delim-literal:PART_2
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::Literal,
    },
    //highlight-1-delim-d
    Action {
        range_expression: RangeExpression::Set(2),
        output_label: NodeType::HighlightC1Delimiter,
    },
    //highlight-2-delim-literal:PART_1
    Action {
        range_expression: RangeExpression::Set(3),
        output_label: NodeType::HighlightC2Delimiter,
    },
    //highlight-2-delim-literal:PART_2
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::Literal,
    },
    //highlight-2-delim-t
    Action {
        range_expression: RangeExpression::Set(3),
        output_label: NodeType::HighlightC2Delimiter,
    },
    //highlight-1-then-2-delim-literal:PART_1
    Action {
        range_expression: RangeExpression::Set(2),
        output_label: NodeType::HighlightC1Delimiter,
    },
    //highlight-1-then-2-delim-literal:PART_2
    Action {
        range_expression: RangeExpression::Set(3),
        output_label: NodeType::HighlightC2Delimiter,
    },
    //highlight-1-then-2-delim-literal:PART_3
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::Literal,
    },
    //highlight-1-then-2-delim:PART_1
    Action {
        range_expression: RangeExpression::Set(2),
        output_label: NodeType::HighlightC1Delimiter,
    },
    //highlight-1-then-2-delim:PART_2
    Action {
        range_expression: RangeExpression::Set(3),
        output_label: NodeType::HighlightC2Delimiter,
    },
    //underline-delim-literal:PART_1
    Action {
        range_expression: RangeExpression::Set(2),
        output_label: NodeType::UnderlineDelimiter,
    },
    //underline-delim-literal:PART_2
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::Literal,
    },
    //underline-delim-d
    Action {
        range_expression: RangeExpression::Set(2),
        output_label: NodeType::UnderlineDelimiter,
    },
    //strikethrough-delim-literal:PART_1
    Action {
        range_expression: RangeExpression::Set(2),
        output_label: NodeType::StrikethroughDelimiter,
    },
    //strikethrough-delim-literal:PART_2
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::Literal,
    },
    //strikethrough-delim-d
    Action {
        range_expression: RangeExpression::Set(2),
        output_label: NodeType::StrikethroughDelimiter,
    },
    //escape-backslash-inline-code:PART_1
    Action {
        range_expression: RangeExpression::Set(1),
        output_label: NodeType::EscapeBackslash,
    },
    //escape-backslash-inline-code:PART_2
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::InlineCodeContent,
    },
    //escape-backslash-inline-comment:PART_1
    Action {
        range_expression: RangeExpression::Set(1),
        output_label: NodeType::EscapeBackslash,
    },
    //escape-backslash-inline-comment:PART_2
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::InlineCommentContent,
    },
    //escape-backslash-link-content:PART_1
    Action {
        range_expression: RangeExpression::Set(1),
        output_label: NodeType::EscapeBackslash,
    },
    //escape-backslash-link-content:PART_2
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::LinkContent,
    },
    //escape-backslash-embed-link-content:PART_1
    Action {
        range_expression: RangeExpression::Set(1),
        output_label: NodeType::EscapeBackslash,
    },
    //escape-backslash-embed-link-content:PART_2
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::EmbeddedLinkContent,
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
    //inline-math-opener-content:PART_1
    Action {
        range_expression: RangeExpression::Set(1),
        output_label: NodeType::InlineMathOpener,
    },
    //inline-math-opener-content:PART_2
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::InlineMathContent,
    },
    //inline-math-closer-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::InlineMathCloser,
    },
    //inline-math-content-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::InlineMathContent,
    },
    //inline-math-opener-e
    Action {
        range_expression: RangeExpression::Sub(1),
        output_label: NodeType::InlineMathOpener,
    },
    //inline-code-opener-content:PART_1
    Action {
        range_expression: RangeExpression::Set(1),
        output_label: NodeType::InlineCodeOpener,
    },
    //inline-code-opener-content:PART_2
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::InlineCodeContent,
    },
    //inline-code-closer-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::InlineCodeCloser,
    },
    //inline-code-content-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::InlineCodeContent,
    },
    //inline-code-content-e
    Action {
        range_expression: RangeExpression::Sub(1),
        output_label: NodeType::InlineCodeContent,
    },
    //inline-code-opener-e
    Action {
        range_expression: RangeExpression::Sub(1),
        output_label: NodeType::InlineCodeOpener,
    },
    //inline-comment-opener-content:PART_1
    Action {
        range_expression: RangeExpression::Set(2),
        output_label: NodeType::InlineCommentOpener,
    },
    //inline-comment-opener-content:PART_2
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::InlineCommentContent,
    },
    //inline-comment-closer-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::InlineCommentCloser,
    },
    //inline-comment-content-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::InlineCommentContent,
    },
    //inline-comment-content-e
    Action {
        range_expression: RangeExpression::Sub(1),
        output_label: NodeType::InlineCommentContent,
    },
    //tag-opener-content:PART_1
    Action {
        range_expression: RangeExpression::Set(1),
        output_label: NodeType::TagOpener,
    },
    //tag-opener-content:PART_2
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::TagContent,
    },
    //tag-content-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::TagContent,
    },
    //tag-content-e
    Action {
        range_expression: RangeExpression::Sub(1),
        output_label: NodeType::TagContent,
    },
    //tag-opener-e
    Action {
        range_expression: RangeExpression::Sub(1),
        output_label: NodeType::TagOpener,
    },
    //link-opener-content:PART_1
    Action {
        range_expression: RangeExpression::Set(2),
        output_label: NodeType::LinkOpener,
    },
    //link-opener-content:PART_2
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::LinkContent,
    },
    //link-content-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::LinkContent,
    },
    //link-content-e
    Action {
        range_expression: RangeExpression::Sub(1),
        output_label: NodeType::LinkContent,
    },
    //link-opener-e
    Action {
        range_expression: RangeExpression::Sub(1),
        output_label: NodeType::LinkOpener,
    },
    //link-closer-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::LinkCloser,
    },
    //embed-link-opener-content:PART_1
    Action {
        range_expression: RangeExpression::Set(3),
        output_label: NodeType::EmbeddedLinkOpener,
    },
    //embed-link-opener-content:PART_2
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::EmbeddedLinkContent,
    },
    //embed-link-content-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::EmbeddedLinkContent,
    },
    //embed-link-content-e
    Action {
        range_expression: RangeExpression::Sub(1),
        output_label: NodeType::EmbeddedLinkContent,
    },
    //embed-link-opener-e
    Action {
        range_expression: RangeExpression::Sub(1),
        output_label: NodeType::EmbeddedLinkOpener,
    },
    //embed-link-closer-p
    Action {
        range_expression: RangeExpression::Identity,
        output_label: NodeType::EmbeddedLinkCloser,
    },
];

/// Lookup array for identifying which action(s) in the [`INLINE_ACTIONS`] array should be executed for each state.
/// Each state has two contiguous elements in this array - the first being the index in [`INLINE_ACTIONS`] of the first action performed by the state, the second being the number of actions executed by the state.
#[rustfmt::skip]
pub static INLINE_ACTION_LOOKUP: [u32; INLINE_STATE_COUNT * ACTION_LOOKUP_WIDTH] = [
      0,   1,    // Start
      3,   1,    // IE1
     40,   2,    // TC1-O-Start
      2,   1,    // TP1-O-Start
      1,   1,    // I1
      6,   1,    // IS1-O-IE1-EL1
      6,   1,    // IS2-O-I4-EL1
      6,   1,    // IS3-O-I7-EL1
      6,   1,    // IS4-O-I9-EL1
      6,   1,    // IS5-O-I11-EL1
      6,   1,    // IS6-O-I12-EL1
      6,   1,    // IS7-O-I16-EL1
      6,   1,    // IS8-O-I13-EL1
      6,   1,    // IS9-O-I17-EL1
      6,   1,    // IS10-O-I20-EL1
      7,   2,    // TC2-O-Start-EL1
      1,   1,    // I2
     13,   1,    // ID1-O-I11-EL2
     13,   1,    // ID2-O-I9-EL2
     13,   1,    // ID3-O-I7-EL2
     13,   1,    // ID4-O-I4-EL2
     13,   1,    // ID5-O-IE1-EL2
     13,   1,    // ID6-O-I12-EL2
     13,   1,    // ID7-O-I16-EL2
     13,   1,    // ID8-O-I13-EL2
     13,   1,    // ID9-O-I17-EL2
     13,   1,    // ID10-O-I20-EL2
      9,   2,    // TC3-O-Start-EL2
      1,   1,    // I3
     14,   1,    // IT1-O-I11-EL3
     14,   1,    // IT2-O-I9-EL3
     14,   1,    // IT3-O-I7-EL3
     14,   1,    // IT4-O-I4-EL3
     14,   1,    // IT5-O-IE1-EL3
     14,   1,    // IT6-O-I12-EL3
     14,   1,    // IT7-O-I16-EL3
     14,   1,    // IT8-O-I13-EL3
     14,   1,    // IT9-O-I17-EL3
     14,   1,    // IT10-O-I20-EL3
     11,   2,    // TC4-O-Start-EL3
      2,   1,    // TP2-O-Start
      1,   1,    // I4
      1,   1,    // I5
     17,   1,    // ID11-O-I11
     17,   1,    // ID12-O-I9
     17,   1,    // ID13-O-I7
     17,   1,    // ID14-O-I1
     17,   1,    // ID15-O-IE1
     17,   1,    // ID16-O-I12
     17,   1,    // ID17-O-I16
     17,   1,    // ID18-O-I13
     17,   1,    // ID19-O-I17
     17,   1,    // ID20-O-I20
     15,   2,    // TC5-O-Start
      1,   1,    // I6
     20,   1,    // IT11-O-I11
     20,   1,    // IT12-O-I9
     20,   1,    // IT13-O-I7
     20,   1,    // IT14-O-I2
     20,   1,    // IT15-O-IE1
     20,   1,    // IT16-O-I12
     20,   1,    // IT17-O-I16
     20,   1,    // IT18-O-I13
     20,   1,    // IT19-O-I17
     20,   1,    // IT20-O-I20
     18,   2,    // TC6-O-Start
      1,   1,    // I24
      5,   1,    // IF5-O-I11
      5,   1,    // IF4-O-I9
      5,   1,    // IF3-O-I7
      5,   1,    // IF2-O-I2
      5,   1,    // IF1-O-IE1
      5,   1,    // IF6-O-I12
      5,   1,    // IF7-O-I16
      5,   1,    // IF8-O-I13
      5,   1,    // IF9-O-I17
      5,   1,    // IF10-O-I20
      2,   1,    // TP18-O-Start
      1,   1,    // I25
     24,   2,    // IF15-O-I11
     24,   2,    // IF14-O-I9
     24,   2,    // IF13-O-I7
     24,   2,    // IF12-O-I2
     24,   2,    // IF11-O-IE1
     24,   2,    // IF16-O-I12
     24,   2,    // IF17-O-I16
     24,   2,    // IF18-O-I13
     24,   2,    // IF19-O-I17
     24,   2,    // IF20-O-I20
     21,   3,    // TC9-O-Start
      2,   1,    // TP3-O-Start
      1,   1,    // I7
      1,   1,    // I8
     28,   1,    // ID21-O-I11
     28,   1,    // ID22-O-I9
     28,   1,    // ID23-O-I4
     28,   1,    // ID24-O-I1
     28,   1,    // ID25-O-IE1
     28,   1,    // ID26-O-I12
     28,   1,    // ID27-O-I16
     28,   1,    // ID28-O-I13
     28,   1,    // ID29-O-I17
     28,   1,    // ID30-O-I20
     26,   2,    // TC7-O-Start
      2,   1,    // TP4-O-Start
      1,   1,    // I9
      1,   1,    // I10
     31,   1,    // ID31-O-I11
     31,   1,    // ID32-O-I7
     31,   1,    // ID33-O-I4
     31,   1,    // ID34-O-I1
     31,   1,    // ID35-O-IE1
     31,   1,    // ID36-O-I12
     31,   1,    // ID37-O-I16
     31,   1,    // ID38-O-I13
     31,   1,    // ID39-O-I17
     31,   1,    // ID40-O-I20
     29,   2,    // TC8-O-Start
      2,   1,    // TP5-O-Start
     46,   1,    // IE8-O-IP4
      1,   1,    // I11
     42,   2,    // IC1
     45,   1,    // IP3
     45,   1,    // IP4
     44,   1,    // TP7-O-Start
      2,   1,    // TP6-O-Start
     52,   1,    // IE7-O-IE2
      1,   1,    // I12
     47,   2,    // IC2
     50,   1,    // IP2
     51,   1,    // IE2
     32,   2,    // IC3-O-IP2
      2,   1,    // TP8-O-Start
     49,   1,    // TP9-O-Start
      1,   1,    // I13
     52,   1,    // IE6-O-IE3
      1,   1,    // I14
     53,   2,    // IC4
     56,   1,    // IP5
     57,   1,    // IE3
     34,   2,    // IC5-O-IP5
      1,   1,    // I15-O-IP5
     55,   1,    // TP11-O-Start
      2,   1,    // TP10-O-Start
     62,   1,    // IE5-O-IE4
      1,   1,    // I16
     58,   2,    // IC6
     60,   1,    // IP6
     61,   1,    // IE4
      2,   1,    // TP13-O-Start
      2,   1,    // TP12-O-Start
      1,   1,    // I17
     67,   1,    // IE10-O-IE9
      1,   1,    // I18
     63,   2,    // IC7
     65,   1,    // IP7
     66,   1,    // IE9
     36,   2,    // IC8-O-IP7
      1,   1,    // I19-O-IP7
     68,   1,    // TP15-O-Start
      2,   1,    // TP14-O-Start
      1,   1,    // I20
      1,   1,    // I21
     73,   1,    // IE12-O-IE11
      1,   1,    // I22
     69,   2,    // IC9
     71,   1,    // IP8
     72,   1,    // IE11
     38,   2,    // IC10-O-IP8
      1,   1,    // I23-O-IP8
      2,   1,    // TP16-O-Start
     74,   1,    // TP17-O-Start
      4,   1,    // IS11-O-I20
      4,   1,    // IS13-O-I13
      4,   1,    // IS14-O-I16
      4,   1,    // IS15-O-I12
      4,   1,    // IS16-O-I11
      4,   1,    // IS17-O-I9
      4,   1,    // IS18-O-I7
      4,   1,    // IS19-O-I4
      4,   1,    // IS20-O-IE1
      4,   1,    // IS21-O-I1
      4,   1,    // IS22-O-I20
      4,   1,    // IS23-O-I17
      4,   1,    // IS25-O-I16
      4,   1,    // IS26-O-I12
      4,   1,    // IS27-O-I11
      4,   1,    // IS28-O-I9
      4,   1,    // IS29-O-I7
      4,   1,    // IS30-O-I4
      4,   1,    // IS31-O-IE1
      4,   1,    // IS32-O-I1
      4,   1,    // IS33-O-I20
      4,   1,    // IS34-O-I17
      4,   1,    // IS35-O-I16
      4,   1,    // IS36-O-I12
      4,   1,    // IS37-O-I11
      4,   1,    // IS38-O-I13
      4,   1,    // IS39-O-I7
      4,   1,    // IS40-O-I4
      4,   1,    // IS41-O-IE1
      4,   1,    // IS42-O-I1
      4,   1,    // IS43-O-I20
      4,   1,    // IS44-O-I17
      4,   1,    // IS45-O-I16
      4,   1,    // IS46-O-I12
      4,   1,    // IS47-O-I11
      4,   1,    // IS48-O-I13
      4,   1,    // IS49-O-I9
      4,   1,    // IS50-O-I4
      4,   1,    // IS51-O-IE1
      4,   1,    // IS52-O-I1
      4,   1,    // IS53-O-I7
      4,   1,    // IS55-O-I16
      4,   1,    // IS56-O-I12
      4,   1,    // IS57-O-I11
      4,   1,    // IS58-O-I13
      4,   1,    // IS59-O-I9
      4,   1,    // IS60-O-I4
      4,   1,    // IS61-O-IE1
      4,   1,    // IS62-O-I1
      4,   1,    // IS63-O-I7
      4,   1,    // IS64-O-I17
      4,   1,    // IS65-O-I16
      4,   1,    // IS66-O-I12
      4,   1,    // IS67-O-I11
      4,   1,    // IS68-O-I13
      4,   1,    // IS69-O-I9
      4,   1,    // IS70-O-I20
      4,   1,    // IS71-O-IE1
      4,   1,    // IS72-O-I1
      0,   1,    // Start-EL1
      3,   1,    // IE1-EL1
     40,   2,    // TC1-EL1-O-Start-EL1
      2,   1,    // TP1-EL1-O-Start-EL1
      1,   1,    // I1-EL1
      6,   1,    // IS1-EL1-O-IE1
      6,   1,    // IS2-EL1-O-I4
      6,   1,    // IS3-EL1-O-I7
      6,   1,    // IS4-EL1-O-I9
      6,   1,    // IS5-EL1-O-I11
      6,   1,    // IS6-EL1-O-I12
      6,   1,    // IS7-EL1-O-I16
      6,   1,    // IS8-EL1-O-I13
      6,   1,    // IS9-EL1-O-I17
      6,   1,    // IS10-EL1-O-I20
      7,   2,    // TC2-EL1-O-Start
      1,   1,    // I2-EL1
     13,   1,    // ID1-EL1-O-I11-EL3
     13,   1,    // ID2-EL1-O-I9-EL3
     13,   1,    // ID3-EL1-O-I7-EL3
     13,   1,    // ID4-EL1-O-I4-EL3
     13,   1,    // ID5-EL1-O-IE1-EL3
     13,   1,    // ID6-EL1-O-I12-EL3
     13,   1,    // ID7-EL1-O-I16-EL3
     13,   1,    // ID8-EL1-O-I13-EL3
     13,   1,    // ID9-EL1-O-I17-EL3
     13,   1,    // ID10-EL1-O-I20-EL3
      9,   2,    // TC3-EL1-O-Start-EL3
      2,   1,    // TP2-EL1-O-Start-EL1
      2,   1,    // IP1-EL1-O-Start-EL1
      2,   1,    // TP19-EL1-O-Start-EL1
      1,   1,    // I4-EL1
      1,   1,    // I5-EL1
     17,   1,    // ID11-EL1-O-I11-EL1
     17,   1,    // ID12-EL1-O-I9-EL1
     17,   1,    // ID13-EL1-O-I7-EL1
     17,   1,    // ID14-EL1-O-I1-EL1
     17,   1,    // ID15-EL1-O-IE1-EL1
     17,   1,    // ID16-EL1-O-I12-EL1
     17,   1,    // ID17-EL1-O-I16-EL1
     17,   1,    // ID18-EL1-O-I13-EL1
     17,   1,    // ID19-EL1-O-I17-EL1
     17,   1,    // ID20-EL1-O-I20-EL1
     15,   2,    // TC5-EL1-O-Start-EL1
      1,   1,    // I6-EL1
     20,   1,    // IT11-EL1-O-I11-EL1
     20,   1,    // IT12-EL1-O-I9-EL1
     20,   1,    // IT13-EL1-O-I7-EL1
     20,   1,    // IT14-EL1-O-I2-EL1
     20,   1,    // IT15-EL1-O-IE1-EL1
     20,   1,    // IT16-EL1-O-I12-EL1
     20,   1,    // IT17-EL1-O-I16-EL1
     20,   1,    // IT18-EL1-O-I13-EL1
     20,   1,    // IT19-EL1-O-I17-EL1
     20,   1,    // IT20-EL1-O-I20-EL1
     18,   2,    // TC6-EL1-O-Start-EL1
      1,   1,    // I24-EL1
      5,   1,    // IF5-EL1-O-I11-EL1
      5,   1,    // IF4-EL1-O-I9-EL1
      5,   1,    // IF3-EL1-O-I7-EL1
      5,   1,    // IF2-EL1-O-I2-EL1
      5,   1,    // IF1-EL1-O-IE1-EL1
      5,   1,    // IF6-EL1-O-I12-EL1
      5,   1,    // IF7-EL1-O-I16-EL1
      5,   1,    // IF8-EL1-O-I13-EL1
      5,   1,    // IF9-EL1-O-I17-EL1
      5,   1,    // IF10-EL1-O-I20-EL1
      2,   1,    // TP18-EL1-O-Start-EL1
      1,   1,    // I25-EL1
     24,   2,    // IF15-EL1-O-I11-EL1
     24,   2,    // IF14-EL1-O-I9-EL1
     24,   2,    // IF13-EL1-O-I7-EL1
     24,   2,    // IF12-EL1-O-I2-EL1
     24,   2,    // IF11-EL1-O-IE1-EL1
     24,   2,    // IF16-EL1-O-I12-EL1
     24,   2,    // IF17-EL1-O-I16-EL1
     24,   2,    // IF18-EL1-O-I13-EL1
     24,   2,    // IF19-EL1-O-I17-EL1
     24,   2,    // IF20-EL1-O-I20-EL1
     21,   3,    // TC9-EL1-O-Start-EL1
      2,   1,    // TP3-EL1-O-Start-EL1
      1,   1,    // I7-EL1
      1,   1,    // I8-EL1
     28,   1,    // ID21-EL1-O-I11-EL1
     28,   1,    // ID22-EL1-O-I9-EL1
     28,   1,    // ID23-EL1-O-I4-EL1
     28,   1,    // ID24-EL1-O-I1-EL1
     28,   1,    // ID25-EL1-O-IE1-EL1
     28,   1,    // ID26-EL1-O-I12-EL1
     28,   1,    // ID27-EL1-O-I16-EL1
     28,   1,    // ID28-EL1-O-I13-EL1
     28,   1,    // ID29-EL1-O-I17-EL1
     28,   1,    // ID30-EL1-O-I20-EL1
     26,   2,    // TC7-EL1-O-Start-EL1
      2,   1,    // TP4-EL1-O-Start-EL1
      1,   1,    // I9-EL1
      1,   1,    // I10-EL1
     31,   1,    // ID31-EL1-O-I11-EL1
     31,   1,    // ID32-EL1-O-I7-EL1
     31,   1,    // ID33-EL1-O-I4-EL1
     31,   1,    // ID34-EL1-O-I1-EL1
     31,   1,    // ID35-EL1-O-IE1-EL1
     31,   1,    // ID36-EL1-O-I12-EL1
     31,   1,    // ID37-EL1-O-I16-EL1
     31,   1,    // ID38-EL1-O-I13-EL1
     31,   1,    // ID39-EL1-O-I17-EL1
     31,   1,    // ID40-EL1-O-I20-EL1
     29,   2,    // TC8-EL1-O-Start-EL1
      2,   1,    // TP5-EL1-O-Start-EL1
     46,   1,    // IE8-EL1-O-IP4-EL1
      1,   1,    // I11-EL1
     42,   2,    // IC1-EL1
     45,   1,    // IP3-EL1
     45,   1,    // IP4-EL1
     44,   1,    // TP7-EL1-O-Start-EL1
      2,   1,    // TP6-EL1-O-Start-EL1
     52,   1,    // IE7-EL1-O-IE2-EL1
      1,   1,    // I12-EL1
     47,   2,    // IC2-EL1
     50,   1,    // IP2-EL1
     51,   1,    // IE2-EL1
     32,   2,    // IC3-EL1-O-IP2-EL1
      2,   1,    // TP8-EL1-O-Start-EL1
     49,   1,    // TP9-EL1-O-Start-EL1
      1,   1,    // I13-EL1
     52,   1,    // IE6-EL1-O-IE3-EL1
      1,   1,    // I14-EL1
     53,   2,    // IC4-EL1
     56,   1,    // IP5-EL1
     57,   1,    // IE3-EL1
     34,   2,    // IC5-EL1-O-IP5-EL1
      1,   1,    // I15-EL1-O-IP5-EL1
     55,   1,    // TP11-EL1-O-Start-EL1
      2,   1,    // TP10-EL1-O-Start-EL1
     62,   1,    // IE5-EL1-O-IE4-EL1
      1,   1,    // I16-EL1
     58,   2,    // IC6-EL1
     60,   1,    // IP6-EL1
     61,   1,    // IE4-EL1
      2,   1,    // TP13-EL1-O-Start-EL1
      2,   1,    // TP12-EL1-O-Start-EL1
      1,   1,    // I17-EL1
     67,   1,    // IE10-EL1-O-IE9-EL1
      1,   1,    // I18-EL1
     63,   2,    // IC7-EL1
     65,   1,    // IP7-EL1
     66,   1,    // IE9-EL1
     36,   2,    // IC8-EL1-O-IP7-EL1
      1,   1,    // I19-EL1-O-IP7-EL1
     68,   1,    // TP15-EL1-O-Start-EL1
      2,   1,    // TP14-EL1-O-Start-EL1
      1,   1,    // I20-EL1
      1,   1,    // I21-EL1
     73,   1,    // IE12-EL1-O-IE11-EL1
      1,   1,    // I22-EL1
     69,   2,    // IC9-EL1
     71,   1,    // IP8-EL1
     72,   1,    // IE11-EL1
     38,   2,    // IC10-EL1-O-IP8-EL1
      1,   1,    // I23-EL1-O-IP8-EL1
      2,   1,    // TP16-EL1-O-Start-EL1
     74,   1,    // TP17-EL1-O-Start-EL1
      4,   1,    // IS11-EL1-O-I20-EL1
      4,   1,    // IS13-EL1-O-I13-EL1
      4,   1,    // IS14-EL1-O-I16-EL1
      4,   1,    // IS15-EL1-O-I12-EL1
      4,   1,    // IS16-EL1-O-I11-EL1
      4,   1,    // IS17-EL1-O-I9-EL1
      4,   1,    // IS18-EL1-O-I7-EL1
      4,   1,    // IS19-EL1-O-I4-EL1
      4,   1,    // IS20-EL1-O-IE1-EL1
      4,   1,    // IS21-EL1-O-I1-EL1
      4,   1,    // IS22-EL1-O-I20-EL1
      4,   1,    // IS23-EL1-O-I17-EL1
      4,   1,    // IS25-EL1-O-I16-EL1
      4,   1,    // IS26-EL1-O-I12-EL1
      4,   1,    // IS27-EL1-O-I11-EL1
      4,   1,    // IS28-EL1-O-I9-EL1
      4,   1,    // IS29-EL1-O-I7-EL1
      4,   1,    // IS30-EL1-O-I4-EL1
      4,   1,    // IS31-EL1-O-IE1-EL1
      4,   1,    // IS32-EL1-O-I1-EL1
      4,   1,    // IS33-EL1-O-I20-EL1
      4,   1,    // IS34-EL1-O-I17-EL1
      4,   1,    // IS35-EL1-O-I16-EL1
      4,   1,    // IS36-EL1-O-I12-EL1
      4,   1,    // IS37-EL1-O-I11-EL1
      4,   1,    // IS38-EL1-O-I13-EL1
      4,   1,    // IS39-EL1-O-I7-EL1
      4,   1,    // IS40-EL1-O-I4-EL1
      4,   1,    // IS41-EL1-O-IE1-EL1
      4,   1,    // IS42-EL1-O-I1-EL1
      4,   1,    // IS43-EL1-O-I20-EL1
      4,   1,    // IS44-EL1-O-I17-EL1
      4,   1,    // IS45-EL1-O-I16-EL1
      4,   1,    // IS46-EL1-O-I12-EL1
      4,   1,    // IS47-EL1-O-I11-EL1
      4,   1,    // IS48-EL1-O-I13-EL1
      4,   1,    // IS49-EL1-O-I9-EL1
      4,   1,    // IS50-EL1-O-I4-EL1
      4,   1,    // IS51-EL1-O-IE1-EL1
      4,   1,    // IS52-EL1-O-I1-EL1
      4,   1,    // IS53-EL1-O-I7-EL1
      4,   1,    // IS55-EL1-O-I16-EL1
      4,   1,    // IS56-EL1-O-I12-EL1
      4,   1,    // IS57-EL1-O-I11-EL1
      4,   1,    // IS58-EL1-O-I13-EL1
      4,   1,    // IS59-EL1-O-I9-EL1
      4,   1,    // IS60-EL1-O-I4-EL1
      4,   1,    // IS61-EL1-O-IE1-EL1
      4,   1,    // IS62-EL1-O-I1-EL1
      4,   1,    // IS63-EL1-O-I7-EL1
      4,   1,    // IS64-EL1-O-I17-EL1
      4,   1,    // IS65-EL1-O-I16-EL1
      4,   1,    // IS66-EL1-O-I12-EL1
      4,   1,    // IS67-EL1-O-I11-EL1
      4,   1,    // IS68-EL1-O-I13-EL1
      4,   1,    // IS69-EL1-O-I9-EL1
      4,   1,    // IS70-EL1-O-I20-EL1
      4,   1,    // IS71-EL1-O-IE1-EL1
      4,   1,    // IS72-EL1-O-I1-EL1
      0,   1,    // Start-EL2
      3,   1,    // IE1-EL2
     40,   2,    // TC1-EL2-O-Start-EL2
      2,   1,    // TP1-EL2-O-Start-EL2
      1,   1,    // I1-EL2
      6,   1,    // IS1-EL2-O-IE1-EL3
      6,   1,    // IS2-EL2-O-I4-EL3
      6,   1,    // IS3-EL2-O-I7-EL3
      6,   1,    // IS4-EL2-O-I9-EL3
      6,   1,    // IS5-EL2-O-I11-EL3
      6,   1,    // IS6-EL2-O-I12-EL3
      6,   1,    // IS7-EL2-O-I16-EL3
      6,   1,    // IS8-EL2-O-I13-EL3
      6,   1,    // IS9-EL2-O-I17-EL3
      6,   1,    // IS10-EL2-O-I20-EL3
      7,   2,    // TC2-EL2-O-Start-EL3
      1,   1,    // I2-EL2
     13,   1,    // ID1-EL2-O-I11
     13,   1,    // ID2-EL2-O-I9
     13,   1,    // ID3-EL2-O-I7
     13,   1,    // ID4-EL2-O-I4
     13,   1,    // ID5-EL2-O-IE1
     13,   1,    // ID6-EL2-O-I12
     13,   1,    // ID7-EL2-O-I16
     13,   1,    // ID8-EL2-O-I13
     13,   1,    // ID9-EL2-O-I17
     13,   1,    // ID10-EL2-O-I20
      9,   2,    // TC3-EL2-O-Start
      2,   1,    // TP2-EL2-O-Start-EL2
      2,   1,    // IP1-EL2-O-Start-EL2
      2,   1,    // TP19-EL2-O-Start-EL2
      1,   1,    // I4-EL2
      1,   1,    // I5-EL2
     17,   1,    // ID11-EL2-O-I11-EL2
     17,   1,    // ID12-EL2-O-I9-EL2
     17,   1,    // ID13-EL2-O-I7-EL2
     17,   1,    // ID14-EL2-O-I1-EL2
     17,   1,    // ID15-EL2-O-IE1-EL2
     17,   1,    // ID16-EL2-O-I12-EL2
     17,   1,    // ID17-EL2-O-I16-EL2
     17,   1,    // ID18-EL2-O-I13-EL2
     17,   1,    // ID19-EL2-O-I17-EL2
     17,   1,    // ID20-EL2-O-I20-EL2
     15,   2,    // TC5-EL2-O-Start-EL2
      1,   1,    // I6-EL2
     20,   1,    // IT11-EL2-O-I11-EL2
     20,   1,    // IT12-EL2-O-I9-EL2
     20,   1,    // IT13-EL2-O-I7-EL2
     20,   1,    // IT14-EL2-O-I2-EL2
     20,   1,    // IT15-EL2-O-IE1-EL2
     20,   1,    // IT16-EL2-O-I12-EL2
     20,   1,    // IT17-EL2-O-I16-EL2
     20,   1,    // IT18-EL2-O-I13-EL2
     20,   1,    // IT19-EL2-O-I17-EL2
     20,   1,    // IT20-EL2-O-I20-EL2
     18,   2,    // TC6-EL2-O-Start-EL2
      1,   1,    // I24-EL2
      5,   1,    // IF5-EL2-O-I11-EL2
      5,   1,    // IF4-EL2-O-I9-EL2
      5,   1,    // IF3-EL2-O-I7-EL2
      5,   1,    // IF2-EL2-O-I2-EL2
      5,   1,    // IF1-EL2-O-IE1-EL2
      5,   1,    // IF6-EL2-O-I12-EL2
      5,   1,    // IF7-EL2-O-I16-EL2
      5,   1,    // IF8-EL2-O-I13-EL2
      5,   1,    // IF9-EL2-O-I17-EL2
      5,   1,    // IF10-EL2-O-I20-EL2
      2,   1,    // TP18-EL2-O-Start-EL2
      1,   1,    // I25-EL2
     24,   2,    // IF15-EL2-O-I11-EL2
     24,   2,    // IF14-EL2-O-I9-EL2
     24,   2,    // IF13-EL2-O-I7-EL2
     24,   2,    // IF12-EL2-O-I2-EL2
     24,   2,    // IF11-EL2-O-IE1-EL2
     24,   2,    // IF16-EL2-O-I12-EL2
     24,   2,    // IF17-EL2-O-I16-EL2
     24,   2,    // IF18-EL2-O-I13-EL2
     24,   2,    // IF19-EL2-O-I17-EL2
     24,   2,    // IF20-EL2-O-I20-EL2
     21,   3,    // TC9-EL2-O-Start-EL2
      2,   1,    // TP3-EL2-O-Start-EL2
      1,   1,    // I7-EL2
      1,   1,    // I8-EL2
     28,   1,    // ID21-EL2-O-I11-EL2
     28,   1,    // ID22-EL2-O-I9-EL2
     28,   1,    // ID23-EL2-O-I4-EL2
     28,   1,    // ID24-EL2-O-I1-EL2
     28,   1,    // ID25-EL2-O-IE1-EL2
     28,   1,    // ID26-EL2-O-I12-EL2
     28,   1,    // ID27-EL2-O-I16-EL2
     28,   1,    // ID28-EL2-O-I13-EL2
     28,   1,    // ID29-EL2-O-I17-EL2
     28,   1,    // ID30-EL2-O-I20-EL2
     26,   2,    // TC7-EL2-O-Start-EL2
      2,   1,    // TP4-EL2-O-Start-EL2
      1,   1,    // I9-EL2
      1,   1,    // I10-EL2
     31,   1,    // ID31-EL2-O-I11-EL2
     31,   1,    // ID32-EL2-O-I7-EL2
     31,   1,    // ID33-EL2-O-I4-EL2
     31,   1,    // ID34-EL2-O-I1-EL2
     31,   1,    // ID35-EL2-O-IE1-EL2
     31,   1,    // ID36-EL2-O-I12-EL2
     31,   1,    // ID37-EL2-O-I16-EL2
     31,   1,    // ID38-EL2-O-I13-EL2
     31,   1,    // ID39-EL2-O-I17-EL2
     31,   1,    // ID40-EL2-O-I20-EL2
     29,   2,    // TC8-EL2-O-Start-EL2
      2,   1,    // TP5-EL2-O-Start-EL2
     46,   1,    // IE8-EL2-O-IP4-EL2
      1,   1,    // I11-EL2
     42,   2,    // IC1-EL2
     45,   1,    // IP3-EL2
     45,   1,    // IP4-EL2
     44,   1,    // TP7-EL2-O-Start-EL2
      2,   1,    // TP6-EL2-O-Start-EL2
     52,   1,    // IE7-EL2-O-IE2-EL2
      1,   1,    // I12-EL2
     47,   2,    // IC2-EL2
     50,   1,    // IP2-EL2
     51,   1,    // IE2-EL2
     32,   2,    // IC3-EL2-O-IP2-EL2
      2,   1,    // TP8-EL2-O-Start-EL2
     49,   1,    // TP9-EL2-O-Start-EL2
      1,   1,    // I13-EL2
     52,   1,    // IE6-EL2-O-IE3-EL2
      1,   1,    // I14-EL2
     53,   2,    // IC4-EL2
     56,   1,    // IP5-EL2
     57,   1,    // IE3-EL2
     34,   2,    // IC5-EL2-O-IP5-EL2
      1,   1,    // I15-EL2-O-IP5-EL2
     55,   1,    // TP11-EL2-O-Start-EL2
      2,   1,    // TP10-EL2-O-Start-EL2
     62,   1,    // IE5-EL2-O-IE4-EL2
      1,   1,    // I16-EL2
     58,   2,    // IC6-EL2
     60,   1,    // IP6-EL2
     61,   1,    // IE4-EL2
      2,   1,    // TP13-EL2-O-Start-EL2
      2,   1,    // TP12-EL2-O-Start-EL2
      1,   1,    // I17-EL2
     67,   1,    // IE10-EL2-O-IE9-EL2
      1,   1,    // I18-EL2
     63,   2,    // IC7-EL2
     65,   1,    // IP7-EL2
     66,   1,    // IE9-EL2
     36,   2,    // IC8-EL2-O-IP7-EL2
      1,   1,    // I19-EL2-O-IP7-EL2
     68,   1,    // TP15-EL2-O-Start-EL2
      2,   1,    // TP14-EL2-O-Start-EL2
      1,   1,    // I20-EL2
      1,   1,    // I21-EL2
     73,   1,    // IE12-EL2-O-IE11-EL2
      1,   1,    // I22-EL2
     69,   2,    // IC9-EL2
     71,   1,    // IP8-EL2
     72,   1,    // IE11-EL2
     38,   2,    // IC10-EL2-O-IP8-EL2
      1,   1,    // I23-EL2-O-IP8-EL2
      2,   1,    // TP16-EL2-O-Start-EL2
     74,   1,    // TP17-EL2-O-Start-EL2
      4,   1,    // IS11-EL2-O-I20-EL2
      4,   1,    // IS13-EL2-O-I13-EL2
      4,   1,    // IS14-EL2-O-I16-EL2
      4,   1,    // IS15-EL2-O-I12-EL2
      4,   1,    // IS16-EL2-O-I11-EL2
      4,   1,    // IS17-EL2-O-I9-EL2
      4,   1,    // IS18-EL2-O-I7-EL2
      4,   1,    // IS19-EL2-O-I4-EL2
      4,   1,    // IS20-EL2-O-IE1-EL2
      4,   1,    // IS21-EL2-O-I1-EL2
      4,   1,    // IS22-EL2-O-I20-EL2
      4,   1,    // IS23-EL2-O-I17-EL2
      4,   1,    // IS25-EL2-O-I16-EL2
      4,   1,    // IS26-EL2-O-I12-EL2
      4,   1,    // IS27-EL2-O-I11-EL2
      4,   1,    // IS28-EL2-O-I9-EL2
      4,   1,    // IS29-EL2-O-I7-EL2
      4,   1,    // IS30-EL2-O-I4-EL2
      4,   1,    // IS31-EL2-O-IE1-EL2
      4,   1,    // IS32-EL2-O-I1-EL2
      4,   1,    // IS33-EL2-O-I20-EL2
      4,   1,    // IS34-EL2-O-I17-EL2
      4,   1,    // IS35-EL2-O-I16-EL2
      4,   1,    // IS36-EL2-O-I12-EL2
      4,   1,    // IS37-EL2-O-I11-EL2
      4,   1,    // IS38-EL2-O-I13-EL2
      4,   1,    // IS39-EL2-O-I7-EL2
      4,   1,    // IS40-EL2-O-I4-EL2
      4,   1,    // IS41-EL2-O-IE1-EL2
      4,   1,    // IS42-EL2-O-I1-EL2
      4,   1,    // IS43-EL2-O-I20-EL2
      4,   1,    // IS44-EL2-O-I17-EL2
      4,   1,    // IS45-EL2-O-I16-EL2
      4,   1,    // IS46-EL2-O-I12-EL2
      4,   1,    // IS47-EL2-O-I11-EL2
      4,   1,    // IS48-EL2-O-I13-EL2
      4,   1,    // IS49-EL2-O-I9-EL2
      4,   1,    // IS50-EL2-O-I4-EL2
      4,   1,    // IS51-EL2-O-IE1-EL2
      4,   1,    // IS52-EL2-O-I1-EL2
      4,   1,    // IS53-EL2-O-I7-EL2
      4,   1,    // IS55-EL2-O-I16-EL2
      4,   1,    // IS56-EL2-O-I12-EL2
      4,   1,    // IS57-EL2-O-I11-EL2
      4,   1,    // IS58-EL2-O-I13-EL2
      4,   1,    // IS59-EL2-O-I9-EL2
      4,   1,    // IS60-EL2-O-I4-EL2
      4,   1,    // IS61-EL2-O-IE1-EL2
      4,   1,    // IS62-EL2-O-I1-EL2
      4,   1,    // IS63-EL2-O-I7-EL2
      4,   1,    // IS64-EL2-O-I17-EL2
      4,   1,    // IS65-EL2-O-I16-EL2
      4,   1,    // IS66-EL2-O-I12-EL2
      4,   1,    // IS67-EL2-O-I11-EL2
      4,   1,    // IS68-EL2-O-I13-EL2
      4,   1,    // IS69-EL2-O-I9-EL2
      4,   1,    // IS70-EL2-O-I20-EL2
      4,   1,    // IS71-EL2-O-IE1-EL2
      4,   1,    // IS72-EL2-O-I1-EL2
      0,   1,    // Start-EL3
      3,   1,    // IE1-EL3
     40,   2,    // TC1-EL3-O-Start-EL3
      2,   1,    // TP1-EL3-O-Start-EL3
      1,   1,    // I1-EL3
      6,   1,    // IS1-EL3-O-IE1-EL2
      6,   1,    // IS2-EL3-O-I4-EL2
      6,   1,    // IS3-EL3-O-I7-EL2
      6,   1,    // IS4-EL3-O-I9-EL2
      6,   1,    // IS5-EL3-O-I11-EL2
      6,   1,    // IS6-EL3-O-I12-EL2
      6,   1,    // IS7-EL3-O-I16-EL2
      6,   1,    // IS8-EL3-O-I13-EL2
      6,   1,    // IS9-EL3-O-I17-EL2
      6,   1,    // IS10-EL3-O-I20-EL2
      7,   2,    // TC2-EL3-O-Start-EL2
      1,   1,    // I2-EL3
     13,   1,    // ID1-EL3-O-I11-EL1
     13,   1,    // ID2-EL3-O-I9-EL1
     13,   1,    // ID3-EL3-O-I7-EL1
     13,   1,    // ID4-EL3-O-I4-EL1
     13,   1,    // ID5-EL3-O-IE1-EL1
     13,   1,    // ID6-EL3-O-I12-EL1
     13,   1,    // ID7-EL3-O-I16-EL1
     13,   1,    // ID8-EL3-O-I13-EL1
     13,   1,    // ID9-EL3-O-I17-EL1
     13,   1,    // ID10-EL3-O-I20-EL1
      9,   2,    // TC3-EL3-O-Start-EL1
      1,   1,    // I3-EL3
     14,   1,    // IT1-EL3-O-I11
     14,   1,    // IT2-EL3-O-I9
     14,   1,    // IT3-EL3-O-I7
     14,   1,    // IT4-EL3-O-I4
     14,   1,    // IT5-EL3-O-IE1
     14,   1,    // IT6-EL3-O-I12
     14,   1,    // IT7-EL3-O-I16
     14,   1,    // IT8-EL3-O-I13
     14,   1,    // IT9-EL3-O-I17
     14,   1,    // IT10-EL3-O-I20
     11,   2,    // TC4-EL3-O-Start
      2,   1,    // TP2-EL3-O-Start-EL3
      2,   1,    // IP1-EL3-O-Start-EL3
      2,   1,    // TP19-EL3-O-Start-EL3
      1,   1,    // I4-EL3
      1,   1,    // I5-EL3
     17,   1,    // ID11-EL3-O-I11-EL3
     17,   1,    // ID12-EL3-O-I9-EL3
     17,   1,    // ID13-EL3-O-I7-EL3
     17,   1,    // ID14-EL3-O-I1-EL3
     17,   1,    // ID15-EL3-O-IE1-EL3
     17,   1,    // ID16-EL3-O-I12-EL3
     17,   1,    // ID17-EL3-O-I16-EL3
     17,   1,    // ID18-EL3-O-I13-EL3
     17,   1,    // ID19-EL3-O-I17-EL3
     17,   1,    // ID20-EL3-O-I20-EL3
     15,   2,    // TC5-EL3-O-Start-EL3
      1,   1,    // I6-EL3
     20,   1,    // IT11-EL3-O-I11-EL3
     20,   1,    // IT12-EL3-O-I9-EL3
     20,   1,    // IT13-EL3-O-I7-EL3
     20,   1,    // IT14-EL3-O-I2-EL3
     20,   1,    // IT15-EL3-O-IE1-EL3
     20,   1,    // IT16-EL3-O-I12-EL3
     20,   1,    // IT17-EL3-O-I16-EL3
     20,   1,    // IT18-EL3-O-I13-EL3
     20,   1,    // IT19-EL3-O-I17-EL3
     20,   1,    // IT20-EL3-O-I20-EL3
     18,   2,    // TC6-EL3-O-Start-EL3
      1,   1,    // I24-EL3
      5,   1,    // IF5-EL3-O-I11-EL3
      5,   1,    // IF4-EL3-O-I9-EL3
      5,   1,    // IF3-EL3-O-I7-EL3
      5,   1,    // IF2-EL3-O-I2-EL3
      5,   1,    // IF1-EL3-O-IE1-EL3
      5,   1,    // IF6-EL3-O-I12-EL3
      5,   1,    // IF7-EL3-O-I16-EL3
      5,   1,    // IF8-EL3-O-I13-EL3
      5,   1,    // IF9-EL3-O-I17-EL3
      5,   1,    // IF10-EL3-O-I20-EL3
      2,   1,    // TP18-EL3-O-Start-EL3
      1,   1,    // I25-EL3
     24,   2,    // IF15-EL3-O-I11-EL3
     24,   2,    // IF14-EL3-O-I9-EL3
     24,   2,    // IF13-EL3-O-I7-EL3
     24,   2,    // IF12-EL3-O-I2-EL3
     24,   2,    // IF11-EL3-O-IE1-EL3
     24,   2,    // IF16-EL3-O-I12-EL3
     24,   2,    // IF17-EL3-O-I16-EL3
     24,   2,    // IF18-EL3-O-I13-EL3
     24,   2,    // IF19-EL3-O-I17-EL3
     24,   2,    // IF20-EL3-O-I20-EL3
     21,   3,    // TC9-EL3-O-Start-EL3
      2,   1,    // TP3-EL3-O-Start-EL3
      1,   1,    // I7-EL3
      1,   1,    // I8-EL3
     28,   1,    // ID21-EL3-O-I11-EL3
     28,   1,    // ID22-EL3-O-I9-EL3
     28,   1,    // ID23-EL3-O-I4-EL3
     28,   1,    // ID24-EL3-O-I1-EL3
     28,   1,    // ID25-EL3-O-IE1-EL3
     28,   1,    // ID26-EL3-O-I12-EL3
     28,   1,    // ID27-EL3-O-I16-EL3
     28,   1,    // ID28-EL3-O-I13-EL3
     28,   1,    // ID29-EL3-O-I17-EL3
     28,   1,    // ID30-EL3-O-I20-EL3
     26,   2,    // TC7-EL3-O-Start-EL3
      2,   1,    // TP4-EL3-O-Start-EL3
      1,   1,    // I9-EL3
      1,   1,    // I10-EL3
     31,   1,    // ID31-EL3-O-I11-EL3
     31,   1,    // ID32-EL3-O-I7-EL3
     31,   1,    // ID33-EL3-O-I4-EL3
     31,   1,    // ID34-EL3-O-I1-EL3
     31,   1,    // ID35-EL3-O-IE1-EL3
     31,   1,    // ID36-EL3-O-I12-EL3
     31,   1,    // ID37-EL3-O-I16-EL3
     31,   1,    // ID38-EL3-O-I13-EL3
     31,   1,    // ID39-EL3-O-I17-EL3
     31,   1,    // ID40-EL3-O-I20-EL3
     29,   2,    // TC8-EL3-O-Start-EL3
      2,   1,    // TP5-EL3-O-Start-EL3
     46,   1,    // IE8-EL3-O-IP4-EL3
      1,   1,    // I11-EL3
     42,   2,    // IC1-EL3
     45,   1,    // IP3-EL3
     45,   1,    // IP4-EL3
     44,   1,    // TP7-EL3-O-Start-EL3
      2,   1,    // TP6-EL3-O-Start-EL3
     52,   1,    // IE7-EL3-O-IE2-EL3
      1,   1,    // I12-EL3
     47,   2,    // IC2-EL3
     50,   1,    // IP2-EL3
     51,   1,    // IE2-EL3
     32,   2,    // IC3-EL3-O-IP2-EL3
      2,   1,    // TP8-EL3-O-Start-EL3
     49,   1,    // TP9-EL3-O-Start-EL3
      1,   1,    // I13-EL3
     52,   1,    // IE6-EL3-O-IE3-EL3
      1,   1,    // I14-EL3
     53,   2,    // IC4-EL3
     56,   1,    // IP5-EL3
     57,   1,    // IE3-EL3
     34,   2,    // IC5-EL3-O-IP5-EL3
      1,   1,    // I15-EL3-O-IP5-EL3
     55,   1,    // TP11-EL3-O-Start-EL3
      2,   1,    // TP10-EL3-O-Start-EL3
     62,   1,    // IE5-EL3-O-IE4-EL3
      1,   1,    // I16-EL3
     58,   2,    // IC6-EL3
     60,   1,    // IP6-EL3
     61,   1,    // IE4-EL3
      2,   1,    // TP13-EL3-O-Start-EL3
      2,   1,    // TP12-EL3-O-Start-EL3
      1,   1,    // I17-EL3
     67,   1,    // IE10-EL3-O-IE9-EL3
      1,   1,    // I18-EL3
     63,   2,    // IC7-EL3
     65,   1,    // IP7-EL3
     66,   1,    // IE9-EL3
     36,   2,    // IC8-EL3-O-IP7-EL3
      1,   1,    // I19-EL3-O-IP7-EL3
     68,   1,    // TP15-EL3-O-Start-EL3
      2,   1,    // TP14-EL3-O-Start-EL3
      1,   1,    // I20-EL3
      1,   1,    // I21-EL3
     73,   1,    // IE12-EL3-O-IE11-EL3
      1,   1,    // I22-EL3
     69,   2,    // IC9-EL3
     71,   1,    // IP8-EL3
     72,   1,    // IE11-EL3
     38,   2,    // IC10-EL3-O-IP8-EL3
      1,   1,    // I23-EL3-O-IP8-EL3
      2,   1,    // TP16-EL3-O-Start-EL3
     74,   1,    // TP17-EL3-O-Start-EL3
      4,   1,    // IS11-EL3-O-I20-EL3
      4,   1,    // IS13-EL3-O-I13-EL3
      4,   1,    // IS14-EL3-O-I16-EL3
      4,   1,    // IS15-EL3-O-I12-EL3
      4,   1,    // IS16-EL3-O-I11-EL3
      4,   1,    // IS17-EL3-O-I9-EL3
      4,   1,    // IS18-EL3-O-I7-EL3
      4,   1,    // IS19-EL3-O-I4-EL3
      4,   1,    // IS20-EL3-O-IE1-EL3
      4,   1,    // IS21-EL3-O-I1-EL3
      4,   1,    // IS22-EL3-O-I20-EL3
      4,   1,    // IS23-EL3-O-I17-EL3
      4,   1,    // IS25-EL3-O-I16-EL3
      4,   1,    // IS26-EL3-O-I12-EL3
      4,   1,    // IS27-EL3-O-I11-EL3
      4,   1,    // IS28-EL3-O-I9-EL3
      4,   1,    // IS29-EL3-O-I7-EL3
      4,   1,    // IS30-EL3-O-I4-EL3
      4,   1,    // IS31-EL3-O-IE1-EL3
      4,   1,    // IS32-EL3-O-I1-EL3
      4,   1,    // IS33-EL3-O-I20-EL3
      4,   1,    // IS34-EL3-O-I17-EL3
      4,   1,    // IS35-EL3-O-I16-EL3
      4,   1,    // IS36-EL3-O-I12-EL3
      4,   1,    // IS37-EL3-O-I11-EL3
      4,   1,    // IS38-EL3-O-I13-EL3
      4,   1,    // IS39-EL3-O-I7-EL3
      4,   1,    // IS40-EL3-O-I4-EL3
      4,   1,    // IS41-EL3-O-IE1-EL3
      4,   1,    // IS42-EL3-O-I1-EL3
      4,   1,    // IS43-EL3-O-I20-EL3
      4,   1,    // IS44-EL3-O-I17-EL3
      4,   1,    // IS45-EL3-O-I16-EL3
      4,   1,    // IS46-EL3-O-I12-EL3
      4,   1,    // IS47-EL3-O-I11-EL3
      4,   1,    // IS48-EL3-O-I13-EL3
      4,   1,    // IS49-EL3-O-I9-EL3
      4,   1,    // IS50-EL3-O-I4-EL3
      4,   1,    // IS51-EL3-O-IE1-EL3
      4,   1,    // IS52-EL3-O-I1-EL3
      4,   1,    // IS53-EL3-O-I7-EL3
      4,   1,    // IS55-EL3-O-I16-EL3
      4,   1,    // IS56-EL3-O-I12-EL3
      4,   1,    // IS57-EL3-O-I11-EL3
      4,   1,    // IS58-EL3-O-I13-EL3
      4,   1,    // IS59-EL3-O-I9-EL3
      4,   1,    // IS60-EL3-O-I4-EL3
      4,   1,    // IS61-EL3-O-IE1-EL3
      4,   1,    // IS62-EL3-O-I1-EL3
      4,   1,    // IS63-EL3-O-I7-EL3
      4,   1,    // IS64-EL3-O-I17-EL3
      4,   1,    // IS65-EL3-O-I16-EL3
      4,   1,    // IS66-EL3-O-I12-EL3
      4,   1,    // IS67-EL3-O-I11-EL3
      4,   1,    // IS68-EL3-O-I13-EL3
      4,   1,    // IS69-EL3-O-I9-EL3
      4,   1,    // IS70-EL3-O-I20-EL3
      4,   1,    // IS71-EL3-O-IE1-EL3
      4,   1,    // IS72-EL3-O-I1-EL3
];
