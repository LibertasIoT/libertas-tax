use crate::common::Owner;
use alloc::{collections::BTreeSet, string::String};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Qualified tax-credit bond or tax-credit Build America bond
/// Original clean renewable energy bonds and pre-October 4, 2008 qualified zone academy bonds retain known income; their different credit limitation and unused-credit deduction are left out with a notice.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum FederalTaxCreditBondKind {
    /// Original clean renewable energy bond; only its known income is included here
    /// Its unused-credit deduction and pass-through limit remain a named benefit gap.
    OriginalCleanRenewableEnergy,
    /// New clean renewable energy bond issued after October 3, 2008
    NewCleanRenewableEnergy,
    /// Qualified energy conservation bond issued after October 3, 2008
    QualifiedEnergyConservation,
    /// Qualified zone academy bond; the credit is supported only for issues after October 3, 2008
    QualifiedZoneAcademy,
    /// Qualified school construction bond issued after February 17, 2009
    QualifiedSchoolConstruction,
    /// Build America bond with a holder tax credit, not a direct-pay issuer election
    BuildAmericaTaxCredit,
}
impl FederalTaxCreditBondKind {
    pub(crate) fn issue_date_allowed(self, date: u32) -> bool {
        match self {
            Self::OriginalCleanRenewableEnergy => (20060101..=20091231).contains(&date),
            Self::NewCleanRenewableEnergy | Self::QualifiedEnergyConservation => {
                (20081004..=20171231).contains(&date)
            }
            Self::QualifiedZoneAcademy => (19980101..=20171231).contains(&date),
            Self::QualifiedSchoolConstruction => (20090218..=20171231).contains(&date),
            Self::BuildAmericaTaxCredit => (20090218..=20101231).contains(&date),
        }
    }
}
/// Direct bond-holder facts, not required of a fund shareholder
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct TaxCreditDirect {
    /// You held the bond throughout 2026, including every allowance/payment date
    pub directly_held_full_year: bool,
    /// The issuer or its agent supplied this report directly for an unstripped bond
    pub direct_issuer_report: bool,
    /// The issuer did not elect direct pay instead of a holder credit
    pub no_direct_pay_election: bool,
    /// Current deemed interest is not already entered on an interest statement
    /// No needs a documented source allocation before required income is resolved.
    pub deemed_interest_not_elsewhere: bool,
    /// All separate coupon interest is entered on the Interest and dividends page
    pub separate_interest_entered: bool,
    /// The deemed interest is U.S.-source personal investment interest
    /// No needs its actual tax character; use the issuer's interest and holding records.
    /// Check the issuer's interest report and the records of how you held the bond.
    pub ordinary_investment_interest: bool,
}
/// Income character designated by the distributing entity
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum TaxCreditIncomeKind {
    /// Regulated investment company shareholder distribution
    RicDistribution,
    /// Interest designated by a K-1 pass-through owner
    /// Its credit interface remains unavailable until the K-1 owner supplies accepted credit facts.
    Interest,
}
/// An entity's designated credit-related income
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct TaxCreditEntity {
    /// Entity name from its report
    #[libertas_size(min = 1)]
    pub entity: String,
    /// Current distribution or interest designated by the entity
    /// From actual entity records, not an assumed second inclusion of box 1 credit. Unreported RIC income enters ordinary dividends. To retain any issuer-designated qualified amount, enter it with the actual DIV first and link that already-reported distribution.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub designated_income: i64,
    /// Designated income already included in the linked accepted source
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub income_already_reported: i64,
    /// Income character from the entity's statement
    pub income_kind: TaxCreditIncomeKind,
    /// Accepted source containing the already reported income
    /// [NullText]
    /// No already-reported income is being reused
    /// [SetText]
    /// The already reported designated income is in
    #[libertas_default($null)]
    pub income_source: Option<i64>,
}
impl TaxCreditEntity {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if self.income_already_reported > self.designated_income {
            return Some(
                ValidationIssue::new("CREDIT_INCOME_ALREADY_ABOVE_DESIGNATED")
                    .at_field("income_already_reported"),
            );
        }
        if self.income_already_reported != 0 && self.income_source.is_none() {
            return Some(
                ValidationIssue::new("CREDIT_INCOME_SOURCE_NEEDED").at_field("income_source"),
            );
        }
        None
    }
}
/// How this bond credit reached you
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum TaxCreditOrigin {
    /// You directly held the bond
    Direct(TaxCreditDirect),
    /// An entity reported your share on Form 1097-BTC
    Entity1097(TaxCreditEntity),
    /// An entity reported your share on another statement
    EntityStatement(TaxCreditEntity),
}
impl TaxCreditOrigin {
    pub(crate) fn entity(&self) -> Option<&TaxCreditEntity> {
        match self {
            Self::Direct(_) => None,
            Self::Entity1097(e) | Self::EntityStatement(e) => Some(e),
        }
    }
}
/// A 2026 bond credit reported directly or through an entity
/// For Form 1097-BTC, enter annual box 1 once, excluding quarterly and monthly duplicates. Direct-holder credit produces deemed interest even when the credit is limited; enter separate coupon interest on the Interest page. An entity report supplies the actual designated income and any linked amount already included on the return. Form 1097-BTC reports feed Form 8912 Part III; other entity statements feed Part IV.
/// Sources: https://www.irs.gov/instructions/i8912 and https://www.irs.gov/instructions/i1097btc.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalTaxCreditBond1097 {
    /// Accepted document identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Printed recipient and issuer names
    /// [NullText]
    /// No printed names entered
    /// [SetText]
    /// Printed names
    #[libertas_default($null)]
    pub printed: Option<crate::PrintedDocumentNames>,
    /// Taxpayer or joint-return spouse to whom the credit was reported
    pub owner: Owner,
    /// Form 1097-BTC as printed
    pub form: Form1097BtcPaper,
    /// Personal bond records not settled by Form 1097-BTC
    pub answers: TaxCreditBondAnswers,
}
/// Personal records for a reported bond credit
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct TaxCreditBondAnswers {
    /// Bond class from the underlying issue records
    /// [NullText]
    /// The bond class is not yet known
    /// [SetText]
    /// The documented bond class is
    #[libertas_default($null)]
    pub kind: Option<FederalTaxCreditBondKind>,
    /// Original bond issue date
    /// [NullText]
    /// The issue date is not yet known
    /// [SetText]
    /// The original issue date is
    #[libertas_date_only]
    #[libertas_default($null)]
    pub issued_on: Option<u32>,
    /// Unused 2025 Form 8912 whole-dollar credit attributable to this same bond; leave zero if none
    /// Copy the whole-dollar amount from the prior Form 8912 carryforward worksheet. Do not enter cents or re-round it.
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub prior_carry_dollars: i64,
    /// This bond's whole-dollar share of the allowed 2026 Form 8912 line 12 credit
    /// Enter when multiple bonds with positive rounded credit have a partly limited total (line 12 is above zero but below line 4). Enter a share for every bond, including zero; together they must equal line 12, and no share may exceed that bond's box 1 credit rounded to dollars plus its same-bond prior carry. The IRS specifies no bond ordering. Leave empty when all credit is allowed, none is allowed, or only one bond has available rounded credit; the App then determines every share uniquely. Any entered share retained in those automatic cases must match the uniquely determined amount.
    /// [NullText]
    /// Use the per-bond share the App can determine uniquely
    /// [SetText]
    /// This bond's documented whole-dollar allowed credit share is
    #[libertas_number(min = 0)]
    #[libertas_default($null)]
    pub allowed_credit_dollars: Option<i64>,

    /// Confirm this bond's documented share of the allowed credit
    /// Choose Yes when you enter a whole-dollar allowed share, from the retained Form 8912 line 12 allocation worksheet tying this bond's Form 1097-BTC identifier to its share. Choose No when the App determines the share automatically.
    /// [NullText]
    /// The allowed-share confirmation is unanswered
    /// [SetText]
    /// The documented allowed-share confirmation is
    #[libertas_default($null)]
    pub allowed_credit_confirmed: Option<bool>,
    /// The prior carry is documented on a prior Form 8912 worksheet for this same bond
    /// Matters only when a positive prior carry is entered: check the 2025 Form 8912 carryforward worksheet. No leaves the prior carry unclaimed.
    /// [NullText]
    /// The prior-carry confirmation is unanswered
    /// [SetText]
    /// The same-bond prior-carry confirmation is
    #[libertas_default($null)]
    pub prior_carry_confirmed: Option<bool>,
    /// Direct holding or actual entity report
    /// [NullText]
    /// How the credit reached you is not yet known
    /// [SetText]
    /// The credit reached you through
    #[libertas_default($null)]
    pub origin: Option<TaxCreditOrigin>,
}
/// Personal records for a reported bond credit
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaxCreditBondResolvedAnswers {
    /// Bond class from underlying issue records; Form 1097-BTC box 3 alone does not distinguish these classes
    pub kind: FederalTaxCreditBondKind,
    /// Original bond issue date; bonds issued after 2017 cannot generate this credit
    pub issued_on: u32,
    /// Unused 2025 Form 8912 whole-dollar credit attributable to this same bond; leave zero if none
    /// Copy the whole-dollar amount from the prior Form 8912 carryforward worksheet. Do not enter cents or re-round it.
    pub prior_carry_dollars: i64,
    /// This bond's whole-dollar share of the allowed 2026 Form 8912 line 12 credit
    /// Enter when multiple bonds with positive rounded credit have a partly limited total (line 12 is above zero but below line 4). Enter a share for every bond, including zero; together they must equal line 12, and no share may exceed that bond's box 1 credit rounded to dollars plus its same-bond prior carry. The IRS specifies no bond ordering. Leave empty when all credit is allowed, none is allowed, or only one bond has available rounded credit; the App then determines every share uniquely. Any entered share retained in those automatic cases must match the uniquely determined amount.
    pub allowed_credit_dollars: Option<i64>,
    /// Confirm this bond's documented share of the allowed credit
    /// Choose Yes when you enter a whole-dollar allowed share, from the retained Form 8912 line 12 allocation worksheet tying this bond's Form 1097-BTC identifier to its share. Choose No when the App determines the share automatically.
    pub allowed_credit_confirmed: bool,
    /// The prior carry is documented on a prior Form 8912 worksheet for this same bond
    /// Matters only when a positive prior carry is entered: check the 2025 Form 8912 carryforward worksheet. No leaves the prior carry unclaimed.
    pub prior_carry_confirmed: bool,
    /// Direct holding or actual entity report
    pub origin: TaxCreditOrigin,
}
impl FederalTaxCreditBond1097 {
    pub(crate) fn resolved_answers(&self) -> Option<TaxCreditBondResolvedAnswers> {
        let answers = &self.answers;
        Some(TaxCreditBondResolvedAnswers {
            kind: answers.kind?,
            issued_on: answers.issued_on?,
            prior_carry_dollars: answers.prior_carry_dollars,
            allowed_credit_dollars: answers.allowed_credit_dollars,
            allowed_credit_confirmed: if answers.allowed_credit_dollars.is_some() {
                answers.allowed_credit_confirmed?
            } else {
                answers.allowed_credit_confirmed.unwrap_or(false)
            },
            prior_carry_confirmed: if answers.prior_carry_dollars > 0 {
                answers.prior_carry_confirmed?
            } else {
                answers.prior_carry_confirmed.unwrap_or(false)
            },
            origin: answers.origin.clone()?,
        })
    }

    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        self.answers
            .origin
            .as_ref()?
            .entity()
            .and_then(|entity| entity.issue())
            .map(|issue| issue.in_field("origin").in_field("answers"))
    }
    fn valid(&self) -> bool {
        let answers = &self.answers;
        let records_valid =
            answers.origin.as_ref().and_then(TaxCreditOrigin::entity).is_none_or(|e| {
                !e.entity.trim().is_empty()
                    && (0..=100_000_000_000).contains(&e.designated_income)
                    && e.income_already_reported >= 0
                    && e.issue().is_none()
            }) && answers.issued_on.is_none_or(crate::dates::source_valid_date)
                && (0..=1_000_000_000).contains(&answers.prior_carry_dollars)
                && answers
                    .allowed_credit_dollars
                    .is_none_or(|amount| (0..=1_000_000_000).contains(&amount));
        records_valid
            && !self.form.issuer.name.trim().is_empty()
            && valid_tin(&self.form.issuer.federal_id)
            && self.form.unique_identifier.as_ref().is_none_or(|identifier| {
                // Form 1097-BTC (April 2025), recipient instructions Box 2b:
                // https://www.irs.gov/pub/irs-pdf/f1097btc.pdf limits the identifier to 39 characters.
                !identifier.is_empty()
                    && identifier.len() <= 39
                    && identifier.bytes().enumerate().all(|(index, character)| {
                        character.is_ascii_alphanumeric()
                            // Box 2a permits one hyphen after the CUSIP, before a following identifier.
                            || (index == 9 && character == b'-' && identifier.len() > 10)
                    })
            })
            && (0..=100_000_000_000).contains(&self.form.box1_credit.unwrap_or(0))
    }

    // Benefit scope is separate from required current income.
    pub(crate) fn supported(&self) -> bool {
        let Some(answers) = self.resolved_answers() else {
            return false;
        };
        let form = Form1097Btc::from(&self.form);
        form.box3_code_199
            && answers.kind.issue_date_allowed(answers.issued_on)
            && answers.kind != FederalTaxCreditBondKind::OriginalCleanRenewableEnergy
            && !(answers.kind == FederalTaxCreditBondKind::QualifiedZoneAcademy
                && answers.issued_on < 20081004)
            && match &answers.origin {
                TaxCreditOrigin::Direct(d) => {
                    d.directly_held_full_year
                        && d.direct_issuer_report
                        && d.no_direct_pay_election
                        && d.deemed_interest_not_elsewhere
                        && d.separate_interest_entered
                        && d.ordinary_investment_interest
                }
                TaxCreditOrigin::Entity1097(e) | TaxCreditOrigin::EntityStatement(e) => {
                    e.income_kind == TaxCreditIncomeKind::RicDistribution
                }
            }
    }
    #[cfg(test)]
    pub(crate) fn direct_scope_mut(&mut self) -> Option<&mut TaxCreditDirect> {
        match self.answers.origin.as_mut()? {
            TaxCreditOrigin::Direct(d) => Some(d),
            _ => None,
        }
    }
    pub(crate) fn income_unknown(&self) -> bool {
        self.answers.origin.as_ref().is_none_or(|origin| matches!(origin, TaxCreditOrigin::Direct(d) if !d.deemed_interest_not_elsewhere || !d.ordinary_investment_interest))
    }
    pub(crate) fn interest_income(&self) -> i64 {
        let Some(origin) = self.answers.origin.as_ref() else {
            return 0;
        };
        let form = Form1097Btc::from(&self.form);
        match origin {
            TaxCreditOrigin::Direct(d) => {
                if d.deemed_interest_not_elsewhere {
                    form.box1_credit
                } else {
                    0
                }
            }
            TaxCreditOrigin::Entity1097(e) | TaxCreditOrigin::EntityStatement(e) => {
                if e.income_kind == TaxCreditIncomeKind::Interest {
                    e.designated_income - e.income_already_reported
                } else {
                    0
                }
            }
        }
    }
    pub(crate) fn dividend_income(&self) -> i64 {
        self.answers
            .origin
            .as_ref()
            .and_then(TaxCreditOrigin::entity)
            .filter(|e| e.income_kind == TaxCreditIncomeKind::RicDistribution)
            .map_or(0, |e| e.designated_income - e.income_already_reported)
    }

    pub(crate) fn current_dollars(&self) -> i64 {
        let form = Form1097Btc::from(&self.form);
        if self.supported() { crate::kernel::ratio::dollars(form.box1_credit) } else { 0 }
    }

    pub(crate) fn carry_dollars(&self) -> i64 {
        self.resolved_answers()
            .filter(|_| self.supported())
            .map_or(0, |answers| answers.prior_carry_dollars)
    }
}
fn valid_tin(value: &str) -> bool {
    match value.as_bytes() {
        [a, b, c, d, e, f, g, h, i] => {
            [a, b, c, d, e, f, g, h, i].iter().all(|digit| digit.is_ascii_digit())
        }
        [a, b, b'-', c, d, e, f, g, h, i] => {
            [a, b, c, d, e, f, g, h, i].iter().all(|digit| digit.is_ascii_digit())
        }
        _ => false,
    }
}
fn tin_digits(value: &str) -> [u8; 9] {
    let mut digits = [0; 9];
    for (slot, digit) in digits.iter_mut().zip(value.bytes().filter(u8::is_ascii_digit)) {
        *slot = digit;
    }
    digits
}
pub(crate) fn valid(entries: &[FederalTaxCreditBond1097]) -> bool {
    let mut keys = BTreeSet::new();
    entries.len() <= 20
        && entries.iter().all(|entry| {
            entry.valid()
                && keys.insert((
                    tin_digits(&entry.form.issuer.federal_id),
                    entry.form.unique_identifier.as_deref(),
                ))
        })
}
pub(crate) fn current_interest_cents(entries: &[FederalTaxCreditBond1097]) -> i64 {
    entries.iter().map(FederalTaxCreditBond1097::interest_income).sum()
}
pub(crate) fn available_dollars(entries: &[FederalTaxCreditBond1097]) -> i64 {
    entries.iter().map(|entry| entry.current_dollars() + entry.carry_dollars()).sum()
}
/// Form 1097-BTC box 2a identification-number code
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum Form1097BtcIdentifierCode {
    /// C — CUSIP number
    #[libertas_form_code("C")]
    Cusip,
    /// A — Account number
    #[libertas_form_code("A")]
    AccountNumber,
    /// O — Other identifier
    #[libertas_form_code("O")]
    OtherIdentifier,
}
/// Form 1097-BTC box 3 bond-type code
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum Form1097BtcBondType {
    /// 101 — Clean renewable energy bond
    #[libertas_form_code("101")]
    CleanRenewableEnergy,
    /// 199 — Other bond
    #[libertas_form_code("199")]
    Other,
}
/// Form 1097-BTC — Bond Tax Credit
/// Printed boxes from the IRS recipient copy, Rev. April 2025.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct Form1097BtcPaper {
    /// FORM 1097-BTC ISSUER'S name, street address, city or town, state or province, country, ZIP or foreign postal code, and TIN
    #[libertas_llm_doc_field]
    pub issuer: crate::PrintedParty,
    /// Telephone no.
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub issuer_telephone: String,
    /// RECIPIENT'S name, TIN, street address (including apt. no.), city or town, state or province, country, and ZIP or foreign postal code
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,
    /// For calendar year
    /// [NullText]
    /// Calendar year is not printed
    /// [SetText]
    /// Calendar year is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_default($null)]
    #[libertas_format("0")]
    pub calendar_year: Option<u32>,
    /// CORRECTED (if checked)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub corrected: bool,
    /// Issuer of bond or its agent filing current year Form 1097-BTC for credit being reported
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub issuer_of_bond_or_agent: bool,
    /// An entity or a person that received or should have received a current year Form 1097-BTC and is distributing part or all of that credit to others
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub distributing_credit: bool,
    /// Box 1 Total
    /// [NullText]
    /// Box 1 Total is blank
    /// [SetText]
    /// Printed box 1 total
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub box1_credit: Option<i64>,
    /// Box 2a Code
    /// [NullText]
    /// Box 2a is blank
    /// [SetText]
    /// Identifier code is printed
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_default($null)]
    pub identifier_code: Option<Form1097BtcIdentifierCode>,
    /// Box 2b Unique identifier
    /// [NullText]
    /// Box 2b is blank
    /// [SetText]
    /// Printed unique identifier
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_default($null)]
    // The IRS limits box 2b to 39 characters, including an optional CUSIP separator.
    #[libertas_size(min = 1, max = 39)]
    #[libertas_pattern(r"[A-Za-z0-9]{1,39}|[A-Za-z0-9]{9}-[A-Za-z0-9]{1,29}")]
    pub unique_identifier: Option<String>,
    /// Box 3 Bond type
    /// [NullText]
    /// Box 3 is blank
    /// [SetText]
    /// Bond type is printed
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_default($null)]
    pub bond_type: Option<Form1097BtcBondType>,
    /// Box 4
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub box4: String,
    /// Box 5a January
    /// [NullText]
    /// Box 5a January is blank
    /// [SetText]
    /// Printed box 5a january
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub january: Option<i64>,
    /// Box 5b February
    /// [NullText]
    /// Box 5b February is blank
    /// [SetText]
    /// Printed box 5b february
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub february: Option<i64>,
    /// Box 5c March
    /// [NullText]
    /// Box 5c March is blank
    /// [SetText]
    /// Printed box 5c march
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub march: Option<i64>,
    /// Box 5d April
    /// [NullText]
    /// Box 5d April is blank
    /// [SetText]
    /// Printed box 5d april
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub april: Option<i64>,
    /// Box 5e May
    /// [NullText]
    /// Box 5e May is blank
    /// [SetText]
    /// Printed box 5e may
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub may: Option<i64>,
    /// Box 5f June
    /// [NullText]
    /// Box 5f June is blank
    /// [SetText]
    /// Printed box 5f june
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub june: Option<i64>,
    /// Box 5g July
    /// [NullText]
    /// Box 5g July is blank
    /// [SetText]
    /// Printed box 5g july
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub july: Option<i64>,
    /// Box 5h August
    /// [NullText]
    /// Box 5h August is blank
    /// [SetText]
    /// Printed box 5h august
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub august: Option<i64>,
    /// Box 5i September
    /// [NullText]
    /// Box 5i September is blank
    /// [SetText]
    /// Printed box 5i september
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub september: Option<i64>,
    /// Box 5j October
    /// [NullText]
    /// Box 5j October is blank
    /// [SetText]
    /// Printed box 5j october
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub october: Option<i64>,
    /// Box 5k November
    /// [NullText]
    /// Box 5k November is blank
    /// [SetText]
    /// Printed box 5k november
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub november: Option<i64>,
    /// Box 5l December
    /// [NullText]
    /// Box 5l December is blank
    /// [SetText]
    /// Printed box 5l december
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub december: Option<i64>,
    /// Box 6 Comments
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub comments: String,
}
impl From<&Form1097BtcPaper> for Form1097Btc {
    fn from(paper: &Form1097BtcPaper) -> Self {
        let Form1097BtcPaper {
            issuer,
            // Header metadata and monthly detail are retained on the paper;
            // existing figures consume only the annual total and bond type.
            issuer_telephone: _,
            recipient,
            calendar_year: _,
            corrected: _,
            issuer_of_bond_or_agent: _,
            distributing_credit: _,
            box1_credit,
            identifier_code: _,
            unique_identifier,
            bond_type,
            box4: _,
            january: _,
            february: _,
            march: _,
            april: _,
            may: _,
            june: _,
            july: _,
            august: _,
            september: _,
            october: _,
            november: _,
            december: _,
            comments: _,
        } = paper;
        Self {
            issuer: issuer.clone(),
            recipient: recipient.clone(),
            unique_identifier: unique_identifier.clone().unwrap_or_default(),
            box3_code_199: *bond_type == Some(Form1097BtcBondType::Other),
            box1_credit: box1_credit.unwrap_or(0),
        }
    }
}
/// Form 1097-BTC
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct Form1097Btc {
    /// ISSUER'S name, TIN and address
    #[libertas_llm_doc_field]
    pub issuer: crate::PrintedParty,
    /// RECIPIENT'S name, TIN and address
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,
    /// Form 1097-BTC box 2b identifier, assigned by this issuer and distinct for its bonds
    /// Another issuer may independently use the same identifier; pair it with issuer TIN when tracing same-bond carry.
    /// Keep any printed hyphen between the nine-character CUSIP and the following account or other identifier.
    #[libertas_llm_doc_field]
    // External Form 1097-BTC box 2b: at most 39 characters (https://www.irs.gov/pub/irs-pdf/f1097btc.pdf).
    #[libertas_size(min = 1, max = 39)]
    #[libertas_pattern(r"[A-Za-z0-9]{1,39}|[A-Za-z0-9]{9}-[A-Za-z0-9]{1,29}")]
    pub unique_identifier: String,
    /// Annual Form 1097-BTC box 3 is code 199, not original clean-renewable-energy code 101
    /// No leaves the credit and unused-credit deduction out with a notice; independently known required income remains included.
    #[libertas_llm_doc_field]
    pub box3_code_199: bool,
    /// Annual 2026 Form 1097-BTC box 1 credit, including any 70% limit already applied by its issuer
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub box1_credit: i64,
}
