use crate::common::Owner;
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};
/// Foreign-credit interest expense allocation
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum ForeignInterestAllocation {
    /// Apportion home mortgage interest by gross income
    GrossIncome,
    /// Allocate interest to U.S. income when foreign gross income is at most $5,000
    /// Optional small-foreign-income treatment. The App checks combined foreign gross income across categories; this is not a separate $5,000 allowance per source.
    SmallForeignIncome,
}

/// How you claim foreign taxes
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum ForeignTaxBasis {
    /// Paid
    /// Taxes count in the year you pay them. This is your choice unless you use the accrual method or chose accrued taxes on an earlier return.
    Paid,
    /// Accrued
    /// Taxes count in the year they relate to, paid or not. Once chosen you keep this choice in every later year, and you can first choose it only on a return filed on time. Tax on earlier years' income paid now belongs to those years' returns.
    Accrued,
}

/// Qualified dividend and capital gain adjustment
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum ForeignRateAdjustment {
    /// Use the adjustment exception when you qualify
    /// Foreign qualified dividends stay unadjusted when your ordinary taxable income stays within the 24% bracket and your foreign qualified dividends total less than $20,000. Otherwise the App adjusts them as required.
    Exception,
    /// Always adjust qualified dividends and capital gain distributions
    /// Applies the rate adjustments even when you qualify for the exception. This can give a larger credit when little of your foreign income is qualified dividends.
    Adjust,
}

/// AMT foreign-credit limitation method
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ForeignAmtLimitation {
    /// Normal AMT income and deductions
    /// No simplified limitation election is in force. An earlier election can be revoked only with IRS consent.
    Normal,
    /// Apply the simplified limitation election
    /// Uses regular-tax net foreign income for the AMT limit. This can increase AMT or unused credit. The election continues in later years and cannot be changed annually to choose the lower tax.
    Simplified {
        /// First year after 1997 for which you claimed an AMT foreign tax credit
        /// Enter 2026 only if this is your first such claim and you are making the election now. Otherwise use the first claim year from your retained returns.
        // Tax structure: this date or year belongs to the stated current or retained-return period.
        #[libertas_number(min = 1998, max = 2026)]
        first_claim_year: u32,
        /// The election was made in that first claim year and remains in force
        /// For a first claim in 2026, confirm that you elect this method. An election omitted in the first eligible year cannot be made later; no IRS-approved revocation or unresolved filing-unit change. The first claim year's Form 1116 (AMT) shows the election. No uses the normal AMT limitation instead.
        confirmed: bool,
    },
}

/// Foreign tax
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ForeignTaxDeducted {
    /// Payer and country
    #[libertas_size(min = 1)]
    pub label: String,
    /// Income it was paid on
    pub income: ForeignTaxIncome,
    /// Tax paid in 2026, in U.S. dollars
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub tax: i64,
    /// People who paid this investment or passive-income tax
    /// Current earned-income taxes keep the earner specified above.
    /// [NullText]
    /// The payer is not established
    /// [SetText]
    /// This investment or passive tax was paid by
    #[libertas_default($null)]
    pub owners: Option<crate::DeductionOwners>,
}

/// Income it was paid on
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum ForeignTaxIncome {
    /// Current-year pay for work or self-employment abroad
    /// The part on income excluded on Form 2555 is not deductible; the App takes it out for this person.
    Earned {
        /// Person who earned the income
        #[libertas_default(Taxpayer)]
        owner: Owner,
    },
    /// Interest, dividends, nonbusiness royalties or investment gains
    /// Income from property held for investment, included in net investment income and the Form 4952 investment-income pool.
    Investment,
    /// Passive business or rental income subject to net investment income tax
    /// These taxes reduce net investment income for Form 8960. Expenses used in passive activity income or loss do not enter the Form 4952 investment-expense pool.
    Passive,
    /// Other taxable income
    /// Includes earlier-year earnings taxed now. Exclude any tax on income excluded from U.S. tax in the year it was earned.
    Other,
}

/// Source classification for ordinary Form 1116 income and taxes
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ForeignTaxCategory {
    /// Employee pay
    Employment,
    /// Foreign income tax paid now on earlier employee pay
    /// Tax first paid in 2026 on employee pay reported on an earlier U.S. return, not credited, deducted or refunded before. It is general category tax with no 2026 income: enter zero income. Only when you claim foreign taxes when paid.
    PriorEmployeeWageTax {
        /// Year the foreign employee pay was included on a U.S. return
        // Tax structure: this date or year belongs to the stated current or retained-return period.
        #[libertas_number(min = 1900, max = 2025)]
        income_year: u32,
    },
    /// Foreign income tax paid now on earlier interest or ordinary dividends
    /// Tax first paid in 2026 on interest or nonqualified dividends reported on an earlier U.S. return, not credited, deducted or refunded before. Tax on zero passive income belongs to the general category: enter zero income. Only when you claim foreign taxes when paid.
    PriorPassiveTax {
        /// Year the passive income was included on a U.S. return
        // Tax structure: this date or year belongs to the stated current or retained-return period.
        #[libertas_number(min = 1900, max = 2025)]
        income_year: u32,
    },
    /// Interest, dividends, royalties or investment sales
    Passive {
        /// Foreign withholding treatment on the payment
        /// Use the rate on the payment record, not total foreign tax divided by net income. Keep payments with different withholding treatments in separate records. The App groups direct passive income across payers before testing high-tax treatment.
        withholding: ForeignWithholding,
        /// Qualified dividends included in this income
        /// The foreign-source part of Form 1099-DIV box 1b already counted in the income above. The App applies the qualified dividend rate adjustment, or the adjustment exception you choose below.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        qualified: i64,
        /// Foreign-source sales in this country
        /// Choose the investment sales, already entered under Investment sales, whose gain or loss is from this country: real property located there, or personal property sold while your tax home was abroad with at least 10% foreign tax paid on the gain. Your other sales are U.S. source.
        sales: Vec<i64>,
    },
    /// Business run abroad
    /// A Schedule C business whose services were performed wholly abroad. Its gross income and expenses come from the business itself: enter zero income here.
    Business {
        /// Business
        business: i64,
        /// The business keeps its own books and records
        /// A separate set of books of original entry and ledger accounts for this business, such as its own receipts and disbursements journal. Yes makes its income foreign branch category income (Form 1116 box b); No makes it general category income (box d).
        separate_books: bool,
    },
}

/// Foreign withholding on direct passive income
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ForeignWithholding {
    /// No foreign withholding tax
    /// Any separately assessed foreign income tax still belongs in the tax amount.
    None,
    /// Positive withholding rate below 15 percent
    Below15,
    /// Withholding rate of 15 percent or more
    AtLeast15,
}

/// Country and category tax record
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ForeignOrdinaryTax {
    /// Country and payer label
    #[libertas_size(min = 1)]
    pub label: String,
    /// Country code — Form 1116 line i
    /// The IRS two-letter code of the country or U.S. territory the income is from and the tax was paid to, for example FR for France or JA for Japan (IRS.gov/CountryCodes). For taxes a mutual fund passed through on Form 1099-DIV, enter RIC.
    // Country identifiers use two- or three-letter ISO country codes.
    #[libertas_size(min = 2, max = 3)]
    pub country: String,
    /// Whose income and tax
    /// On a joint return, choose the spouse who earned the income and paid the tax. Foreign earned income excluded on Form 2555 is taken out of that person's employment sources.
    #[libertas_default(Taxpayer)]
    pub owner: Owner,
    /// Category of the foreign-source income already entered
    pub category: ForeignTaxCategory,
    /// Current-year gross income in this country and category
    /// Already included once in Income: interest, dividends, royalties or employee pay. Enter zero for a prior-year tax paid in 2026 and for a business run abroad.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub income: i64,
    /// Eligible foreign income tax for 2026 in this country and category
    /// Claiming taxes when paid: the tax paid in 2026, at the exchange rate on the date paid or withheld. Claiming them when accrued: the tax for 2026, at the 2026 average rate (the payment-date rate when paid before 2026, more than 2 years after it, or in an inflationary currency). Only tax you legally owe: leave out amounts you can get refunded, for example under a treaty, and a contested tax until it is settled. For a prior-year choice, only tax first paid in 2026 on the earlier income.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub tax: i64,
}

impl ForeignAmtLimitation {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::Normal => true,
            Self::Simplified { first_claim_year, .. } => (1998..=2026).contains(first_claim_year),
        }
    }
}

impl ForeignWithholding {
    // The Form 1116 passive group (1.904-4(c)(3)) of a payment with this
    // withholding treatment and tax: 1 and 2 by rate, 3 for no tax, 4 for a
    // separately assessed tax with no withholding.
    pub(crate) fn group(&self, tax: i64) -> usize {
        match self {
            Self::AtLeast15 => 1,
            Self::Below15 => 2,
            Self::None if tax == 0 => 3,
            Self::None => 4,
        }
    }
}

impl ForeignOrdinaryTax {
    pub(crate) fn qualified(&self) -> i64 {
        self.category.qualified()
    }
    pub(crate) fn group(&self) -> usize {
        self.category.group(&self.country, self.tax)
    }
}

impl ForeignTaxCategory {
    pub(crate) fn qualified(&self) -> i64 {
        match self {
            Self::Passive { qualified, .. } => *qualified,
            _ => 0,
        }
    }
    // The Form 1116 grouping of a row (see `crate::form_1116::Group`): 0 general,
    // 1-4 the passive withholding groups, 5 foreign branch, 6 section 901(j)
    // (a row whose country is on the sanctioned list, whatever its category).
    pub(crate) fn group(&self, country: &str, tax: i64) -> usize {
        if crate::year_constants::is_section_901j_country(country) {
            return SANCTIONED;
        }
        match self {
            Self::Employment | Self::PriorEmployeeWageTax { .. } => 0,
            Self::PriorPassiveTax { .. } => 4,
            Self::Passive { withholding, .. } => withholding.group(tax),
            Self::Business { separate_books: true, .. } => BRANCH,
            Self::Business { .. } => 0,
        }
    }
}

// Form 1116 foreign-branch and section 901(j) income-group indices.
pub(crate) const BRANCH: usize = 5;
pub(crate) const SANCTIONED: usize = 6;
