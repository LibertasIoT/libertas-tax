#[allow(unused_imports)]
use super::*;
pub use crate::mortgage_certificate_source::{
    FederalMortgageCertificate, FederalMortgageCredit, MortgageCertificateHistory,
};
pub use crate::mortgage_subsidy_source::{
    MortgageSubsidyDisposition, MortgageSubsidyEntry, MortgageSubsidyFacts,
    MortgageSubsidyGiftOwners, MortgageSubsidyKind, MortgageSubsidyPayoff,
    MortgageSubsidyQsmlGainShare, MortgageSubsidySpouseRecipient, MortgageSubsidyTransfer,
};

/// One federally subsidized home disposition in the return
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MortgageSubsidyEntryView {
    /// Name or address that identifies this home on screen
    #[libertas_size(min = 1)]
    pub label: String,
    /// Form 8828 treatment or explicit exemption
    pub disposition: MortgageSubsidyDispositionView,
}
impl From<MortgageSubsidyEntry> for MortgageSubsidyEntryView {
    fn from(value: MortgageSubsidyEntry) -> Self {
        Self { label: value.label, disposition: value.disposition.into() }
    }
}
impl From<MortgageSubsidyEntryView> for MortgageSubsidyEntry {
    fn from(value: MortgageSubsidyEntryView) -> Self {
        Self { label: value.label, disposition: value.disposition.into() }
    }
}
impl MortgageSubsidyEntryView {
    pub(crate) fn valid(&self) -> bool {
        MortgageSubsidyEntry::from(self.clone()).valid()
    }

    pub(crate) fn home_sale_income_id(&self) -> Option<i64> {
        MortgageSubsidyEntry::from(self.clone()).home_sale_income_id()
    }
}

/// Form 8828 treatment of a federally subsidized home
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MortgageSubsidyDispositionView {
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
        transfer: MortgageSubsidyTransferView,
        /// Original loan and issuer notice
        facts: MortgageSubsidyFacts,
    },
    /// Disposition, original loan, or recapture records need review
    Unreviewed,
}

impl From<MortgageSubsidyDisposition> for MortgageSubsidyDispositionView {
    fn from(value: MortgageSubsidyDisposition) -> Self {
        match value {
            MortgageSubsidyDisposition::ExemptSection1041 {
                recipient,
                disposed_on,
                qualifies_without_recognized_gain_or_loss,
            } => Self::ExemptSection1041 {
                recipient,
                disposed_on,
                qualifies_without_recognized_gain_or_loss,
            },
            MortgageSubsidyDisposition::ExemptHomeImprovementLoan {
                disposed_on,
                original_principal,
                qualified_sole_subsidy,
            } => Self::ExemptHomeImprovementLoan {
                disposed_on,
                original_principal,
                qualified_sole_subsidy,
            },
            MortgageSubsidyDisposition::ExemptBefore1991 {
                original_closed_on,
                disposed_on,
                no_later_subsidy,
            } => Self::ExemptBefore1991 { original_closed_on, disposed_on, no_later_subsidy },
            MortgageSubsidyDisposition::ExemptReplacedCasualty {
                destroyed_on,
                replaced_on,
                qualified_replacement_and_casualty_tax_resolved,
            } => Self::ExemptReplacedCasualty {
                destroyed_on,
                replaced_on,
                qualified_replacement_and_casualty_tax_resolved,
            },
            MortgageSubsidyDisposition::Ordinary { subsidy, transfer, facts } => {
                Self::Ordinary { subsidy, transfer: transfer.into(), facts }
            }
            MortgageSubsidyDisposition::Unreviewed => Self::Unreviewed,
        }
    }
}
impl From<MortgageSubsidyDispositionView> for MortgageSubsidyDisposition {
    fn from(value: MortgageSubsidyDispositionView) -> Self {
        match value {
            MortgageSubsidyDispositionView::ExemptSection1041 {
                recipient,
                disposed_on,
                qualifies_without_recognized_gain_or_loss,
            } => Self::ExemptSection1041 {
                recipient,
                disposed_on,
                qualifies_without_recognized_gain_or_loss,
            },
            MortgageSubsidyDispositionView::ExemptHomeImprovementLoan {
                disposed_on,
                original_principal,
                qualified_sole_subsidy,
            } => Self::ExemptHomeImprovementLoan {
                disposed_on,
                original_principal,
                qualified_sole_subsidy,
            },
            MortgageSubsidyDispositionView::ExemptBefore1991 {
                original_closed_on,
                disposed_on,
                no_later_subsidy,
            } => Self::ExemptBefore1991 { original_closed_on, disposed_on, no_later_subsidy },
            MortgageSubsidyDispositionView::ExemptReplacedCasualty {
                destroyed_on,
                replaced_on,
                qualified_replacement_and_casualty_tax_resolved,
            } => Self::ExemptReplacedCasualty {
                destroyed_on,
                replaced_on,
                qualified_replacement_and_casualty_tax_resolved,
            },
            MortgageSubsidyDispositionView::Ordinary { subsidy, transfer, facts } => {
                Self::Ordinary { subsidy, transfer: transfer.into(), facts }
            }
            MortgageSubsidyDispositionView::Unreviewed => Self::Unreviewed,
        }
    }
}

/// Transfer used for Form 8828 line 9
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MortgageSubsidyTransferView {
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
        #[libertas_constrained_by("$.mortgage_gift_owners")]
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
impl From<MortgageSubsidyTransfer> for MortgageSubsidyTransferView {
    fn from(value: MortgageSubsidyTransfer) -> Self {
        match value {
            MortgageSubsidyTransfer::Sale { home_sale_income_id, qsml_gain_share } => {
                Self::Sale { home_sale_income_id, qsml_gain_share }
            }
            MortgageSubsidyTransfer::GiftToOther {
                owners,
                fair_market_value,
                adjusted_basis,
                no_consideration_or_debt_relief,
            } => Self::GiftToOther {
                owners,
                fair_market_value,
                adjusted_basis,
                no_consideration_or_debt_relief,
            },
        }
    }
}
impl From<MortgageSubsidyTransferView> for MortgageSubsidyTransfer {
    fn from(value: MortgageSubsidyTransferView) -> Self {
        match value {
            MortgageSubsidyTransferView::Sale { home_sale_income_id, qsml_gain_share } => {
                Self::Sale { home_sale_income_id, qsml_gain_share }
            }
            MortgageSubsidyTransferView::GiftToOther {
                owners,
                fair_market_value,
                adjusted_basis,
                no_consideration_or_debt_relief,
            } => Self::GiftToOther {
                owners,
                fair_market_value,
                adjusted_basis,
                no_consideration_or_debt_relief,
            },
        }
    }
}
