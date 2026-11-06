// Shared transient owner projection for Form8379 and the changed-filer
// Form3800 liability. This module neither emits form rows nor changes the draft.
use super::*;
use crate::kernel::attribution::{
    exclusive, native_allowed_bounds_for, native_unique_allowed, project_source_class,
    project_source_class_exact, sole_contributor,
};

// All amounts are actual current whole dollars, not fabricated Form1040 rows.
// The nonrefundable pool is the pre-Form3800 ordering context: it excludes GBC
// and the later MTC/bond credits which depend on GBC. Every field is mandatory.
pub(crate) struct LiabilityTotals {
    pub wages: i64,
    pub interest: i64,
    pub dividends: i64,
    pub retirement: [i64; 2],
    pub social: i64,
    pub capital: i64,
    pub total_income: i64,
    pub adjustments: i64,
    pub standard_or_itemized: i64,
    pub cash_charity: i64,
    pub child_credit: i64,
    pub other_nonrefundable: i64,
    // Includes Schedule2 line1z and PartII, like Form8379 line18.
    pub other_taxes: i64,
    pub taxable: i64,
    pub qualified_dividends: i64,
    // Actual current components in native units (whole dollars for money,
    // integer percent for care rates) from their native owners. This
    // avoids fabricating emitted rows before those rows normally exist.
    // A missing queried component is unavailable, even if its guessed zero
    // would happen to reconcile the aggregate taxable income.
    pub calculation_amounts: alloc::collections::BTreeMap<&'static str, i64>,
}

// Borrow only the actual source answers. An absent Form8379 claim supplies
// no selections; it never fabricates an injured spouse, debt or payment answer.
pub(super) struct SourceSelections<'a> {
    pub(super) injured: Owner,
    pub(super) student_loan_borrower: Option<InjuredLoanBorrower>,
    pub(super) allocations: &'a [InjuredAmount],
    pub(super) claimants: &'a [InjuredClaimantFacts],
}
impl<'a> SourceSelections<'a> {
    fn from_claim(claim: Option<&'a InjuredSpouseAllocation>) -> Self {
        Self {
            injured: claim.map_or(Owner::Taxpayer, |v| v.injured),
            student_loan_borrower: claim.and_then(|v| v.student_loan_borrower),
            allocations: claim.map_or(&[], |v| v.allocations.as_slice()),
            claimants: claim.map_or(&[], |v| v.claimants.as_slice()),
        }
    }
}

pub(super) struct AllocationProblem {
    #[cfg(test)]
    pub(super) english: String,
    pub(super) display: Message,
}

// Project only an actual native annual income category. Unknown owners remain
// unavailable even when the joint amount is zero; other known categories keep
// their own independent rounding and signed owner contributions.
fn income_category(
    part: &mut Part,
    problems: &mut Vec<AllocationProblem>,
    joint: i64,
    owners: Option<[i64; 2]>,
) {
    if let Some(owners) = owners {
        part.add(joint, owners);
    } else {
        part.unavailable(joint);
        problems.push(AllocationProblem {
            #[cfg(test)]
            english: String::from("A final native income category has unavailable source owners."),
            display: Message::new(
                "ISSUE_8379_OTHER_INCOME_UNATTRIBUTABLE",
                alloc::vec![MessageArg::usd_dollars(joint)],
            ),
        });
    }
}

pub(crate) struct OwnerProjection {
    financial_income: Part,
    pub(super) l13a: Part,
    pub(super) l13b: Part,
    pub(super) l14: Part,
    pub(super) l15: Part,
    pub(super) supplemental: Part,
    pub(super) l16: Part,
    pub(super) l17: Part,
    pub(super) l18: Part,
    child_tax: [i64; 2],     // actual Form8814 line15, native dollars
    foreign_stack: [i64; 2], // final income exclusion plus housing dollars
    foreign_election: [bool; 2],
    qualified_owners: [i64; 2],     // actual joint-rounded source dollars
    preferential_capital: [i64; 2], // actual positive long/net minimum, dollars
    capital_rates: [[i64; 2]; 2],   // admitted [28%,25%] pools, dollars
    elected_owners: [[i64; 2]; 2],  // [capital,qualified][owner], dollars
    pub(super) supplemental_qbi: Option<Part>,
    pub(super) plan: AllocationPlan,
    pub(super) missing: bool,
    pub(super) problems: Vec<AllocationProblem>,
    liability_missing: bool,
    liability_totals_valid: bool,
}

impl OwnerProjection {
    pub(crate) fn agi(&self) -> [i64; 2] {
        [0, 1].map(|i| self.financial_income.owners[i] - self.l14.owners[i])
    }
    pub(super) fn printed_agi(&self) -> [i64; 2] {
        [0, 1].map(|i| self.l13a.owners[i] + self.l13b.owners[i] - self.l14.owners[i])
    }
    pub(super) fn income_taxes(&self) -> [i64; 2] {
        self.taxes_for_agi(self.agi())
    }
    pub(super) fn printed_income_taxes(&self) -> [i64; 2] {
        self.taxes_for_agi(self.printed_agi())
    }
    fn taxes_for_agi(&self, agi: [i64; 2]) -> [i64; 2] {
        [0, 1].map(|i| {
            let taxable = positive(agi[i] - self.l15.owners[i] - self.supplemental.owners[i]);
            let preference = crate::income_tax::Preference {
                dividends: positive(self.qualified_owners[i] - self.elected_owners[1][i]),
                gain: positive(self.preferential_capital[i] - self.elected_owners[0][i]),
                line18: self.capital_rates[i][0],
                line19: self.capital_rates[i][1],
            };
            crate::income_tax::RegularTaxWorksheet {
                taxable,
                preference,
                excluded: self.foreign_stack[i],
                foreign_election: self.foreign_election[i],
                status: FederalStatus::Separate,
            }
            .tax()
                + self.child_tax[i]
        })
    }
}

// Both consumers use the same owner parts. Incomplete attribution or a zero
// denominator cannot allocate a changed-filer carryforward or joint payment.
pub(crate) fn separate_taxes(d: &FederalDraft, projection: &OwnerProjection) -> Option<[i64; 2]> {
    separate_taxes_for_income(d, projection, projection.financial_income, || {
        projection.income_taxes()
    })
}

pub(super) fn printed_separate_taxes(
    d: &FederalDraft,
    projection: &OwnerProjection,
) -> Option<[i64; 2]> {
    let mut income = projection.l13a;
    merge_income(&mut income, projection.l13b);
    separate_taxes_for_income(d, projection, income, || projection.printed_income_taxes())
}

fn merge_income(target: &mut Part, source: Part) {
    target.add(source.joint, source.owners);
    target.complete &= source.complete;
}

fn separate_taxes_for_income(
    d: &FederalDraft,
    projection: &OwnerProjection,
    income: Part,
    taxes: impl FnOnce() -> [i64; 2],
) -> Option<[i64; 2]> {
    if d.setup.as_ref()?.filing_status() != FederalStatus::Joint
        || projection.liability_missing
        || !projection.liability_totals_valid
        || [
            income,
            projection.l14,
            projection.l15,
            projection.supplemental,
            projection.l16,
            projection.l18,
        ]
        .iter()
        .any(|part| !part.complete)
    {
        return None;
    }
    let tax = taxes();
    let separate =
        [0, 1].map(|i| positive(tax[i] - projection.l16.owners[i]) + projection.l18.owners[i]);
    (separate.iter().sum::<i64>() > 0).then_some(separate)
}

pub(crate) fn project_sources(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    r: &FederalResult,
    claim: Option<&InjuredSpouseAllocation>,
    f: &JointFacts,
    before_business: Option<&LiabilityTotals>,
) -> Option<OwnerProjection> {
    let d = form_context.draft;
    let direct_participation = form_context.participation();
    let people = d.people.as_ref()?;
    let personal_present = d.income.iter().any(|entry| {
        matches!(
            entry.income,
            FederalIncome::NonbusinessProperty {
                kind: crate::NonbusinessPropertyKind::PersonalRental { .. },
                ..
            }
        )
    });
    let selections = SourceSelections::from_claim(claim);
    let a = &selections;
    let f1040 = |result: &FederalResult, line: &str| {
        before_business.map_or_else(
            || super::f1040(result, line),
            |totals| match line {
                "1a" => totals.wages,
                "2b" => totals.interest,
                "3a" => totals.qualified_dividends,
                "3b" => totals.dividends,
                "4b" => totals.retirement[0],
                "5b" => totals.retirement[1],
                "6b" => totals.social,
                "7a" => totals.capital,
                "9" => totals.total_income,
                "10" => totals.adjustments,
                "12e" => totals.standard_or_itemized,
                "12f" => totals.cash_charity,
                "19" => totals.child_credit,
                "20" => totals.other_nonrefundable,
                _ => unreachable!("liability projection does not read refund/payment totals"),
            },
        )
    };
    let missing_component = core::cell::Cell::new(false);
    let calc = |result: &FederalResult, id: &str| {
        before_business.map_or_else(
            || super::calc(result, id),
            |totals| {
                totals.calculation_amounts.get(id).copied().unwrap_or_else(|| {
                    missing_component.set(true);
                    0
                })
            },
        )
    };
    let inj = owner_index(a.injured);
    let capital_sources = f.capital.map(|source| source.by_first[inj]);
    let split = |amount, cents| crate::kernel::ratio::project_rounded_pair(amount, cents, inj);
    let split_exact = |amount, cents: [Exact; 2]| {
        let total = cents.iter().sum();
        exact::project_rounded_pair(amount, &total, cents, inj)
    };
    let single_or_exact = |amount, cents| super::single_or_exact(amount, cents, inj);
    let oth = 1 - inj;
    let mut problems: Vec<AllocationProblem> = Vec::new();
    let mut plan = AllocationPlan::default();
    let mut missing = false;
    let mut credit_ids: Vec<_> = f
        .ctc_ids
        .iter()
        .map(|id| (*id, crate::year_constants::CHILD_TAX_CREDIT_PER_CHILD_DOLLARS))
        .collect();
    credit_ids.extend(
        f.odc_ids.iter().map(|id| (*id, crate::year_constants::OTHER_DEPENDENT_CREDIT_DOLLARS)),
    );
    for (id, cap) in &credit_ids {
        if let Some(c) = a.claimants.iter().find(|c| c.person == *id) {
            plan.child_caps[usize::from(c.claimant != a.injured)] += cap * 100;
        }
    }
    let child = if before_business.is_some() && f1040(r, "19") == 0 {
        Some([0; 2])
    } else {
        child_pool(&mut plan, a, &credit_ids, f1040(r, "19"), "ctc_odc_pool", &mut missing)
    };
    let before_actc_missing = missing;
    let actc_ids: Vec<_> = f
        .ctc_ids
        .iter()
        .map(|id| (*id, crate::year_constants::ADDITIONAL_CHILD_TAX_CREDIT_PER_CHILD_DOLLARS))
        .collect();
    let additional_child = if before_business.is_some() {
        Some([0; 2])
    } else {
        child_pool(&mut plan, a, &actc_ids, f1040(r, "28"), "actc_pool", &mut missing)
    };
    let actc_missing = missing;
    missing = before_actc_missing;
    if let (Some(child), Some(additional_child)) = (child, additional_child)
        && [0, 1].iter().any(|i| {
            (child[*i] + additional_child[*i]) * 100 > plan.child_caps[usize::from(*i != inj)]
        })
    {
        missing = true;
    }
    let claimant = |amount: i64, persons: &[i64]| -> Option<[i64; 2]> {
        if amount == 0 {
            return Some([0; 2]);
        }
        let mut owner = None;
        for person in persons {
            let current = match *person {
                -1 => Some(Owner::Taxpayer),
                -2 => Some(Owner::Spouse),
                id => a.claimants.iter().find(|c| c.person == id).map(|c| c.claimant),
            }?;
            if owner.is_some_and(|prior| prior != current) {
                return None;
            }
            owner = Some(current);
        }
        let mut owners = [0; 2];
        owners[owner_index(owner?)] = amount;
        Some(owners)
    };
    let education_weights = (|| {
        let mut weights = [[0; 2]; 2];
        for source in f.education_sources {
            if source.aotc == 0 && source.llc == 0 {
                continue;
            }
            let owners = claimant(1, &[source.student])?;
            for owner in 0..2 {
                weights[0][owner] += owners[owner] * source.aotc;
                weights[1][owner] += owners[owner] * source.llc;
            }
        }
        Some(weights)
    })();
    let education_pool = |pool: i64, group: usize| -> Option<[i64; 2]> {
        if pool == 0 {
            return Some([0; 2]);
        }
        let weights = education_weights?[group];
        let total = weights.iter().sum();
        if total <= 0 {
            return None;
        }
        // Each source shares the same joint statutory phaseout/rate within
        // this pool. Project its actual allowed amount once, rounding the
        // injured column and retaining the other column's exact residual.
        let mut owners = [0; 2];
        owners[inj] = fraction(pool, weights[inj], total);
        owners[oth] = pool - owners[inj];
        Some(owners)
    };
    let care_caps = (|| {
        let credit_pool = |pool: i64, basis: i64, rate: i64, expenses: [i64; 2], shared: i64| {
            if pool == 0 {
                return Some([0; 2]);
            }
            let total = expenses.iter().sum::<i64>() + shared;
            if total <= 0 {
                return None;
            }
            if shared == 0 && dollars(total) == basis {
                // With no joint expense limit, the same statutory rate applies
                // to every actual person's expense. Preserve joint rounding.
                let mut owners = [0; 2];
                owners[inj] = fraction(pool, expenses[inj], total);
                owners[oth] = pool - owners[inj];
                Some(owners)
            } else {
                // Benefits/provider exclusions, expense/earned-income ceilings
                // can consume either claimant group's expenses. These are
                // source ceilings for a permitted direct allocation, not a
                // fabricated per-person portion of the joint limitation.
                Some(expenses.map(|expense| {
                    phase::cap(
                        pool,
                        [crate::kernel::ratio::ceiling(dollars(expense + shared) * rate, 100)],
                    )
                }))
            }
        };
        let mut expenses = [0; 2];
        if calc(r, "care_credit_before_prior") != 0 {
            for (person, expense) in &f.care_sources {
                if *expense == 0 {
                    continue;
                }
                let owners = claimant(1, &[*person])?;
                for owner in 0..2 {
                    expenses[owner] += owners[owner] * expense;
                }
            }
        }
        let current = credit_pool(
            calc(r, "care_credit_before_prior"),
            calc(r, "care_expenses_smallest"),
            calc(r, "care_percent"),
            expenses,
            f.care_shared_expenses,
        )?;
        let prior_pool = calc(r, "prior_year_care_credit");
        let prior = if prior_pool == 0 {
            [0; 2]
        } else {
            let care = d.credits.as_ref()?.care.prior_year.as_ref()?;
            let facts = care.injured_claimants.as_ref()?;
            let mut expenses = [0; 2];
            for person in facts.people() {
                expenses[owner_index(person.claimant)] += person.expense;
            }
            if expenses.iter().sum::<i64>() != care.paid_in_return_year {
                return None;
            }
            credit_pool(
                prior_pool,
                calc(r, "care_prior_worksheet_a_line10"),
                calc(r, "care_prior_worksheet_a_line12"),
                expenses,
                0,
            )?
        };
        Some([0, 1].map(|owner| current[owner] + prior[owner]))
    })();
    macro_rules! shared {
        ($label:expr, $amount:expr $(,)?) => {{
            let label = $label;
            let amount = $amount;
            shared_amount(
                &mut plan,
                a,
                label.message_key(),
                label.message_key(),
                amount,
                (phase::cap(0, [amount]) * 100, phase::floor(0, amount) * 100),
                &mut missing,
            )
        }};
    }

    // Source classification is independent of1040 line placement. Known
    // non-W2 Form2106 excess remains native1a but belongs in13b. W-2 rounds
    // first and Other receives the exact native-category dollar residual.
    let mut l13a = Part::default();
    let mut l13b = Part::default();
    let line_1a = f1040(r, "1a");
    let mut financial_income = Part::default();
    income_category(&mut financial_income, &mut problems, line_1a, split(line_1a, f.w2_wages));
    for (joint, cents) in &f.income_categories.whole_groups {
        let joint = *joint;
        let cents = cents.clone();
        income_category(
            &mut financial_income,
            &mut problems,
            joint,
            exact::project_rounded_pair(joint, &cents.iter().sum(), cents, inj),
        );
    }
    // Form2441 has already settled each owner's annual amount. Preserve its
    // financial owners even when a grace balance leaves source class unknown.
    let care_taxable = f.income_categories.care_taxable_dollars;
    let care_joint = care_taxable.iter().sum::<i64>();
    financial_income.add(care_joint, care_taxable);
    let care_w2 = [0, 1]
        .map(|i| if f.income_categories.care_taxable_from_w2[i] { care_taxable[i] } else { 0 });
    l13a.add(care_w2.iter().sum(), care_w2);
    // Form8839's known employee amounts are independent of its source
    // classification and of a joint special-needs deemed excess. Preserve
    // the exact signed annual owner values; adoption never entered legacy
    // Rest, so there is no source-cent removal or second rounding here.
    let mut adoption_joint = 0;
    if let Some(adoption) = &f.income_categories.adoption {
        let known = adoption.owners.iter().sum::<i64>();
        adoption_joint = known - adoption.deemed_excess;
        financial_income.add(known, adoption.owners);
        if adoption.source_class_complete {
            l13a.add(adoption.w2_owners.iter().sum(), adoption.w2_owners);
            let other = [0, 1].map(|i| adoption.owners[i] - adoption.w2_owners[i]);
            l13b.add(other.iter().sum(), other);
        } else {
            l13a.unavailable(0);
            l13b.unavailable(known);
            problems.push(AllocationProblem {
                #[cfg(test)]
                english: String::from("Final adoption income has known employees but incomplete W-2 source classification."),
                display: Message::new("ISSUE_8379_OTHER_INCOME_UNATTRIBUTABLE", alloc::vec![MessageArg::usd_dollars(known)]),
            });
        }
        if adoption.deemed_excess != 0 {
            // The joint exclusion beyond actual employee benefits remains a
            // separately pending attribution policy. A known employee is not
            // permission to assign that joint negative amount to the employee.
            financial_income.unavailable(-adoption.deemed_excess);
            l13a.unavailable(0);
            l13b.unavailable(-adoption.deemed_excess);
            problems.push(AllocationProblem {
                #[cfg(test)]
                english: String::from(
                    "Joint adoption deemed-excess income needs its own approved source allocation.",
                ),
                display: Message::new(
                    "ISSUE_8379_OTHER_INCOME_UNATTRIBUTABLE",
                    alloc::vec![MessageArg::usd_dollars(-adoption.deemed_excess)],
                ),
            });
        }
    }
    let other_1a = f.income_categories.line_1a_other_cents;
    let w2 = [0, 1].map(|i| f.w2_wages[i] - other_1a[i]);
    let classes = income_categories::NativeIncomeCategories::source_class_totals(
        line_1a,
        w2.map(exact::whole),
        other_1a.map(exact::whole),
    );
    if !f.income_categories.line_1a_class_unavailable
        && let Some([w2_joint, other_1a_joint]) = classes
    {
        match project_source_class(w2_joint, w2, a.injured) {
            Some(owners) => l13a.add(w2_joint, owners),
            None => {
                l13a.unavailable(w2_joint);
                problems.push(AllocationProblem { #[cfg(test)] english: String::from("Form 8379 line 13a: the actual W-2 source wages cannot be attributed between you."), display: Message::new("ISSUE_8379_W2_WAGES_UNATTRIBUTABLE", alloc::vec![]) });
            }
        }
        income_category(
            &mut l13b,
            &mut problems,
            other_1a_joint,
            project_source_class(other_1a_joint, other_1a, a.injured),
        );
    } else {
        l13a.unavailable(0);
        l13b.unavailable(0);
        problems.push(AllocationProblem { #[cfg(test)] english: String::from("Form 8379 line 13a: the W-2 wages on Form 1040 line 1a cannot be attributed between you."), display: Message::new("ISSUE_8379_W2_WAGES_UNATTRIBUTABLE", alloc::vec![]) });
    }

    // Line13b retains each native annual category's rounding before adding
    // owner columns. A combined raw-cent bucket can move a dollar between
    // exclusive owners even when its grand total happens to reconcile.
    let social = f1040(r, "6b");
    let capital = f1040(r, "7a");
    let business = calc(r, "schedule_1_business_income") + calc(r, "schedule_1_farm_income");
    let statutory_business_present = d.income.iter().any(
        |entry| matches!(&entry.income, FederalIncome::Wage { data } if crate::FederalWage::from(data).statutory_employee),
    );
    let statutory_targets =
        [calc(r, "schedule_1_business_income"), calc(r, "schedule_1_farm_income")];
    let statutory_source = f
        .income_categories
        .statutory_business
        .filter(|source| source.reconciles(statutory_targets, f.business));
    if statutory_business_present {
        if let Some(source) = statutory_source {
            for form in 0..2 {
                let w2 = source.w2_cents[form];
                let other = source.other_cents[form];
                let Some([w2_joint, other_joint]) =
                    income_categories::NativeIncomeCategories::source_class_totals(
                        source.joint_dollars[form],
                        w2.map(exact::whole),
                        other.map(exact::whole),
                    )
                else {
                    // Keep known financial ownership above; unavailable printed
                    // classes never receive an invented source residual.
                    l13a.unavailable(0);
                    l13b.unavailable(source.joint_dollars[form]);
                    problems.push(AllocationProblem {
                        #[cfg(test)]
                        english: String::from(
                            "The native business source-class amount cannot be represented.",
                        ),
                        display: Message::new("ISSUE_8379_W2_WAGES_UNATTRIBUTABLE", alloc::vec![]),
                    });
                    continue;
                };

                income_category(
                    &mut l13a,
                    &mut problems,
                    w2_joint,
                    project_source_class(w2_joint, w2, a.injured),
                );
                income_category(
                    &mut l13b,
                    &mut problems,
                    other_joint,
                    project_source_class(other_joint, other, a.injured),
                );
            }
        } else {
            l13a.unavailable(0);
            l13b.unavailable(business);
            problems.push(AllocationProblem {
                #[cfg(test)]
                english: String::from(
                    "Statutory W-2 income needs complete final Schedule C/F source classes.",
                ),
                display: Message::new(
                    "ISSUE_8379_BUSINESS_UNATTRIBUTABLE",
                    alloc::vec![MessageArg::usd_dollars(business)],
                ),
            });
        }
    }
    let interest = f1040(r, "2b");
    let dividends = f1040(r, "3b");
    let jury = if d.income.iter().any(|e| matches!(e.income, FederalIncome::JuryDuty { .. })) {
        calc(r, "jury_pay")
    } else {
        0
    };
    let unemployment =
        if d.income.iter().any(|e| matches!(e.income, FederalIncome::GovernmentPayment { .. })) {
            calc(r, "schedule_1_unemployment")
        } else {
            0
        };
    let mut category_total =
        f.income_categories.whole_groups.iter().map(|(joint, _)| joint).sum::<i64>()
            + care_joint
            + adoption_joint;
    for (joint, cents) in &f.income_categories.w2_groups {
        let joint = *joint;
        let cents = cents.clone();
        match project_source_class_exact(joint, cents, a.injured) {
            Some(owners) => l13a.add(joint, owners),
            None => {
                l13a.unavailable(joint);
                problems.push(AllocationProblem {
                    #[cfg(test)]
                    english: String::from(
                        "A native W-2 source category cannot be attributed to its owners.",
                    ),
                    display: Message::new("ISSUE_8379_W2_WAGES_UNATTRIBUTABLE", alloc::vec![]),
                });
            }
        }
    }
    if f.income_categories.source_class_unavailable {
        l13a.complete = false;
        l13b.complete = false;
        problems.push(AllocationProblem {
            #[cfg(test)]
            english: String::from(
                "Native joint income cannot yet be classified between W-2 and other sources.",
            ),
            display: Message::new(
                "ISSUE_8379_OTHER_INCOME_UNATTRIBUTABLE",
                alloc::vec![MessageArg::usd_dollars(
                    f.income_categories.unclassified_dollars
                        + if f.income_categories.line_1a_class_unavailable { line_1a } else { 0 },
                )],
            ),
        });
    }
    for (joint, cents) in &f.income_categories.groups {
        let joint = *joint;
        let cents = cents.clone();
        if let Some(owners) = project_source_class_exact(joint, cents, a.injured) {
            l13b.add(joint, owners);
        } else {
            l13b.unavailable(joint);
            problems.push(AllocationProblem {
                #[cfg(test)]
                english: String::from(
                    "Native income category cannot be attributed to its final owners.",
                ),
                display: Message::new(
                    "ISSUE_8379_OTHER_INCOME_UNATTRIBUTABLE",
                    alloc::vec![MessageArg::usd_dollars(joint)],
                ),
            });
        }
    }
    // The remaining actual source owners and permitted shared allocations are
    // resolved once. Both consumers receive the same Part, including any real
    // missing-source status; financial ownership never clears those failures.
    let mut common_income = Part::default();
    for (category, line) in ["4b", "5b"].iter().enumerate() {
        let joint = f1040(r, line);
        category_total += joint;
        if let Some(owners) = split(joint, f.retirement[category]) {
            common_income.add(joint, owners);
        } else {
            common_income.unavailable(joint);
            problems.push(AllocationProblem {
                #[cfg(test)]
                english: String::from(
                    "Final retirement category does not reconcile to its owning result.",
                ),
                display: Message::new("WARN_8379_RETIREMENT_OWNER_UNAVAILABLE", alloc::vec![]),
            });
        }
    }
    let account_joint = f.accounts.income_joint()?;
    category_total += account_joint;
    if let Some(owners) = f.accounts.income_owners(inj) {
        common_income.add(account_joint, owners);
    } else {
        common_income.unavailable(account_joint);
        problems.push(AllocationProblem {
            #[cfg(test)]
            english: String::from(
                "Final account income does not reconcile to its native source groups.",
            ),
            display: Message::new(
                "ISSUE_8379_OTHER_INCOME_UNATTRIBUTABLE",
                alloc::vec![MessageArg::usd_dollars(account_joint)],
            ),
        });
    }
    let foreign_owners = [0, 1].map(|i| f.foreign.wages[i] - f.foreign.excluded[i]);
    let foreign_joint = foreign_owners.iter().sum::<i64>();
    category_total += foreign_joint;
    if statutory_business_present && let Some(source) = statutory_source {
        // The actual native per-C2555 allocation already has owner Dollars.
        // Preserve the whole financial category and reclassify only that
        // statutory exclusion; unrelated foreign sources stay unchanged.
        financial_income.add(foreign_joint, foreign_owners);
        let w2 = source.foreign_exclusion_dollars.map(|amount| -amount);
        let other = [0, 1].map(|i| foreign_owners[i] - w2[i]);
        l13a.add(w2.iter().sum(), w2);
        l13b.add(other.iter().sum(), other);
    } else {
        common_income.add(foreign_joint, foreign_owners);
    }
    let child_joint = f.child.other_by_owner.iter().sum::<i64>();
    category_total += child_joint;
    common_income.add(child_joint, f.child.other_by_owner);
    for part in &f.regular_nol {
        let amount = part.deduction;
        let owners = part.balances.as_ref().and_then(|balances| {
            // A full deduction or sole source owner needs no allocation choice.
            use crate::kernel::ratio::exact;
            let owners = if balances[0] != exact::whole(0) && balances[1] == exact::whole(0) {
                Some([amount, 0])
            } else if balances[0] == exact::whole(0) && balances[1] != exact::whole(0) {
                Some([0, amount])
            } else {
                exact::project_rounded_pair(
                    amount,
                    &(&balances[0] + &balances[1]),
                    (*balances).clone(),
                    inj,
                )
            };
            owners.or_else(|| {
                let max_injured = exact::rounded(&phase::cap(
                    exact::whole(amount),
                    [exact::rounded_exact(&(&balances[inj] / exact::whole(100)))],
                ))
                .expect("injured maximum is bounded by the native deduction");
                let min_injured = exact::rounded(&phase::floor(
                    exact::whole(amount)
                        - exact::rounded_exact(&(&balances[oth] / exact::whole(100))),
                    exact::whole(0),
                ))
                .expect("injured minimum is bounded by the native deduction");
                shared_amount(
                    &mut plan,
                    a,
                    &part.key,
                    "INJURED_COMPONENT_REGULAR_NOL",
                    amount,
                    (min_injured * 100, max_injured * 100),
                    &mut missing,
                )
            })
        });
        // Each retained deduction already has its native annual-dollar
        // boundary. Do not mix its negative amount with another class's cents.
        category_total -= amount;
        income_category(
            &mut common_income,
            &mut problems,
            -amount,
            owners.map(|owners| owners.map(|value| -value)),
        );
    }
    let ltc_joint = f.income_categories.ltc_dollars;
    // Only a completely retained, nonnegative native category can prove that
    // every annual owner amount is zero without knowing a source-Cents split.
    // This does not turn unknown raw taxable ownership into a zero source.
    let ltc_known_zero = f.income_categories.ltc_sources_complete
        && ltc_joint == 0
        && f.ltc
            .iter()
            .try_fold(0_i64, |sum, part| {
                (part.taxable >= 0).then(|| sum.checked_add(part.taxable)).flatten()
            })
            .is_some_and(|sum| dollars(sum) == 0);
    let mut ltc_owners = [0; 2];
    let mut ltc_complete = true;
    for part in f.ltc.iter().filter(|p| p.taxable != 0 && !ltc_known_zero) {
        let Some(receipts) = part.receipts else {
            ltc_complete = false;
            continue;
        };
        let owners = match sole_contributor(part.taxable, receipts) {
            Some(owners) => owners,
            None if receipts.iter().sum::<i64>() == part.taxable => receipts,
            None => {
                let Some(bounds) = crate::kernel::attribution::allowed_bounds(
                    part.taxable,
                    [receipts[inj], receipts[oth]],
                ) else {
                    ltc_complete = false;
                    continue;
                };
                let Some(parts) = shared_amount_cents(
                    &mut plan,
                    a,
                    &part.key,
                    "INJURED_COMPONENT_LTC",
                    part.taxable,
                    bounds,
                    &mut missing,
                ) else {
                    ltc_complete = false;
                    continue;
                };
                parts
            }
        };
        for (value, amount) in ltc_owners.iter_mut().zip(owners) {
            *value += amount;
        }
    }
    category_total += ltc_joint;
    income_category(
        &mut common_income,
        &mut problems,
        ltc_joint,
        ltc_complete.then(|| split(ltc_joint, ltc_owners)).flatten(),
    );

    // Actual current receipts are gross13b income. The separate final allowed
    // expense packet belongs on14 and must not be netted or deducted again.
    let personal_sources =
        f.nii_rental.personal_inputs.iter().try_fold([0; 2], |mut owners, (id, input)| {
            let entry = d.income.iter().find(|entry| entry.id == *id)?;
            if entry.owner != crate::DocumentOwner::from(input.owner)
                || !matches!(
                    entry.income,
                    FederalIncome::NonbusinessProperty {
                        kind: crate::NonbusinessPropertyKind::PersonalRental { .. },
                        ..
                    }
                )
                || f.other_sources.get(id).cloned().flatten()
                    != Some(exact::whole(input.receipts_cents))
            {
                return None;
            }
            owners[owner_index(input.owner)] += input.receipts_cents;
            Some(owners)
        });
    let personal_joint = f.income_categories.personal_rent_dollars;
    category_total += personal_joint;
    income_category(
        &mut common_income,
        &mut problems,
        personal_joint,
        personal_sources.and_then(|cents| split(personal_joint, cents)),
    );

    // These are final native annual dollars after the owning activity limits,
    // not raw property cents or a second rental/PAL calculation. The ordinary
    // source ledger marks a missing input or unresolved461 adjustment unknown.
    let rental = f.nii_rental;
    let rental_sources = rental
        .inputs_complete
        .then(|| {
            rental.income_by_id.iter().try_fold([0; 2], |mut owners, (id, amount)| {
                let entry = d.income.iter().find(|entry| entry.id == *id)?;
                f.other_sources.get(id).cloned().flatten()?;
                owners[owner_index(entry.owner.owned())] += amount;
                Some(owners)
            })
        })
        .flatten()
        .filter(|owners| owners.iter().sum::<i64>() == rental.income);
    category_total += rental.income;
    income_category(&mut common_income, &mut problems, rental.income, rental_sources);

    //461 is the actual separate Schedule1line8p addback. Earlier native
    // source rows retain their owners; the one joint limit is allocated only
    // within established owner-NET eligible losses (dispatcher07a1fe4d).
    let ebl = &f.income_categories.excess_business_loss;
    let ebl_joint = ebl.joint_addback_dollars;
    category_total += ebl_joint;
    match ebl.bounds(inj) {
        Ok([minimum, maximum]) => {
            let owners = if minimum == maximum {
                let mut owners = [0; 2];
                owners[inj] = minimum;
                owners[oth] = ebl_joint - minimum;
                Some(owners)
            } else {
                shared_amount(
                    &mut plan,
                    a,
                    "excess_business_loss",
                    "INJURED_COMPONENT_EXCESS_BUSINESS_LOSS",
                    ebl_joint,
                    (minimum * 100, maximum * 100),
                    &mut missing,
                )
            };
            common_income.add(ebl_joint, owners);
        }
        Err(issue) => {
            common_income.unavailable(ebl_joint);
            let display = match issue {
                income_categories::Native461SourceIssue::CapitalOwnerUnavailable => {
                    Message::new("WARN_8379_EBL_CAPITAL_OWNER_UNAVAILABLE", alloc::vec![])
                }
                income_categories::Native461SourceIssue::IncompleteSources => Message::new(
                    "ISSUE_8379_BUSINESS_UNATTRIBUTABLE",
                    alloc::vec![MessageArg::usd_dollars(ebl_joint)],
                ),
            };
            problems.push(AllocationProblem {
                #[cfg(test)] english: String::from("Form8379 needs complete native461 sources and actual ownership of its business-capital allowance."),
                display,
            });
        }
    }

    let rest = f1040(r, "9")
        - line_1a
        - social
        - capital
        - business
        - interest
        - dividends
        - jury
        - unemployment
        - category_total;
    let mut capital_owners = [0; 2];
    let mut capital_complete = false;
    for (part, amount) in [
        (Part13b::Social, social),
        (Part13b::Capital, capital),
        (Part13b::Business, business),
        (Part13b::Interest, interest),
        (Part13b::Dividends, dividends),
        (Part13b::Jury, jury),
        (Part13b::Unemployment, unemployment),
        (Part13b::Rest, rest),
    ] {
        let mut w = if part == Part13b::Business {
            Weights { cents: f.business.map(exact::whole), unknown: [false; 2] }
        } else if part == Part13b::Unemployment {
            Weights { cents: f.unemployment.map(exact::whole), unknown: [false; 2] }
        } else if matches!(part, Part13b::Interest | Part13b::Dividends) {
            match f.income_categories.investment_sources[usize::from(part == Part13b::Dividends)]
                .clone()
            {
                Some(cents) => Weights { cents, unknown: [false; 2] },
                None => {
                    Weights { cents: core::array::from_fn(|_| exact::whole(0)), unknown: [true; 2] }
                }
            }
        } else {
            weights_13b(d, part, &f.other_sources, &f.accounts)
        };
        if part == Part13b::Interest {
            if let Some(exclusion) = f.savings_bond_exclusion {
                for (value, amount) in w.cents.iter_mut().zip(exclusion) {
                    *value -= exact::whole(amount) * exact::whole(100);
                }
            } else {
                w.unknown = [true; 2];
            }
        }
        if part == Part13b::Rest {
            for (value, cents) in w.cents.iter_mut().zip(&f.income_categories.removed_cents) {
                *value -= cents;
            }
            for (owner, value) in w.cents.iter_mut().enumerate() {
                // The original native per-ID map includes these components.
                // They are now projected at their actual annual boundaries,
                // rather than counted again in the remaining source classes.
                *value -= exact::whole(f.k1_portfolio[0][owner])
                    + exact::whole(f.k1_portfolio[1][owner])
                    + exact::whole(f.unemployment[owner])
                    + (exact::whole(f.child.qualified_by_owner[owner])
                        + exact::whole(f.child.other_by_owner[owner]))
                        * exact::whole(100);
            }
            if let Some(owners) = f.property_ordinary {
                for (amount, source) in w.cents.iter_mut().zip(owners) {
                    *amount += exact::whole(source);
                }
            } else {
                w.unknown = [true; 2];
            }
            // Remove only the contributions actually present in the legacy
            // ledger. LTC/NOL were never entry-ledger amounts and are no
            // longer injected here after their independent projections above.
            for (i, value) in w.cents.iter_mut().enumerate() {
                *value -= exact::whole(personal_sources.map_or(0, |owners| owners[i]));
                *value -=
                    exact::whole(rental_sources.map_or(0, |owners| owners[i])) * exact::whole(100);
            }
        }
        let owners = (part == Part13b::Social)
            .then(|| sole_class_owner(d, part))
            .flatten()
            .map(|i| {
                let mut owners = [0; 2];
                owners[i] = amount;
                owners
            })
            .or_else(|| {
                if part == Part13b::Social {
                    shared_amount(
                        &mut plan,
                        a,
                        "taxable_social_security",
                        "INJURED_COMPONENT_TAXABLE_SOCIAL_SECURITY",
                        amount,
                        AllocationBounds {
                            minimum: 0,
                            maximum: amount * 100,
                            // Approved App default: allocate the actual joint
                            // taxable benefit by the known benefit amounts.
                            // Signed repayment cases retain a direct choice.
                            initial: (!w.unknown.iter().any(|v| *v)
                                && w.cents.iter().all(|v| *v >= exact::whole(0))
                                && w.cents.iter().sum::<Exact>() > exact::whole(0))
                            .then(|| {
                                exact::rounded(
                                    &(exact::whole(amount) * &w.cents[inj]
                                        / w.cents.iter().sum::<Exact>()),
                                )
                            })
                            .flatten()
                            .and_then(|v| v.checked_mul(100)),
                        },
                        &mut missing,
                    )
                } else if part == Part13b::Capital {
                    capital_sources.and_then(|source| {
                        let nets = source.net;
                        let net = nets.iter().sum::<i64>();
                        if net == amount {
                            return Some(nets);
                        }
                        if net >= amount || amount > 0 {
                            return None;
                        }
                        // Only the actual joint annual loss floor remains.
                        // Keep an exclusive gain intact; an allowed loss can
                        // range from its actual source loss through zero.
                        let minimum = phase::floor(nets[inj], amount - positive(nets[oth]));
                        let maximum = phase::cap(positive(nets[inj]), [amount - nets[oth]]);
                        if minimum > maximum {
                            return None;
                        }
                        if minimum == maximum {
                            let mut owners = [0; 2];
                            owners[inj] = minimum;
                            owners[oth] = amount - minimum;
                            return Some(owners);
                        }
                        shared_amount(
                            &mut plan,
                            a,
                            "capital_loss_limit",
                            "INJURED_COMPONENT_CAPITAL_LOSS",
                            amount,
                            (minimum * 100, maximum * 100),
                            &mut missing,
                        )
                    })
                } else {
                    bucket(amount, w.clone(), inj)
                }
            });
        match owners {
            Some(owners) => {
                if part == Part13b::Capital {
                    capital_owners = owners;
                    capital_complete = true;
                }
                if part == Part13b::Business && statutory_business_present {
                    // Printed C/F source classes were emitted above. Preserve
                    // the original combined financial owner projection once.
                    financial_income.add(amount, owners);
                } else {
                    common_income.add(amount, owners);
                }
            }
            None => {
                if part == Part13b::Business && statutory_business_present {
                    financial_income.unavailable(amount);
                } else {
                    common_income.unavailable(amount);
                }
                let key = match part {
                    Part13b::Social => "ISSUE_8379_SOCIAL_SECURITY_UNATTRIBUTABLE",
                    Part13b::Capital => "ISSUE_8379_CAPITAL_UNATTRIBUTABLE",
                    Part13b::Business => "ISSUE_8379_BUSINESS_UNATTRIBUTABLE",
                    Part13b::Interest
                    | Part13b::Dividends
                    | Part13b::Jury
                    | Part13b::Unemployment
                    | Part13b::Rest => "ISSUE_8379_OTHER_INCOME_UNATTRIBUTABLE",
                };
                problems.push(AllocationProblem { #[cfg(test)] english: format!("Form 8379 line 13b: {} (${amount}) cannot be attributed between you: one spouse's loss offsets the other's income under a joint limit, or both of you have a document the App cannot value on its own.", part_name(part)), display: Message::new(key, alloc::vec![MessageArg::usd_dollars(amount)]) });
            }
        }
    }

    merge_income(&mut financial_income, common_income);
    merge_income(&mut l13b, common_income);

    // Line 14: Form 1040 line 10 by its Schedule 1 components. The instructions
    // allocate each adjustment "to the spouse who would have claimed it if a
    // separate return had been filed" (Instructions for Form 8379, line 14).
    let mut l14 = Part::default();
    let adjustments = f1040(r, "10");
    // Paid alimony and the recipient's recapture are the same native
    // Schedule1 category. Add the actual retained deduction once before its
    // existing annual projection; a current payer does not own the recap.
    let mut alimony = f.received_alimony_recapture;
    if let Some(v) = &d.adjustments {
        for p in &v.alimony_paid {
            if p.payment.eligible {
                alimony[owner_index(p.owner)] += p.payment.amount();
            }
        }
    }
    let moving = crate::moving::amounts(d).deduction;
    let jury = documents_by_owner(d, |entry| match &entry.income {
        FederalIncome::JuryDuty { data } => data.remitted,
        _ => 0,
    });
    let estate = documents_by_owner(d, |entry| match &entry.income {
        FederalIncome::EstateTrustPortfolio { data } => {
            crate::FederalEstateTrustPortfolioK1::from(data).final_67e_deduction
        }
        _ => 0,
    });
    let penalty = documents_by_owner(d, |entry| match &entry.income {
        FederalIncome::Interest { data } => crate::FederalInterest::from(data).early_penalty,
        FederalIncome::Oid { data } => crate::FederalOid::from(data).early_penalty,
        _ => 0,
    });
    let foreign_housing: [i64; 2] =
        core::array::from_fn(|i| f.foreign.housing_deduction[i] + f.foreign_business_housing[i]);
    let source_cents = |cents: [i64; 2]| Some(cents.map(exact::whole));
    let mut known = 0;
    for (id, label, w) in [
        (
            "foreign_housing_deduction",
            EqualAllocation::Adjustment,
            source_cents(foreign_housing.map(|v| v * 100)),
        ),
        ("olympic_exclusion", EqualAllocation::Adjustment, source_cents(f.olympic)),
        (
            "schedule_1_half_self_employment_tax",
            EqualAllocation::Adjustment,
            source_cents(f.half_se.map(|v| v * 100)),
        ),
        (
            "schedule_1_self_employed_retirement",
            EqualAllocation::Adjustment,
            source_cents(f.sep.map(|v| v * 100)),
        ),
        (
            "schedule_1_self_employed_health_insurance",
            EqualAllocation::Adjustment,
            source_cents(f.health.map(|v| v * 100)),
        ),
        ("schedule_1_hsa_deduction", EqualAllocation::Adjustment, source_cents(f.hsa)),
        ("schedule_1_early_withdrawal_penalty", EqualAllocation::Adjustment, penalty.clone()),
        ("schedule_1_alimony_paid", EqualAllocation::Adjustment, source_cents(alimony)),
        (
            "schedule_1_employee_business_expenses",
            EqualAllocation::Adjustment,
            source_cents(f.employee_expenses.map(|v| v * 100)),
        ),
        ("schedule_1_moving_expenses", EqualAllocation::Adjustment, source_cents(moving)),
        (
            "schedule_1_educator_expenses",
            EqualAllocation::Educator,
            source_cents(f.educator.map(|v| v * 100)),
        ),
        ("schedule_1_ira_deduction", EqualAllocation::Ira, source_cents(f.ira.map(|v| v * 100))),
        ("jury_pay_remitted", EqualAllocation::Adjustment, jury),
        (
            "schedule_1_personal_property_rental_expenses",
            EqualAllocation::Adjustment,
            f.personal_expenses.map(|cents| cents.map(exact::whole)),
        ),
        ("schedule_1_pension_contributions", EqualAllocation::Adjustment, source_cents(f.pension)),
        (
            "schedule_1_archer_msa_deduction",
            EqualAllocation::Adjustment,
            source_cents(
                ["archer_contribution_1_deduction", "archer_contribution_2_deduction"].map(|id| {
                    before_business.map_or_else(
                        || r.lines.iter().find(|line| line.id == id).map_or(0, |line| line.amount),
                        |_| calc(r, id) * 100,
                    )
                }),
            ),
        ),
        ("schedule_1_estate_deductions", EqualAllocation::Adjustment, estate),
        ("schedule_1_student_loan_interest", EqualAllocation::StudentLoan, None),
    ] {
        // No personal source means this component does not apply. Do not
        // require a synthetic zero row from a pre-business liability caller.
        if (id == "schedule_1_personal_property_rental_expenses" && !personal_present)
            || (id == "foreign_housing_deduction"
                && !f.foreign.election()
                && foreign_housing == [0; 2])
        {
            continue;
        }
        let amount = calc(r, id);
        known += amount;
        let owners = if id == "schedule_1_student_loan_interest" && amount != 0 {
            match a.student_loan_borrower {
                Some(InjuredLoanBorrower::Taxpayer) => Some(exclusive(amount, Owner::Taxpayer)),
                Some(InjuredLoanBorrower::Spouse) => Some(exclusive(amount, Owner::Spouse)),
                Some(InjuredLoanBorrower::Both) => shared_amount(
                    &mut plan,
                    a,
                    id,
                    label.message_key(),
                    amount,
                    (0, amount * 100),
                    &mut missing,
                ),
                None => {
                    missing = true;
                    None
                }
            }
        } else if id == "schedule_1_archer_msa_deduction" {
            if let Some(parts) = f
                .accounts
                .msa_deduction_owners(inj)
                .filter(|parts| parts.iter().sum::<i64>() == amount)
            {
                Some(parts)
            } else {
                l14.complete = false;
                None
            }
        } else if amount == 0 {
            Some([0; 2])
        } else if let Some(owners) = w.and_then(|w| split_exact(amount, w)) {
            Some(owners)
        } else {
            l14.complete = false;
            problems.push(AllocationProblem {
                #[cfg(test)]
                english: String::from(
                    "Form8379 needs the actual allowed adjustment by its source owners.",
                ),
                display: Message::new("WARN_8379_ADJUSTMENT_OWNER_UNAVAILABLE", alloc::vec![]),
            });
            None
        };
        l14.add(amount, owners);
    }
    if adjustments != known {
        l14.unavailable(adjustments - known);
        problems.push(AllocationProblem {
            #[cfg(test)]
            english: String::from(
                "Form 8379 needs final source-owner outputs for the remaining adjustments.",
            ),
            display: Message::new("WARN_8379_ADJUSTMENT_OWNER_UNAVAILABLE", alloc::vec![]),
        });
    }

    // Printed line 15 is Form 1040 line 12e; other deductions stay supplemental.
    let mut l15 = Part::default();
    let deduction = f1040(r, "12e");
    let mut accepted_itemized_owners = None;
    let pure_interest_bounds = f
        .income_categories
        .niit_itemized
        .as_deref()
        .and_then(|source| source.itemized_bounds(deduction, inj));
    if f.itemized {
        if let Some((minimum, maximum)) = pure_interest_bounds {
            if minimum == maximum {
                let mut owners = [0; 2];
                owners[inj] = minimum;
                owners[oth] = deduction - minimum;
                accepted_itemized_owners = Some(owners);
                l15.add(deduction, owners);
            } else {
                let mut unanswered = false;
                let label = EqualAllocation::Itemized;
                let owners = shared_amount(
                    &mut plan,
                    a,
                    label.message_key(),
                    label.message_key(),
                    deduction,
                    (minimum * 100, maximum * 100),
                    &mut unanswered,
                );
                missing |= unanswered;
                if unanswered {
                    // A displayed minimum is not an accepted allocation fact.
                    l15.unavailable(deduction);
                } else {
                    accepted_itemized_owners = owners;
                    l15.add(deduction, owners);
                }
            }
        } else {
            l15.add(deduction, shared!(EqualAllocation::Itemized, deduction));
        }
    } else {
        // The instructions' worksheet: half the basic standard deduction
        // each, plus each spouse's own age and blindness additions.
        let persons = [Some(&people.taxpayer), people.spouse.as_ref()];
        let additions = persons.map(|p| {
            p.map_or(0, |p| {
                crate::year_constants::STANDARD_DEDUCTION_AGE_BLIND_MARRIED_DOLLARS
                    * (i64::from(p.senior()) + i64::from(p.blind))
            })
        });
        let basic = deduction - additions[0] - additions[1];
        let halves = equal(basic);
        l15.add(deduction, [halves[0] + additions[0], halves[1] + additions[1]]);
    }
    let mut supplemental = Part::default();
    let cash = f1040(r, "12f");
    supplemental.add(cash, shared!(EqualAllocation::Charity, cash));
    let persons = [Some(&people.taxpayer), people.spouse.as_ref()];
    let seniors = persons.map(|p| i64::from(p.is_some_and(|p| p.senior())));
    for (id, w) in [
        ("senior_deduction", Some(seniors)),
        ("tips_deduction", Some(f.tips)),
        ("overtime_deduction", Some(f.overtime)),
        ("vehicle_interest_deduction", None),
    ] {
        let amount = calc(r, id);
        let owners = match w.and_then(|w| single_or_exact(amount, w)) {
            Some(owners) => Some(owners),
            None => shared_amount(
                &mut plan,
                a,
                id,
                if id == "vehicle_interest_deduction" {
                    "INJURED_COMPONENT_VEHICLE_INTEREST"
                } else {
                    "INJURED_COMPONENT_SCHEDULE_1A_QBI"
                },
                amount,
                (0, amount * 100),
                &mut missing,
            ),
        };
        supplemental.add(amount, owners);
    }

    let mut supplemental_qbi = None;
    let qbi = calc(r, "qbi_deduction");
    if qbi != 0 {
        let mut qbi_part = Part::default();
        if let Some(source) = &f.qbi {
            // The instructions allocate the actual joint deduction by each
            // spouse's QBI percentage; no separate-return eligibility or
            // wage/property/income limit is recomputed here.
            let ordinary = qbi - source.allowed_dpad;
            let total = source.weights.iter().sum::<i64>();
            if ordinary == 0 || total > 0 {
                let injured = if ordinary == 0 {
                    0
                } else {
                    fraction(ordinary, source.weights[owner_index(a.injured)], total)
                };
                let mut owners = [0; 2];
                owners[owner_index(a.injured)] = injured;
                owners[1 - owner_index(a.injured)] = ordinary - injured;
                qbi_part.add(ordinary, owners);
            } else {
                qbi_part.unavailable(ordinary);
            }
            // A cooperative's passed-through deduction belongs to the notice
            // recipient. Only a shared joint cap needs the existing bounded
            // choice, within both recipients' actual source amounts.
            let dpad = source.allowed_dpad;
            let owners = single_or_exact(dpad, source.dpad).or_else(|| {
                let injured = owner_index(a.injured);
                shared_amount(
                    &mut plan,
                    a,
                    "qbi_cooperative_dpad",
                    "INJURED_COMPONENT_QBI_DPAD",
                    dpad,
                    (
                        positive(dpad * 100 - source.dpad[1 - injured]),
                        phase::cap(source.dpad[injured], [dpad * 100]),
                    ),
                    &mut missing,
                )
            });
            qbi_part.add(dpad, owners);
        } else {
            qbi_part.unavailable(qbi);
        }
        supplemental.add(qbi, qbi_part.owners);
        supplemental.complete &= qbi_part.complete;
        if qbi_part.complete {
            supplemental_qbi = Some(qbi_part);
        }
        if !qbi_part.complete {
            problems.push(AllocationProblem {
                #[cfg(test)]
                english: String::from(
                    "Form8379 needs the actual final QBI sources and retained loss owners.",
                ),
                display: Message::new("WARN_8379_QBI_OWNER_UNAVAILABLE", alloc::vec![]),
            });
        }
    }

    // Line 16: Form 1040 line 21.
    let mut l16 = Part::default();
    l16.add(f1040(r, "19"), child);
    let other_credits = f1040(r, "20");
    let mut dependent_credits = 0;
    for key in ["care_credit", "education_credit"] {
        let amount = phase::cap(calc(r, key), [positive(other_credits - dependent_credits)]);
        dependent_credits += amount;
        let owners = if key == "care_credit" {
            care_caps.and_then(|caps| {
                if amount > caps.iter().sum() {
                    return None;
                }
                single_or_exact(amount, caps.map(|v| v * 100)).or_else(|| {
                    shared_amount(
                        &mut plan,
                        a,
                        "care_nonrefundable_credit",
                        "INJURED_COMPONENT_CARE_CREDIT",
                        amount,
                        (positive(amount - caps[oth]) * 100, phase::cap(amount, [caps[inj]]) * 100),
                        &mut missing,
                    )
                })
            })
        } else {
            education_pool(f.education_pools[0], 0).and_then(|aotc| {
                let llc = education_pool(f.education_pools[1], 1)?;
                let caps = [aotc[0] + llc[0], aotc[1] + llc[1]];
                if amount > caps.iter().sum() {
                    return None;
                }
                single_or_exact(amount, caps.map(|v| v * 100)).or_else(|| {
                    // The joint tax limit does not identify a unique allowed
                    // student amount. Keep the accepted joint credit and ask
                    // within the actual claimant-source ceilings.
                    shared_amount(
                        &mut plan,
                        a,
                        "education_nonrefundable_credit",
                        "INJURED_COMPONENT_EDUCATION_CREDIT",
                        amount,
                        (positive(amount - caps[oth]) * 100, phase::cap(amount, [caps[inj]]) * 100),
                        &mut missing,
                    )
                })
            })
        };
        if let Some(owners) = owners {
            l16.add(amount, owners);
        } else {
            l16.unavailable(amount);
            problems.push(AllocationProblem { #[cfg(test)] english: String::from("Form 8379 needs the final credit amounts for differently assigned qualifying people."),display:Message::new("WARN_8379_DEPENDENT_CREDIT_OWNER_UNAVAILABLE",alloc::vec![]) });
        }
    }
    let mut remaining_credits = other_credits - dependent_credits;
    if let Some(owners) = f.income_categories.saver_credit_owners
        && let Some(joint) = owners[0]
            .checked_add(owners[1])
            .filter(|joint| owners.iter().all(|amount| *amount >= 0) && *joint <= remaining_credits)
    {
        l16.add(joint, owners);
        remaining_credits -= joint;
    }
    let mut credit_pool_partitioned = false;
    // The pre-Form3800 liability input excludes FTC: its separate native
    // capacity owns that subtraction. Never consume an unrelated credit in
    // that view merely because its amount happens to fit the foreign credit.
    if before_business.is_none() {
        let joint = calc(r, "foreign_tax_credit");
        if joint > 0 && joint <= remaining_credits {
            let source = f
                .income_categories
                .foreign_credit_sources
                .as_deref()
                .filter(|source| source.joint == joint);
            let known_owners = source.and_then(|source| source.owners).filter(|owners| {
                owners.iter().all(|amount| *amount >= 0)
                    && owners[0].checked_add(owners[1]) == Some(joint)
            });
            let mut component = "foreign_nonrefundable_credit";
            let bounds = source.filter(|source| source.owners.is_none()).and_then(|source| {
                use income_categories::NativeForeignCreditCapacity;
                match source.capacity.as_ref()? {
                    NativeForeignCreditCapacity::PayeeCents(cents) => {
                        crate::kernel::attribution::allowed_bounds(
                            joint.checked_mul(100)?,
                            [cents[inj], cents[oth]],
                        )
                    }
                    NativeForeignCreditCapacity::CategoryDollars(dollars) => {
                        component = "foreign_ordinary_nonrefundable_credit";
                        let (minimum, maximum) = crate::kernel::attribution::allowed_bounds(
                            joint,
                            [dollars[inj], dollars[oth]],
                        )?;
                        // These bounds are reported native Dollars. Lifting
                        // them to a control's Cents invents no raw precision.
                        Some((minimum.checked_mul(100)?, maximum.checked_mul(100)?))
                    }
                }
            });
            let owners = known_owners.or_else(|| {
                let (minimum, maximum) = bounds?;
                if minimum == maximum {
                    let mut owners = [0; 2];
                    owners[inj] = dollars(minimum);
                    owners[oth] = joint - owners[inj];
                    return Some(owners);
                }
                shared_amount(
                    &mut plan,
                    a,
                    component,
                    "INJURED_COMPONENT_FOREIGN_CREDIT",
                    joint,
                    (minimum, maximum),
                    &mut missing,
                )
            });
            l16.add(joint, owners);
            if known_owners.is_none() && bounds.is_none() {
                problems.push(AllocationProblem {
                    #[cfg(test)]
                    english: String::from(
                        "Form 8379 needs the native foreign credit's final source owners.",
                    ),
                    display: Message::new(
                        "WARN_8379_FOREIGN_CREDIT_OWNER_UNAVAILABLE",
                        alloc::vec![],
                    ),
                });
            }
            remaining_credits -= joint;
            credit_pool_partitioned = true;
        }
    }
    // Retain the actual Form3800 pool through its native allowance/use
    // limits. Line16 asks once within that pool; line17 uses its complement.
    // These are already reported Dollars, not reconstructed source Cents.
    let elective_sources = before_business.is_none().then_some(()).and_then(|()| {
        let source = r.elective_sources.as_deref()?;
        (source.valid()
            && source.joint == calc(r, "schedule_3_elective_payment")
            && source.used_joint == calc(r, "general_business_credit")
            && source.used_joint <= remaining_credits)
            .then_some(source)
    });
    let mut elective_used = Vec::new();
    if let Some(source) = elective_sources {
        let mut used_groups = source.groups.iter().filter(|group| group.used > 0);
        let sole_pool = used_groups.next().is_some_and(|group| remaining_credits == group.used)
            && used_groups.next().is_none()
            && l16.joint == 0;
        for group in &source.groups {
            let bounds = group.use_bounds(inj);
            if group.used > 0 && bounds.is_none() {
                problems.push(AllocationProblem {
                    #[cfg(test)]
                    english: String::from("Form 8379 needs the native business-credit owner's final allowed and used amounts."),
                    display: Message::new("WARN_8379_BUSINESS_CREDIT_OWNER_UNAVAILABLE", alloc::vec![]),
                });
            }
            let owners = bounds.and_then(|[minimum, maximum]| {
                if minimum == maximum {
                    // The native source already determines this amount. A
                    // superseded aggregate answer cannot override its owner.
                    let mut owners = [0; 2];
                    owners[inj] = minimum;
                    owners[oth] = group.used.checked_sub(minimum)?;
                    return Some(owners);
                }
                // Converting an existing native Dollar bound to the control's
                // Cents does not claim precision absent from Form3800.
                group.used.checked_mul(100)?;
                let bounds = (minimum.checked_mul(100)?, maximum.checked_mul(100)?);
                let (key, label) = if sole_pool {
                    let key = EqualAllocation::Nonrefundable.message_key();
                    (key, key)
                } else {
                    (group.component.as_str(), group.label)
                };
                shared_amount(&mut plan, a, key, label, group.used, bounds, &mut missing)
            });
            l16.add(group.used, owners);
            credit_pool_partitioned |= group.used > 0;
            remaining_credits -= group.used;
            elective_used.push(owners);
        }
    } else if before_business.is_none()
        && (r.elective_sources.is_some()
            || calc(r, "schedule_3_elective_payment") > 0
            || calc(r, "general_business_credit") > 0)
    {
        // An inconsistent or missing packet cannot be replaced by an
        // unrestricted split of a known source-specific credit.
        let used = phase::cap(calc(r, "general_business_credit"), [remaining_credits]);
        if used > 0 {
            credit_pool_partitioned = true;
            l16.unavailable(used);
            problems.push(AllocationProblem {
                #[cfg(test)]
                english: String::from("Form 8379 needs the native business-credit owner's final allowed and used amounts."),
                display: Message::new("WARN_8379_BUSINESS_CREDIT_OWNER_UNAVAILABLE", alloc::vec![]),
            });
        }
        remaining_credits -= used;
    }
    if before_business.is_some() && remaining_credits != 0 {
        l16.unavailable(remaining_credits);
    } else if credit_pool_partitioned {
        // The old aggregate answer named a different pool. Do not silently
        // reinterpret it as ownership of this now-smaller residual category.
        l16.add(
            remaining_credits,
            shared_amount(
                &mut plan,
                a,
                "nonrefundable_credit_remainder",
                "INJURED_COMPONENT_NONREFUNDABLE_REMAINDER",
                remaining_credits,
                (0, remaining_credits * 100),
                &mut missing,
            ),
        );
    } else {
        l16.add(remaining_credits, shared!(EqualAllocation::Nonrefundable, remaining_credits));
    }

    let nonrefundable_missing = missing;
    missing |= actc_missing;
    let l17 = if before_business.is_none() {
        // Line 17: Form 1040 lines 28-31 without the EIC, the extension payment
        // and excess Social Security withholding (lines 19 and 20).
        let mut l17 = Part::default();
        l17.add(f1040(r, "28"), additional_child);
        let education_refund = f1040(r, "29");
        if let Some(owners) = (education_refund == f.education_pools[2])
            .then(|| education_pool(education_refund, 0))
            .flatten()
        {
            l17.add(education_refund, owners);
        } else {
            l17.unavailable(education_refund);
            problems.push(AllocationProblem {
                #[cfg(test)]
                english: String::from(
                    "Form 8379 needs the final refundable education credit by student claimant.",
                ),
                display: Message::new(
                    "WARN_8379_DEPENDENT_CREDIT_OWNER_UNAVAILABLE",
                    alloc::vec![],
                ),
            });
        }
        let adoption_refund = f1040(r, "30");
        l17.add(
            adoption_refund,
            shared_amount(
                &mut plan,
                a,
                "adoption_refundable_credit",
                "INJURED_COMPONENT_ADOPTION_REFUND",
                adoption_refund,
                (0, adoption_refund * 100),
                &mut missing,
            ),
        );
        let excess = f.excess_social[0] + f.excess_social[1];
        let other_refundable = f1040(r, "31") - f.extension - excess;
        let farmland_offset = farmland_installment_offset(r);
        let elective_payment = calc(r, "schedule_3_elective_payment");
        let source_specific_refund = calc(r, "undistributed_gain_tax_credit")
            + calc(r, "schedule_3_fuel_tax_credit")
            + elective_payment
            + farmland_offset;
        let refundable_remainder = other_refundable - source_specific_refund;
        let remainder_owners = if elective_payment > 0 {
            // This residual is not the former aggregate which also included
            // elective payments. Its existing direct-allocation class remains
            // the same, but an old aggregate answer cannot identify its share.
            shared_amount(
                &mut plan,
                a,
                "other_refundable_credit_remainder",
                "INJURED_COMPONENT_REFUNDABLE_REMAINDER",
                refundable_remainder,
                (0, refundable_remainder * 100),
                &mut missing,
            )
        } else {
            shared!(EqualAllocation::Refundable, refundable_remainder)
        };
        l17.add(refundable_remainder, remainder_owners);
        let fund_credit = calc(r, "undistributed_gain_tax_credit");
        if let Some(owners) = split(fund_credit, f.fund_credit) {
            l17.add(fund_credit, owners);
        } else {
            l17.unavailable(fund_credit);
        }
        let elective_owners = if elective_payment == 0 {
            Some([0; 2])
        } else {
            elective_sources.and_then(|source| {
                source.groups.iter().zip(&elective_used).try_fold(
                    [0_i64; 2],
                    |owners, (group, used)| {
                        let net = group.net_owners(*used)?;
                        Some([owners[0].checked_add(net[0])?, owners[1].checked_add(net[1])?])
                    },
                )
            })
        };
        l17.add(elective_payment, elective_owners);
        let fuel_credit = calc(r, "schedule_3_fuel_tax_credit");
        // Home and business fuel share one native reported category. Resolve
        // the actual household source in Cents, combine the genuine owner
        // operands, then project once at that category's annual boundary.
        let fuel_owners = (|| {
            let source = f.income_categories.fuel_credit_sources.as_ref()?;
            if source.home_cents < 0 || source.business_cents.iter().any(|amount| *amount < 0) {
                return None;
            }
            let whole = source.business_cents[0]
                .checked_add(source.business_cents[1])?
                .checked_add(source.home_cents)?;
            if whole != source.joint_cents || dollars(whole) != fuel_credit {
                return None;
            }
            // A closed nonnegative category reporting zero has unique annual
            // zero owners. This does not infer zero source Cents or require a
            // household choice that cannot affect this native reported amount.
            if fuel_credit == 0 {
                return Some([0; 2]);
            }
            let home = shared_amount_cents(
                &mut plan,
                a,
                "home_fuel_tax_credit",
                "INJURED_COMPONENT_HOME_FUEL",
                source.home_cents,
                (0, source.home_cents),
                &mut missing,
            )?;
            let owners = [
                source.business_cents[0].checked_add(home[0])?,
                source.business_cents[1].checked_add(home[1])?,
            ];
            split(fuel_credit, owners)
        })();
        l17.add(fuel_credit, fuel_owners);
        if !l17.complete {
            problems.push(AllocationProblem {
                #[cfg(test)]
                english: String::from(
                    "Form 8379 needs the actual source-owned refundable credit amounts.",
                ),
                display: Message::new("WARN_8379_REFUNDABLE_OWNER_UNAVAILABLE", alloc::vec![]),
            });
        }

        // Keep the actual printed joint total, with owner columns unavailable.
        // Form1062's offset must never create a shared-refundable question.
        if farmland_offset != 0 {
            l17.unavailable(farmland_offset);
        }

        l17
    } else {
        Part::default()
    };
    let refundable_missing = missing;
    missing = nonrefundable_missing;

    // Line 18 includes Schedule 2 line 1z and Part II (1040 line 23).
    let mut l18 = Part::default();
    let repayments = calc(r, "schedule_2_credit_repayments_total");
    let other_taxes =
        before_business.map_or_else(|| f1040(r, "23") + repayments, |totals| totals.other_taxes);
    let se_tax = f.se[0] + f.se[1];
    l18.add(se_tax, f.se);
    let investment = &f.income_categories.investment_sources;
    // A final Form 8960 total alone cannot assign passive activity income,
    // deductible investment costs or capital-loss limits to either spouse.
    let nii_kinds_complete = d.income.iter().all(|e| {
        matches!(
            e.income,
            FederalIncome::Wage { .. }
                | FederalIncome::EmployeeWages { .. }
                | FederalIncome::ForeignEmployment { .. }
                | FederalIncome::Interest { .. }
                | FederalIncome::Oid { .. }
                | FederalIncome::Dividend { .. }
                | FederalIncome::Retirement { .. }
                | FederalIncome::InheritedIra { .. }
                | FederalIncome::Roth { .. }
                | FederalIncome::SocialSecurity { .. }
                | FederalIncome::Alimony { .. }
                | FederalIncome::JuryDuty { .. }
                | FederalIncome::Scholarship { .. }
                | FederalIncome::Gambling { .. }
                | FederalIncome::HobbySales { .. }
                // These native ordinary-income categories affect MAGI, but
                // none enters the existing gross_nii collector. Their presence
                // cannot make separately complete investment owners unknown.
                | FederalIncome::HealthLoanAssistance { .. }
                | FederalIncome::WildfireRelief { .. }
                | FederalIncome::DisasterRelief { .. }
                | FederalIncome::CasualtyLivingInsurance { .. }
                | FederalIncome::TrumpDeath { .. }
                | FederalIncome::Sale { .. }
                | FederalIncome::InstallmentSale { .. }
                | FederalIncome::HomeSale { .. }
                | FederalIncome::NuaSale { .. }
                | FederalIncome::EmployeeStockSale { .. }
                | FederalIncome::EsppSale { .. }
                | FederalIncome::EsppSaleSource { .. }
                | FederalIncome::IsoSaleSource { .. }
                | FederalIncome::NuaSaleSource { .. }
                | FederalIncome::IsoSale { .. }
                | FederalIncome::WorthlessSecurity { .. }
                | FederalIncome::SmallBusinessStock { .. }
                | FederalIncome::ReturnOfCapital { .. }
                | FederalIncome::UndistributedGain { .. }
                | FederalIncome::QualifiedSmallBusinessSale { .. }
                | FederalIncome::OpportunityFund { .. }
                | FederalIncome::OpportunityFundDeferral { .. }
                | FederalIncome::NonbusinessBadDebt { .. }
                | FederalIncome::Section1256 { .. }
                | FederalIncome::DisasterMitigation { .. }
                | FederalIncome::TrumpAbleRollover { .. }
                | FederalIncome::EducationAccountExpenses { .. } // The native Form8960 business adjustment excludes this accepted
                                                                 // materially participated C/F income. Its presence changes MAGI,
                                                                 // not the source of the actual investment-income tax. Passive
                                                                 // business, property character and deductions still need their own
                                                                 // source outputs and the reconciliation below remains mandatory.
        ) || matches!(&e.income,
            FederalIncome::Broker { data }
                if crate::broker_calculation::known_components(data, true, true)
        ) || matches!(&e.income,
            // These supporting records contribute through the existing native
            // capital collector, never an independent Rest/NII amount. The
            // complete final capital owner packet and NII footing below still
            // apply; crop/business dispositions need their own final feed.
            FederalIncome::PropertyAcquisition { data }
                if {
                    let data = crate::PropertyAcquisition::from(data);
                    data.crop.is_none()
                        && matches!(data.property,
                            Some(crate::TransferredProperty::PersonalUse { .. }
                                | crate::TransferredProperty::Investment { .. }))
                }
        ) || matches!(&e.income,
            FederalIncome::NonbusinessProperty {
                kind: crate::NonbusinessPropertyKind::PersonalRental { .. }, ..
            } if f.personal_expenses.is_some()
        ) || matches!(&e.income,
            FederalIncome::Business { data }
                if direct_participation.get(e.id) == crate::participation::Participation::Yes
                    // Passive C retains its native NIIT classification. The
                    // reconciled statutory source proves wage character;
                    // other sources need their completed regular PAL packet.
                    || (data.current_passive_supported(direct_participation.get(e.id))
                        && f.income_categories.niit_passive_business.is_some()
                        && ((statutory_source.is_some() && crate::statutory_wages::business(d, e.id))
                        || (f.nii_rental.inputs_complete
                        && f.nii_rental.source_issue.is_none()
                        && f.nii_rental.pool_issue.is_none()
                        && f.nii_rental.post_pal.as_ref().is_some_and(|packet| {
                            !packet.alternative
                                && packet.sources.iter().any(|source| {
                                    source.income == e.id
                                        && source.owner == e.owner.owned()
                                        && source.component == crate::extended_rules::PostPalComponent::DirectBusiness
                                        && source.common_pool
                                })
                        }))))
        ) || (matches!(
            e.income,
            FederalIncome::OtherIncome { .. }
                | FederalIncome::GovernmentPayment { .. }
                | FederalIncome::Nec1099 { .. }
                | FederalIncome::Misc1099 { .. }
                | FederalIncome::K1099 { .. }
                    | FederalIncome::ChildElection { .. }
                | FederalIncome::PersonalDebt { .. }
                | FederalIncome::SavingsBonds { .. }
                | FederalIncome::Cooperative { .. }
                | FederalIncome::Rental { .. }
                | FederalIncome::Partnership { .. }
                | FederalIncome::Scorp { .. }
                | FederalIncome::PartnershipActivities { .. }
                | FederalIncome::ScorpActivities { .. }
                | FederalIncome::EstateTrustPortfolio { .. }
                | FederalIncome::EducationDistribution { .. }
                | FederalIncome::Able { .. }
                | FederalIncome::MedicalDistribution { .. }
                | FederalIncome::Royalty { .. }
                | FederalIncome::FarmRental { .. }
        ) && (f.accounts.contains(e.id)
            || f.other_sources.get(&e.id).is_some_and(Option::is_some)))
            || (matches!(
                e.income,
                FederalIncome::LtcBenefits { .. } | FederalIncome::LtcInsured { .. }
            ) && f.nii_ltc_sources.contains_key(&e.id))
    });
    let niit = calc(r, "net_investment_income_tax");
    let nii_owners = (|| {
        // No source choice is needed to allocate a genuinely zero tax pool.
        if niit == 0 {
            return Some([0; 2]);
        }
        if !nii_kinds_complete || !capital_complete || f.savings_bond_exclusion.is_none() {
            return None;
        }
        let savings_penalty = calc(r, "schedule_1_early_withdrawal_penalty");
        let estate_deduction = f.k1_nii.targets[3];
        let itemized = if let Some(source) = &f.income_categories.niit_itemized {
            let owners =
                if source.same_itemized_capacities.or(source.pure_interest_capacities).is_some() {
                    // This verified pool is the same actual itemized component.
                    // Reuse its one valid accepted R5 fact or automatic singleton.
                    accepted_itemized_owners
                } else if let Some(owners) = source.owners_dollars {
                    Some(owners)
                } else {
                    let (minimum, maximum) =
                        source.subset_bounds(accepted_itemized_owners?, inj)?;
                    if minimum == maximum {
                        let mut owners = [0; 2];
                        owners[inj] = minimum;
                        owners[oth] = source.joint_dollars - minimum;
                        Some(owners)
                    } else {
                        let mut unanswered = false;
                        let owners = shared_amount(
                            &mut plan,
                            a,
                            "niit_itemized_deduction",
                            "INJURED_COMPONENT_NIIT_ITEMIZED_DEDUCTION",
                            source.joint_dollars,
                            (minimum * 100, maximum * 100),
                            &mut unanswered,
                        );
                        missing |= unanswered;
                        owners
                    }
                }?;
            (owners.iter().sum::<i64>() == source.joint_dollars).then_some(owners)?
        } else {
            // A synthetic inventory without this new source packet retains
            // its former known-zero admission, never a guessed deduction.
            [0; 2]
        };
        if calc(r, "form_8960_deductions_and_modifications")
            != savings_penalty + estate_deduction + itemized.iter().sum::<i64>()
        {
            return None;
        }
        let mut interest_cents = investment[0].clone()?;
        for (cents, exclusion) in interest_cents.iter_mut().zip(f.savings_bond_exclusion?) {
            *cents -= exact::whole(exclusion) * exact::whole(100);
        }
        let interest_total = interest_cents.iter().sum();
        let interest =
            exact::project_rounded_pair(f1040(r, "2b"), &interest_total, interest_cents, inj)?;
        let dividend_cents = investment[1].clone()?;
        let dividend_total = dividend_cents.iter().sum();
        let dividends =
            exact::project_rounded_pair(f1040(r, "3b"), &dividend_total, dividend_cents, inj)?;
        let annuities = split(calc(r, "nonqualified_annuity_investment_income"), f.nii_annuity)?;
        let penalty_cents = penalty.clone()?;
        let penalty_total = penalty_cents.iter().sum::<Exact>();
        if penalty_total != f.nii_savings_penalty.into_iter().map(exact::whole).sum::<Exact>() {
            return None;
        }
        let penalties =
            exact::project_rounded_pair(savings_penalty, &penalty_total, penalty_cents, inj)?;
        let substitutes = split(dollars(f.nii_substitute.iter().sum()), f.nii_substitute)?;
        let mut royalty_cents = f.k1_nii.royalties.map(exact::whole);
        for entry in &d.income {
            if matches!(entry.income, FederalIncome::Royalty { .. }) {
                royalty_cents[owner_index(entry.owner.owned())] +=
                    f.other_sources.get(&entry.id).cloned().flatten()?;
            }
        }
        let royalty_total = royalty_cents.iter().sum();
        let royalties =
            exact::project_rounded_pair(f.nii_royalties, &royalty_total, royalty_cents, inj)?;
        let rental_owners = f.nii_rental.niit_owners(d)?;
        let mut k1_categories = [[0; 2]; 4];
        for (index, cents) in f.k1_nii.categories.iter().enumerate() {
            k1_categories[index] = split(f.k1_nii.targets[index], *cents)?;
        }
        let loans = split(f.k1_nii.targets[4], f.k1_nii.loans)?;
        let ordinary_cut_cents = f.k1_nii.ordinary_cuts?.map(|amount| amount.get());
        let ordinary = split(
            f.k1_nii.targets[5],
            [0, 1].map(|owner| f.k1_nii.ordinary[owner] + ordinary_cut_cents[owner]),
        )?;
        let personal_net = if personal_present {
            let personal_receipts = documents_by_owner(d, |entry| match &entry.income {
                FederalIncome::NonbusinessProperty {
                    kind: crate::NonbusinessPropertyKind::PersonalRental { .. },
                    receipts,
                    ..
                } => *receipts,
                _ => 0,
            });
            let gross = split_exact(calc(r, "nonbusiness_personal_rent"), personal_receipts?)?;
            let expenses = split(
                calc(r, "schedule_1_personal_property_rental_expenses"),
                f.personal_expenses?,
            )?;
            [0, 1].map(|owner| gross[owner] - expenses[owner])
        } else {
            // A genuinely empty source class is known, not a missing feed.
            [0; 2]
        };
        let (hobby_target, hobby_cents) = f.income_categories.niit_hobby.as_ref()?;
        let hobby_total = hobby_cents.iter().sum();
        let hobby =
            exact::project_rounded_pair(*hobby_target, &hobby_total, hobby_cents.clone(), inj)?;
        let passive_business = match f.income_categories.niit_passive_business {
            Some((target, cents)) => split(target, cents)?,
            None => {
                if d.income.iter().any(|entry| {
                    matches!(&entry.income,
                    FederalIncome::Business { data }
                    if data.current_passive_supported(direct_participation.get(entry.id))
                        && !crate::statutory_wages::business(d, entry.id))
                }) {
                    return None;
                }
                [0; 2]
            }
        };
        let base_owners: [i64; 2] = core::array::from_fn(|i| {
            passive_business[i]
                + interest[i]
                + dividends[i]
                + annuities[i]
                + substitutes[i]
                + royalties[i]
                + rental_owners[i]
                + k1_categories[0][i]
                + k1_categories[1][i]
                + k1_categories[2][i]
                + loans[i]
                + ordinary[i]
                + personal_net[i]
                + hobby[i]
                + f.child.investment_by_owner[i]
                + capital_owners[i]
                - penalties[i]
                - k1_categories[3][i]
                - itemized[i]
        });
        let source = f.income_categories.niit_retained.as_ref()?;
        let adjustment = calc(r, "niit_disposition_adjustment");
        // Do not ask for a retained-use choice while some other source is
        // missing. This is the existing final native NII, not another8960.
        if source.used_dollars != adjustment
            || base_owners.iter().sum::<i64>() + adjustment
                != calc(r, "form_8960_net_investment_income")
        {
            return None;
        }
        let prior = capital::niit_difference_owners(d)?;
        let (used, minimum, maximum, current_owners) =
            if let Some(nua) = f.income_categories.niit_current_nua_cents {
                // This packet confirms each actual NUA owner and every other
                // current component's zero. J==0 alone proves neither axis.
                let excluded = dollars(nua[0].checked_add(nua[1])?);
                let (used, minimum, maximum) = source.bounds_with_current(prior, inj, excluded)?;
                let current = single_or_exact(excluded, nua)?;
                (used, minimum, maximum, current)
            } else {
                // Older genuine prior-only packets retain their explicit
                // per-component zero proof; an absent proof stays unknown.
                let (minimum, maximum) = source.bounds(prior, inj)?;
                (source.used_dollars, minimum, maximum, [0; 2])
            };
        let prior_owners = if minimum == maximum {
            let mut owners = [0; 2];
            owners[inj] = minimum;
            owners[oth] = used - minimum;
            owners
        } else {
            // The retained-use axis is independent of current NUA and regular
            // capital ownership. Its existing question represents U, not J.
            shared_amount(
                &mut plan,
                a,
                "niit_retained_difference_used",
                "INJURED_COMPONENT_NIIT_RETAINED_USE",
                used,
                (minimum * 100, maximum * 100),
                &mut missing,
            )?
        };
        let adjustment_owners = [0, 1].map(|i| prior_owners[i] - current_owners[i]);
        if adjustment_owners.iter().sum::<i64>() != adjustment {
            return None;
        }
        let owners = [0, 1].map(|i| base_owners[i] + adjustment_owners[i]);
        (owners.iter().all(|v| *v >= 0)
            && owners.iter().sum::<i64>() == calc(r, "form_8960_net_investment_income"))
        .then_some(owners)
    })();
    if niit != 0 {
        let owners = nii_owners.and_then(|sources| match sole_contributor(niit, sources) {
            Some(owners) => Some(owners),
            None if sources.iter().sum::<i64>() > 0 => {
                // Instructions8379 line18: use the same actual NII allocation
                // as line13b. The joint MAGI ceiling limits the one tax pool;
                // it does not replace those source shares or create MFS NIIT.
                let mut owners = [0; 2];
                owners[inj] = fraction(niit, sources[inj], sources.iter().sum());
                owners[oth] = niit - owners[inj];
                Some(owners)
            }
            _ => None,
        });
        if let Some(owners) = owners {
            l18.add(niit, owners);
        } else {
            l18.unavailable(niit);
            problems.push(AllocationProblem {
                #[cfg(test)]
                english: String::from(
                    "Form 8379 needs complete final net-investment income by owner.",
                ),
                display: Message::new("WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE", alloc::vec![]),
            });
        }
    }
    let additional_medicare = calc(r, "additional_medicare_tax");
    let medicare_capacities = f.income_categories.additional_medicare.as_ref().map_or_else(
        || {
            // Older synthetic portfolio contexts can establish only this closed
            // ordinary-W2 path. They cannot invent zero SE/RRTA/covered pay or
            // rely on raw ScheduleC profit as the actual ScheduleSE source.
            let mut wages = [0i64; 2];
            let closed = d.income.iter().all(|entry| match &entry.income {
                FederalIncome::Wage { data }
                    if {
                        let data = crate::FederalWage::from(data);
                        matches!(data.payroll, crate::W2Payroll::SocialSecurity)
                            && data.employee_tips.is_none()
                            && !data.statutory_employee
                    } =>
                {
                    let data = crate::FederalWage::from(data);
                    wages[owner_index(entry.owner.owned())] += data.medicare_wages;
                    true
                }
                FederalIncome::Interest { .. }
                | FederalIncome::Dividend { .. }
                | FederalIncome::Sale { .. }
                | FederalIncome::InstallmentSale { .. }
                | FederalIncome::QualifiedSmallBusinessSale { .. } => true,
                FederalIncome::Broker { data } => {
                    crate::broker_calculation::known_components(data, true, false)
                }
                _ => false,
            }) && d
                .screening
                .as_ref()
                .is_some_and(|screen| screen.clergy.is_empty() && screen.optional_se.is_empty());
            (closed && wages == f.medicare_wages && f.business == [0; 2] && f.se == [0; 2])
                .then(|| wages.map(|amount| if amount > 0 { additional_medicare } else { 0 }))
        },
        |source| source.capacities(additional_medicare),
    );
    let medicare_owners = if additional_medicare == 0 {
        Some([0; 2])
    } else {
        medicare_capacities.and_then(|capacities| {
            let bounds = crate::kernel::attribution::allowed_bounds(
                additional_medicare,
                [capacities[inj], capacities[oth]],
            )?;
            crate::kernel::attribution::unique_allowed(additional_medicare, capacities).or_else(
                || {
                    shared_amount(
                        &mut plan,
                        a,
                        "additional_medicare_tax",
                        "INJURED_COMPONENT_OTHER_TAX",
                        additional_medicare,
                        (bounds.0 * 100, bounds.1 * 100),
                        &mut missing,
                    )
                },
            )
        })
    };
    if let Some(owners) = medicare_owners {
        l18.add(additional_medicare, owners);
    } else {
        l18.unavailable(additional_medicare);
        problems.push(AllocationProblem {
            #[cfg(test)]
            english: String::from(
                "Form 8379 needs complete native Additional Medicare tax sources by owner.",
            ),
            display: Message::new("WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE", alloc::vec![]),
        });
    }
    let deferred_tax = f.deferred_compensation_tax.joint_dollars;
    if let Some(owners) = f.deferred_compensation_tax.owners(inj) {
        l18.add(deferred_tax, owners);
    } else {
        l18.unavailable(deferred_tax);
    }
    let mut taxed = se_tax + niit + additional_medicare + deferred_tax;
    let employee_wage_taxes = f.income_categories.employee_wage_tax_dollars.or_else(|| {
        // A missing packet cannot establish observed payroll zero for any
        // present EmployeeWages record, including incomplete coverage. Only
        // the real closed native call can retain that omitted-tax zero.
        (!d.income.iter().any(|entry| matches!(&entry.income, FederalIncome::EmployeeWages { .. })))
            .then_some([0; 2])
    });
    if let Some(owners) = employee_wage_taxes {
        let joint = owners.iter().sum();
        l18.add(joint, owners);
        taxed += joint;
    } else {
        l18.unavailable(0);
    }
    if let Some(source) = &f.income_categories.other_payroll_taxes {
        let joint = source.joint();
        taxed += joint;
        if let Some(owners) = source.owners(inj) {
            l18.add(joint, owners);
        } else {
            l18.unavailable(joint);
        }
    } else if d.income.iter().any(|entry| match &entry.income {
        FederalIncome::Wage { data } => {
            let data = &crate::FederalWage::from(data);
            data.employee_tips.is_some()
                || data.uncollected_tax.is_some()
                || data.other_box12.iter().any(|row| matches!(row.code, crate::W2Code::K))
        }
        FederalIncome::Nec1099 { data } => {
            crate::FederalForm1099Nec::from(data).golden_parachute != 0
        }
        _ => false,
    }) {
        // Absent retained metadata is not an observed zero for a present
        // source, even if another source would cancel its rounded amount.
        l18.unavailable(0);
    }
    let property_recapture = f.investment_recapture.iter().sum();
    l18.add(property_recapture, f.investment_recapture);
    taxed += property_recapture;
    let mortgage = f.mortgage_recapture;
    taxed += mortgage.joint;
    if !mortgage.complete {
        l18.unavailable(mortgage.joint);
        problems.push(AllocationProblem {
            #[cfg(test)]
            english: String::from(
                "Form 8379 needs the actual source owner of mortgage subsidy recapture tax.",
            ),
            display: Message::new("WARN_8379_MORTGAGE_TAX_OWNER_UNAVAILABLE", alloc::vec![]),
        });
    } else if mortgage.shared == 0 {
        l18.add(mortgage.joint, mortgage.exclusive);
    } else {
        l18.add(
            mortgage.joint,
            shared_amount(
                &mut plan,
                a,
                "mortgage_subsidy_recapture_tax",
                "INJURED_COMPONENT_OTHER_TAX",
                mortgage.joint,
                (mortgage.exclusive[inj] * 100, (mortgage.exclusive[inj] + mortgage.shared) * 100),
                &mut missing,
            ),
        );
    }
    if let Some(owners) = f.accounts.tax_owners(inj) {
        let joint = owners.iter().sum();
        l18.add(joint, owners);
        taxed += joint;
    } else {
        l18.unavailable(0);
    }
    for ids in [
        ["early_taxpayer_tax", "early_spouse_tax"],
        ["account_taxpayer_tax", "account_spouse_tax"],
        ["rmd_0_tax", "rmd_1_tax"],
    ] {
        let owners = ids.map(|id| calc(r, id));
        let amount = owners.iter().sum();
        taxed += amount;
        l18.add(amount, owners);
    }
    if d.screening.as_ref().is_some_and(|s| !s.household_employment.is_empty()) {
        // ScheduleH emits final tax by employer identity, not row order.
        // With no employer records the native engine emits no ScheduleH;
        // an applicable before-business projection must supply these keys.
        let owners = ["household_0_tax", "household_1_tax"].map(|id| calc(r, id));
        let amount = owners.iter().sum();
        taxed += amount;
        l18.add(amount, owners);
    }
    // APTC repayment is a joint component the instructions allow the spouses
    // to divide as determined. Source-specific additional taxes are not.
    let premium_repayment = calc(r, "schedule_2_premium_credit_repayment");
    l18.add(
        premium_repayment,
        shared_amount(
            &mut plan,
            a,
            "schedule_2_premium_credit_repayment",
            "INJURED_COMPONENT_OTHER_TAX",
            premium_repayment,
            (0, premium_repayment * 100),
            &mut missing,
        ),
    );
    let vehicle_repayment = f.vehicle_repayment.iter().sum::<i64>();
    l18.add(vehicle_repayment, f.vehicle_repayment);
    taxed += premium_repayment + vehicle_repayment;
    if other_taxes != taxed {
        l18.unavailable(other_taxes - taxed);
        problems.push(AllocationProblem { #[cfg(test)] english:String::from("Form 8379 needs final source-owner outputs for the remaining account, recapture or household taxes."),display:Message::new("WARN_8379_OTHER_TAX_OWNER_UNAVAILABLE",alloc::vec![]) });
    }

    let qualified = documents_by_owner(d, |entry| {
        crate::investment_statements::components(entry).map_or(0, |parts| parts.qualified_dividends)
    });
    let qualified_total = f1040(r, "3a");
    let qualified = qualified
        .and_then(|mut cents| {
            for (owner, value) in cents.iter_mut().enumerate() {
                *value += exact::whole(f.k1_portfolio[2][owner])
                    + exact::whole(f.child.qualified_by_owner[owner]) * exact::whole(100);
            }
            split_exact(qualified_total, cents)
        })
        .unwrap_or_else(|| {
            missing = true;
            [0; 2]
        });
    let mut elected_owners = [[0; 2]; 2];
    for (class, key, label, capacity) in [
        (
            0,
            "investment_elected_capital_gain",
            "INJURED_COMPONENT_INVESTMENT_ELECTED_CAPITAL",
            f.investment_election.capital.map(|views| views[inj]),
        ),
        (
            1,
            "investment_elected_qualified_dividends",
            "INJURED_COMPONENT_INVESTMENT_ELECTED_DIVIDENDS",
            Some(qualified),
        ),
    ] {
        let elected = f.investment_election.elected[class];
        if elected == 0 {
            continue;
        }
        let bounds = capacity
            .filter(|_| elected > 0)
            .and_then(|parts| native_allowed_bounds_for(elected, parts, a.injured));
        if let Some((minimum, maximum)) = bounds {
            elected_owners[class] = if let Some(parts) =
                capacity.and_then(|parts| native_unique_allowed(elected, parts))
            {
                parts
            } else {
                shared_amount(
                    &mut plan,
                    a,
                    key,
                    label,
                    elected,
                    (minimum * 100, maximum * 100),
                    &mut missing,
                )
                .unwrap_or([0; 2])
            };
        } else {
            missing = true;
            problems.push(AllocationProblem {
                #[cfg(test)]
                english: String::from("The actual eligible investment sources of the Form 4952 election must be established before the Form 8379 liability estimate can be calculated."),
                display: Message::new("WARN_8379_INVESTMENT_ELECTION_OWNER_UNAVAILABLE", alloc::vec![]),
            });
        }
    }
    let liability_missing = missing;
    missing |= refundable_missing;
    // QDCGT worksheet line3 uses positive min(net long gain, total net gain).
    // Keep each source's actual long character and the already allocated joint
    // capital loss; do not rerun MFS carryovers or its different loss floor.
    let preferential_capital = capital_sources.map_or([0; 2], |source| {
        core::array::from_fn(|i| positive(phase::cap(capital_owners[i], [source.long[i]])))
    });
    let liability_totals_valid = !missing_component.get()
        && before_business.is_none_or(|totals| {
            positive(financial_income.joint - l14.joint - l15.joint - supplemental.joint)
                == totals.taxable
                && qualified.iter().sum::<i64>() == totals.qualified_dividends
        });
    Some(OwnerProjection {
        financial_income,
        l13a,
        l13b,
        l14,
        l15,
        supplemental,
        l16,
        l17,
        l18,
        child_tax: f.child.tax_by_owner,
        foreign_stack: core::array::from_fn(|i| f.foreign.excluded[i] + foreign_housing[i]),
        foreign_election: f.foreign.elected,
        qualified_owners: qualified,
        preferential_capital,
        capital_rates: capital_sources.map_or([[0; 2]; 2], |source| source.rates),
        elected_owners,
        supplemental_qbi,
        plan,
        missing,
        problems,
        liability_missing,
        liability_totals_valid,
    })
}
