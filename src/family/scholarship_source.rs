use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Required scholarship services
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum ScholarshipService {
    /// No required teaching, research or other services
    None,
    /// National Health Service Corps Scholarship Program
    /// Confirm the award is under this specific program, not a loan repayment or other health-service program.
    NationalHealthServiceCorps,
    /// Armed Forces Health Professions Scholarship and Financial Assistance Program
    /// Confirm the named program. Do not include military wages or duplicate amounts reported on Form W-2.
    ArmedForcesHealthProfessions,
    /// Qualifying work-college work-learning-service program
    /// Confirm a comprehensive program operated by a work college under section 448(e) of the Higher Education Act. Ordinary work-study employment is not this exception.
    WorkCollege,
    /// Teaching, research or other services required for the award
    /// The part that pays for those services is taxable. Leave out any part already on your W-2.
    PaidServices {
        /// Part of the award that pays for the required services
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
}

/// Education scholarship or fellowship
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct EducationAward {
    /// Total award received
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub received: i64,
    /// Award restricted to room, board or other nonqualified costs
    /// Use the award's terms, not total living expenses. Zero for an unrestricted award.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub restricted: i64,
    /// Award funds spent on required tuition, fees, books, supplies and equipment
    /// Exclude room, board, travel, optional equipment, costs paid with other tax-free funds and expenses assigned to a credit or other tax benefit. Do not include funds restricted to nonqualified costs. Enter what the award actually paid toward tuition; a part you choose to treat as paying living costs so the education credit can use more of the tuition is chosen on the Education page and added to your income there, not here.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub qualified_paid: i64,
    /// Candidate for a degree at an eligible educational institution
    /// Check your enrollment records or Form 1098-T. No taxes the whole award.
    pub degree_candidate: bool,
    /// Services required by the award
    /// The three named programs waive only the service restriction; degree and qualified-spending rules still apply.
    pub services: ScholarshipService,
    /// No other special exclusion
    /// No qualifying graduate/postdoctoral non-tuition stipend. Use the dedicated stipend choice for that award. No applies the ordinary rules without any special exclusion.
    /// Check the award letter and its conditions for any special exclusion.
    pub ordinary: bool,
}

/// Scholarship or fellowship not reported on Form W-2
/// Enter award and spending records. Do not duplicate wages. Tuition reductions, service awards outside the listed exceptions and foreign/treaty exclusions require separate review. A part of a grant you include in income to raise an education credit is chosen on the Education page.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum Scholarship {
    /// Education scholarship or fellowship
    Education {
        /// Award and spending records
        data: EducationAward,
    },
    /// Graduate or postdoctoral non-tuition fellowship or stipend
    GraduateStipend {
        /// Non-tuition payment received
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        received: i64,
        /// Entire payment aids graduate or postdoctoral study and is an ordinary taxable non-tuition fellowship or stipend
        /// Not W-2 pay, a service-required award, a tax-free emergency grant or a treaty-exempt payment. Check the award letter and whether it was paid on a W-2. No taxes the payment without counting it as IRA compensation.
        eligible: bool,
    },
}
impl Scholarship {
    // The page's client rule, repeated on save: spending, the restricted part
    // and the service part together stay within the award.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if !(0..=100_000_000_000).contains(&self.amount()) {
            let issue =
                ValidationIssue::new("SCHOLARSHIP_SPENDING_ABOVE_AWARD").at_field("received");
            return Some(if matches!(self, Self::Education { .. }) {
                issue.in_field("data")
            } else {
                issue
            });
        }
        if let Self::Education { data } = self {
            let service_part = match data.services {
                ScholarshipService::PaidServices { amount } => amount,
                _ => 0,
            };
            if data.restricted < 0 || data.restricted > data.received {
                return Some(
                    ValidationIssue::new("SCHOLARSHIP_SPENDING_ABOVE_AWARD")
                        .at_field("restricted")
                        .in_field("data"),
                );
            }
            if !(0..=100_000_000_000).contains(&service_part) {
                return Some(
                    ValidationIssue::new("SCHOLARSHIP_SPENDING_ABOVE_AWARD")
                        .at_field("amount")
                        .in_field("services")
                        .in_field("data"),
                );
            }
            if data.qualified_paid < 0
                || data.qualified_paid > data.received - data.restricted - service_part
            {
                return Some(
                    ValidationIssue::new("SCHOLARSHIP_SPENDING_ABOVE_AWARD")
                        .at_field("qualified_paid")
                        .in_field("data"),
                );
            }
        }
        None
    }
    pub(crate) fn amount(&self) -> i64 {
        match self {
            Self::Education { data } => data.received,
            Self::GraduateStipend { received, .. } => *received,
        }
    }
    pub(crate) fn valid(&self) -> bool {
        self.issue().is_none()
    }
    // The taxable amount, the part that is IRA compensation, and the answer
    // given to `ordinary` (No applies the ordinary rules; the
    // caller states which). Pub. 970 Worksheet 1-1: the award less the part
    // spent on qualified expenses by a degree candidate. Qualified spending
    // never exceeds the award less its restricted and service parts (valid),
    // so the taxable amount equals line 9 for every input.
    pub(crate) fn taxable(&self) -> (i64, i64, bool) {
        match self {
            Self::Education { data } => (
                data.received - if data.degree_candidate { data.qualified_paid } else { 0 },
                0,
                data.ordinary,
            ),
            Self::GraduateStipend { received, eligible } => {
                (*received, if *eligible { *received } else { 0 }, true)
            }
        }
    }
}
