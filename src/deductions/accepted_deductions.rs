use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Voluntary itemizing election
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum ItemizeOverride {
    /// Itemize even if the standard deduction is larger
    Itemize,
}

/// Deduction choice
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum DeductionChoice {
    /// Use the supported choice with the lower total tax
    Automatic,
    /// Itemized deductions
    Itemized,
}

/// Deduction
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum OtherItemizedLine {
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
        source: crate::BondPremiumSource,
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
        owner: crate::Owner,
        /// Expenses
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        amount: i64,
    },
}
impl OtherItemizedLine {
    pub(crate) fn amount(&self) -> i64 {
        match self {
            Self::GenerationSkippingTax { amount }
            | Self::BondPremium { amount, .. }
            | Self::DebtInstrumentLoss { amount }
            | Self::ImpairmentWork { amount, .. } => *amount,
        }
    }
    pub(crate) fn valid(&self) -> bool {
        (1..=100000000000).contains(&self.amount())
    }
}

/// Deductions
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalDeductions {
    /// Prior-year deduction facts keyed to refunded vehicle-interest documents
    pub vehicle_recoveries: Vec<crate::VehicleInterestRecovery>,
    /// Investment borrowing retained independently of the deduction method
    #[libertas_default($null)]
    pub investment_interest: Option<crate::InvestmentBorrowing>,
    /// Home mortgages, retained independently of the deduction method
    /// Empty when there is no home mortgage interest.
    /// ----
    /// Mortgage
    pub mortgage: Vec<crate::Mortgage1098Source>,
    /// Earlier-year unemployment repayment deduction or credit
    /// Supply original-return records when repayments recorded in Income exceed $3,000. Automatic compares both complete supported methods; no printed form is produced.
    #[libertas_default(None)]
    pub repayment: crate::RepaymentClaim,
    /// Unrecovered pension or annuity cost at the final annuitant's death
    /// ----
    /// Ended annuity contract
    pub annuity_losses: Vec<crate::AnnuityLoss>,
    /// Optional election to itemize even when the standard deduction is larger
    pub itemize_override: Option<ItemizeOverride>,
    /// Retained itemized deduction facts, independent of the selected method
    pub itemized: ItemizedExpenses,
    /// Accepted separate-return spouse itemizing answer, absent until asked
    pub spouse_itemizes: Option<bool>,
    /// Current-year cash gifts to eligible public charities, excluding goods/services received
    /// Zero when there are none.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub cash_charity: i64,
    /// Cash to a donor-advised fund or section 509(a)(3) supporting organization
    /// Deductible only when itemizing; it never counts toward the non-itemizer cash deduction. Enter 0 if none.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub fund_cash_charity: i64,
    /// Cash to a private nonoperating foundation or other 30% organization
    /// Non-federally chartered veterans' organizations, fraternal societies, nonprofit cemetery companies, and gifts held for the use of any charity. Enter 0 if none.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_cash_charity: i64,
    /// Unreimbursed costs while giving services to eligible charities or government units
    /// ----
    /// Volunteer expense
    pub volunteer_expenses: Vec<crate::VolunteerExpense>,
    /// Eligible section 170(g) education-program students living in this household
    /// ----
    /// Hosted student
    pub hosted_students: Vec<crate::HostedStudent>,
    /// Section 170(n) expenses of AEWC-recognized captains carrying out sanctioned Native Alaskan subsistence whaling
    // Tax structure: one whaling captain claim per filer.
    #[libertas_size(max = 2)]
    /// ----
    /// Captain's expenses
    pub whaling_captains: Vec<crate::WhalingCaptainClaim>,
    /// Property contributions in 2026
    /// ----
    /// Gift
    pub property_gifts: Vec<crate::PropertyGift>,
    /// Vehicle, boat and airplane contributions in 2026 (Form 1098-C)
    /// ----
    /// Gift
    pub vehicle_gifts: Vec<crate::VehicleGiftSource>,
    /// Accepted Forms 1098-C
    pub vehicle_gift_documents: Vec<crate::Form1098CDocument>,
    /// Qualified appraisal reports and declarations
    /// ----
    /// Appraisal
    pub appraisals: Vec<crate::Appraisal>,
    /// Deduct every current capital-gain gift to a public charity at basis
    pub basis_election: bool,
    /// Reconciled charitable contribution carryovers
    /// ----
    /// Carryover
    pub charity_carryovers: Vec<crate::CharityCarryover>,
    /// Ordinary, qualified or undeclared 2026 personal-use casualty and theft events
    /// ----
    /// Disaster event
    pub personal_casualties: Vec<crate::PersonalCasualty>,
    /// Personal vehicle loan interest
    /// ----
    /// Vehicle loan
    pub vehicle_loans: Vec<crate::VehicleLoanSource>,
}

/// Expenses for an itemized deduction
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ItemizedExpenses {
    /// Direct connection of existing paid income tax with investment income
    pub investment_income_tax: crate::InvestmentIncomeTax,
    /// Unreimbursed eligible medical and dental expenses, excluding HSA/FSA reimbursements
    /// Exclude long-term-care premiums entered below and premiums reported in Marketplace coverage, business owner health insurance, the public-safety insurance election or accepted SSA-1099 and RRB-1099-R statements. The App adds their remaining eligible portion once.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub medical: i64,
    /// Personal qualified long-term-care premiums
    /// Enter full premiums before age limits. Business and public-safety premiums belong in their own sections; the App shares each insured person's limit across all three.
    pub long_term_care: Vec<crate::MedicalCarePremium>,
    /// State/local income or sales tax
    pub personal_tax: crate::PersonalTaxElection,
    /// Taxes on personal-use real estate (Schedule A line 5b)
    /// Exclude home taxes already entered in Business use of your homes; the App includes their remaining personal share automatically. Exclude foreign taxes, service charges and assessments.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub real_estate_taxes: i64,
    /// Value-based personal property taxes (Schedule A line 5c)
    /// Eligible annual value-based taxes on personal-use property. Exclude real-estate taxes, business/rental/investment shares, special assessments, service charges and taxes entered elsewhere.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub personal_property_taxes: i64,
    /// Real-estate taxes on property held solely for investment appreciation
    /// Enter taxes paid for investment land or other real estate held for taxable appreciation, outside a business or rental. Exclude personal-use shares, special assessments for improvements, taxes added to basis under a capitalization election, and amounts entered elsewhere. These taxes are separate from personal state/local taxes. Do not enter income taxes here.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub investment_property_taxes: i64,
    /// Other itemized deductions (Schedule A lines 6, 17f, 17g and 17j)
    pub other_lines: Vec<crate::OtherItemizedLine>,
    /// People who paid the investment-property taxes
    /// This relationship applies to the investment-property expense, not personal or business taxes.
    /// [NullText]
    /// The payer is not established
    /// [SetText]
    /// The investment-property taxes were paid by
    #[libertas_default($null)]
    pub investment_property_tax_owners: Option<crate::DeductionOwners>,
    /// People who paid the state and local income taxes
    /// This is the actual payer of the income-tax payment above, independently of its investment-income allocation method.
    /// [NullText]
    /// Ownership not supplied
    /// [SetText]
    /// The income taxes were paid by
    #[libertas_default($null)]
    pub income_tax_owners: Option<crate::DeductionOwners>,
}
impl FederalDeductions {
    pub(crate) fn choice(&self) -> DeductionChoice {
        if self.itemize_override.is_some() {
            DeductionChoice::Itemized
        } else {
            DeductionChoice::Automatic
        }
    }
    pub(crate) fn other_itemized_lines(&self) -> &[OtherItemizedLine] {
        &self.itemized.other_lines
    }
    pub(crate) fn personal_property_taxes(&self) -> i64 {
        self.itemized.personal_property_taxes
    }
    pub(crate) fn medical(&self) -> i64 {
        self.itemized.medical
    }
    pub(crate) fn long_term_care(&self) -> &[crate::MedicalCarePremium] {
        &self.itemized.long_term_care
    }
    pub(crate) fn state_local_taxes(&self) -> i64 {
        self.itemized.personal_tax.paid()
            + self.itemized.real_estate_taxes
            + self.itemized.personal_property_taxes
    }
    pub(crate) fn investment_property_taxes(&self) -> i64 {
        self.itemized.investment_property_taxes
    }
    pub(crate) fn investment_property_tax_owners(&self) -> Option<crate::DeductionOwners> {
        self.itemized.investment_property_tax_owners
    }
    pub(crate) fn income_tax_owners(&self) -> Option<crate::DeductionOwners> {
        self.itemized.income_tax_owners
    }
    pub(crate) fn property_taxes(&self) -> i64 {
        self.itemized.real_estate_taxes
    }
    pub(crate) fn income_tax_paid(&self) -> i64 {
        match self.itemized.personal_tax {
            crate::PersonalTaxElection::Income { paid, .. } => paid,
            _ => 0,
        }
    }
    pub(crate) fn state_income_tax_elected(&self) -> bool {
        matches!(&self.itemized.personal_tax, crate::PersonalTaxElection::Income { .. })
    }
    pub(crate) fn state_sales_tax_elected(&self) -> bool {
        matches!(&self.itemized.personal_tax, crate::PersonalTaxElection::Sales { .. })
    }
    pub(crate) fn investment_income_tax_bounds(
        &self,
        salt_paid: &crate::kernel::ratio::exact::Exact,
        salt_allowed: i64,
    ) -> [i64; 2] {
        self.itemized.investment_income_tax.allowed_bounds(
            self.income_tax_paid(),
            salt_paid,
            salt_allowed,
        )
    }
    pub(crate) fn no_investment_income_tax(&self) -> bool {
        self.income_tax_paid() == 0
            || matches!(
                self.itemized.investment_income_tax,
                crate::InvestmentIncomeTax::None
                    | crate::InvestmentIncomeTax::Records { paid_part: 0, .. }
            )
    }
    pub(crate) fn investment_interest(&self) -> Option<crate::InvestmentInterest> {
        self.investment_interest.as_ref().map(crate::InvestmentBorrowing::aggregate)
    }
    pub(crate) fn valid_taxes(&self) -> bool {
        let f = &self.itemized;
        self.investment_interest.as_ref().is_none_or(|v| v.valid()) && {
            f.personal_tax.valid()
                && f.investment_income_tax.valid(
                    f.personal_tax.paid(),
                    matches!(f.personal_tax, crate::PersonalTaxElection::Income { .. }),
                )
                && crate::health::medical_care_issue(&f.long_term_care).is_none()
                && (0..=100000000000).contains(&f.personal_property_taxes)
                && (0..=100000000000).contains(&f.real_estate_taxes)
                && f.other_lines.len() <= 20
                && f.other_lines.iter().all(OtherItemizedLine::valid)
        }
    }
    pub(crate) fn income_tax_niit(
        &self,
        salt: i64,
        total_state_local_taxes: &crate::kernel::ratio::exact::Exact,
        investment: i64,
        agi: i64,
    ) -> Option<i64> {
        self.itemized.personal_tax.niit_expense(salt, total_state_local_taxes, investment, agi)
    }
}
