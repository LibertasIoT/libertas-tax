use crate::Owner;
use crate::care::{
    CareBenefitSource, CareDeductions, CareFilerSituation, CareFilingStatus, CarePlanMaximum,
    FederalCare, PriorYearCare, ProviderIdentification, ProviderRelationship,
};
use crate::kernel::lines::{LineValue, row};
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction};
use crate::line_emission::emit;
use crate::validation::ValidationIssue;
use alloc::vec::Vec;

impl FederalCare {
    pub(crate) fn has_benefits(&self, d: &crate::FederalDraft) -> bool {
        !self.grace.is_empty()
            || !self.own_business.is_empty()
            || d.income.iter().any(
                |e| matches!(&e.income,crate::FederalIncome::Wage {data} if crate::FederalWage::from(data).care_benefits>0),
            )
    }
    pub(crate) fn left_out(
        &self,
        d: &crate::FederalDraft,
        r: &mut crate::FederalResult,
    ) -> Option<i64> {
        for (index, f) in self.filers.iter().enumerate() {
            if matches!(f.situation,CareFilerSituation::Incapable {same_home_over_half_year:false,care_paid,..} if care_paid>0)
                && let Some(p) = d.people.as_ref().and_then(|p| {
                    if f.owner == Owner::Taxpayer { Some(&p.taxpayer) } else { p.spouse.as_ref() }
                })
            {
                crate::federal_rules::note_once_scoped(
                    r,
                    crate::Message::new(
                        "WARN_CARE_COSTS_LEFT_OUT",
                        alloc::vec![crate::MessageArg::Encoded(crate::interview::person_name(
                            &p.first_name,
                            &p.last_name
                        ))],
                    ),
                    crate::IssueContext::Section(crate::TaxSection::Care).item("filers", index),
                );
            }
        }
        self.providers.iter().enumerate().for_each(|(i, p)| {
            let context =
                crate::IssueContext::Section(crate::TaxSection::Care).item("providers", i);
            let marker = match p.identification {
                ProviderIdentification::Individual => "CARE_PROVIDER_INDIVIDUAL",
                ProviderIdentification::Organization => "CARE_PROVIDER_ORGANIZATION",
                ProviderIdentification::TaxExempt => "CARE_PROVIDER_TAX_EXEMPT",
                ProviderIdentification::EmployerPlan => "CARE_PROVIDER_SEE_W2",
                ProviderIdentification::LivingAbroad => "CARE_PROVIDER_LAFCP",
                ProviderIdentification::Refused => "CARE_PROVIDER_REFUSED",
                ProviderIdentification::NotObtained => "CARE_PROVIDER_NOT_OBTAINED",
            };
            crate::federal_rules::note_once_scoped(
                r,
                crate::Message::new(
                    "NOTE_CARE_PROVIDER_DETAILS",
                    alloc::vec![
                        crate::MessageArg::Text(p.name.clone()),
                        crate::MessageArg::Text(p.address.clone()),
                        crate::MessageArg::Nested(alloc::boxed::Box::new(crate::Message::new(
                            marker,
                            Vec::new()
                        )))
                    ],
                ),
                context.clone(),
            );
            if p.household_employee
                && d.screening
                    .as_ref()
                    .is_none_or(|screening| screening.household_employment.is_empty())
            {
                crate::federal_rules::note_once_scoped(
                    r,
                    msg!("NOTE_CARE_HOUSEHOLD_EMPLOYEE_SCHEDULE_H"),
                    context.clone(),
                );
            }
            let key = p.omission_key();
            if let Some(key) = key {
                crate::federal_rules::note_once_scoped(
                    r,
                    crate::Message::new(key, alloc::vec![crate::MessageArg::Text(p.name.clone())]),
                    context.clone(),
                );
            } else {
                if p.identification == ProviderIdentification::Refused {
                    crate::federal_rules::note_once_scoped(
                        r,
                        crate::Message::new(
                            "NOTE_CARE_PROVIDER_STATEMENT_ATTACH",
                            alloc::vec![crate::MessageArg::Text(p.name.clone())],
                        ),
                        context.clone(),
                    );
                }
            }
        });
        self.omitted_amount()
    }
    pub(crate) fn earned(
        &self,
        actual: [i64; 2],
        people: i64,
        r: &mut crate::FederalResult,
    ) -> [i64; 2] {
        let unit = if people >= 2 {
            500
        } else if people == 1 {
            250
        } else {
            0
        };
        let mut months: [Vec<i64>; 2] = [Vec::new(), Vec::new()];
        let mut wages = [0; 2];
        for f in &self.filers {
            let i = f.owner as usize;
            if f.situation.eligible() {
                months[i] = f.situation.months().to_vec();
                wages[i] = dollars(f.situation.earnings());
            }
        }
        let shared = months[0].iter().filter(|m| months[1].contains(m)).count() as i64;
        let counts = [months[0].len() as i64, months[1].len() as i64];
        let calculate = |winner: usize| {
            core::array::from_fn(|i| {
                let n = counts[i] - if i != winner { shared } else { 0 };
                actual[i] - wages[i] + phase::floor(wages[i], n * unit)
            })
        };
        let first: [i64; 2] = calculate(0);
        let second: [i64; 2] = calculate(1);
        let out = if phase::cap(first[0], [first[1]]) >= phase::cap(second[0], [second[1]]) {
            first
        } else {
            second
        };
        for i in 0..2 {
            if (wages[i] > 0 && wages[i] < counts[i] * unit) || shared > 0 {
                crate::federal_rules::note_once(r, msg!("WARN_CARE_DEEMED_INCOME_MONTHLY"));
            }
        }
        out
    }
    pub(crate) fn rule(&self, d: &crate::FederalDraft) -> Option<&'static str> {
        self.issue(d).map(|issue| issue.key)
    }
    pub(crate) fn valid(&self, d: &crate::FederalDraft) -> bool {
        self.validate_issue(d, false).is_none()
    }
    pub(crate) fn issue(&self, d: &crate::FederalDraft) -> Option<ValidationIssue> {
        self.validate_issue(d, true)
    }
    fn validate_issue(&self, d: &crate::FederalDraft, page: bool) -> Option<ValidationIssue> {
        let family = d.people.as_ref().map(|p| {
            crate::federal_rules::family(d, &mut crate::FederalResult::empty(d.revision), p)
        });
        let care_cost = family.as_ref().map_or(0, |f| f.care_expenses);
        let money = |v: i64| (0..=100000000000).contains(&v);
        // A return has at most two filers; these rows are unique by filer.
        for (field, len, max) in [("grace", self.grace.len(), 2), ("filers", self.filers.len(), 2)]
        {
            if len > max {
                return Some(ValidationIssue::new("CARE_INVALID_RECORD").at_field(field));
            }
        }
        for (i, p) in self.providers.iter().enumerate() {
            let field = if p.name.trim().is_empty() {
                Some("name")
            } else if p.address.trim().is_empty() {
                Some("address")
            } else if p.paid < 0 {
                Some("paid")
            } else {
                None
            };
            if let Some(field) = field {
                return Some(
                    ValidationIssue::new("CARE_INVALID_RECORD")
                        .at_field(field)
                        .in_list("providers", i),
                );
            }
        }
        if page
            && !family.as_ref().is_some_and(|f| f.care_under_13_child)
            && let Some((i, _)) = self
                .providers
                .iter()
                .enumerate()
                .find(|(_, p)| p.relationship == ProviderRelationship::ParentOfChild)
        {
            return Some(
                ValidationIssue::new("CARE_PARENT_PROVIDER_NEEDS_A_CHILD")
                    .at_field("relationship")
                    .in_list("providers", i),
            );
        }
        if page && care_cost + self.filer_expenses() > 0 && self.providers.is_empty() {
            return Some(ValidationIssue::new("CARE_COSTS_NEED_A_PROVIDER").at_field("providers"));
        }
        match self.plan_maximum {
            CarePlanMaximum::Plans { maximum } if maximum <= 0 => {
                return Some(
                    ValidationIssue::new("CARE_INVALID_RECORD")
                        .at_field("maximum")
                        .in_field("plan_maximum"),
                );
            }
            _ => {}
        }
        if page
            && self.has_benefits(d) != matches!(self.plan_maximum, CarePlanMaximum::Plans { .. })
        {
            return Some(
                ValidationIssue::new("CARE_PLAN_MAXIMUM_REQUIRED").at_field("plan_maximum"),
            );
        }
        let joint =
            d.setup.as_ref().is_some_and(|s| s.filing_status() == crate::FederalStatus::Joint);
        let allowed = |o: Owner| o == Owner::Taxpayer || joint;
        for (i, b) in self.benefits.iter().enumerate() {
            let box10 =
                d.income.iter().find_map(|e| (e.id == b.wage).then_some(&e.income)).and_then(|v| {
                    if let crate::FederalIncome::Wage { data } = v {
                        Some(crate::FederalWage::from(data).care_benefits)
                    } else {
                        None
                    }
                });
            let Some(box10) = box10.filter(|v| *v > 0) else {
                return Some(
                    ValidationIssue::new("CARE_WAGE_SOURCE_INVALID")
                        .at_field("wage")
                        .in_list("benefits", i),
                );
            };
            if self.benefits[..i].iter().any(|other| other.wage == b.wage) {
                return Some(
                    ValidationIssue::new("CARE_WAGE_SOURCE_INVALID")
                        .at_field("wage")
                        .in_list("benefits", i),
                );
            }
            if b.in_box_1.is_some_and(|amount| amount < 0)
                || b.forfeited.is_some_and(|amount| amount < 0)
                || b.in_box_1
                    .unwrap_or(0)
                    .checked_add(b.forfeited.unwrap_or(0))
                    .is_none_or(|total| total > box10)
            {
                return Some(
                    ValidationIssue::new("CARE_BENEFIT_PARTS_ABOVE_BOX_10")
                        .at_field(
                            if b.in_box_1.is_some_and(|amount| amount < 0)
                                || b.in_box_1.unwrap_or(0) > box10
                            {
                                "in_box_1"
                            } else {
                                "forfeited"
                            },
                        )
                        .in_list("benefits", i),
                );
            }
        }
        for (i, f) in self.filers.iter().enumerate() {
            if !joint || self.filers[..i].iter().any(|other| other.owner == f.owner) {
                return Some(
                    ValidationIssue::new("CARE_FILER_SOURCE_INVALID")
                        .at_field("owner")
                        .in_list("filers", i),
                );
            }
            let Some(p) = d.people.as_ref().and_then(|p| {
                if f.owner == Owner::Taxpayer { Some(&p.taxpayer) } else { p.spouse.as_ref() }
            }) else {
                return Some(
                    ValidationIssue::new("CARE_FILER_SOURCE_INVALID")
                        .at_field("owner")
                        .in_list("filers", i),
                );
            };
            if match f.situation {
                CareFilerSituation::Student { .. } => !p.full_time_student,
                CareFilerSituation::Incapable { .. } => !p.incapable_self_care,
            } {
                return Some(
                    ValidationIssue::new("CARE_FILER_SOURCE_INVALID")
                        .at_field("situation")
                        .in_list("filers", i),
                );
            }
            let m = f.situation.months();
            if m.iter().enumerate().any(|(i, x)| !(1..=12).contains(x) || m[..i].contains(x)) {
                return Some(
                    ValidationIssue::new("CARE_INVALID_RECORD")
                        .at_field("months")
                        .in_field("situation")
                        .in_list("filers", i),
                );
            }
            if matches!(f.situation, CareFilerSituation::Student { .. }) && m.len() < 5 {
                return Some(
                    ValidationIssue::new("CARE_STUDENT_NEEDS_FIVE_MONTHS")
                        .at_field("months")
                        .in_field("situation")
                        .in_list("filers", i),
                );
            }
            if !money(f.situation.earnings())
                || matches!(f.situation,CareFilerSituation::Incapable {care_paid,..} if !money(care_paid))
            {
                return Some(
                    ValidationIssue::new("CARE_INVALID_RECORD")
                        .at_field(if !money(f.situation.earnings()) {
                            "earned_in_months"
                        } else {
                            "care_paid"
                        })
                        .in_field("situation")
                        .in_list("filers", i),
                );
            }
            if p.death_date()
                .is_some_and(|date| m.iter().any(|month| *month > i64::from(date / 100 % 100)))
            {
                return Some(
                    ValidationIssue::new("CARE_FILER_SOURCE_INVALID")
                        .at_field("months")
                        .in_field("situation")
                        .in_list("filers", i),
                );
            }
        }
        if let Some((i, _)) = self.grace.iter().enumerate().find(|(i, g)| {
            !allowed(g.owner)
                || g.used_in_return_year <= 0
                || g.used_in_return_year
                    > crate::year_constants::DEPENDENT_CARE_BENEFIT_LIMIT_2025_CENTS
                || self.grace[..*i].iter().any(|x| x.owner == g.owner)
        }) {
            return Some(
                ValidationIssue::new("CARE_INVALID_RECORD").at_field("owner").in_list("grace", i),
            );
        }
        if let Some((i, _)) = self.own_business.iter().enumerate().find(|(i,b)|self.own_business[..*i].iter().any(|other|other.owner==b.owner&&other.source==b.source)||!allowed(b.owner)||b.received<=0||match b.source { CareBenefitSource::Partnership {partnership}=>!d.income.iter().any(|e|e.id==partnership&&e.owner==crate::DocumentOwner::from(b.owner)&&matches!(e.income,crate::FederalIncome::Partnership {..})),CareBenefitSource::Business {business}|CareBenefitSource::Farm {farm:business}=>!d.income.iter().any(|e|e.id==business&&e.owner==crate::DocumentOwner::from(b.owner)&&matches!(&e.income,crate::FederalIncome::Business {data} if matches!(data.activity,crate::BusinessActivity::Farming {..})==matches!(b.source,CareBenefitSource::Farm {..}))) }) {return Some(ValidationIssue::new("CARE_BUSINESS_SOURCE_INVALID").at_field("source").in_list("own_business", i));}
        if matches!(self.filing,CareFilingStatus::Separate {spouse_earned_income:Some(v)} if !money(v))
        {
            return Some(
                ValidationIssue::new("CARE_INVALID_RECORD")
                    .at_field("spouse_earned_income")
                    .in_field("filing"),
            );
        }
        if matches!(self.unpaid_return_year_care, Some(amount) if amount <= 0 || !money(amount)) {
            return Some(
                ValidationIssue::new("CARE_INVALID_RECORD").at_field("unpaid_return_year_care"),
            );
        }
        if self.prior_year.as_ref().is_some_and(|p| {
            p.injured_claimants.as_ref().is_some_and(|facts| {
                facts.people().iter().map(|p| p.expense).sum::<i64>() != p.paid_in_return_year
            })
        }) {
            return Some(
                ValidationIssue::new("INJURED_PRIOR_CARE_EXPENSE_TOTAL")
                    .at_field("injured_claimants")
                    .in_field("prior_year"),
            );
        }
        if self.prior_year.as_ref().is_some_and(|p| !p.valid()) {
            return Some(
                ValidationIssue::new("PRIOR_CARE_LINE_6_ABOVE_LINE_3")
                    .at_field("prior_line_6")
                    .in_field("prior_year"),
            );
        }
        let separate =
            d.setup.as_ref().is_some_and(|s| s.filing_status() == crate::FederalStatus::Separate);
        if self.active() && separate != matches!(self.filing, CareFilingStatus::Separate { .. }) {
            return Some(ValidationIssue::new("CARE_FILING_STATUS_INVALID").at_field("filing"));
        }
        for (field, available) in [
            ("benefits", crate::forms::form_2441::benefit_sources(d, self).is_some()),
            ("providers", self.omitted_amount().is_some()),
        ] {
            if !available {
                return Some(
                    ValidationIssue::new("CARE_SOURCE_AMOUNTS_UNAVAILABLE").at_field(field),
                );
            }
        }
        None
    }
}

impl PriorYearCare {
    pub(crate) fn credit(&self, r: &mut crate::FederalResult) -> i64 {
        use crate::year_constants::*;
        let line1 = dollars(self.prior_line_3);
        let line2 = dollars(self.paid_in_return_year);
        let line3 = line1 + line2;
        let line4 = if self.two_or_more {
            CARE_EXPENSE_LIMIT_TWO_PERSONS_2025_DOLLARS
        } else {
            CARE_EXPENSE_LIMIT_ONE_PERSON_2025_DOLLARS
        };
        let line5 = dollars(self.prior_benefits);
        let line6 =
            crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(line4, line5)
                .get();
        let line7 = dollars(self.prior_earned_smaller);
        let line8 = phase::cap(line3, [line6, line7]);
        let line9 = dollars(self.prior_line_6);
        let line10 =
            crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(line8, line9)
                .get();
        let line11 = dollars(self.prior_agi);
        let rate = phase::floor(
            CARE_CREDIT_MAXIMUM_2025_PERCENT
                - phase::steps(
                    line11 - CARE_CREDIT_PHASEOUT_START_2025_DOLLARS,
                    CARE_CREDIT_PHASEOUT_STEP_2025_DOLLARS,
                    phase::StepRounding::Up,
                ),
            CARE_CREDIT_MINIMUM_2025_PERCENT,
        );
        let credit = fraction(line10, rate, 100);
        emit(
            r,
            &mut (),
            [
                (1, line1),
                (2, line2),
                (3, line3),
                (4, line4),
                (5, line5),
                (6, line6),
                (7, line7),
                (8, line8),
                (9, line9),
                (10, line10),
                (11, line11),
                (13, credit),
            ],
            |(line, amount), _, _| {
                Some(row(
                    Some(crate::FormLine::new(
                        crate::TaxForm::Form2441,
                        &alloc::format!("Worksheet A {line}"),
                    )),
                    &alloc::format!("care_prior_worksheet_a_line{line}"),
                    LineValue::MoneyDollars(amount),
                    &["credits.care.prior_year"],
                    "https://www.irs.gov/instructions/i2441",
                    msg!("EXPLAIN_CARE_WORKSHEET_A"),
                ))
            },
        );
        crate::line_emission::form_value(
            r,
            crate::FormLine::new(crate::TaxForm::Form2441, "Worksheet A 12"),
            "care_prior_worksheet_a_line12",
            LineValue::PercentageHundredths(rate * 100),
            &["credits.care.prior_year"],
            "https://www.irs.gov/instructions/i2441",
            msg!("EXPLAIN_CARE_WORKSHEET_A"),
        );
        crate::federal_rules::note_once_scoped(
            r,
            msg!("NOTE_CARE_WORKSHEET_A_ATTACH"),
            crate::IssueContext::Section(crate::TaxSection::Care),
        );
        credit
    }
}

impl CareDeductions {
    pub(crate) fn owners(&self, d: &crate::FederalDraft) -> [i64; 2] {
        let mut out = [0; 2];
        for e in &d.income {
            out[e.owner.owned() as usize] += self.source(e.id);
        }
        out
    }
    pub(crate) fn business_projection(&self, d: &crate::FederalDraft) -> crate::FederalDraft {
        let mut out = d.clone();
        for e in &mut out.income {
            let amount = self.source(e.id);
            if amount != 0
                && let crate::FederalIncome::Business { data } = &mut e.income
            {
                data.expenses.push(crate::FederalBusinessExpense {
                    category: crate::BusinessExpenseCategory::EmployeeBenefits,
                    amount: amount * 100,
                    treatment: crate::BusinessExpenseKind::Ordinary,
                });
            }
        }
        out
    }
}
