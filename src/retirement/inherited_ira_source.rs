use crate::dates::valid_birth;
use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Inherited IRA
/// One for each person you inherited from and each kind of IRA: all their traditional, SEP and SIMPLE IRAs together, or all their Roth IRAs together. The Forms 1099-R you choose it on supply the distributions and withholding. A spouse who treated the IRA as their own enters it as their own IRA instead.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InheritedIraAccount {
    /// Name of the person you inherited from
    #[libertas_size(min = 1)]
    pub decedent: String,
    /// Date they died
    #[libertas_date_only]
    // Tax structure: a current-return distribution or its source event cannot be after December 31, 2026.
    #[libertas_number(min = 19750101, max = 20261231)]
    pub died: u32,
    /// Kind of IRA
    pub kind: InheritedIraKindFacts,
}

/// Kind of IRA
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InheritedIraKindFacts {
    /// Traditional, SEP or SIMPLE IRA
    /// All of them from this person together.
    Traditional {
        /// After-tax basis passed to you
        /// Leave empty when the person had no nondeductible contributions: their Forms 8606 and the estate's allocation show it.
        /// [NullText]
        /// No after-tax traditional IRA basis passed to me
        /// [SetText]
        /// The after-tax basis allocated to me is
        basis: Option<InheritedIraBasis>,
        /// Federal estate tax on this IRA
        /// Leave empty unless a federal estate tax return was filed and tax paid.
        /// No estate-tax deduction is claimed without these facts.
        /// [NullText]
        /// Do not claim an estate-tax deduction for this inherited IRA
        /// [SetText]
        /// The paid federal estate-tax facts for this IRA are
        #[libertas_default($null)]
        estate_tax: Option<InheritedEstateTax>,
    },
    /// Roth IRA
    /// All Roth IRAs from this person together.
    Roth {
        /// Their first Roth IRA year
        /// The first tax year they contributed to or converted into any Roth IRA. The five-year clock continues after the death.
        // Tax structure: Roth IRA history starts with 1998 and cannot begin after this tax year.
        #[libertas_number(min = 1998, max = 2026)]
        first_year: u32,
        /// Contributions and conversions passed to you
        /// Needed only when the five years are not over. Leave empty if you don't have it: the App then taxes the whole withdrawal and tells you.
        /// [NullText]
        /// No inherited Roth basis record is needed or available
        /// [SetText]
        /// The inherited contributions and conversions allocated to me are
        basis: Option<crate::InheritedRothBasis>,
    },
}

/// Inherited IRA
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalInheritedIra {
    /// Name of the person inherited from
    #[libertas_size(min = 1)]
    pub decedent: String,
    /// Original owner's death date
    #[libertas_date_only]
    pub died: u32,
    /// Kind of IRA and its allocated basis
    pub kind: InheritedIraKindFacts,
}

/// Basis allocated to this inherited IRA group
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InheritedIraBasis {
    /// Unrecovered basis before this year's beneficiary payments
    /// Use this beneficiary's allocated basis from filed Form 8606 and estate/custodian records after prior withdrawals. For a 2026 death, use the completed estate allocation before this beneficiary's payments, including any death-year basis adjustment. Never use the entire decedent's basis when other beneficiaries received shares.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub prior: i64,
    /// Total December 31, 2026 value of this inherited IRA group
    /// Include every inherited traditional, SEP and SIMPLE IRA from this deceased owner belonging to this recipient, including accounts with no withdrawal. Exclude Roth IRAs, other deceased owners and the recipient's personal accounts. No transfer may be omitted from the year-end value; unresolved in-transit amounts need source reconciliation.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub year_end: i64,
}
/// Estate-return facts for inherited IRA income
/// The App calculates this year's section 691(c) deduction. Keep the administrator's remaining allocation for later distributions; never transfer another beneficiary's share or reuse value already used for a deduction.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InheritedEstateTax {
    /// Federal estate tax after credits, actually paid
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub paid: i64,
    /// Estate tax after credits without net income-in-respect-of-decedent value
    /// Use the estate administrator's recomputation excluding all IRD value net of related deductible expenses. Do not subtract unrelated estate assets or state inheritance taxes.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub without_ird: i64,
    /// Total estate-tax value of all income in respect of the decedent
    /// Gross IRD value in the entire estate, before related expenses, including other beneficiaries' shares and non-IRA items. This is the allocation denominator, not just this IRA's value.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub total_ird: i64,
    /// Unused estate-tax IRD value allocated to this beneficiary's IRA group
    /// Start with the administrator's estate-tax value for this beneficiary and deceased owner, excluding after-tax basis and amounts already used for prior-year deductions. Do not include later investment growth or another beneficiary's share.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub remaining_value: i64,
    /// Final estate values, payment and unused beneficiary allocation confirmed
    /// No unresolved estate adjustment or duplicate allocation to another income entry, beneficiary or year. The App does not prepare Form 706. Surviving-annuitant special computations and capital-gain IRD use different calculations. No takes no estate-tax deduction this year, with a review note.
    pub confirmed: bool,
}

/// Basis allocated to an inherited Roth IRA
/// Use this beneficiary's allocated balances before this year's payments, reconciled to prior withdrawals and all inherited Roth accounts from the same deceased owner. No unentered current-year basis adjustment. Keep other deceased owners and personal contributions separate.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InheritedRothBasis {
    /// Remaining regular-contribution basis allocated to this inherited group
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub regular: i64,
    /// Remaining conversion and qualified-plan-rollover basis allocated to this inherited group
    /// Combine originally taxable and nontaxable conversion portions; both are recovered before earnings. The inherited death exception removes conversion early-tax recapture regardless of conversion year.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub converted: i64,
}

impl InheritedEstateTax {
    fn valid(&self) -> bool {
        [self.paid, self.without_ird, self.total_ird, self.remaining_value]
            .iter()
            .all(|v| (0..=100000000000).contains(v))
    }
}
impl InheritedIraAccount {
    // The page rule, checked again on save with the same key: the Roth IRA's
    // first year is not after the year of death.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        matches!(&self.kind, InheritedIraKindFacts::Roth { first_year, .. }
                if *first_year > self.died / 10000)
        .then(|| {
            crate::validation::ValidationIssue::new("INHERITED_ROTH_FIRST_YEAR_AFTER_DEATH")
                .at_field("first_year")
                .in_field("kind")
        })
    }
    pub(crate) fn record(&self) -> FederalInheritedIra {
        FederalInheritedIra {
            decedent: self.decedent.clone(),
            died: self.died,
            kind: self.kind.clone(),
        }
    }
}
impl FederalInheritedIra {
    pub(crate) fn account(&self) -> InheritedIraAccount {
        InheritedIraAccount {
            decedent: self.decedent.clone(),
            died: self.died,
            kind: self.kind.clone(),
        }
    }
    pub(crate) fn roth(&self) -> bool {
        matches!(self.kind, InheritedIraKindFacts::Roth { .. })
    }
    // Section 408A(d)(2): a distribution is qualified once the original
    // owner's five-tax-year period is over; death satisfies the age leg.
    pub(crate) fn roth_qualified(&self) -> bool {
        matches!(self.kind, InheritedIraKindFacts::Roth { first_year, .. } if first_year <= 2021)
    }
    pub(crate) fn valid(&self) -> bool {
        valid_birth(self.died)
            && !self.decedent.trim().is_empty()
            && match &self.kind {
                InheritedIraKindFacts::Traditional { basis, estate_tax } => {
                    estate_tax.as_ref().is_none_or(InheritedEstateTax::valid)
                        && basis.as_ref().is_none_or(|b| {
                            [b.prior, b.year_end].iter().all(|v| (0..=100000000000).contains(v))
                        })
                }
                InheritedIraKindFacts::Roth { first_year, basis } => {
                    (1998..=self.died / 10000).contains(first_year)
                        && basis.as_ref().is_none_or(|v| {
                            [v.regular, v.converted].iter().all(|v| (0..=100000000000).contains(v))
                        })
                }
            }
    }
    // The traditional record's facts.
    pub(crate) fn basis(&self) -> Option<&InheritedIraBasis> {
        match &self.kind {
            InheritedIraKindFacts::Traditional { basis, .. } => basis.as_ref(),
            InheritedIraKindFacts::Roth { .. } => None,
        }
    }
    pub(crate) fn estate_tax(&self) -> Option<&InheritedEstateTax> {
        match &self.kind {
            InheritedIraKindFacts::Traditional { estate_tax, .. } => estate_tax.as_ref(),
            InheritedIraKindFacts::Roth { .. } => None,
        }
    }
    pub(crate) fn roth_basis(&self) -> Option<&crate::InheritedRothBasis> {
        match &self.kind {
            InheritedIraKindFacts::Roth { basis, .. } => basis.as_ref(),
            InheritedIraKindFacts::Traditional { .. } => None,
        }
    }
}
