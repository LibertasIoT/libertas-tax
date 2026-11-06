// Coverage: T3-982 Canceled debt topic page types (2026-09-27). The Form 1099-C kind
//   (CanceledDebtDocument::Cancel1099C) maps onto
//   FederalIncome::PersonalDebt (PersonalDebtCancellation) and the Form 1099-A kind
//   (CanceledDebtDocument::Acquisition1099A) onto FederalIncome::PropertyAcquisition, both ways
//   (record / from_record; reopening a saved page gives the same page). The creditor's or
//   lender's name is the entry label. Printed boxes in box order: 1099-C boxes 1 through 7 (the
//   PropertyTaken fold: None exactly when box 7 is blank); 1099-A boxes 1, 2, 4, 5 and 6. Filer facts
//   stay apart: what the property was (TransferredProperty) and why the debt is or is not taxable
//   (CanceledDebtChoice). Each CanceledDebtChoice branch is its stored treatment; Something
//   else writes PersonalDebtTreatment::Unsupported with the named situation, refused by name in
//   debt.rs. Original strict topic rules (Cancel1099C::issue, same keys):
//   DEBT_LIABILITIES_BELOW_CANCELED, DEBT_PRICE_REDUCTION_ABOVE_BASIS,
//   DEBT_PRICE_REDUCTION_ABOVE_LIABILITIES, DEBT_SALE_BEFORE_REDUCTION,
//   DEBT_PRINCIPAL_BELOW_CANCELED, DEBT_ELECTION_ABOVE_CANCELED. Tests: src/tests/home/interview.rs
//   canceled_debt_*.
use crate::{
    DebtInsolvency, DebtProperty, DebtPropertySale, FarmDebtExclusion, PersonalBankruptcy,
    PersonalDebtCancellation, PersonalDebtTreatment, PersonalInsolvency, PropertyAcquisition,
    PropertyTransfer, RealPropertyDebtExclusion, StudentDischargeReason, TransferredProperty,
    UnsupportedDebt,
};
use alloc::{string::String, vec::Vec};
use libertas_macros::*;

/// Form 1099-C
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read the calendar year and the CORRECTED checkbox, then each printed party block into creditor, debtor. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read the 1099-C exactly. Check its tax year against the supplied year; report a mismatch. Copy the account number exactly as printed, using an empty string when blank. Boxes 1 and 2 must show values; report a blank one as an error. Return the box 1 date and box 2 amount as printed. Copy box 3 interest as null when blank, including a printed zero as zero; copy box 4 description exactly, using an empty string when blank. For box 6, select the declared identifiable-event code matching the printed letter A through H; return null when blank. Report an unrecognized or illegible code as an error, and never infer it from other boxes. Return true for box 5 only when it is checked. Return null for box 7 when it is blank; otherwise return the box 7 fair market value. Report illegible or conflicting values as errors. Do not infer what the property was, whether the canceled debt is income, insolvency, bankruptcy, a shared debt, or other facts absent from the 1099-C. The file may hold several copies of this form, such as Copy B and Copy 2; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.creditor.federal_id,@.debtor.federal_id,@.calendar_year,@.date,@.amount,@.interest,@.property_value,@.account_number",
    form_date_order = "MonthDayYear"
)]
pub struct Form1099C {
    /// For calendar year
    #[libertas_llm_doc_field]
    #[libertas_default(0)]
    pub calendar_year: u32,
    /// CORRECTED (if checked)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub corrected: bool,
    /// CREDITOR'S name, TIN and address
    #[libertas_llm_doc_field]
    pub creditor: crate::PrintedParty,
    /// DEBTOR'S name, TIN and address
    #[libertas_llm_doc_field]
    pub debtor: crate::PrintedParty,
    /// Account number (see instructions)
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub account_number: String,
    /// Box 1 date of identifiable event
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    // Tax-year source window: only the stated current or prior tax years apply.
    #[libertas_number(min = 20260101, max = 20261231)]
    pub date: u32,
    /// Box 2 amount of debt discharged
    /// If someone else was also liable and got a form for the whole debt, enter only your share.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
    /// Box 3 interest, if included in box 2
    /// [NullText]
    /// Box 3 is blank
    /// [SetText]
    /// The printed box 3 interest is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub interest: Option<i64>,
    /// Box 4 debt description
    #[libertas_llm_doc_field]
    #[libertas_default("")]
    pub debt_description: String,
    /// Box 5 check here if the debtor was personally liable for repayment of the debt
    #[libertas_llm_doc_field]
    #[libertas_default(true)]
    pub liable: bool,
    /// Box 6 identifiable event code
    /// [NullText]
    /// Box 6 is blank
    /// [SetText]
    /// The printed box 6 identifiable event code is
    #[libertas_llm_doc_field]
    #[libertas_default($null)]
    pub event_code: Option<Form1099CEventCode>,
    /// Box 7 fair market value of property
    /// [NullText]
    /// Box 7 is blank
    /// [SetText]
    /// The printed box 7 fair market value is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub property_value: Option<i64>,
}

// IRS box 6 codes and labels: https://www.irs.gov/pub/irs-pdf/f1099c.pdf
// Event definitions: https://www.irs.gov/instructions/i1099ac
/// Box 6 identifiable event code
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum Form1099CEventCode {
    /// A — Bankruptcy
    #[libertas_form_code("A")]
    Bankruptcy,
    /// B — Other judicial debt relief
    #[libertas_form_code("B")]
    OtherJudicialDebtRelief,
    /// C — Statute of limitations or expiration of deficiency period
    #[libertas_form_code("C")]
    StatuteOfLimitations,
    /// D — Foreclosure election
    #[libertas_form_code("D")]
    ForeclosureElection,
    /// E — Debt relief from probate or similar proceeding
    #[libertas_form_code("E")]
    ProbateDebtRelief,
    /// F — By agreement
    #[libertas_form_code("F")]
    Agreement,
    /// G — Decision or policy to discontinue collection
    #[libertas_form_code("G")]
    DiscontinueCollection,
    /// H — Other actual discharge before identifiable event
    #[libertas_form_code("H")]
    OtherActualDischarge,
}

impl Form1099CEventCode {
    pub const fn printed_code(&self) -> &'static str {
        match self {
            Self::Bankruptcy => "A",
            Self::OtherJudicialDebtRelief => "B",
            Self::StatuteOfLimitations => "C",
            Self::ForeclosureElection => "D",
            Self::ProbateDebtRelief => "E",
            Self::Agreement => "F",
            Self::DiscontinueCollection => "G",
            Self::OtherActualDischarge => "H",
        }
    }
}

/// Accepted Form 1099-C
/// The printed form and the topic answers it calls for, kept apart.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PersonalDebtSource {
    /// Form 1099-C as printed
    pub form: Form1099C,
    /// Is the canceled debt income?
    /// [NullText]
    /// The cancellation treatment is not established
    /// [SetText]
    /// The cancellation treatment is
    #[libertas_default($null)]
    pub treatment: Option<CanceledDebtChoice>,
    /// What the property was
    /// Asked only when box 7 shows a value.
    /// [NullText]
    /// The property is not established
    /// [SetText]
    /// The property was
    #[libertas_default($null)]
    pub property: Option<TransferredProperty>,
}

impl PersonalDebtSource {
    /// A newly entered form: no topic answer is given yet.
    pub(crate) fn printed(form: Form1099C) -> Self {
        Self { form, treatment: None, property: None }
    }
    /// The topic answers of an accepted source survive a correction of its paper,
    /// including a property answer while box 7 is blank: the topic then asks for
    /// the box 7 value again (DEBT_PROPERTY_FMV_NEEDED) instead of losing it.
    pub(crate) fn keep_answers(&mut self, accepted: &Self) {
        self.treatment = accepted.treatment.clone();
        self.property = accepted.property.clone();
    }
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        if self.form.property_value.is_none() && self.property.is_some() {
            return Some(
                crate::validation::ValidationIssue::new("DEBT_PROPERTY_FMV_NEEDED")
                    .at_field("property"),
            );
        }
        Cancel1099C::from(self).issue()
    }
}

// One-way: every printed box either reaches the internal record or is named as
// not read by any calculation, so a new box cannot be forgotten.
impl From<&PersonalDebtSource> for Cancel1099C {
    fn from(source: &PersonalDebtSource) -> Self {
        let PersonalDebtSource { form, treatment, property } = source;
        let Form1099C {
            // Printed year and CORRECTED box; no figure reads them.
            calendar_year: _,
            corrected: _,
            creditor,
            debtor,
            // The account number names the document; no figure reads it.
            account_number: _,
            date,
            amount,
            interest,
            debt_description,
            liable,
            event_code,
            property_value,
        } = form;
        Cancel1099C {
            creditor: creditor.clone(),
            debtor: debtor.clone(),
            date: *date,
            amount: *amount,
            interest: *interest,
            debt_description: debt_description.clone(),
            liable: *liable,
            event_code: event_code
                .as_ref()
                .map(Form1099CEventCode::printed_code)
                .unwrap_or("")
                .into(),
            property: match property_value {
                None => PropertyTaken::None,
                Some(value) => PropertyTaken::Taken { value: *value, property: property.clone() },
            },
            treatment: treatment.clone(),
        }
    }
}
impl From<&PersonalDebtSource> for PersonalDebtCancellation {
    fn from(source: &PersonalDebtSource) -> Self {
        Self {
            transfer: source
                .property
                .as_ref()
                .map(|property| PropertyTransfer { property: Some(property.clone()) }),
            ..Self::from(&Cancel1099C::from(source))
        }
    }
}

/// The internal record of a Form 1099-C: the paper and its topic answers as the calculation
/// and the rules read them (see PersonalDebtSource).
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Cancel1099C {
    /// CREDITOR'S name, TIN and address
    #[libertas_llm_doc_field]
    pub creditor: crate::PrintedParty,
    /// DEBTOR'S name, TIN and address
    #[libertas_llm_doc_field]
    pub debtor: crate::PrintedParty,
    /// Box 1 date of identifiable event
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    // Tax-year source window: only the stated current or prior tax years apply.
    #[libertas_number(min = 20260101, max = 20261231)]
    pub date: u32,
    /// Box 2 amount of debt discharged
    /// If someone else was also liable and got a form for the whole debt, enter only your share.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
    /// Box 3 interest included in box 2
    /// [NullText]
    /// Box 3 is blank
    /// [SetText]
    /// The printed box 3 interest is
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub interest: Option<i64>,
    /// Box 4 description of debt
    #[libertas_llm_doc_field]
    pub debt_description: String,
    /// Box 5 you were personally liable for repayment of the debt
    #[libertas_llm_doc_field]
    #[libertas_default(true)]
    pub liable: bool,
    /// Box 6 identifiable event code
    #[libertas_llm_doc_field]
    pub event_code: String,
    /// Box 7 fair market value of property
    #[libertas_llm_doc_field]
    #[libertas_default(None)]
    pub property: PropertyTaken,
    /// Is the canceled debt income?
    /// [NullText]
    /// The cancellation treatment is not established
    /// [SetText]
    /// The cancellation treatment is
    #[libertas_hidden]
    #[libertas_default($null)]
    pub treatment: Option<CanceledDebtChoice>,
}

/// Box 7 fair market value of property
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum PropertyTaken {
    /// Box 7 is blank
    None,
    /// Box 7 shows a value
    /// Copy the printed value only. The personal disposition is answered on the income topic.
    Taken {
        /// Box 7 fair market value of property
        #[libertas_llm_doc_field(allow_empty = false)]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        value: i64,
        /// What the property was
        /// [NullText]
        /// The property is not established
        /// [SetText]
        /// The property was
        #[libertas_hidden]
        #[libertas_default($null)]
        property: Option<TransferredProperty>,
    },
}

/// Is the canceled debt income?
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CanceledDebtChoice {
    /// Yes, it is income
    /// A personal debt: you were solvent, not in bankruptcy, and nothing below applies.
    Taxable,
    /// No, you were insolvent
    /// Your debts were more than the value of everything you owned immediately before the cancellation. Choosing this confirms the snapshots and the property list, completed after the rest of your 2026 return.
    Insolvent {
        /// Total liabilities immediately before the cancellation
        /// Include this debt and every other debt you owed, with your share of joint debts.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        liabilities: i64,
        /// Fair market value of everything you owned immediately before
        /// Include cash, retirement accounts, exempt assets and property that secured the debt.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        assets: i64,
        /// Tax basis of your property plus money immediately after
        /// Not its value: what you paid, adjusted for depreciation and improvements.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        basis_and_cash_after: i64,
        /// Alternative minimum tax basis plus money immediately after
        /// Leave empty when it equals the regular basis.
        /// [NullText]
        /// AMT basis plus money equals the regular amount
        /// [SetText]
        /// Different AMT basis plus money is
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default($null)]
        amt_basis_and_cash_after: Option<i64>,
        /// It was a business, farm or rental debt
        /// Any part above your insolvency is then business income, which is not calculated.
        #[libertas_default(false)]
        business_debt: bool,
        /// Property you hold on January 1, 2027
        /// Every asset except money, once each, with its adjusted basis. The excluded debt reduces its basis.
        #[libertas_unordered]
        /// ----
        /// Property
        property: Vec<DebtProperty>,
        /// Amount you elect to apply first to depreciable property
        /// Form 982 line 5. Leave zero unless you make this election.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        depreciable_first: i64,
    },
    /// No, discharged in a chapter 12 or 13 bankruptcy
    /// The court or its approved plan discharged the debt. Choosing this confirms the discharge, the snapshots and the property list.
    Bankruptcy {
        /// Tax basis of your property plus money immediately after
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        basis_and_cash_after: i64,
        /// Alternative minimum tax basis plus money immediately after
        /// Leave empty when it equals the regular basis.
        /// [NullText]
        /// AMT basis plus money equals the regular amount
        /// [SetText]
        /// Different AMT basis plus money is
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default($null)]
        amt_basis_and_cash_after: Option<i64>,
        /// Liabilities remaining immediately after the discharge
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        liabilities_after: i64,
        /// Property you hold on January 1, 2027
        /// Every asset except money, once each, with its adjusted basis.
        #[libertas_unordered]
        /// ----
        /// Property
        property: Vec<DebtProperty>,
        /// Amount you elect to apply first to depreciable property
        /// Form 982 line 5. Leave zero unless you make this election.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        depreciable_first: i64,
    },
    /// No, a qualifying student-loan discharge
    /// Choosing this confirms the servicer's final discharge determination.
    StudentLoan {
        /// Reason in the final discharge determination
        reason: StudentDischargeReason,
    },
    /// No, a gift or inheritance
    /// The creditor forgave the debt as a genuine gift or through an estate, not as a business write-off.
    Gift,
    /// No, the seller reduced the price of property you bought
    /// The original seller of personal-use property you still owned reduced the debt you owed for it, while you were solvent.
    SellerReduction {
        /// Property's adjusted basis immediately before the reduction
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        basis: i64,
        /// Fair market value of everything you owned immediately before
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        assets: i64,
        /// Total liabilities immediately before, including this debt
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        liabilities: i64,
        /// Sold later in 2026
        /// Leave empty if you still owned it at the end of 2026.
        /// [NullText]
        /// Still owned the property at the end of 2026
        /// [SetText]
        /// The later 2026 sale is
        sale: Option<DebtPropertySale>,
    },
    /// No, qualified farm debt
    /// Debt from operating your farm, canceled by a lender in the business of lending or a government agency, with at least half your 2023–2025 gross receipts from farming. Choosing this confirms those facts.
    FarmDebt {
        /// Were you insolvent immediately before?
        #[libertas_default(Solvent)]
        solvency: DebtSolvency,
        /// Property you hold on January 1, 2027
        /// Every asset except money, once each, with its adjusted basis.
        #[libertas_unordered]
        /// ----
        /// Property
        property: Vec<DebtProperty>,
        /// Amount you elect to apply first to depreciable property
        /// Form 982 line 5. Leave zero unless you make this election.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        depreciable_first: i64,
    },
    /// No, qualified real property business debt
    /// A loan to buy, build or improve real property used in your business, secured by it, that you elect to exclude. Choosing this confirms the election.
    RealPropertyBusiness {
        /// Were you insolvent immediately before?
        #[libertas_default(Solvent)]
        solvency: DebtSolvency,
        /// Outstanding principal of this debt immediately before
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        principal: i64,
        /// Fair market value of the property securing it
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        value: i64,
        /// Other qualifying business debt secured by that property
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        other_secured: i64,
        /// Adjusted basis of all your depreciable real property immediately before
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        depreciable_basis: i64,
        /// Property you hold on January 1, 2027
        /// Every asset except money, once each, with its adjusted basis. Mark the building that secured the debt.
        #[libertas_unordered]
        /// ----
        /// Property
        property: Vec<DebtProperty>,
        /// Amount you elect to apply first to depreciable property
        /// Form 982 line 5, for a part excluded because you were insolvent. Leave zero unless you make this election.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        depreciable_first: i64,
    },
    /// Something else applies
    SomethingElse {
        /// Situation
        situation: UnsupportedDebt,
    },
}

/// Were you insolvent immediately before?
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum DebtSolvency {
    /// No
    Solvent,
    /// Yes
    /// The insolvency exclusion applies first.
    Insolvent {
        /// Total liabilities immediately before the cancellation
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        liabilities: i64,
        /// Fair market value of everything you owned immediately before
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        assets: i64,
        /// Tax basis of your property plus money immediately after
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        basis_and_cash_after: i64,
        /// Alternative minimum tax basis plus money immediately after
        /// Leave empty when it equals the regular basis.
        /// [NullText]
        /// AMT basis plus money equals the regular amount
        /// [SetText]
        /// Different AMT basis plus money is
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default($null)]
        amt_basis_and_cash_after: Option<i64>,
    },
}
impl DebtSolvency {
    fn record(&self) -> Option<DebtInsolvency> {
        match self {
            Self::Solvent => None,
            Self::Insolvent {
                liabilities,
                assets,
                basis_and_cash_after,
                amt_basis_and_cash_after,
            } => Some(DebtInsolvency {
                liabilities: *liabilities,
                assets: *assets,
                basis_and_cash_after: *basis_and_cash_after,
                amt_basis_and_cash_after: *amt_basis_and_cash_after,
            }),
        }
    }
    fn liabilities(&self) -> Option<i64> {
        match self {
            Self::Solvent => None,
            Self::Insolvent { liabilities, .. } => Some(*liabilities),
        }
    }
}

impl Cancel1099C {
    // The client rules' server twins, in declaration order.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        let Some(t) = self.treatment.as_ref() else {
            return Some(
                crate::validation::ValidationIssue::new("DEBT_TREATMENT_NEEDED")
                    .at_field("treatment"),
            );
        };
        if matches!(self.property, PropertyTaken::Taken { property: None, .. }) {
            return Some(
                crate::validation::ValidationIssue::new("DEBT_PROPERTY_NEEDED")
                    .at_field("property")
                    .in_field("property"),
            );
        }

        let amount = self.amount;
        let solvency = match t {
            CanceledDebtChoice::FarmDebt { solvency, .. }
            | CanceledDebtChoice::RealPropertyBusiness { solvency, .. } => solvency.liabilities(),
            CanceledDebtChoice::Insolvent { liabilities, .. } => Some(*liabilities),
            _ => None,
        };
        if solvency.is_some_and(|v| v < amount) {
            let issue = crate::validation::ValidationIssue::new("DEBT_LIABILITIES_BELOW_CANCELED")
                .at_field("liabilities");
            return Some(match t {
                CanceledDebtChoice::FarmDebt { .. }
                | CanceledDebtChoice::RealPropertyBusiness { .. } => {
                    issue.in_field("solvency").in_field("treatment")
                }
                _ => issue.in_field("treatment"),
            });
        }
        match t {
            CanceledDebtChoice::SellerReduction { basis, .. } if *basis < amount => {
                return Some(
                    crate::validation::ValidationIssue::new("DEBT_PRICE_REDUCTION_ABOVE_BASIS")
                        .at_field("basis")
                        .in_field("treatment"),
                );
            }
            CanceledDebtChoice::SellerReduction { liabilities, .. } if *liabilities < amount => {
                return Some(
                    crate::validation::ValidationIssue::new(
                        "DEBT_PRICE_REDUCTION_ABOVE_LIABILITIES",
                    )
                    .at_field("liabilities")
                    .in_field("treatment"),
                );
            }
            CanceledDebtChoice::SellerReduction { sale: Some(sale), .. }
                if sale.date < self.date =>
            {
                return Some(
                    crate::validation::ValidationIssue::new("DEBT_SALE_BEFORE_REDUCTION")
                        .at_field("date")
                        .in_field("sale")
                        .in_field("treatment"),
                );
            }
            CanceledDebtChoice::RealPropertyBusiness { principal, .. } if *principal < amount => {
                return Some(
                    crate::validation::ValidationIssue::new("DEBT_PRINCIPAL_BELOW_CANCELED")
                        .at_field("principal")
                        .in_field("treatment"),
                );
            }
            _ => (),
        }
        match t {
            CanceledDebtChoice::Insolvent { depreciable_first, .. }
            | CanceledDebtChoice::Bankruptcy { depreciable_first, .. }
            | CanceledDebtChoice::FarmDebt { depreciable_first, .. }
            | CanceledDebtChoice::RealPropertyBusiness { depreciable_first, .. }
                if *depreciable_first > amount =>
            {
                Some(
                    crate::validation::ValidationIssue::new("DEBT_ELECTION_ABOVE_CANCELED")
                        .at_field("depreciable_first")
                        .in_field("treatment"),
                )
            }
            _ => None,
        }
    }
    fn record(&self) -> PersonalDebtCancellation {
        let treatment = self.treatment.as_ref().map(|treatment| match treatment {
            CanceledDebtChoice::Taxable => PersonalDebtTreatment::Taxable,
            CanceledDebtChoice::Insolvent {
                liabilities,
                assets,
                basis_and_cash_after,
                amt_basis_and_cash_after,
                business_debt,
                property,
                depreciable_first,
            } => PersonalDebtTreatment::Insolvent {
                data: PersonalInsolvency {
                    liabilities: *liabilities,
                    assets: *assets,
                    basis_and_cash_after: *basis_and_cash_after,
                    amt_basis_and_cash_after: *amt_basis_and_cash_after,
                    property: property.clone(),
                    depreciable_first: *depreciable_first,
                    business_debt: *business_debt,
                },
            },
            CanceledDebtChoice::Bankruptcy {
                basis_and_cash_after,
                amt_basis_and_cash_after,
                liabilities_after,
                property,
                depreciable_first,
            } => PersonalDebtTreatment::Bankruptcy {
                data: PersonalBankruptcy {
                    basis_and_cash_after: *basis_and_cash_after,
                    amt_basis_and_cash_after: *amt_basis_and_cash_after,
                    liabilities_after: *liabilities_after,
                    property: property.clone(),
                    depreciable_first: *depreciable_first,
                },
            },
            CanceledDebtChoice::StudentLoan { reason } => {
                PersonalDebtTreatment::StudentDischarge { reason: *reason }
            }
            CanceledDebtChoice::Gift => PersonalDebtTreatment::GiftOrBequest,
            CanceledDebtChoice::SellerReduction { basis, assets, liabilities, sale } => {
                PersonalDebtTreatment::PriceReduction {
                    basis: *basis,
                    assets: *assets,
                    liabilities: *liabilities,
                    sale: sale.clone(),
                }
            }
            CanceledDebtChoice::FarmDebt { solvency, property, depreciable_first } => {
                PersonalDebtTreatment::FarmDebt {
                    data: FarmDebtExclusion {
                        insolvency: solvency.record(),
                        property: property.clone(),
                        depreciable_first: *depreciable_first,
                    },
                }
            }
            CanceledDebtChoice::RealPropertyBusiness {
                solvency,
                principal,
                value,
                other_secured,
                depreciable_basis,
                property,
                depreciable_first,
            } => PersonalDebtTreatment::RealPropertyDebt {
                data: RealPropertyDebtExclusion {
                    insolvency: solvency.record(),
                    principal: *principal,
                    value: *value,
                    other_secured: *other_secured,
                    depreciable_basis: *depreciable_basis,
                    property: property.clone(),
                    depreciable_first: *depreciable_first,
                },
            },
            CanceledDebtChoice::SomethingElse { situation } => {
                PersonalDebtTreatment::Unsupported { situation: *situation }
            }
        });
        PersonalDebtCancellation {
            interest: self.interest,
            debt_description: self.debt_description.clone(),
            event_code: self.event_code.clone(),
            box7_fmv: match self.property {
                PropertyTaken::None => None,
                PropertyTaken::Taken { value, .. } => Some(value),
            },
            date: self.date,
            amount: self.amount,
            treatment,
            personally_liable: self.liable,
            transfer: match &self.property {
                PropertyTaken::None => None,
                PropertyTaken::Taken { property, .. } => property
                    .as_ref()
                    .map(|property| PropertyTransfer { property: Some(property.clone()) }),
            },
        }
    }
}

/// Form 1099-A
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read the calendar year and the CORRECTED checkbox, then each printed party block into lender, borrower. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read the 1099-A exactly. Check its tax year against the supplied year; report a mismatch. Copy the account number exactly as printed, using an empty string when blank. Boxes 1, 2 and 4 must show values; report a blank one as an error. Return the box 1 date and the box 2 and box 4 amounts as printed. Copy box 3 exactly as printed, using an empty string when blank. Copy the box 6 property description exactly, using an empty string when blank. Return true for box 5 only when it is checked. Report illegible or conflicting values as errors. Do not infer what the property was or other facts absent from the 1099-A. The file may hold several copies of this form, such as Copy B and Copy 2; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.lender.federal_id,@.borrower.federal_id,@.calendar_year,@.date,@.description,@.balance,@.value,@.account_number",
    form_date_order = "MonthDayYear"
)]
pub struct Form1099A {
    /// For calendar year
    #[libertas_llm_doc_field]
    #[libertas_default(0)]
    pub calendar_year: u32,
    /// CORRECTED (if checked)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub corrected: bool,
    /// LENDER'S name, TIN and address
    #[libertas_llm_doc_field]
    pub lender: crate::PrintedParty,
    /// BORROWER'S name, TIN and address
    #[libertas_llm_doc_field]
    pub borrower: crate::PrintedParty,
    /// Account number (see instructions)
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub account_number: String,
    /// Box 1 date of lender's acquisition or knowledge of abandonment
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    // Tax-year source window: only the stated current or prior tax years apply.
    #[libertas_number(min = 20260101, max = 20261231)]
    pub date: u32,
    /// Box 2 balance of principal outstanding
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub balance: i64,
    /// Box 3
    /// The form prints this box without a caption.
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub box3: String,
    /// Box 4 fair market value of property
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub value: i64,
    /// Box 5 check if the borrower was personally liable for repayment of the debt
    #[libertas_llm_doc_field]
    #[libertas_default(true)]
    pub liable: bool,
    /// Box 6 description of property
    #[libertas_llm_doc_field]
    #[libertas_default("")]
    pub description: String,
}

/// Accepted Form 1099-A
/// The printed form and the topic answer it calls for, kept apart.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PropertyAcquisitionSource {
    /// Form 1099-A as printed
    pub form: Form1099A,
    /// What the property was
    /// [NullText]
    /// The property is not established
    /// [SetText]
    /// The property was
    #[libertas_default($null)]
    pub property: Option<AcquiredProperty>,
}

impl PropertyAcquisitionSource {
    /// A newly entered form: no topic answer is given yet.
    pub(crate) fn printed(form: Form1099A) -> Self {
        Self { form, property: None }
    }
    /// The topic answer of an accepted source survives a correction of its paper.
    pub(crate) fn keep_answers(&mut self, accepted: &Self) {
        self.property = accepted.property.clone();
    }
}

// One-way: every printed box either reaches the internal record or is named as
// not read by any calculation, so a new box cannot be forgotten.
impl From<&PropertyAcquisitionSource> for Acquisition1099A {
    fn from(source: &PropertyAcquisitionSource) -> Self {
        let PropertyAcquisitionSource { form, property } = source;
        let Form1099A {
            // Printed year, CORRECTED box and the uncaptioned box 3; no figure reads them.
            calendar_year: _,
            corrected: _,
            box3: _,
            lender,
            borrower,
            // The account number names the document; no figure reads it.
            account_number: _,
            date,
            balance,
            value,
            liable,
            description,
        } = form;
        Acquisition1099A {
            lender: lender.clone(),
            borrower: borrower.clone(),
            date: *date,
            balance: *balance,
            value: *value,
            liable: *liable,
            description: description.clone(),
            property: property.clone(),
        }
    }
}
impl From<&PropertyAcquisitionSource> for PropertyAcquisition {
    fn from(source: &PropertyAcquisitionSource) -> Self {
        Self::from(&Acquisition1099A::from(source))
    }
}

/// The internal record of a Form 1099-A: the paper and its topic answer as the calculation
/// reads them (see PropertyAcquisitionSource).
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Acquisition1099A {
    /// LENDER'S name, TIN and address
    #[libertas_llm_doc_field]
    pub lender: crate::PrintedParty,
    /// BORROWER'S name, TIN and address
    #[libertas_llm_doc_field]
    pub borrower: crate::PrintedParty,
    /// Box 1 date of lender's acquisition or knowledge of abandonment
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    // Tax-year source window: only the stated current or prior tax years apply.
    #[libertas_number(min = 20260101, max = 20261231)]
    pub date: u32,
    /// Box 2 balance of principal outstanding
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub balance: i64,
    /// Box 4 fair market value of property
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub value: i64,
    /// Box 5 you were personally liable for repayment of the debt
    #[libertas_llm_doc_field]
    #[libertas_default(true)]
    pub liable: bool,
    /// Box 6 description of property
    #[libertas_llm_doc_field]
    pub description: String,
    /// What the property was
    /// [NullText]
    /// The property is not established
    /// [SetText]
    /// The property was
    #[libertas_hidden]
    #[libertas_default($null)]
    pub property: Option<AcquiredProperty>,
}
impl Acquisition1099A {
    fn record(&self) -> PropertyAcquisition {
        let (property, crop) = match &self.property {
            Some(AcquiredProperty::CccCrop { loan }) => {
                (Some(TransferredProperty::Business), Some(*loan))
            }
            Some(AcquiredProperty::Property { kind }) => (Some(kind.clone()), None),
            None => (None, None),
        };
        PropertyAcquisition {
            description: self.description.clone(),
            date: self.date,
            balance: self.balance,
            value: self.value,
            personally_liable: self.liable,
            property,
            crop,
        }
    }
}

/// What the property was
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum AcquiredProperty {
    /// A crop forfeited to the Commodity Credit Corporation
    CccCrop {
        /// Forfeited loan
        loan: i64,
    },
    /// Other property
    Property {
        /// What the property was
        kind: TransferredProperty,
    },
}

/// What the property was
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum AcquiredPropertyView {
    /// A crop forfeited to the Commodity Credit Corporation
    /// The crop pledged for one of the CCC loans entered on your farm or farm rental. The loan is counted there, not again here.
    CccCrop {
        /// Forfeited loan
        /// A loan marked forfeited in 2026 whose principal equals box 2. A farm entered on this same page is chosen after you save it.
        #[libertas_enum_source("$.ccc_lots")]
        loan: i64,
    },
    /// Other property
    Property {
        /// What the property was
        kind: TransferredProperty,
    },
}

impl From<&AcquiredProperty> for AcquiredPropertyView {
    fn from(property: &AcquiredProperty) -> Self {
        match property {
            AcquiredProperty::CccCrop { loan } => Self::CccCrop { loan: *loan },
            AcquiredProperty::Property { kind } => Self::Property { kind: kind.clone() },
        }
    }
}

impl From<AcquiredPropertyView> for AcquiredProperty {
    fn from(property: AcquiredPropertyView) -> Self {
        match property {
            AcquiredPropertyView::CccCrop { loan } => Self::CccCrop { loan },
            AcquiredPropertyView::Property { kind } => Self::Property { kind },
        }
    }
}

// One-way: the stored paper derives the record the calculations read.
impl From<&Cancel1099C> for PersonalDebtCancellation {
    fn from(paper: &Cancel1099C) -> Self {
        paper.record()
    }
}
impl From<&Acquisition1099A> for PropertyAcquisition {
    fn from(paper: &Acquisition1099A) -> Self {
        paper.record()
    }
}
