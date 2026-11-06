// Coverage: Employee pay without a W-2 — Form 8919 and actual payroll coverage.
//   Ordinary both-covered employee pay with an actual A/C/G/H reason uses Form8919.
//   Domestic state/local positions use their actual independent SS/Medicare coverage;
//   other coverage uses an unnumbered payroll worksheet, not fabricated Form8919 lines.
//   Valid totalization certificates establish the exemption for this pay only.
//   RRTA source amounts reuse W2Payroll and Form8959; unpaid ordinary RRTA and CT2
//   remain explicit limitations. Foreign government employment is never inferred exempt.
//   Sources: 2026 draft f8919; IRC3101; 26CFR31.3102-1(d); i8959; IRS state/local
//   coverage and totalization guidance. Printing/e-file are outside this App's scope.
use libertas_macros::*;

/// Employee pay without a W-2
/// Enter employee pay once, including its actual payroll system and coverage. Not pay already on a W-2, tips, or independent-contractor receipts.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into source_issuer and source_recipient, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Check the document's tax year against the supplied year and report a mismatch. For an actual Form 8919, read one firm's row: the firm in columns (a) and (b) is the issuer, the filer named in the header is the recipient, column (f) is wages and column (e) supplies form1099. Fill payroll from the actual reason in column (c) and its required date only when the supplied records establish both; do not invent a filing or determination date. For a supplied Form 1099-NEC read box 1 as wages; for Form 1099-MISC read box 3 as wages; a blank required amount is an error. Read box 4 federal income tax withholding; a clearly blank withholding box is zero. Set form1099 true for either 1099. A 1099 alone does not establish employee status, payroll coverage or a Form 8919 reason. Fill payroll only from the payroll system and coverage established by the supplied document; use Unavailable when those facts are not settled. Do not infer exemption from foreign work or a government employer. Report illegible or conflicting printed values as errors. Copies of the same form are one document.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.source_issuer.federal_id,@.source_recipient.federal_id,@.wages,@.withholding",
    form_date_order = "MonthDayYear"
)]
pub struct EmployeeWages {
    /// Source document issuer as printed
    #[libertas_llm_doc_field]
    pub source_issuer: crate::PrintedParty,
    /// Source document recipient as printed
    #[libertas_llm_doc_field]
    pub source_recipient: crate::PrintedParty,
    /// Taxable employee pay not included on a W-2
    /// Enter this income once. For Form 8919, use column (f), and with reason H enter only the omitted 1099 pay that belonged on the firm's W-2. Railroad compensation and actual payroll taxes are separate source amounts below. Do not also enter this pay as business receipts.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub wages: i64,
    /// Federal income tax withheld from this pay
    /// Including backup withholding; not Social Security or Medicare tax.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// You received a Form 1099-NEC or 1099-MISC from this firm for this pay
    /// Form 8919 column (e).
    #[libertas_llm_doc_field]
    pub form1099: bool,
    /// Actual payroll system and coverage for this pay
    /// Use employer records, the State Social Security Administrator's position determination, or the actual certificate. Working abroad or a government employer does not establish exemption.
    #[libertas_llm_doc_field]
    #[libertas_default(Unavailable)]
    pub payroll: EmployeePayrollSource,
}

/// Form 8919 filing reason
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum EmployeeWageReason {
    /// IRS employee determination after Form SS-8 — code A
    Ss8Determination {
        /// Date of IRS determination
        #[libertas_date_only]
        date: u32,
    },
    /// Other IRS correspondence establishing employee status — code C
    IrsCorrespondence {
        /// Date of IRS correspondence
        #[libertas_date_only]
        date: u32,
    },
    /// Form SS-8 filed; IRS reply pending — code G
    /// Form SS-8 must be filed separately on or before the return filing date. A pending request does not guarantee IRS agreement with employee status.
    Ss8Pending {
        /// Date Form SS-8 was filed
        #[libertas_date_only]
        filed_on: u32,
    },
    /// Same firm issued W-2 and 1099 for employee payments — code H
    /// The additional 1099-MISC/NEC pay should have been wages on that firm's W-2. Enter only the omitted amount here. Do not file Form SS-8 for this reason.
    SameFirmForms,
}

/// Actual Social Security or Medicare coverage of this position
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum EmployeeCoverage {
    /// The coverage determination is unavailable
    Unavailable,
    /// The position is not covered by this tax
    NotCovered,
    /// The position is covered by this tax
    Covered,
}
impl EmployeeCoverage {
    pub(crate) fn fact(self) -> crate::kernel::source_fact::SourceFact<bool> {
        use crate::kernel::source_fact::SourceFact;
        match self {
            Self::Unavailable => SourceFact::Unavailable,
            Self::NotCovered => SourceFact::Reported(false),
            Self::Covered => SourceFact::Reported(true),
        }
    }
}

/// Actual payroll treatment established for this employee pay
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum EmployeePayrollSource {
    /// Payroll system or coverage is unavailable
    Unavailable,
    /// Both Social Security and Medicare apply; an actual Form 8919 reason applies
    Form8919 { reason: EmployeeWageReason },
    /// Domestic state or local government position
    /// Use the actual position determination from your State Social Security Administrator. This is not foreign-government or international-organization employment.
    Government {
        /// Actual Social Security coverage
        #[libertas_default(Unavailable)]
        social: EmployeeCoverage,
        /// Actual Medicare coverage
        #[libertas_default(Unavailable)]
        medicare: EmployeeCoverage,
        /// Actual Form 8919 reason, if one applies
        /// No reason applies / A listed reason applies. A section 218 agreement alone is not a reason. Only both-covered pay with a reason uses Form 8919.
        /// [NullText]
        /// No Form 8919 reason applies
        /// [SetText]
        /// A listed Form 8919 reason applies
        #[libertas_default($null)]
        reason: Option<EmployeeWageReason>,
    },
    /// Railroad employee compensation subject to RRTA
    /// Use actual payroll records. Additional Medicare Tax is calculated; ordinary RRTA collection is outside this estimate and remains an explicit limitation.
    Railroad {
        /// Actual railroad compensation and tax amounts
        reported: EmployeeRailroadSource,
    },
    /// Actual totalization certificate establishes exemption for all this pay
    /// Select only with a valid foreign-system certificate of coverage covering this employment and period and establishing exemption from U.S. Social Security and Medicare. Foreign work or foreign tax payment alone is insufficient.
    TotalizationCertificate,
    /// Foreign government or international organization employment
    /// Employee FICA exemption does not establish exemption from U.S. self-employment tax. This employment's special payroll obligation remains outside the estimate.
    ForeignGovernment,
    /// Railroad employee representative compensation (Form CT-2)
    /// The employee representative's ordinary RRTA tax is outside this estimate. Actual compensation still supplies the separate Additional Medicare calculation.
    RailroadRepresentative { reported: EmployeeRailroadSource },
}

/// Actual railroad payroll source amounts
/// Compensation can differ from taxable employee income. Enter only established
/// amounts; a missing tax amount is unavailable, never inferred from compensation.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EmployeeRailroadSource {
    /// Actual RRTA compensation subject to Medicare
    #[libertas_default(Unavailable)]
    pub compensation: crate::SourceCents,
    /// Actual Tier 1 tax withheld
    #[libertas_default(Unavailable)]
    pub tier_1: crate::SourceCents,
    /// Actual Tier 2 tax withheld
    #[libertas_default(Unavailable)]
    pub tier_2: crate::SourceCents,
    /// Actual ordinary Medicare tax withheld
    #[libertas_default(Unavailable)]
    pub medicare: crate::SourceCents,
    /// Actual Additional Medicare Tax withheld
    #[libertas_default(Unavailable)]
    pub additional_medicare: crate::SourceCents,
}
impl EmployeeRailroadSource {
    pub(crate) fn known(&self) -> [Option<i64>; 5] {
        [&self.compensation, &self.tier_1, &self.tier_2, &self.medicare, &self.additional_medicare]
            .map(|source| {
                source
                    .source_fact()
                    .ok()
                    .and_then(|fact| fact.require(|| ()).ok())
                    .map(|value| value.unwrap_or(0))
            })
    }
    fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        [
            (&self.compensation, "compensation"),
            (&self.tier_1, "tier_1"),
            (&self.tier_2, "tier_2"),
            (&self.medicare, "medicare"),
            (&self.additional_medicare, "additional_medicare"),
        ]
        .into_iter()
        .find_map(|(source, field)| {
            source.source_fact().err().map(|_| {
                crate::validation::ValidationIssue::new("EMPLOYEE_RAILROAD_SOURCE_INVALID")
                    .at_field("amount")
                    .in_field(field)
            })
        })
    }
    // Mechanical known-component bridge to the existing RRTA accumulator.
    // Callers retain the independent known() flags; zeros here are only the
    // known subtotal and never establish absence of an unavailable component.
    pub(crate) fn known_payroll(&self) -> crate::W2Payroll {
        let [compensation, tier_1, tier_2, medicare, additional_medicare] =
            self.known().map(|amount| amount.unwrap_or(0));
        crate::W2Payroll::Railroad { compensation, tier_1, tier_2, medicare, additional_medicare }
    }
}

impl EmployeeWages {
    pub(crate) fn reason(&self) -> Option<&EmployeeWageReason> {
        match &self.payroll {
            EmployeePayrollSource::Form8919 { reason } => Some(reason),
            EmployeePayrollSource::Government {
                social: EmployeeCoverage::Covered,
                medicare: EmployeeCoverage::Covered,
                reason,
            } => reason.as_ref(),
            _ => None,
        }
    }
    pub(crate) fn coverage(&self) -> [Option<bool>; 2] {
        match &self.payroll {
            EmployeePayrollSource::Form8919 { .. } => [Some(true); 2],
            EmployeePayrollSource::Government { social, medicare, .. } => {
                [*social, *medicare].map(|value| value.fact().require(|| ()).ok().flatten())
            }
            EmployeePayrollSource::TotalizationCertificate => [Some(false); 2],
            _ => [None; 2],
        }
    }
    pub(crate) fn railroad(&self) -> Option<&EmployeeRailroadSource> {
        match &self.payroll {
            EmployeePayrollSource::Railroad { reported }
            | EmployeePayrollSource::RailroadRepresentative { reported } => Some(reported),
            _ => None,
        }
    }
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        if matches!(self.reason(), Some(EmployeeWageReason::SameFirmForms)) && !self.form1099 {
            Some(
                crate::validation::ValidationIssue::new("FORM_8919_CODE_H_NEEDS_1099")
                    .at_field("form1099"),
            )
        } else if self.withholding > self.wages {
            Some(
                crate::validation::ValidationIssue::new("FORM_8919_WITHHOLDING_ABOVE_PAY")
                    .at_field("withholding"),
            )
        } else if let Some(payroll) = self.railroad() {
            payroll.issue().map(|issue| issue.in_field("reported").in_field("payroll"))
        } else {
            None
        }
    }
    pub(crate) fn valid(&self) -> bool {
        self.wages >= 0 && self.withholding >= 0 && self.issue().is_none()
    }
    pub(crate) fn income(&self) -> Result<i64, &'static str> {
        if !self.valid() {
            return Err(crate::fixture_english!(
                "Enter valid employee pay, withholding and actual payroll coverage; reason H needs the Form 1099 you received."
            ));
        }
        Ok(self.wages)
    }
}
