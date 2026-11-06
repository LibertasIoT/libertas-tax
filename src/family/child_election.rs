//! Form 8814: a parent's election to report a child's interest and dividends. Amounts entering arithmetic are whole dollars.

// Coverage: Form 8814 — Supported (audited 2026-10-09).
//   Lines: A (the child's name, the row's label), C (box C, printed only), 1a–15 per child. The
//   page is a document kind of the Interest and dividends topic (IncomeRecord::Form8814,
//   FederalIncome::ChildElection), offered only when the return has a child who can be reported
//   (the hidden children list of SaveIncomes, EstimateDocuments and PreviewDocument: Relationship::Child,
//   under 19 or a full-time student under 24 at the end of 2026 by day-exact age, no joint return of
//   their own). Each child's 1099-INT, -DIV
//   and -OID statements are the filer's own importable structs (Int1099, Div1099, Oid1099), so AI
//   import reads them per statement; the Alaska Permanent Fund dividend, whose return reports the
//   child's income (OtherParentChoice) and the facts no statement shows (ChildElectionFacts) are
//   the person's. Line 1a is the statements' own per-payer netting (FederalInterest::net_taxable,
//   FederalOid::net_taxable); line 1b is the child's tax-exempt interest (its private-activity part
//   is the parent's Form 6251 line 2g preference and Pub 596 Worksheet 1 line 2); line 4 over $1,350
//   and under $13,500 reports the child (at or below $1,350 nothing is reported and
//   NOTE_CHILD_ELECTION_NOT_REQUIRED says so); lines 6-12 split the excess over $2,700 by the
//   dividend and capital gain distribution fractions (line 9 to Form 1040 lines 3a and 3b, line 10 to
//   Schedule D line 13 with its 28% and unrecaptured section 1250 shares for the worksheets, line 12
//   to Schedule 1 line 8z); line 15 (10% of the excess over $1,350, at most $135) joins Form 1040
//   line 16 and is never a credit base, Form 8615 line 10 or Schedule J income. Line 12 less the
//   Alaska Permanent Fund share (Pub 596 Worksheet 2) is the Form 8960 line 7 amount, Form 4952
//   investment income and the earned income credit's Worksheet 1 line 4 amount; the increased
//   AGI reduces every AGI-based deduction and credit through the shared AGI.
//   Left out with one notice naming the child (the election is optional, so the parent's return
//   without it is correct and the child files their own): line 4 at or above $13,500, any federal
//   withholding, any statement with a special treatment or nominee amount, other income or
//   estimated payments, a noncustodial parent (joint return or LivedApart with the child in the home
//   under 183 days), the other parent's return when their taxable income is higher, and an unresolved
//   choice of return (assumed the other parent's). Kept with a note: a foreign account or trust
//   (Schedule B Part III and Form 8938 are not prepared) and a Form 4952 deduction (the child's
//   qualified dividends and capital gain distributions are not offered for the line 4g election).
//   Not prepared: the child's own blind addition, early withdrawal penalty and charitable gifts
//   (the early withdrawal penalty box is ignored), the comparison with the child's own Form 8615 tax
//   (a later unit), printed Form 8814 and Form 1040 line 3c / 7b / 16 box 1 checkboxes (the return
//   has no checked-box line kind; box C is printed only). Form 8615 stays independent: a child
//   reported here files no return, a filer's own Form 8615 parent page is unchanged.
//   Evidence: Form 8814 and instructions (2026 draft, retrieved 2026-10-08), Rev. Proc. 2025-32
//   section 3.02, IRC 1(g)(7), Pub 596 Worksheets 1 and 2, i8960 "Form 8814 election", i6251 line 2g;
//   year_constants.rs CHILD_ELECTION_*; src/tests/family/family.rs child_election_* (fixtures A-E of the
//   map), src/tests/investment/investment.rs child_election_import_*.

pub use crate::child_election_source::{ChildElectionFacts, OtherParentChoice};
use crate::child_election_source::{ChildIncome, ChildStatementKept, FederalChildElection};
pub(crate) use crate::child_election_source::{ChildLines, eligible_child};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Child's interest and dividends (Form 8814)
/// Report your child's interest, dividends and capital gain distributions on your return instead of filing a return for the child: a child under 19, or a full-time student under 24, at the end of 2026, whose only income was interest and dividends. Income between $1,350 and $2,700 is taxed at 10% here even if the child's own rate would be lower.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Form8814 {
    /// Child
    #[libertas_enum_source("$.children")]
    pub child: i64,
    /// The child's statements
    /// Forms 1099-INT, 1099-DIV and 1099-OID issued in the child's name and Social Security number, not yours.
    #[libertas_unordered]
    /// ----
    /// Statement
    pub statements: Vec<ChildStatement>,
    /// Alaska Permanent Fund dividend paid to the child
    /// The 2026 dividend from the State of Alaska. It counts as the child's ordinary dividends; it is not on a Form 1099-DIV.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub alaska_dividend: i64,
    /// Child's Social Security benefits that are not taxable
    /// Include the nontaxable portion for Marketplace household income. This is not taxable income on your Form 8814. Do not enter it again as a required-to-file dependent return.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub nontaxable_social: i64,
    /// Whose return reports this child's income
    /// Asked only when this is not a joint return. The child's other parent is the person who did not file this return; a new spouse of the parent the child lives with counts as the other parent.
    #[libertas_constrained_by("$.allowed_other_parent")]
    pub other_parent: OtherParentChoice,
    /// Does any of these apply to this child?
    #[libertas_bitflags(ChildElectionFacts)]
    pub facts: u32,
}

/// Statement
#[derive(
    Clone,
    Debug,
    PartialEq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
#[allow(clippy::large_enum_variant)]
pub enum ChildStatement {
    /// 1099-INT
    Int1099(crate::Form1099IntSource),
    /// 1099-DIV
    Div1099(ChildDividendStatement),
    /// 1099-OID
    Oid1099(crate::Form1099OidSource),
}

/// Child's dividend statement
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ChildDividendStatement {
    /// Accepted statement identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Form 1099-DIV as printed
    pub document: crate::Form1099DivPaper,
}
impl ChildDividendStatement {
    pub(crate) fn source(&self) -> crate::DividendSource {
        crate::DividendSource::printed(self.document.clone())
    }
}

// One child who can be reported on this return. The label is the child's
// name as a read-only FormattedText, the way PersonChoice labels a person;
// the stored child identity is the key, so a catalog change never moves a
// row to another child.
/// Child choice
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ChildChoice {
    /// Child identity
    #[libertas_key]
    #[libertas_unique(1)]
    #[libertas_hidden]
    pub id: i64,
    /// Child
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
}

impl Form8814 {
    // Each statement uses its shared 1099 record mapping; dividend paper and
    // personal answers stay together in their existing source owner.
    pub(crate) fn record(&self) -> FederalChildElection {
        use crate::interview::{interest_record, oid_record};
        FederalChildElection {
            child_id: self.child,
            statements: self
                .statements
                .iter()
                .map(|statement| match statement {
                    ChildStatement::Int1099(v) => ChildIncome::Interest {
                        data: interest_record(v).into(),
                        kept: ChildStatementKept {
                            payer: v.form.payer.clone(),
                            recipient: v.form.recipient.clone(),
                            state_details: v.form.state_details.clone(),
                        },
                    },
                    ChildStatement::Div1099(v) => {
                        ChildIncome::Dividend { statement_id: v.id, data: v.source().into() }
                    }
                    ChildStatement::Oid1099(v) => ChildIncome::Oid {
                        data: oid_record(v).into(),
                        kept: ChildStatementKept {
                            payer: v.form.payer.clone(),
                            recipient: v.form.recipient.clone(),
                            state_details: v.form.state_details.clone(),
                        },
                    },
                })
                .collect(),
            alaska_dividend: self.alaska_dividend,
            nontaxable_social: self.nontaxable_social,
            other_parent: self.other_parent,
            facts: self.facts,
        }
    }
    // The stored election as its page row.
    pub(crate) fn from_record(record: &FederalChildElection) -> Self {
        use crate::interview::{interest_page, oid_page};
        Self {
            child: record.child_id,
            statements: record
                .statements
                .iter()
                .map(|statement| match statement {
                    ChildIncome::Interest { data, kept } => {
                        ChildStatement::Int1099(interest_page(data, kept))
                    }
                    ChildIncome::Dividend { statement_id, data } => {
                        ChildStatement::Div1099(ChildDividendStatement {
                            id: *statement_id,
                            document: data.form.clone(),
                        })
                    }
                    ChildIncome::Oid { data, kept } => {
                        ChildStatement::Oid1099(oid_page(data, kept))
                    }
                })
                .collect(),
            alaska_dividend: record.alaska_dividend,
            nontaxable_social: record.nontaxable_social,
            other_parent: record.other_parent,
            facts: record.facts,
        }
    }
}

// One election's Form 8814 lines on the return, each one line of the form
// (a line with nothing on it is left off, as the other forms leave theirs).
pub(crate) fn report(r: &mut crate::FederalResult, id: i64, lines: &ChildLines) {
    use crate::line_emission::form_line;
    use crate::{FormLine, TaxForm};
    const SOURCE: &str = "https://www.irs.gov/instructions/i8814";
    let rows: [(&str, &str, i64, bool, crate::Message); 13] = [
        ("1a", "interest", lines.line_1a, false, msg!("EXPLAIN_CHILD_ELECTION_N_INTEREST")),
        (
            "1b",
            "exempt_interest",
            lines.line_1b,
            false,
            msg!("EXPLAIN_CHILD_ELECTION_N_EXEMPT_INTEREST"),
        ),
        (
            "2a",
            "ordinary_dividends",
            lines.line_2a,
            false,
            msg!("EXPLAIN_CHILD_ELECTION_N_ORDINARY_DIVIDENDS"),
        ),
        (
            "2b",
            "qualified_dividends",
            lines.line_2b,
            false,
            msg!("EXPLAIN_CHILD_ELECTION_N_QUALIFIED_DIVIDENDS"),
        ),
        (
            "3",
            "capital_gain_distributions",
            lines.line_3,
            false,
            msg!("EXPLAIN_CHILD_ELECTION_N_CAPITAL_GAIN_DISTRIBUTIONS"),
        ),
        ("4", "total_income", lines.line_4, true, msg!("EXPLAIN_CHILD_ELECTION_N_TOTAL_INCOME")),
        (
            "6",
            "income_over_base",
            lines.line_6,
            false,
            msg!("EXPLAIN_CHILD_ELECTION_N_INCOME_OVER_BASE"),
        ),
        (
            "9",
            "qualified_dividends_reported",
            lines.line_9,
            false,
            msg!("EXPLAIN_CHILD_ELECTION_N_QUALIFIED_DIVIDENDS_REPORTED"),
        ),
        (
            "10",
            "capital_gain_distributions_reported",
            lines.line_10,
            false,
            msg!("EXPLAIN_CHILD_ELECTION_N_CAPITAL_GAIN_DISTRIBUTIONS_REPORTED"),
        ),
        (
            "11",
            "reported_total",
            lines.line_11,
            false,
            msg!("EXPLAIN_CHILD_ELECTION_N_REPORTED_TOTAL"),
        ),
        ("12", "other_income", lines.line_12, false, msg!("EXPLAIN_CHILD_ELECTION_N_OTHER_INCOME")),
        (
            "14",
            "taxable_over_untaxed",
            lines.line_14,
            true,
            msg!("EXPLAIN_CHILD_ELECTION_N_TAXABLE_OVER_UNTAXED"),
        ),
        ("15", "tax", lines.line_15, true, msg!("EXPLAIN_CHILD_ELECTION_N_TAX")),
    ];
    for (line, name, amount, always, explanation) in rows {
        if amount != 0 || always {
            form_line(
                r,
                FormLine::new(TaxForm::Form8814, line),
                &alloc::format!("child_election_{id}_{name}"),
                amount,
                &["income.child_election"],
                SOURCE,
                explanation,
            );
        }
    }
}

impl FederalChildElection {
    // Stored values inside the limits of the filer's own statements.
    pub(crate) fn valid(&self) -> bool {
        use crate::FederalIncome as F;
        let ok = |income: F| crate::federal_rules::validate_income_amount(&income).is_ok();
        self.statements.len() <= 20
            && (0..=100000000000).contains(&self.alaska_dividend)
            && (0..=100000000000).contains(&self.nontaxable_social)
            && self.facts
                & !(ChildElectionFacts::OTHER_INCOME.0
                    | ChildElectionFacts::ESTIMATED_PAYMENTS.0
                    | ChildElectionFacts::NOMINEE.0
                    | ChildElectionFacts::FOREIGN_ACCOUNT.0)
                == 0
            && self.statements.iter().all(|statement| match statement {
                ChildIncome::Interest { data, kept } => {
                    kept.valid()
                        && ok(F::Interest {
                            data: crate::InterestSource::Record(data.as_ref().clone()),
                        })
                }
                ChildIncome::Dividend { data, .. } => {
                    crate::IncomeStateSource::Investment(data.form.state_details.clone()).valid()
                        && crate::federal_rules::validate_income_amount_issue_with_mode(
                            &F::Dividend { data: data.as_ref().clone() },
                            crate::year_constants::TAX_YEAR,
                            false,
                        )
                        .is_ok()
                }
                ChildIncome::Oid { data, kept } => {
                    kept.valid() && ok(F::Oid { data: crate::interview::oid_page(data, kept) })
                }
            })
    }
}
