use crate::dates::source_valid_date;
use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// One federally subsidized home disposition in the return
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MortgageSubsidyEntry {
    /// Name or address that identifies this home on screen
    #[libertas_size(min = 1)]
    pub label: String,
    /// Form 8828 treatment or explicit exemption
    pub disposition: MortgageSubsidyDisposition,
}
impl MortgageSubsidyEntry {
    pub(crate) fn valid(&self) -> bool {
        !self.label.trim().is_empty() && self.disposition.valid()
    }

    pub(crate) fn home_sale_income_id(&self) -> Option<i64> {
        match &self.disposition {
            MortgageSubsidyDisposition::Ordinary {
                transfer: MortgageSubsidyTransfer::Sale { home_sale_income_id, .. },
                ..
            } => Some(*home_sale_income_id),
            _ => None,
        }
    }
}

/// Form 8828 treatment of a federally subsidized home
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MortgageSubsidyDisposition {
    /// Direct spouse or former-spouse transfer with no recognized gain or loss under section 1041
    ExemptSection1041 {
        /// Recipient and divorce relationship
        recipient: MortgageSubsidySpouseRecipient,
        /// Date the interest in the subsidized home was transferred in 2026
        #[libertas_date_only]
        disposed_on: u32,
        /// This direct transfer qualifies under section 1041; no gain or loss was recognized or deducted
        /// The divorce decree and transfer deed show it. Transfers to a nonresident-alien spouse, certain trusts or third parties, and transfers not incident to divorce require separate review.
        qualifies_without_recognized_gain_or_loss: bool,
    },
    /// Only a qualified home improvement loan financed this home; no Form 8828 recapture
    ExemptHomeImprovementLoan {
        /// Date the interest in the home was disposed of in 2026
        #[libertas_date_only]
        disposed_on: u32,
        /// Original principal of the qualified home improvement loan, at most $15,000
        #[libertas_money("USD", 2)]
        // Tax structure: The Form 8828 qualified home-improvement-loan exception requires principal of at most $15,000.
        #[libertas_number(min = 1, max = 1500000)]
        original_principal: i64,
        /// This QMB-funded loan paid only for qualified livability or energy-efficiency improvements, and no other federally subsidized debt or MCC financed the home
        /// The housing agency's loan closing documents show it.
        qualified_sole_subsidy: bool,
    },
    /// Original federal mortgage subsidy predates 1991, with no later qualifying subsidized debt
    ExemptBefore1991 {
        /// Date the original federally subsidized mortgage loan was provided
        #[libertas_date_only]
        original_closed_on: u32,
        /// Date the home interest was disposed of in 2026
        #[libertas_date_only]
        disposed_on: u32,
        /// No newer QMB-funded loan or MCC became federally subsidized debt on this home after 1990
        /// The loan and refinance documents and any MCC issued for the home show it.
        no_later_subsidy: bool,
    },
    /// Home destroyed by a 2026 casualty and already replaced on its original site in time
    ExemptReplacedCasualty {
        /// Date the original home was destroyed in 2026
        #[libertas_date_only]
        destroyed_on: u32,
        /// Date replacement home on the original site became your main home
        #[libertas_date_only]
        replaced_on: u32,
        /// Replacement on the original site within the ordinary deadline and any separate casualty tax are confirmed
        /// Yes when the replacement home on the original site became your main home within the ordinary deadline and the casualty's own tax is settled. No (a pending replacement, a different site, a disaster extension or unentered insurance gain) needs the ordinary disposition's sale facts, which this choice does not record.
        qualified_replacement_and_casualty_tax_resolved: bool,
    },
    /// Ordinary 2026 sale or gift of a federally subsidized home
    Ordinary {
        /// Source of the federal mortgage subsidy
        subsidy: MortgageSubsidyKind,
        /// Sale or nonspouse gift
        transfer: MortgageSubsidyTransfer,
        /// Original loan and issuer notice
        facts: MortgageSubsidyFacts,
    },
    /// Disposition, original loan, or recapture records need review
    Unreviewed,
}

/// Recipient of a section 1041 federally subsidized-home transfer
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum MortgageSubsidySpouseRecipient {
    /// Your spouse at the time of transfer
    CurrentSpouse,
    /// Your former spouse in a transfer incident to divorce
    FormerSpouseIncidentToDivorce,
}

/// Federally subsidized financing subject to Form 8828
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum MortgageSubsidyKind {
    /// Qualified Mortgage Credit Certificate
    Mcc,
    /// Tax-exempt qualified mortgage bond loan
    Qmb,
}

/// Transfer used for Form 8828 line 9
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MortgageSubsidyTransfer {
    /// Sold this home
    /// Enter the sale itself under Investment sales, Sale of your main home; this home is matched to it.
    Sale {
        /// Matching home sale
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_default(0)]
        home_sale_income_id: i64,
        /// Gain share actually paid to a qualified subordinate mortgage-loan governmental lender, if any
        /// [NullText]
        /// No gain share was paid to a qualified subordinate mortgage-loan governmental lender
        /// [SetText]
        /// The gain share paid to the qualified governmental lender was
        #[libertas_default($null)]
        qsml_gain_share: Option<MortgageSubsidyQsmlGainShare>,
    },
    /// Pure gift of the home, other than to a spouse or ex-spouse incident to divorce
    GiftToOther {
        /// Whose interest in the subsidized home was given away
        /// Required when this joint return has Form 8828 recapture tax. This source fact does not change the joint tax.
        /// [NullText]
        /// This gift has no Form 8828 recapture tax on a joint return
        owners: Option<MortgageSubsidyGiftOwners>,
        /// Fair market value of your interest on the gift date, treated as Form 8828 sales price
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        fair_market_value: i64,
        /// Adjusted basis of your interest at the gift date
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        adjusted_basis: i64,
        /// No cash, other consideration, assumed mortgage or other debt relief to the donor
        /// The transfer deed and closing statement show it. A transfer for consideration or with liability relief may also create recognized income and needs a separate source treatment.
        no_consideration_or_debt_relief: bool,
    },
}
/// Owners of the interest given away in a federally subsidized home
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum MortgageSubsidyGiftOwners {
    /// Taxpayer's interest only
    Taxpayer,
    /// Spouse's interest only
    Spouse,
    /// Both spouses' interests
    Both,
}

impl MortgageSubsidyTransfer {
    pub(super) fn valid(&self) -> bool {
        match self {
            Self::Sale { qsml_gain_share, .. } => {
                qsml_gain_share.as_ref().is_none_or(MortgageSubsidyQsmlGainShare::valid)
            }
            Self::GiftToOther { fair_market_value, adjusted_basis, .. } => {
                (0..=100_000_000_000).contains(fair_market_value)
                    && (0..=100_000_000_000).contains(adjusted_basis)
            }
        }
    }
}

/// Form 8828 line 13 gain paid to a qualified subordinate mortgage-loan governmental lender
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MortgageSubsidyQsmlGainShare {
    /// Retained qualified subordinate mortgage-loan or grant agreement
    #[libertas_size(min = 1)]
    pub agreement_reference: String,
    /// Date the governmental lender was actually paid its share of sale gain
    #[libertas_date_only]
    pub paid_on: u32,
    /// Amount of this owner's sale gain actually paid to the governmental lender
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub paid_gain_share: i64,
    /// Qualified governmental lender, actual payment, and nonduplication in HomeSale expenses are confirmed
    /// HomeSale sale expenses must exclude this gain share; Form 8828 line 13 subtracts it separately. No leaves the share in the recapture gain.
    /// Check the governmental loan agreement, closing statement, payment record and sale-expense entry.
    pub qualified_and_distinct: bool,
}
impl MortgageSubsidyQsmlGainShare {
    pub(super) fn valid(&self) -> bool {
        !self.agreement_reference.trim().is_empty()
            && source_valid_date(self.paid_on)
            && (1..=100_000_000_000).contains(&self.paid_gain_share)
    }
}

/// When the original federally subsidized loan was fully repaid
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MortgageSubsidyPayoff {
    /// At disposition, or a qualifying reissued MCC continued the original loan until disposition
    AtDispositionOrReissued,
    /// Fully repaid earlier without a qualifying reissued MCC
    Earlier {
        /// Date the federally subsidized loan was fully repaid
        #[libertas_date_only]
        repaid_on: u32,
    },
    /// Loan payoff or reissue history requires review
    Unreviewed,
}
impl MortgageSubsidyDisposition {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::Ordinary { transfer, facts, .. } => transfer.valid() && facts.valid(),
            Self::ExemptSection1041 { disposed_on, .. } => {
                source_valid_date(*disposed_on) && (20260101..=20261231).contains(disposed_on)
            }
            Self::ExemptHomeImprovementLoan { disposed_on, original_principal, .. } => {
                source_valid_date(*disposed_on)
                    && (20260101..=20261231).contains(disposed_on)
                    && (1..=1_500_000).contains(original_principal)
            }
            Self::ExemptBefore1991 { original_closed_on, disposed_on, .. } => {
                source_valid_date(*original_closed_on)
                    && *original_closed_on < 19910101
                    && source_valid_date(*disposed_on)
                    && (20260101..=20261231).contains(disposed_on)
            }
            Self::ExemptReplacedCasualty { destroyed_on, replaced_on, .. } => {
                source_valid_date(*destroyed_on)
                    && (20260101..=20261231).contains(destroyed_on)
                    && source_valid_date(*replaced_on)
                    && *replaced_on >= *destroyed_on
                    && *replaced_on <= 20281231
            }
            _ => true,
        }
    }
}

/// Retained Form 8828 issuer notice and original subsidized-loan facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MortgageSubsidyFacts {
    /// Original federally subsidized mortgage closing date
    #[libertas_date_only]
    pub original_closed_on: u32,
    /// Date your interest in the home was sold or given away in 2026
    #[libertas_date_only]
    pub disposed_on: u32,
    /// Issuer or lender recapture notice retained with the original mortgage records
    #[libertas_size(min = 1)]
    pub notice_reference: String,
    /// Number of family members living with you at disposition, used to select the issuer table column
    #[libertas_number(min = 1)]
    pub family_size_at_disposition: u32,
    /// Adjusted qualifying income from the issuer table for that family size and holding period
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub adjusted_qualifying_income: i64,
    /// Your share of the highest federally subsidized mortgage principal
    /// Form 8828 line 19 is 6.25% of this amount. Enter only this return's ownership share, not the full principal when another taxpayer files separately.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub highest_subsidized_principal: i64,
    /// Holding-period share from the issuer's table for this disposition
    /// Use the row for the full and partial years held. When the loan was repaid before disposition without an eligible reissued MCC, the App applies the separate worksheet if repayment occurred within four years of the original closing.
    // Tax structure: the stated percentage or plan/certificate rate is bounded by its defined scale or statutory rate.
    #[libertas_number(min = 0, max = 100)]
    #[libertas_physical_unit("percent")]
    pub holding_percentage: u32,
    /// Original federally subsidized loan payoff or qualifying MCC reissue
    pub payoff: MortgageSubsidyPayoff,
    /// Ordinary transfer, notice, qualified subsidy type, loan, holding period and separate-owner share are confirmed
    /// The lender's or issuer's recapture notice given at closing and the loan documents show it. Choose MCC only for a qualified Mortgage Credit Certificate, or QMB only for a tax-exempt qualified mortgage bond loan. GiftToOther excludes spouse or ex-spouse transfers incident to divorce; casualty destruction and subordinate-loan gain sharing need separate treatment.
    pub confirmed: bool,
}
impl MortgageSubsidyFacts {
    pub(crate) fn valid(&self) -> bool {
        source_valid_date(self.original_closed_on)
            && source_valid_date(self.disposed_on)
            && self.original_closed_on > 19901231
            && (20260101..=20261231).contains(&self.disposed_on)
            && self.original_closed_on < self.disposed_on
            && !self.notice_reference.trim().is_empty()
            && (1..=100).contains(&self.family_size_at_disposition)
            && (0..=100_000_000_000).contains(&self.adjusted_qualifying_income)
            && (1..=100_000_000_000).contains(&self.highest_subsidized_principal)
            && self.holding_percentage <= 100
            && match self.payoff {
                MortgageSubsidyPayoff::Earlier { repaid_on } => {
                    source_valid_date(repaid_on)
                        && repaid_on > self.original_closed_on
                        && repaid_on < self.disposed_on
                }
                _ => true,
            }
    }
}
