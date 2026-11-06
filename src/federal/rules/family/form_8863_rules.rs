// form_8863 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

// Form 8863 / Pub. 970 chapter 2: an education entry that lets the App choose
// the part of the grant the student includes in income is resolved by the
// return's own calculation. Each candidate runs the whole estimate with the
// part as the student's choice (the other Automatic entries as none) and the
// part with the lowest tax less refundable credits wins, the smaller part on a
// tie; that sums the credit, the added tax and the earned income credit lost
// for a filer student, and just the credit for a dependent student, whose own
// return the App does not see. None when no entry is Automatic.

pub(super) fn resolved_scholarship_choices_with_claims(
    d: &FederalDraft,
    financial_source_scope_complete: bool,
    medical: &crate::medical_rollover_return::Prepared,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,

    participation: &alloc::sync::Arc<crate::direct_participation::DirectParticipationView>,

    macrs_elections: &alloc::sync::Arc<crate::macrs_source::MacrsElectionView>,
    source_draft: &FederalDraft,
) -> Option<FederalDraft> {
    let credits = d.credits.as_ref()?;
    let annual = credits.education_inputs();
    let automatic: Vec<usize> = (0..annual.len())
        .filter(|&i| {
            matches!(
                annual[i].expenses.scholarship_in_income,
                crate::ScholarshipInclusion::Automatic { .. }
            )
        })
        .collect();
    if automatic.is_empty() {
        return None;
    }
    let set = |draft: &mut FederalDraft, index: usize, part: i64| {
        draft
            .credits
            .as_mut()
            .unwrap()
            .students
            .iter_mut()
            .find(|row| row.student == annual[index].student)
            .unwrap()
            .more
            .scholarship_in_income = if part > 0 {
            crate::ScholarshipInclusion::Amount { included: part }
        } else {
            crate::ScholarshipInclusion::None
        };
    };
    let mut working = d.clone();
    for &index in &automatic {
        set(&mut working, index, 0);
    }
    for &index in &automatic {
        let mut best: Option<(i64, i64)> = None;
        // The credit's breakpoints and the return's own kinks (the earned income credit's
        // plateau, the deduction, the bracket edges) are the parts worth trying: the cost
        // is linear between them.
        let mut parts = crate::scholarship::candidates(&annual[index]);
        if let Some(setup) = d.setup.as_ref() {
            let base = estimate_choices_with_claims(
                &working,
                financial_source_scope_complete,
                medical,
                source_claims,
                participation,
                macrs_elections,
                source_draft,
            );
            if base.state == ResultState::EstimatedForSupportedScope {
                parts.extend(crate::scholarship::return_kinks(
                    setup.filing_status(),
                    &base,
                    crate::scholarship::most(&annual[index]),
                ));
                parts.sort_unstable();
                parts.dedup();
            }
        }
        for part in parts {
            let mut trial = working.clone();
            set(&mut trial, index, part);
            let result = estimate_choices_with_claims(
                &trial,
                financial_source_scope_complete,
                medical,
                source_claims,
                participation,
                macrs_elections,
                source_draft,
            );
            if result.state != ResultState::EstimatedForSupportedScope {
                continue;
            }
            let cost = result.tax.unwrap_or(0) - result.refundable_credits.unwrap_or(0);
            if best.is_none_or(|(_, lowest)| cost < lowest) {
                best = Some((part, cost));
            }
        }
        set(&mut working, index, best.map_or(0, |(part, _)| part));
    }
    Some(working)
}
// You, your spouse and the people you support, by their identities on the
// people page; the labels are the platform's, or the name each person typed.
pub(crate) fn people_students(d: &FederalDraft) -> Vec<PersonChoice> {
    // The SSN answer comes from the person's own page; the catalog only carries it.
    let people = d.people.as_ref();
    let mut current = vec![PersonChoice {
        id: -1,
        label: Message::new("PERSON_YOU", vec![]).encode(),
        valid_ssn: people.is_some_and(|p| p.taxpayer.valid_ssn),
    }];
    if let Some(spouse) = people.and_then(|p| p.spouse.as_ref()) {
        current.push(PersonChoice {
            id: -2,
            label: Message::new("PERSON_SPOUSE", vec![]).encode(),
            valid_ssn: spouse.valid_ssn,
        });
    }
    current.extend(d.dependents().iter().map(|v| PersonChoice {
        id: v.id,
        label: v.name(),
        valid_ssn: v.valid_ssn,
    }));
    current
}

pub(super) struct Form8863In<'a> {
    pub(super) foreign_magi: i64,
    pub(super) education_allowed: bool,
    pub(super) savings_bond_context: &'a crate::savings_bonds::SavingsBondContext<'a>,
    pub(super) education_context: &'a crate::education_account::EducationContext<'a>,
}

pub(super) struct Form8863Out {
    pub(super) aotc: i64,
    pub(super) llc: i64,
    pub(super) injured_education_sources: Vec<crate::injured_spouse::EducationSource>,
    pub(super) education_refund: i64,
    pub(super) education_nonrefund: i64,
}

pub(super) fn form_8863_phase(
    pass: &Pass,
    r: &mut FederalResult,
    i: Form8863In<'_>,
) -> Form8863Out {
    let &Pass { allocated_entries, p, c, student_catalog, j, separate, family, .. } = pass;
    let Form8863In { foreign_magi, education_allowed, savings_bond_context, education_context } = i;
    // Education credits are ordered before the child credit, following Schedule 8812's credit limit.
    let education_factor = phase::cap(
        positive(
            if j {
                crate::year_constants::EDUCATION_CREDIT_PHASEOUT_END_JOINT_DOLLARS
            } else {
                crate::year_constants::EDUCATION_CREDIT_PHASEOUT_END_OTHER_DOLLARS
            } - foreign_magi,
        ),
        [if j {
            crate::year_constants::EDUCATION_CREDIT_PHASEOUT_RANGE_JOINT_DOLLARS
        } else {
            crate::year_constants::EDUCATION_CREDIT_PHASEOUT_RANGE_OTHER_DOLLARS
        }],
    );
    let mut aotc = 0;
    let mut llc_expenses = 0;
    let mut injured_education_sources = Vec::new();
    // The expenses each entry's credit takes into account: the one definition
    // that the savings bond exclusion also reads, so no tuition earns both
    // benefits (credit first).
    let education_used = c.education_expenses_used(student_catalog, !separate && education_allowed);
    let annual_education = c.education_inputs();
    for (index, e) in annual_education.iter().enumerate() {
        debug_assert!(!e.sources.is_empty());
        crate::require_localized!(
            r,
            !annual_education[..index].iter().any(|prior| prior.student == e.student),
            &crate::IssueContext::Section(crate::TaxSection::Education),
            "Education",
            "ISSUE_USE_ONE_CREDIT_ENTRY_PER_STUDENT_DO_NOT_DUPLICATE_EXPENSES_ACROSS_C_D52B32DA",
            "Use one credit entry per student; do not duplicate expenses across credits.",
        );
        let student = student_catalog.iter().find(|v| v.id == e.student);
        crate::require_localized!(
            r,
            student.is_some_and(|v| {
                v.id == -1 || (v.id == -2 && j) || family.dependent_ids.contains(&v.id)
            }),
            &crate::IssueContext::Section(crate::TaxSection::Education),
            "Education",
            "ISSUE_THE_SELECTED_STUDENT_MUST_BE_A_FILER_OR_AN_ELIGIBLE_DEPENDENT_ON_TH_879AF16D",
            "The selected student must be a filer or an eligible dependent on this return.",
        );
        // A filer's or spouse's 1098-T box 5 above the tuition and books
        // entered may include taxable scholarship (Pub. 970 Worksheet 1-1):
        // a review note while no scholarship is entered for that person.
        let paid_for =
            e.expenses.tuition + e.expenses.school_materials + e.expenses.other_materials;
        if e.expenses.tax_free_assistance() > paid_for
            && let Some(person) = student.and_then(|v| match v.id {
                -1 => Some(Owner::Taxpayer),
                -2 => Some(Owner::Spouse),
                _ => None,
            })
            && !allocated_entries.iter().any(|entry| {
                entry.owner == crate::DocumentOwner::from(person)
                    && matches!(entry.income, FederalIncome::Scholarship { .. })
            })
        {
            note_once_scoped(
                r,
                msg!("NOTE_SCHOLARSHIP_MAY_BE_TAXABLE"),
                crate::IssueContext::Section(crate::TaxSection::OtherIncome),
            );
        }
        // Pub. 970 chapter 2: the part of the grant the student includes in
        // income. A filer's part is already in Schedule 1 line 8r; a dependent
        // student's is a note to the student's own return.
        let included = dollars(e.expenses.included());
        if included > 0 {
            let name = student.map(|v| v.label.clone()).unwrap_or_default();
            line(
                r,
                &alloc::format!("education_{}_scholarship_included", index + 1),
                included,
                &["credits.education"],
                "https://www.irs.gov/publications/p970",
                Message::new(
                    "EXPLAIN_EDUCATION_N_SCHOLARSHIP_INCLUDED",
                    alloc::vec![MessageArg::Encoded(name.clone())],
                ),
            );
            if !matches!(e.student, -1 | -2) {
                note_once_scoped(
                    r,
                    Message::new(
                        "NOTE_SCHOLARSHIP_INCLUDED_BY_STUDENT",
                        alloc::vec![MessageArg::Encoded(name), MessageArg::usd_dollars(included),],
                    ),
                    crate::IssueContext::Section(crate::TaxSection::Education),
                );
            }
        }
        // Form 8863 line 21: no credit for a student without an SSN issued
        // by the return due date (education_expenses_used takes none).
        let expense = education_used[index].used;
        if e.method() == EducationMethod::AmericanOpportunity {
            let credit = phase::cap(expense, [2000])
                + fraction(phase::cap(positive(expense - 2000), [2000]), 1, 4);
            aotc += credit;
            injured_education_sources.push(crate::injured_spouse::EducationSource {
                student: e.student,
                aotc: credit,
                llc: 0,
            });
        } else {
            llc_expenses += expense;
            injured_education_sources.push(crate::injured_spouse::EducationSource {
                student: e.student,
                aotc: 0,
                llc: expense,
            });
        }
    }
    aotc = if separate || !education_allowed {
        0
    } else {
        phase::remaining_fraction(
            aotc,
            education_factor,
            if j {
                crate::year_constants::EDUCATION_CREDIT_PHASEOUT_RANGE_JOINT_DOLLARS
            } else {
                crate::year_constants::EDUCATION_CREDIT_PHASEOUT_RANGE_OTHER_DOLLARS
            },
        )
    };
    let llc = if separate || !education_allowed {
        0
    } else {
        crate::education_recapture::lifetime_learning_credit(
            llc_expenses,
            education_factor,
            if j {
                crate::year_constants::EDUCATION_CREDIT_PHASEOUT_RANGE_JOINT_DOLLARS
            } else {
                crate::year_constants::EDUCATION_CREDIT_PHASEOUT_RANGE_OTHER_DOLLARS
            },
        )
    };
    crate::savings_bonds::note_credit_expenses(savings_bond_context, aotc, llc, r);
    crate::education_account::note_credit_expenses(education_context, aotc, llc, r);
    // Form 8863 line 7 follows the section 1(g) ages of Form 8615 (2026 i8863
    // line 7): the day-before rule, as kiddie_age tests them.
    let age = p.taxpayer.tax_age();
    let no_refund = !j
        && p.taxpayer.parent_alive
        && (age < 18
            || ((age == 18 || (age < 24 && p.taxpayer.full_time_student))
                && p.taxpayer.support_share == SupportShare::LessThanHalf));
    let education_refund = if no_refund { 0 } else { fraction(aotc, 40, 100) };
    let education_nonrefund = aotc - education_refund + llc;
    Form8863Out { aotc, llc, injured_education_sources, education_refund, education_nonrefund }
}

pub(super) struct EducationDistributionsIn<'a> {
    pub(super) education_allowed: bool,
    pub(super) qtp_excess_by_entry: &'a alloc::collections::BTreeMap<i64, i64>,
}

pub(super) struct EducationDistributionsOut<'a> {
    pub(super) education_context: crate::education_account::EducationContext<'a>,
    pub(super) education: crate::education_account::EducationTotals,
    pub(super) education_income: i64,
}

pub(super) fn education_credit_allowed(pass: &Pass<'_>) -> bool {
    let &Pass { c, p, j, .. } = pass;
    // A filer another taxpayer claims takes no education credit. The 529 and
    // Coverdell distributions read the credit's expenses (credit first), so the
    // question is settled before the income is totaled.
    // 2026 i8863 "Identification requirement" and "Who cannot claim a credit"
    // item 5: you, or on a joint return either spouse, must have a valid SSN;
    // the student's own SSN is tested separately, and only for a dependent
    // student (FederalCredits::education_expenses_used).
    if c.education.is_empty() {
        true
    } else {
        let allowed = unclaimed_filers(p);
        allowed
            && (p.taxpayer.valid_ssn
                || (j && p.spouse.as_ref().is_some_and(|spouse| spouse.valid_ssn)))
    }
}

pub(super) fn education_distributions_phase<'a>(
    pass: &Pass<'a>,
    r: &mut FederalResult,
    t: &mut Totals,
    i: EducationDistributionsIn<'a>,
) -> EducationDistributionsOut<'a> {
    let &Pass { d, c, s, separate, family, student_catalog, .. } = pass;
    let EducationDistributionsIn { education_allowed, qtp_excess_by_entry } = i;
    // Form 1099-Q: the taxable earnings of 529 and Coverdell distributions.
    let education_context = crate::education_account::EducationContext {
        qtp_excess: qtp_excess_by_entry,
        income: &d.income,
        coverdell: &d.coverdell_excess,
        credits: c,
        students: student_catalog,
        status: s.filing_status(),
        dependent_ids: &family.dependent_ids,
        credit_allowed: !separate && education_allowed,
    };
    let education = crate::education_account::calculate(&education_context, r);
    let education_income = dollars(education.income.iter().sum());
    for source in &education.sources {
        t.injured_other_sources.insert(source.id, Some(exact::whole(source.taxable)));
        t.injured_accounts.source(source.id, source.owner, exact::whole(source.taxable));
    }
    t.injured_accounts.income(education_income, education.income.map(exact::whole));
    EducationDistributionsOut { education_context, education, education_income }
}
