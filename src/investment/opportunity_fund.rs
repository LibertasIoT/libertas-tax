// Coverage: Form 8997 and qualified opportunity funds. QofInvestment (an
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
use crate::kernel::phase;
use crate::year_constants as yc;
use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

const MAX: i64 = 100_000_000_000;

/// Qualified opportunity fund investment (Form 8997)
/// One for each investment in a qualified opportunity fund made before 2026 that still defers a gain: one row of Part IV of your 2025 Form 8997. The deferral ends on December 31, 2026, so the deferred gain is taxed this year, or when you sold the investment if that was earlier.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read the Name(s) shown on return and the Identifying number at the top of the form into source_recipient, with empty text for its address. Copy names and identification numbers exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Form 8997 has no issuer; never read the form header or the Department of the Treasury as a party. Check the actual printed Form 8997 tax year: accept only 2025 Part IV (QOF investments with deferrals held at year end) or 2026 Part I. The 2025 paper reports the deferred investment carried into the 2026 return and is not a year mismatch. Read one row from that year's indicated part exactly; reject a missing, illegible, conflicting or other printed tax year. Never rewrite the printed year or acquisition date as 2026. If the document shows more than one row, report an error that says how many rows it shows and that each row needs its own opportunity fund entry; never add, combine, or choose among rows. Report an illegible value, or two values that conflict, as an error. Return amounts as decimal numbers without commas or currency signs. Never infer a value that is not printed on the form. Return column (a), the QOF's EIN, as fund_ein exactly as printed, preserving punctuation and leading zeros, or empty text when it is blank. Return column (c), the complete description of the QOF investment, as fund without truncation; report an error when it is blank. Return column (b), the date the QOF investment was acquired, as invested_on; report an error when it is blank. Read column (e) short-term deferred gain into short_term and column (f) long-term deferred gain into long_term, returning an empty string for a blank column so it remains null; never substitute zero or another column's amount. Preserve an explicitly printed zero as zero. Report an error when both are blank. For column (d), the special gain code, return None when it is blank, Section1231 for code B, Collectibles for code D, Section1256 for code A, Straddle for code C, ExemptByTreaty for code E and NoninclusionTransfer for code F. Do not read any other part of the form or the totals lines. Do not report whether the investment was sold, its value, or whether a special treatment applies; the filer answers those. Read the printed calendar or tax year at the top of the form as an integer number such as 2026, or zero when it is clearly blank; never substitute the form revision year.",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.source_recipient.federal_id,@.fund_ein,@.invested_on,@.gain_code,@.short_term,@.long_term",
    form_date_order = "MonthDayYear"
)]
pub struct QofInvestmentPaper {
    /// For calendar year
    #[libertas_llm_doc_field]
    #[libertas_default(0)]
    pub calendar_year: u32,
    /// Source document recipient as printed
    #[libertas_llm_doc_field]
    pub source_recipient: crate::PrintedParty,
    /// EIN of the QOF (Form 8997 column (a))
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub fund_ein: String,
    /// Date you invested (column (b))
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    // Tax structure: this entry reports QOF deferrals carried into 2026 from investments made through December 31, 2025.
    #[libertas_number(min = $QOF_FIRST_INVESTMENT_DATE, max = $QOF_LAST_EARLIER_INVESTMENT_DATE)]
    pub invested_on: u32,
    /// Fund and investment (column (c))
    /// For example "100 shares of Example Opportunity Fund".
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_size(min = 1)]
    pub fund: String,
    /// Special gain code (column (d))
    #[libertas_llm_doc_field]
    #[libertas_default(None)]
    pub gain_code: QofGainCode,
    /// Short-term gain still deferred (column (e))
    /// [NullText]
    /// This printed column is blank
    /// [SetText]
    /// The printed deferred gain is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub short_term: Option<i64>,
    /// Long-term gain still deferred (column (f))
    /// [NullText]
    /// This printed column is blank
    /// [SetText]
    /// The printed deferred gain is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub long_term: Option<i64>,
}

/// Opportunity fund investment and its current-year answers
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct QofInvestment {
    /// Source document recipient as printed
    pub source_recipient: crate::PrintedParty,
    /// EIN of the QOF (Form 8997 column (a))
    #[libertas_default("")]
    pub fund_ein: String,
    /// Date you invested (column (b))
    #[libertas_date_only]
    // Tax structure: this entry reports QOF deferrals carried into 2026 from investments made through December 31, 2025.
    #[libertas_number(min = $QOF_FIRST_INVESTMENT_DATE, max = $QOF_LAST_EARLIER_INVESTMENT_DATE)]
    pub invested_on: u32,
    /// Fund and investment (column (c))
    /// For example "100 shares of Example Opportunity Fund".
    #[libertas_size(min = 1)]
    pub fund: String,
    /// Special gain code (column (d))
    #[libertas_default(None)]
    pub gain_code: QofGainCode,
    /// Short-term gain still deferred (column (e))
    /// [NullText]
    /// This printed column is blank
    /// [SetText]
    /// The printed deferred gain is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub short_term: Option<i64>,
    /// Long-term gain still deferred (column (f))
    /// [NullText]
    /// This printed column is blank
    /// [SetText]
    /// The printed deferred gain is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub long_term: Option<i64>,
    /// What happened to the investment in 2026
    /// [NullText]
    /// I do not know what happened yet
    pub event: Option<QofEvent>,
    /// A partial sale, a distribution from the fund, a gift of the investment, a fund interest held through a partnership or S corporation, or another special treatment
    /// Yes leaves this investment out of the estimate with a notice; that treatment is not supported.
    #[libertas_default(false)]
    pub special: bool,
}

// Source: Form 8997 instructions, special gain code table (SGC A-F):
// https://www.irs.gov/pub/irs-pdf/f8997.pdf
/// Special gain code
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum QofGainCode {
    /// No code
    None,
    /// A: Section 1256 gain
    /// Leaves this investment out of the estimate with a notice; that treatment is not supported.
    #[libertas_form_code("A")]
    Section1256,
    /// B: Form 4797 (section 1231) gain
    /// Taxed as a section 1231 gain on Form 4797 when included.
    #[libertas_form_code("B")]
    Section1231,
    /// C: Straddle gain
    /// Leaves this investment out of the estimate with a notice; that treatment is not supported.
    #[libertas_form_code("C")]
    Straddle,
    /// D: Collectibles gain
    /// Taxed at up to 28% when included.
    #[libertas_form_code("D")]
    Collectibles,
    /// E: Exempt by treaty (but for the election to defer rather than claim the exemption)
    /// Leaves this investment out of the estimate with a notice; that treatment is not supported.
    #[libertas_form_code("E")]
    ExemptByTreaty,
    /// F: Deferred gain relating to a noninclusion transfer
    /// Leaves this investment out of the estimate with a notice; that treatment is not supported.
    #[libertas_form_code("F")]
    NoninclusionTransfer,
}

impl QofGainCode {
    // The codes the App cannot compute (A, C, E, F).
    pub(crate) fn unsupported(self) -> bool {
        matches!(
            self,
            Self::Section1256 | Self::Straddle | Self::ExemptByTreaty | Self::NoninclusionTransfer
        )
    }
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
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
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
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
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
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub short_term: i64,
    /// Long-term gain deferred
    /// Up to your 2026 long-term capital gains on this page.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
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

impl QofInvestmentPaper {
    pub(crate) fn printed_issue(&self) -> Option<crate::validation::ValidationIssue> {
        if i128::from(self.short_term.unwrap_or(0)) + i128::from(self.long_term.unwrap_or(0)) <= 0 {
            return Some(
                crate::validation::ValidationIssue::new("QOF_NO_DEFERRED_GAIN")
                    .at_field("short_term"),
            );
        }
        if self.gain_code == QofGainCode::Section1231 && self.short_term.unwrap_or(0) != 0 {
            return Some(
                crate::validation::ValidationIssue::new("QOF_SECTION_1231_LONG_TERM")
                    .at_field("short_term"),
            );
        }
        if self.gain_code == QofGainCode::Collectibles && self.short_term.unwrap_or(0) != 0 {
            return Some(
                crate::validation::ValidationIssue::new("QOF_COLLECTIBLES_LONG_TERM")
                    .at_field("short_term"),
            );
        }
        None
    }

    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        self.printed_issue()
    }
}

impl From<&QofInvestment> for QofInvestmentPaper {
    fn from(value: &QofInvestment) -> Self {
        Self {
            calendar_year: Default::default(),
            source_recipient: value.source_recipient.clone(),
            fund_ein: value.fund_ein.clone(),
            fund: value.fund.clone(),
            invested_on: value.invested_on,
            short_term: value.short_term,
            long_term: value.long_term,
            gain_code: value.gain_code,
        }
    }
}

impl QofInvestment {
    pub fn printed(form: QofInvestmentPaper) -> Self {
        Self {
            source_recipient: form.source_recipient,
            fund_ein: form.fund_ein,
            fund: form.fund,
            invested_on: form.invested_on,
            short_term: form.short_term,
            long_term: form.long_term,
            gain_code: form.gain_code,
            event: None,
            special: false,
        }
    }

    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }

    pub(crate) fn printed_issue(&self) -> Option<crate::validation::ValidationIssue> {
        QofInvestmentPaper::from(self).printed_issue()
    }

    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        if let Some(issue) = self.printed_issue() {
            return Some(issue);
        }
        if let Some(QofEvent::Sold { sold_on, .. }) = self.event
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
            && self.short_term.is_none_or(|amount| (0..=MAX).contains(&amount))
            && self.long_term.is_none_or(|amount| (0..=MAX).contains(&amount))
            && self.event.as_ref().is_none_or(QofEvent::valid)
            && self.rule().is_none()
    }

    // A special gain code the App cannot compute (A, C, E, F) or any other special
    // treatment leaves the investment out with a notice.
    pub(crate) fn left_out(&self) -> bool {
        self.special || self.gain_code.unsupported()
    }

    // The inclusion on the earlier of the sale and December 31, 2026 (IRC 1400Z-2(b)(1)(B)).
    pub(crate) fn inclusion(&self) -> Option<Inclusion> {
        let event = self.event.as_ref()?;
        let short_term = self.short_term.unwrap_or(0);
        let deferred = short_term + self.long_term.unwrap_or(0);
        let (five, seven) = match event {
            QofEvent::Held { .. } => (
                self.invested_on <= yc::QOF_FIVE_YEAR_LAST_INVESTMENT_DATE,
                self.invested_on <= yc::QOF_SEVEN_YEAR_LAST_INVESTMENT_DATE,
            ),
            QofEvent::Sold { sold_on, .. } => {
                (self.invested_on + 50000 <= *sold_on, self.invested_on + 70000 <= *sold_on)
            }
        };
        let percent = if five { yc::QOF_FIVE_YEAR_PERCENT } else { 0 }
            + if seven { yc::QOF_SEVEN_YEAR_PERCENT } else { 0 };
        let increase = crate::kernel::ratio::fraction(deferred, percent, 100);
        let (value, value_assumed) = match event {
            QofEvent::Held { fair_value: Some(value) } => (*value, false),
            QofEvent::Held { fair_value: None } => (deferred, true),
            QofEvent::Sold { amount, .. } => (*amount, false),
        };
        let included = phase::floor(phase::cap(deferred, [value]) - increase, 0);
        let short = if deferred == 0 {
            0
        } else {
            crate::kernel::ratio::fraction(included, short_term, deferred)
        };
        let long_part = included - short;
        let (long, section_1231, collectibles) = match self.gain_code {
            QofGainCode::Section1231 => (0, long_part, 0),
            QofGainCode::Collectibles => (long_part, 0, long_part),
            _ => (long_part, 0, 0),
        };
        Some(Inclusion {
            deferred,
            increase,
            included,
            short,
            long,
            section_1231,
            collectibles,
            basis_after: increase + included,
            value_assumed,
        })
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
}
