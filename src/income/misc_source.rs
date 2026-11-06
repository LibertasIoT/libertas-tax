use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Jury duty pay
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct JuryPay {
    /// Jury pay received in 2026
    /// From the court's payment records; leave out separately paid travel reimbursements.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub received: i64,
    /// Jury pay you gave to your employer
    /// Only pay you turned over because your employer kept paying your salary while you served. Schedule 1 line 24a.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub remitted: i64,
}
impl JuryPay {
    pub(crate) fn valid(&self) -> bool {
        (0..=100_000_000_000).contains(&self.received)
            && (0..=100_000_000_000).contains(&self.remitted)
            && self.remitted <= self.received
    }
    // The page's client rule, repeated on save.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        (self.remitted > self.received).then(|| {
            crate::validation::ValidationIssue::new("JURY_REMITTED_ABOVE_RECEIVED")
                .at_field("remitted")
        })
    }
}

/// Foreign account or trust
/// Schedule B Part III, for the whole return.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ForeignAccountOrTrust {
    /// You had a financial interest in or signature authority over a financial account in a foreign country
    #[libertas_default(true)]
    pub account: bool,
    /// You are required to file FinCEN Form 114 (FBAR) for these accounts
    /// Generally required when aggregate foreign financial accounts exceed $10,000 at any time, subject to FinCEN's exceptions. This is separate from Form 8938 asset thresholds. Check the accounts’ highest balances, ownership and signature-authority records against FinCEN’s filing requirements.
    /// [NullText]
    /// No foreign financial account is reported
    /// [SetText]
    /// For the reported foreign accounts, the FBAR filing answer is
    pub fbar_required: Option<bool>,
    /// Countries of the accounts reportable on your FBAR
    /// Select actual account countries only when FBAR filing is required. U.S. territories and possessions are not foreign for this purpose.
    #[libertas_enum_source("$.foreign_countries")]
    pub fbar_countries: Vec<i64>,
    /// You received a distribution from, or created or transferred money or property to, a foreign trust
    #[libertas_default(false)]
    pub trust: bool,
    /// Highest total value during 2026 of your foreign accounts and other foreign financial assets
    /// In U.S. dollars, at the highest point of the year: foreign bank and brokerage accounts, foreign stocks, bonds and fund shares held outside a U.S. account, and interests in foreign entities or trusts. Add your spouse's on a joint return, counting jointly owned assets once. Used only to tell you whether you also need Form 8938.
    /// [NullText]
    /// No foreign account or trust assets are reported
    /// [SetText]
    /// The highest total foreign financial-asset value was
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub highest_value: Option<i64>,
}
/// Last year's QBI loss carryforward
/// From your 2025 Form 8995 or 8995-A; enter each loss as a positive amount.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct QbiLossCarryforward {
    /// Business loss carried forward (Form 8995 line 16 or Form 8995-A Schedule C line 6)
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub business: i64,
    /// REIT dividend and publicly traded partnership loss carried forward (Form 8995 line 17 or Form 8995-A line 29)
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub reit_ptp: i64,
    /// Ownership of retained QBI losses
    /// Only for a Form 8379 claim. Use actual source records; this never changes the joint calculation.
    /// [NullText]
    /// The retained loss ownership has not been established
    /// [SetText]
    /// The retained losses belong to the spouses as follows
    #[libertas_default($null)]
    #[libertas_constrained_by("$.allowed_injured_source_facts")]
    pub injured_owners: Option<crate::InjuredQbiLossOwners>,
}
