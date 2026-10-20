// Coverage: Form 8997 and qualified opportunity funds (B9-5, 2026-10-10). QofInvestment (an
//   investment made from December 22, 2017 through 2025 whose deferral ends December 31, 2026:
//   one row of Part IV of the 2025 Form 8997) and QofDeferral (a 2026 gain invested in a fund in
//   2027 within 180 days, Form 8997 Part II). The remaining deferred gain is included on the
//   earlier of the sale of the investment and December 31, 2026 (IRC 1400Z-2(b)(1)(B)): the
//   smaller of the deferred gain and the investment's value, less the basis increase (10 percent of
//   the deferred gain at five years, a further 5 percent at seven; (b)(2)(B)(iii), (iv); Reg.
//   1.1400Z2(b)-1(e)(3), (e)(5), (g)(2)), with the character of the deferred gain (short-term,
//   long-term, section 1231 by special gain code B on Form 4797 Part I, collectibles by code D at
//   28 percent) shared pro rata. Form 8949 code Y rows (Part I box C, Part II box F), the O/Y pair
//   of a section 1231 gain, and code Z rows (the deferral, negative column (g), capped by the
//   return's positive 8949 rows of that character, ISSUE_QOF_DEFERRAL_ABOVE_GAINS). The basis after
//   the inclusion (Form 8997 Part V) is shown for the fund interest's own sale entry; the 10-year
//   election cannot be reached before December 22, 2027. Left out with a notice: special gain
//   codes A, C, E and F, partial sales, distributions, gifts and pass-through holdings
//   (WARN_QOF_INVESTMENT_LEFT_OUT, WARN_QOF_DEFERRAL_LEFT_OUT); a deferral whose 180 days passed
//   defers nothing (WARN_QOF_DEFERRAL_NOT_ALLOWED); an unknown year-end value taxes the whole
//   deferred gain (WARN_QOF_VALUE_ASSUMED). The 180 days cannot be checked (the model keeps no sale
//   dates); the person confirms. Forms: 8997, 8949 (codes O, Y, Z), 4797 Part I, Schedule D.
//   Tests: src/tests/investment/investment.rs qof_*.
use crate::capital::Form8949Row;
use crate::kernel::phase;
use crate::year_constants as yc;
use crate::{FederalDraft, FederalIncome, FederalIncomeEntry};
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

const MAX: i64 = 100_000_000_000;

/// Qualified opportunity fund investment (Form 8997)
/// One for each investment in a qualified opportunity fund made before 2026 that still defers a gain: one row of Part IV of your 2025 Form 8997. The deferral ends on December 31, 2026, so the deferred gain is taxed this year, or when you sold the investment if that was earlier.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read one row of Part IV (QOF investments with deferrals held at year end) of a 2025 Form 8997, or one row of Part I of a 2026 Form 8997, exactly. If the document shows more than one row, report an error that says how many rows it shows and that each row needs its own opportunity fund entry; never add, combine, or choose among rows. Report an illegible value, or two values that conflict, as an error. Return amounts as decimal numbers without commas or currency signs. Never infer a value that is not printed on the form. Return column (c), the complete description of the QOF investment, as fund without truncation; report an error when it is blank. Return column (b), the date the QOF investment was acquired, as invested_on; report an error when it is blank. Read column (e) short-term deferred gain into short_term and column (f) long-term deferred gain into long_term, returning an empty string for a blank column; report an error when both are blank. For column (d), the special gain code, return None when it is blank, Section1231 for code B, Collectibles for code D, and Other for code A, C, E or F. Do not read column (a), the fund's EIN, any other part of the form, or the totals lines. Do not report whether the investment was sold, its value, or whether a special treatment applies; the filer answers those.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*"
)]
pub struct QofInvestment {
    /// Fund and investment (Form 8997 column (c))
    /// For example "100 shares of Example Opportunity Fund".
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_size(min = 1)]
    pub fund: String,
    /// Date you invested (column (b))
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    // Tax structure: this entry reports QOF deferrals carried into 2026 from investments made through December 31, 2025.
    #[libertas_number(min = $QOF_FIRST_INVESTMENT_DATE, max = $QOF_LAST_EARLIER_INVESTMENT_DATE)]
    pub invested_on: u32,
    /// Short-term gain still deferred (column (e))
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub short_term: i64,
    /// Long-term gain still deferred (column (f))
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub long_term: i64,
    /// Special gain code (column (d))
    #[libertas_llm_doc_field]
    #[libertas_default(None)]
    pub gain_code: QofGainCode,
    /// What happened to the investment in 2026
    pub event: QofEvent,
    /// A partial sale, a distribution from the fund, a gift of the investment, a fund interest held through a partnership or S corporation, or another special treatment
    /// Yes leaves this investment out of the estimate with a notice; that treatment is not supported.
    #[libertas_default(false)]
    pub special: bool,
}

/// Special gain code
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum QofGainCode {
    /// No code
    None,
    /// B: a section 1231 gain from Form 4797
    /// Taxed as a section 1231 gain on Form 4797 when included.
    Section1231,
    /// D: a collectibles gain
    /// Taxed at up to 28% when included.
    Collectibles,
    /// A, C, E or F
    /// A section 1256 gain, a straddle, a gain exempt by treaty, or an investment you received from someone else in a transfer that was not an inclusion event, such as an inheritance. Leaves this investment out of the estimate with a notice; that treatment is not supported.
    Other,
}

/// What happened in 2026
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum QofEvent {
    /// Still held on December 31, 2026
    Held {
        /// Fair market value of the investment on December 31, 2026
        /// From the fund's year-end statement. If it is less than the deferred gain, only the value less your basis increase is taxed. If the value is unknown, the App uses the deferred gain as the assumed value before subtracting any eligible basis increase.
        /// [NullText]
        /// Use the deferred gain as the assumed year-end value
        /// [SetText]
        /// The investment's December 31, 2026 fair market value is
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default($null)]
        fair_value: Option<i64>,
    },
    /// Sold in full in 2026
    /// Also enter the sale of the fund interest as a 1099-B or 1099-DA sale, with the basis the estimate shows for this investment after the inclusion.
    Sold {
        /// Date sold
        #[libertas_date_only]
        // Tax structure: this source event must occur on or before the end of the 2026 return year.
        #[libertas_number(min = 20260101, max = 20261231)]
        sold_on: u32,
        /// Amount realized
        /// What you received for the whole investment, the value used in place of its fair market value.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        amount: i64,
    },
}

/// 2026 gain invested in an opportunity fund in 2027
/// A capital gain from a 2026 sale that you invested in a qualified opportunity fund in 2027, within 180 days of the sale, and elect to defer (Form 8997 Part II, Form 8949 code Z). One for each fund and investment date. An investment made in 2026 defers nothing past December 31, 2026; do not enter it.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct QofDeferral {
    /// Fund and investment
    /// For example "100 shares of Example Opportunity Fund".
    #[libertas_size(min = 1)]
    pub fund: String,
    /// Date you invested
    #[libertas_date_only]
    // Tax structure: a qualifying 2026 gain can reach the QOF 180-day reinvestment deadline no later than June 28, 2027.
    #[libertas_number(min = $QOF_AMENDED_FIRST_INVESTMENT_DATE, max = $QOF_DEFERRAL_LAST_INVESTMENT_DATE)]
    pub invested_on: u32,
    /// Short-term gain deferred
    /// Up to your 2026 short-term capital gains on this page.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub short_term: i64,
    /// Long-term gain deferred
    /// Up to your 2026 long-term capital gains on this page.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub long_term: i64,
    /// You invested within 180 days after the sale that produced each gain deferred
    /// The 180 days begin on the sale's trade date. No means no gain can be deferred by this investment.
    pub within_180_days: bool,
    /// A section 1231, section 1256, straddle or collectibles gain, a gain passed through a partnership, S corporation, estate or trust, or another special treatment
    /// Yes leaves this deferral out of the estimate with a notice; that treatment is not supported.
    #[libertas_default(false)]
    pub special: bool,
}

// What the inclusion of one investment's deferred gain makes of it, in cents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Inclusion {
    // The remaining deferred gain, short-term and long-term.
    pub deferred: i64,
    // The basis increase for the years held (Form 8997 Part III).
    pub increase: i64,
    // The gain included: the smaller of the deferred gain and the value, less the increase.
    pub included: i64,
    pub short: i64,
    // The long-term capital part (neither section 1231 nor collectibles gain code excluded).
    pub long: i64,
    pub section_1231: i64,
    // The collectibles part of `long` (28 percent rate).
    pub collectibles: i64,
    // The basis after the inclusion (Form 8997 Part V).
    pub basis_after: i64,
    // The year-end value was not entered and the whole deferred gain stands in for it.
    pub value_assumed: bool,
}

impl QofEvent {
    fn valid(&self) -> bool {
        match self {
            Self::Held { fair_value } => fair_value.is_none_or(|v| (0..=MAX).contains(&v)),
            Self::Sold { sold_on, amount } => {
                crate::federal_rules::valid_date(*sold_on)
                    && sold_on / 10000 == 2026
                    && *sold_on <= yc::QOF_DEFERRAL_END_DATE
                    && (0..=MAX).contains(amount)
            }
        }
    }
}

impl QofInvestment {
    // The first relationship between this page's fields that fails, by its rule's key, in the
    // order of the client rules.
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }

    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        if i128::from(self.short_term) + i128::from(self.long_term) <= 0 {
            return Some(
                crate::validation::ValidationIssue::new("QOF_NO_DEFERRED_GAIN")
                    .at_field("short_term"),
            );
        }
        if self.gain_code == QofGainCode::Section1231 && self.short_term != 0 {
            return Some(
                crate::validation::ValidationIssue::new("QOF_SECTION_1231_LONG_TERM")
                    .at_field("short_term"),
            );
        }
        if self.gain_code == QofGainCode::Collectibles && self.short_term != 0 {
            return Some(
                crate::validation::ValidationIssue::new("QOF_COLLECTIBLES_LONG_TERM")
                    .at_field("short_term"),
            );
        }
        if let QofEvent::Sold { sold_on, .. } = self.event
            && sold_on <= self.invested_on
        {
            return Some(
                crate::validation::ValidationIssue::new("QOF_SOLD_BEFORE_INVESTED")
                    .at_field("sold_on")
                    .in_field("event"),
            );
        }
        None
    }

    pub(crate) fn valid(&self) -> bool {
        !self.fund.is_empty()
            && crate::federal_rules::valid_date(self.invested_on)
            && (yc::QOF_FIRST_INVESTMENT_DATE..=yc::QOF_LAST_EARLIER_INVESTMENT_DATE)
                .contains(&self.invested_on)
            && (0..=MAX).contains(&self.short_term)
            && (0..=MAX).contains(&self.long_term)
            && self.event.valid()
            && self.rule().is_none()
    }

    // Decision 2: a special gain code the App cannot compute (A, C, E, F) or any other special
    // treatment leaves the investment out with a notice.
    pub(crate) fn left_out(&self) -> bool {
        self.special || self.gain_code == QofGainCode::Other
    }

    // The inclusion on the earlier of the sale and December 31, 2026 (IRC 1400Z-2(b)(1)(B)).
    pub(crate) fn inclusion(&self) -> Inclusion {
        let deferred = self.short_term + self.long_term;
        let (five, seven) = match self.event {
            QofEvent::Held { .. } => (
                self.invested_on <= yc::QOF_FIVE_YEAR_LAST_INVESTMENT_DATE,
                self.invested_on <= yc::QOF_SEVEN_YEAR_LAST_INVESTMENT_DATE,
            ),
            QofEvent::Sold { sold_on, .. } => {
                (self.invested_on + 50000 <= sold_on, self.invested_on + 70000 <= sold_on)
            }
        };
        let percent = if five { yc::QOF_FIVE_YEAR_PERCENT } else { 0 }
            + if seven { yc::QOF_SEVEN_YEAR_PERCENT } else { 0 };
        let increase = crate::kernel::ratio::fraction(deferred, percent, 100);
        let (value, value_assumed) = match self.event {
            QofEvent::Held { fair_value: Some(value) } => (value, false),
            QofEvent::Held { fair_value: None } => (deferred, true),
            QofEvent::Sold { amount, .. } => (amount, false),
        };
        let included = phase::floor(phase::cap(deferred, [value]) - increase, 0);
        let short = if deferred == 0 {
            0
        } else {
            crate::kernel::ratio::fraction(included, self.short_term, deferred)
        };
        let long_part = included - short;
        let (long, section_1231, collectibles) = match self.gain_code {
            QofGainCode::Section1231 => (0, long_part, 0),
            QofGainCode::Collectibles => (long_part, 0, long_part),
            QofGainCode::None | QofGainCode::Other => (long_part, 0, 0),
        };
        Inclusion {
            deferred,
            increase,
            included,
            short,
            long,
            section_1231,
            collectibles,
            basis_after: increase + included,
            value_assumed,
        }
    }
}

impl Inclusion {
    // The Form 8949 rows of the inclusion: code Y in Part I box C and Part II box F; a section
    // 1231 gain is the Y row and a code O row that backs it out of Schedule D (it is on Form 4797
    // Part I instead), netting to zero.
    pub(crate) fn rows(&self, entry: &FederalIncomeEntry) -> Vec<Form8949Row> {
        let mut rows = Vec::new();
        let mut push = |line: usize, box_letter: char, adjustment: i64, codes: &str| {
            rows.push(Form8949Row {
                entry: entry.id,
                row: rows.len() as u32 + 1,
                line,
                box_letter,
                label: crate::interview::income_display_arg(entry),
                proceeds: 0,
                basis: 0,
                adjustment,
                codes: codes.into(),
                summary: false,
            });
        };
        if self.short != 0 {
            push(3, 'C', self.short, "Y");
        }
        if self.long != 0 {
            push(7, 'F', self.long, "Y");
        }
        if self.section_1231 != 0 {
            push(7, 'F', self.section_1231, "Y");
            push(7, 'F', -self.section_1231, "O");
        }
        rows
    }
}

impl QofDeferral {
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }

    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        (i128::from(self.short_term) + i128::from(self.long_term) <= 0).then_some(
            crate::validation::ValidationIssue::new("QOF_DEFERRAL_NO_GAIN").at_field("short_term"),
        )
    }

    pub(crate) fn valid(&self) -> bool {
        !self.fund.is_empty()
            && crate::federal_rules::valid_date(self.invested_on)
            && (yc::QOF_AMENDED_FIRST_INVESTMENT_DATE..=yc::QOF_DEFERRAL_LAST_INVESTMENT_DATE)
                .contains(&self.invested_on)
            && (0..=MAX).contains(&self.short_term)
            && (0..=MAX).contains(&self.long_term)
            && self.rule().is_none()
    }

    // The Form 8949 rows of the election: code Z, the deferred gain as a negative column (g).
    pub(crate) fn rows(&self, entry: &FederalIncomeEntry) -> Vec<Form8949Row> {
        let mut rows = Vec::new();
        for (line, box_letter, amount) in [(3, 'C', self.short_term), (7, 'F', self.long_term)] {
            if amount != 0 {
                rows.push(Form8949Row {
                    entry: entry.id,
                    row: rows.len() as u32 + 1,
                    line,
                    box_letter,
                    label: crate::interview::income_display_arg(entry),
                    proceeds: 0,
                    basis: 0,
                    adjustment: -amount,
                    codes: "Z".into(),
                    summary: false,
                });
            }
        }
        rows
    }
}

// The investments' section 1231 gains as Form 4797 Part I sources (whole dollars): an
// investment, not a business, so the gain is net investment income. `crate::section1231` joins
// them to the earlier-year sources.
pub(crate) fn sources(draft: &FederalDraft) -> Vec<(i64, crate::farm_land::LandGain, bool)> {
    draft
        .income
        .iter()
        .filter_map(|entry| match &entry.income {
            FederalIncome::OpportunityFund { data } if data.valid() && !data.left_out() => {
                let gain = crate::kernel::ratio::dollars(data.inclusion().section_1231);
                (gain != 0).then_some((
                    entry.id,
                    crate::farm_land::LandGain {
                        gain,
                        percent: 0,
                        recapture: 0,
                        ordinary: 0,
                        section1231: gain,
                        unrecaptured: 0,
                    },
                    false,
                ))
            }
            _ => None,
        })
        .collect()
}
