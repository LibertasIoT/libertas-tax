pub use crate::mortgage_certificate_source::{
    MortgageInterestSource, MortgageOriginalInterestMethod, MortgageReissueFacts,
};
pub use crate::mortgage_source::{
    AverageBalance, Form1098, Mortgage1098Source, MortgageDebtDate, MortgageDetails,
    MortgageFirstLender, MortgagePaidShare,
};

#[allow(unused_imports)]
use super::*;

/// A needed loan fact cannot be derived from the source as supplied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mortgage1098ConversionError {
    PaidInterestUnavailable,
    PaidInterestAboveBox1,
    InsuranceContractDateNeeded,
    PointsTreatmentNeeded,
    DrawDateNeeded,
}

impl Mortgage1098Source {
    pub(crate) fn conversion_issue(&self) -> Option<crate::validation::ValidationIssue> {
        use Mortgage1098ConversionError as E;
        let (key, field) = match crate::MortgageLoan::try_from(self).err()? {
            E::PaidInterestUnavailable => ("MORTGAGE_PAID_INTEREST_NEEDED", "paid_share"),
            E::PaidInterestAboveBox1 => ("MORTGAGE_PAID_INTEREST_ABOVE_BOX1", "paid_share"),
            E::InsuranceContractDateNeeded => {
                ("MORTGAGE_INSURANCE_CONTRACT_DATE_NEEDED", "insurance_contract")
            }
            E::PointsTreatmentNeeded => ("MORTGAGE_POINTS_TREATMENT_NEEDED", "points"),
            E::DrawDateNeeded => {
                return Some(
                    crate::validation::ValidationIssue::new("MORTGAGE_DRAW_DATE_NEEDED")
                        .at_field("date")
                        .in_field("debt_date"),
                );
            }
        };
        Some(crate::validation::ValidationIssue::new(key).at_field(field))
    }
}

impl TryFrom<&Mortgage1098Source> for crate::MortgageLoan {
    type Error = Mortgage1098ConversionError;

    fn try_from(source: &Mortgage1098Source) -> Result<Self, Self::Error> {
        use Mortgage1098ConversionError as E;
        let Mortgage1098Source {
            id,
            form,
            assigned,
            owner_source,
            joint_share,
            property,
            lender: loan_lender,
            details,
            paid_share,
            insurance_contract,
            points,
            debt_date,
            refund_origin,
            answered: _,
        } = source;
        let Form1098 {
            // These printed facts are retained for display and import, not loan calculations.
            calendar_year: _,
            corrected: _,
            lender,
            lender_telephone: _,
            borrower,
            account_number: _,
            box_1,
            box_2,
            box_3,
            box_4,
            box_5,
            box_6,
            box_7: _,
            box_8: _,
            box_9: _,
            box_10: _,
            box_11: _,
        } = form;
        let interest = match paid_share {
            MortgagePaidShare::All => *box_1,
            MortgagePaidShare::Amount(crate::SourceCents::Reported { amount }) => *amount,
            MortgagePaidShare::Amount(crate::SourceCents::NotReported) => 0,
            MortgagePaidShare::Amount(crate::SourceCents::Unavailable) => {
                return Err(E::PaidInterestUnavailable);
            }
        };
        if interest > *box_1 {
            return Err(E::PaidInterestAboveBox1);
        }
        let mortgage_insurance_premiums = if *box_5 == 0 {
            0
        } else {
            // Pub. 936, Mortgage Insurance Premiums: the contract must have
            // been issued after 2006. https://www.irs.gov/publications/p936
            match insurance_contract {
                crate::SourceDate::Reported { date } if *date >= 20070101 => *box_5,
                crate::SourceDate::Reported { date: _ } => 0,
                // Without a contract date, retain the printed premiums in the
                // projection. The contract-date question belongs on the loan topic.
                crate::SourceDate::Unavailable | crate::SourceDate::NotReported => *box_5,
            }
        };
        let incurred = match debt_date {
            MortgageDebtDate::Origination => *box_3,
            MortgageDebtDate::Draw { date: crate::SourceDate::Reported { date } } => *date,
            MortgageDebtDate::Draw {
                date: crate::SourceDate::Unavailable | crate::SourceDate::NotReported,
            } => return Err(E::DrawDateNeeded),
        };
        let points = if *box_6 == 0 {
            crate::MortgagePoints::No
        } else {
            match points {
                crate::MortgagePointTreatment::None => return Err(E::PointsTreatmentNeeded),
                crate::MortgagePointTreatment::Purchase => {
                    crate::MortgagePoints::Purchase { amount: *box_6 }
                }
                crate::MortgagePointTreatment::Amortized { term_months, ended } => {
                    crate::MortgagePoints::Amortized {
                        amount: *box_6,
                        term_months: *term_months,
                        ended: *ended,
                    }
                }
                crate::MortgagePointTreatment::Improvement { term_months } => {
                    crate::MortgagePoints::Improvement { amount: *box_6, term_months: *term_months }
                }
            }
        };
        let details = match details {
            MortgageDetails::EarlierLender { loan } => {
                crate::MortgageLoanDetails::EarlierLender { loan: *loan }
            }
            MortgageDetails::FirstLender(facts) => {
                crate::MortgageLoanDetails::FirstLender(crate::MortgageLedger {
                    home: facts.home,
                    purpose: facts.purpose.clone(),
                    average: match &facts.average {
                        AverageBalance::WithinLimit => crate::MortgageAverageBalance::WithinLimit,
                        AverageBalance::FirstAndLast { last_balance } => {
                            crate::MortgageAverageBalance::FirstAndLast {
                                last_balance: *last_balance,
                            }
                        }
                        AverageBalance::InterestOverRate { rate } => {
                            crate::MortgageAverageBalance::InterestOverRate { rate: *rate }
                        }
                        AverageBalance::Statements { average } => {
                            crate::MortgageAverageBalance::Statements { average_balance: *average }
                        }
                        AverageBalance::Monthly { balances } => {
                            crate::MortgageAverageBalance::Monthly { balances: balances.clone() }
                        }
                    },
                })
            }
        };
        Ok(Self {
            printed: Some(crate::PrintedDocumentNames {
                recipient: borrower.clone(),
                issuer: Some(lender.clone()),
                insured_party: None,
                fiduciary: None,
                transfer_corporation: None,
                assigned: *assigned,
                owner_source: *owner_source,
                joint_share: *joint_share,
            }),
            id: *id,
            property: property.clone(),
            lender: loan_lender.clone(),
            interest,
            outstanding_principal: *box_2,
            incurred,
            details,
            refund_of_interest: if *box_4 == 0 {
                crate::MortgageInterestRefund::No
            } else {
                crate::MortgageInterestRefund::Yes { amount: *box_4, origin: refund_origin.clone() }
            },
            mortgage_insurance_premiums,
            points,
        })
    }
}

// Mortgages/Mortgage1098 project FederalDeductions.mortgage independently of the
// deduction method. Each saved lender document has an ID; a transferred document
// selects one saved canonical FirstLender ledger for home, purpose and balances.
// Form1098 imports boxes1–6; personal labels, links, terms and OID facts stay manual.
// Box10 escrow/property taxes are entered once on Taxes you paid, ScheduleA5b.
// Charity carryovers remain FederalDeductions.prior_cash_charity (Pub526).
// interview.rs owns mappings, accepted-ID catalogs and WithinLimit save checks.
/// Home mortgage interest
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Mortgages {
    /// Forms 1098
    #[libertas_unordered]
    #[libertas_read_only]
    /// ----
    /// Form 1098
    pub loans: Vec<crate::MortgageDocumentView>,
    /// Forms 1098 not answered yet
    /// Forms entered since this page was saved: answer each loan's facts.
    // Asked unanswered beside the answered loans (BeginMortgages); empty once
    // every loan is answered.
    #[libertas_unordered]
    #[libertas_read_only]
    /// ----
    /// Form 1098
    pub new_loans: Vec<crate::MortgageDocumentView>,
}
/// Form 1098
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into lender, borrower. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read this one Form 1098 (Mortgage Interest Statement) exactly. Check its calendar year against the supplied year; report a mismatch. Read box 1 mortgage interest received, box 2 outstanding mortgage principal and box 3 mortgage origination date exactly as printed; a blank, illegible or conflicting box 1, 2 or 3 is an error, never zero or a guess. Return box 3 as printed, also for a refinance. Read box 4 refund of overpaid interest, box 5 mortgage insurance premiums and box 6 points paid on the purchase of a principal residence when printed; a blank box 4, 5 or 6 means none. Read box 6 as the printed amount; its treatment is answered on the mortgage topic. Do not read boxes 7 through 11 or any other box, and do not infer which home secures the loan, how the loan was used, what a refund repays, the average balance or any fact absent from the form. The file may hold several copies of this form, such as Copy B and Copy C; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    form_date_order = "MonthDayYear"
)]
pub struct Mortgage1098 {
    /// RECIPIENT'S/LENDER'S name, TIN and address
    #[libertas_llm_doc_field]
    pub lender: crate::PrintedParty,
    /// PAYER'S/BORROWER'S name, TIN and address
    #[libertas_llm_doc_field]
    pub borrower: crate::PrintedParty,
    /// Loan record identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Box 1 mortgage interest
    /// When you and someone other than your spouse owe this loan, enter only the interest you paid.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub interest: i64,
    /// Box 2 outstanding principal
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub principal: i64,
    /// Box 3 origination date
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub originated: u32,
    /// Box 4 refund of overpaid interest
    /// A refund of interest you paid in an earlier year. It does not reduce this year's interest; More about this loan asks what it repays.
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub refund: i64,
    /// Box 5 mortgage insurance premiums
    /// Deductible for 2026 on Schedule A line 8d, reduced when your adjusted gross income is over $100,000 ($50,000 if married filing separately) and none above $109,000 ($54,500). Leave at zero when the mortgage insurance contract was issued before 2007.
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub insurance: i64,
    /// Box 6 points
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub points: i64,
    /// More about this loan
    pub more: MortgageMore,
}
/// Loan facts or the earlier lender
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MortgageDetailsView {
    /// First lender's entry for this loan
    /// Enter the balance for the whole year, including time with later lenders.
    FirstLender(MortgageFirstLender),
    /// A later lender acquired this same loan in 2026
    EarlierLender {
        /// Earlier lender's loan
        /// Save the first lender's entry before choosing it here. Its home, use and year-wide balances apply to every lender's interest; they are entered only once.
        #[libertas_enum_source("$.mortgage_loans")]
        loan: i64,
    },
}
impl From<MortgageDetails> for MortgageDetailsView {
    fn from(value: MortgageDetails) -> Self {
        match value {
            MortgageDetails::FirstLender(facts) => Self::FirstLender(facts),
            MortgageDetails::EarlierLender { loan } => Self::EarlierLender { loan },
        }
    }
}
impl From<MortgageDetailsView> for MortgageDetails {
    fn from(value: MortgageDetailsView) -> Self {
        match value {
            MortgageDetailsView::FirstLender(facts) => Self::FirstLender(facts),
            MortgageDetailsView::EarlierLender { loan } => Self::EarlierLender { loan },
        }
    }
}
/// More about this loan
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MortgageMore {
    /// Refund answers the printed box 4 calls for
    #[libertas_hidden]
    #[libertas_read_only]
    pub allowed_refund_origins: Vec<i32>,
    /// Property label or address
    /// Optional label to distinguish saved loans in the earlier-lender and rental pickers. This is not imported from the Form 1098. Renaming a label keeps every loan selection.
    #[libertas_hidden]
    #[libertas_default("")]
    pub property: String,
    /// What the box 4 refund repays
    /// Shown when box 4 reports a refund. Check the lender’s statement and the earlier return to identify its treatment.
    /// [SetText]
    /// The box 4 refund repays
    #[libertas_hidden]
    #[libertas_constrained_by("^<MortgageMore>.allowed_refund_origins")]
    pub refund_origin: Option<crate::MortgageRefundOrigin>,
    /// Lender
    #[libertas_default(Institution)]
    pub lender: crate::MortgageLender,
    /// Loan facts or the earlier lender
    #[libertas_hidden]
    #[libertas_default(FirstLender)]
    pub details: MortgageDetails,
}
impl Default for MortgageMore {
    fn default() -> Self {
        Self {
            allowed_refund_origins: Vec::new(),
            property: String::new(),
            refund_origin: None,
            lender: crate::MortgageLender::Institution,
            details: MortgageDetails::FirstLender(MortgageFirstLender::default()),
        }
    }
}
