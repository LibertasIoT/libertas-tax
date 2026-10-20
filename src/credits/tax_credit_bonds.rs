use crate::Owner;
use alloc::{collections::BTreeSet, string::String, vec::Vec};
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
    #[libertas_number(min = 0)]
    pub designated_income: i64,
    /// Designated income already included in the linked accepted source
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub income_already_reported: i64,
    /// Income character from the entity's statement
    pub income_kind: TaxCreditIncomeKind,
    /// Accepted source containing the already reported income
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
    /// Taxpayer or joint-return spouse to whom the credit was reported
    pub owner: Owner,
    /// Name of the bond issuer or its reporting agent on Form 1097-BTC
    #[libertas_size(min = 1)]
    pub issuer_name: String,
    /// Form 1097-BTC issuer TIN, nine digits with an optional EIN hyphen
    // Legal identifier: nine-digit SSN/EIN, allowing its printed separators.
    #[libertas_size(min = 9, max = 10)]
    #[libertas_pattern(r"[0-9]{2}-?[0-9]{7}")]
    pub issuer_tin: String,
    /// Form 1097-BTC box 2b identifier, assigned by this issuer and distinct for its bonds
    /// Another issuer may independently use the same identifier; pair it with issuer TIN when tracing same-bond carry.
    /// Keep any printed hyphen between the nine-character CUSIP and the following account or other identifier.
    // External Form 1097-BTC box 2b: at most 39 characters (https://www.irs.gov/pub/irs-pdf/f1097btc.pdf).
    #[libertas_size(min = 1, max = 39)]
    #[libertas_pattern(r"[A-Za-z0-9]{1,39}|[A-Za-z0-9]{9}-[A-Za-z0-9]{1,29}")]
    pub unique_identifier: String,
    /// Bond class from underlying issue records; Form 1097-BTC box 3 alone does not distinguish these classes
    pub kind: FederalTaxCreditBondKind,
    /// Annual Form 1097-BTC box 3 is code 199, not original clean-renewable-energy code 101
    /// No leaves the credit and unused-credit deduction out with a notice; independently known required income remains included.
    pub box3_code_199: bool,
    /// Original bond issue date; bonds issued after 2017 cannot generate this credit
    #[libertas_date_only]
    pub issued_on: u32,
    /// Annual 2026 Form 1097-BTC box 1 credit, including any 70% limit already applied by its issuer
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub box1_credit: i64,
    /// Unused 2025 Form 8912 whole-dollar credit attributable to this same bond; leave zero if none
    /// Copy the whole-dollar amount from the prior Form 8912 carryforward worksheet. Do not enter cents or re-round it.
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub prior_carry_dollars: i64,
    /// This bond's whole-dollar share of the allowed 2026 Form 8912 line 12 credit
    /// Enter when multiple bonds with positive rounded credit have a partly limited total (line 12 is above zero but below line 4). Enter a share for every bond, including zero; together they must equal line 12, and no share may exceed that bond's box 1 credit rounded to dollars plus its same-bond prior carry. The IRS specifies no bond ordering. Leave empty when all credit is allowed, none is allowed, or only one bond has available rounded credit; the App then determines every share uniquely. Any entered share retained in those automatic cases must match the uniquely determined amount.
    #[libertas_number(min = 0)]
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
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        self.origin.entity().and_then(|entity| entity.issue()).map(|issue| issue.in_field("origin"))
    }
    fn valid(&self) -> bool {
        self.origin.entity().is_none_or(|e| {
            !e.entity.trim().is_empty()
                && (0..=100_000_000_000).contains(&e.designated_income)
                && e.income_already_reported >= 0
                && e.issue().is_none()
        }) && !self.issuer_name.trim().is_empty()
            && valid_tin(&self.issuer_tin)
            && !self.unique_identifier.is_empty()
            // Form 1097-BTC (April 2025), recipient instructions Box 2b:
            // https://www.irs.gov/pub/irs-pdf/f1097btc.pdf limits the identifier to 39 characters.
            && self.unique_identifier.len() <= 39
            && self
                .unique_identifier
                .bytes()
                .enumerate()
                .all(|(index, character)| {
                    character.is_ascii_alphanumeric()
                        // Box 2a permits one hyphen after the CUSIP, before a following identifier.
                        || (index == 9 && character == b'-' && self.unique_identifier.len() > 10)
                })
            && crate::federal_rules::valid_date(self.issued_on)
            && (1..=100_000_000_000).contains(&self.box1_credit)
            && (0..=1_000_000_000).contains(&self.prior_carry_dollars)
            && self
                .allowed_credit_dollars
                .is_none_or(|amount| (0..=1_000_000_000).contains(&amount))
    }

    // Benefit scope is separate from required current income.
    pub(crate) fn supported(&self) -> bool {
        self.box3_code_199
            && self.kind.issue_date_allowed(self.issued_on)
            && self.kind != FederalTaxCreditBondKind::OriginalCleanRenewableEnergy
            && !(self.kind == FederalTaxCreditBondKind::QualifiedZoneAcademy
                && self.issued_on < 20081004)
            && match &self.origin {
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
        match &mut self.origin {
            TaxCreditOrigin::Direct(d) => Some(d),
            _ => None,
        }
    }
    pub(crate) fn income_unknown(&self) -> bool {
        matches!(&self.origin, TaxCreditOrigin::Direct(d) if !d.deemed_interest_not_elsewhere || !d.ordinary_investment_interest)
    }
    pub(crate) fn interest_income(&self) -> i64 {
        match &self.origin {
            TaxCreditOrigin::Direct(d) => {
                if d.deemed_interest_not_elsewhere {
                    self.box1_credit
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
        self.origin
            .entity()
            .filter(|e| e.income_kind == TaxCreditIncomeKind::RicDistribution)
            .map_or(0, |e| e.designated_income - e.income_already_reported)
    }

    pub(crate) fn current_dollars(&self) -> i64 {
        if self.supported() { crate::kernel::ratio::dollars(self.box1_credit) } else { 0 }
    }

    pub(crate) fn carry_dollars(&self) -> i64 {
        if self.supported() { self.prior_carry_dollars } else { 0 }
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
                && keys.insert((tin_digits(&entry.issuer_tin), entry.unique_identifier.as_str()))
        })
}

pub(crate) fn current_interest_cents(entries: &[FederalTaxCreditBond1097]) -> i64 {
    entries.iter().map(FederalTaxCreditBond1097::interest_income).sum()
}

pub(crate) fn available_dollars(entries: &[FederalTaxCreditBond1097]) -> i64 {
    entries.iter().map(|entry| entry.current_dollars() + entry.carry_dollars()).sum()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BondAllocationError {
    Line12OutsideAvailable,
    MissingShare,
    UnconfirmedShare,
    ShareOutsideBondAvailable,
    SharesDoNotSum,
    UnusedShareNeedsBothFields,
    UniqueShareMismatch,
}

impl BondAllocationError {
    pub(crate) fn message(self) -> (&'static str, &'static str) {
        match self {
            Self::Line12OutsideAvailable => (
                "ISSUE_BOND_ALLOC_LINE12_OUTSIDE_AVAILABLE",
                crate::fixture_english!(
                    "Form 8912 line 12 must be between zero and the total available bond credit."
                ),
            ),
            Self::MissingShare => (
                "ISSUE_BOND_ALLOC_MISSING_SHARE",
                crate::fixture_english!(
                    "A partly limited Form 8912 with multiple bonds needs an allowed whole-dollar share for every bond, including zero."
                ),
            ),
            Self::UnconfirmedShare => (
                "ISSUE_BOND_ALLOC_UNCONFIRMED_SHARE",
                crate::fixture_english!(
                    "Confirm each entered bond share against the retained Form 8912 allocation worksheet and its Form 1097-BTC identifier."
                ),
            ),
            Self::ShareOutsideBondAvailable => (
                "ISSUE_BOND_ALLOC_SHARE_OUTSIDE_AVAILABLE",
                crate::fixture_english!(
                    "Each bond's allowed share must be nonnegative and no greater than its current rounded credit plus its own prior carry."
                ),
            ),
            Self::SharesDoNotSum => (
                "ISSUE_BOND_ALLOC_SHARES_DO_NOT_SUM",
                crate::fixture_english!(
                    "The allowed shares for all bonds must sum exactly to Form 8912 line 12; the IRS does not impose a bond order."
                ),
            ),
            Self::UnusedShareNeedsBothFields => (
                "ISSUE_BOND_ALLOC_UNUSED_SHARE_FIELDS",
                crate::fixture_english!(
                    "Confirm each entered bond share, or leave both the share and its confirmation unused when the App allocates automatically."
                ),
            ),
            Self::UniqueShareMismatch => (
                "ISSUE_BOND_ALLOC_UNIQUE_SHARE_MISMATCH",
                crate::fixture_english!(
                    "When Form 8912 has only one possible per-bond allocation, any entered share must equal the automatically determined amount."
                ),
            ),
        }
    }
}

/// Attribute the aggregate Form 8912 line 12 to the same bonds for next year's carry.
/// The IRS states the per-bond carry rule but prescribes no allocation order.
pub(crate) fn allocated_used_dollars(
    entries: &[FederalTaxCreditBond1097],
    line12: i64,
) -> Result<Vec<i64>, BondAllocationError> {
    let available = available_dollars(entries);
    if !(0..=available).contains(&line12) {
        return Err(BondAllocationError::Line12OutsideAvailable);
    }
    let positive_bonds =
        entries.iter().filter(|bond| bond.current_dollars() + bond.carry_dollars() > 0).count();
    let ambiguous = positive_bonds > 1 && line12 > 0 && line12 < available;
    if ambiguous {
        let mut used = Vec::with_capacity(entries.len());
        for bond in entries {
            let Some(amount) = bond.allowed_credit_dollars else {
                return Err(BondAllocationError::MissingShare);
            };
            if !bond.allowed_credit_confirmed {
                return Err(BondAllocationError::UnconfirmedShare);
            }
            if !(0..=bond.current_dollars() + bond.carry_dollars()).contains(&amount) {
                return Err(BondAllocationError::ShareOutsideBondAvailable);
            }
            used.push(amount);
        }
        if used.iter().sum::<i64>() != line12 {
            return Err(BondAllocationError::SharesDoNotSum);
        }
        return Ok(used);
    }
    let mut used = Vec::with_capacity(entries.len());
    for bond in entries {
        if !matches!(
            (bond.allowed_credit_dollars, bond.allowed_credit_confirmed),
            (None, false) | (Some(_), true)
        ) {
            return Err(BondAllocationError::UnusedShareNeedsBothFields);
        }
        let bond_available = bond.current_dollars() + bond.carry_dollars();
        let unique_amount = if line12 == 0 || bond_available == 0 {
            0
        } else if line12 == available {
            bond_available
        } else {
            // A partial limit with one positive bond identifies its share.
            line12
        };
        if bond.allowed_credit_dollars.is_some_and(|entered| entered != unique_amount) {
            return Err(BondAllocationError::UniqueShareMismatch);
        }
        used.push(unique_amount);
    }
    Ok(used)
}

/// A 2026 bond credit reported directly or through an entity
/// For Form 1097-BTC, enter annual box 1 once, excluding quarterly and monthly duplicates. Direct-holder credit produces deemed interest even when the credit is limited; enter separate coupon interest on the Interest page. An entity report supplies the actual designated income and any linked amount already included on the return. Form 1097-BTC reports feed Form 8912 Part III; other entity statements feed Part IV.
/// Sources: https://www.irs.gov/instructions/i8912 and https://www.irs.gov/instructions/i1097btc.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read this one annual Form 1097-BTC (Bond Tax Credit) exactly. Check its calendar year against the supplied year; report a mismatch. A quarterly statement or a corrected-quarter statement is not the annual form; report an error. Return the issuer's or agent's name and its TIN as printed, the box 2b unique identifier, and the box 1 total credit. Return Yes for the box 3 code question when box 3 shows code 199 and No when it shows 101 or any other code. A blank, illegible or conflicting value is an error, never zero or a guess. Do not read boxes 5 through 16 as separate credits. Do not infer the owner, the bond class, its issue date, carryovers, allowed shares, holding period, election or any confirmation or fact absent from the form. The file may hold several copies of this form, such as Copy B and Copy C; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*"
)]
pub struct TaxCreditBondInput {
    /// Taxpayer or joint-return spouse to whom the credit was reported
    #[libertas_constrained_by("$.allowed_owners")]
    pub owner: Owner,
    /// Name of the bond issuer or its reporting agent on Form 1097-BTC
    #[libertas_llm_doc_field]
    #[libertas_size(min = 1)]
    pub issuer_name: String,
    /// Form 1097-BTC issuer TIN, nine digits with an optional EIN hyphen
    #[libertas_llm_doc_field]
    // Legal identifier: nine-digit SSN/EIN, allowing its printed separators.
    #[libertas_size(min = 9, max = 10)]
    #[libertas_pattern(r"[0-9]{2}-?[0-9]{7}")]
    pub issuer_tin: String,
    /// Form 1097-BTC box 2b identifier, assigned by this issuer and distinct for its bonds
    /// Another issuer may independently use the same identifier; pair it with issuer TIN when tracing same-bond carry.
    /// Keep any printed hyphen between the nine-character CUSIP and the following account or other identifier.
    #[libertas_llm_doc_field]
    // External Form 1097-BTC box 2b: at most 39 characters (https://www.irs.gov/pub/irs-pdf/f1097btc.pdf).
    #[libertas_size(min = 1, max = 39)]
    #[libertas_pattern(r"[A-Za-z0-9]{1,39}|[A-Za-z0-9]{9}-[A-Za-z0-9]{1,29}")]
    pub unique_identifier: String,
    /// Bond class from underlying issue records; Form 1097-BTC box 3 alone does not distinguish these classes
    pub kind: FederalTaxCreditBondKind,
    /// Annual Form 1097-BTC box 3 is code 199, not original clean-renewable-energy code 101
    /// No leaves the credit and unused-credit deduction out with a notice; independently known required income remains included.
    #[libertas_llm_doc_field]
    pub box3_code_199: bool,
    /// Original bond issue date; bonds issued after 2017 cannot generate this credit
    #[libertas_date_only]
    pub issued_on: u32,
    /// Annual 2026 Form 1097-BTC box 1 credit, including any 70% limit already applied by its issuer
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub box1_credit: i64,
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
    pub allowed_credit_confirmed: bool,
    /// The prior carry is documented on a prior Form 8912 worksheet for this same bond
    /// Matters only when a positive prior carry is entered: check the 2025 Form 8912 carryforward worksheet. No leaves the prior carry unclaimed.
    pub prior_carry_confirmed: bool,
    /// Direct holding or actual entity report
    pub origin: TaxCreditOriginInput,
}

/// An entity's designated credit-related income
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct TaxCreditEntityInput {
    /// Entity name from its report
    #[libertas_size(min = 1)]
    pub entity: String,
    /// Current distribution or interest designated by the entity
    /// From actual entity records, not an assumed second inclusion of box 1 credit. Unreported RIC income enters ordinary dividends. To retain any issuer-designated qualified amount, enter it with the actual DIV first and link that already-reported distribution.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub designated_income: i64,
    /// Designated income already included in the linked accepted source
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub income_already_reported: i64,
    /// Income character from the entity's statement
    pub income_kind: TaxCreditIncomeKind,
    /// Accepted source containing the already reported income
    /// [NullText]
    /// No already-reported income is being reused
    /// [SetText]
    /// The already reported designated income is in
    #[libertas_enum_source("^<MoreCredits>.credit_income_sources")]
    #[libertas_default($null)]
    pub income_source: Option<i64>,
}

/// How the credit reached you
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum TaxCreditOriginInput {
    /// You directly held the bond
    Direct(TaxCreditDirect),
    /// Entity Form 1097-BTC
    Entity1097(TaxCreditEntityInput),
    /// Other entity statement
    EntityStatement(TaxCreditEntityInput),
}

impl From<TaxCreditEntityInput> for TaxCreditEntity {
    fn from(v: TaxCreditEntityInput) -> Self {
        Self {
            entity: v.entity,
            designated_income: v.designated_income,
            income_already_reported: v.income_already_reported,
            income_kind: v.income_kind,
            income_source: v.income_source,
        }
    }
}

impl From<TaxCreditEntity> for TaxCreditEntityInput {
    fn from(v: TaxCreditEntity) -> Self {
        Self {
            entity: v.entity,
            designated_income: v.designated_income,
            income_already_reported: v.income_already_reported,
            income_kind: v.income_kind,
            income_source: v.income_source,
        }
    }
}

impl From<TaxCreditOriginInput> for TaxCreditOrigin {
    fn from(v: TaxCreditOriginInput) -> Self {
        match v {
            TaxCreditOriginInput::Direct(v) => Self::Direct(v),
            TaxCreditOriginInput::Entity1097(v) => Self::Entity1097(v.into()),
            TaxCreditOriginInput::EntityStatement(v) => Self::EntityStatement(v.into()),
        }
    }
}

impl From<TaxCreditOrigin> for TaxCreditOriginInput {
    fn from(v: TaxCreditOrigin) -> Self {
        match v {
            TaxCreditOrigin::Direct(v) => Self::Direct(v),
            TaxCreditOrigin::Entity1097(v) => Self::Entity1097(v.into()),
            TaxCreditOrigin::EntityStatement(v) => Self::EntityStatement(v.into()),
        }
    }
}

impl From<TaxCreditBondInput> for FederalTaxCreditBond1097 {
    fn from(v: TaxCreditBondInput) -> Self {
        Self {
            owner: v.owner,
            issuer_name: v.issuer_name,
            issuer_tin: v.issuer_tin,
            unique_identifier: v.unique_identifier,
            kind: v.kind,
            box3_code_199: v.box3_code_199,
            issued_on: v.issued_on,
            box1_credit: v.box1_credit,
            prior_carry_dollars: v.prior_carry_dollars,
            allowed_credit_dollars: v.allowed_credit_dollars,
            allowed_credit_confirmed: v.allowed_credit_confirmed,
            prior_carry_confirmed: v.prior_carry_confirmed,
            origin: v.origin.into(),
        }
    }
}

impl From<FederalTaxCreditBond1097> for TaxCreditBondInput {
    fn from(v: FederalTaxCreditBond1097) -> Self {
        Self {
            owner: v.owner,
            issuer_name: v.issuer_name,
            issuer_tin: v.issuer_tin,
            unique_identifier: v.unique_identifier,
            kind: v.kind,
            box3_code_199: v.box3_code_199,
            issued_on: v.issued_on,
            box1_credit: v.box1_credit,
            prior_carry_dollars: v.prior_carry_dollars,
            allowed_credit_dollars: v.allowed_credit_dollars,
            allowed_credit_confirmed: v.allowed_credit_confirmed,
            prior_carry_confirmed: v.prior_carry_confirmed,
            origin: v.origin.into(),
        }
    }
}

/// Accepted on-return distribution available to an entity credit statement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CreditIncomeSource {
    /// Accepted income identity
    #[libertas_key]
    #[libertas_unique]
    #[libertas_hidden]
    pub id: i64,
    /// Statement
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
    /// Statement owner
    #[libertas_hidden]
    #[libertas_read_only]
    pub owner: Owner,
    /// Retained ordinary distribution on this return
    #[libertas_money("USD", 2)]
    #[libertas_hidden]
    #[libertas_read_only]
    pub amount: i64,
}

// Source ownership lives in investment_statements. This projection consumes
// that owner's supported working copy, including nominee/death adjustments,
// and the existing community-property owner once. It never reimplements them.
pub(crate) fn income_sources(d: &crate::FederalDraft) -> Vec<CreditIncomeSource> {
    let mut notes = Vec::new();
    let normalized = crate::unknowns::supported(d, &mut notes);
    let working = normalized.as_ref().unwrap_or(d);
    working
        .income
        .iter()
        .filter_map(|entry| {
            if !matches!(entry.income, crate::FederalIncome::Dividend { .. }) {
                return None;
            }
            let components = crate::investment_statements::components(entry)?;
            let share = working.setup.as_ref().and_then(|setup| {
                crate::community::share(setup, entry.owner, entry.community, &entry.income)
            });
            // Preserve the fund's actual shareholder designation on its DIV.
            // Underlying bond interest does not establish a nonqualified-only
            // cap on an already reported RIC distribution (IRC853A/854).
            let amount = share.map_or(components.ordinary_dividends, |share| {
                share.filer(components.ordinary_dividends)
            });
            Some(CreditIncomeSource {
                id: entry.id,
                owner: entry.owner,
                label: crate::Message::new(
                    "BOND_PREMIUM_INSTRUMENT_LABEL",
                    alloc::vec![
                        crate::MessageArg::Text(entry.label.clone()),
                        crate::MessageArg::Text(String::from("Form 8912 distribution")),
                    ],
                )
                .encode(),
                amount,
            })
        })
        .collect()
}

pub(crate) fn source_rule(d: &crate::FederalDraft) -> Option<&'static str> {
    source_issue(d).map(|issue| issue.key)
}

// Relative to the actual TaxCreditBonds payload. The owner preserves its
// per-bond pass before the source-wide aggregate pass.
pub(crate) fn source_issue(d: &crate::FederalDraft) -> Option<crate::validation::ValidationIssue> {
    use crate::validation::ValidationIssue;
    let entries = &d.credits.as_ref()?.tax_credit_bonds;
    if !entries.iter().any(|b| {
        b.origin
            .entity()
            .is_some_and(|e| e.income_source.is_some() || e.income_already_reported > 0)
    }) {
        return None;
    }
    let sources = income_sources(d);
    for (index, bond) in entries.iter().enumerate() {
        let Some(entity) = bond.origin.entity() else {
            continue;
        };
        if let Some(id) = entity.income_source {
            if entity.income_kind != TaxCreditIncomeKind::RicDistribution
                || !sources.iter().any(|source| {
                    source.id == id
                        && source.owner == bond.owner
                        && entity.income_already_reported <= source.amount
                })
            {
                let field = if entity.income_kind != TaxCreditIncomeKind::RicDistribution {
                    "income_kind"
                } else if sources.iter().any(|source| source.id == id && source.owner == bond.owner)
                {
                    "income_already_reported"
                } else {
                    "income_source"
                };
                return Some(
                    ValidationIssue::new("CREDIT_INCOME_SOURCE_INVALID")
                        .at_field(field)
                        .in_field("origin")
                        .in_list("tax_credit_bonds", index),
                );
            }
        } else if entity.income_already_reported > 0 {
            return Some(
                ValidationIssue::new("CREDIT_INCOME_SOURCE_NEEDED")
                    .at_field("income_source")
                    .in_field("origin")
                    .in_list("tax_credit_bonds", index),
            );
        }
    }
    for source in &sources {
        let allocated: i64 = entries
            .iter()
            .filter_map(|b| b.origin.entity())
            .filter(|e| e.income_source == Some(source.id))
            .map(|e| e.income_already_reported)
            .sum();
        if allocated > source.amount {
            return Some(
                ValidationIssue::new("CREDIT_INCOME_SOURCE_INVALID").at_field("tax_credit_bonds"),
            );
        }
    }
    None
}
