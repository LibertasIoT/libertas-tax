// disaster items of src/federal/federal_model.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

fn relief_printed_issue(
    amount: i64,
    withholding: i64,
) -> Option<crate::validation::ValidationIssue> {
    (withholding > amount).then(|| {
        crate::validation::ValidationIssue::new("RELIEF_WITHHOLDING_ABOVE_PAYMENT")
            .at_field("withholding")
    })
}

/// Qualified wildfire relief payment received in 2026
/// Section 139M excludes only uncompensated losses, expenses or damages from a federally declared forest or range fire. A partially qualifying payment needs separate taxable-source treatment.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into source_issuer, source_recipient. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read one Form 1099-MISC reporting this wildfire relief payment exactly. Check the form's tax year against the supplied year and report a mismatch as an error. Read box 3 other income; report an error when box 3 is blank. Read box 4 federal income tax withheld, returning an empty string when it is clearly blank. Report an illegible amount, or two values that conflict, as an error. Do not read any other box. Do not report the fire, its declaration, the dates or whether the payment qualifies; the filer answers those. The file may hold several copies of this form, such as Copy B and Copy 2; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.source_issuer.federal_id,@.source_recipient.federal_id,@.amount,@.withholding"
)]
pub struct FederalWildfireRelief {
    /// Source document issuer as printed
    #[libertas_llm_doc_field]
    pub source_issuer: crate::PrintedParty,
    /// Source document recipient as printed
    #[libertas_llm_doc_field]
    pub source_recipient: crate::PrintedParty,
    /// Date payment was received in 2026
    #[libertas_date_only]
    pub paid: u32,
    /// Federal forest or range-fire disaster declaration date
    /// The declaration must be after December 31, 2014 and before January 1, 2027.
    #[libertas_date_only]
    pub declared: u32,
    /// Federal disaster declaration reference
    #[libertas_size(min = 1)]
    pub reference: String,
    /// Qualified payment received, excluding any separately taxable component
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
    /// Federal tax actually withheld on a Form 1099 reporting this payment
    /// Enter zero if no Form 1099 withheld tax; a regular employer wage payment is not qualified wildfire relief.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// This entire payment meets section 139M and has no duplicate tax benefit
    /// Confirm a federally declared forest or range fire, payment to or for this individual for resulting uncompensated losses, expenses or damages, and no overlap with insurance or other compensation. Employer-paid wages that would otherwise have been paid are excluded from this entry. Do not also enter the payment as wages or other income; do not claim a deduction or credit for a compensated expense or increase property basis by the excluded amount. Separate any nonqualifying portion into its proper taxable income source. No taxes the payment as other income.
    /// Check the payer statement, disaster declaration and reimbursed-expense records.
    pub qualified: bool,
}
impl FederalWildfireRelief {
    pub(crate) fn printed_issue(&self) -> Option<crate::validation::ValidationIssue> {
        relief_printed_issue(self.amount, self.withholding)
    }

    // What the payment adds to income: nothing when it qualifies as excluded.
    pub(crate) fn on_return(&self) -> i64 {
        if self.qualified { 0 } else { self.amount }
    }
    // The page's client rules, repeated on save (the server twins).
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        if !(20260101..=20261231).contains(&self.paid) {
            Some(
                crate::validation::ValidationIssue::new("RELIEF_RECEIVED_OUTSIDE_2026")
                    .at_field("paid"),
            )
        } else if !(20150101..=20261231).contains(&self.declared) {
            Some(
                crate::validation::ValidationIssue::new("WILDFIRE_DECLARED_OUTSIDE_2015_2026")
                    .at_field("declared"),
            )
        } else {
            self.printed_issue()
        }
    }
    pub(crate) fn valid(&self) -> bool {
        (20260101..=20261231).contains(&self.paid)
            && crate::federal_rules::valid_date(self.paid)
            && (20150101..=20261231).contains(&self.declared)
            && crate::federal_rules::valid_date(self.declared)
            && !self.reference.trim().is_empty()
            && (1..=100_000_000_000).contains(&self.amount)
            && (0..=self.amount).contains(&self.withholding)
    }
}

/// What a Form 1099-MISC box 3 wildfire relief payment was
/// The payment, payer and withholding come from the form as printed.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct WildfireReliefFacts {
    /// Date payment was received in 2026
    #[libertas_date_only]
    pub paid: u32,
    /// Federal forest or range-fire disaster declaration date
    /// The declaration must be after December 31, 2014 and before January 1, 2027.
    #[libertas_date_only]
    pub declared: u32,
    /// Federal disaster declaration reference
    #[libertas_size(min = 1)]
    pub reference: String,
    /// This entire payment meets section 139M and has no duplicate tax benefit
    /// Confirm a federally declared forest or range fire, payment to or for you for resulting uncompensated losses, expenses or damages, and no overlap with insurance or other compensation. No taxes the payment as other income.
    pub qualified: bool,
}
impl WildfireReliefFacts {
    // The relief calculation's own record for this printed amount.
    pub(crate) fn relief(&self, amount: i64) -> FederalWildfireRelief {
        FederalWildfireRelief {
            source_issuer: Default::default(),
            source_recipient: Default::default(),
            paid: self.paid,
            declared: self.declared,
            reference: self.reference.clone(),
            amount,
            withholding: 0,
            qualified: self.qualified,
        }
    }
}

/// Purpose of a section 139 qualified disaster relief payment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum DisasterReliefPurpose {
    /// Reasonable and necessary personal, family, living or funeral expense
    PersonalExpense,
    /// Reasonable and necessary repair of a personal residence or its contents
    ResidenceOrContents,
    /// Government payment to promote the general welfare after the disaster
    GovernmentWelfare,
    /// Common carrier's payment because a disaster caused death or physical injury
    CommonCarrier {
        /// Transportation common carrier that made the payment
        #[libertas_size(min = 1)]
        carrier: String,
        /// Death or physical injury caused by the disaster
        loss: DisasterCarrierLoss,
    },
}
/// Harm compensated by a common carrier under section 139(b)(3)
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum DisasterCarrierLoss {
    /// Death caused by the disaster
    Death,
    /// Personal physical injury caused by the disaster
    PhysicalInjury,
}
/// Section 692(c)(2) event type used by section 139(c)(1)
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum DisasterActionKind {
    /// Terroristic activity directed against the United States or an ally
    Terrorism,
    /// U.S. Armed Forces action responding to violence or aggression against the United States or an ally, excluding training
    Military,
}
impl DisasterReliefPurpose {
    pub(super) fn valid(&self) -> bool {
        self.issue().is_none()
    }
    pub(super) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        match self {
            Self::CommonCarrier { carrier, .. } if carrier.trim().is_empty() => Some(
                crate::validation::ValidationIssue::new("DISASTER_SOURCE_TEXT_INVALID")
                    .at_field("carrier")
                    .in_field("purpose"),
            ),
            _ => None,
        }
    }
}

/// Why the event is a qualified disaster under section 139(c)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum DisasterQualification {
    /// Terroristic or military action under section 692(c)(2)
    TerrorOrMilitary {
        /// Type of qualifying action
        kind: DisasterActionKind,
        /// Event and supporting determination or evidence
        #[libertas_size(min = 1)]
        event: String,
    },
    /// Federally declared disaster under section 139(c)(2)
    FederalDeclaration {
        /// Federal disaster declaration reference
        #[libertas_size(min = 1)]
        reference: String,
    },
    /// Accident involving a transportation common carrier under section 139(c)(3)
    CarrierAccident {
        /// Qualifying accident or disaster reference
        #[libertas_size(min = 1)]
        event: String,
    },
    /// Other event determined catastrophic by the Treasury Secretary under section 139(c)(3)
    SecretaryCatastrophe {
        /// Secretary's determination or IRS notice identifying the event
        #[libertas_size(min = 1)]
        determination: String,
    },
    /// Government assistance after an applicable authority determined the disaster warrants aid under section 139(c)(4)
    /// This basis applies only to a government general-welfare payment, not a private payment for personal or residence expenses.
    GovernmentAssistance {
        /// Event or disaster reference
        #[libertas_size(min = 1)]
        event: String,
        /// Federal, state or local authority that determined aid was warranted
        #[libertas_size(min = 1)]
        authority: String,
        /// Federal, state or local government agency that paid the assistance
        #[libertas_size(min = 1)]
        payer: String,
    },
}
impl DisasterQualification {
    pub(crate) fn valid(&self, purpose: &DisasterReliefPurpose) -> bool {
        self.issue(purpose).is_none()
    }
    pub(super) fn issue(
        &self,
        purpose: &DisasterReliefPurpose,
    ) -> Option<crate::validation::ValidationIssue> {
        self.binding_issue(purpose).or_else(|| self.text_issue())
    }
    pub(super) fn binding_issue(
        &self,
        purpose: &DisasterReliefPurpose,
    ) -> Option<crate::validation::ValidationIssue> {
        match self {
            Self::CarrierAccident { .. }
                if !matches!(purpose, DisasterReliefPurpose::CommonCarrier { .. }) =>
            {
                Some(
                    crate::validation::ValidationIssue::new(
                        "DISASTER_CARRIER_ACCIDENT_NEEDS_CARRIER_PAYMENT",
                    )
                    .at_field("purpose"),
                )
            }
            Self::GovernmentAssistance { .. }
                if !matches!(purpose, DisasterReliefPurpose::GovernmentWelfare) =>
            {
                Some(
                    crate::validation::ValidationIssue::new(
                        "DISASTER_ASSISTANCE_NEEDS_GOVERNMENT_PAYMENT",
                    )
                    .at_field("purpose"),
                )
            }
            _ => None,
        }
    }
    pub(super) fn text_issue(&self) -> Option<crate::validation::ValidationIssue> {
        let named = |value: &str, field| {
            value.trim().is_empty().then(|| {
                crate::validation::ValidationIssue::new("DISASTER_SOURCE_TEXT_INVALID")
                    .at_field(field)
                    .in_field("qualification")
            })
        };
        match self {
            Self::TerrorOrMilitary { event, .. } | Self::CarrierAccident { event } => {
                named(event, "event")
            }
            Self::FederalDeclaration { reference } => named(reference, "reference"),
            Self::SecretaryCatastrophe { determination } => named(determination, "determination"),
            Self::GovernmentAssistance { event, authority, payer } => named(event, "event")
                .or_else(|| named(authority, "authority"))
                .or_else(|| named(payer, "payer")),
        }
    }
}

/// Qualified disaster relief payment received in 2026
/// Section 139 excludes eligible individual assistance after a qualified disaster. Use a separate income source for any taxable portion or for a payment with a different tax treatment. If a Form 1099-G box 6 shows this grant, enter it here and leave it out of that 1099-G's box 6.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into source_issuer, source_recipient. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read one Form 1099-MISC or Form 1099-G reporting this disaster relief payment exactly. Check the form's tax year against the supplied year and report a mismatch as an error. For a 1099-MISC read box 3 other income; for a 1099-G read box 6 taxable grants; report an error when that box is blank. Read box 4 federal income tax withheld, returning an empty string when it is clearly blank. Report an illegible amount, or two values that conflict, as an error. Do not read any other box. Do not report the disaster, its basis, what the payment covered or whether it qualifies; the filer answers those. The file may hold several copies of this form, such as Copy B and Copy 2; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.source_issuer.federal_id,@.source_recipient.federal_id,@.amount,@.withholding"
)]
pub struct FederalDisasterRelief {
    /// Source document issuer as printed
    #[libertas_llm_doc_field]
    pub source_issuer: crate::PrintedParty,
    /// Source document recipient as printed
    #[libertas_llm_doc_field]
    pub source_recipient: crate::PrintedParty,
    /// Date the payment was received in 2026
    #[libertas_date_only]
    pub received: u32,
    /// Qualified-disaster basis
    pub qualification: DisasterQualification,
    /// What the qualified payment covered
    pub purpose: DisasterReliefPurpose,
    /// Entire eligible payment; enter any taxable component separately
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
    /// Actual federal income tax withheld on a payer's Form 1099, if any
    /// Section 139 relief is excluded even when a payer reports it as Form 1099 income. Enter zero when no tax was withheld; claim only the tax actually withheld, not a hypothetical amount.
    #[libertas_llm_doc_field]
    #[libertas_default(0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub withholding: i64,
    /// This entire payment qualifies under section 139
    /// Confirm the selected qualified-disaster basis and payment purpose, payment to or for this individual, and no insurance or other payment for the same compensated expense. Terrorism must target the United States or an ally; military action must involve U.S. forces responding to violence or aggression against the United States or an ally and cannot be training. A Secretary-catastrophe basis needs an actual Treasury or IRS determination. A common-carrier payment must come from the named transportation carrier because the disaster caused death or personal physical injury; an accident involving that carrier can itself be the qualifying disaster. A nonfederal government-assistance basis requires an applicable authority's determination and a government welfare payment; it cannot support a private expense reimbursement. The payment is not wage or business-income replacement or compensation for services. A government-welfare payment came from a federal, state or local government or its agency and served individual or family needs. Do not enter this amount elsewhere as taxable income or claim a deduction or credit for a compensated expense; account for any property-loss reimbursement separately. No taxes the payment as other income.
    /// Check the payer statement, disaster declaration and reimbursed-expense records.
    pub qualified: bool,
}
impl FederalDisasterRelief {
    pub(crate) fn printed_issue(&self) -> Option<crate::validation::ValidationIssue> {
        relief_printed_issue(self.amount, self.withholding)
    }

    // What the payment adds to income: nothing when it qualifies as excluded.
    pub(crate) fn on_return(&self) -> i64 {
        if self.qualified { 0 } else { self.amount }
    }
    // The page's client rules, repeated on save (the server twins).
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        if !(20260101..=20261231).contains(&self.received) {
            Some(
                crate::validation::ValidationIssue::new("RELIEF_RECEIVED_OUTSIDE_2026")
                    .at_field("received"),
            )
        } else {
            self.printed_issue().or_else(|| {
                self.qualification
                    .binding_issue(&self.purpose)
                    .or_else(|| self.purpose.issue())
                    .or_else(|| self.qualification.text_issue())
            })
        }
    }

    pub(crate) fn valid(&self) -> bool {
        (20260101..=20261231).contains(&self.received)
            && crate::federal_rules::valid_date(self.received)
            && self.purpose.valid()
            && self.qualification.valid(&self.purpose)
            && (1..=100_000_000_000).contains(&self.amount)
            && (0..=self.amount).contains(&self.withholding)
    }
}

/// What a Form 1099-MISC box 3 qualified disaster relief payment was
/// The payment, payer and withholding come from the form as printed.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct DisasterReliefFacts {
    /// Date the payment was received in 2026
    #[libertas_date_only]
    pub received: u32,
    /// Qualified-disaster basis
    pub qualification: DisasterQualification,
    /// What the qualified payment covered
    pub purpose: DisasterReliefPurpose,
    /// This entire payment qualifies under section 139
    /// Confirm the selected basis and purpose, payment to or for you, and no insurance or other payment for the same compensated expense. It is not wage or business-income replacement or compensation for services. No taxes the payment as other income.
    pub qualified: bool,
}
impl DisasterReliefFacts {
    // The relief calculation's own record for this printed amount.
    pub(crate) fn relief(&self, amount: i64) -> FederalDisasterRelief {
        FederalDisasterRelief {
            source_issuer: Default::default(),
            source_recipient: Default::default(),
            received: self.received,
            qualification: self.qualification.clone(),
            purpose: self.purpose.clone(),
            amount,
            withholding: 0,
            qualified: self.qualified,
        }
    }
}

/// One insurer payment for temporary living expenses after loss of a main home
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalCasualtyLivingPayment {
    /// Insurance company that made this payment
    #[libertas_size(min = 1)]
    pub insurer: String,
    /// Insurer's claim number
    #[libertas_size(min = 1)]
    pub claim_reference: String,
    /// Unique insurer payment or allocated transaction reference across main-home events
    #[libertas_size(min = 1)]
    pub payment_reference: String,
    /// Date this payment was received, no later than 2026
    #[libertas_date_only]
    pub received_on: u32,
    /// Payment solely for temporary living expenses
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
    /// Federal income tax actually withheld on this payment, if any
    #[libertas_default(0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub withholding: i64,
}
impl FederalCasualtyLivingPayment {
    pub(super) fn valid(&self, lost_use_on: u32) -> bool {
        !self.insurer.trim().is_empty()
            && !self.claim_reference.trim().is_empty()
            && !self.payment_reference.trim().is_empty()
            && (19000101..=20261231).contains(&self.received_on)
            && crate::federal_rules::valid_date(self.received_on)
            && self.received_on >= lost_use_on
            && (1..=100_000_000_000).contains(&self.amount)
            && (0..=self.amount).contains(&self.withholding)
    }
}

/// Why the main home could not be used during the insured period
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum FederalHomeLossOfUse {
    /// A casualty caused loss of use of the main home
    Casualty,
    /// Government authorities denied access because of a casualty or threat of one
    GovernmentDeniedAccess {
        /// Government authority that restricted access
        authority: String,
        /// Access order or public notice reference
        order_reference: String,
    },
}
impl FederalHomeLossOfUse {
    pub(super) fn valid(&self) -> bool {
        match self {
            Self::Casualty => true,
            Self::GovernmentDeniedAccess { authority, order_reference } => {
                !authority.trim().is_empty() && !order_reference.trim().is_empty()
            }
        }
    }
}

/// Insurance for temporary living expenses after losing use of a main home
/// Record one event and all related insurer payments through 2026, even when several insurers or claims paid. The temporary expense increase is applied once to the combined payments. Payments for property damage belong in the casualty record instead.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalCasualtyLivingInsurance {
    /// Distinct event or main-home loss-of-use reference
    #[libertas_size(min = 1)]
    pub event_reference: String,
    /// Casualty or government access restriction that prevented using the main home
    pub loss_of_use: FederalHomeLossOfUse,
    /// Date the casualty or threat caused loss of use of the main home
    #[libertas_date_only]
    pub lost_use_on: u32,
    /// Date use of the main home was regained, no later than 2026; choose the still-unusable answer if use was not regained by the end of 2026
    /// [NullText]
    /// The main home was still unusable at the end of 2026
    /// [SetText]
    /// Use of the main home was regained on
    #[libertas_date_only]
    pub regained_use_on: Option<u32>,
    /// All temporary-living-expense insurer payments for this event through 2026
    #[libertas_size(min = 1)]
    /// ----
    /// One payment with its insurer, claim, transaction, date, amount and withholding
    pub payments: Vec<FederalCasualtyLivingPayment>,
    /// Reasonable actual living expenses during the loss-of-use period
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub actual_living_expenses: i64,
    /// Normal expenses for the same period that were avoided or replaced because of the casualty
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub normal_living_expenses: i64,
    /// Taxable part of these insurer payments actually reported on returns before 2026, from retained returns
    /// Enter zero when use of the main home was regained in 2026. If the home was regained earlier, this must match the excess of pre-2026 payments over the whole temporary expense increase.
    #[libertas_default(0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub prior_taxable_reported: i64,
    /// Did the casualty occur in a federally declared disaster area?
    /// Look the event up in FEMA's declaration list (fema.gov/disaster/declarations). If Yes, enter the FEMA declaration reference below.
    /// Check insurer settlements, actual temporary expenses and earlier returns.
    pub federal_disaster_area: bool,
    /// FEMA declaration reference when the casualty occurred in a federally declared disaster area
    #[libertas_default("")]
    pub federal_declaration_reference: String,
    /// Confirm the main-home casualty or a government access order caused by a casualty or its threat, all insurers and payments through 2026, prior return treatment, reasonable actual and normal expenses, and no duplicate income, expense, casualty-loss reimbursement or qualified disaster-relief entry
    /// If regained use is empty, confirm the home remained unusable through December 31, 2026; the final expense increase is then still pending and 2026 income is zero. A separate property-damage reimbursement cannot be entered here. No counts every 2026 insurer payment as taxable.
    /// Check insurer settlements, actual temporary expenses and earlier returns.
    pub confirmed: bool,
}
impl FederalCasualtyLivingInsurance {
    // The page's client rules, repeated on save (the server twins).
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        if self.regained_use_on.is_some_and(|regained| regained < self.lost_use_on) {
            Some(
                crate::validation::ValidationIssue::new("CASUALTY_LIVING_REGAINED_BEFORE_LOST")
                    .at_field("regained_use_on"),
            )
        } else if self.federal_disaster_area && self.federal_declaration_reference.is_empty() {
            Some(
                crate::validation::ValidationIssue::new("CASUALTY_LIVING_DISASTER_NEEDS_REFERENCE")
                    .at_field("federal_declaration_reference"),
            )
        } else if !self.federal_disaster_area && !self.federal_declaration_reference.is_empty() {
            Some(
                crate::validation::ValidationIssue::new(
                    "CASUALTY_LIVING_REFERENCE_WITHOUT_DISASTER",
                )
                .at_field("federal_declaration_reference"),
            )
        } else {
            None
        }
    }
    pub(crate) fn valid(&self) -> bool {
        (19000101..=20261231).contains(&self.lost_use_on)
            && crate::federal_rules::valid_date(self.lost_use_on)
            && self.regained_use_on.is_none_or(|date| {
                (19000101..=20261231).contains(&date)
                    && crate::federal_rules::valid_date(date)
                    && self.lost_use_on <= date
            })
            && !self.event_reference.trim().is_empty()
            && self.loss_of_use.valid()
            && (1..=100).contains(&self.payments.len())
            && self.payments.iter().enumerate().all(|(index, payment)| {
                payment.valid(self.lost_use_on)
                    && !self.payments[..index].iter().any(|earlier| {
                        earlier.insurer == payment.insurer
                            && earlier.payment_reference == payment.payment_reference
                    })
            })
            && (0..=100_000_000_000).contains(&self.actual_living_expenses)
            && (0..=100_000_000_000).contains(&self.normal_living_expenses)
            && (0..=self.prior_payments()).contains(&self.prior_taxable_reported)
            && (self.regained_use_on.is_some_and(|date| date < 20260101)
                || self.prior_taxable_reported == 0)
            && if self.federal_disaster_area {
                !self.federal_declaration_reference.trim().is_empty()
            } else {
                self.federal_declaration_reference.trim().is_empty()
            }
    }

    pub(crate) fn temporary_increase(&self) -> i64 {
        phase::floor(self.actual_living_expenses - self.normal_living_expenses, 0)
    }

    pub(crate) fn prior_payments(&self) -> i64 {
        self.payments
            .iter()
            .filter(|payment| payment.received_on < 20260101)
            .map(|payment| payment.amount)
            .sum()
    }

    pub(crate) fn current_payments(&self) -> i64 {
        self.payments
            .iter()
            .filter(|payment| payment.received_on >= 20260101)
            .map(|payment| payment.amount)
            .sum()
    }

    pub(crate) fn current_withholding(&self) -> i64 {
        self.payments
            .iter()
            .filter(|payment| payment.received_on >= 20260101)
            .map(|payment| payment.withholding)
            .sum()
    }

    pub(crate) fn expected_prior_taxable(&self) -> i64 {
        if !self.federal_disaster_area && self.regained_use_on.is_some_and(|date| date < 20260101) {
            phase::floor(self.prior_payments() - self.temporary_increase(), 0)
        } else {
            0
        }
    }

    pub(crate) fn taxable_in_year(&self) -> i64 {
        if !self.federal_disaster_area && self.regained_use_on.is_some() {
            phase::floor(
                self.prior_payments() + self.current_payments() - self.temporary_increase(),
                0,
            ) - self.expected_prior_taxable()
        } else {
            0
        }
    }

    /// 2026 income: every 2026 insurer payment when the casualty facts are
    /// not confirmed.
    pub(crate) fn included_in_return_year(&self) -> i64 {
        if self.confirmed { self.taxable_in_year() } else { self.current_payments() }
    }
}

/// Statutory program funding hazard mitigation under section 139(g)
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum DisasterMitigationProgram {
    /// Stafford Act hazard-mitigation payment
    StaffordAct,
    /// National Flood Insurance Act hazard-mitigation payment
    NationalFloodInsuranceAct,
}

/// Qualified disaster-mitigation payment received by a property owner in 2026
/// Section 139(g) excludes statutory hazard-mitigation payments, not consideration for the sale or disposition of the property.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalDisasterMitigation {
    /// Date the payment was received in 2026
    #[libertas_date_only]
    pub received: u32,
    /// Grant or payment reference identifying the qualifying program
    #[libertas_size(min = 1)]
    pub award_reference: String,
    /// Which federal law authorized this payment
    pub program: DisasterMitigationProgram,
    /// Property whose future hazard risk was reduced
    #[libertas_size(min = 1)]
    pub property: String,
    /// Entire qualifying payment; enter a nonqualifying part separately
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
    /// This entire payment qualifies for the section 139(g) exclusion
    /// The FEMA or NFIP hazard-mitigation award letter names the program. Yes when the payment was made under the selected law as in effect on April 15, 2005, to or for this property's owner for hazard mitigation, not a sale or other disposition. Do not enter it elsewhere as income, claim a deduction or credit for a funded expenditure, or increase the property's basis by the excluded payment. No leaves the payment out of this estimate.
    pub qualified: bool,
}
impl FederalDisasterMitigation {
    // The page's client rule, repeated on save.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        (!(20260101..=20261231).contains(&self.received)).then(|| {
            crate::validation::ValidationIssue::new("RELIEF_RECEIVED_OUTSIDE_2026")
                .at_field("received")
        })
    }
    pub(crate) fn valid(&self) -> bool {
        (20260101..=20261231).contains(&self.received)
            && crate::federal_rules::valid_date(self.received)
            && !self.award_reference.trim().is_empty()
            && !self.property.trim().is_empty()
            && (1..=100_000_000_000).contains(&self.amount)
    }
}
