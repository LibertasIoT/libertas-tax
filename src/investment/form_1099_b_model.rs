// form_1099_b items of src/federal/federal_model.rs, split out by tools/split-hubs.
use crate::validation::amounts;
use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

pub use crate::capital_sale_model::{
    BasisReporting, CapitalSaleAmt, FederalSale, GainTerm, SaleAdjustment, SaleAmt, SaleSpecial,
    SaleUse,
};

/// How the worthless security was acquired
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum SecurityAcquisition {
    /// Purchased
    Purchased {
        /// Acquisition trade date
        #[libertas_date_only]
        date: u32,
    },
    /// Inherited capital asset with ordinary inherited-property holding treatment
    /// Use documented inherited basis, not the decedent's purchase cost. Excludes special carryover-basis elections.
    Inherited,
}
impl SecurityAcquisition {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::Purchased { date } => {
                crate::dates::source_valid_date(*date)
                    && *date <= crate::year_constants::WORTHLESS_SECURITY_DEEMED_SALE_DATE
            }
            Self::Inherited => true,
        }
    }
    pub(crate) fn term(&self) -> GainTerm {
        match self {
            // Held more than a year to the deemed sale date (December 31, 2026).
            Self::Purchased { date }
                if *date + 10000 >= crate::year_constants::WORTHLESS_SECURITY_DEEMED_SALE_DATE =>
            {
                GainTerm::Short
            }
            _ => GainTerm::Long,
        }
    }
}

/// Security that became worthless
/// One lot of stock, a stock right or a bond held as an investment. Do not also enter it as a sale with zero proceeds or as a bad debt.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct WorthlessSecurity {
    /// Adjusted basis of the worthless lot
    /// Include prior basis adjustments; do not use its original cost again if basis changed. Enter separate lots with different acquisition histories.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub basis: i64,
    /// Acquisition history
    pub acquisition: SecurityAcquisition,
    /// This capital-asset security became completely worthless in 2026
    /// Retain facts establishing the year and total worthlessness. A decline in price, delisting or bankruptcy filing alone is not enough. Do not also enter a zero-proceeds sale or a nonbusiness bad debt for this lot. No takes no loss this year; you can amend within 7 years once worthlessness is established.
    /// Check issuer liquidation and bankruptcy records establishing complete worthlessness in this year.
    pub worthless: bool,
    /// Ordinary stock, stock right or qualifying bond held for investment
    /// The brokerage statement or certificate shows the security type and how you acquired it. Not dealer inventory, a partnership interest, digital asset, gifted or carryover-basis property, employee stock with unresolved basis, section 1244 ordinary-loss stock, or another special treatment; No leaves this loss out of this estimate.
    /// Check issuer liquidation and bankruptcy records establishing complete worthlessness in this year.
    pub ordinary: bool,
}
impl WorthlessSecurity {
    // The relationship between this page's fields that fails, by its rule's
    // key.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        match self.acquisition {
            SecurityAcquisition::Purchased { date }
                if date > crate::year_constants::WORTHLESS_SECURITY_DEEMED_SALE_DATE =>
            {
                Some(
                    crate::validation::ValidationIssue::new("WORTHLESS_ACQUIRED_AFTER_2026")
                        .at_field("date")
                        .in_field("acquisition"),
                )
            }
            _ => None,
        }
    }
}

/// Personal loan or debt that became worthless
/// One debt. Retain a statement of the debt, the debtor and your relationship, the due date, your collection efforts and why it is worthless.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct NonbusinessBadDebt {
    /// Source facts and unrecovered basis of this debt
    pub source: BadDebtSource,
    /// Debt held to earn investment income
    /// Use the lending agreement and purpose: Yes for a debt held to earn taxable interest or another investment return; No for a wholly personal claim, including a contractor deposit or former-home mortgage shortfall. An expectation of principal repayment alone does not establish an investment purpose. This answer controls the investment-interest worksheet, not whether a genuine worthless nonbusiness debt is a capital loss.
    /// Check the loan agreement, collection records and records of the investment it protected.
    pub investment: bool,
    /// Genuine enforceable nonbusiness debt, guarantee or deposit claim
    /// Not a gift, voluntary forgiveness, business debt or uncollectible unpaid service income. Retain the obligation, payment and collection records, especially for related borrowers or contractors. No takes no loss.
    /// Check the loan agreement, collection records and records of the investment it protected.
    pub bona_fide: bool,
    /// The debtor is a political party or election committee
    /// Check whether the debtor is registered with the FEC or a state election office. Include party committees and organizations that accept contributions or spend money to influence an election. Section 271 disallows their worthless debts.
    /// Check the loan agreement, collection records and records of the investment it protected.
    pub political_debtor: bool,
    /// The entire remaining debt or guaranteed payment became unrecoverable in 2026
    /// Reasonable collection efforts and the debtor's financial facts establish no prospect of repayment. A late payment or partial impairment alone does not qualify. Retain a statement of the debtor, relationship, amount, due date, collection efforts and why the loss belongs in 2026. No takes no loss this year; you can amend within 7 years.
    /// Check the loan agreement, collection records and records of the investment it protected.
    pub worthless: bool,
}
impl NonbusinessBadDebt {
    // The first relationship between this page's fields that fails, by its
    // rule's key, in the order of the client rules.
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        self.source_issue().map(|issue| issue.in_field("source")).or_else(|| {
            (self.source.personal() && self.investment).then(|| {
                crate::validation::ValidationIssue::new("BAD_DEBT_PERSONAL_CLAIM_NOT_INVESTMENT")
                    .at_field("investment")
            })
        })
    }
    pub(super) fn source_issue(&self) -> Option<crate::validation::ValidationIssue> {
        let issue = |key, field| crate::validation::ValidationIssue::new(key).at_field(field);
        match &self.source {
            BadDebtSource::CashLoan { advanced, recovered } if recovered > advanced => {
                Some(issue("BAD_DEBT_RECOVERED_ABOVE_ADVANCED", "recovered"))
            }
            BadDebtSource::PaidGuarantee { paid, reimbursed, .. } if reimbursed > paid => {
                Some(issue("BAD_DEBT_RECOVERED_ABOVE_ADVANCED", "reimbursed"))
            }
            BadDebtSource::ContractorDeposit { deposited, recovered, .. }
                if recovered > deposited =>
            {
                Some(issue("BAD_DEBT_RECOVERED_ABOVE_ADVANCED", "recovered"))
            }
            BadDebtSource::FormerHomeMortgage { paid, recovered, .. } if recovered > paid => {
                Some(issue("BAD_DEBT_RECOVERED_ABOVE_ADVANCED", "recovered"))
            }
            BadDebtSource::PurchasedNonsecurity {
                face,
                price_paid,
                acquisition_costs,
                principal_collected,
                acquired,
                ..
            } => {
                if principal_collected >= face {
                    Some(issue("BAD_DEBT_COLLECTED_FACE", "principal_collected"))
                } else if *acquired > 20261231 {
                    Some(issue("BAD_DEBT_PURCHASED_AFTER_2026", "acquired"))
                } else if i128::from(*principal_collected)
                    > i128::from(*price_paid) + i128::from(*acquisition_costs)
                {
                    Some(issue("BAD_DEBT_COLLECTED_ABOVE_COST", "principal_collected"))
                } else if *principal_collected != 0 && price_paid != face {
                    Some(issue("BAD_DEBT_COLLECTED_NEEDS_FACE_PRICE", "price_paid"))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
struct SaleValues {
    personal_loss: i64,
    market_discount: i64,
    raw_gain: i64,
    column_g: i64,
    amt_personal_loss: i64,
    amt_gain: i64,
}

impl FederalSale {
    // Keep source cents exact through the complete column calculation. Narrow
    // only the actual values that the native financial owners must carry.
    fn checked_values(&self) -> Option<SaleValues> {
        let proceeds = i128::from(self.proceeds);
        let basis = i128::from(self.basis_amount());
        let expenses = i128::from(self.selling_expenses());
        let premium = i128::from(self.option_premium());
        let personal = matches!(self.use_kind, SaleUse::Personal);
        let personal_loss = if personal { (basis - proceeds + expenses).max(0) } else { 0 };
        let market_discount =
            if self.nominee() || !matches!(self.use_kind, SaleUse::Investment { .. }) {
                0
            } else {
                (proceeds - basis).max(0).min(i128::from(self.market_discount))
            };
        let raw_gain = proceeds - basis + i128::from(self.wash_sale()) + personal_loss - expenses
            + premium
            - market_discount;
        let gain = if self.nominee() { 0 } else { raw_gain };
        let amt_basis = self.amt.different().map_or(basis, |value| i128::from(value.basis));
        let amt_personal_loss = if personal { (amt_basis - proceeds + expenses).max(0) } else { 0 };
        let amt_gain = match self.amt.different() {
            Some(value) if !self.nominee() => {
                proceeds - amt_basis + i128::from(value.wash_sale) + amt_personal_loss - expenses
                    + premium
                    - market_discount
            }
            _ => gain,
        };
        Some(SaleValues {
            personal_loss: personal_loss.try_into().ok()?,
            market_discount: market_discount.try_into().ok()?,
            raw_gain: raw_gain.try_into().ok()?,
            column_g: (gain - (proceeds - i128::from(self.column_e()))).try_into().ok()?,
            amt_personal_loss: amt_personal_loss.try_into().ok()?,
            amt_gain: amt_gain.try_into().ok()?,
        })
    }
    pub(crate) fn arithmetic_issue(&self) -> Option<&'static str> {
        self.checked_values().is_none().then_some("BROKER_SALE_ARITHMETIC_UNAVAILABLE")
    }

    // The explicit conservative basis choice counts an unestablished basis as
    // zero, with WARN_SALE_BASIS_ASSUMED_ZERO.
    pub(crate) fn basis_amount(&self) -> i64 {
        self.basis.unwrap_or(0)
    }
    pub(crate) fn wash_sale(&self) -> i64 {
        match self.use_kind {
            SaleUse::Investment { wash_sale } | SaleUse::WrittenOption { wash_sale } => wash_sale,
            SaleUse::Personal => 0,
        }
    }
    pub(crate) fn written_option(&self) -> bool {
        matches!(self.use_kind, SaleUse::WrittenOption { .. })
    }
    pub(crate) fn nominee(&self) -> bool {
        matches!(self.adjustment, SaleAdjustment::Nominee)
    }
    // Form 8949 code L, a positive column (g) amount: the loss on personal-use property after
    // any code E selling expenses, so expenses can never create a deductible loss (section
    // 165(c); Instructions for Form 8949, code L).
    pub(crate) fn personal_loss(&self) -> i64 {
        self.checked_values().expect("validated capital-sale arithmetic").personal_loss
    }
    pub(crate) fn selling_expenses(&self) -> i64 {
        match self.adjustment {
            SaleAdjustment::SellingExpenses { amount } => amount,
            _ => 0,
        }
    }
    // Code E premium of an exercised option the form's proceeds leave out: positive when
    // received (a written call), negative when paid (a bought put).
    pub(crate) fn option_premium(&self) -> i64 {
        match self.adjustment {
            SaleAdjustment::OptionPremium { amount } => amount,
            _ => 0,
        }
    }
    // Code D (section 1276(a)(1), (a)(4); the Form 8949 worksheet for accrued market
    // discount): the smaller of the gain before adjustments (not below zero) and the accrued
    // discount is interest, not capital gain. A nominee's sale is the owner's.
    pub(crate) fn market_discount_interest(&self) -> i64 {
        self.checked_values().expect("validated capital-sale arithmetic").market_discount
    }
    // The gain before the nominee adjustment backs it out: column (h) of every other row.
    pub(super) fn raw_gain(&self) -> i64 {
        self.checked_values().expect("validated capital-sale arithmetic").raw_gain
    }
    // Column (h): zero for a nominee's sale (code N), else the capital gain or loss.
    pub(crate) fn gain(&self) -> i64 {
        if self.nominee() { 0 } else { self.raw_gain() }
    }
    // Column (e): the basis the form reported when code B corrects it, else the basis.
    pub(crate) fn column_e(&self) -> i64 {
        match self.adjustment {
            SaleAdjustment::BasisCorrection { reported_basis } => reported_basis,
            _ => self.basis_amount(),
        }
    }
    // Column (g): what takes (d) less (e) to column (h).
    pub(crate) fn column_g(&self) -> i64 {
        self.checked_values().expect("validated capital-sale arithmetic").column_g
    }
    // Column (f): the Form 8949 codes of this lot in alphabetical order (T carries no amount).
    pub(crate) fn codes(&self) -> String {
        let mut codes = String::new();
        if matches!(self.adjustment, SaleAdjustment::BasisCorrection { .. }) {
            codes.push('B');
        }
        if self.collectible {
            codes.push('C');
        }
        if self.market_discount_interest() > 0 {
            codes.push('D');
        }
        if matches!(
            self.adjustment,
            SaleAdjustment::SellingExpenses { .. } | SaleAdjustment::OptionPremium { .. }
        ) {
            codes.push('E');
        }
        if self.personal_loss() > 0 {
            codes.push('L');
        }
        if self.several_sales {
            codes.push('M');
        }
        if self.nominee() {
            codes.push('N');
        }
        if matches!(self.adjustment, SaleAdjustment::HoldingPeriodCorrection) {
            codes.push('T');
        }
        if self.wash_sale() > 0 {
            codes.push('W');
        }
        codes
    }
    // Exception 1 of the Instructions for Form 8949: a lot whose basis was reported to the
    // IRS and that needs no adjustment, code or collectible treatment goes on Schedule D line
    // 1a or 8a with no Form 8949 row. A personal-use loss, a market discount, several sales,
    // an option premium and a nominee sale all need a row.
    pub(crate) fn summary(&self) -> bool {
        matches!(self.basis_reporting, BasisReporting::ReportedToIrs)
            && self.adjustment == SaleAdjustment::None
            && self.wash_sale() == 0
            && !self.collectible
            && self.personal_loss() == 0
            && self.market_discount == 0
            && !self.several_sales
    }
    // The Form 8949 box (A-L) of this lot.
    pub(crate) fn box_letter(&self) -> char {
        let index = match self.basis_reporting {
            BasisReporting::ReportedToIrs => 0,
            BasisReporting::NotReported => 1,
            BasisReporting::NoForm1099B => 2,
        };
        let letters = match (self.term, self.digital_asset) {
            (GainTerm::Short, false) => ['A', 'B', 'C'],
            (GainTerm::Short, true) => ['G', 'H', 'I'],
            (GainTerm::Long, false) => ['D', 'E', 'F'],
            (GainTerm::Long, true) => ['J', 'K', 'L'],
        };
        letters[index]
    }
    pub(crate) fn amt_personal_loss(&self) -> i64 {
        self.checked_values().expect("validated capital-sale arithmetic").amt_personal_loss
    }
    pub(crate) fn amt_term(&self) -> GainTerm {
        self.amt.different().and_then(|v| v.term).unwrap_or(self.term)
    }
    pub(crate) fn amt_gain(&self) -> i64 {
        self.checked_values().expect("validated capital-sale arithmetic").amt_gain
    }
}

pub(crate) fn validate_sale(v: &crate::FederalSale) -> Result<(), (&'static str, &'static str)> {
    validate_sale_with_mode(v, true)
}

pub(crate) fn validate_sale_with_mode(
    v: &crate::FederalSale,
    check_static_bounds: bool,
) -> Result<(), (&'static str, &'static str)> {
    if i128::from(v.wash_sale()) > (i128::from(v.basis_amount()) - i128::from(v.proceeds)).max(0) {
        return Err((
            "ISSUE_FEDERAL_RULES_A_DISALLOWED_WASH_SALE_LOSS_CANNOT_EXCEED_BD3C925B",
            crate::fixture_english!("A disallowed wash-sale loss cannot exceed this sale's loss."),
        ));
    }
    // Only an option you wrote can be closed at a loss reported as negative proceeds, and it is
    // short-term however long it was open (section 1234(b)(1)); an option-premium adjustment
    // belongs on the sale of the shares; market discount needs a basis and a bond (an investment).
    if v.proceeds < 0 && !v.written_option() {
        return Err((
            "SALE_NEGATIVE_PROCEEDS_WRITTEN_OPTION_ONLY",
            crate::fixture_english!("Only an option you wrote can have negative proceeds."),
        ));
    }
    if v.written_option() && v.term != GainTerm::Short {
        return Err((
            "SALE_WRITTEN_OPTION_SHORT_TERM",
            crate::fixture_english!("An option you wrote is short-term however long it was open."),
        ));
    }
    if matches!(v.adjustment, SaleAdjustment::OptionPremium { .. })
        && !matches!(v.use_kind, SaleUse::Investment { .. })
    {
        return Err((
            "SALE_OPTION_PREMIUM_ON_STOCK",
            crate::fixture_english!(
                "An option premium adjustment belongs on the sale of the shares, an investment."
            ),
        ));
    }
    if v.market_discount != 0 && v.basis.is_none() {
        return Err((
            "SALE_MARKET_DISCOUNT_NEEDS_BASIS",
            crate::fixture_english!("Enter the bond's basis to figure the market discount."),
        ));
    }
    if v.market_discount != 0 && !matches!(v.use_kind, SaleUse::Investment { .. }) {
        return Err((
            "SALE_MARKET_DISCOUNT_ON_A_BOND",
            crate::fixture_english!("Market discount applies to a bond held as an investment."),
        ));
    }
    if let Some(amt) = v.amt.different() {
        // The explicit conservative zero-basis answer cannot supply a different
        // AMT basis; this adjustment needs an established regular basis.
        if v.basis.is_none() {
            return Err((
                "SALE_AMT_BASIS_NEEDS_BASIS",
                crate::fixture_english!("Enter the sale's basis to use a different AMT basis."),
            ));
        }
        if check_static_bounds && !amounts(&[amt.basis, amt.wash_sale]) {
            return Err((
                "ISSUE_FEDERAL_RULES_AMT_SALE_AMOUNTS_MUST_BE_WITHIN_BBFF1988",
                crate::fixture_english!(
                    "AMT sale amounts must be within the supported nonnegative money range."
                ),
            ));
        }
        if i128::from(amt.wash_sale) > (i128::from(amt.basis) - i128::from(v.proceeds)).max(0)
            || matches!(v.use_kind, SaleUse::Personal) && amt.wash_sale != 0
        {
            return Err((
                "ISSUE_FEDERAL_RULES_AMT_WASH_SALE_LOSS_MUST_NOT_EXCEED_B24A0E6B",
                crate::fixture_english!(
                    "AMT wash-sale loss must not exceed the AMT investment loss; personal property has no wash-sale adjustment."
                ),
            ));
        }
    }
    if !check_static_bounds {
        return Ok(());
    }
    let adjustment_amount = match v.adjustment {
        SaleAdjustment::BasisCorrection { reported_basis } => reported_basis,
        SaleAdjustment::SellingExpenses { amount } => amount,
        SaleAdjustment::OptionPremium { amount } => amount.abs(),
        SaleAdjustment::None
        | SaleAdjustment::HoldingPeriodCorrection
        | SaleAdjustment::Nominee => 0,
    };
    if amounts(&[
        v.proceeds.abs(),
        v.basis_amount(),
        v.wash_sale(),
        v.withholding,
        v.market_discount,
        adjustment_amount,
    ]) {
        Ok(())
    } else {
        Err((
            "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
            crate::fixture_english!(
                "Review the document amounts; they must be within the supported input range."
            ),
        ))
    }
}

/// Source of basis for a nonbusiness bad-debt claim
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum BadDebtSource {
    /// Cash advanced directly to the borrower
    CashLoan {
        /// Cash principal actually advanced
        /// Exclude unpaid interest, wages, rent and other income never included in taxable income.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        advanced: i64,
        /// Principal collected before complete worthlessness
        /// Include collateral proceeds applied to principal; exclude interest. Do not count a possible future recovery as a payment.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        recovered: i64,
    },
    /// Cash paid to a creditor under a genuine loan guarantee
    PaidGuarantee {
        /// Cash paid to the guaranteed creditor
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        paid: i64,
        /// Reimbursement or collateral recovered before complete worthlessness
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        reimbursed: i64,
        /// Calendar year the guarantor paid the creditor
        // Tax structure: this date or year belongs to the stated current or retained-return period.
        #[libertas_number(min = 1900, max = 2026)]
        paid_year: u32,
        /// Why the guarantee was made
        motive: GuaranteeMotive,
        /// Whether an enforceable claim against the borrower survives payment
        recovery_right: GuaranteeRight,
    },
    /// Unrecoverable deposit with an insolvent contractor for personal work
    ContractorDeposit {
        /// Cash deposited with the contractor
        /// Include only an enforceable repayment claim, not a gift or a charge for services already received.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        deposited: i64,
        /// Value recovered before the claim became worthless
        /// Include cash refunds and the value of property or contracted work actually delivered; only the unreturned deposit can be a loss.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        recovered: i64,
        /// The contractor became insolvent and the remaining deposit cannot be collected
        /// No takes no loss this year.
        /// Check the contractor's insolvency or bankruptcy records and your collection file.
        contractor_insolvent: bool,
        /// The contracted work was solely for personal use
        /// The contract shows what the work was for. Business or investment work needs separate expense, basis and bad-debt treatment; No leaves this loss out of this estimate.
        personal_work: bool,
    },
    /// Payment on a former home's mortgage after the buyer defaults
    /// The buyer assumed the mortgage, but you remained secondarily liable. A resulting shortfall you paid and cannot collect from the buyer can be a nonbusiness bad debt; do not duplicate a home-sale basis or proceeds adjustment.
    FormerHomeMortgage {
        /// Cash you paid to satisfy the buyer's mortgage shortfall
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        paid: i64,
        /// Cash or property recovered from the buyer before complete worthlessness
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        recovered: i64,
        /// The buyer assumed the mortgage when purchasing your former main home
        /// The sale's closing statement or assumption agreement shows it.
        buyer_assumed: bool,
        /// You remained legally secondarily liable on that mortgage
        /// The assumption agreement shows it, or the lender never released you.
        secondary_liability: bool,
        /// The buyer defaulted and the home was sold for less than the outstanding mortgage
        /// The lender's foreclosure or deficiency notice shows it.
        foreclosure_shortfall: bool,
        /// The payment was not also used to reduce your home-sale proceeds or claimed elsewhere
        /// Check the home-sale entry and earlier returns.
        no_duplicate_basis_or_loss: bool,
    },
    /// Purchased nonsecurity debt with cost basis and ordinary at-par collections
    /// Use a private debt, not a corporate or government bond, debenture or note with interest coupons or in registered form. The debt must have had value when purchased. Partial principal collections are supported only when purchase price equals principal face amount; discount and other adjusted-basis histories need separate treatment.
    PurchasedNonsecurity {
        /// Enforceable principal face amount of the debt acquired
        /// Exclude interest, fees and other claims from the principal amount.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        face: i64,
        /// Cash purchase price actually paid for this debt
        /// Do not enter the debt's face amount as the price unless that is what you paid.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        price_paid: i64,
        /// Direct acquisition costs paid
        /// Include documented fees attributable to buying this debt, not collection or financing costs.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        acquisition_costs: i64,
        /// Principal collected after purchase and before complete worthlessness
        /// Include collateral proceeds applied to principal. Enter zero if none. A purchased debt with any such collection is supported only when bought at its principal face amount; discounted-debt collections need market-discount and basis-allocation treatment.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        principal_collected: i64,
        /// Date the debt was purchased
        #[libertas_date_only]
        acquired: u32,
        /// The debt still had a real prospect of collection when purchased
        /// No takes no loss.
        /// Check the debt purchase file, borrower financial records and collection prospects at acquisition.
        had_value_when_bought: bool,
        /// The debt is evidenced by a section 165(g) security
        /// The instrument itself shows it. Yes for a corporate or government bond, debenture, note or other debt evidence with interest coupons or in registered form (a CUSIP on a statement). This is a worthless-security case, not a nonbusiness bad debt, so Yes leaves this loss out of this estimate.
        security: bool,
        /// Other basis-changing events after purchase
        /// Forms 1099-OID or 1099-INT and earlier returns show them. Yes includes taxable original-issue or market-discount accruals, amortized premium, or any basis change other than the principal collected above. Ordinary stated interest received without a basis change does not count. These histories need separate treatment, so Yes leaves this loss out of this estimate.
        basis_adjustments: bool,
    },
}

/// Reason for guaranteeing another person's loan
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum GuaranteeMotive {
    /// Protect an existing investment
    ProtectInvestment,
    /// Enter the guarantee to earn a profit
    Profit,
    /// Personal favor without an investment or profit purpose
    Favor,
    /// Another or unresolved purpose
    // An unresolved legal character cannot be reconstructed from payment amounts.
    // WARN_BAD_DEBT_NOT_DEDUCTED omits the loss until the guarantee's purpose is established.
    Other,
}

/// Recovery rights after paying a loan guarantee
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum GuaranteeRight {
    /// No subrogation or other right to demand repayment from the borrower
    None,
    /// An enforceable claim against the borrower became wholly worthless in 2026
    WhollyWorthless,
    /// The claim still has potential value
    Recoverable,
    /// Recovery rights or worthlessness await a legal determination
    // Pending recovery/enforceability cannot be looked up as a settled source fact.
    // WARN_BAD_DEBT_NOT_DEDUCTED omits the loss until that determination exists.
    NotSure,
}

impl BadDebtSource {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::CashLoan { advanced, recovered } => {
                (1..=100_000_000_000).contains(advanced) && (0..=*advanced).contains(recovered)
            }
            Self::PaidGuarantee { paid, reimbursed, paid_year, .. } => {
                (1..=100_000_000_000).contains(paid)
                    && (0..=*paid).contains(reimbursed)
                    && (1900..=2026).contains(paid_year)
            }
            Self::ContractorDeposit { deposited, recovered, .. } => {
                (1..=100_000_000_000).contains(deposited) && (0..=*deposited).contains(recovered)
            }
            Self::FormerHomeMortgage { paid, recovered, .. } => {
                (1..=100_000_000_000).contains(paid) && (0..=*paid).contains(recovered)
            }
            Self::PurchasedNonsecurity {
                face,
                price_paid,
                acquisition_costs,
                principal_collected,
                acquired,
                ..
            } => {
                (1..=100_000_000_000).contains(face)
                    && (1..=100_000_000_000).contains(price_paid)
                    && (0..=100_000_000_000).contains(acquisition_costs)
                    && (0..=*face).contains(principal_collected)
                    && principal_collected < face
                    && price_paid + acquisition_costs <= 100_000_000_000
                    && *principal_collected <= *price_paid + *acquisition_costs
                    && (*principal_collected == 0 || price_paid == face)
                    && (19000101..=20261231).contains(acquired)
                    && crate::dates::source_valid_date(*acquired)
            }
        }
    }

    // Business or investment work, a security, or a purchased debt with
    // other basis changes needs another treatment: the loss is left out.
    pub(crate) fn supported(&self) -> bool {
        match self {
            Self::ContractorDeposit { personal_work, .. } => *personal_work,
            Self::PurchasedNonsecurity { security, basis_adjustments, .. } => {
                !security && !basis_adjustments
            }
            _ => true,
        }
    }

    // Source facts: a failed required fact means no loss; a pending legal
    // guarantee determination remains unavailable under WARN_BAD_DEBT_NOT_DEDUCTED.
    pub(crate) fn facts(&self) -> Option<bool> {
        match self {
            Self::ContractorDeposit { contractor_insolvent, .. } => Some(*contractor_insolvent),
            Self::FormerHomeMortgage {
                buyer_assumed,
                secondary_liability,
                foreclosure_shortfall,
                no_duplicate_basis_or_loss,
                ..
            } => Some(
                *buyer_assumed
                    && *secondary_liability
                    && *foreclosure_shortfall
                    && *no_duplicate_basis_or_loss,
            ),
            Self::PurchasedNonsecurity { had_value_when_bought, .. } => {
                Some(*had_value_when_bought)
            }
            // The guarantee's terms: a personal favor, a claim against the
            // borrower that can still be collected, or a payment outside the
            // year the claim became worthless means no loss; an unresolved
            // purpose or claim is assumed to mean the same with a note.
            Self::PaidGuarantee { paid_year, motive, recovery_right, .. } => {
                let fails = *motive == GuaranteeMotive::Favor
                    || match recovery_right {
                        GuaranteeRight::None => *paid_year != 2026,
                        GuaranteeRight::WhollyWorthless => *paid_year > 2026,
                        GuaranteeRight::Recoverable => true,
                        GuaranteeRight::NotSure => false,
                    };
                if fails {
                    Some(false)
                } else if *motive == GuaranteeMotive::Other
                    || *recovery_right == GuaranteeRight::NotSure
                {
                    None
                } else {
                    Some(true)
                }
            }
            _ => Some(true),
        }
    }

    // A contractor deposit or former-home shortfall is a personal claim.
    pub(crate) fn personal(&self) -> bool {
        matches!(self, Self::ContractorDeposit { .. } | Self::FormerHomeMortgage { .. })
    }

    pub(crate) fn unrecovered_basis(&self) -> i64 {
        match self {
            Self::CashLoan { advanced, recovered } => advanced - recovered,
            Self::PaidGuarantee { paid, reimbursed, .. } => paid - reimbursed,
            Self::ContractorDeposit { deposited, recovered, .. } => deposited - recovered,
            Self::FormerHomeMortgage { paid, recovered, .. } => paid - recovered,
            Self::PurchasedNonsecurity {
                price_paid,
                acquisition_costs,
                principal_collected,
                ..
            } => price_paid + acquisition_costs - principal_collected,
        }
    }
}

/// Sale from your own records
/// An investment or personal sale, including a digital asset or an option you bought or wrote. Enter available Forms 1099-B and 1099-DA in their own Document center entries; this worksheet also supports a sale with no broker form. An exercised option has no entry of its own; section 1256 options use their separate worksheet.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Sale1099B {
    /// PAYER'S name, TIN and address
    pub payer: crate::PrintedParty,
    /// RECIPIENT'S name, TIN and address
    pub recipient: crate::PrintedParty,
    /// Description (Form 1099-B box 1a, Form 1099-DA boxes 1b and 1c)
    /// Needed only for a printed Form 8949. For a digital asset, its name and the units sold.
    #[libertas_default("")]
    pub description: String,
    /// A digital asset reported on Form 1099-DA, or sold with no Form 1099-DA or 1099-B
    /// Cryptocurrency, stablecoins and NFTs. Leave unchecked for a tokenized security reported on a Form 1099-B. This selects Form 8949 box G to L instead of A to F.
    #[libertas_default(false)]
    pub digital_asset: bool,
    /// Short-term or long-term (Form 1099-B box 2, Form 1099-DA box 6)
    /// An option you wrote is always short-term. Inherited property is long-term. Replacement shares from a wash sale include the sold shares' holding period. A short sale takes the holding period of the shares you delivered, unless you held substantially identical shares one year or less when you sold short.
    /// [NullText]
    /// The holding period cannot be established
    /// [SetText]
    /// The established holding period is
    pub term: Option<GainTerm>,
    /// Proceeds (Form 1099-B box 1d, Form 1099-DA box 1f)
    /// Net of selling fees, as the form reports them. For an option you wrote, the premium you received, or the net the form shows (a loss as negative).
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub proceeds: i64,
    /// Cost or other basis (Form 1099-B box 1e, Form 1099-DA box 1g)
    /// For an option you bought, what you paid; for one you wrote, what you paid to buy it back (zero if it expired). For a digital asset, its cost with fees in the same wallet or account. Blank on the form? Enter the cost from your records. Choose “Basis cannot be established; assume zero basis” only if you cannot establish it: the whole proceeds are then counted as gain.
    /// [NullText]
    /// Basis cannot be established; assume zero basis
    /// [SetText]
    /// My established cost or other basis is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub basis: Option<i64>,
    /// Basis reported to the IRS
    #[libertas_default(ReportedToIrs)]
    pub reporting: BasisReporting,
    /// What you sold, and any wash sale loss disallowed
    #[libertas_default(Investment)]
    pub use_kind: SaleUse,
    /// Box 3 collectibles
    #[libertas_default(false)]
    pub collectible: bool,
    /// Accrued market discount (Form 1099-B box 1f, Form 1099-DA box 1h)
    /// On a bond bought for less than its redemption price. The gain up to this amount is interest. Enter zero if you include market discount in income each year, and add those amounts to the basis instead.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub market_discount: i64,
    /// This entry totals several sales (Form 1099-DA box 11a)
    /// The broker reported a year of sales of one stablecoin or NFT as one amount, or you enter one total for several sales from your own statement that share a Form 8949 box (code M).
    #[libertas_default(false)]
    pub several_sales: bool,
    /// Box 4 federal income tax withheld
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// Payer and state boxes 14–16
    pub state_details: crate::StateWithholdingFields,
}

/// Accepted 1099-B or 1099-DA sale
/// The printed form and the topic answers it calls for, kept apart.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct SaleSource {
    /// The form as printed
    pub form: Sale1099B,
    /// Correction to this sale
    #[libertas_default(None)]
    pub adjustment: SaleAdjustment,
    /// A treatment this page does not calculate
    /// Stock compensation not yet in your basis, business or home property, or another treatment such as a contingent payment debt instrument, a rollover of gain or the DC Zone exclusion. Any choice but None of these leaves this sale out of the estimate with a notice. Ordinary employee shares, ESPP shares, ISO shares and NUA shares have their own entries on this page.
    #[libertas_default(NoneOfThese)]
    pub special: SaleSpecial,
    /// AMT sale facts
    #[libertas_default(SameAsRegular)]
    pub amt: SaleAmt,
}

impl SaleSource {
    /// A newly entered form: no topic answer is given yet.
    pub(crate) fn printed(form: Sale1099B) -> Self {
        Self {
            form,
            adjustment: SaleAdjustment::None,
            special: SaleSpecial::NoneOfThese,
            amt: SaleAmt::SameAsRegular,
        }
    }
    /// The topic answers of an accepted source survive a correction of its paper.
    pub(crate) fn keep_answers(&mut self, accepted: &Self) {
        self.adjustment = accepted.adjustment.clone();
        self.special = accepted.special;
        self.amt = accepted.amt.clone();
    }
    /// A sale known only by its calculation facts, such as an installment sale's full sale.
    pub(crate) fn from_sale(data: &FederalSale, description: &str) -> Self {
        Self {
            form: Sale1099B {
                payer: Default::default(),
                recipient: Default::default(),
                state_details: Default::default(),
                description: description.into(),
                term: Some(data.term),
                proceeds: data.proceeds,
                basis: data.basis,
                reporting: data.basis_reporting,
                use_kind: data.use_kind.clone(),
                collectible: data.collectible,
                withholding: data.withholding,
                digital_asset: data.digital_asset,
                market_discount: data.market_discount,
                several_sales: data.several_sales,
            },
            adjustment: data.adjustment.clone(),
            special: data.special,
            amt: data.amt.clone(),
        }
    }
}

// One-way: every printed box either reaches the internal model or is named as
// not read by any calculation, so a new box cannot be forgotten.
impl From<&SaleSource> for FederalSale {
    fn from(source: &SaleSource) -> Self {
        let SaleSource { form, adjustment, special, amt } = source;
        let Sale1099B {
            // The parties and the description name and assign the document; no figure reads them.
            payer: _,
            recipient: _,
            description: _,
            digital_asset,
            term,
            proceeds,
            basis,
            reporting,
            use_kind,
            collectible,
            market_discount,
            several_sales,
            withholding,
            // State boxes reach the state return through the accepted entry.
            state_details: _,
        } = form;
        FederalSale {
            // The save holds the choice present (SALE_TERM_NEEDED); the conservative term otherwise.
            term: term.unwrap_or(GainTerm::Short),
            collectible: *collectible,
            proceeds: *proceeds,
            basis: *basis,
            amt: amt.clone(),
            withholding: *withholding,
            use_kind: use_kind.clone(),
            special: *special,
            adjustment: adjustment.clone(),
            basis_reporting: *reporting,
            digital_asset: *digital_asset,
            market_discount: *market_discount,
            several_sales: *several_sales,
        }
    }
}

/// Futures and section 1256 options from your records
/// The annual contract total and your own treatment when no Form 1099-B is available. Enter an available printed Form 1099-B in its own Document center entry.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Contracts1099B {
    /// PAYER'S name, TIN and address
    pub payer: crate::PrintedParty,
    /// RECIPIENT'S name, TIN and address
    pub recipient: crate::PrintedParty,
    /// Box 11 aggregate profit or (loss) on contracts
    /// Enter a loss as negative. No Form 1099-B? Enter the year's total from your statements: profit or loss on contracts closed in 2026, less last year-end's unrealized amount, plus the unrealized amount on contracts open at the end of 2026.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub aggregate: i64,
    /// Box 4 federal income tax withheld
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// Payer and state boxes 14–16
    pub state_details: crate::StateWithholdingFields,
}

/// Accepted 1099-B futures and section 1256 options
/// The printed form and the topic answer it calls for, kept apart.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ContractsSource {
    /// The form as printed
    pub form: Contracts1099B,
    /// A business or farm hedge, a section 475(f) or 988 election, or you deal in these contracts
    /// Yes leaves this statement out of the estimate with a notice; that treatment is not supported. Part of a straddle with stock or other positions? Choose Straddles under Situations people often miss.
    #[libertas_default(false)]
    pub special: bool,
}

impl ContractsSource {
    /// A newly entered form: no topic answer is given yet.
    pub(crate) fn printed(form: Contracts1099B) -> Self {
        Self { form, special: false }
    }
    /// The topic answer of an accepted source survives a correction of its paper.
    pub(crate) fn keep_answers(&mut self, accepted: &Self) {
        self.special = accepted.special;
    }
}

// One-way: every printed box either reaches the internal model or is named as
// not read by any calculation, so a new box cannot be forgotten.
impl From<&ContractsSource> for FederalSection1256 {
    fn from(source: &ContractsSource) -> Self {
        let ContractsSource { form, special } = source;
        let Contracts1099B {
            // The parties name and assign the document; no figure reads them.
            payer: _,
            recipient: _,
            aggregate,
            withholding,
            // State boxes reach the state return through the accepted entry.
            state_details: _,
        } = form;
        FederalSection1256 {
            aggregate: *aggregate,
            withholding: *withholding,
            special_treatment: *special,
        }
    }
}

/// Section 1256 contracts on one Form 1099-B
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalSection1256 {
    /// Form 1099-B box 11 aggregate profit or (loss)
    /// A loss is negative. Form 6781 line 1.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub aggregate: i64,
    /// Form 1099-B box 4 federal income tax withheld
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub withholding: i64,
    /// A business or farm hedge, a section 475(f) or 988 election, or a dealer's contracts
    /// Yes leaves this statement out of the estimate with a notice; that treatment is not supported.
    pub special_treatment: bool,
}

impl FederalSection1256 {
    pub(crate) fn valid(&self) -> bool {
        (-100000000000..=100000000000).contains(&self.aggregate)
            && (0..=100000000000).contains(&self.withholding)
    }
}
