use super::*;

#[test]
fn newborn_residence_days_are_real_days_and_exception_preserves_credits() {
    let mut d = complete(FederalStatus::Head);
    d.income[0] = wage(1, Owner::Taxpayer, 3000000, 0);
    add_child(&mut d);
    d.dependents[0].set_care_expenses(10000);
    let older = {
        document_care(&mut d);
        supported(&d)
    };
    d.dependents[0].birth_date = 20261231;
    assert!(!d.valid_stored());
    d.dependents[0].residency = DependentResidency::Days { days: 1 };
    assert!(d.valid_stored());
    d.dependents[0].residency = DependentResidency::WholeYearException;
    let newborn = {
        document_care(&mut d);
        supported(&d)
    };
    for key in ["care_credit", "child_credit", "earned_income_credit"] {
        assert_eq!(value(&newborn, key), value(&older, key));
    }
    assert_eq!(restart(d.clone()).draft, d);
}

// C-fixture (2026-09-25, matrix row 22, form-8863.md): a full MFJ return with
// one AOTC student (a dependent) and one LLC student (the filer), computed by
// hand against the 2026 draft Form 8863/Pub 970 formulas already audited in
// federal_rules.rs (education_allowed/education_factor/aotc/llc/no_refund).
#[test]
fn education_credits_return_matches_hand_computation() {
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19710615;
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().birth_date = 19730815;
    // Single W-2: box 1 $118,000, box 2 $12,500, boxes 3/5 = box 1 and boxes
    // 4/6 follow from `wage()`'s standard 6.2%/1.45% computation (consistent).
    d.income[0] = wage(1, Owner::Taxpayer, 11_800_000, 1_250_000);
    // Dependent child born 2006 (age 20 in 2026): a half-time (not full-time)
    // second-year undergraduate. `student` stays false (half-time only), so
    // the qualifying-child age test fails, but the qualifying-relative test
    // (support/gross-income, from `child()`'s defaults) still makes this a
    // dependent -- the $500 other-dependent credit, not the child credit.
    add_child(&mut d);
    d.dependents[0].birth_date = 20060615;
    // Regenerate the hidden student catalog now that the dependent exists:
    // index 0 = You, index 1 = Your spouse, index 2 = the dependent.
    let sid0_3621 = -1; // You, the first person of the catalog
    let sid2_3616 = sid(&d, 2);
    d.credits.as_mut().unwrap().education = vec![
        FederalEducation {
            student: sid2_3616,
            expenses: education_expenses(650_000),
            method: education_choice(EducationMethod::AmericanOpportunity, Answer::Yes),
        },
        FederalEducation {
            student: sid0_3621,
            expenses: education_expenses(300_000),
            method: education_choice(EducationMethod::LifetimeLearning, Answer::No),
        },
    ];
    let r = supported(&d);

    // Form 8863 Part III, AOTC student (dependent, $6,500 of expenses):
    //   Line 27 (adjusted qualified expenses, capped at $4,000): $4,000
    //   Line 28 (line 27 - $2,000): $2,000
    //   Line 29 (line 28 x 25%): $500
    //   Line 30 (100% of first $2,000 + line 29) -> Part I line 1: $2,500
    // Form 8863 Part III, LLC student (filer, $3,000 of expenses):
    //   Line 31 (adjusted qualified expenses) -> Part II line 10: $3,000
    //
    // Form 8863 Part I:
    //   Line 1: $2,500          Line 2 (MFJ): $180,000
    //   Line 3 (MAGI = AGI, no foreign exclusion): $118,000
    //   Line 4 (line 2 - line 3): $62,000     Line 5 (MFJ): $20,000
    //   Line 6 (line 4 >= line 5, so capped at 1.000): 1.000
    //   Line 7 (line 1 x line 6, no under-24 restriction on this filer): $2,500
    //   Line 8/refundable (40% of line 7) -> 1040 line 29: $1,000
    //   Line 9 (line 7 - line 8), the nonrefundable AOTC remainder: $1,500
    // Form 8863 Part II:
    //   Line 10: $3,000   Line 11 (smaller of line 10, $10,000): $3,000
    //   Line 12 (line 11 x 20%): $600
    //   Lines 13-17 reuse the Part I MAGI/ratio (same MFJ thresholds): 1.000
    //   Line 18 (line 12 x line 17) -> Credit Limit Worksheet: $600
    // Line 19 (Credit Limit Worksheet: line 9 + line 18, limited by tax
    // remaining after care/elderly, both $0 here) -> Schedule 3 line 3:
    //   $1,500 + $600 = $2,100
    assert_eq!(value(&r, "refundable_education_credit"), 1_000);
    assert_eq!(value(&r, "education_credit"), 2_100);

    // AGI (1040 line 11) = $118,000 wages, no adjustments.
    assert_eq!(value(&r, "agi"), 118_000);
    // Standard deduction (1040 line 12e), MFJ, neither spouse 65+ or blind,
    // and no one else can claim either filer as a dependent: $32,200 flat.
    assert_eq!(return_line(&r, "12e"), 32_200 * 100);
    // Taxable income (1040 line 15) = 118,000 - 32,200 = $85,800.
    assert_eq!(return_line(&r, "15"), 85_800 * 100);
    // Regular tax (1040 line 16): taxable income is under $100,000, so the
    // September 2026 draft Tax Table applies. $85,800 falls in the "at least
    // $85,800 but less than $85,850" row; its midpoint is $85,825. 2026 MFJ
    // brackets: 10% to $24,800, 12% to $100,800. Tax on $85,825 =
    // 10% x $24,800 + 12% x ($85,825 - $24,800) = $2,480 + $7,323 = $9,803.
    assert_eq!(return_line(&r, "16"), 9_803 * 100);
    // Child and other-dependent credits (1040 line 19): the dependent is 20,
    // too old for the child tax credit, so only the $500 ODC applies -- no
    // child credit.
    assert_eq!(return_line(&r, "19"), 500 * 100);
    // Other nonrefundable credits (1040 line 20): the $2,100 education credit
    // computed above is the only one in scope for this return.
    assert_eq!(return_line(&r, "20"), 2_100 * 100);
    // Total nonrefundable credits (1040 line 21) = 500 + 2,100 = $2,600.
    assert_eq!(return_line(&r, "21"), 2_600 * 100);
    // Income tax after credits (1040 line 22) = 9,803 - 2,600 = $7,203.
    assert_eq!(return_line(&r, "22"), 7_203 * 100);
    // No additional taxes (line 23): total tax before installments (1040
    // line 24a) is unchanged at $7,203.
    assert_eq!(return_line(&r, "24a"), 7_203 * 100);
    // W-2 withholding (1040 line 25a) = box 2 = $12,500.
    assert_eq!(return_line(&r, "25a"), 12_500 * 100);
    // Refundable education credit (1040 line 29) = $1,000, as computed above.
    assert_eq!(return_line(&r, "29"), 1_000 * 100);
    // Total payments and refundable credits (1040 line 33) = withholding
    // $12,500 + refundable credits $1,000 = $13,500.
    assert_eq!(return_line(&r, "33"), 13_500 * 100);
    // Overpayment (1040 line 34) = 13,500 - 7,203 = $6,297; no balance due.
    assert_eq!(return_line(&r, "34"), 6_297 * 100);
    assert_eq!(return_line(&r, "37"), 0);
    assert_eq!(r.tax, Some(7_203 * 100));
}

// Boundary: MAGI at the exact midpoint of the AOTC/LLC $80,000-$90,000
// single phase-out band halves both credits (i8863 lines 4-6/15-17).
#[test]
fn education_credits_phase_out_at_mid_range_magi() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 8_500_000, 0);
    // A second student identity is needed to hold an independent LLC entry
    // (one credit entry per student); its own age/support facts do not
    // matter to this phase-out computation.
    add_child(&mut d);
    let sid0_3708 = -1; // You, the first person of the catalog
    let sid1_3713 = sid(&d, 1);
    d.credits.as_mut().unwrap().education = vec![
        FederalEducation {
            student: sid0_3708,
            expenses: education_expenses(400_000),
            method: education_choice(EducationMethod::AmericanOpportunity, Answer::Yes),
        },
        FederalEducation {
            student: sid1_3713,
            expenses: education_expenses(1_200_000),
            method: education_choice(EducationMethod::LifetimeLearning, Answer::No),
        },
    ];
    let r = supported(&d);
    // MAGI = AGI = $85,000 (single wages only, no adjustments).
    // Line 4 = $90,000 - $85,000 = $5,000; line 6 ratio = 5,000/10,000 = 0.500.
    // AOTC: $4,000 of expenses -> Part III line 30 = 100%*$2,000 + 25%*$2,000
    // = $2,500; line 7 = $2,500 x 0.500 = $1,250; refundable (40%) = $500;
    // nonrefundable remainder = $750.
    // LLC: $12,000 of expenses capped at $10,000 (line 11); line 12 =
    // $10,000 x 20% = $2,000; line 18 = $2,000 x 0.500 = $1,000 -- exactly
    // half of the uncapped $2,000 LLC credit, using the same 0.500 ratio.
    // Line 19 = $750 + $1,000 = $1,750, well under available tax.
    assert_eq!(value(&r, "refundable_education_credit"), 500);
    assert_eq!(value(&r, "education_credit"), 1_750);
}

// Boundary: Form 8863 Part II line 11 caps combined LLC expenses at $10,000
// per return, regardless of how much more was actually paid.
#[test]
fn lifetime_learning_expenses_cap_at_10000() {
    let mut d = complete(FederalStatus::Single);
    let sid0_3742 = -1; // You, the first person of the catalog
    d.credits.as_mut().unwrap().education.push(FederalEducation {
        student: sid0_3742,
        expenses: education_expenses(1_400_000),
        method: education_choice(EducationMethod::LifetimeLearning, Answer::No),
    });
    let r = supported(&d);
    // MAGI = AGI = $60,000 (single default wage), below the $80,000
    // phase-out floor, so line 17's ratio is 1.000 (no reduction).
    // Part II line 10 = $14,000; line 11 caps it at $10,000; line 12 =
    // $10,000 x 20% = $2,000 -- not 20% of the full $14,000 ($2,800).
    assert_eq!(value(&r, "refundable_education_credit"), 0);
    assert_eq!(value(&r, "education_credit"), 2_000);
}

// Boundary: i8863's five-part under-24 test denies the 40% refundable AOTC
// portion when the filer is under 24, a full-time student providing less
// than half their own support, with a parent alive at year end, and not
// filing jointly (`no_refund` in federal_rules.rs) -- the nonrefundable part
// still applies.
#[test]
fn refundable_aotc_is_denied_under_the_kiddie_conditions() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 5_000_000, 0);
    let sid0_3769 = -1; // You, the first person of the catalog
    d.credits.as_mut().unwrap().education.push(FederalEducation {
        student: sid0_3769,
        expenses: education_expenses(400_000),
        method: education_choice(EducationMethod::AmericanOpportunity, Answer::Yes),
    });
    let baseline = supported(&d);
    // MAGI $50,000 is below the $80,000 phase-out floor: AOTC = 100%*$2,000
    // + 25%*$2,000 = $2,500; refundable 40% = $1,000; nonrefundable = $1,500.
    assert_eq!(value(&baseline, "refundable_education_credit"), 1_000);
    assert_eq!(value(&baseline, "education_credit"), 1_500);

    // `person()` already sets parent_alive = true. Make the taxpayer a
    // 20-year-old (born 2006) full-time student providing less than half
    // their own support: the five-part test is met (not under 18, but 18-23
    // and a full-time student, with support_share less than half, a living
    // parent, and a non-joint return).
    d.people.as_mut().unwrap().taxpayer.birth_date = 20060615;
    d.people.as_mut().unwrap().taxpayer.full_time_student = true;
    d.people.as_mut().unwrap().taxpayer.support_share = SupportShare::LessThanHalf;
    let kiddie = supported(&d);
    // The entire $2,500 AOTC now carries into the nonrefundable pool
    // (Form 8863 "skip line 8, enter the line 7 amount here"); none is
    // refundable, but the nonrefundable credit is fully available against
    // the filer's own tax ($3,823 on $33,900 of taxable income).
    assert_eq!(value(&kiddie, "refundable_education_credit"), 0);
    assert_eq!(value(&kiddie, "education_credit"), 2_500);
}

// The one definition of "expenses an education credit takes into account"
// (IRC 135(d)(2)(A), 529(c)(3)(B)(v)): $4,000 for each American opportunity
// credit entry, and the return's $10,000 for the lifetime learning credit
// shared in list order; nothing when no credit is allowed or the student has
// no valid SSN (Form 8863 line 21).
#[test]
fn education_expenses_used_follows_the_credit_caps_in_list_order() {
    let mut d = complete(FederalStatus::Single);
    for _ in 0..3 {
        add_child(&mut d);
    }
    let mut students = crate::federal_rules::people_students(&d);
    let ids: Vec<i64> = students.iter().map(|s| s.id).collect();
    let entry = |student: usize, tuition: i64, method: EducationMethod| FederalEducation {
        student: ids[student],
        expenses: education_expenses(tuition * 100),
        method: education_choice(method, Answer::Yes),
    };
    let credits = d.credits.as_mut().unwrap();
    credits.education = vec![
        entry(1, 8_000, EducationMethod::LifetimeLearning),
        entry(2, 6_000, EducationMethod::LifetimeLearning),
        entry(3, 11_000, EducationMethod::AmericanOpportunity),
    ];
    let used = |credits: &FederalCredits,
                students: &[crate::PersonChoice],
                allowed: bool|
     -> Vec<(i64, i64)> {
        credits
            .education_expenses_used(students, allowed)
            .iter()
            .map(|u| (u.qualified, u.used))
            .collect()
    };
    // 8,000 of the lifetime cap, then the remaining 2,000; the AOTC entry 4,000.
    assert_eq!(
        used(credits, &students, true),
        vec![(8_000, 8_000), (6_000, 2_000), (11_000, 4_000)]
    );
    assert_eq!(credits.education_expenses_used(&students, true)[0].person, Some(students[1].id));
    // No credit allowed (married filing separately, a claimed filer): none used.
    assert_eq!(used(credits, &students, false), vec![(8_000, 0), (6_000, 0), (11_000, 0)]);
    // A student without a valid SSN takes no credit and leaves the lifetime
    // cap to the next entry.
    students[1].valid_ssn = false;
    assert_eq!(used(credits, &students, true), vec![(8_000, 0), (6_000, 6_000), (11_000, 4_000)]);
    // Scholarships and 529 allocations reduce the credit's own expenses first.
    students[1].valid_ssn = true;
    credits.education[2].expenses.assistance = 800_000;
    assert_eq!(used(credits, &students, true)[2], (3_000, 3_000));
    // Form 8863 uses the same amounts: the AOTC entry's credit is figured on
    // its 4,000: 2,000 + 25% of 2,000, and the lifetime entries on 10,000.
    d.credits.as_mut().unwrap().education[2].expenses.assistance = 0;
    let r = supported(&d);
    // Lifetime: 20% of 10,000 = 2,000; AOTC: 2,000 + 500 = 2,500.
    assert_eq!(value(&r, "education_credit") + value(&r, "refundable_education_credit"), 4_500);
}
