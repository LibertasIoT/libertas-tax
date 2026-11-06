use crate::common::Owner;
#[allow(unused_imports)]
pub use crate::tax_credit_bond_source::TaxCreditBondResolvedAnswers;
pub use crate::tax_credit_bond_source::{
    FederalTaxCreditBond1097, FederalTaxCreditBondKind, Form1097Btc, Form1097BtcBondType,
    Form1097BtcIdentifierCode, Form1097BtcPaper, TaxCreditBondAnswers, TaxCreditDirect,
    TaxCreditEntity, TaxCreditIncomeKind, TaxCreditOrigin,
};
pub(crate) use crate::tax_credit_bond_source::{available_dollars, current_interest_cents, valid};

use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

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

pub(crate) fn positive_bonds(entries: &[FederalTaxCreditBond1097]) -> usize {
    entries.iter().filter(|bond| bond.current_dollars() + bond.carry_dollars() > 0).count()
}

// Several bonds with positive credit and a partly limited line 12: the IRS
// prescribes no order, so each bond's allowed share is the person's answer.
pub(crate) fn shares_needed(entries: &[FederalTaxCreditBond1097], line12: i64) -> bool {
    positive_bonds(entries) > 1 && line12 > 0 && line12 < available_dollars(entries)
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
    if shares_needed(entries, line12) {
        let mut used = Vec::with_capacity(entries.len());
        for bond in entries {
            let Some(answers) = bond.resolved_answers() else {
                return Err(BondAllocationError::MissingShare);
            };
            let Some(amount) = answers.allowed_credit_dollars else {
                return Err(BondAllocationError::MissingShare);
            };
            if !answers.allowed_credit_confirmed {
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
        let Some(answers) = bond.resolved_answers() else {
            return Err(BondAllocationError::MissingShare);
        };
        if !matches!(
            (answers.allowed_credit_dollars, answers.allowed_credit_confirmed),
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
        if answers.allowed_credit_dollars.is_some_and(|entered| entered != unique_amount) {
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
    instruction = "Read one annual Form 1097-BTC (Bond Tax Credit) exactly as printed. Copy each printed party into issuer and recipient, retaining masking, punctuation and leading zeros; never infer a household identity. Read the issuer telephone, calendar year, CORRECTED checkbox and both issuer-status checkboxes. Read the printed calendar year as a number, or null when no year is printed; never substitute zero or the form revision year. Check a printed calendar year against the supplied year and report a mismatch. A quarterly statement or corrected-quarter statement is not the annual form; report an error. Read box 1 Total, box 2a Code, box 2b Unique identifier, box 3 Bond type, box 4, each monthly amount in boxes 5a through 5l and box 6 Comments, in printed order. Read box 2a as C (CUSIP), A (account number) or O (other identifier), and box 3 as 101 (clean renewable energy bond) or 199 (other bond); use null when either code is blank and never infer a code from the identifier or other bond facts. Return null for clearly blank money boxes, numeric zero for a printed zero, and empty text for clearly blank text boxes, never the text 0. Box 2b unique_identifier is null when blank; preserve a printed identifier exactly. Report illegible or conflicting entries as errors; never guess. Do not add monthly amounts to the annual total, or infer bond class, issue date, carryovers, allowed shares, holding period, elections or confirmations. Duplicate copies of the same form count once.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.form.issuer.federal_id,@.form.recipient.federal_id,@.form.calendar_year,@.form.unique_identifier,@.form.box1_credit,@.form.january,@.form.february,@.form.march,@.form.april,@.form.may,@.form.june,@.form.july,@.form.august,@.form.september,@.form.october,@.form.november,@.form.december"
)]
pub struct TaxCreditBondDocument {
    /// Form 1097-BTC as printed
    #[libertas_llm_doc_field]
    pub form: Form1097BtcPaper,
    /// Accepted document identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
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
    #[libertas_enum_source("$.credit_income_sources")]
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

impl TaxCreditBondDocument {
    pub(crate) fn record(
        &self,
        accepted: Option<&FederalTaxCreditBond1097>,
        owner: Owner,
    ) -> FederalTaxCreditBond1097 {
        FederalTaxCreditBond1097 {
            id: self.id,
            printed: Some(crate::PrintedDocumentNames {
                recipient: self.form.recipient.clone(),
                issuer: Some(self.form.issuer.clone()),
                ..Default::default()
            }),
            owner,
            form: self.form.clone(),
            // Zero is the existing optional prior-carry default; it does not
            // supply a bond class, issue date, origin or confirmation.
            answers: accepted
                .filter(|bond| bond.id == self.id)
                .map(|bond| bond.answers.clone())
                .unwrap_or_default(),
        }
    }
}

impl From<FederalTaxCreditBond1097> for TaxCreditBondDocument {
    fn from(v: FederalTaxCreditBond1097) -> Self {
        Self { id: v.id, form: v.form }
    }
}

/// Personal answers for an accepted bond document
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct TaxCreditBondPage {
    /// Accepted document identity
    #[libertas_hidden]
    #[libertas_read_only]
    pub id: i64,
    /// Form 1097-BTC as printed
    #[libertas_read_only]
    pub form: alloc::boxed::Box<Form1097BtcPaper>,
    /// Personal bond records not settled by Form 1097-BTC
    #[libertas_read_only]
    pub answers: TaxCreditBondAnswersInput,
}

/// Personal records for a reported bond credit
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct TaxCreditBondAnswersInput {
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
    pub origin: Option<TaxCreditOriginInput>,
}

impl From<TaxCreditBondAnswersInput> for TaxCreditBondAnswers {
    fn from(v: TaxCreditBondAnswersInput) -> Self {
        Self {
            kind: v.kind,
            issued_on: v.issued_on,
            prior_carry_dollars: v.prior_carry_dollars,
            allowed_credit_dollars: v.allowed_credit_dollars,
            allowed_credit_confirmed: v.allowed_credit_confirmed,
            prior_carry_confirmed: v.prior_carry_confirmed,
            origin: v.origin.map(Into::into),
        }
    }
}

impl From<TaxCreditBondAnswers> for TaxCreditBondAnswersInput {
    fn from(v: TaxCreditBondAnswers) -> Self {
        Self {
            kind: v.kind,
            issued_on: v.issued_on,
            prior_carry_dollars: v.prior_carry_dollars,
            allowed_credit_dollars: v.allowed_credit_dollars,
            allowed_credit_confirmed: v.allowed_credit_confirmed,
            prior_carry_confirmed: v.prior_carry_confirmed,
            origin: v.origin.map(Into::into),
        }
    }
}

impl From<&FederalTaxCreditBond1097> for TaxCreditBondPage {
    fn from(v: &FederalTaxCreditBond1097) -> Self {
        Self {
            id: v.id,
            form: alloc::boxed::Box::new(v.form.clone()),
            answers: v.answers.clone().into(),
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
                crate::community::share(setup, entry.owner.owned(), entry.community, &entry.income)
            });
            // Preserve the fund's actual shareholder designation on its DIV.
            // Underlying bond interest does not establish a nonqualified-only
            // cap on an already reported RIC distribution (IRC853A/854).
            let amount = share.map_or(components.ordinary_dividends, |share| {
                share.filer(components.ordinary_dividends)
            });
            Some(CreditIncomeSource {
                id: entry.id,
                owner: entry.owner.owned(),
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
    source_document_issue(d).map(|(_, issue)| issue)
}

pub(crate) fn source_document_issue(
    d: &crate::FederalDraft,
) -> Option<(Option<i64>, crate::validation::ValidationIssue)> {
    use crate::validation::ValidationIssue;
    let entries = &d.credits.as_ref()?.tax_credit_bonds;
    if !entries.iter().any(|b| {
        b.answers
            .origin
            .as_ref()
            .and_then(TaxCreditOrigin::entity)
            .is_some_and(|e| e.income_source.is_some() || e.income_already_reported > 0)
    }) {
        return None;
    }
    let sources = income_sources(d);
    for (index, bond) in entries.iter().enumerate() {
        let Some(entity) = bond.answers.origin.as_ref().and_then(TaxCreditOrigin::entity) else {
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
                return Some((
                    Some(bond.id),
                    ValidationIssue::new("CREDIT_INCOME_SOURCE_INVALID")
                        .at_field(field)
                        .in_field("origin")
                        .in_field("answers")
                        .in_list("tax_credit_bonds", index),
                ));
            }
        } else if entity.income_already_reported > 0 {
            return Some((
                Some(bond.id),
                ValidationIssue::new("CREDIT_INCOME_SOURCE_NEEDED")
                    .at_field("income_source")
                    .in_field("origin")
                    .in_field("answers")
                    .in_list("tax_credit_bonds", index),
            ));
        }
    }
    for source in &sources {
        let allocated: i64 = entries
            .iter()
            .filter_map(|b| b.answers.origin.as_ref().and_then(TaxCreditOrigin::entity))
            .filter(|e| e.income_source == Some(source.id))
            .map(|e| e.income_already_reported)
            .sum();
        if allocated > source.amount {
            return Some((
                None,
                ValidationIssue::new("CREDIT_INCOME_SOURCE_INVALID").at_field("tax_credit_bonds"),
            ));
        }
    }
    None
}
