#[allow(unused_imports)]
use super::*;
pub use crate::accepted_credits::{FederalCredits, MoreCreditRefusal};

impl FederalCredits {
    pub(crate) fn reconcile_education_insurance_sources(&mut self, previous: &[EducationSource]) {
        for row in &mut self.students {
            let matches = |source: &EducationSource, others: &[EducationSource]| {
                others.iter().any(|other| {
                    other.id == source.id
                        && other.student == source.student
                        && other.form.box_10 == source.form.box_10
                })
            };
            let relevant =
                |source: &&EducationSource| source.student == row.student && source.form.box_10 > 0;
            if previous.iter().filter(relevant).any(|source| !matches(source, &self.education))
                || self.education.iter().filter(relevant).any(|source| !matches(source, previous))
            {
                row.more.insurance_refund_current_expenses = crate::SourceCents::Unavailable;
            }
        }
    }

    pub(crate) fn education_insurance_refund_total(&self, student: i64) -> Option<i64> {
        self.education
            .iter()
            .filter(|source| source.student == student)
            .try_fold(0_i64, |sum, source| sum.checked_add(source.form.box_10))
    }

    // Form 1098-T box 10 reports the year received, not the expense year.
    // Pub. 970 chapters 2–3 require allocation to the actual paid expenses:
    // https://www.irs.gov/publications/p970
    pub(crate) fn education_insurance_refund_current(
        &self,
        row: &EducationStudent,
    ) -> Result<i64, &'static str> {
        let total = self
            .education_insurance_refund_total(row.student)
            .ok_or("EDUCATION_INSURANCE_REFUND_ABOVE_BOX_10")?;
        if total == 0 {
            return Ok(0);
        }
        row.more
            .insurance_refund_current_expenses
            .source_fact()
            .map_err(|_| "EDUCATION_INSURANCE_REFUND_ABOVE_BOX_10")?
            .check_reported(|amount| {
                if *amount <= total {
                    Ok(())
                } else {
                    Err("EDUCATION_INSURANCE_REFUND_ABOVE_BOX_10")
                }
            })?
            .require(|| "EDUCATION_INSURANCE_REFUND_ALLOCATION_NEEDED")
            .map(|amount| amount.unwrap_or(0))
    }

    pub(crate) fn education_insurance_refund_issue(
        &self,
        row: &EducationStudent,
    ) -> Option<&'static str> {
        let current = match self.education_insurance_refund_current(row) {
            Ok(current) => current,
            Err(key) => return Some(key),
        };
        if row.more.refunds.checked_add(current).is_none() {
            return Some("EDUCATION_INSURANCE_REFUND_ABOVE_BOX_10");
        }
        if self.education_insurance_refund_total(row.student)? > current {
            // A remainder does not identify a prior year or bind the refund
            // to the separately entered single-credit recapture history.
            return Some("ISSUE_EDUCATION_RECAPTURE_COMPLEX_PRIOR_RETURN");
        }
        None
    }

    pub(crate) fn education_student_page(&self, row: &EducationStudent) -> EducationStudentPage {
        let mut page = EducationStudentPage::from(row.clone());
        if self.education_insurance_refund_total(row.student) == Some(0) {
            page.more.insurance_refund_current_expenses = crate::SourceCents::NotReported;
            // SourceCents::NotReported is the second schema variant.
            page.allowed_insurance_refund_allocations = alloc::vec![1];
        }
        page
    }

    pub(crate) fn reconcile_education_students(&mut self) {
        self.students.retain(|row| {
            self.education
                .iter()
                .any(|source| source.assigned.is_known() && source.student == row.student)
        });
        for source in self.education.iter().filter(|source| source.assigned.is_known()) {
            if !self.students.iter().any(|row| row.student == source.student) {
                self.students.push(EducationStudent {
                    student: source.student,
                    method: None,
                    more: StudentMore::default(),
                });
            }
        }
    }

    pub(crate) fn education_pending(&self) -> Vec<(i64, &'static str)> {
        let mut pending = Vec::new();
        for row in &self.students {
            if row.method.is_none() {
                pending.push((row.student, "EDUCATION_METHOD_NEEDED"));
            }
            if row.more.american_opportunity_used_up.is_none() {
                pending.push((row.student, "EDUCATION_AOTC_HISTORY_NEEDED"));
            }
            if let Some(key) = self.education_insurance_refund_issue(row) {
                pending.push((row.student, key));
            }
        }
        for source in self.education.iter().filter(|source| source.assigned.is_known()) {
            if !self.students.iter().any(|row| row.student == source.student) {
                pending.push((source.student, "EDUCATION_METHOD_NEEDED"));
            }
        }
        pending
    }

    pub(crate) fn education_inputs(&self) -> Vec<EducationAnnualProjection> {
        self.students
            .iter()
            .filter_map(|row| {
                let method = row.method?;
                row.more.american_opportunity_used_up?;
                let sources: Vec<_> = self
                    .education
                    .iter()
                    .filter(|source| source.assigned.is_known() && source.student == row.student)
                    .collect();
                if sources.is_empty() {
                    return None;
                }
                let tuition = sources
                    .iter()
                    .try_fold(0_i64, |sum, source| sum.checked_add(source.form.box_1))?;
                let assistance = sources
                    .iter()
                    .try_fold(0_i64, |sum, source| sum.checked_add(source.form.box_5))?;
                let refunds = row
                    .more
                    .refunds
                    .checked_add(self.education_insurance_refund_current(row).ok()?)?;
                Some(EducationAnnualProjection {
                    student: row.student,
                    method,
                    expenses: FederalEducationExpenses {
                        tuition,
                        assistance,
                        school_materials: row.more.books_from_school,
                        other_materials: row.more.other_books,
                        refunds,
                        other_benefits: row.more.tax_free_529,
                        scholarship_in_income: row.more.scholarship_in_income.clone(),
                    },
                    sources: sources.iter().map(|source| source.id).collect(),
                })
            })
            .collect()
    }
}

// The credits hub's last topic: every FederalCredits field this
// page exposes onto one list item, each variant carrying the existing model
// record unchanged. A variant absent from the list clears its field(s) on
// save; a repeated variant of the same kind keeps the last one entered.
/// More credits
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MoreCredits {
    /// Your records for accepted credit documents
    #[libertas_read_only]
    /// ----
    /// Document
    pub documents: Vec<crate::CreditDocumentView>,
    /// Your answers about these documents
    // The answered questions, each as its canonical row; unanswered ones are
    // asked on the questions page (credit_questions).
    #[libertas_read_only]
    /// ----
    /// Answer
    pub answers: Vec<crate::CreditDocumentQuestion>,
    /// More credits
    #[libertas_unordered]
    /// ----
    /// Credit
    pub credits: Vec<MoreCredit>,
}
/// Credit
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MoreCredit {
    /// Credit for prior-year minimum tax (Form 8801)
    /// Leave empty only if there is no prior AMT or credit history to evaluate.
    MinimumTax(crate::MinimumTaxHistoriesPage),
    /// Mortgage Credit Certificate credit (Form 8396)
    /// Use one qualified certificate for current-year interest. A reissue needs the original mortgage schedule and refinance date; a 2026 reissue also records interest paid before refinancing so each rate period can be calculated. Multiple current certificates need separate review. Enter prior balances from 2025 Form 8396 even if there is no 2026 certificate interest.
    MortgageCredit {
        /// Current-year qualified certificate, if interest was paid in 2026
        /// [NullText]
        /// No current-year mortgage certificate interest credit is claimed
        /// [SetText]
        /// The current-year qualified mortgage certificate is
        #[libertas_default($null)]
        current: Option<FederalMortgageCertificate>,
        /// 2023 credit carryforward from 2025 Form 8396 line 16
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        carry_three_years_prior: i64,
        /// 2024 credit carryforward from 2025 Form 8396 line 14
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        carry_two_years_prior: i64,
        /// 2025 credit carryforward from 2025 Form 8396 line 17
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        carry_prior_year: i64,
        /// Saved Home mortgage interest entry for the certified loan
        /// Select the first lender's saved entry so the credit reduces that loan's interest deduction (section 163(g)), including the rental share of a home you also rent out. Choose “No deduction is claimed for this certified-loan interest” when you do not deduct the interest.
        /// [NullText]
        /// No deduction is claimed for this certified-loan interest
        /// [SetText]
        /// The certified interest is deducted on this saved mortgage entry
        #[libertas_enum_source("$.mortgage_loans")]
        loan: Option<i64>,
    },
    /// Residential energy and D.C. homebuyer credit carryovers (Forms 5695, 8859)
    /// These are prior credit carryforwards, not new 2026 purchases; new 2026 residential clean energy or D.C. first-time homebuyer purchases do not qualify.
    PriorCreditCarryovers {
        /// Unused residential clean energy credit from 2025 Form 5695, line 16
        /// Choose “No prior-year residential clean energy credit is claimed” if none.
        /// [NullText]
        /// No prior-year residential clean energy credit is claimed
        /// [SetText]
        /// The unused 2025 residential clean energy credit claimed is
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default($null)]
        residential_energy_carry: Option<i64>,
        /// Unused District of Columbia first-time homebuyer credit from 2025 Form 8859, line 4
        /// Choose “No prior-year D.C. first-time homebuyer credit is claimed” if none.
        /// [NullText]
        /// No prior-year D.C. first-time homebuyer credit is claimed
        /// [SetText]
        /// The unused 2025 D.C. first-time homebuyer credit claimed is
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default($null)]
        dc_homebuyer_carry: Option<i64>,
    },
    /// Clean vehicles and refueling property (Forms 8936, 8911)
    CleanVehiclesAndRefueling {
        /// Clean-vehicle income limits and vehicles acquired by September 30, 2025 and first placed in service in 2026
        /// For a completed business-form credit entered under General business credits, enter any 2025 modified AGI and 2026 territory addbacks here without duplicating that business vehicle in the personal vehicle list. For a personal vehicle, use the accepted IRS time-of-sale report for each VIN. A dealer transfer or a business-use allocation calculated from vehicle details needs separate return review.
        /// [NullText]
        /// No clean-vehicle income-limit facts or qualifying new or used vehicles are entered here
        /// [SetText]
        /// Clean-vehicle income-limit facts or qualifying new or used vehicles are entered here
        clean_vehicles: Option<crate::FederalCleanVehicleClaimPage>,
        /// Personal-use alternative-fuel or electric-vehicle refueling property placed in service by June 30, 2026
        /// Enter each qualified single item separately for its own $1,000 credit ceiling; for EV charging, the item is a charging port. Allocate shared installation costs among ports once. Business or mixed-use property belongs to Form 3800 review.
        /// ----
        /// Refueling property
        personal_refueling: Vec<FederalPersonalRefueling>,
    },
    /// Federal tax on kerosene used in your home (Form 4136)
    /// Undyed kerosene bought away from a blocked pump and burned in your home for heating, lighting or cooking. Fuel used in a business or vehicle belongs with that business.
    HomeKerosene(FederalHomeKerosene),
    /// Adoption (Form 8839)
    Adoption {
        /// Adoptions
        /// ----
        /// Child
        adoptions: Vec<crate::AdoptionChildPage>,
        /// Prior nonrefundable adoption credit balances
        // Tax structure: adoption credits carry forward for five tax years.
        #[libertas_size(max = 5)]
        /// ----
        /// Carryover
        adoption_carry: Vec<crate::AdoptionCarry>,
    },
    /// Credit for the elderly or disabled (Schedule R)
    ElderlyOrDisabled {
        /// Nontaxable pensions for the elderly credit
        /// Enter each source once for filers on this return. Social Security is calculated from the income entries.
        /// ----
        /// Benefit
        exempt_pensions: Vec<crate::ExemptPension>,
    },
    /// Foreign tax credit or deduction
    /// Choose Form 1116, the small payee-statement credit (at most $300 or $600 jointly), or an itemized deduction for foreign income taxes. The App keeps your choice; it does not compare or choose a method for you.
    ForeignTax(crate::ForeignTaxPage),
    /// Qualified electric vehicle credit carryover (Form 8834)
    ElectricVehicleCarryover {
        /// Prior qualified-electric-vehicle passive credit released for 2026 on Form 8582-CR
        /// Enter only the amount allowed for this year, not the original pre-2007 vehicle credit or unreleased carryover. Form 8834 applies a second regular-tax and tentative-minimum-tax limit.
        /// [NullText]
        /// No released prior-year electric-vehicle credit is claimed
        /// [SetText]
        /// The released prior-year electric-vehicle credit claimed for 2026 is
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default($null)]
        legacy_ev_passive_allowed: Option<i64>,
    },
    /// Employer tip FICA credit carryover (Form 8846)
    EmployerTip(crate::EmployerTipFicaCarryforward),
    /// General business credits (Form 3800)
    /// Credits figured on their own forms, carryforwards and carrybacks, and credits bought, sold or elected as a payment. Business-page, property and K-1 credits are added from those sources; the 2025 Form8846 carryforward has its own item.
    BusinessCredits(crate::BusinessCreditHistoryPage),
    /// 2025 education credit recapture
    /// A 2025 American opportunity or lifetime learning credit reduced by a refund or tax-free education assistance received in 2026.
    EducationRecapture(crate::PriorYearEducationCreditRecapture),
    /// Credits claimed after an earlier disallowance (Form 8862)
    /// Answer for each credit the IRS reduced or disallowed for an earlier year. The App fills Form 8862 from the people, children and students already on this return.
    AfterDisallowance(crate::CreditsAfterDisallowance),
    // The credit notice kinds: stored in FederalDraft.credit_refusals and
    // named as left out of the estimate.
    /// Choosing not to claim the earned income credit
    DeclineEarnedIncomeCredit,
    /// Another credit not listed
    /// A credit with no 2026 form line, such as a fuel cell vehicle bought before 2022 and first placed in service in 2026. Its benefit is left out of the estimate with a notice. A general business credit figured on its source form belongs in General business credits above.
    OtherCredit,
}

impl From<MoreCreditRefusal> for MoreCredit {
    fn from(kind: MoreCreditRefusal) -> Self {
        match kind {
            MoreCreditRefusal::OtherCredit => Self::OtherCredit,
        }
    }
}

impl MoreCredit {
    // The stored form of a refusal kind.
    pub(crate) fn refusal(&self) -> Option<MoreCreditRefusal> {
        Some(match self {
            Self::OtherCredit => MoreCreditRefusal::OtherCredit,
            _ => return None,
        })
    }
}

impl FederalCredits {
    // The one owner of "expenses taken into account for an education credit"
    // (IRC 135(d)(2)(A), 529(c)(3)(B)(v), 530(d)(2)(C)): Form 8863 figures its
    // credits from it, and every benefit that must not reuse those expenses
    // (the savings bond exclusion, the education account distributions) reads
    // it. One element per education entry, in list order, in whole dollars.
    // `credit_allowed` is false when the App allows no credit (married filing
    // separately, a filer another taxpayer claims, no filer with a valid SSN);
    // a dependent student without a valid SSN takes none either. A filer who
    // is the student needs no SSN of their own beyond the filer rule: on a
    // joint return one spouse's SSN suffices even for a spouse-student (2026
    // i8863 "Identification requirement" Tip).
    pub(crate) fn education_expenses_used(
        &self,
        students: &[PersonChoice],
        credit_allowed: bool,
    ) -> Vec<EducationUse> {
        // The lifetime learning credit's per-return cap is shared in list order.
        let mut lifetime_room = crate::year_constants::LIFETIME_LEARNING_EXPENSE_CAP_DOLLARS;
        self.education_inputs()
            .iter()
            .map(|entry| {
                let student = students.iter().find(|v| v.id == entry.student);
                let method = entry.method();
                let qualified = crate::kernel::ratio::dollars(entry.expenses.qualified(method));
                let used =
                    if !credit_allowed || student.is_some_and(|v| !v.is_filer() && !v.valid_ssn) {
                        0
                    } else if method == EducationMethod::AmericanOpportunity {
                        phase::cap(
                            qualified,
                            [crate::year_constants::AMERICAN_OPPORTUNITY_EXPENSE_CAP_DOLLARS],
                        )
                    } else {
                        let used = phase::cap(qualified, [lifetime_room]);
                        lifetime_room = crate::kernel::carry::native_balance(lifetime_room, used);
                        used
                    };
                // Form 8815 line 2 takes tuition and fees only (i8815 line 2).
                // The credit's consumed expenses are attributed to the
                // course materials first, so the tuition the credit did not
                // need stays available for the exclusion; no rule orders the
                // credit's expenses, and this reading favors the filer.
                let materials = entry.expenses.school_materials
                    + if method == EducationMethod::AmericanOpportunity {
                        entry.expenses.other_materials
                    } else {
                        0
                    };
                let used_tuition =
                    used - phase::cap(used, [crate::kernel::ratio::dollars(materials)]);
                EducationUse {
                    person: student.map(|v| v.id),
                    method,
                    qualified,
                    used,
                    used_tuition,
                }
            })
            .collect()
    }
}

/// Saved credit answers
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub enum MoreCreditInitial {
    MinimumTax(crate::MinimumTaxHistoriesPage),
    MortgageCredit {
        current: Option<FederalMortgageCertificate>,
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        carry_three_years_prior: i64,
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        carry_two_years_prior: i64,
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        carry_prior_year: i64,
        loan: Option<i64>,
    },
    PriorCreditCarryovers {
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        residential_energy_carry: Option<i64>,
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        dc_homebuyer_carry: Option<i64>,
    },
    CleanVehiclesAndRefueling {
        clean_vehicles: Option<crate::FederalCleanVehicleClaimPage>,
        personal_refueling: Vec<FederalPersonalRefueling>,
    },
    HomeKerosene(FederalHomeKerosene),
    Adoption {
        adoptions: Vec<crate::AdoptionChildInitial>,
        // Tax structure: adoption credits carry forward for five tax years.
        adoption_carry: Vec<crate::AdoptionCarry>,
    },
    ElderlyOrDisabled {
        exempt_pensions: Vec<crate::ExemptPension>,
    },
    ForeignTax(crate::extended_model::ForeignTaxInitial),
    TaxCreditBonds {
        tax_credit_bonds: Vec<crate::FederalTaxCreditBond1097>,
    },
    ElectricVehicleCarryover {
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        legacy_ev_passive_allowed: Option<i64>,
    },
    EmployerTip(crate::EmployerTipFicaCarryforward),
    BusinessCredits(crate::BusinessCreditHistoryInitial),
    EducationRecapture(crate::PriorYearEducationCreditRecapture),
    AfterDisallowance(crate::CreditsAfterDisallowance),
    // The credit notice kinds: stored in FederalDraft.credit_refusals and
    // named as left out of the estimate.
    DeclineEarnedIncomeCredit,
    OtherCredit,
}
impl From<MoreCredit> for MoreCreditInitial {
    fn from(value: MoreCredit) -> Self {
        match value {
            MoreCredit::MinimumTax(value) => Self::MinimumTax(value),
            MoreCredit::MortgageCredit {
                current,
                carry_three_years_prior,
                carry_two_years_prior,
                carry_prior_year,
                loan,
            } => Self::MortgageCredit {
                current,
                carry_three_years_prior,
                carry_two_years_prior,
                carry_prior_year,
                loan,
            },
            MoreCredit::PriorCreditCarryovers { residential_energy_carry, dc_homebuyer_carry } => {
                Self::PriorCreditCarryovers { residential_energy_carry, dc_homebuyer_carry }
            }
            MoreCredit::CleanVehiclesAndRefueling { clean_vehicles, personal_refueling } => {
                Self::CleanVehiclesAndRefueling { clean_vehicles, personal_refueling }
            }
            MoreCredit::HomeKerosene(value) => Self::HomeKerosene(value),
            MoreCredit::Adoption { adoptions, adoption_carry } => Self::Adoption {
                adoptions: adoptions.into_iter().map(Into::into).collect(),
                adoption_carry,
            },
            MoreCredit::ElderlyOrDisabled { exempt_pensions } => {
                Self::ElderlyOrDisabled { exempt_pensions }
            }
            MoreCredit::ForeignTax(value) => Self::ForeignTax(value.into()),
            MoreCredit::ElectricVehicleCarryover { legacy_ev_passive_allowed } => {
                Self::ElectricVehicleCarryover { legacy_ev_passive_allowed }
            }
            MoreCredit::EmployerTip(value) => Self::EmployerTip(value),
            MoreCredit::BusinessCredits(value) => Self::BusinessCredits(value.into()),
            MoreCredit::EducationRecapture(value) => Self::EducationRecapture(value),
            MoreCredit::AfterDisallowance(value) => Self::AfterDisallowance(value),
            MoreCredit::DeclineEarnedIncomeCredit => Self::DeclineEarnedIncomeCredit,
            MoreCredit::OtherCredit => Self::OtherCredit,
        }
    }
}

/// More credits
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct MoreCreditsInitial {
    /// Your records for accepted credit documents
    #[libertas_read_only]
    /// ----
    /// Document
    pub documents: Vec<crate::CreditDocumentView>,
    pub credits: Vec<MoreCreditInitial>,
}
