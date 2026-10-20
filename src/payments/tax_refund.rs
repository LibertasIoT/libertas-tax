use crate::kernel::phase;
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Filing status on the prior return for the 2013–2017 overall itemized-deduction limit
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum PeaseFilingStatus {
    /// Single
    Single,
    /// Married filing jointly or qualifying surviving spouse
    JointOrSurviving,
    /// Head of household
    HeadOfHousehold,
    /// Married filing separately
    MarriedSeparate,
}
impl PeaseFilingStatus {
    fn threshold(&self, year: i32) -> i64 {
        match (year, self) {
            (2013, Self::Single) => 25_000_000,
            (2013, Self::JointOrSurviving) => 30_000_000,
            (2013, Self::HeadOfHousehold) => 27_500_000,
            (2013, Self::MarriedSeparate) => 15_000_000,
            (2014, Self::Single) => 25_420_000,
            (2014, Self::JointOrSurviving) => 30_505_000,
            (2014, Self::HeadOfHousehold) => 27_965_000,
            (2014, Self::MarriedSeparate) => 15_252_500,
            (2015, Self::Single) => 25_825_000,
            (2015, Self::JointOrSurviving) => 30_990_000,
            (2015, Self::HeadOfHousehold) => 28_405_000,
            (2015, Self::MarriedSeparate) => 15_495_000,
            (2016, Self::Single) => 25_940_000,
            (2016, Self::JointOrSurviving) => 31_130_000,
            (2016, Self::HeadOfHousehold) => 28_535_000,
            (2016, Self::MarriedSeparate) => 15_565_000,
            (2017, Self::Single) => 26_150_000,
            (2017, Self::JointOrSurviving) => 31_380_000,
            (2017, Self::HeadOfHousehold) => 28_765_000,
            (2017, Self::MarriedSeparate) => 15_690_000,
            _ => 0,
        }
    }
}

/// Retained 2013–2017 Schedule A line 29 overall itemized-deduction worksheet
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PeaseWorksheet {
    /// Prior Form 1040 adjusted gross income; worksheet line 5
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub agi: i64,
    /// Total prior itemized deductions before the overall limit; worksheet line 1
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub before_limit: i64,
    /// Protected deductions excluded from the 80% limit; worksheet line 2
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub protected: i64,
    /// Filing status on that prior return; determines the year's worksheet line 6 threshold
    pub filing_status: PeaseFilingStatus,
}
impl PeaseWorksheet {
    fn itemized_after(&self, year: i32, source_reduction: i64) -> i64 {
        let before = self.before_limit - source_reduction;
        let subject = before - self.protected;
        let eighty_percent = crate::kernel::ratio::half_offset_i64(subject * 4, 5);
        let three_percent = crate::kernel::ratio::half_offset_i64(
            phase::floor(self.agi - self.filing_status.threshold(year), 0) * 3,
            100,
        );
        before - phase::cap(eighty_percent, [three_percent])
    }
    fn valid(&self, year: i32, taxes: i64, original_itemized: i64) -> bool {
        [self.agi, self.before_limit, self.protected]
            .iter()
            .all(|value| (0..=100_000_000_000).contains(value))
            && (2013..=2017).contains(&year)
            && self.before_limit >= self.protected + taxes
            && self.agi > self.filing_status.threshold(year)
            && self.itemized_after(year, 0) < self.before_limit
            && self.itemized_after(year, 0) == original_itemized
    }
}

/// One federal deduction year in a state/local income-tax refund spanning prior payment years
/// Publication 525 allocates one recovery pro rata by the payments made in each year, then applies that year's own tax-benefit worksheet. See https://www.irs.gov/publications/p525.
// Original Schedule A line references and the 2017 Pease worksheet:
// https://www.irs.gov/pub/irs-prior/f1040sa--2012.pdf
// https://www.irs.gov/pub/irs-prior/i1040sca--2017.pdf
// https://www.irs.gov/pub/irs-prior/f1040sa--2018.pdf
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct TaxRefundYearPayment {
    /// Federal calendar year in which this part of the state/local tax was PAID, 2010–2025
    /// This narrow case also requires the same payment to have been deducted on that year's federal Schedule A; allocation follows payment timing, and the tax-benefit worksheet follows the confirmed deduction year.
    // Tax-year structure: the recorded date/year cannot be later than this applicable tax-year window.
    #[libertas_number(min = 2010, max = 2025)]
    pub payment_year: i32,
    /// Tax paid in this federal year toward the ONE state/local return that produced this refund
    /// Use its withholding and estimated/final payments for this same state return only; all year entries together must cover every payment that produced the refund. This is the pro-rata weight, not all state taxes deducted in the federal year.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub payment_for_refunded_return: i64,
    /// All state/local income tax paid and included in this year's federal Schedule A
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub income_paid: i64,
    /// General sales tax deduction available instead in this year
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub sales_available: i64,
    /// Prior Schedule A deductible taxes before any SALT cap: line 9 for 2010–2017, or line 5d for 2018–2025
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub salt_paid: i64,
    /// Prior Schedule A allowed taxes: line 9 for 2010–2017 (equal to taxes paid), or line 5e after the SALT cap for 2018–2025
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub salt_deducted: i64,
    /// Prior Schedule A total itemized deductions: line 29 for 2010–2017, or line 17 for 2018–2025
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub itemized: i64,
    /// Standard deduction available for this prior federal year
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub standard: i64,
    /// Retained 2013–2017 Schedule A line 29 overall itemized-deduction worksheet, when it applied
    /// Leave empty for 2010–2012 and 2018–2025. When a 2013–2017 overall limit applied, enter its original worksheet so the recovered share can refigure that year's deduction.
    #[libertas_default($null)]
    pub overall_limit: Option<PeaseWorksheet>,
    /// This deduction year's ordinary Publication 525 refund-worksheet conditions are met
    /// Confirm positive prior taxable income and tax; no unused credit, AMT, preferential-rate recomputation, changed joint ownership, prior NIIT deduction or other itemized recovery from this federal year. Each year's worksheet is independent. That year's federal return shows it. No leaves this refund out of the estimate with a notice; the full tax-benefit worksheet is not calculated.
    pub ordinary: bool,
}

impl TaxRefundYearPayment {
    fn as_single_year(&self) -> TaxRefundHistory {
        TaxRefundHistory::IncomeTax {
            deduction_year: self.payment_year,
            income_paid: self.income_paid,
            sales_available: self.sales_available,
            salt_paid: self.salt_paid,
            salt_deducted: self.salt_deducted,
            itemized: self.itemized,
            standard: self.standard,
            overall_limit: self.overall_limit.clone(),
            property_refunded: 0,
            ordinary: self.ordinary,
        }
    }

    fn valid(&self) -> bool {
        (2010..=2025).contains(&self.payment_year)
            && (1..=100_000_000_000).contains(&self.payment_for_refunded_return)
            && self.payment_for_refunded_return <= self.income_paid
            && self.as_single_year().valid()
    }
}

/// (payment year, pro-rata refund allocation in cents, taxable recovery in cents) for each year.
pub(crate) type MultiYearRefundBreakdown = Vec<(i32, i64, i64)>;
/// Prior-year taxable shares and the current-year deduction reduction, all in cents.
pub(crate) type PriorCurrentRefundBreakdown = (MultiYearRefundBreakdown, i64);

fn taxable_prior_shares(
    years: &[TaxRefundYearPayment],
    shares: &[i64],
) -> Result<MultiYearRefundBreakdown, (&'static str, &'static str)> {
    years
        .iter()
        .zip(shares)
        .map(|(year, &share)| Ok((year.payment_year, share, year.as_single_year().taxable(share)?)))
        .collect()
}

fn pro_rata_refund(refund: i64, payments: &[i64]) -> Option<Vec<i64>> {
    let total = payments.iter().try_fold(0_i64, |sum, payment| sum.checked_add(*payment))?;
    if payments.is_empty()
        || total <= 0
        || refund < 0
        || refund > total
        || payments.iter().any(|payment| *payment <= 0)
    {
        return None;
    }
    // Publication 525's Recovery for 2 or more years allocates a single refund
    // pro rata by payments in each year. The cent-rounding convention here
    // assigns each cent once, with equal remainders going to earlier years.
    // https://www.irs.gov/publications/p525
    Some(crate::kernel::ratio::allocate_largest_nonnegative(refund, payments, total))
}

/// Federal deduction used for the refunded state income tax
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum TaxRefundHistory {
    /// Standard deduction, or no itemized deduction for this tax
    NotDeducted,
    /// Itemized general sales tax instead of state income tax
    SalesTax,
    /// Deducted on a business schedule (Form 1099-G box 8)
    /// A refund of a state or local tax that applies only to business income is income of that business, not Schedule 1 line 1.
    BusinessExpense {
        /// Business that deducted the tax
        /// The identity of one of this owner's businesses. Empty uses the owner's only business.
        #[libertas_number(min = 1)]
        #[libertas_default($null)]
        business: Option<i64>,
    },
    /// Itemized state income tax from one prior federal return
    IncomeTax {
        /// Federal deduction year, 2010–2025; combine refunds for the same year before entering them
        /// A 2013–2017 return with an overall itemized-deduction limit also needs its retained line 29 worksheet. Pre-2010 returns need historical review.
        // Tax-year structure: the recorded date/year cannot be later than this applicable tax-year window.
        #[libertas_number(min = 2010, max = 2025)]
        deduction_year: i32,
        /// State/local income tax paid and included in the prior Schedule A
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        income_paid: i64,
        /// General sales tax deduction available instead, from the retained prior-year worksheet
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        sales_available: i64,
        /// Total prior Schedule A deductible taxes before any SALT cap: 2010–2017 line 9, or 2018–2025 line 5d
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        salt_paid: i64,
        /// Taxes allowed after any SALT cap: 2010–2017 line 9, or 2018–2025 line 5e
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        salt_deducted: i64,
        /// Total prior itemized deductions: 2010–2017 Schedule A line 29, or 2018–2025 line 17
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        itemized: i64,
        /// Prior-year available standard deduction from retained return worksheets
        /// Include age/blind additions and filing-status restrictions; this is a prior-return fact, not the current year's standard deduction.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        standard: i64,
        /// Retained 2013–2017 overall itemized-deduction worksheet, when it applied
        #[libertas_default($null)]
        overall_limit: Option<PeaseWorksheet>,
        /// Real estate and personal property taxes refunded to you in 2026 for that year
        /// Publication 525 Worksheet 2a line 2. They are figured with the income tax refund against the same Schedule A taxes and the same limit, and any taxable part goes on Schedule 1 line 8z.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        property_refunded: i64,
        /// Ordinary prior-year refund worksheet conditions are met
        /// This is the combined refund of income taxes deducted in the stated year, with positive prior taxable income and tax, no unentered overall itemized limit, preferential-rate tax-benefit recomputation, unused credits, AMT, dependent-filer restriction, changed joint ownership, other itemized recoveries, or prior net-investment-income deduction for these taxes. No tax payments first deducted in 2026 are included. Use a separate review for exceptions. That year's federal return shows it. No leaves this refund out of the estimate with a notice; the full tax-benefit worksheet is not calculated.
        ordinary: bool,
    },
    /// One 2026 state/local income-tax refund from payments deducted in 2–8 distinct prior federal years
    /// Enter this ONE refund once. Publication 525 allocates it pro rata by payments in each year; the App applies each confirmed deduction year's own Schedule A tax-benefit worksheet.
    IncomeTaxMultiplePriorYears {
        /// Prior federal payment and deduction years, 2010–2025, earliest first
        /// ----
        /// One year's payments toward this same refunded state return, with its retained federal Schedule A facts
        #[libertas_size(min = 2)]
        years: Vec<TaxRefundYearPayment>,
        /// The state return, refund notice and payment ledger reconcile to exactly these 2–8 years
        /// Exclude interest, any 2026 payment, another refund from these deduction years, changed joint ownership, and payment/deduction timing that differs by year. Each share was deducted on this taxpayer's federal Schedule A in the year paid, with its own ordinary tax-benefit facts. The state refund notice and payment ledger show it. No leaves this refund out of the estimate with a notice; that situation is not supported.
        allocation_complete: bool,
    },
    /// One 2026 state/local income-tax refund from 1–7 prior payment years and 2026 payments
    /// Each prior share is a section 111 recovery. The one 2026 share reduces current state-income-tax payments, not income. If 2026 income tax is elected on Schedule A, enter the net payment in Deductions once; with the standard deduction or a general-sales-tax election, keep only the gross payment ledger here.
    IncomeTaxPriorYearsAndCurrent {
        /// Prior federal payment and deduction years, 2010–2025, earliest first
        /// ----
        /// One prior year's payments toward the same refunded state return and its retained Schedule A facts
        #[libertas_size(min = 1)]
        prior_years: Vec<TaxRefundYearPayment>,
        /// State/local income tax paid in 2026 toward this same refunded state return
        /// This is the 2026 pro-rata weight, not every state/local tax paid in 2026.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 1)]
        current_payment_for_refunded_return: i64,
        /// All personal 2026 state/local income taxes paid before this ONE refund
        /// Include the current payment above and all other personal state-income-tax payments. No other same-year refund, credit or offset may be netted here in this narrow case. If 2026 state-income-tax is elected, Deductions > Income tax > Paid must equal this amount less the allocated 2026 refund share. With the standard deduction or general-sales-tax election, retain this gross source ledger without entering an income-tax payment in Deductions.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 1)]
        gross_current_income_tax_paid: i64,
        /// The refund notice and state payment ledger identify exactly these prior years and 2026
        /// Exclude separately paid interest. Each prior payment was deducted in its own payment year on this taxpayer's federal Schedule A. The 2026 payment belongs to this taxpayer's eligible personal state-income-tax payments; the same refund and its allocated shares have not been entered elsewhere. Changed joint ownership, another same-year refund or payment/deduction in different years needs separate review. The state refund notice and payment ledger show it. No leaves this refund out of the estimate with a notice; that situation is not supported.
        allocation_complete: bool,
    },
}
impl TaxRefundHistory {
    // Answers and unknowns: a refund whose worksheet conditions or year
    // allocation are not confirmed is left out (unknowns::supported).
    pub(crate) fn supported(&self) -> bool {
        match self {
            Self::IncomeTax { ordinary, .. } => *ordinary,
            Self::IncomeTaxMultiplePriorYears { years, allocation_complete } => {
                *allocation_complete && years.iter().all(|year| year.ordinary)
            }
            Self::IncomeTaxPriorYearsAndCurrent { prior_years, allocation_complete, .. } => {
                *allocation_complete && prior_years.iter().all(|year| year.ordinary)
            }
            _ => true,
        }
    }
    pub(crate) fn deduction_years(&self) -> Vec<i32> {
        match self {
            Self::IncomeTax { deduction_year, .. } => alloc::vec![*deduction_year],
            Self::IncomeTaxMultiplePriorYears { years, .. } => {
                years.iter().map(|year| year.payment_year).collect()
            }
            Self::IncomeTaxPriorYearsAndCurrent { prior_years, .. } => {
                prior_years.iter().map(|year| year.payment_year).collect()
            }
            Self::NotDeducted | Self::SalesTax | Self::BusinessExpense { .. } => Vec::new(),
        }
    }

    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::NotDeducted | Self::SalesTax => true,
            Self::BusinessExpense { business } => business.is_none_or(|id| id >= 1),
            Self::IncomeTaxMultiplePriorYears { years, .. } => {
                (2..=8).contains(&years.len())
                    && years.iter().all(TaxRefundYearPayment::valid)
                    && years.windows(2).all(|pair| pair[0].payment_year < pair[1].payment_year)
                    && years
                        .iter()
                        .try_fold(0_i64, |sum, year| {
                            sum.checked_add(year.payment_for_refunded_return)
                        })
                        .is_some_and(|total| total <= 100_000_000_000)
            }
            Self::IncomeTaxPriorYearsAndCurrent {
                prior_years,
                current_payment_for_refunded_return,
                gross_current_income_tax_paid,
                ..
            } => {
                (1..=7).contains(&prior_years.len())
                    && prior_years.iter().all(TaxRefundYearPayment::valid)
                    && prior_years
                        .windows(2)
                        .all(|pair| pair[0].payment_year < pair[1].payment_year)
                    && (1..=100_000_000_000).contains(current_payment_for_refunded_return)
                    && (1..=100_000_000_000).contains(gross_current_income_tax_paid)
                    && current_payment_for_refunded_return <= gross_current_income_tax_paid
                    && prior_years
                        .iter()
                        .try_fold(*current_payment_for_refunded_return, |sum, year| {
                            sum.checked_add(year.payment_for_refunded_return)
                        })
                        .is_some_and(|total| total <= 100_000_000_000)
            }
            Self::IncomeTax {
                deduction_year,
                income_paid,
                sales_available,
                salt_paid,
                salt_deducted,
                itemized,
                standard,
                overall_limit,
                property_refunded,
                ..
            } => {
                (2010..=2025).contains(deduction_year)
                    && [
                        income_paid,
                        sales_available,
                        salt_paid,
                        salt_deducted,
                        itemized,
                        standard,
                        property_refunded,
                    ]
                    .iter()
                    .all(|n| (0..=100_000_000_000).contains(*n))
                    && *property_refunded <= salt_paid - income_paid
                    && income_paid <= salt_paid
                    && salt_deducted <= salt_paid
                    && salt_deducted <= itemized
                    && if *deduction_year <= 2017 {
                        salt_deducted == salt_paid
                            && overall_limit.as_ref().is_none_or(|limit| {
                                limit.valid(*deduction_year, *salt_paid, *itemized)
                            })
                    } else {
                        overall_limit.is_none()
                    }
            }
        }
    }
    pub(crate) fn taxable(&self, refund: i64) -> Result<i64, (&'static str, &'static str)> {
        match self {
            Self::NotDeducted | Self::SalesTax => Ok(0),
            // The refund is business income, not Schedule 1 line 1
            // (information_returns::route adds it to the business).
            Self::BusinessExpense { .. } => Ok(0),
            Self::IncomeTaxMultiplePriorYears { .. } => self
                .multi_year_breakdown(refund)?
                .map(|years| years.iter().map(|(_, _, taxable)| taxable).sum())
                .ok_or((
                    "ISSUE_REFUND_MULTI_YEAR_ALLOCATION_INCOMPLETE",
                    crate::fixture_english!("The multi-year refund allocation is incomplete."),
                )),
            Self::IncomeTaxPriorYearsAndCurrent { .. } => self
                .prior_current_breakdown(refund)?
                .map(|(years, _)| years.iter().map(|(_, _, taxable)| taxable).sum())
                .ok_or((
                    "ISSUE_REFUND_PRIOR_CURRENT_ALLOCATION_INCOMPLETE",
                    crate::fixture_english!(
                        "The prior/current-year refund allocation is incomplete."
                    ),
                )),
            Self::IncomeTax { itemized, standard, .. } => {
                let (income, property) = self.state_local_lost(refund)?.expect("income tax");
                Ok(phase::cap(income + property, [phase::floor(itemized - standard, 0)]))
            }
        }
    }

    // The year of a single-year income tax refund that is figured in
    // Worksheet 2 beside the same year's other recoveries.
    pub(crate) fn single_year(&self) -> Option<i32> {
        match self {
            Self::IncomeTax { deduction_year, ordinary: true, .. } => Some(*deduction_year),
            _ => None,
        }
    }

    // That year's Schedule A line 17 and standard deduction (Worksheet 2 lines 4
    // and 7).
    pub(crate) fn year_deductions(&self) -> Option<(i64, i64)> {
        match self {
            Self::IncomeTax { itemized, standard, .. } => Some((*itemized, *standard)),
            _ => None,
        }
    }

    // Worksheet 2 lines 1a and 1b for a single-year income tax refund: the
    // part of the income tax refund and of the real estate and personal
    // property tax refunded that lowered that year's tax, before the line 8
    // limit (Worksheet 2a). Refigure the taxes deducted with the refunds
    // removed, allowing the sales-tax alternative, and cap the loss at the
    // deduction originally used: the recovered amount cannot create a new
    // prior-year benefit. The loss is shared between lines 1a and 1b by the
    // refunds' sizes (Worksheet 2a lines 7 and 8). None for any other history.
    pub(crate) fn state_local_lost(
        &self,
        refund: i64,
    ) -> Result<Option<(i64, i64)>, (&'static str, &'static str)> {
        let Self::IncomeTax {
            income_paid,
            deduction_year,
            sales_available,
            salt_paid,
            salt_deducted,
            itemized,
            overall_limit,
            property_refunded,
            ordinary,
            ..
        } = self
        else {
            return Ok(None);
        };
        if !*ordinary {
            return Err((
                "ISSUE_REFUND_TAX_BENEFIT_WORKSHEET_NEEDED",
                crate::fixture_english!(
                    "This refund needs the Publication 525 tax-benefit recovery worksheet, including prior credit, AMT or other recovery coordination, and the recovery of a prior Form 8960 line 9b deduction on Form 8960 line 7."
                ),
            ));
        }
        if refund > *income_paid {
            return Err((
                "ISSUE_REFUND_EXCEEDS_PRIOR_TAX_PAYMENT",
                crate::fixture_english!(
                    "The refund exceeds the state/local income tax payment recorded for this deduction year. Separate other-year taxes, interest, credits or other recovery sources."
                ),
            ));
        }
        let other_taxes = salt_paid - income_paid;
        let revised_taxes = phase::floor(income_paid - refund, *sales_available)
            + (other_taxes - property_refunded);
        let lost_taxes = phase::floor(salt_deducted - revised_taxes, 0);
        let lost_deduction = overall_limit.as_ref().map_or(lost_taxes, |limit| {
            itemized - limit.itemized_after(*deduction_year, lost_taxes)
        });
        let refunded = refund + property_refunded;
        let income_share = if refunded == 0 {
            0
        } else {
            (crate::kernel::ratio::trunc_i128(
                i128::from(lost_deduction) * i128::from(refund),
                i128::from(refunded),
            )) as i64
        };
        Ok(Some((income_share, lost_deduction - income_share)))
    }

    pub(crate) fn multi_year_breakdown(
        &self,
        refund: i64,
    ) -> Result<Option<MultiYearRefundBreakdown>, (&'static str, &'static str)> {
        let Self::IncomeTaxMultiplePriorYears { years, allocation_complete } = self else {
            return Ok(None);
        };
        if !self.valid() || !*allocation_complete {
            return Err((
                "ISSUE_REFUND_PRIOR_YEARS_UNRECONCILED",
                crate::fixture_english!(
                    "Reconcile this one refund to 2–8 distinct prior federal payment and deduction years and their retained state and federal records."
                ),
            ));
        }
        let payments: Vec<_> = years.iter().map(|year| year.payment_for_refunded_return).collect();
        let shares = pro_rata_refund(refund, &payments)
            .ok_or(("ISSUE_REFUND_EXCEEDS_PRIOR_YEAR_PAYMENTS", crate::fixture_english!("The refund exceeds payments in the recorded prior years of the state return; separate interest or other sources.")))?;
        taxable_prior_shares(years, &shares).map(Some)
    }

    /// (prior allocated/taxable cents by year, 2026 deduction-reduction cents).
    pub(crate) fn prior_current_breakdown(
        &self,
        refund: i64,
    ) -> Result<Option<PriorCurrentRefundBreakdown>, (&'static str, &'static str)> {
        let Self::IncomeTaxPriorYearsAndCurrent {
            prior_years,
            current_payment_for_refunded_return,
            allocation_complete,
            ..
        } = self
        else {
            return Ok(None);
        };
        if !self.valid() || !*allocation_complete {
            return Err((
                "ISSUE_REFUND_PRIOR_CURRENT_UNRECONCILED",
                crate::fixture_english!(
                    "Reconcile this one refund, its prior federal Schedule A deductions, and its 2026 income-tax payment to the retained state and federal records."
                ),
            ));
        }
        let mut payments: Vec<_> =
            prior_years.iter().map(|year| year.payment_for_refunded_return).collect();
        payments.push(*current_payment_for_refunded_return);
        let shares = pro_rata_refund(refund, &payments)
            .ok_or(("ISSUE_REFUND_EXCEEDS_PRIOR_CURRENT_PAYMENTS", crate::fixture_english!("The refund exceeds payments in the recorded prior years and 2026 of the state return; separate interest or other sources.")))?;
        let prior = taxable_prior_shares(prior_years, &shares[..prior_years.len()])?;
        Ok(Some((prior, shares[prior_years.len()])))
    }
}
