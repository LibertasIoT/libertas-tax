// Line audit of the family credits: Schedule 8812, the earned income credit
// (Form 1040 line 27a, Schedule EIC, Form 8862), Form 2441 and Form 8863
// (2026-10-10, defects D1-D14; fix unit F8). Every expected amount is the
// audit's hand-worked fixture figure from the 2026 drafts (Schedule 8812,
// Schedule EIC, Forms 2441, 8863 and the Form 8863 instructions of Sep 11,
// 2026), the 2025 i1040 EIC steps and Pub. 596, Pub. 503, Form 8862 (Rev.
// 12-2025) and Rev. Rul. 2003-72, never the App's own output.
use super::*;

fn has(r: &FederalResult, id: &str) -> bool {
    r.lines.iter().any(|l| l.id == id)
}
// The printed form and line of a calculation row.
fn form_row(r: &FederalResult, id: &str) -> (crate::TaxForm, alloc::string::String) {
    printed_line(r, id)
}
fn single_with_child(wages: i64, born: u32) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, wages * 100, 0);
    add_child(&mut d);
    d.dependents[0].birth_date = born;
    d.dependents[0].situation = DependentSituation::ChildUnder19;
    d
}
fn care_yes(d: &mut FederalDraft) {
    d.credits.as_mut().unwrap().care = FederalCare::default();
    document_care(d);
}
fn aotc_entry(student: i64, tuition: i64) -> FederalEducation {
    FederalEducation {
        student,
        expenses: education_expenses(tuition * 100),
        method: education_choice(EducationMethod::AmericanOpportunity, Answer::Yes),
    }
}

// D1 (a). Rev. Rul. 2003-72: "a child born on January 1, 1987, attains the age
// of 17 on January 1, 2004". A child born 2010-01-01 is 16 on December 31,
// 2026 and earns the $2,200 child tax credit; one born 2009-12-31 attains 17
// on December 31, 2026 and is a $500 other dependent (Schedule 8812 line 4).
#[test]
fn d1_a_child_attains_17_on_the_birthday_for_the_child_tax_credit() {
    for (born, credit, count) in
        [(20100101u32, 2200, 1), (20100102, 2200, 1), (20091231, 500, 0), (20090101, 500, 0)]
    {
        let d = single_with_child(60_000, born);
        let r = supported(&d);
        assert_eq!(value(&r, "child_credit"), credit, "born {born}");
        assert_eq!(value(&r, "child_credit_children"), count, "born {born}");
    }
    // Form 1040 line 18 is 5,023 on the audit's single $60,000 wage fixture.
    let r = supported(&single_with_child(60_000, 20100101));
    assert_eq!(value(&r, "income_tax"), 5023);
}

// D1 (b). The 2026 Schedule EIC line 3 ("If born after 2007 and the child is
// younger than you ... skip lines 4a and 4b") and the 2025 i1040 Step 3: a
// child born 2008-01-01 is under 19 at the end of 2026 and a qualifying child;
// born 2007-12-31 the child is 19 and is neither a qualifying child nor, with
// no student status, a dependent.
#[test]
fn d1_a_child_born_on_january_1_2008_is_under_19_for_the_eic_and_dependency() {
    for (born, eic, odc) in [(20080101u32, 4427, 393), (20080102, 4427, 393), (20071231, 0, 0)] {
        let d = single_with_child(20_000, born);
        let r = supported(&d);
        assert_eq!(value(&r, "earned_income_credit"), eic, "born {born}");
        assert_eq!(value(&r, "child_credit"), odc, "born {born}");
        assert_eq!(value(&r, "child_credit_children"), 0, "born {born}");
    }
    // Head of household is not refused for the January 1 child.
    let mut d = single_with_child(20_000, 20080101);
    d.setup.as_mut().unwrap().status = filing_choice(FederalStatus::Head);
    let r = estimate(&d);
    assert!(
        r.issues.iter().all(|i| !i.display.key.contains("QUALIFIES_YOU_FOR_IT")),
        "{:?}",
        r.issues
    );
}

// D1 (c). Rev. Rul. 2003-72 and the 2026 i2441 ("If the child turned 13
// during the year, the child is a qualifying person for the part of the year
// they were under age 13"): a child born 2013-01-02 is still 12 on January 1,
// 2026, so two qualifying persons raise the ceiling to $6,000 (line 3), the
// line 8 percentage is 35, line 11 is 2,100, Schedule 8812 line 14 is 5,023 -
// 2,100 = 2,923 and the additional child tax credit is min(4,400 - 2,923,
// 2 x 1,700, 15% x 57,500) = 1,477. Born 2013-01-01 the child is 13 on January
// 1 and never under 13 in 2026: the ceiling is $3,000 and the credit 1,050.
#[test]
fn d1_a_child_born_on_january_2_2013_is_a_qualifying_person_for_part_of_the_year() {
    let fixture = |born_b: u32| {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, 6_000_000, 650_000);
        add_child(&mut d);
        add_child(&mut d);
        d.dependents[0].set_care_expenses(600_000);
        d.dependents[1].birth_date = born_b;
        care_yes(&mut d);
        d
    };
    let r = supported(&fixture(20130102));
    assert_eq!(value(&r, "care_expenses_allowed"), 6000);
    assert_eq!(value(&r, "care_percent"), 35);
    assert_eq!(value(&r, "care_credit"), 2100);
    assert_eq!(value(&r, "child_credit"), 2923);
    assert_eq!(value(&r, "additional_child_credit"), 1477);
    assert_eq!(return_line(&r, "33") / 100, 7977);
    // The January 1 boundary of the care credit.
    let r = supported(&fixture(20130101));
    assert_eq!(value(&r, "care_expenses_allowed"), 3000);
    assert_eq!(value(&r, "care_credit"), 1050);
    // The people page refuses care costs for someone already 13 on January 1
    // (the client rule and its server twin share the cutoff).
    let r = supported(&fixture(20130103));
    assert_eq!(value(&r, "care_credit"), 2100);
}

// D2. 2026 i1040s8 "Death or birth of child": a qualifying child who was born
// and died in 2026 and has no SSN still counts for the child tax credit and,
// per Schedule EIC line 2 ("Died"), the earned income credit.
#[test]
fn d2_a_child_born_and_died_in_the_year_without_an_ssn_still_qualifies() {
    let born_and_died = |wages: i64, fact: bool| {
        let mut d = single_with_child(wages, 20260301);
        d.dependents[0].residency = DependentResidency::WholeYearException;
        d.dependents[0].valid_ssn = false;
        d.dependents[0].valid_tin = false;
        set_fact(&mut d.dependents[0], PersonFacts::BORN_AND_DIED, fact);
        d
    };
    let r = supported(&born_and_died(60_000, true));
    assert_eq!(value(&r, "income_tax"), 5023);
    assert_eq!(value(&r, "child_credit"), 2200);
    let r = supported(&born_and_died(20_000, true));
    assert_eq!(value(&r, "earned_income_credit"), 4427);
    // Schedule EIC line 2 reads "Died".
    assert!(has(&r, "schedule_eic_child_1_died"));
    assert!(!has(&r, "schedule_eic_child_1_ssn"));
    // Without the fact a child with no number earns neither credit.
    let r = supported(&born_and_died(60_000, false));
    assert_eq!(value(&r, "child_credit"), 0);
    let r = supported(&born_and_died(20_000, false));
    assert_eq!(value(&r, "earned_income_credit"), 0);
    // The fact is for a child born in 2026: a child born earlier ignores it.
    let mut d = born_and_died(60_000, true);
    d.dependents[0].birth_date = 20250301;
    d.dependents[0].residency = DependentResidency::Days { days: 365 };
    assert_eq!(value(&supported(&d), "child_credit"), 0);
}

// D3. IRC 24(c)(2), 24(h)(4)(B): only a person who is not a U.S. citizen,
// national or resident is excluded; where a citizen child lived is no test. A
// U.S.-citizen child who lived abroad with the filer keeps the credit; the
// tick is for a person who is not a citizen, national or resident alien.
#[test]
fn d3_a_us_citizen_child_who_lived_abroad_keeps_the_child_tax_credit() {
    let mut d = single_with_child(60_000, 20180615);
    d.dependents[0].us_home_over_half_year = false;
    let r = supported(&d);
    assert_eq!(value(&r, "child_credit"), 2200);
    assert_eq!(value(&r, "earned_income_credit"), 0);
    // The control: the same child ticked as not a citizen, national or resident.
    set_fact(&mut d.dependents[0], PersonFacts::ABROAD, true);
    assert_eq!(value(&supported(&d), "child_credit"), 0);
}

// Row 611: the 2026 Schedule 8812 adopted-child exception. A child who is not a
// U.S. citizen, national or resident alien meets the citizenship condition when
// the filer adopted the child, the child lived in the home all year and the
// filer is a citizen or national (one tick). An ATIN is not an SSN: it gives
// the $500 credit for other dependents but not the $2,200 child tax credit.
#[test]
fn adopted_child_exception_restores_dependency_and_the_credit_its_number_allows() {
    let mut d = single_with_child(60_000, 20180615);
    set_fact(&mut d.dependents[0], PersonFacts::ABROAD, true);
    assert_eq!(value(&supported(&d), "child_credit"), 0);
    // Ticked and holding an SSN: the child tax credit.
    set_fact(&mut d.dependents[0], PersonFacts::ADOPTED_ALL_YEAR, true);
    let r = supported(&d);
    assert_eq!(value(&r, "child_credit"), 2200);
    assert_eq!(value(&r, "child_credit_children"), 1);
    // Ticked with an ATIN (a valid TIN but no SSN): the credit for other dependents.
    d.dependents[0].valid_ssn = false;
    d.dependents[0].valid_tin = true;
    let r = supported(&d);
    assert_eq!(value(&r, "child_credit"), 500);
    assert_eq!(value(&r, "child_credit_children"), 0);
    // Without the tick the ATIN child is no dependent at all.
    set_fact(&mut d.dependents[0], PersonFacts::ADOPTED_ALL_YEAR, false);
    assert_eq!(value(&supported(&d), "child_credit"), 0);
}

// D4. 2025 i1040 "Married child": "A child who was married at the end of 2025
// is a qualifying child only if (a) you can claim the child as your dependent,
// or (b) you could have claimed the child as your dependent except for the
// special rule for Children of divorced or separated parents."
#[test]
fn d4_a_married_child_who_is_not_the_filers_dependent_is_not_an_eic_child() {
    let fixture = |own_support: bool, married: bool| {
        let mut d = single_with_child(20_000, 20060615);
        d.dependents[0].situation = DependentSituation::Student;
        set_fact(&mut d.dependents[0], PersonFacts::OWN_SUPPORT, own_support);
        set_fact(&mut d.dependents[0], PersonFacts::MARRIED_AT_YEAR_END, married);
        d
    };
    // Self-only EIC on 20,000 is 0 (above the 19,540 completed phase-out).
    assert_eq!(value(&supported(&fixture(true, true)), "earned_income_credit"), 0);
    // Married without own support (the child is the filer's dependent) and
    // unmarried with own support (no dependency needed): both qualify.
    assert_eq!(value(&supported(&fixture(false, true)), "earned_income_credit"), 4427);
    assert_eq!(value(&supported(&fixture(true, false)), "earned_income_credit"), 4427);
    // (b) A married child the filer released to the other parent (Form 8332) is
    // one the filer could have claimed but for section 152(e): still counted.
    let mut d = fixture(false, true);
    set_fact(&mut d.dependents[0], PersonFacts::YOU_RELEASED, true);
    assert_eq!(value(&supported(&d), "earned_income_credit"), 4427);
}

// D5. Pub. 596 (2025) Rule 3, special rule for separated spouses: "If the child
// who meets the conditions to be your qualifying child for purposes of
// claiming the EIC doesn't have a valid SSN, you may still qualify to claim a
// self-only EIC." The self-only credit on 15,000 in the other-status column is
// 345; with an SSN the child earns the one-child maximum.
#[test]
fn d5_a_separated_spouse_whose_child_has_only_an_itin_claims_the_self_only_eic() {
    let fixture = |ssn: bool| {
        let mut d = complete(FederalStatus::Separate);
        d.setup.as_mut().unwrap().marital_state =
            MaritalState::Married { living: SpouseLiving::LastSixMonths };
        d.income[0] = wage(1, Owner::Taxpayer, 1_500_000, 0);
        add_child(&mut d);
        d.dependents[0].valid_ssn = ssn;
        d.dependents[0].valid_tin = true;
        d.dependents[0].situation = DependentSituation::ChildUnder19;
        d
    };
    let r = supported(&fixture(false));
    assert_eq!(value(&r, "earned_income_credit"), 345);
    // Schedule EIC lists the child for the self-only credit.
    assert!(has(&r, "schedule_eic_child_1_name"));
    assert_eq!(value(&supported(&fixture(true)), "earned_income_credit"), 4427);
}

// D6. 2025 i1040 EIC Step 5 and the 2026 Schedule 8812 Earned Income Worksheet
// start from Form 1040 line 1z; Form 8839 line 31 puts the negative special-
// needs amount on line 1f. MFJ, W-2 box 1 $4,000 and $2,000 (taxpayer) and
// $2,000 (spouse), code T 6,000 / 2,000 / 4,000: line 1f is -5,670 and line 1z
// 2,330. EIC on 2,330 (MFJ, one child) is 791; line 19 is 2,330 - 2,500 -> 0.
#[test]
fn d6_a_negative_special_needs_line_1f_reduces_earned_income() {
    let mut d = joint_special_needs_below_code_t_cap();
    set_joint_adoption_w2(&mut d, 0, 400_000, 600_000);
    set_joint_adoption_w2(&mut d, 1, 200_000, 200_000);
    set_joint_adoption_w2(&mut d, 2, 200_000, 400_000);
    for e in d.income.iter_mut() {
        if let FederalIncome::Wage { data } = &mut e.income {
            data.withholding = 0;
        }
    }
    add_child(&mut d);
    let r = supported(&d);
    assert_eq!(return_line(&r, "1f") / 100, -5670);
    assert_eq!(return_line(&r, "1z") / 100, 2330);
    assert_eq!(value(&r, "earned_income_credit"), 791);
    assert_eq!(value(&r, "additional_child_credit"), 0);
    assert_eq!(value(&r, "additional_child_credit_earned_income"), 2330);
}

// D7. 2025 i1040 Step 3 question 2 ("Are you filing a joint return for 2025?
// Yes. Skip questions 3 through 6 and Step 4") and Step 4 question 4 ("Yes.
// Skip questions 5 and 6"): a joint return never tests whether a spouse can be
// another's qualifying child or dependent, so a Not sure answer there denies
// nothing.
#[test]
fn d7_joint_returns_skip_the_eic_tests_the_instructions_skip() {
    // (a) one child; the spouse's "another taxpayer's qualifying child" is Yes.
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 2_000_000, 0);
    add_child(&mut d);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().another_eitc_child = true;
    let r = supported(&d);
    assert_eq!(value(&r, "earned_income_credit"), 4427);
    // (b) no child; the spouse's dependency is Yes.
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 1_500_000, 0);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().dependency =
        FilerDependency::CanBeClaimed { claimed: true };
    assert_eq!(value(&supported(&d), "earned_income_credit"), 664);
    // A filer on any other return still follows the actual eligibility answers.
    let mut d = single_with_child(20_000, 20180615);
    d.people.as_mut().unwrap().taxpayer.another_eitc_child = true;
    let r = supported(&d);
    assert_eq!(value(&r, "earned_income_credit"), 0);
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 900_000, 0);
    d.people.as_mut().unwrap().taxpayer.dependency =
        FilerDependency::CanBeClaimed { claimed: true };
    assert_eq!(value(&supported(&d), "earned_income_credit"), 0);
}

// D8. Form 8862 Rev. 12-2025 line 10: "the rule for reaching age 65 is different;
// your spouse, or the deceased taxpayer, reaches age 65 on their 65th
// birthday". A filer born 1962-01-01 turns 65 on January 1, 2027: line 10a is 64
// and the childless credit (664) is allowed; age 25 uses the day before.
#[test]
fn d8_form_8862_line_10_prints_64_for_a_filer_who_turns_65_on_january_1_2027() {
    let fixture = |born: u32| {
        let mut d = complete(FederalStatus::Single);
        d.people.as_mut().unwrap().taxpayer.birth_date = born;
        d.income[0] = wage(1, Owner::Taxpayer, 900_000, 0);
        d.credits.as_mut().unwrap().after_disallowance = Some(CreditsAfterDisallowance {
            earned_income: EicDisallowance::Disallowed {
                facts: DisallowanceFacts {
                    year: 2022,
                    finding: DisallowanceFinding::Ineligible,
                    reinstated: false,
                },
                cause: EicDisallowanceCause::Other,
                your_us_home_days: Some(365),
                spouse_us_home_days: None,
            },
            child_credits: CreditDisallowance::Never,
            education_credit: CreditDisallowance::Never,
        });
        d
    };
    let r = supported(&fixture(19620101));
    assert_eq!(value(&r, "earned_income_credit"), 664);
    assert_eq!(value(&r, "form_8862_line_10a"), 64);
    // Born 1961-12-31 reaches 65 on December 31, 2026: no credit, no form.
    let r = supported(&fixture(19611231));
    assert_eq!(value(&r, "earned_income_credit"), 0);
    assert!(!has(&r, "form_8862_line_10a"));
    // Born 2002-01-01 is 25 the day before the birthday: allowed, printed 25.
    let r = supported(&fixture(20020101));
    assert_eq!(value(&r, "form_8862_line_10a"), 25);
    assert_eq!(value(&r, "earned_income_credit"), 664);
}

// D9. IRC 21(e)(10): "No credit shall be allowed ... with respect to any
// qualifying individual unless the TIN of such individual is included on the
// return"; the 2026 i2441 line 2 column (b) excepts only a child born and died
// in 2026. A child with no number issued by the due date earns no care credit.
#[test]
fn d9_a_care_qualifying_person_without_a_taxpayer_identification_number_earns_no_credit() {
    let fixture = |ssn: bool, tin: bool| {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, 6_000_000, 0);
        add_child(&mut d);
        d.dependents[0].valid_ssn = ssn;
        d.dependents[0].valid_tin = tin;
        d.dependents[0].set_care_expenses(300_000);
        care_yes(&mut d);
        d
    };
    let r = supported(&fixture(false, false));
    assert_eq!(value(&r, "care_credit"), 0);
    assert!(r.review_notes.iter().any(|n| n.display.key == "WARN_CARE_COSTS_NEED_TIN"));
    // An ITIN or ATIN suffices, as does an SSN.
    assert_eq!(value(&supported(&fixture(false, true)), "care_credit"), 1050);
    assert_eq!(value(&supported(&fixture(true, true)), "care_credit"), 1050);
    // The born-and-died child needs no number.
    let mut d = fixture(false, false);
    d.dependents[0].birth_date = 20260301;
    d.dependents[0].residency = DependentResidency::WholeYearException;
    set_fact(&mut d.dependents[0], PersonFacts::BORN_AND_DIED, true);
    assert_eq!(value(&supported(&d), "care_credit"), 1050);
}

fn spouse_student_return() -> FederalDraft {
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 10_000_000, 0);
    d
}

// D10. 2026 i8863 "Identification requirement": "If you are filing a joint
// return, only one spouse is required to have a valid SSN"; Tip: "This is true
// even when one spouse on a joint return is the student for whom qualified
// education expenses were paid and does not have a valid SSN." MFJ, MAGI
// 100,000, tuition 4,000: line 30 = 2,000 + 25% x 2,000 = 2,500, line 8 =
// 1,000 and line 9 = 1,500.
#[test]
fn d10_a_spouse_student_without_an_ssn_keeps_the_credit_when_the_other_spouse_has_one() {
    let mut d = spouse_student_return();
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().valid_ssn = false;
    d.credits.as_mut().unwrap().education = vec![aotc_entry(-2, 4000)];
    let r = supported(&d);
    assert_eq!(value(&r, "refundable_education_credit"), 1000);
    assert_eq!(value(&r, "education_credit"), 1500);
    assert_eq!(value(&r, "income_tax"), 7643);
}

// D11. 2026 i8863 "Who cannot claim a credit" item 5: you (or your spouse if
// filing jointly) and the student (if the student is a dependent on your tax
// return) must have valid SSNs. Allowed when (the taxpayer, or on a joint
// return either spouse, has a valid SSN) and (the student is a filer or the
// dependent student has a valid SSN).
#[test]
fn d11_the_filers_own_ssn_requirement_applies_to_the_education_credit() {
    let with_student = |mut d: FederalDraft, student_ssn: bool| {
        add_child(&mut d);
        d.dependents[0].birth_date = 20070615;
        d.dependents[0].situation = DependentSituation::Student;
        d.dependents[0].valid_ssn = student_ssn;
        let id = d.dependents[0].id;
        d.credits.as_mut().unwrap().education = vec![aotc_entry(id, 4000)];
        d
    };
    // (a) Single with an ITIN, dependent student with an SSN: nothing.
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.valid_ssn = false;
    let r = supported(&with_student(d, true));
    assert_eq!(value(&r, "education_credit"), 0);
    assert_eq!(value(&r, "refundable_education_credit"), 0);
    // (b) MFJ, both ITIN, W-2 100,000, the same student.
    let mut d = spouse_student_return();
    d.people.as_mut().unwrap().taxpayer.valid_ssn = false;
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().valid_ssn = false;
    let r = supported(&with_student(d, true));
    assert_eq!(value(&r, "education_credit"), 0);
    assert_eq!(value(&r, "refundable_education_credit"), 0);
    // Controls: one spouse's SSN suffices; a single filer with an SSN gets the
    // credit; a dependent student without an SSN still gets none.
    let mut d = spouse_student_return();
    d.people.as_mut().unwrap().taxpayer.valid_ssn = false;
    let r = supported(&with_student(d, true));
    assert_eq!(value(&r, "refundable_education_credit"), 1000);
    assert_eq!(value(&r, "education_credit"), 1500);
    let r = supported(&with_student(complete(FederalStatus::Single), true));
    assert_eq!(value(&r, "refundable_education_credit"), 1000);
    let r = supported(&with_student(complete(FederalStatus::Single), false));
    assert_eq!(value(&r, "refundable_education_credit"), 0);
}

// D12. Schedule 8812: every nonzero printed line is a row. Single, W-2 $60,000
// (line 18 tax 5,023), one child under 17 and one 17-year-old: line 4 = 1,
// 5 = 2,200, 6 = 1, 7 = 500, 8 = 2,700, 9 = 200,000, 12 = 14 = 2,700.
#[test]
fn d12_schedule_8812_shows_every_nonzero_line() {
    let mut d = single_with_child(60_000, 20180615);
    add_child(&mut d);
    d.dependents[1].birth_date = 20080615;
    d.dependents[1].situation = DependentSituation::ChildUnder19;
    let r = supported(&d);
    for (line, id, amount) in [
        ("1", "child_credit_agi", 60_000),
        ("3", "child_credit_magi", 60_000),
        ("4", "child_credit_children", 1),
        ("5", "child_credit_children_amount", 2200),
        ("6", "child_credit_other_dependents", 1),
        ("7", "child_credit_other_dependents_amount", 500),
        ("8", "child_credit_before_phaseout", 2700),
        ("9", "child_credit_phaseout_threshold", 200_000),
        ("12", "child_credit_after_phaseout", 2700),
        ("13", "child_credit_limit", 5023),
        ("14", "child_credit", 2700),
        ("16b", "additional_child_credit_children_limit", 1700),
        ("18a", "additional_child_credit_earned_income", 60_000),
        ("19", "additional_child_credit_earned_over_floor", 57_500),
        ("20", "additional_child_credit_earned_share", 8625),
    ] {
        assert_eq!(value(&r, id), amount, "line {line}");
        assert_eq!(form_row(&r, id), (crate::TaxForm::Schedule8812, line.into()), "{id}");
    }
    // A zero line is not shown on the return; a nonzero one is.
    let rows = crate::draft_return(&r);
    let shown = |line: &str| {
        rows.iter().any(|row| {
            row.form_line == Some(crate::FormLine::new(crate::TaxForm::Schedule8812, line))
        })
    };
    assert!(shown("4") && shown("8") && shown("14") && shown("20"));
    assert!(!shown("10") && !shown("11") && !shown("16a"));
    // The phase-out lines, MFJ with MAGI 410,500 and one child and one other
    // dependent: line 10 = 11,000, line 11 = 550, line 12 = 2,150.
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 41_050_000, 0);
    add_child(&mut d);
    add_child(&mut d);
    d.dependents[1].birth_date = 20080615;
    d.dependents[1].situation = DependentSituation::ChildUnder19;
    let r = supported(&d);
    assert_eq!(value(&r, "child_credit_phaseout_threshold"), 400_000);
    assert_eq!(value(&r, "child_credit_excess_income"), 11_000);
    assert_eq!(value(&r, "child_credit_phaseout_reduction"), 550);
    assert_eq!(value(&r, "child_credit_after_phaseout"), 2150);
}

// D12, Part II-B: head of household, wages 4,000 and interest 15,000 (the EIC
// is barred by the $12,200 investment-income limit), three children: line 16b
// = 5,100, line 20 = 225, line 21 = 306, line 25 = 306, line 26 = 306, line 27
// = 306.
#[test]
fn d12_schedule_8812_part_ii_b_lines_appear_when_the_payroll_alternative_applies() {
    let mut d = complete(FederalStatus::Head);
    d.income[0] = wage(1, Owner::Taxpayer, 400_000, 0);
    add(&mut d, income(FederalIncome::Interest { data: native_interest(1_500_000, 0) }));
    for _ in 0..3 {
        add_child(&mut d);
    }
    let r = supported(&d);
    for (id, amount) in [
        ("additional_child_credit_children_limit", 5100),
        ("additional_child_credit_earned_share", 225),
        ("additional_child_credit_withheld_taxes", 306),
        ("additional_child_credit_payroll_total", 306),
        ("additional_child_credit_payroll_net", 306),
        ("additional_child_credit_larger", 306),
        ("additional_child_credit", 306),
    ] {
        assert_eq!(value(&r, id), amount, "{id}");
    }
    assert_eq!(
        form_row(&r, "additional_child_credit_larger"),
        (crate::TaxForm::Schedule8812, "26".into())
    );
}

// D13. Schedule EIC (2026 draft) lines 1 to 6 per child are shown when the
// credit is claimed: the child, the SSN, the year of birth, the months and (for
// a student) line 4a. Child born 2008-01-01, a full-time student... the line 3
// skip applies to a child born after 2007 who is younger than the filer.
#[test]
fn d13_schedule_eic_is_shown() {
    let mut d = single_with_child(20_000, 20080101);
    add_child(&mut d);
    d.dependents[1].birth_date = 20060615;
    d.dependents[1].situation = DependentSituation::Student;
    let r = supported(&d);
    assert_eq!(value(&r, "earned_income_credit"), 7316);
    for (id, line, amount) in [
        ("schedule_eic_child_1_name", "1", 0),
        ("schedule_eic_child_1_ssn", "2", 0),
        ("schedule_eic_child_1_birth_year", "3", 2008),
        ("schedule_eic_child_1_months", "6", 12),
        ("schedule_eic_child_2_name", "1", 0),
        ("schedule_eic_child_2_birth_year", "3", 2006),
        ("schedule_eic_child_2_student", "4a", 1),
        ("schedule_eic_child_2_months", "6", 12),
    ] {
        assert_eq!(value(&r, id), amount, "{id}");
        assert_eq!(form_row(&r, id), (crate::TaxForm::ScheduleEic, line.into()));
    }
    // The born-after-2007 child skips lines 4a and 4b.
    assert!(!has(&r, "schedule_eic_child_1_student"));
    assert!(!has(&r, "schedule_eic_child_1_disabled"));
    // No credit, no schedule.
    let r = supported(&single_with_child(80_000, 20080101));
    assert!(!has(&r, "schedule_eic_child_1_name"));
}

// D14. Form 2441 lines 3 to 10 are rows; line 5 repeats line 4 on a return that
// is not joint. D1 fixture (c): 3 6,000, 4 60,000, 5 60,000, 6 6,000, 7 60,000,
// 8 .35, 9a 2,100, 9c 2,100, 10 5,023.
#[test]
fn d14_form_2441_lines_3_to_10_are_shown() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 6_000_000, 0);
    add_child(&mut d);
    add_child(&mut d);
    d.dependents[0].set_care_expenses(600_000);
    d.dependents[1].birth_date = 20130102;
    care_yes(&mut d);
    let r = supported(&d);
    for (line, id, amount) in [
        ("3", "care_expenses_allowed", 6000),
        ("4", "care_earned_taxpayer", 60_000),
        ("5", "care_earned_spouse", 60_000),
        ("6", "care_expenses_smallest", 6000),
        ("7", "care_agi", 60_000),
        ("8", "care_percent", 35),
        ("9a", "care_credit_before_prior", 2100),
        ("9c", "care_credit_total", 2100),
        ("10", "care_credit_limit", 5023),
        ("11", "care_credit", 2100),
    ] {
        assert_eq!(value(&r, id), amount, "line {line}");
        assert_eq!(form_row(&r, id), (crate::TaxForm::Form2441, line.into()));
    }
    // A joint return keeps the spouse's own earned income on line 5.
    let mut d = complete(FederalStatus::Joint);
    add(&mut d, wage(0, Owner::Spouse, 200_000, 0));
    add_child(&mut d);
    d.dependents[0].set_care_expenses(400_000);
    care_yes(&mut d);
    let r = supported(&d);
    assert_eq!(value(&r, "care_earned_taxpayer"), 100_000);
    assert_eq!(value(&r, "care_earned_spouse"), 2000);
    assert_eq!(value(&r, "care_expenses_smallest"), 2000);
    // No care claim, no Part II rows.
    let r = supported(&complete(FederalStatus::Single));
    assert!(!has(&r, "care_expenses_allowed"));
}

// D1 keeps the day-before rule where the instructions write it: the 2025
// i1040 for Forms 8814 and 8615 ("A child born on January 1, 2008, is
// considered to be age 18 at the end of 2025"), so a child born 2008-01-01 is
// 19 at the end of 2026 for the election and the kiddie-tax age test, while the
// same child is 18 for the child tax credit, EIC and dependency tests above.
#[test]
fn d1_forms_8814_and_8615_keep_the_day_before_age_rule() {
    for (born, eligible) in [(20080101u32, false), (20080102, true)] {
        let mut dependent = child(5);
        dependent.birth_date = born;
        dependent.situation = DependentSituation::ChildUnder19;
        assert_eq!(
            crate::child_election::eligible_child(&dependent),
            eligible,
            "Form 8814, born {born}"
        );
        let mut filer = person();
        filer.birth_date = born;
        filer.full_time_student = false;
        filer.support_share = SupportShare::LessThanHalf;
        // Form 8615 (age 18 counts only without half of own support).
        assert_eq!(crate::federal_rules::kiddie_age(&filer), eligible, "Form 8615, born {born}");
    }
}
