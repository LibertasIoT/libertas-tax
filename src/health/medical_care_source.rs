use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Qualification of this long-term-care insurance contract
/// Use the insurer's contract classification and actual issue/change history.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
pub enum CareContractQualification {
    /// Meets section 7702B(b)'s qualified-contract requirements
    Qualified,
    /// Contract issued before 1997 under the state's then-applicable LTC law
    Grandfathered {
        /// Actual original issue date
        #[libertas_date_only]
        // IRC 7702B(f): grandfathered LTC contracts were issued before January 1, 1997.
        #[libertas_number(min = 10101, max = 19961231)]
        issued: u32,
        /// Met the LTC insurance law of the state where issued at that time
        /// Check the insurer’s original contract and classification statement.
        state_compliant: bool,
        /// Contract has since been materially changed
        /// Check the insurer’s amendments and current classification statement.
        /// A material change can end grandfathered treatment. If the changed contract independently satisfies section 7702B(b), choose Qualified instead.
        materially_changed: bool,
    },
    /// Does not qualify for the qualified-LTC premium allowance
    Nonqualified,
}
impl CareContractQualification {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::Grandfathered { issued, .. } => {
                let year = issued / 10000;
                let month = issued / 100 % 100;
                let day = issued % 100;
                (1..=1996).contains(&year)
                    && (1..=crate::dates::days_in_month(year, month)).contains(&day)
            }
            _ => true,
        }
    }
    pub(crate) fn qualified(&self) -> bool {
        self.valid()
            && matches!(
                self,
                Self::Qualified
                    | Self::Grandfathered { state_compliant: true, materially_changed: false, .. }
            )
    }
}

/// Personal qualified long-term-care premiums
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MedicalCarePremium {
    /// Insured person
    /// The person's identity on the people page: -1 for you, -2 for your spouse, otherwise the dependent's identity. A person is told apart by this, never by name.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub person: i64,
    /// Unreimbursed after-tax premiums paid in 2026
    /// Enter full premiums before the age limit. Exclude employer-paid or tax-free reimbursed amounts, HSA/FSA payments, and amounts already reported in business or public-safety coverage. Enter only premiums eligible for your medical deduction for yourself, your spouse or a medical-expense dependent.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub premiums: i64,
    /// Actual contract qualification
    /// Nonqualified contracts receive no qualified-LTC premium benefit; known income and other independent source facts remain retained.
    pub qualified: CareContractQualification,
}
impl MedicalCarePremium {
    #[allow(dead_code)] // Retain the existing Boolean validation API beside its typed owner.
    pub(crate) fn valid(&self) -> bool {
        self.issue().is_none()
    }
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        let field = if self.person == 0 {
            "person"
        } else if !self.qualified.valid() {
            "qualified"
        } else if self.premiums < 0 {
            "premiums"
        } else {
            return None;
        };
        Some(medical_source_invalid().at_field(field))
    }
}

fn medical_source_invalid() -> crate::validation::ValidationIssue {
    crate::validation::ValidationIssue::new(
        "ISSUE_INTERVIEW_REVIEW_THE_ACCEPTED_PAGE_VALUES_BEFORE_65B6D7B7",
    )
}

// These stages also serve the original stored-validation positions: LTC
// precedes the other tax fields, while medical expenses follow those fields.
pub(crate) fn medical_care_issue(
    premiums: &[MedicalCarePremium],
) -> Option<crate::validation::ValidationIssue> {
    premiums.iter().enumerate().find_map(|(index, premium)| {
        premium.issue().map(|issue| issue.in_list("long_term_care", index))
    })
}

pub(crate) fn medical_expenses_issue(expenses: i64) -> Option<crate::validation::ValidationIssue> {
    (expenses < 0).then(|| medical_source_invalid().at_field("expenses"))
}

pub(crate) fn medical_source_amount(
    expenses: i64,
    premiums: &[MedicalCarePremium],
) -> Result<i64, crate::validation::ValidationIssue> {
    if let Some(issue) = medical_care_issue(premiums).or_else(|| medical_expenses_issue(expenses)) {
        return Err(issue);
    }
    // Known source cents, checked only after the actual first source failure.
    crate::kernel::ratio::checked_sum(
        core::iter::once(expenses).chain(premiums.iter().map(|premium| premium.premiums)),
    )
    .ok_or_else(|| medical_source_invalid().at_record())
}
