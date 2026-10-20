// Coverage: Schedule 2 additional taxes — Partial (2026-10-10).
//   Named uncomputed mandatory-tax source histories stay explicit; no generic
//   entered-tax override, zero-tax assumption, or claim of closing mixed rows.
//   Existing calculated components keep their owning modules and destinations.
//   Sources: IRS 2026 draft Schedule 2; final 2025 Form1040 instructions pp. 111–114
//   (annual numbering differs); Code sections named at each source category.
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// An additional tax whose source calculation is not supported
/// Select the actual tax situation from its source records. These choices describe an App calculation limit; they do not prohibit filing a return for that situation. Do not select a tax already calculated from its own income or account record.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RareAdditionalTax {
    /// Form 4255 net elective-payment recapture
    /// Sections 6417/6418; Schedule 2 lines 1d and 10. The required source computation is not yet calculated.
    Form4255NetEpe,
    /// Form 4255 excessive payment and penalty
    /// Sections 6417/6418; Schedule 2 lines 1e and 1f (separate penalty). The required source computation is not yet calculated.
    Form4255ExcessivePayment,
    /// Other credit recapture
    /// Credit-specific source histories; Schedule 2 lines 1y/13a. The required source computation is not yet calculated.
    CreditRecaptureOther,
    /// Lots or timeshares installment-sale interest
    /// Section 453(l)(3); Schedule 2 line 7. The required source computation is not yet calculated.
    InstallmentLotsTimeshares,
    /// Earlier installment obligations subject to interest
    /// Section 453A(c); Schedule 2 line 8. The required source computation is not yet calculated.
    InstallmentInterestHistory,
    /// Direct-building or ordinary partnership housing-credit recapture
    /// Section 42(j); Schedule 2 line 9. The required source computation is not yet calculated.
    HousingDirectOrOrdinaryPartner,
    /// Fractional charitable-interest recapture
    /// Section 170(o); Schedule 2 line 13g. The required source computation is not yet calculated.
    FractionalCharitableRecapture,
    /// Nonqualified deferred-compensation additional tax and premium interest
    /// Section 409A(a)(1)(B); Schedule 2 line 13h. The required source computation is not yet calculated.
    DeferredCompensation409A,
    /// Offshore deferred-compensation additional tax and interest
    /// Section 457A(c); Schedule 2 line 13i. The required source computation is not yet calculated.
    DeferredCompensation457A,
    /// Excess-benefits tax for an owner-employee
    /// Section 72(m)(5); Schedule 2 line 13j. The required source computation is not yet calculated.
    ExcessBenefits72m5,
    /// Trust accumulation-distribution tax
    /// Sections 665–668; Form 4970; Schedule 2 line 13l. The required source computation is not yet calculated.
    TrustAccumulation4970,
    /// Insider-stock excise tax
    /// Section 4985; Schedule 2 line 13m. The required source computation is not yet calculated.
    InsiderStock4985,
    /// Long-term-contract look-back interest
    /// Section 460; Form 8697; Schedule 2 line 13n. The required source computation is not yet calculated.
    LookBack8697,
    /// Income-forecast look-back interest
    /// Section 167(g); Form 8866; Schedule 2 line 13n. The required source computation is not yet calculated.
    LookBack8866,
    /// PFIC tax and interest
    /// Section 1291; Form 8621; Schedule 2 lines 19a–19c and related income tax. The required source computation is not yet calculated.
    Pfic8621Interest,
    /// Another identified additional tax
    /// An additional federal tax whose computation is outside the App scope. The required source computation is not yet calculated.
    AnotherIdentifiedAddedTax,
}

impl RareAdditionalTax {
    #[cfg(test)]
    pub(crate) const ALL: [Self; 16] = [
        Self::Form4255NetEpe,
        Self::Form4255ExcessivePayment,
        Self::CreditRecaptureOther,
        Self::InstallmentLotsTimeshares,
        Self::InstallmentInterestHistory,
        Self::HousingDirectOrOrdinaryPartner,
        Self::FractionalCharitableRecapture,
        Self::DeferredCompensation409A,
        Self::DeferredCompensation457A,
        Self::ExcessBenefits72m5,
        Self::TrustAccumulation4970,
        Self::InsiderStock4985,
        Self::LookBack8697,
        Self::LookBack8866,
        Self::Pfic8621Interest,
        Self::AnotherIdentifiedAddedTax,
    ];
    pub(crate) fn refusal_info(
        &self,
    ) -> (&'static str, &'static str, &'static str, &'static str, &'static str) {
        match self {
            Self::Form4255NetEpe => (
                crate::fixture_english!("Form 4255 net elective-payment recapture"),
                "TOPIC_RARE_FORM4255_NET_EPE",
                crate::fixture_english!("Sections 6417/6418; Schedule 2 lines 1d and 10"),
                "ISSUE_RARE_FORM4255_NET_EPE",
                "NAME_RARE_FORM4255_NET_EPE",
            ),
            Self::Form4255ExcessivePayment => (
                crate::fixture_english!("Form 4255 excessive payment and penalty"),
                "TOPIC_RARE_FORM4255_EXCESSIVE_PAYMENT",
                crate::fixture_english!(
                    "Sections 6417/6418; Schedule 2 lines 1e and 1f (separate penalty)"
                ),
                "ISSUE_RARE_FORM4255_EXCESSIVE_PAYMENT",
                "NAME_RARE_FORM4255_EXCESSIVE_PAYMENT",
            ),
            Self::CreditRecaptureOther => (
                crate::fixture_english!("Other credit recapture"),
                "TOPIC_RARE_CREDIT_RECAPTURE_OTHER",
                crate::fixture_english!(
                    "Credit-specific source histories; Schedule 2 lines 1y/13a"
                ),
                "ISSUE_RARE_CREDIT_RECAPTURE_OTHER",
                "NAME_RARE_CREDIT_RECAPTURE_OTHER",
            ),
            Self::InstallmentLotsTimeshares => (
                crate::fixture_english!("Lots or timeshares installment-sale interest"),
                "TOPIC_RARE_INSTALLMENT_LOTS_TIMESHARES",
                crate::fixture_english!("Section 453(l)(3); Schedule 2 line 7"),
                "ISSUE_RARE_INSTALLMENT_LOTS_TIMESHARES",
                "NAME_RARE_INSTALLMENT_LOTS_TIMESHARES",
            ),
            Self::InstallmentInterestHistory => (
                crate::fixture_english!("Earlier installment obligations subject to interest"),
                "TOPIC_RARE_INSTALLMENT_INTEREST_HISTORY",
                crate::fixture_english!("Section 453A(c); Schedule 2 line 8"),
                "ISSUE_RARE_INSTALLMENT_INTEREST_HISTORY",
                "NAME_RARE_INSTALLMENT_INTEREST_HISTORY",
            ),
            Self::HousingDirectOrOrdinaryPartner => (
                crate::fixture_english!(
                    "Direct-building or ordinary partnership housing-credit recapture"
                ),
                "TOPIC_RARE_HOUSING_DIRECT_OR_ORDINARY_PARTNER",
                crate::fixture_english!("Section 42(j); Schedule 2 line 9"),
                "ISSUE_RARE_HOUSING_DIRECT_OR_ORDINARY_PARTNER",
                "NAME_RARE_HOUSING_DIRECT_OR_ORDINARY_PARTNER",
            ),
            Self::FractionalCharitableRecapture => (
                crate::fixture_english!("Fractional charitable-interest recapture"),
                "TOPIC_RARE_FRACTIONAL_CHARITABLE_RECAPTURE",
                crate::fixture_english!("Section 170(o); Schedule 2 line 13g"),
                "ISSUE_RARE_FRACTIONAL_CHARITABLE_RECAPTURE",
                "NAME_RARE_FRACTIONAL_CHARITABLE_RECAPTURE",
            ),
            Self::DeferredCompensation409A => (
                crate::fixture_english!(
                    "Nonqualified deferred-compensation additional tax and premium interest"
                ),
                "TOPIC_RARE_DEFERRED_COMPENSATION409_A",
                crate::fixture_english!("Section 409A(a)(1)(B); Schedule 2 line 13h"),
                "ISSUE_RARE_DEFERRED_COMPENSATION409_A",
                "NAME_RARE_DEFERRED_COMPENSATION409_A",
            ),
            Self::DeferredCompensation457A => (
                crate::fixture_english!(
                    "Offshore deferred-compensation additional tax and interest"
                ),
                "TOPIC_RARE_DEFERRED_COMPENSATION457_A",
                crate::fixture_english!("Section 457A(c); Schedule 2 line 13i"),
                "ISSUE_RARE_DEFERRED_COMPENSATION457_A",
                "NAME_RARE_DEFERRED_COMPENSATION457_A",
            ),
            Self::ExcessBenefits72m5 => (
                crate::fixture_english!("Excess-benefits tax for an owner-employee"),
                "TOPIC_RARE_EXCESS_BENEFITS72M5",
                crate::fixture_english!("Section 72(m)(5); Schedule 2 line 13j"),
                "ISSUE_RARE_EXCESS_BENEFITS72M5",
                "NAME_RARE_EXCESS_BENEFITS72M5",
            ),
            Self::TrustAccumulation4970 => (
                crate::fixture_english!("Trust accumulation-distribution tax"),
                "TOPIC_RARE_TRUST_ACCUMULATION4970",
                crate::fixture_english!("Sections 665–668; Form 4970; Schedule 2 line 13l"),
                "ISSUE_RARE_TRUST_ACCUMULATION4970",
                "NAME_RARE_TRUST_ACCUMULATION4970",
            ),
            Self::InsiderStock4985 => (
                crate::fixture_english!("Insider-stock excise tax"),
                "TOPIC_RARE_INSIDER_STOCK4985",
                crate::fixture_english!("Section 4985; Schedule 2 line 13m"),
                "ISSUE_RARE_INSIDER_STOCK4985",
                "NAME_RARE_INSIDER_STOCK4985",
            ),
            Self::LookBack8697 => (
                crate::fixture_english!("Long-term-contract look-back interest"),
                "TOPIC_RARE_LOOK_BACK8697",
                crate::fixture_english!("Section 460; Form 8697; Schedule 2 line 13n"),
                "ISSUE_RARE_LOOK_BACK8697",
                "NAME_RARE_LOOK_BACK8697",
            ),
            Self::LookBack8866 => (
                crate::fixture_english!("Income-forecast look-back interest"),
                "TOPIC_RARE_LOOK_BACK8866",
                crate::fixture_english!("Section 167(g); Form 8866; Schedule 2 line 13n"),
                "ISSUE_RARE_LOOK_BACK8866",
                "NAME_RARE_LOOK_BACK8866",
            ),
            Self::Pfic8621Interest => (
                crate::fixture_english!("PFIC tax and interest"),
                "TOPIC_RARE_PFIC8621_INTEREST",
                crate::fixture_english!(
                    "Section 1291; Form 8621; Schedule 2 lines 19a–19c and related income tax"
                ),
                "ISSUE_RARE_PFIC8621_INTEREST",
                "NAME_RARE_PFIC8621_INTEREST",
            ),
            Self::AnotherIdentifiedAddedTax => (
                crate::fixture_english!("Another identified additional tax"),
                "TOPIC_RARE_ANOTHER_IDENTIFIED_ADDED_TAX",
                crate::fixture_english!(
                    "An additional federal tax whose computation is outside the App scope"
                ),
                "ISSUE_RARE_ANOTHER_IDENTIFIED_ADDED_TAX",
                "NAME_RARE_ANOTHER_IDENTIFIED_ADDED_TAX",
            ),
        }
    }
}
