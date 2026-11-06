#[allow(unused_imports)]
use super::*;
pub use crate::accepted_deductions::{
    DeductionChoice, FederalDeductions, ItemizeOverride, ItemizedExpenses, OtherItemizedLine,
};

/// Deductions
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Deductions {
    /// Deduction method
    /// The App compares the deductions from your records automatically.
    /// [NullText]
    /// Let the App choose
    /// [SetText]
    /// Itemize even if the standard deduction is larger
    #[libertas_default($null)]
    pub itemize_override: Option<ItemizeOverride>,
    /// Cash gifts to charity in 2026
    /// Cash, check or card gifts to public charities, with receipts and written acknowledgment for $250 or more. Donor-advised funds, supporting organizations and private foundations belong in the next two boxes. Enter 0 if none.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub cash_gifts: i64,
    /// Cash to a donor-advised fund or section 509(a)(3) supporting organization
    /// Deductible only when itemizing; it never counts toward the non-itemizer cash deduction. Enter 0 if none.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub fund_cash_gifts: i64,
    /// Cash to a private nonoperating foundation or other 30% organization
    /// Non-federally chartered veterans' organizations, fraternal societies, nonprofit cemetery companies, and gifts held for the use of any charity. Enter 0 if none.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_cash_gifts: i64,
    /// Interest on a loan for a new personal vehicle
    /// You can deduct up to $10,000 whether or not you itemize (Schedule 1-A). The loan was taken out after 2024 to buy a new car, minivan, van, SUV, pickup or motorcycle under 14,000 pounds for personal use, and is secured by the vehicle. Not a lease, and not a loan from a relative.
    #[libertas_unordered]
    #[libertas_read_only]
    /// ----
    /// Vehicle loan
    pub car_loans: Vec<crate::VehicleLoanDocumentView>,
    /// Refunds of vehicle interest deducted in an earlier year
    /// Only forms reporting a refund need prior-return facts.
    #[libertas_read_only]
    pub car_refunds: Vec<crate::VehicleRefundDocumentView>,
    /// Casualty and theft losses (Form 4684)
    /// Personal property damaged, destroyed or stolen in 2026. A loss from a federally or state-declared disaster is deductible when you itemize; any other loss only offsets casualty gains. Business or rental property and investment scams are under More itemized deductions.
    #[libertas_unordered]
    /// ----
    /// Event
    pub casualties: Vec<crate::PersonalCasualtyPage>,
    /// Repaid more than $3,000 of income you reported in an earlier year
    /// Offered when your unemployment or railroad benefits show earlier-year repayments above $3,000. You can deduct the repayment or take a credit for the tax it caused (claim of right). Repayments of $3,000 or less give no deduction.
    #[libertas_default(None)]
    #[libertas_constrained_by("$.allowed_repayment")]
    pub repayment: crate::RepaymentClaim,
}
// The question a separate return on the standard deduction asks after the
// deductions page; each choice is a real answer. The stored
// SeparateSpouseBoxes keeps its "not answered yet" value for a return not
// asked yet.
/// Your spouse's standard deduction boxes
/// Only when your spouse had no gross income, is not filing a return and cannot be claimed as another person's dependent (Form 1040 line 12d). Each box adds $1,650 to your standard deduction.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum SpouseBoxes {
    /// None of these
    NoneOfThese,
    /// Born before January 2, 1962
    Born1961OrEarlier,
    /// Blind
    Blind,
    /// Born before January 2, 1962 and blind
    BornAndBlind,
}

impl From<SpouseBoxes> for crate::SeparateSpouseBoxes {
    fn from(value: SpouseBoxes) -> Self {
        match value {
            SpouseBoxes::NoneOfThese => Self::NoneOfThese,
            SpouseBoxes::Born1961OrEarlier => Self::Born1961OrEarlier,
            SpouseBoxes::Blind => Self::Blind,
            SpouseBoxes::BornAndBlind => Self::BornAndBlind,
        }
    }
}

impl crate::SeparateSpouseBoxes {
    // The saved answer, or None while the return has not been asked.
    pub(crate) fn answer(self) -> Option<SpouseBoxes> {
        match self {
            Self::NotAnswered => None,
            Self::NoneOfThese => Some(SpouseBoxes::NoneOfThese),
            Self::Born1961OrEarlier => Some(SpouseBoxes::Born1961OrEarlier),
            Self::Blind => Some(SpouseBoxes::Blind),
            Self::BornAndBlind => Some(SpouseBoxes::BornAndBlind),
        }
    }
}

/// Medical and dental expenses
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Medical {
    /// Unreimbursed medical and dental expenses
    /// Not paid from an HSA or FSA, and not premiums entered elsewhere. Medicare premiums printed on accepted SSA-1099 and RRB-1099-R statements are included automatically; do not repeat them here. Include what you paid for your child even when the child's other parent claims them. Leave out what you paid for someone another person claims under a multiple support agreement you signed.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub expenses: i64,
    /// Long-term-care insurance premiums
    #[libertas_unordered]
    /// ----
    /// Policy
    pub long_term_care: Vec<CarePremium>,
    /// People on this return
    #[libertas_hidden]
    #[libertas_read_only]
    pub people: Vec<PersonChoice>,
}
/// Policy
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CarePremium {
    /// Insured person
    #[libertas_enum_source("^<Medical>.people")]
    pub person: i64,
    /// Premiums paid in 2026
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub premiums: i64,
    /// Actual contract qualification
    pub qualified: crate::CareContractQualification,
}
/// Taxes you paid
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Taxes {
    /// Income tax directly connected with investment income
    /// This is part of the income tax paid below, not another payment.
    pub investment_income_tax: crate::InvestmentIncomeTax,
    /// State and local tax you deduct (Schedule A line 5a)
    #[libertas_default(IncomeTax)]
    pub kind: SaltElection,
    /// Real estate taxes (line 5b)
    /// State and local taxes on personal-use real estate, including lender escrow payments (Form 1098 box 10). Exclude taxes already entered in Business use of your homes; the App includes their remaining personal share automatically. Exclude foreign taxes, improvement assessments and service charges.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub real_estate: i64,
    /// Personal property taxes (line 5c)
    /// Yearly state and local taxes based on value alone, such as the value-based part of a car registration fee.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub personal_property: i64,
    /// People who paid the state and local income taxes
    /// This is the actual payer of the income-tax payment above, independently of its investment-income allocation method.
    /// [NullText]
    /// Ownership not supplied
    /// [SetText]
    /// The income taxes were paid by
    #[libertas_default($null)]
    pub income_tax_owners: Option<crate::DeductionOwners>,
}
/// State and local tax you deduct
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum SaltElection {
    /// Income tax
    IncomeTax {
        /// State and local income tax paid in 2026
        /// W-2 box 17, W-2G box 15, estimated payments, and any balance paid with your 2025 state return.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        paid: i64,
    },
    /// General sales tax
    SalesTax {
        /// Sales tax from the IRS table or your receipts
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        paid: i64,
    },
}
/// Investment interest and other deductions
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct OtherItemized {
    /// Investment borrowing costs and retained separate regular and AMT facts
    /// [NullText]
    /// No investment borrowing expense or prior carryover is claimed
    /// [SetText]
    /// The investment borrowing expenses and carryovers are
    #[libertas_default($null)]
    pub investment_interest: Option<crate::InvestmentBorrowingPage>,
    /// Real estate taxes on land held for investment
    /// Not your home, a rental or a business property.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub investment_property_taxes: i64,
    /// Annuity cost not recovered at death (Schedule A line 17i)
    /// On the final return of someone who died in 2026 while receiving a pension or annuity they had paid into, and no one receives further payments. Listed for the retirement income of a filer who died.
    #[libertas_unordered]
    /// ----
    /// Ended annuity
    pub annuity_losses: Vec<FinalAnnuityLoss>,
    /// Other itemized deductions
    /// Generation-skipping transfer tax on a trust distribution, amortizable bond premium, a loss on a contingent payment or inflation-indexed debt instrument, and impairment-related work expenses. Casualty losses of investment property are under More itemized deductions.
    #[libertas_unordered]
    /// ----
    /// Deduction
    pub other_lines: Vec<OtherItemizedInput>,

    /// Retirement income of a filer who died
    #[libertas_hidden]
    #[libertas_read_only]
    pub contracts: Vec<AnnuityContract>,
    /// Accepted annual instrument sources for a manually documented deduction
    #[libertas_hidden]
    #[libertas_read_only]
    pub bond_instruments: Vec<crate::BondPremiumInstrument>,
    /// People who paid the investment-property taxes
    /// This relationship applies to the investment-property expense, not personal or business taxes.
    /// [NullText]
    /// The payer is not established
    /// [SetText]
    /// The investment-property taxes were paid by
    #[libertas_default($null)]
    pub investment_property_tax_owners: Option<crate::DeductionOwners>,
}
/// Deduction
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum OtherItemizedInput {
    /// Generation-skipping transfer tax on a trust income distribution (Schedule A line 6)
    /// The GST tax on a taxable distribution included in your income, from your Form 706-GS(D).
    GenerationSkippingTax {
        /// Tax paid
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        amount: i64,
    },
    /// Amortizable bond premium (Schedule A line 17f)
    /// Premium on taxable bonds bought before October 23, 1986; the premium on a bond above this year's interest on it, up to the interest you included from it in earlier years less premium deducted before; and a premium carryforward left when you sold the bond. Bond premium on a Form 1099-INT reduces that form's interest instead.
    BondPremium {
        /// Documented source of this manually figured deduction
        /// A source already supplying a computed annual excess deduction cannot also be entered here.
        source: BondPremiumSelection,
        /// Deduction
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        amount: i64,
    },
    /// Loss on a contingent payment or inflation-indexed debt instrument (Schedule A line 17g)
    /// The ordinary loss shown by your broker's statement or the issuer's schedule, such as a deflation adjustment on Treasury Inflation-Protected Securities above the year's interest, up to the interest you included in earlier years.
    DebtInstrumentLoss {
        /// Loss
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        amount: i64,
    },
    /// Impairment-related work expenses (Schedule A line 17j)
    /// Costs of an employee with a physical or mental disability for attendant care at work and other expenses needed to be able to work, as figured on Form 2106. A business owner deducts them on that business.
    ImpairmentWork {
        /// Whose expenses
        #[libertas_default(Taxpayer)]
        #[libertas_constrained_by("$.allowed_owners")]
        owner: crate::Owner,
        /// Expenses
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        amount: i64,
    },
}
/// Ended annuity
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FinalAnnuityLoss {
    /// Retirement income
    #[libertas_enum_source("^^^.contracts")]
    pub contract: i64,
    /// Annuity starting date
    #[libertas_date_only]
    // Tax structure: this date or year belongs to the stated current or retained-return period.
    #[libertas_number(min = 19860702, max = 20261231)]
    pub started: u32,
    /// Net investment in the contract
    /// What was paid in, from the contract records, before any tax-free recovery.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub cost: i64,
    /// Tax-free part recovered through 2025
    /// Include the recovery of every earlier annuitant. The 2026 tax-free part comes from the retirement income entry.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub recovered: i64,
    /// Other tax-free refunds or death settlements from the contract
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub refunds: i64,
    /// All payments stopped at this annuitant's death
    /// No surviving annuitant or guaranteed payments continue, and the cost and recovery agree with the retirement income entry.
    pub eligible: bool,
}
/// Retirement income
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AnnuityContract {
    /// Income entry
    #[libertas_key]
    #[libertas_unique]
    #[libertas_hidden]
    pub id: i64,
    /// Payer
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
}

impl FederalDeductions {
    /// The loans the calculations read, derived from the stored Forms 1098 and their answers.
    /// A source whose conversion fails is an error, never an absent loan: the stored-value
    /// validity check refuses the estimate, and display callers show the remaining loans.
    pub(crate) fn mortgage_loans(
        &self,
    ) -> Result<Vec<crate::MortgageLoan>, crate::Mortgage1098ConversionError> {
        self.mortgage.iter().map(crate::MortgageLoan::try_from).collect()
    }
    // One lender-reported loan incurred in 2020 whose average balance ($100,000)
    // is within every limit, so all of `value` is deductible.
    #[cfg(test)]
    pub(crate) fn set_mortgage_interest(&mut self, value: i64) {
        let mut source = crate::Mortgage1098Source::printed(crate::Form1098 {
            calendar_year: 2026,
            box_1: value,
            box_2: 10_000_000,
            box_3: 20200601,
            ..Default::default()
        });
        source.id = 1;
        // The borrower is you, as your section added it.
        source.assigned = crate::DocumentRecipient::Taxpayer;
        source.owner_source = Some(crate::OwnerSource::Added);
        source.property = "Synthetic home".into();
        source.lender = crate::MortgageLender::Institution;
        source.details = crate::MortgageDetails::FirstLender(crate::MortgageFirstLender {
            home: crate::MortgageHome::Main,
            purpose: crate::MortgageUse::Home,
            average: crate::AverageBalance::Statements { average: 10_000_000 },
        });
        self.mortgage = alloc::vec![source];
    }
}

impl FederalDeductions {
    #[cfg(test)]
    pub(crate) fn set_cash_charity(&mut self, value: i64) {
        self.cash_charity = value;
    }
}

/// Source selected for a manually documented bond-premium deduction
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum BondPremiumSelection {
    /// A different documented bond, with its own deduction records
    OtherDocumented {
        /// Instrument name or description, not an accepted record identity
        #[libertas_size(min = 1)]
        instrument: String,
    },
    /// An annual instrument already accepted on Interest and dividends
    AcceptedAnnualInstrument {
        /// Accepted instrument
        #[libertas_enum_source("^<OtherItemized>.bond_instruments")]
        instrument: i64,
    },
}
impl From<crate::BondPremiumSource> for BondPremiumSelection {
    fn from(source: crate::BondPremiumSource) -> Self {
        match source {
            crate::BondPremiumSource::OtherDocumented { instrument } => {
                Self::OtherDocumented { instrument }
            }
            crate::BondPremiumSource::AcceptedAnnualInstrument { instrument } => {
                Self::AcceptedAnnualInstrument { instrument }
            }
        }
    }
}
impl From<BondPremiumSelection> for crate::BondPremiumSource {
    fn from(source: BondPremiumSelection) -> Self {
        match source {
            BondPremiumSelection::OtherDocumented { instrument } => {
                Self::OtherDocumented { instrument }
            }
            BondPremiumSelection::AcceptedAnnualInstrument { instrument } => {
                Self::AcceptedAnnualInstrument { instrument }
            }
        }
    }
}
impl From<OtherItemizedLine> for OtherItemizedInput {
    fn from(row: OtherItemizedLine) -> Self {
        match row {
            OtherItemizedLine::GenerationSkippingTax { amount } => {
                Self::GenerationSkippingTax { amount }
            }
            OtherItemizedLine::BondPremium { source, amount } => {
                Self::BondPremium { source: source.into(), amount }
            }
            OtherItemizedLine::DebtInstrumentLoss { amount } => Self::DebtInstrumentLoss { amount },
            OtherItemizedLine::ImpairmentWork { owner, amount } => {
                Self::ImpairmentWork { owner, amount }
            }
        }
    }
}
impl From<OtherItemizedInput> for OtherItemizedLine {
    fn from(row: OtherItemizedInput) -> Self {
        match row {
            OtherItemizedInput::GenerationSkippingTax { amount } => {
                Self::GenerationSkippingTax { amount }
            }
            OtherItemizedInput::BondPremium { source, amount } => {
                Self::BondPremium { source: source.into(), amount }
            }
            OtherItemizedInput::DebtInstrumentLoss { amount } => {
                Self::DebtInstrumentLoss { amount }
            }
            OtherItemizedInput::ImpairmentWork { owner, amount } => {
                Self::ImpairmentWork { owner, amount }
            }
        }
    }
}
