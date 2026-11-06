// Coverage: Section 1244 stock loss and documented investment capital fallback (B7a, 2026-10-09). Section1244Stock (original-issue stock of a
//   small domestic corporation sold at a loss or worthless in 2026: the loss is ordinary up to
//   $50,000 a year, $100,000 on a joint return even if only one spouse has the loss, and the excess
//   is a capital loss; contributed property is limited to its value for the ordinary part and later
//   cash contributions are allocated to the other stock, Pub. 550 pp. 80-81; IRC 1244; limits in
//   year_constants.rs) gains its own Investment sales kind, SmallBusinessStockLoss, with client
//   rules and server twins for the dates and amounts. The ordinary part is Form 4797 Part II line
//   11 (2026 draft; line 10 before 2026), "Losses on Section 1244 (Small Business Stock)"; the
//   capital excess is Form 8949 with code S in box C or F (Schedule D line 3 or 10). A loss whose
//   qualification is No is a capital loss; No on complete worthlessness takes no 2026 loss.
//   Missing or unmodeled deductible history leaves only this loss out with
//   WARN_SMALL_BUSINESS_STOCK_LEFT_OUT. Section 1242 (SBIC stock) is not supported: the stored
//   InvestmentCompanyStock kind was deleted, because corporate small business investment companies
//   held by individuals have practically ceased to exist; such a sale takes SaleSpecial::Another.
//   Forms: Form 4797 line 11, 8949 (code S), Schedule D. No import (no IRS form reports the stock).
//   CapitalInvestment preserves adjusted basis/holding facts and uses capital character even with
//   issuer qualification. Missing basis/wash-sale/other history leaves only this loss unclaimed;
//   documented full section267 related-party disallowance computes zero, with no fabricated loss.
//   Tests: src/tests/investment/investment.rs section1244_*; src/tests/investment/investment_kinds.rs section_1244_*.
use crate::GainTerm;
use crate::kernel::phase;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// How the small-business stock loss occurred
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum StockLossEvent {
    /// Sold at a loss
    Sale {
        /// Sale trade date
        #[libertas_date_only]
        date: u32,
        /// Proceeds after selling expenses
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        proceeds: i64,
    },
    /// Became completely worthless in 2026
    Worthless {
        /// Records establish complete worthlessness this year
        /// A price decline, delisting or bankruptcy filing alone is insufficient. Retain the facts establishing total worthlessness and its year. No takes no loss for this stock in 2026.
        /// Check the issuer's liquidation, bankruptcy and collection records for this year.
        confirmed: bool,
    },
}

/// Documented stock basis history
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum StockLossBasis {
    /// Cash purchase, with any later cash contributions to this lot
    Cash {
        /// Cash paid for these shares; no unentered basis adjustment
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        cost: i64,
        /// Later cash capital contributions allocated to these same shares
        /// Enter zero if none. Contributions increase basis but do not qualify for section 1244 ordinary loss. A payment for newly issued shares is a separate lot, not an addition here.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        added: i64,
    },
    /// Investment property exchanged tax-free for the original shares
    /// One investment capital asset, other than stock or securities, transferred in a qualifying section 351 exchange solely for stock. Ordinary liabilities assumed by the corporation may reduce stock basis. No boot, tax-avoidance liability assumption, liabilities exceeding basis, later capital contributions, or other election/adjustment changing stock basis. Separate mixed-property exchanges require further allocation.
    Property {
        /// Property's adjusted tax basis immediately before the exchange
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        basis: i64,
        /// Property's fair market value at the exchange
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        value: i64,
        /// Property liabilities assumed by the corporation in the exchange
        /// Enter zero if none. Include only liabilities treated as assumed under section 357 that reduce stock basis under section 358; exclude liabilities whose payment would create a deduction. No section 357(b) tax-avoidance treatment or section 357(c) gain.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        liabilities: i64,
        /// Original acquisition date of the exchanged investment property
        /// Its holding period carries into the shares. Excludes property whose holding period requires other attribution or adjustments.
        #[libertas_date_only]
        acquired: u32,
    },
}
impl StockLossBasis {
    pub(crate) fn total(&self) -> i64 {
        match self {
            Self::Cash { cost, added } => cost + added,
            Self::Property { basis, liabilities, .. } => basis - liabilities,
        }
    }
    fn valid(&self, issued: u32) -> bool {
        let amount = |n: i64| (0..=100_000_000_000).contains(&n);
        match self {
            Self::Cash { cost, added } => amount(*cost) && amount(*added) && amount(self.total()),
            Self::Property { basis, value, liabilities, acquired } => {
                amount(*basis)
                    && amount(*value)
                    && amount(*liabilities)
                    && liabilities <= basis
                    && crate::federal_rules::valid_date(*acquired)
                    && *acquired <= issued
            }
        }
    }
}

/// Ownership and restrictions for this investment loss
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum StockLossOwnership {
    /// Original individual investment, with only the declared basis history
    /// No securities-dealing business, gift, inheritance, stock dividend, reorganization, wash sale, related-party restriction, AMT basis difference or other unentered adjustment. Apply the section 1244 issuer tests below.
    OriginalIndividual {
        /// Issuer records establish section 1244 qualification
        /// Domestic corporation with no more than $1 million of money and property received for stock, capital contributions and paid-in surplus when issued; stock issued for money or eligible property. More than half of gross receipts over the applicable preceding five-year or shorter existence period came from qualifying operating sources, or the statutory excess-deductions exception applies. Pre-July 19, 1984 stock must be common stock. Retain issuer records and any required issuance designation. No treats the loss as a capital loss.
        /// Check the issuer's issuance, capital and gross-receipts records.
        qualified: bool,
    },
    /// Investment capital asset with documented adjusted basis and holding period
    /// No unentered loss restriction or basis adjustment. Do not claim a section 1244 ordinary loss.
    CapitalInvestment,
    /// Adjusted basis records are missing; leave only this loss unclaimed
    MissingBasis,
    /// An unentered wash-sale restriction affects the loss; leave only this loss unclaimed
    WashSaleRestriction,
    /// The entire loss is disallowed under the related-party rules
    /// Choose this only when the documented relationship and transaction establish the entire section 267 disallowance; the current deduction is zero. An unresolved or partially disallowed loss uses Other unmodeled history.
    RelatedPartyRestriction,
    /// Another ownership or basis history needs a worksheet not entered here
    OtherUnmodeledHistory,
}
impl StockLossOwnership {
    pub(crate) fn supported(self) -> bool {
        matches!(
            self,
            Self::OriginalIndividual { .. }
                | Self::CapitalInvestment
                | Self::RelatedPartyRestriction
        )
    }
}

/// Stock investment loss and section 1244 ordinary-loss treatment
/// A documented investment stock loss sold or established worthless in 2026. Original qualifying small-business stock can produce up to $50,000 ordinary loss ($100,000 jointly); documented other investment stock uses capital treatment. Enter each lot once, without duplicating securities sales or worthless securities. An unentered basis or loss restriction leaves only this loss unclaimed.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct Section1244Stock {
    /// Investment and contribution records
    pub basis: StockLossBasis,
    /// Date these shares were acquired
    #[libertas_date_only]
    pub acquired: u32,
    /// Loss event
    pub event: StockLossEvent,
    /// Actual investment ownership and loss restrictions
    pub ownership: StockLossOwnership,
}
impl Section1244Stock {
    // The first relationship between this page's fields that fails, by its
    // rule's key, in the order of the client rules.

    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        if self.acquired > 20261231 {
            return Some(
                crate::validation::ValidationIssue::new("SECTION_1244_ISSUED_AFTER_RETURN_YEAR")
                    .at_field("acquired"),
            );
        }
        if let StockLossEvent::Sale { date, proceeds } = self.event {
            if date < self.acquired {
                return Some(
                    crate::validation::ValidationIssue::new("SECTION_1244_SOLD_BEFORE_ISSUED")
                        .at_field("date")
                        .in_field("event"),
                );
            }
            if !(20260101..=20261231).contains(&date) {
                return Some(
                    crate::validation::ValidationIssue::new("SECTION_1244_SALE_NOT_IN_RETURN_YEAR")
                        .at_field("date")
                        .in_field("event"),
                );
            }
            let basis = match &self.basis {
                StockLossBasis::Cash { cost, added } => i128::from(*cost) + i128::from(*added),
                StockLossBasis::Property { basis, liabilities, .. } => {
                    i128::from(*basis) - i128::from(*liabilities)
                }
            };
            if i128::from(proceeds) > basis {
                return Some(
                    crate::validation::ValidationIssue::new("SECTION_1244_PROCEEDS_ABOVE_BASIS")
                        .at_field("proceeds")
                        .in_field("event"),
                );
            }
        }
        if let StockLossBasis::Property { basis, liabilities, acquired, .. } = &self.basis {
            if liabilities > basis {
                return Some(
                    crate::validation::ValidationIssue::new("SECTION_1244_LIABILITIES_ABOVE_BASIS")
                        .at_field("liabilities")
                        .in_field("basis"),
                );
            }
            if *acquired > self.acquired {
                return Some(
                    crate::validation::ValidationIssue::new("SECTION_1244_PROPERTY_AFTER_ISSUE")
                        .at_field("acquired")
                        .in_field("basis"),
                );
            }
        }
        None
    }
    pub(crate) fn valid(&self) -> bool {
        let date = self.date();
        self.basis.valid(self.acquired)
            && crate::federal_rules::valid_date(self.acquired)
            && crate::federal_rules::valid_date(date)
            && date / 10000 == 2026
            && self.acquired <= date
            && self.proceeds() >= 0
            && self.proceeds() <= self.basis.total()
    }
    fn date(&self) -> u32 {
        match self.event {
            StockLossEvent::Sale { date, .. } => date,
            StockLossEvent::Worthless { .. } => {
                crate::year_constants::WORTHLESS_SECURITY_DEEMED_SALE_DATE
            }
        }
    }
    pub(crate) fn proceeds(&self) -> i64 {
        match self.event {
            StockLossEvent::Sale { proceeds, .. } => proceeds,
            StockLossEvent::Worthless { .. } => 0,
        }
    }
    pub(crate) fn loss(&self) -> i64 {
        if self.ownership == StockLossOwnership::RelatedPartyRestriction {
            return 0;
        }
        self.basis.total() - self.proceeds()
    }
    pub(crate) fn qualification(&self) -> Option<bool> {
        match self.ownership {
            StockLossOwnership::OriginalIndividual { qualified } => Some(qualified),
            _ => None,
        }
    }
    pub(crate) fn ordinary_loss(&self) -> i64 {
        // Without confirmed section 1244 qualification the loss is capital.
        if self.qualification() != Some(true) {
            return 0;
        }
        match self.basis {
            StockLossBasis::Cash { cost, .. } => {
                let total = self.basis.total();
                if total == 0 {
                    return 0;
                }
                // Round the ordinary portion to cents and assign the remainder
                // to capital loss, preserving the complete realized loss.
                (crate::kernel::ratio::half_offset_i128(
                    i128::from(self.loss()) * i128::from(cost),
                    i128::from(total),
                )) as i64
            }
            StockLossBasis::Property { basis, value, .. } => phase::floor(
                self.basis.total() - phase::floor(basis - value, 0) - self.proceeds(),
                0,
            ),
        }
    }
    pub(crate) fn term(&self) -> GainTerm {
        let acquired = match self.basis {
            StockLossBasis::Property { acquired, .. } => acquired,
            StockLossBasis::Cash { .. } => self.acquired,
        };
        if crate::dates::held_more_than_one_year(acquired, self.date()) {
            GainTerm::Long
        } else {
            GainTerm::Short
        }
    }
}

impl StockLossEvent {
    // An unconfirmed worthlessness event takes no 2026 loss.
    pub(crate) fn unconfirmed(&self) -> Option<bool> {
        match self {
            Self::Worthless { confirmed } if !*confirmed => Some(*confirmed),
            _ => None,
        }
    }
}
