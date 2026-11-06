// Record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::*;
use crate::tests::cross::interview::{
    at_the_credits_hub, continued, education_value, open_credits_topic, save_education,
    save_more_credits, student,
};

// Form 8958 (Pub. 555): fixtures F4 of the whole-return map. Emery files
// separately in Arizona and divides every item by half; the spouse Finley's
// documents are entered under "Your spouse".
pub(in crate::tests) fn community_pair(
    state: CommunityState,
    rule: CommunityRule,
    filer_is_emery: bool,
) -> FederalDraft {
    let mut d = complete(FederalStatus::Separate);
    d.setup.as_mut().unwrap().community_property = CommunityProperty::Yes { state, rule };
    d.deductions.as_mut().unwrap().itemize_override = None;
    let (own, other) = if filer_is_emery {
        (Owner::Taxpayer, Owner::Spouse)
    } else {
        (Owner::Spouse, Owner::Taxpayer)
    };
    // Emery's W-2 ($50,000, $5,000 withheld) and Finley's ($70,000, $8,000),
    // the joint bank's interest ($1,200, Emery's document) and Finley's
    // inherited account ($800, Finley's own).
    let mut entries = vec![
        wage(1, own, 5_000_000, 500_000),
        wage(2, other, 7_000_000, 800_000),
        income(savings_income(1200, 0)),
        income(savings_income(800, 0)),
    ];
    entries[2].owner = crate::DocumentOwner::from(own);
    entries[2].label = "Joint bank".into();
    entries[3].owner = crate::DocumentOwner::from(other);
    entries[3].label = "Inherited account".into();
    for (id, entry) in entries.iter_mut().enumerate() {
        entry.id = id as i64 + 1;
        entry.community = CommunityShare::Community;
    }
    entries[3].community = CommunityShare::Separate;
    if !filer_is_emery {
        // Finley is the filer: the W-2s and the inherited account swap.
        entries.swap(0, 1);
        entries[0].id = 1;
        entries[1].id = 2;
        entries[0].owner = crate::DocumentOwner::Yours;
        entries[1].owner = crate::DocumentOwner::Spouses;
        entries[2].owner = crate::DocumentOwner::Spouses;
        entries[3].owner = crate::DocumentOwner::Yours;
    }
    d.income = entries;
    d.next_id = 5;
    d
}
#[test]
fn child_and_other_dependent_credit() {
    let mut d = complete(FederalStatus::Joint);
    add_child(&mut d);
    add_child(&mut d);
    assert_eq!(supported(&d).tax, Some(324_300));
    d.dependents[1].birth_date = 20040615;
    d.dependents[1].situation = DependentSituation::Student;
    assert_eq!(value(&supported(&d), "child_credit"), 2700);
    // No longer a student, and a relative whose gross income is at the limit.
    d.dependents[1].situation =
        DependentSituation::Supported { your_support_over_half: true, gross_income: 530_000 };
    assert_eq!(value(&supported(&d), "child_credit"), 2200);
}
// IRC 152(c)(1)(D): a qualifying child must not have provided over half of
// their own support. The people page asks it (PersonFacts::OWN_SUPPORT) and
// family() reads it; before, no page set the stored field, so a working
// 17-year-old who paid for most of their own living was always a child.
//   Single, wages $50,000, one child born 2009-01-15 (17 at year end, so the
//   $500 credit for other dependents, not the $2,200 child tax credit) who
//   lived with the filer all year. Paying more than half of their own support
//   makes them neither a qualifying child nor a qualifying relative (the
//   filer did not pay over half): no credit for other dependents. The earned
//   income credit is unchanged because IRC 32(c)(3)(A) reads section 152(c)
//   "without regard to paragraph (1)(D)". Born 2018-06-15 (8): the $2,200
//   child tax credit is lost the same way.
#[test]
fn a_child_who_pays_over_half_of_their_own_support_is_not_a_qualifying_child() {
    let build = |born: u32, own_support: bool| {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, 5_000_000, 0);
        add_child(&mut d);
        d.dependents[0].birth_date = born;
        d.dependents[0].situation = DependentSituation::ChildUnder19;
        set_fact(&mut d.dependents[0], PersonFacts::OWN_SUPPORT, own_support);
        d
    };
    let r = supported(&build(20090115, false));
    assert_eq!(value(&r, "child_credit"), 500);
    let eic = value(&supported(&build(20090115, false)), "earned_income_credit");
    let r = supported(&build(20090115, true));
    assert_eq!(value(&r, "child_credit"), 0);
    assert_eq!(value(&r, "earned_income_credit"), eic);
    let r = supported(&build(20180615, false));
    assert_eq!(value(&r, "child_credit"), 2200);
    let r = supported(&build(20180615, true));
    assert_eq!(value(&r, "child_credit"), 0);
}

// M2-F boundary case: a dependent who turns 17 during 2026 is Schedule 8812's
// $500 credit for other dependents (ODC), not the $2,200 child tax credit
// (CTC). The year-end age (federal_model.rs age_at_year_end) of a birth date of
// 2009-04-15 (turning 17 in April 2026) is 17, which fails family()'s
// `year_end_age < 17` CTC test but still passes the dependent's own under-19 test
// (age 17 < 19), so it falls to the `else if dependent && valid_tin` ODC
// branch instead. MAGI $100,000 is far under the $400,000 joint phaseout
// threshold, so Schedule 8812 line 12 potential = $500 unreduced; joint
// wages of $100,000 give income tax far above $500, so the full $500 is
// used as a nonrefundable credit (line 14) and there is no ACTC (line 27):
// ODC does not count toward the children used in the ACTC's $1,700-per-child
// cap.
#[test]
fn child_turning_17_during_2026_gets_odc_not_ctc() {
    let mut d = complete(FederalStatus::Joint);
    add_child(&mut d);
    d.dependents[0].birth_date = 20090415;
    let r = supported(&d);
    assert_eq!(value(&r, "child_credit"), 500);
    assert_eq!(value(&r, "additional_child_credit"), 0);
}
#[test]
fn head_of_household_eitc_and_additional_child_credit() {
    let mut d = complete(FederalStatus::Head);
    d.income[0] = wage(1, Owner::Taxpayer, 2_000_000, 0);
    add_child(&mut d);
    let r = supported(&d);
    assert_eq!(r.tax, Some(0));
    assert_eq!(value(&r, "earned_income_credit"), 4427);
    assert_eq!(value(&r, "additional_child_credit"), 1700);
    assert_eq!(r.refund, Some(612_700));
}
// M2-F boundary case: head of household requires paying more than half the
// cost of keeping up the home (household() in federal_rules.rs: `valid`
// requires `s.paid_over_half_home` in addition to `f.qualifying_head`).
// `complete(FederalStatus::Head)` plus one qualifying child already
// satisfies every other head-of-household test (unmarried, a qualifying
// child who lived with the filer all year), so setting `paid_over_half_home`
// to false isolates this one requirement and must refuse the return.
#[test]
fn head_of_household_requires_paying_over_half_the_home_cost() {
    let mut d = complete(FederalStatus::Head);
    add_child(&mut d);
    d.setup.as_mut().unwrap().paid_over_half_home = false;
    unsupported(&d, "Filing status");
}
// Family unit (2026-10-08), defect 1, completed by the Forms 8332 and 2120
// unit with the marital-status fact. IRC section 2(b)(1)(A)(i) makes a
// qualifying child under section 152(c) a head-of-household qualifying person
// without asking that the child be a dependent, and Pub. 501 (2025) Table 4
// says a single qualifying child is a qualifying person "whether or not the
// child meets the citizen or resident test"; section 2(b)(1)(A)(i)(I)-(II)
// excludes a child who is married at the close of the year and is not a
// dependent by reason of section 152(b)(2) or (b)(3): Table 4 says a married
// child counts only if the filer can claim the child, and a child who is not
// a citizen, national or resident cannot be claimed. A citizen or resident
// child therefore qualifies the filer married or not; a child who is not one
// qualifies the filer only if unmarried, now a stored fact
// (`married_at_year_end`).
// Fact pattern: an unmarried filer who chose head of household, $60,000 of
// wages, and a ten-year-old child (ITIN, not a citizen, national or
// resident) who lived with the filer all year in a home the filer kept up.
// Head of household stands: the standard deduction is $24,150, taxable
// income $35,850, the Tax Table midpoint $35,875 taxes 10% of $17,700 plus
// 12% of $18,175 = $1,770 + $2,181 = $3,951 (the Single amounts, $16,100 and
// $5,023, would apply if the child did not qualify). The child is not a
// dependent, so no credit for other dependents. Married at year end the same
// child no longer qualifies: Unsupported on the filing status.

#[test]
fn head_of_household_takes_a_non_citizen_child_unless_the_child_is_married() {
    let mut d = complete(FederalStatus::Head);
    noncitizen_child(&mut d);
    let r = supported(&d);
    assert_eq!(value(&r, "deduction"), 24150);
    assert_eq!(value(&r, "taxable_income"), 35850);
    assert_eq!(value(&r, "regular_income_tax"), 3951);
    assert_eq!(value(&r, "child_credit"), 0);
    // No marital-status unknown remains: no note, no substituted status.
    assert!(r.review_notes.iter().all(|n| !n.display.key.contains("MARITAL")));
    assert_eq!(d.setup.as_ref().unwrap().filing_status(), FederalStatus::Head);
    // Married at the close of the year and not a U.S. citizen, national or
    // resident: not a dependent by reason of section 152(b)(3), so not a
    // head-of-household qualifying person.
    set_fact(&mut d.dependents[0], PersonFacts::MARRIED_AT_YEAR_END, true);
    unsupported(&d, "Filing status");
    // The same married child who is a citizen keeps head of household.
    set_fact(&mut d.dependents[0], PersonFacts::ABROAD, false);
    let r = supported(&d);
    assert_eq!(value(&r, "deduction"), 24150);
    // Another qualifying person beside the married non-citizen child.
    let mut d = complete(FederalStatus::Head);
    noncitizen_child(&mut d);
    set_fact(&mut d.dependents[0], PersonFacts::MARRIED_AT_YEAR_END, true);
    add_child(&mut d);
    assert_eq!(value(&supported(&d), "deduction"), 24150);
    // A single filer is unchanged; not paying over half the home cost stays
    // a refusal.
    let mut d = complete(FederalStatus::Single);
    noncitizen_child(&mut d);
    let r = supported(&d);
    assert_eq!(value(&r, "deduction"), 16100);
    assert_eq!(value(&r, "regular_income_tax"), 5023);
    let mut d = complete(FederalStatus::Head);
    noncitizen_child(&mut d);
    d.setup.as_mut().unwrap().paid_over_half_home = false;
    unsupported(&d, "Filing status");

    let mut d = complete(FederalStatus::Head);
    noncitizen_child(&mut d);
    // Controls for defect 1: only a qualifying child is exempt from the
    // dependent tests. A child who provides over half of their own support is not
    // a qualifying child (section 152(c)(1)(D)); a qualifying relative who is not
    // a citizen is not a dependent (section 152(b)(3)) and Table 4 requires that
    // the relative be claimable. Both stay refused.
    set_fact(&mut d.dependents[0], PersonFacts::OWN_SUPPORT, true);
    d.dependents[0].situation = DependentSituation::Supported {
        your_support_over_half: false,
        gross_income: d.dependents[0].gross_income(),
    };
    unsupported(&d, "Filing status");
    let mut d = complete(FederalStatus::Head);
    noncitizen_child(&mut d);
    let c = &mut d.dependents[0];
    c.relationship = Relationship::OtherRelative;
    c.birth_date = 19500101;
    unsupported(&d, "Filing status");
    // The same relative who is a citizen is a qualifying relative.
    set_fact(&mut d.dependents[0], PersonFacts::ABROAD, false);
    supported(&d);
}
// Family unit (2026-10-08), defect 2. IRC section 2(a)(1)(B) takes "a
// dependent (i) who (within the meaning of section 152, determined without
// regard to subsections (b)(1), b(2), and (d)(1)(B)) is a son, stepson,
// daughter, or stepdaughter"; Pub. 501 (2025) "Qualifying Surviving Spouse"
// test 3 allows a child or stepchild (not a foster child) "whom you can claim
// as a dependent or could claim ... except that" the child had gross income of
// $5,300 or more, filed a joint return, or you could be claimed by someone
// else, and test 4 asks that the child lived in the home all year. A son who
// is a qualifying relative, not a qualifying child, therefore counts. Fact
// pattern: a spouse who died in 2025, a 24-year-old son (not a student) who
// lived at home all year, supported by the filer, with $4,000 of gross income.
// Before: only a qualifying child counted and the estimate was refused. After:
// the joint standard deduction $32,200 gives taxable income $27,800; the Tax
// Table midpoint $27,825 taxes 10% of $24,800 plus 12% of $3,025 = $2,480 +
// $363 = $2,843, less the $500 credit for other dependents for this
// qualifying-relative dependent = $2,343. With $9,000 of gross income the son
// is not a dependent (section 152(d)(1)(B)) yet still qualifies the filer,
// since that test is waived: tax $2,843 and no credit.

#[test]
fn qualifying_survivor_counts_a_son_who_is_a_qualifying_relative() {
    let r = supported(&survivor_with_adult_son(400_000));
    assert_eq!(value(&r, "deduction"), 32200);
    assert_eq!(value(&r, "taxable_income"), 27800);
    assert_eq!(value(&r, "regular_income_tax"), 2843);
    assert_eq!(value(&r, "child_credit"), 500);
    let r = supported(&survivor_with_adult_son(900_000));
    assert_eq!(value(&r, "deduction"), 32200);
    assert_eq!(value(&r, "regular_income_tax"), 2843);
    assert_eq!(value(&r, "child_credit"), 0);
    // A son who filed a joint return other than for a refund (section
    // 152(b)(2) is waived) also counts when the filer supports the son.
    let mut d = survivor_with_adult_son(400_000);
    set_fact(&mut d.dependents[0], PersonFacts::FILED_JOINTLY, true);
    assert_eq!(value(&supported(&d), "deduction"), 32200);
}
// Controls for defect 2: the support test and the citizen or resident test
// (section 152(b)(3), not waived), a foster child, an adult who is not a son
// or daughter, a part-year child and a child who provides own support stay
// refused; the ordinary qualifying child is unchanged.
#[test]
fn qualifying_survivor_keeps_the_support_citizenship_and_relationship_tests() {
    let refused = |change: fn(&mut FederalDependent)| {
        let mut d = survivor_with_adult_son(400_000);
        change(&mut d.dependents[0]);
        unsupported(&d, "Filing status");
    };
    refused(|c| {
        c.situation = DependentSituation::Supported {
            your_support_over_half: false,
            gross_income: c.gross_income(),
        }
    });
    refused(|c| set_fact(c, PersonFacts::ABROAD, true));
    refused(|c| c.relationship = Relationship::Grandchild);
    refused(|c| c.relationship = Relationship::Foster);
    refused(|c| c.residency = DependentResidency::Days { days: 200 });
    // A young child who provides over half of their own support.
    refused(|c| {
        c.birth_date = 20180615;
        set_fact(c, PersonFacts::OWN_SUPPORT, true);
        c.situation = DependentSituation::Supported {
            your_support_over_half: false,
            gross_income: c.gross_income(),
        };
    });
    // The ordinary qualifying child, as before, even without a support answer.
    let mut d = survivor_with_adult_son(0);
    d.dependents[0].birth_date = 20180615;
    d.dependents[0].situation = DependentSituation::Supported {
        your_support_over_half: false,
        gross_income: d.dependents[0].gross_income(),
    };
    let r = supported(&d);
    assert_eq!(value(&r, "deduction"), 32200);
    assert_eq!(value(&r, "child_credit"), 2200);
}
#[test]
fn qualifying_survivor_uses_joint_rates_but_own_credit_thresholds() {
    let mut d = complete(FederalStatus::Surviving);
    d.setup.as_mut().unwrap().marital_state =
        MaritalState::WidowedPriorYear { could_file_joint: true };
    add_child(&mut d);
    let r = supported(&d);
    assert_eq!(value(&r, "deduction"), 32200);
    assert_eq!(value(&r, "child_credit"), 2200);
    d.dependents[0].residency = DependentResidency::Days { days: 200 };
    unsupported(&d, "Filing status");
}
#[test]
fn eitc_2026_phase_boundaries() {
    assert_eq!(eic_amount(8680, 8680, 0, false), 664);
    assert_eq!(eic_amount(13020, 13020, 1, false), 4427);
    assert_eq!(eic_amount(18290, 18290, 2, false), 7316);
    assert_eq!(eic_amount(18290, 18290, 3, false), 8231);
    assert_eq!(eic_amount(70244, 70244, 3, true), 0);
    assert_eq!(eic_amount(19540, 19540, 0, false), 0);
    assert_eq!(eic_amount(20000, 80000, 1, false), 0);
}
#[test]
fn three_child_payroll_refund_alternative() {
    let mut d = complete(FederalStatus::Head);
    d.income[0] = wage(1, Owner::Taxpayer, 1_000_000, 0);
    for _ in 0..3 {
        add_child(&mut d);
    }
    let r = supported(&d);
    assert_eq!(value(&r, "additional_child_credit"), 1125);
    assert_eq!(value(&r, "earned_income_credit"), 4511);
}
// C-fixture Schedule 8812 boundary (1), map row 6 line 9-11: MAGI exactly at
// the single $200,000 threshold rounds ceiling(0, $1,000) = 0 steps, so line
// 11 is $0.00 and the full $2,200 CTC applies unreduced; MAGI of $200,001 is
// $1 over, and ceiling(1, $1,000) = 1 step, so line 11 = 1 x $50 = $50, and
// the credit drops to $2,150. Wages equal AGI/MAGI here (no other income or
// adjustments); the resulting tax comfortably exceeds either credit amount,
// so neither case is limited by available tax.
#[test]
fn child_credit_phaseout_starts_at_the_first_dollar_over_the_threshold() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 20_000_000, 0);
    add_child(&mut d);
    assert_eq!(value(&supported(&d), "child_credit"), 2200);

    d.income[0] = wage(1, Owner::Taxpayer, 20_000_100, 0);
    assert_eq!(value(&supported(&d), "child_credit"), 2150);
}
// C-fixture Schedule 8812 boundary (2), map row 6 lines 4-7: family() only
// counts a dependent toward the $2,200 CTC (line 4) when the dependent's own
// `valid_ssn` is Yes; without an SSN but with a valid ITIN (`valid_tin`
// remains Yes), the same dependent falls through to the $500 ODC (line 6)
// instead. complete(Joint)'s default $100,000 wage produces enough tax to
// absorb the $500 nonrefundable credit in full.
#[test]
fn itin_child_earns_the_other_dependent_credit() {
    let mut d = complete(FederalStatus::Joint);
    add_child(&mut d);
    d.dependents[0].valid_ssn = false;
    let r = supported(&d);
    assert_eq!(value(&r, "child_credit"), 500);
    assert_eq!(value(&r, "additional_child_credit"), 0);
}
// C-fixture Schedule 8812 boundary (3), map row 6 line 27 "Zeroed whenever
// foreign.election() is true": a Form 2555 foreign earned-income exclusion
// election zeroes actc_limit unconditionally (forms/schedule_8812.rs
// `compute`), even though the nonrefundable CTC (line 14) is unaffected and
// still fully offsets the $100,000 domestic wage's tax. The excluded foreign
// wages are small enough (MAGI well under the $200,000 single threshold) that
// they do not themselves reduce the $2,200 potential.
#[test]
fn foreign_exclusion_election_zeroes_the_additional_child_credit() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(foreign_pay(20000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    add_child(&mut d);
    let r = supported(&d);
    assert_eq!(value(&r, "child_credit"), 2200);
    assert_eq!(value(&r, "additional_child_credit"), 0);
}
// C-fixture Schedule 8812 boundary (4), map row 6 lines 18a-20: the ACTC
// earned-income test is `15% x (earned - $2,500)`. At earned income of
// exactly $2,500 that base is $0, so the earned-income ceiling (line 20) is
// $0 regardless of the unused CTC potential; with one child (fewer than
// three), no three-or-more-children payroll alternative (lines 21-26)
// applies. The same $2,500 of wages is also too small to generate any tax,
// so the nonrefundable credit (line 14) is $0 as well: neither Form 1040
// line 19 nor line 28 is silently populated from the other.
#[test]
fn additional_child_credit_needs_earned_income_over_2500() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 250_000, 0);
    add_child(&mut d);
    let r = supported(&d);
    assert_eq!(value(&r, "child_credit"), 0);
    assert_eq!(value(&r, "additional_child_credit"), 0);
}
#[test]
fn care_2026_rates_and_joint_earned_income_caps() {
    for (agi, j, rate) in [
        (15000, false, 50),
        (15001, false, 49),
        (45000, false, 35),
        (75000, false, 35),
        (75001, false, 34),
        (103001, false, 20),
        (150000, true, 35),
        (150001, true, 34),
    ] {
        assert_eq!(care_rate(agi, j), rate);
    }
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 5_000_000, 0);
    let id = d.next_id;
    add(&mut d, wage(id, Owner::Spouse, 5_000_000, 0));
    add_child(&mut d);
    d.dependents[0].set_care_expenses(300_000);
    assert_eq!(
        {
            document_care(&mut d);
            supported(&d)
        }
        .tax,
        Some(439_300)
    );
}
#[test]
fn form_1098t_prepaid_period_keeps_paid_year_credit_and_refunds_count_once() {
    // Pub. 970 chapters 2–3, prepaid expenses and refunds:
    // https://www.irs.gov/publications/p970
    // A January–March 2027 term paid in 2026 stays in the 2026 credit.
    for (method, full_credit, reduced_credit) in [
        (EducationChoice::AmericanOpportunity, 2_250, 2_000),
        (EducationChoice::LifetimeLearning, 600, 400),
    ] {
        let (mut session, _) = at_the_credits_hub(complete(FederalStatus::Single));
        let page = crate::tests::support::client::income_pages::open_document_center(&mut session);
        let mut document = student(-1, 300_000);
        document.form.box_7 = true;
        let request = save_education(
            &mut session,
            EducationCredits {
                students: vec![document],
                people: education_value(&page).people.clone(),
            },
        );
        assert!(session.draft.credits.as_ref().unwrap().education.is_empty());
        accepted(&send(&mut session, request));
        let credits = session.draft.credits.as_ref().unwrap();
        assert_eq!(credits.education[0].form.box_1, 300_000);
        assert!(credits.education[0].form.box_7);
        assert!(credits.education_inputs().is_empty(), "annual answers are still owed");

        let mut topic = open_credits_topic(&mut session, TaxSection::Education);
        let P::SaveEducation { value: education, .. } = &mut topic else { unreachable!() };
        assert!(education.documents[0].following_year_academic_period);
        assert_eq!(education.documents[0].tuition, 300_000);
        assert_eq!(education.students[0].allowed_insurance_refund_allocations, vec![1],);
        assert_eq!(
            education.students[0].more.insurance_refund_current_expenses,
            SourceCents::NotReported,
        );
        education.students[0].method = Some(method);
        education.students[0].more.american_opportunity_used_up = Some(false);
        accepted(&send(&mut session, topic));
        let full = continued(&mut session);
        assert_eq!(
            value(&full, "education_credit") + value(&full, "refundable_education_credit"),
            full_credit,
        );

        // A separate $1,000 refund reduces $3,000 to $2,000 exactly once.
        let mut topic = open_credits_topic(&mut session, TaxSection::Education);
        let P::SaveEducation { value: education, .. } = &mut topic else { unreachable!() };
        education.students[0].more.refunds = 100_000;
        accepted(&send(&mut session, topic));
        let reduced = continued(&mut session);
        assert_eq!(
            value(&reduced, "education_credit") + value(&reduced, "refundable_education_credit"),
            reduced_credit,
        );
        assert_eq!(session.draft.credits.as_ref().unwrap().education[0].form.box_1, 300_000);
        let saved = session.draft.clone();
        let topic = open_credits_topic(&mut session, TaxSection::Education);
        accepted(&send(&mut session, topic));
        assert_eq!(session.draft, saved);
        let mut restored = restart(saved.clone());
        let topic = open_credits_topic(&mut restored, TaxSection::Education);
        accepted(&send(&mut restored, topic));
        assert_eq!(restored.draft, saved);
        assert_eq!(continued(&mut restored).tax, reduced.tax);

        // Correcting only the period checkbox preserves the paid-year amount
        // and the annual refund answer through the ordinary document editor.
        let mut paper =
            crate::tests::support::client::income_pages::open_document_center(&mut restored);
        let P::SaveDocuments { documents, .. } = &mut paper else { unreachable!() };
        documents.common.education.students[0].form.box_7 = false;
        accepted(&send(&mut restored, paper));
        let topic = open_credits_topic(&mut restored, TaxSection::Education);
        let P::SaveEducation { value: education, .. } = &topic else { unreachable!() };
        assert!(!education.documents[0].following_year_academic_period);
        assert_eq!(education.students[0].more.refunds, 100_000);
        accepted(&send(&mut restored, topic));
        assert_eq!(continued(&mut restored).tax, reduced.tax);
    }
}

#[test]
fn form_1098t_insurance_refunds_reduce_only_allocated_current_expenses() {
    // Form 1098-T box 10 and Pub. 970 chapters 2–3:
    // https://www.irs.gov/pub/irs-pdf/f1098t.pdf
    // https://www.irs.gov/publications/p970
    // $3,000 tuition less a separately reported $1,000 insurance refund
    // gives $2,000 current expenses only when the records allocate it here.
    for (method, original_credit, insured_credit, other_refund_credit, corrected_credit) in [
        (EducationChoice::AmericanOpportunity, 2_250, 2_000, 1_800, 2_075),
        (EducationChoice::LifetimeLearning, 600, 400, 360, 460),
    ] {
        let (mut session, _) = at_the_credits_hub(complete(FederalStatus::Single));
        let page = crate::tests::support::client::income_pages::open_document_center(&mut session);
        let request = save_education(
            &mut session,
            EducationCredits {
                students: vec![student(-1, 300_000)],
                people: education_value(&page).people.clone(),
            },
        );
        accepted(&send(&mut session, request));
        let mut topic = open_credits_topic(&mut session, TaxSection::Education);
        let P::SaveEducation { value: education, .. } = &mut topic else { unreachable!() };
        education.students[0].method = Some(method);
        education.students[0].more.american_opportunity_used_up = Some(false);
        accepted(&send(&mut session, topic));
        let original = continued(&mut session);
        assert_eq!(
            value(&original, "education_credit") + value(&original, "refundable_education_credit"),
            original_credit,
        );

        let mut paper =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        let P::SaveDocuments { documents, .. } = &mut paper else { unreachable!() };
        let mut insurer = student(-1, 0);
        insurer.form.student = documents.common.education.students[0].form.student.clone();
        insurer.form.box_8 = false;
        insurer.form.box_10 = 100_000;
        documents.common.education.students.push(insurer);
        assert_eq!(session.draft.credits.as_ref().unwrap().education.len(), 1);
        accepted(&send(&mut session, paper));
        let credits = session.draft.credits.as_ref().unwrap();
        assert_eq!(credits.education.len(), 2);
        assert_eq!(credits.education[0].form.box_1, 300_000);
        assert_eq!(credits.education[1].form.box_10, 100_000);
        assert_eq!(
            credits.students[0].more.insurance_refund_current_expenses,
            SourceCents::Unavailable,
        );
        assert!(
            credits
                .education_pending()
                .contains(&(-1, "EDUCATION_INSURANCE_REFUND_ALLOCATION_NEEDED"))
        );
        assert!(credits.education_inputs().is_empty());

        let mut topic = open_credits_topic(&mut session, TaxSection::Education);
        let P::SaveEducation { value: education, .. } = &mut topic else { unreachable!() };
        assert_eq!(education.documents[1].insurance_reimbursements, 100_000);
        assert_eq!(education.students[0].allowed_insurance_refund_allocations.len(), 3);
        education.students[0].more.insurance_refund_current_expenses =
            SourceCents::Reported { amount: 100_000 };
        accepted(&send(&mut session, topic));
        let insured = continued(&mut session);
        assert_eq!(
            value(&insured, "education_credit") + value(&insured, "refundable_education_credit"),
            insured_credit,
        );
        assert_eq!(
            session.draft.credits.as_ref().unwrap().education_inputs()[0].expenses.refunds,
            100_000
        );

        let mut topic = open_credits_topic(&mut session, TaxSection::Education);
        let P::SaveEducation { value: education, .. } = &mut topic else { unreachable!() };
        education.students[0].more.refunds = 20_000;
        accepted(&send(&mut session, topic));
        let reduced = continued(&mut session);
        assert_eq!(
            value(&reduced, "education_credit") + value(&reduced, "refundable_education_credit"),
            other_refund_credit,
        );
        assert_eq!(
            session.draft.credits.as_ref().unwrap().education_inputs()[0].expenses.refunds,
            120_000
        );
        let saved = session.draft.clone();
        let topic = open_credits_topic(&mut session, TaxSection::Education);
        accepted(&send(&mut session, topic));
        assert_eq!(session.draft, saved);
        let mut restored = restart(saved.clone());
        let topic = open_credits_topic(&mut restored, TaxSection::Education);
        accepted(&send(&mut restored, topic));
        assert_eq!(restored.draft, saved);
        assert_eq!(continued(&mut restored).tax, reduced.tax);

        // Correcting the accepted insurance receipt reopens only its source
        // allocation; it retains the separate refund and chosen credit.
        let mut paper =
            crate::tests::support::client::income_pages::open_document_center(&mut restored);
        let P::SaveDocuments { documents, .. } = &mut paper else { unreachable!() };
        documents.common.education.students[1].form.box_10 = 50_000;
        accepted(&send(&mut restored, paper));
        assert!(
            restored
                .draft
                .credits
                .as_ref()
                .unwrap()
                .education_pending()
                .contains(&(-1, "EDUCATION_INSURANCE_REFUND_ALLOCATION_NEEDED"))
        );
        let mut topic = open_credits_topic(&mut restored, TaxSection::Education);
        let P::SaveEducation { value: education, .. } = &mut topic else { unreachable!() };
        assert_eq!(education.students[0].method, Some(method));
        assert_eq!(education.students[0].more.refunds, 20_000);
        assert_eq!(
            education.students[0].more.insurance_refund_current_expenses,
            SourceCents::Unavailable
        );
        education.students[0].more.insurance_refund_current_expenses =
            SourceCents::Reported { amount: 50_000 };
        accepted(&send(&mut restored, topic));
        let corrected = continued(&mut restored);
        assert_eq!(
            value(&corrected, "education_credit")
                + value(&corrected, "refundable_education_credit"),
            corrected_credit,
        );
    }
}

#[test]
fn form_1098t_insurance_refunds_keep_unknown_and_prior_year_history_unresolved() {
    let (mut session, _) = at_the_credits_hub(complete(FederalStatus::Single));
    let page = crate::tests::support::client::income_pages::open_document_center(&mut session);
    let mut insurer = student(-1, 0);
    insurer.form.box_8 = false;
    insurer.form.box_10 = 100_000;
    let request = save_education(
        &mut session,
        EducationCredits {
            students: vec![student(-1, 300_000), insurer],
            people: education_value(&page).people.clone(),
        },
    );
    accepted(&send(&mut session, request));
    for (allocation, current_refund, key) in [
        (SourceCents::Unavailable, None, "EDUCATION_INSURANCE_REFUND_ALLOCATION_NEEDED"),
        (SourceCents::NotReported, Some(0), "ISSUE_EDUCATION_RECAPTURE_COMPLEX_PRIOR_RETURN"),
        (
            SourceCents::Reported { amount: 50_000 },
            Some(50_000),
            "ISSUE_EDUCATION_RECAPTURE_COMPLEX_PRIOR_RETURN",
        ),
    ] {
        let mut topic = open_credits_topic(&mut session, TaxSection::Education);
        let P::SaveEducation { value: education, .. } = &mut topic else { unreachable!() };
        education.students[0].method = Some(EducationChoice::LifetimeLearning);
        education.students[0].more.american_opportunity_used_up = Some(false);
        education.students[0].more.insurance_refund_current_expenses = allocation.clone();
        accepted(&send(&mut session, topic));
        let credits = session.draft.credits.as_ref().unwrap();
        assert_eq!(credits.students[0].more.insurance_refund_current_expenses, allocation);
        assert!(credits.education_pending().contains(&(-1, key)));
        assert_eq!(
            credits.education_inputs().first().map(|row| row.expenses.refunds),
            current_refund
        );
        assert!(credits.education_recapture.is_none(), "receipt year establishes no prior history");
        assert_ne!(estimate(&session.draft).state, ResultState::EstimatedForSupportedScope);
        let restored = restart(session.draft.clone());
        assert_eq!(restored.draft, session.draft);
        assert!(restored.draft.credits.as_ref().unwrap().education_pending().contains(&(-1, key)));
    }

    // An independently entered 2025 history has no source/student link to
    // this receipt. Even a matching $500 reduction cannot establish one.
    let request = save_more_credits(
        &session.draft,
        MoreCredits {
            answers: Vec::new(),
            documents: Vec::new(),
            credits: vec![MoreCredit::EducationRecapture(PriorYearEducationCreditRecapture {
                method: EducationMethod::AmericanOpportunity,
                filed_qualified_expenses: 400_000,
                after_filing_reduction: 50_000,
                filed_education_magi: 5_000_000,
                filed_joint: false,
                filed_credit_used: 250_000,
                ordinary_prior_return: true,
            })],
        },
    );
    accepted(&send(&mut session, request));
    let credits = session.draft.credits.as_ref().unwrap();
    assert_eq!(credits.education_recapture.as_ref().unwrap().additional_tax(), Ok(125));
    assert!(
        credits
            .education_pending()
            .contains(&(-1, "ISSUE_EDUCATION_RECAPTURE_COMPLEX_PRIOR_RETURN"))
    );
    assert_ne!(estimate(&session.draft).state, ResultState::EstimatedForSupportedScope);
}

#[test]
fn education_credits_and_claimant_refund_restriction() {
    let mut d = complete(FederalStatus::Single);
    let sid0_1578 = -1; // You, the first person of the catalog
    add_education(
        d.credits.as_mut().unwrap(),
        native_education(
            sid0_1578,
            education_expenses(400_000),
            education_choice(EducationMethod::AmericanOpportunity, Answer::Yes),
        ),
    );
    let r = supported(&d);
    assert_eq!(r.tax, Some(352_300));
    assert_eq!(value(&r, "refundable_education_credit"), 1000);
    d.people.as_mut().unwrap().taxpayer.birth_date = 20060615;
    d.people.as_mut().unwrap().taxpayer.full_time_student = true;
    d.people.as_mut().unwrap().taxpayer.support_share = SupportShare::LessThanHalf;
    let r = supported(&d);
    assert_eq!(value(&r, "refundable_education_credit"), 0);
    assert_eq!(r.tax, Some(252_300));
}
#[test]
fn education_credit_requires_a_valid_student_ssn() {
    let mut d = complete(FederalStatus::Single);
    let sid0_1600 = -1; // You, the first person of the catalog
    add_education(
        d.credits.as_mut().unwrap(),
        native_education(
            sid0_1600,
            education_expenses(400_000),
            education_choice(EducationMethod::AmericanOpportunity, Answer::Yes),
        ),
    );
    let expected = supported(&d);
    assert_eq!(expected.tax, Some(352_300));
    // The student catalog carries the taxpayer's own SSN answer from the filer
    // page; without an SSN issued by the due date the student takes no credit
    // (Form 8863 line 21).
    d.people.as_mut().unwrap().taxpayer.valid_ssn = false;
    let r = supported(&d);
    assert!(r.tax > expected.tax, "{r:?}");
}
#[test]
fn prior_education_credit_refund_recapture_reaches_2026_tax_and_survives_restart() {
    let mut d = complete(FederalStatus::Single);
    let original = supported(&d);
    d.credits.as_mut().unwrap().education_recapture = Some(PriorYearEducationCreditRecapture {
        method: EducationMethod::AmericanOpportunity,
        filed_qualified_expenses: 700_000,
        after_filing_reduction: 400_000,
        filed_education_magi: 5_000_000,
        filed_joint: false,
        filed_credit_used: 250_000,
        ordinary_prior_return: true,
    });
    let calculated = supported(&d);
    assert_eq!(value(&calculated, "education_credit_recapture_tax"), 250);
    assert_eq!(
        value(&calculated, "chapter_one_net_tax"),
        value(&original, "chapter_one_net_tax") + 250
    );
    assert_eq!(calculated.tax, original.tax.map(|value| value + 25_000));
    // The recapture is Form 1040 line 16 box 3, not Schedule 2 (line 23).
    assert_eq!(return_line(&calculated, "16"), return_line(&original, "16") + 25_000);
    assert_eq!(return_line(&calculated, "23"), return_line(&original, "23"));
    assert_eq!(supported(&restart(d.clone()).draft).tax, calculated.tax);

    d.credits.as_mut().unwrap().education_recapture.as_mut().unwrap().ordinary_prior_return = false;
    unsupported(&d, "Education recapture");
    d.credits.as_mut().unwrap().education_recapture.as_mut().unwrap().ordinary_prior_return = true;
    d.credits.as_mut().unwrap().education_recapture.as_mut().unwrap().filed_credit_used = 200_000;
    unsupported(&d, "Education recapture");
    let entry = d.credits.as_mut().unwrap().education_recapture.as_mut().unwrap();
    entry.filed_credit_used = 250_000;
    entry.filed_joint = true;
    unsupported(&d, "Education recapture");
    let entry = d.credits.as_mut().unwrap().education_recapture.as_mut().unwrap();
    entry.filed_joint = false;
    entry.after_filing_reduction = 800_000;
    assert_eq!(estimate(&d).state, ResultState::Incomplete);
}
#[test]
fn care_ceiling_counts_a_second_child_even_when_only_one_has_expenses() {
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 5_000_000, 0);
    let id = d.next_id;
    add(&mut d, wage(id, Owner::Spouse, 5_000_000, 0));
    add_child(&mut d);
    add_child(&mut d);
    d.dependents[0].set_care_expenses(600_000);
    assert_eq!(
        value(
            &{
                document_care(&mut d);
                supported(&d)
            },
            "care_credit"
        ),
        2100
    );
}
#[test]
fn joint_status_change_opens_the_spouse_page_and_keeps_later_pages() {
    let d = complete(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, d.clone());
    // Becoming joint erases nothing; the spouse page is sent because the
    // spouse record is missing, even from the review.
    let reply = send(
        &mut s,
        P::SaveSetup {
            user: TEST_USER,
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: 0,
            previous: TaxSection::Review as i32,
            review: true,
            estimate: None,
            warnings: vec![],
            progress: vec![],
            value: ReturnSetup { filing: Filing::MarriedJointly },
        },
    );
    assert_eq!(s.draft.setup.as_ref().unwrap().status, filing_choice(FederalStatus::Joint));
    assert!(matches!(reply, Reply::Response(P::BeginSpouse { .. })), "{reply:?}");
    assert_eq!(s.draft.income, d.income);
    assert!(s.draft.finished_sections.contains(&TaxSection::Income));
    // A joint return without its spouse is never estimated.
    let r = estimate(&s.draft);
    assert_ne!(r.state, ResultState::EstimatedForSupportedScope);
    assert_eq!(r.issues[0].section, "Filers");
    let old = s.draft.clone();
    // The filer page alone does not complete a joint return's filers: the
    // spouse page follows and the review stays closed until it is saved.
    let reply = send(
        &mut s,
        P::SavePeople {
            professional_election: crate::ProfessionalRentalElection::SeparateActivities,
            allowed_professional_elections: vec![0, 1],
            user: TEST_USER,
            cookie: old.cookie.clone(),
            revision: old.revision,
            page: 1,
            previous: 0,
            review: false,
            estimate: None,
            warnings: vec![],
            progress: vec![],
            you: crate::interview::filer(&d.people.unwrap().taxpayer),
        },
    );
    assert!(matches!(reply, Reply::Response(P::BeginSpouse { .. })));
    // The unchanged filer page stores nothing.
    assert_eq!(s.draft, old);
    assert!(s.draft.people.as_ref().unwrap().spouse.is_none());
    assert_ne!(estimate(&s.draft).state, ResultState::EstimatedForSupportedScope);
}
#[test]
fn work_deductions_share_joint_caps_and_require_claimant_ssn() {
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 8_000_000, 0);
    let id = d.next_id;
    add(&mut d, wage(id, Owner::Spouse, 8_000_000, 0));
    for entry in &mut d.income {
        let FederalIncome::Wage { data: w } = &mut entry.income else { panic!() };
        let mut w = edit_wage(w);
        w.work_deductions = work_deductions(2_000_000, 2_000_000);
    }
    let r = supported(&d);
    assert_eq!(value(&r, "tips_deduction"), 25000);
    assert_eq!(value(&r, "overtime_deduction"), 25000);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().valid_ssn = false;
    let r = supported(&d);
    assert_eq!(value(&r, "tips_deduction"), 20000);
    assert_eq!(value(&r, "overtime_deduction"), 20000);
}

#[test]
fn spouse_living_choice_preserves_the_two_tax_period_tests() {
    let mut setup = complete(FederalStatus::Separate).setup.unwrap();
    for (living, all_year, last_six) in [
        (SpouseLiving::Together, false, false),
        (SpouseLiving::LastSixMonths, false, true),
        (SpouseLiving::AllYear, true, true),
    ] {
        setup.marital_state = MaritalState::Married { living };
        assert_eq!(setup.lived_apart_all_year(), all_year);
        assert_eq!(setup.lived_apart_last_six_months(), last_six);
    }
    setup.marital_state = MaritalState::Unmarried;
    assert!(!setup.lived_apart_all_year());
    assert!(!setup.lived_apart_last_six_months());
}

#[test]
fn education_expenses_derive_the_selected_credit_base() {
    let mut expenses = education_expenses(300_000);
    expenses.school_materials = 50_000;
    expenses.other_materials = 100_000;
    expenses.assistance = 40_000;
    expenses.refunds = 20_000;
    expenses.other_benefits = 30_000;
    assert_eq!(expenses.qualified(EducationMethod::AmericanOpportunity), 360_000);
    assert_eq!(expenses.qualified(EducationMethod::LifetimeLearning), 260_000);
    let mut draft = complete(FederalStatus::Single);
    let sid0_1899 = -1; // You, the first person of the catalog
    add_education(
        draft.credits.as_mut().unwrap(),
        native_education(
            sid0_1899,
            expenses.clone(),
            education_choice(EducationMethod::AmericanOpportunity, Answer::Yes),
        ),
    );
    let result = supported(&draft);
    assert_eq!(
        value(&result, "education_credit") + value(&result, "refundable_education_credit"),
        2400
    );
    expenses.assistance = 1_000_000;
    assert_eq!(expenses.qualified(EducationMethod::AmericanOpportunity), 0);
}

#[test]
fn exclusion_magi_is_restored_for_niit_and_child_phaseout() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_pay(250000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    add(
        &mut d,
        income(FederalIncome::Interest { data: interest_source(native_interest(1_000_000, 0)) }),
    );
    let dep = child(d.next_id);
    d.next_id += 1;
    d.dependents.push(dep);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 127100);
    assert_eq!(value(&r, "net_investment_income_tax"), 380);
    assert_eq!(value(&r, "child_credit"), 0);
}

#[test]
fn adoption_timing_special_needs_and_prior_claim_cap() {
    let mut d = complete(FederalStatus::Single);
    let child = adoption_child(
        AdoptionKind::Domestic { finalized: AdoptionFinalization::Pending, special_needs: false },
        &[(2024, 1000), (2025, 3000), (2026, 6000)],
    );
    d.credits.as_mut().unwrap().adoptions.push(child);
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_refundable_credit"), 3000);
    assert_eq!(value(&r, "adoption_nonrefundable_credit"), 0);
    d.credits.as_mut().unwrap().adoptions[0].kind = AdoptionKind::Domestic {
        finalized: AdoptionFinalization::Final { year: 2026 },
        special_needs: false,
    };
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_refundable_credit"), 5120);
    assert_eq!(value(&r, "adoption_nonrefundable_credit"), 3880);
    d.credits.as_mut().unwrap().adoptions[0].kind =
        AdoptionKind::Foreign { finalized: AdoptionFinalization::Final { year: 2026 } };
    assert_eq!(value(&supported(&d), "adoption_nonrefundable_credit"), 4880);
    d.credits.as_mut().unwrap().adoptions[0].kind =
        AdoptionKind::Foreign { finalized: AdoptionFinalization::Pending };
    assert_eq!(value(&supported(&d), "adoption_refundable_credit"), 0);
    let child = &mut d.credits.as_mut().unwrap().adoptions[0];
    child.kind = AdoptionKind::Domestic {
        finalized: AdoptionFinalization::Final { year: 2026 },
        special_needs: true,
    };
    child.expenses.clear();
    child.prior = AdoptionPriorCredit { previous: 500_000, expenses: 1_000_000 };
    assert_eq!(value(&supported(&d), "adoption_refundable_credit"), 2670);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn adoption_child_credit_coordination_preserves_actc_and_carries_unused_credit() {
    let mut d = complete(FederalStatus::Single);
    add_child(&mut d);
    d.credits.as_mut().unwrap().adoptions.push(adoption_child(
        AdoptionKind::Domestic {
            finalized: AdoptionFinalization::Final { year: 2026 },
            special_needs: true,
        },
        &[],
    ));
    let r = supported(&d);
    assert_eq!(value(&r, "child_credit"), 500);
    assert_eq!(value(&r, "additional_child_credit"), 1700);
    assert_eq!(value(&r, "adoption_refundable_credit"), 5120);
    assert_eq!(value(&r, "adoption_nonrefundable_credit"), 4523);
    assert_eq!(value(&r, "adoption_carryforward"), 8027);
    assert_eq!(r.tax, Some(0));
    assert_eq!(return_line(&r, "30"), 512_000);
    assert_eq!(r.refundable_credits, Some(682_000));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn alimony_instrument_dates_modifications_and_child_priority() {
    let mut v = alimony_record(20181231);
    assert_eq!(v.amount(), 1000000);
    v.paid = 2500000;
    assert_eq!(v.amount(), 1200000); // No deduction for excess voluntary payments.
    v.paid = 700000;
    assert_eq!(v.amount(), 0); // Underpayment is child support first.
    v.paid = 1800000;
    v.agreement = 20190101;
    assert_eq!(v.amount(), 0);
    v.agreement = 20181231;
    v.new_rules = AlimonyNewRules::BeforeReturnYear;
    assert_eq!(v.amount(), 0);
}

#[test]
fn alimony_paid_and_received_flow_through_return_without_double_counting() {
    let mut d = complete(FederalStatus::Single);
    d.adjustments.as_mut().unwrap().alimony_paid = vec![paid_alimony(alimony_record(20181231))];
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 50000);
    assert_eq!(return_line(&r, "10"), 1000000);
    assert_eq!(value(&r, "alimony_paid_deduction"), 10000);
    d.adjustments.as_mut().unwrap().alimony_paid.clear();
    add(&mut d, income(FederalIncome::Alimony { data: alimony_record(20181231) }));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 70000);
    assert_eq!(return_line(&r, "8"), 1000000);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(restart(d.clone()).draft, d);
    d.income[1].income = FederalIncome::Alimony { data: alimony_record(20190101) };
    assert_eq!(value(&supported(&d), "agi"), 60000);
}

#[test]
fn alimony_special_treatment_remains_visible_even_with_no_current_payment() {
    let mut d = complete(FederalStatus::Single);
    let mut v = alimony_record(20100101);
    v.paid = 0;
    v.eligible = false;
    d.adjustments.as_mut().unwrap().alimony_paid = vec![paid_alimony(v)];
    // A confirmed No still takes no deduction when there is no current payment.
    assert_eq!(value(&supported(&d), "agi"), 60000);

    // A pre-1985 instrument's special rules are refused by name.
    d.adjustments.as_mut().unwrap().alimony_paid[0].payment.agreement = 19840101;
    unsupported(&d, "Alimony");
}

#[test]
fn alimony_recapture_matches_irs_example_for_both_parties() {
    let mut payer = complete(FederalStatus::Single);
    payer.adjustments.as_mut().unwrap().alimony_paid = vec![paid_alimony(recapture_alimony())];
    let r = supported(&payer);
    assert_eq!(value(&r, "alimony_recapture_income"), 1500);
    assert_eq!(return_line(&r, "8"), 150000);
    assert_eq!(return_line(&r, "10"), 2800000);
    assert_eq!(value(&r, "agi"), 33500);
    let mut recipient = complete(FederalStatus::Single);
    add(&mut recipient, income(FederalIncome::Alimony { data: recapture_alimony() }));
    let r = supported(&recipient);
    assert_eq!(value(&r, "alimony_recapture_deduction"), 1500);
    assert_eq!(return_line(&r, "8"), 2800000);
    assert_eq!(return_line(&r, "10"), 150000);
    assert_eq!(value(&r, "agi"), 86500);
    assert_eq!(restart(recipient.clone()).draft, recipient);
    assert_eq!(restart(payer.clone()).draft, payer);
}

#[test]
fn alimony_recapture_zero_payments_exceptions_and_excluded_amounts() {
    let mut v = recapture_alimony();
    let mut r = supported(&complete(FederalStatus::Single));
    v.paid = 0;
    assert_eq!(
        v.recaptured(
            &mut r,
            &crate::IssueContext::Section(TaxSection::MoreAdjustments),
            true,
            "Alimony recapture (paid)"
        ),
        5150000
    );
    let AlimonyRecapture::ThirdYear { death_or_remarriage, .. } = &mut v.recapture else {
        unreachable!()
    };
    *death_or_remarriage = true;
    assert_eq!(
        v.recaptured(
            &mut r,
            &crate::IssueContext::Section(TaxSection::MoreAdjustments),
            true,
            "Alimony recapture (paid)"
        ),
        0
    );
    v = recapture_alimony();
    let AlimonyRecapture::ThirdYear { first_excluded, second_excluded, current_excluded, .. } =
        &mut v.recapture
    else {
        unreachable!()
    };
    *first_excluded = 5000000;
    *second_excluded = 3900000;
    *current_excluded = 2800000;
    assert!(v.valid());
    assert_eq!(
        v.recaptured(
            &mut r,
            &crate::IssueContext::Section(TaxSection::MoreAdjustments),
            true,
            "Alimony recapture (paid)"
        ),
        0
    );
    let AlimonyRecapture::ThirdYear { first_excluded, .. } = &mut v.recapture else {
        unreachable!()
    };
    *first_excluded += 1;
    assert!(!v.valid());
}

#[test]
fn alimony_compensation_distinguishes_decrees_and_spouse_recapture() {
    let mut d = empty_return(FederalStatus::Single);
    let mut payment = alimony_record(20100101);
    payment.decree = false;
    add(&mut d, income(FederalIncome::Alimony { data: payment }));
    assert_eq!(value(&supported(&d), "agi"), 10000);
    d.adjustments.as_mut().unwrap().iras = ira_facts(100000, 0, Answer::No);
    d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    unsupported(&d, "IRA");

    let mut joint = complete(FederalStatus::Joint);
    joint.adjustments.as_mut().unwrap().alimony_paid =
        vec![AlimonyPaid { owner: Owner::Spouse, payment: recapture_alimony() }];
    assert_eq!(value(&supported(&joint), "alimony_recapture_income"), 1500);
    assert_eq!(restart(joint.clone()).draft, joint);
    let mut single = complete(FederalStatus::Single);
    single.adjustments.as_mut().unwrap().alimony_paid = joint.adjustments.unwrap().alimony_paid;
    unsupported(&single, "Alimony");
}

#[test]
fn care_credit_precedes_education_when_tax_capacity_is_limited() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 3000000, 0);
    add_child(&mut d);
    d.dependents[0].set_care_expenses(300000);
    let sid0_2217 = -1; // You, the first person of the catalog
    add_education(
        d.credits.as_mut().unwrap(),
        native_education(
            sid0_2217,
            education_expenses(400000),
            education_choice(EducationMethod::AmericanOpportunity, Answer::Yes),
        ),
    );
    let r = {
        document_care(&mut d);
        supported(&d)
    };
    // 2026 single taxable income $13,900 uses the $13,900–13,950 Tax Table row.
    // Care: $3,000 * 42% = $1,260; education uses the remaining $163.
    assert_eq!(value(&r, "regular_income_tax"), 1423);
    assert_eq!(value(&r, "care_credit"), 1260);
    assert_eq!(value(&r, "education_credit"), 163);
    assert_eq!(value(&r, "refundable_education_credit"), 1000);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn prior_year_care_uses_previous_rate_and_does_not_require_current_qualifying_child() {
    let mut d = complete(FederalStatus::Single);
    d.credits.as_mut().unwrap().care =
        FederalCare { prior_year: Some(prior_care_facts()), ..FederalCare::default() };
    document_care(&mut d);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_year_care_credit"), 132); // 600 * 22%, not 2026's 35%.
    assert_eq!(value(&r, "care_credit"), 132);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn prior_year_care_caps_and_history_validation() {
    let mut r = supported(&complete(FederalStatus::Single));
    let mut p = prior_care_facts();
    p.paid_in_return_year = 1000000;
    assert_eq!(p.credit(&mut r), 220);
    p.prior_benefits = 100000;
    assert_eq!(p.credit(&mut r), 0);
    // Copied filed line6 need not equal an invented recomputation.
    p.prior_line_6 = 0;
    assert!(p.valid());
    assert_eq!(p.credit(&mut r), 440);
    p.paid_in_return_year = 60000;
    p.prior_line_6 = 200001;
    assert!(!p.valid());
}

#[test]
fn prior_care_copied_facts_compute_and_explicit_none_claims_no_credit() {
    let mut d = complete(FederalStatus::Single);
    // The retired eligibility answer is no longer a gate. Actual copied facts
    // compute the independently derived $132 credit; absence claims none.
    d.credits.as_mut().unwrap().care.prior_year = Some(prior_care_facts());
    assert_eq!(value(&supported(&d), "prior_year_care_credit"), 132);
    d.credits.as_mut().unwrap().care.prior_year = None;
    assert_eq!(value(&supported(&d), "care_credit"), 0);
}

#[test]
fn two_employers_both_spouses_share_one_care_exclusion_and_credit_ceiling() {
    let mut d = joint_employer_care_return();
    let r = supported(&d);
    assert!(
        matches!(&d.income[0].income, FederalIncome::Wage { data } if FederalWage::from(data).care_benefits == 200_000)
    );
    assert!(
        matches!(&d.income[1].income, FederalIncome::Wage { data } if FederalWage::from(data).care_benefits == 200_000)
    );
    assert_eq!(value(&r, "care_benefits_excluded"), 4000);
    assert_eq!(value(&r, "care_benefits_taxable"), 0);
    assert_eq!(value(&r, "care_credit"), 700); // ($6,000 - $4,000) at 35%.
    assert_eq!(return_line(&r, "1a"), 14_000_000);
    assert_eq!(return_line(&r, "1e"), 0);
    assert_eq!(return_line(&r, "25a"), 900_000);
    let restored = restart(d.clone()).draft;
    assert_eq!(restored, d);
    assert_eq!(supported(&restored).return_lines, r.return_lines);

    care_box10(&mut d, 1, 500_000);
    let r = supported(&d);
    assert_eq!(value(&r, "care_benefits_excluded"), 7000);
    assert_eq!(value(&r, "care_credit"), 0); // One shared $6,000 care-credit ceiling.
    assert_eq!(return_line(&r, "1a"), 14_000_000);
}
#[test]
fn dependent_care_grace_period_forfeiture_and_expense_limit() {
    let mut d = benefit_return(FederalStatus::Single);
    care_costs(&mut d, 300_000);
    d.credits.as_mut().unwrap().care.grace =
        vec![CareGraceBalance { owner: Owner::Taxpayer, used_in_return_year: 100_000 }];
    d.credits.as_mut().unwrap().care.benefits[0].forfeited = Some(50_000);
    let r = supported(&d);
    assert_eq!(value(&r, "care_benefits_excluded"), 3000);
    assert_eq!(value(&r, "care_benefits_taxable"), 2500);
    assert_eq!(value(&r, "care_credit"), 0);
    d.credits.as_mut().unwrap().care.benefits[0].forfeited = Some(600_000);
    assert!(!d.valid_stored());
}

#[test]
fn dependent_care_nonworking_spouse_limits_exclusion_without_a_false_failure() {
    let mut d = benefit_return(FederalStatus::Joint);
    let r = supported(&d);
    assert_eq!(value(&r, "care_benefits_excluded"), 0);
    assert_eq!(value(&r, "care_benefits_taxable"), 5000);
    assert_eq!(value(&r, "care_credit"), 0);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().full_time_student = true;
    assert_eq!(value(&supported(&d), "care_benefits_excluded"), 0);
}

#[test]
fn dependent_care_spouse_benefits_and_unknown_or_missing_qualification() {
    let mut d = benefit_return(FederalStatus::Joint);
    let id = d.next_id;
    add(&mut d, wage(id, Owner::Spouse, 400000, 0));
    clear_care_benefits(&mut d);
    let sid = w2(&d, id);
    care_w2(&mut d, sid, 500000, 0, 0, 750000);
    let r = supported(&d);
    assert_eq!(value(&r, "care_benefits_excluded"), 4000);
    assert_eq!(value(&r, "care_benefits_taxable"), 1000);
    assert_eq!(return_line(&r, "1e"), 100000);
    // Actual qualifying-person facts replace the retired eligibility answer.
    d.dependents.clear();
    let r = supported(&d);
    assert_eq!(value(&r, "care_benefits_excluded"), 0);
    assert_eq!(value(&r, "care_benefits_taxable"), 5000);
}

#[test]
fn dependent_care_full_exclusion_preserves_prior_year_credit() {
    let mut d = benefit_return(FederalStatus::Single);
    care_box10(&mut d, 0, 750000);
    d.credits.as_mut().unwrap().care.prior_year = Some(prior_care_facts());
    let r = supported(&d);
    assert_eq!(value(&r, "care_benefits_excluded"), 7500);
    assert_eq!(value(&r, "care_credit"), 132);
}

#[test]
fn both_student_spouses_share_each_month_allowance_once() {
    let mut d = benefit_return(FederalStatus::Joint);
    clear_care_benefits(&mut d);
    d.income.clear();
    let mut payment = alimony_record(20180101);
    payment.paid = 6000000;
    payment.alimony_due = 6000000;
    payment.child_support_due = 0;
    add(&mut d, income(FederalIncome::Alimony { data: payment }));
    d.people.as_mut().unwrap().taxpayer.full_time_student = true;
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().full_time_student = true;
    let mut months = care_months(Owner::Taxpayer, 12);
    months.extend(care_months(Owner::Spouse, 12));
    set_care_months(&mut d, months);
    let r = supported(&d);
    // ApprovedF2d: common months cannot manufacture income for both.
    assert_eq!(value(&r, "care_earned_taxpayer"), 6000);
    assert_eq!(value(&r, "care_earned_spouse"), 0);
    assert_eq!(value(&r, "care_credit"), 0);
    assert_eq!(value(&r, "earned_income_credit"), 0);
    assert_eq!(value(&r, "agi"), 60000);
}
#[test]
fn student_spouse_months_coordinate_employee_benefit_exclusion() {
    let mut d = benefit_return(FederalStatus::Joint);
    care_box10(&mut d, 0, 750000);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().full_time_student = true;
    set_care_months(&mut d, care_months(Owner::Spouse, 12));
    let r = supported(&d);
    assert_eq!(value(&r, "care_benefits_excluded"), 6000);
    assert_eq!(value(&r, "care_benefits_taxable"), 1500);
    assert_eq!(value(&r, "agi"), 101500);
    assert_eq!(value(&r, "care_credit"), 0);
}
#[test]
fn care_selected_months_reject_duplicates() {
    let mut d = benefit_return(FederalStatus::Joint);
    clear_care_benefits(&mut d);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().full_time_student = true;
    let mut months = care_months(Owner::Spouse, 5);
    months[1].month = 1;
    set_care_months(&mut d, months);
    assert!(!d.valid_stored());
    set_care_months(&mut d, care_months(Owner::Spouse, 5));
    assert!(d.valid_stored());
}

#[test]
fn care_student_treatment_requires_five_months() {
    let mut d = benefit_return(FederalStatus::Joint);
    clear_care_benefits(&mut d);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().full_time_student = true;
    let id = d.next_id;
    add(&mut d, wage(id, Owner::Spouse, 50000, 0));
    let mut months = care_months(Owner::Spouse, 4);
    months[0].wages = 50000;
    set_care_months(&mut d, months);
    assert_eq!(d.credits.as_ref().unwrap().care.rule(&d), Some("CARE_STUDENT_NEEDS_FIVE_MONTHS"));
    assert!(!d.valid_stored());
}

#[test]
fn care_self_care_exception_uses_only_the_qualifying_months() {
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().incapable_self_care = true;
    let mut months = care_months(Owner::Spouse, 0);
    for m in &mut months {
        m.incapable = m.month <= 11;
    }
    d.credits.as_mut().unwrap().care = FederalCare { prior_year: None, ..FederalCare::default() };
    set_care_months(&mut d, months);
    if let CareFilerSituation::Incapable { care_paid, .. } =
        &mut d.credits.as_mut().unwrap().care.filers[0].situation
    {
        *care_paid = 330000;
    }
    document_care(&mut d);
    let r = supported(&d);
    assert_eq!(value(&r, "care_earned_spouse"), 2750);
    assert_eq!(value(&r, "care_credit"), 963);
    assert_eq!(value(&r, "agi"), 100000);
}
#[test]
fn prior_care_alone_does_not_require_current_student_months() {
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().full_time_student = true;
    d.credits.as_mut().unwrap().care =
        FederalCare { prior_year: Some(prior_care_facts()), ..FederalCare::default() };
    document_care(&mut d);
    assert_eq!(value(&supported(&d), "care_credit"), 132);
}

#[test]
fn birth_adoption_withdrawals_share_parent_child_limit_and_one_year_window() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, birth_withdrawal(Owner::Taxpayer, "Child A", 4000, 1000));
    add(&mut d, birth_withdrawal(Owner::Taxpayer, "Child A", 4000, 1000));
    let r = supported(&d);
    assert_eq!(value(&r, "early_distribution_tax"), 400);
    assert_eq!(value(&r, "birth_adoption_used_0_Child A"), 5000);
    assert_eq!(value(&r, "agi"), 68000);
    assert_eq!(restart(d.clone()).draft, d);
    {
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
        let mut data = edit_retirement(data);
        let Some(RetirementTreatment::Exception { distributed, .. }) =
            &mut data.ordinary_distribution
        else {
            unreachable!()
        };
        *distributed = 20260301;
    }
    unsupported(&d, "Income");
}

// Family unit (2026-10-08), defect 3. Care costs entered for a person who is
// not a Form 2441 qualifying person (IRC section 21(b)(1); 2026 Form 2441
// instructions draft "Qualifying Person(s)": a dependent child under age 13, or
// a spouse or dependent who can't care for themselves and lived with the
// filer over half the year) used to refuse the whole estimate. Contract
// decision 2: the person's costs are left out with one WARN_CARE_COSTS_LEFT_OUT
// notice in the person's own words, and the estimate proceeds.
// Fact pattern: a single filer with $60,000 of wages (35% rate) and two
// children, one born in 2012 (age 14, not incapable of self care) with $6,000
// of care costs and one born in 2018 with none. Before: refused. After: the
// 2012 child's costs are left out, so Form 2441 has one qualifying person and
// no expenses, and the credit is $0; with $3,000 of costs for the 2018 child
// too the credit is 35% of the $3,000 one-person ceiling = $1,050 (the left-out
// costs do not raise it to the two-person $6,000 ceiling; the 2012 child is
// not a qualifying person).
#[test]
fn care_costs_for_a_non_qualifying_person_are_left_out_with_one_notice() {
    let mut d = complete(FederalStatus::Single);
    add_child(&mut d);
    add_child(&mut d);
    d.dependents[0].birth_date = 20120101;
    d.dependents[0].first_name = "Older".into();
    d.dependents[0].last_name = "Child".into();
    d.dependents[0].set_care_expenses(600_000);
    let r = {
        document_care(&mut d);
        supported(&d)
    };
    assert_eq!(value(&r, "care_credit"), 0);
    let notes: Vec<_> =
        r.review_notes.iter().filter(|n| n.display.key == "WARN_CARE_COSTS_LEFT_OUT").collect();
    assert_eq!(notes.len(), 1, "{r:?}");
    assert_eq!(
        notes[0].display.args,
        vec![crate::MessageArg::Encoded(crate::interview::person_name("Older", "Child"))]
    );
    d.dependents[1].set_care_expenses(300_000);
    let r = {
        document_care(&mut d);
        supported(&d)
    };
    assert_eq!(value(&r, "care_credit"), 1050);
    assert!(noted(&r, "WARN_CARE_COSTS_LEFT_OUT"), "{r:?}");
    // Control: only qualifying people's costs, and no notice.
    d.dependents[0].set_care_expenses(0);
    let r = {
        document_care(&mut d);
        supported(&d)
    };
    assert_eq!(value(&r, "care_credit"), 1050);
    assert!(!noted(&r, "WARN_CARE_COSTS_LEFT_OUT"), "{r:?}");
}
// The costs of a person who is not a qualifying person stay out whatever else
// is confirmed: an age or relationship that fails the test is not overridden
// by the answers, except by incapability of self care (Form 2441 qualifying
// person 3), which makes the person qualify again.
#[test]
fn care_expense_confirmation_cannot_override_age_or_qualifying_child_test() {
    let mut d = complete(FederalStatus::Single);
    add_child(&mut d);
    add_child(&mut d);
    d.dependents[0].birth_date = 20120101;
    d.dependents[0].set_care_expenses(600000);
    let r = {
        document_care(&mut d);
        supported(&d)
    };
    assert!(noted(&r, "WARN_CARE_COSTS_LEFT_OUT"), "{r:?}");
    d.dependents[0].incapable_self_care = true;
    let r = {
        document_care(&mut d);
        supported(&d)
    };
    assert_eq!(value(&r, "care_credit"), 2100);
    assert!(!noted(&r, "WARN_CARE_COSTS_LEFT_OUT"), "{r:?}");
    d.dependents[0].incapable_self_care = false;
    d.dependents[0].birth_date = 20130615;
    // Turns 13 during 2026; confirmed earlier eligible expenses still count.
    assert_eq!(
        value(
            &{
                document_care(&mut d);
                supported(&d)
            },
            "care_credit"
        ),
        2100
    );
    d.dependents[0].birth_date = 20180615;
    d.dependents[0].relationship = Relationship::Unrelated;
    assert!(noted(
        &{
            document_care(&mut d);
            supported(&d)
        },
        "WARN_CARE_COSTS_LEFT_OUT"
    ));
    d.dependents[0].incapable_self_care = true;
    assert_eq!(
        value(
            &{
                document_care(&mut d);
                supported(&d)
            },
            "care_credit"
        ),
        2100
    );
}

#[test]
fn eic_compares_earned_credit_with_phased_maximum_instead_of_reducing_twice() {
    assert_eq!(eic_amount(5000, 15000, 0, false), 345);
    assert_eq!(eic_amount(1000, 15000, 0, false), 78);
    assert_eq!(eic_amount(5000, 25000, 1, false), 1709);
    assert_eq!(eic_amount(5000, 45000, 1, false), 1050);
    assert_eq!(eic_amount(0, 15000, 0, false), 0);
    assert_eq!(eic_amount(-5000, 15000, 0, false), 0);
}

#[test]
fn another_taxpayers_eic_child_disallows_credit_without_blocking_return() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 1000000, 0);
    d.people.as_mut().unwrap().taxpayer.disabled = true;
    assert_eq!(value(&supported(&d), "earned_income_credit"), 664);
    d.people.as_mut().unwrap().taxpayer.another_eitc_child = true;
    let r = supported(&d);
    assert_eq!(value(&r, "earned_income_credit"), 0);
    assert_eq!(value(&r, "agi"), 10000);
    assert_eq!(restart(d.clone()).draft, d);
    add_child(&mut d);
    assert_eq!(value(&supported(&d), "earned_income_credit"), 0);
}

#[test]
fn childless_eic_return_matches_hand_computation() {
    // Single filer born 1996-06-15 (age 30 in 2026, inside the childless
    // 25-64 EIC band; `FederalPerson::childless_eic_age()`), unmarried, not a
    // dependent, no children. One W-2: box 1 wages $14,000.00, box 2 federal
    // withholding $300.00, boxes 3-6 consistent (`wage()`'s standard formula:
    // box3 = box1, box4 = 6.2% of box1 = $868.00, box5 = box1,
    // box6 = 1.45% of box1 = $203.00). One 1099-INT box 1 $200.00.
    //
    // Form 1040:
    //   Line 1a wages = $14,000.00 (the single W-2 box 1).
    //   Line 2b taxable interest = $200.00 (the single 1099-INT box 1).
    //   Line 9 total income = $14,000.00 + $200.00 = $14,200.00.
    //   Line 10 adjustments = $0.00 (none entered).
    //   Line 11 AGI = $14,200.00.
    //   Line 12e standard deduction: Rev. Proc. 2025-32 Table 8 2026 single
    //     base is $16,100.00 (`standard_base()`, federal_rules.rs); not a
    //     dependent, not 65 (born 1996), not blind, so no addition.
    //   Line 15 taxable income = max($14,200.00 - $16,100.00, 0) = $0.00.
    //   Line 16 tax on $0.00 taxable income = $0.00.
    //   Line 22/23 = $0.00 (no other taxes: wages and AGI are far below every
    //     Additional Medicare/NIIT threshold, and there is no SE income).
    //   Line 24a total tax = $0.00.
    //   Schedule EIC (forms/schedule_eic.rs; tests/fixtures/irs-eic-2026-draft.csv):
    //     Step 1-4 gate passes (valid SSN, no Form 2555, not MFS, not another
    //     filer's qualifying child, childless age/US-home/non-dependent all
    //     true). Step 2 investment income = $200.00 interest, far under the
    //     $12,200 Pub 596 Worksheet 1 limit. Step 3 children = 0. Step 5
    //     earned income = $14,000.00 (wages only). Step 6: the childless
    //     ("single_0") column of the draft table --
    //       earned-income lookup, row "14000,14050,422,...": $422 (earned income $14,000).
    //       AGI lookup, row "14200,14250,407,...": $407 (AGI $14,200; AGI reaches the
    //         childless single phase-out start of $10,860, so the AGI lookup applies).
    //     `eic_amount()` takes the smaller of the two: min($422, $407) = $407.00.
    //   Line 27a earned income credit = $407.00.
    //   Line 25a W-2 withholding = $300.00 (box 2; no other withholding source).
    //   Line 28 additional child tax credit = $0.00 (no children).
    //   Lines 26, 29-31 = $0.00 (no estimated payments, education, adoption or
    //     other payments).
    //   Line 33 total payments = $300.00 (25d) + $407.00 (27a) = $707.00.
    //   Line 34 overpayment (refund) = $707.00 - $0.00 (line 24a) = $707.00.
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19960615;
    d.income[0] = wage(1, Owner::Taxpayer, 1_400_000, 30_000);
    add(&mut d, income(savings_income(200, 0)));
    let r = supported(&d);
    assert_eq!(return_line(&r, "1a"), 1_400_000);
    assert_eq!(return_line(&r, "2b"), 20_000);
    assert_eq!(return_line(&r, "9"), 1_420_000);
    assert_eq!(return_line(&r, "11a / 11b"), 1_420_000);
    assert_eq!(return_line(&r, "12e"), 1_610_000);
    assert_eq!(return_line(&r, "15"), 0);
    assert_eq!(return_line(&r, "16"), 0);
    assert_eq!(return_line(&r, "24a"), 0);
    assert_eq!(return_line(&r, "25a"), 30_000);
    assert_eq!(return_line(&r, "27a"), 40_700);
    assert_eq!(return_line(&r, "33"), 70_700);
    assert_eq!(return_line(&r, "34"), 70_700);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn eic_investment_income_limit_is_12200() {
    // Pub 596 Worksheet 1's investment-income limit, as the App records it
    // for the 2026 draft (schedule_eic.rs `compute()`: `investment <= 12200`).
    // Single filer (default `complete()`/`person()` birth date 1980-06-15,
    // age 46, inside the childless 25-64 band), $5,000 wages, no children.
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 500_000, 0);
    add(&mut d, income(savings_income(12_200, 0)));
    let r = supported(&d);
    assert_eq!(value(&r, "eic_investment_income"), 12_200);
    assert_eq!(value(&r, "earned_income_credit"), eic_amount(5000, 17200, 0, false));
    assert!(value(&r, "earned_income_credit") > 0);
    assert_eq!(restart(d.clone()).draft, d);

    let mut over = d.clone();
    {
        let FederalIncome::Interest { data } = &mut over.income[1].income else { unreachable!() };
        let mut data = edit_interest(data);
        data.taxable = 1_220_100; // $12,201.00: one cent over the limit.
    }
    let over_result = supported(&over);
    assert_eq!(value(&over_result, "eic_investment_income"), 12_201);
    assert_eq!(value(&over_result, "earned_income_credit"), 0);
    assert_eq!(restart(over.clone()).draft, over);
}

#[test]
fn child_under_six_months_residency_is_not_an_eic_child() {
    // Pub 596/i1040gi Step 3-5: an EIC qualifying child must live with the
    // filer more than half the year (`family()`'s `lived = c.days_in_home()
    // >= 183`, federal_rules.rs). A dependent present only 100 days (under
    // six months) fails that test, so `child` is false and the dependent
    // never reaches `f.eic_children`, even though the same dependent still
    // counts as a (non-EIC) "qualifying relative" dependent for other-
    // dependent purposes (gross income $0 < $530,000, support and residence
    // otherwise satisfied). The filer -- otherwise childless-EIC-eligible by
    // age 30 (born 1996, inside the 25-64 band) -- falls to the childless
    // amount rather than the one-child amount.
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19960615;
    d.income[0] = wage(1, Owner::Taxpayer, 500_000, 0);
    add_child(&mut d);
    d.dependents[0].residency = DependentResidency::Days { days: 100 };
    let r = supported(&d);
    assert_eq!(value(&r, "earned_income_credit"), eic_amount(5000, 5000, 0, false));
    assert_ne!(value(&r, "earned_income_credit"), eic_amount(5000, 5000, 1, false));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn eic_lookup_takes_the_smaller_of_earned_income_and_agi() {
    use crate::forms::schedule_eic::eic_table_2026;
    // One child, not joint. Draft 2026 EIC table (tests/fixtures/irs-eic-2026-draft.csv):
    //   row "20000,20050,0,4427,7316,8231,...": one-child ("single_1") column = $4,427.00
    //     (the earned-income lookup, on $20,000.00 wages).
    //   row "30000,30050,0,3447,6024,6939,...": one-child column = $3,447.00
    //     (the AGI lookup, on AGI $30,000.00: $20,000.00 wages + $10,000.00 interest).
    // AGI exceeds earned income, so `eic_amount()` looks up both and takes the
    // smaller: min($4,427.00, $3,447.00) = $3,447.00.
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 2_000_000, 0);
    add(&mut d, income(savings_income(10_000, 0)));
    add_child(&mut d);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 30_000);
    assert_eq!(eic_table_2026(20000, 1, false), 4427);
    assert_eq!(eic_table_2026(30000, 1, false), 3447);
    assert_eq!(value(&r, "earned_income_credit"), 3447);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn dependent_standard_deduction_uses_2026_limits_then_age_and_blindness() {
    use crate::federal_rules::standard_deduction;
    let mut d = complete(FederalStatus::Single);
    let s = d.setup.as_ref().unwrap();
    let p = d.people.as_mut().unwrap();
    p.taxpayer.dependency = FilerDependency::CanBeClaimed { claimed: true };
    for (earned, expected) in [
        (-500, 1350),
        (150, 1350),
        (900, 1350),
        (901, 1351),
        (4000, 4450),
        (15650, 16100),
        (20000, 16100),
    ] {
        assert_eq!(standard_deduction(s, p, earned, false), expected);
    }
    p.taxpayer.blind = true;
    assert_eq!(standard_deduction(s, p, 4000, false), 6500);
    p.taxpayer.birth_date = 19620101;
    assert_eq!(standard_deduction(s, p, 4000, false), 8550);
    p.taxpayer.dependency = FilerDependency::Independent;
    assert_eq!(standard_deduction(s, p, 4000, false), 20200);
}

#[test]
fn dependent_spouse_uses_joint_income_and_married_additional_deduction() {
    use crate::federal_rules::standard_deduction;
    let mut d = complete(FederalStatus::Joint);
    let s = d.setup.as_ref().unwrap();
    let p = d.people.as_mut().unwrap();
    let spouse = p.spouse.as_mut().unwrap();
    spouse.dependency = FilerDependency::CanBeClaimed { claimed: true };
    spouse.blind = true;
    assert_eq!(standard_deduction(s, p, 5000, false), 7100);
    assert_eq!(standard_deduction(s, p, 40000, false), 33850);
    assert_eq!(supported(&d).state, ResultState::EstimatedForSupportedScope);
}

#[test]
fn education_actual_claim_does_not_change_dependent_deduction_or_saver_status() {
    use crate::federal_rules::{saver_potential, standard_deduction, unclaimed_filers};
    let mut d = complete(FederalStatus::Single);
    for (claimed, expected) in [(true, false), (false, true)] {
        d.people.as_mut().unwrap().taxpayer.dependency = FilerDependency::CanBeClaimed { claimed };
        let p = d.people.as_ref().unwrap();
        assert_eq!(unclaimed_filers(p), expected);
        assert_eq!(standard_deduction(d.setup.as_ref().unwrap(), p, 4000, false), 4450);
        assert_eq!(saver_potential(&saver(100000, 0), p, FederalStatus::Single, 4000), Ok(0));
        assert_eq!(restart(d.clone()).draft, d);
    }
    d.people.as_mut().unwrap().taxpayer.dependency = FilerDependency::Independent;
    assert!(unclaimed_filers(d.people.as_ref().unwrap()));
}

#[test]
fn education_actual_claim_checks_either_joint_filer() {
    use crate::federal_rules::unclaimed_filers;
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().dependency =
        FilerDependency::CanBeClaimed { claimed: true };
    assert!(!unclaimed_filers(d.people.as_ref().unwrap()));
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().dependency =
        FilerDependency::CanBeClaimed { claimed: false };
    assert!(unclaimed_filers(d.people.as_ref().unwrap()));
}

#[test]
fn dependent_returns_apply_limited_deduction_and_disallow_childless_eic() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 400000, 0);
    add(&mut d, income(savings_income(1000, 0)));
    d.people.as_mut().unwrap().taxpayer.dependency =
        FilerDependency::CanBeClaimed { claimed: false };
    let r = supported(&d);
    assert_eq!(value(&r, "deduction"), 4450);
    assert_eq!(value(&r, "income_tax"), 56);
    assert_eq!(value(&r, "earned_income_credit"), 0);
    assert_eq!(restart(d.clone()).draft, d);
    // Section 152(b)(1): a filer someone can claim has no dependents, so the
    // listed child gives no child tax credit; the return is computed.
    add_child(&mut d);
    let with_child = supported(&d);
    assert!(noted(&with_child, "NOTE_DEPENDENT_FILER_HAS_NO_DEPENDENTS"));
    assert_eq!(value(&with_child, "child_credit"), 0);
    assert_eq!(value(&with_child, "income_tax"), 56);
}

#[test]
fn dependent_student_loan_uses_actual_claim_not_claimability() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 2000000, 0);
    d.adjustments.as_mut().unwrap().set_student_loan_interest(150000);
    for (claimed, expected_agi) in [(true, 20000), (false, 18500)] {
        d.people.as_mut().unwrap().taxpayer.dependency = FilerDependency::CanBeClaimed { claimed };
        assert_eq!(value(&supported(&d), "agi"), expected_agi);
    }
}

#[test]
fn named_scholarship_service_programs_preserve_only_qualified_expense_exclusion() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(scholarship(10000, 3000, 7000)));
    for service in [
        ScholarshipService::NationalHealthServiceCorps,
        ScholarshipService::ArmedForcesHealthProfessions,
        ScholarshipService::WorkCollege,
    ] {
        let FederalIncome::Scholarship {
            data:
                Scholarship::Education {
                    data: crate::EducationAward { services, degree_candidate, .. },
                },
        } = &mut d.income[0].income
        else {
            panic!()
        };
        *services = service;
        *degree_candidate = true;
        assert_eq!(value(&supported(&d), "taxable_scholarships"), 3000);
        assert_eq!(restart(d.clone()).draft, d);
        let FederalIncome::Scholarship {
            data: Scholarship::Education { data: crate::EducationAward { degree_candidate, .. } },
        } = &mut d.income[0].income
        else {
            panic!()
        };
        *degree_candidate = false;
        assert_eq!(value(&supported(&d), "taxable_scholarships"), 10000);
    }
    // Services of another kind are no longer refused: the part of the award
    // that pays for them is taxable (Pub. 970 Worksheet 1-1 line 2).
    let FederalIncome::Scholarship {
        data:
            Scholarship::Education {
                data: crate::EducationAward { services, qualified_paid, degree_candidate, .. },
            },
    } = &mut d.income[0].income
    else {
        panic!()
    };
    *services = ScholarshipService::PaidServices { amount: 200_000 };
    *qualified_paid = 500_000;
    *degree_candidate = true;
    assert_eq!(value(&supported(&d), "taxable_scholarships"), 5000);
}
#[test]
fn main_home_exclusion_uses_closing_costs_and_each_spouses_history() {
    for (status, excluded, taxable) in
        [(FederalStatus::Single, 250000, 300000), (FederalStatus::Joint, 500000, 50000)]
    {
        let mut d = complete(status);
        add(
            &mut d,
            income(FederalIncome::HomeSale {
                data: home_sale_source(status == FederalStatus::Joint),
            }),
        );
        let r = supported(&d);
        let id = d.income[1].id;
        assert_eq!(value(&r, &alloc::format!("home_{id}_realized_gain")), 550000);
        assert_eq!(value(&r, &alloc::format!("home_{id}_excluded_gain")), excluded);
        assert_eq!(value(&r, "capital_gain"), taxable);
        assert_eq!(
            value(&r, "agi"),
            if status == FederalStatus::Joint { 100000 } else { 60000 } + taxable
        );
        assert_eq!(restart(d.clone()).draft, d);
    }
}
#[test]
fn joint_home_exclusion_can_use_one_spouses_ownership_but_not_the_others_residence() {
    let mut d = complete(FederalStatus::Joint);
    let mut home = home_sale_source(true);
    let HomeSpouse::Joint { lived, .. } = &mut home.facts_mut().spouse else { panic!() };
    *lived = HomeLived::Days { days: 729 };
    add(&mut d, income(FederalIncome::HomeSale { data: home }));
    assert_eq!(value(&supported(&d), "capital_gain"), 300000);
    let FederalIncome::HomeSale { data } = &mut d.income[1].income else { panic!() };
    data.facts_mut().earlier_sale = EarlierHomeSale::Yes { sold: 20251231 };
    unsupported(&d, "Income:");
}
#[test]
fn home_care_facility_days_complete_residence_but_not_ownership() {
    let mut d = complete(FederalStatus::Single);
    let mut home = home_sale_source(false);
    home.facts_mut().lived =
        HomeLived::CareFacility { home_days: 365, facility_days: 365, owned_residence_days: 365 };
    assert!(HomeSale::from(&home).valid());
    add(&mut d, income(FederalIncome::HomeSale { data: home }));
    assert_eq!(value(&supported(&d), "capital_gain"), 300000);
    assert_eq!(restart(d.clone()).draft, d);
    let FederalIncome::HomeSale { data } = &mut d.income[1].income else { panic!() };
    let HomeLived::CareFacility { owned_residence_days, .. } = &mut data.facts_mut().lived else {
        panic!()
    };
    *owned_residence_days = 364;
    unsupported(&d, "Income:");
}
#[test]
fn home_care_requires_a_year_at_home_and_excludes_unowned_or_overlapping_days() {
    let mut home = home_sale(false);
    let HomeExclusion::Full { taxpayer, .. } = &mut home.exclusion else { panic!() };
    *taxpayer = home_use(730, 365);
    taxpayer.care = home_care(365, 365);
    assert!(home.excluded(FederalStatus::Single).is_ok());
    let HomeExclusion::Full { taxpayer, .. } = &mut home.exclusion else { panic!() };
    taxpayer.care = home_care(365, 364);
    assert!(home.excluded(FederalStatus::Single).is_err());
    let HomeExclusion::Full { taxpayer, .. } = &mut home.exclusion else { panic!() };
    taxpayer.care = home_care(366, 365);
    assert!(!home.valid());
}

#[test]
fn retained_2013_through_2016_pease_refund_uses_year_and_filing_status() {
    for (year, filing_status, threshold) in [
        (2013, PeaseFilingStatus::Single, 25_000_000),
        (2014, PeaseFilingStatus::HeadOfHousehold, 27_965_000),
        (2015, PeaseFilingStatus::JointOrSurviving, 30_990_000),
        (2016, PeaseFilingStatus::MarriedSeparate, 15_565_000),
    ] {
        let mut d = complete(FederalStatus::Single);
        add(
            &mut d,
            income(FederalIncome::GovernmentPayment {
                data: tax_refund_source(FederalTaxRefund {
                    amount: 300_000,
                    withholding: 0,
                    history: TaxRefundHistory::IncomeTax {
                        deduction_year: year,
                        income_paid: 1_000_000,
                        sales_available: 500_000,
                        salt_paid: 1_000_000,
                        salt_deducted: 1_000_000,
                        itemized: 1_970_000,
                        standard: 600_000,
                        overall_limit: Some(PeaseWorksheet {
                            agi: threshold + 1_000_000,
                            before_limit: 2_000_000,
                            protected: 1_000_000,
                            filing_status,
                        }),
                        property_refunded: 0,
                        ordinary: true,
                    },
                }),
            }),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "taxable_state_tax_refund"), 3_000, "{year}");
        assert_eq!(estimate(&restart(d.clone()).draft), r, "{year}");
        let FederalIncome::GovernmentPayment {
            data:
                GovernmentPaymentSource {
                    history: TaxRefundHistory::IncomeTax { overall_limit, .. },
                    ..
                },
        } = &mut d.income[1].income
        else {
            unreachable!()
        };
        overall_limit.as_mut().unwrap().agi += 100_000;
        assert!(!d.valid_stored(), "{year}");
    }
}
#[test]
fn social_security_negative_box_five_offsets_joint_spouse_benefits() {
    let mut d = complete(FederalStatus::Joint);
    add(&mut d, income(social_security_net(-1000000, 10000)));
    let mut spouse = income(social_security_net(2000000, 50000));
    spouse.owner = crate::DocumentOwner::Spouses;
    add(&mut d, spouse);
    let r = supported(&d);
    assert_eq!(return_line(&r, "6a"), 1000000);
    assert_eq!(return_line(&r, "6b"), 850000);
    assert_eq!(value(&r, "agi"), 108500);
    assert_eq!(value(&r, "social_security_net_repayments"), 0);
    let mut equivalent = complete(FederalStatus::Joint);
    add(&mut equivalent, income(social_security_net(1000000, 60000)));
    let expected = supported(&equivalent);
    assert_eq!(r.tax, expected.tax);
    assert_eq!(r.refund, expected.refund);
    d.income.swap(1, 2);
    assert_eq!(supported(&d).tax, expected.tax);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn eic_combines_personal_property_rent_and_royalties_before_zero_floor() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 1000000, 0);
    add_child(&mut d);
    add(&mut d, income(personal_rent(18200, 2000)));
    add(&mut d, income(royalty(1000, 5000)));
    // Pub. 596 worksheet 1: $16,200 rental profit less $4,000 royalty loss.
    assert!(value(&supported(&d), "earned_income_credit") > 0);
    d.income[1].income = personal_rent(18201, 2000);
    assert_eq!(value(&supported(&d), "earned_income_credit"), 0);
}
#[test]
fn joint_general_rule_matches_irs_different_survivor_payment_example() {
    let a = joint_annuity(false);
    assert!(a.valid(600000));
    assert_eq!(a.expected(), 12120000);
    assert_eq!(a.ratio(), 517);
    assert_eq!(a.excluded(), 310200);
    let survivor = joint_annuity(true);
    assert!(survivor.valid(420000));
    assert_eq!(survivor.expected(), a.expected());
    assert_eq!(survivor.ratio(), a.ratio());
    assert_eq!(survivor.excluded(), 217140);
    let mut d = complete(FederalStatus::Single);
    let mut e = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Nonqualified,
        4200,
        RetirementTreatment::Ordinary,
    );
    if let FederalIncome::Retirement { data } = &mut e.income {
        let mut data = edit_retirement(data);
        data.taxable = Some(RetirementTaxable::GeneralAnnuity { data: survivor });
        assert_eq!(data.taxable_income(), 202860);
    }
    add(&mut d, e);
    assert_eq!(return_line(&supported(&d), "5b"), 202900);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn joint_survivor_recovery_shares_original_cost_and_validates_recipient_payments() {
    let mut a = joint_annuity(true);
    a.recovered = a.cost - 10000;
    assert_eq!(a.excluded(), 10000);
    assert!(!a.valid(600000)); // Survivor's payment is $350, not the primary's $500.
}

#[test]
fn final_filer_age_uses_death_date_and_distinct_eic_boundaries() {
    let mut d = complete(FederalStatus::Single);
    let p = &mut d.people.as_mut().unwrap().taxpayer;
    p.birth_date = 19610615;
    p.died = death(20260613);
    assert!(p.valid_dates());
    assert!(!p.senior());
    assert!(p.childless_eic_age());
    p.died = death(20260614);
    assert!(p.senior()); // Standard deduction: day before age 65.
    assert!(p.childless_eic_age()); // EIC: still under age 65 until birthday.
    p.died = death(20260615);
    assert!(!p.childless_eic_age());
    p.birth_date = 20010615;
    p.died = death(20260613);
    assert!(!p.childless_eic_age());
    p.died = death(20260614);
    assert!(p.childless_eic_age());
}
#[test]
fn a_final_joint_return_with_one_or_both_deaths_is_an_ordinary_married_return() {
    // Section 6013(a)(2): the joint return of the year a spouse died uses the
    // ordinary married situation; nothing is refused for the death, and no
    // separate widowed situation exists.
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().died =
        Some(FinalReturn { bonds: None, date: 20260615, ordinary: true, estate_medical: 0 });
    supported(&d);
    d.people.as_mut().unwrap().taxpayer.died =
        Some(FinalReturn { bonds: None, date: 20261215, ordinary: true, estate_medical: 0 });
    supported(&d);
}

#[test]
fn final_return_allows_childless_eic_and_young_wage_filers() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.died =
        Some(FinalReturn { bonds: None, date: 20260615, ordinary: true, estate_medical: 0 });
    d.income[0] = wage(1, Owner::Taxpayer, 1000000, 0);
    assert!(return_line(&supported(&d), "27a") > 0);
    d.income[0] = wage(1, Owner::Taxpayer, 6000000, 0);
    d.people.as_mut().unwrap().taxpayer.birth_date = 20050615;
    supported(&d);
}
#[test]
fn final_childless_eic_uses_full_credit_and_actual_death_age() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 1000000, 0);
    let full = return_line(&supported(&d), "27a");
    assert!(full > 0);
    let p = &mut d.people.as_mut().unwrap().taxpayer;
    p.birth_date = 19610615;
    p.died = Some(FinalReturn { date: 20260614, ordinary: true, bonds: None, estate_medical: 0 });
    assert_eq!(return_line(&supported(&d), "27a"), full); // Not prorated, under 65 at death.
    d.people.as_mut().unwrap().taxpayer.died.as_mut().unwrap().date = 20260615;
    assert_eq!(return_line(&supported(&d), "27a"), 0);
    d.people.as_mut().unwrap().taxpayer.died.as_mut().unwrap().date = 20260614;
    d.people.as_mut().unwrap().taxpayer.us_home_over_half_year = false;
    assert_eq!(return_line(&supported(&d), "27a"), 0);
}
#[test]
fn final_joint_childless_eic_can_use_deceased_spouses_age() {
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 1000000, 0);
    d.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::Together };
    let p = d.people.as_mut().unwrap();
    p.taxpayer.birth_date = 19590101;
    let spouse = p.spouse.as_mut().unwrap();
    spouse.birth_date = 19611101;
    spouse.died =
        Some(FinalReturn { date: 20260801, ordinary: true, bonds: None, estate_medical: 0 });
    assert!(return_line(&supported(&d), "27a") > 0);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().us_home_over_half_year = false;
    assert_eq!(return_line(&supported(&d), "27a"), 0);
}

#[test]
fn final_education_refund_restriction_uses_age_at_tax_year_close() {
    let mut d = complete(FederalStatus::Single);
    let p = &mut d.people.as_mut().unwrap().taxpayer;
    p.birth_date = 20020615;
    p.full_time_student = true;
    p.support_share = SupportShare::LessThanHalf;
    p.died = Some(FinalReturn { date: 20260613, ordinary: true, bonds: None, estate_medical: 0 });
    let sid0_3795 = -1; // You, the first person of the catalog
    add_education(
        d.credits.as_mut().unwrap(),
        native_education(
            sid0_3795,
            education_expenses(400000),
            education_choice(EducationMethod::AmericanOpportunity, Answer::Yes),
        ),
    );
    assert_eq!(value(&supported(&d), "refundable_education_credit"), 0);
    d.people.as_mut().unwrap().taxpayer.died.as_mut().unwrap().date = 20260614;
    assert_eq!(value(&supported(&d), "refundable_education_credit"), 1000);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn kiddie_parent_rate_flows_to_return_and_saved_sources() {
    let d = young_interest_return();
    let r = supported(&d);
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 7300);
    assert_eq!(value(&r, "kiddie_tax_parent_share"), 1606);
    assert_eq!(value(&r, "regular_income_tax"), 1742);
    assert_eq!(return_line(&r, "16"), 174200);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn kiddie_siblings_share_bracket_crossing_and_child_tax_is_floor() {
    let mut parent = ordinary_parent();
    parent.status = FederalStatus::Single;
    parent.taxable = 5000000;
    parent.others.push(OtherChildTax { gains: None, label: "Sibling".into(), net: 300000 });
    // The family worksheet uses Tax Table values for each ordinary-tax lookup.
    assert_eq!(parent.calculate(1000, 2350, FederalStatus::Single, [0, 0], 0, [0; 3]), (211, 347));
    parent.taxable = 0;
    parent.others.clear();
    let own = crate::federal_rules::preferential_tax(100000, 0, FederalStatus::Single);
    assert_eq!(parent.calculate(1000, 100000, FederalStatus::Single, [0, 0], 0, [0; 3]).1, own);
    parent.taxable = 100000000000;
    parent.others = (0..100)
        .map(|_| OtherChildTax { gains: None, label: "Sibling".into(), net: 100000000000 })
        .collect();
    assert!(
        parent.calculate(1000000000, 1000000000, FederalStatus::Single, [0, 0], 0, [0; 3]).1 > 0
    );
}
#[test]
fn kiddie_missing_or_special_sources_do_not_publish_estimates() {
    let mut d = young_interest_return();
    d.people.as_mut().unwrap().parent_tax = None;
    unsupported(&d, "Investment tax");
    d.people.as_mut().unwrap().parent_tax = Some(ordinary_parent());
    d.people.as_mut().unwrap().parent_tax.as_mut().unwrap().ordinary = false;
    unsupported(&d, "Investment tax");
    let parent = d.people.as_mut().unwrap().parent_tax.as_mut().unwrap();
    parent.ordinary = true;
    parent.complete = false;
    unsupported(&d, "Investment tax");
}
#[test]
fn kiddie_savings_penalty_reduces_unearned_income_before_parent_allocation() {
    let mut d = young_interest_return();
    d.income[0].income = savings_income(10000, 1000);
    let r = supported(&d);
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 6300);
    assert_eq!(value(&r, "regular_income_tax"), 1522);
    d.people.as_mut().unwrap().parent_tax = None;
    d.income[0].income = savings_income(10000, 7300);
    supported(&d);
}

#[test]
fn kiddie_parent_dividends_shift_capital_gain_brackets() {
    let mut d = young_interest_return();
    let parent = d.people.as_mut().unwrap().parent_tax.as_mut().unwrap();
    parent.status = FederalStatus::Single;
    parent.taxable = 5000000;
    parent.gains = Some(ParentGains {
        investment_election: 0,
        collectibles: 0,
        unrecaptured: 0,
        qualified: 4000000,
        long_term: 0,
        combined: 0,
    });
    let r = supported(&d);
    // Ordinary income rises $10,000 -> $17,300: $240 + $588.
    // The same $40,000 dividends incur $1,095 more capital-gain tax.
    assert_eq!(value(&r, "kiddie_tax_parent_share"), 1923);
    assert_eq!(value(&r, "regular_income_tax"), 2059);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn kiddie_parent_capital_losses_reduce_preferential_income() {
    let mut p = ordinary_parent();
    p.status = FederalStatus::Single;
    p.taxable = 5000000;
    p.gains = Some(ParentGains {
        investment_election: 0,
        collectibles: 0,
        unrecaptured: 0,
        qualified: 0,
        long_term: 5000000,
        combined: 4000000,
    });
    assert_eq!(p.calculate(7300, 8650, FederalStatus::Single, [0, 0], 0, [0; 3]), (1923, 2059));
    p.gains.as_mut().unwrap().combined = -100000;
    let zero_gain = p.calculate(7300, 8650, FederalStatus::Single, [0, 0], 0, [0; 3]);
    p.gains = None;
    assert_eq!(p.calculate(7300, 8650, FederalStatus::Single, [0, 0], 0, [0; 3]), zero_gain);
}

#[test]
fn kiddie_sibling_allocated_gains_keep_preferential_rates() {
    let mut p = ordinary_parent();
    p.status = FederalStatus::Single;
    p.taxable = 5000000;
    p.others.push(OtherChildTax {
        label: "Sibling".into(),
        net: 300000,
        gains: Some(ChildGains {
            investment_election: 0,
            qualified: 100000,
            capital: 200000,
            special: None,
        }),
    });
    // One-quarter of the parent increase rounds to $158; the child's
    // remaining ordinary income follows the Tax Table.
    assert_eq!(p.calculate(1000, 2350, FederalStatus::Single, [0, 0], 0, [0; 3]), (158, 294));
    assert!(p.valid());
    let mut d = young_interest_return();
    d.people.as_mut().unwrap().parent_tax = Some(p.clone());
    supported(&d);
    assert_eq!(restart(d.clone()).draft, d);
    p.others[0].gains.as_mut().unwrap().capital += 1;
    assert!(!p.valid());
    p.others[0].net = 100;
    p.others[0].gains =
        Some(ChildGains { investment_election: 0, qualified: 50, capital: 50, special: None });
    assert!(!p.valid()); // Separately rounded source lines would exceed line 5.
}

#[test]
fn kiddie_standard_deduction_allocates_dividends_and_gains() {
    use crate::kiddie::gain_allocation;
    assert_eq!(gain_allocation(7300, 10000, 10000, 1350, 6000, 4000, [0; 2]), [4380, 2920]);
    assert_eq!(gain_allocation(3900, 20000, 20000, 16100, 12000, 8000, [0; 2]), [2340, 1560]);
    assert_eq!(gain_allocation(7300, 10000, 10000, 1350, 5000, 0, [0; 2]), [3650, 0]);
    assert_eq!(gain_allocation(0, 10000, 10000, 16100, 5000, 0, [0; 2]), [0, 0]);
}
#[test]
fn kiddie_child_preferential_income_uses_parent_capital_gain_rates() {
    let p = ordinary_parent();
    // All $7,300 allocated income is preferential at the parent's 15% rate.
    // The child's remaining $1,350 is also preferential, taxed at zero.
    assert_eq!(
        p.calculate(7300, 8650, FederalStatus::Single, [10000, 7300], 0, [0; 3]),
        (1095, 1095)
    );
}

#[test]
fn kiddie_child_gain_worksheets_flow_to_return_and_persist() {
    let mut d = young_interest_return();
    d.income[0].income =
        FederalIncome::Dividend { data: dividend_source(native_dividend(600000, 600000, 400000)) };
    let r = supported(&d);
    assert_eq!(value(&r, "kiddie_tax_allocated_dividends"), 4380);
    assert_eq!(value(&r, "kiddie_tax_allocated_capital_gain"), 2920);
    assert_eq!(return_line(&r, "16"), 109500);
    assert_eq!(restart(d.clone()).draft, d);
    d.people.as_mut().unwrap().taxpayer.dependency = FilerDependency::Independent;
    if let FederalIncome::Dividend { data } = &mut d.income[0].income {
        let mut data = edit_dividend(data);
        data.ordinary *= 2;
        data.qualified *= 2;
        data.capital_distributions *= 2;
    }
    let r = supported(&d);
    assert_eq!(value(&r, "kiddie_tax_allocated_dividends"), 2340);
    assert_eq!(value(&r, "kiddie_tax_allocated_capital_gain"), 1560);
    assert_eq!(return_line(&r, "16"), 58500);
}

#[test]
fn kiddie_personal_service_profit_and_loss_preserve_unearned_income() {
    let mut d = young_interest_return();
    add(&mut d, income(cash_business(10000)));
    let r = supported(&d);
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 7300);
    assert_eq!(value(&r, "kiddie_tax_parent_share"), 1606);
    assert_eq!(value(&r, "self_employment_tax"), 1413);
    assert_eq!(restart(d.clone()).draft, d);
    d.income[1].income = cash_business(-1000);
    let r = supported(&d);
    // No wages or service receipts: the IRS directs line 1 to AGI.
    assert_eq!(value(&r, "agi"), 9000);
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 6300);
    assert_eq!(value(&r, "regular_income_tax"), 1522);
}
#[test]
fn kiddie_capital_business_requires_allocation_without_investment_receipts() {
    let mut d = young_interest_return();
    d.income[0].income = cash_business(20000);
    d.people.as_mut().unwrap().parent_tax = None;
    supported(&d);
    {
        let answer = BusinessIncomeSource::Capital;
        let FederalIncome::Business { data } = &mut d.income[0].income else { panic!() };
        data.income_source = answer;
        unsupported(&d, "Investment tax");
    }
    d.people.as_mut().unwrap().taxpayer.birth_date = 19800615;
    supported(&d); // This source distinction does not block an adult's ordinary return.
}

// T2-src: "My work is worth at least 30% of the net profit" needs no amount.
// i8615 "Earned income": for a business where both services and capital are
// material, earned income is a reasonable allowance for services, but not
// more than 30% of the net profit after the half-SE deduction; the 30% limit
// does not apply when there is no net profit.
#[test]
fn thirty_percent_answer_is_thirty_percent_of_profit_after_half_se() {
    let source = BusinessIncomeSource::MixedThirtyPercent;
    // 30% x (10,000 - 707) = 2,787.90 -> 2,788.
    assert_eq!(source.earned(10000, 707), Some(2788));
    // Boundary: $1 of profit has no SE tax; 30% x 1 = 0.30 -> 0.
    assert_eq!(source.earned(1, 0), Some(0));
    // Boundary: no net profit gives no amount.
    assert_eq!(source.earned(0, 0), None);
    assert_eq!(source.earned(-1000, 0), None);
    // Without net profit Form 8615 takes the source as capital (assumed).
    assert_eq!(source.for_8615(false, 0), (&BusinessIncomeSource::Capital, true));
    assert_eq!(source.for_8615(false, 1), (&source, false));
    let mixed = BusinessIncomeSource::Mixed { compensation: 600000 };
    assert_eq!(mixed.for_8615(false, -1000), (&mixed, false));
    // Not sure, and a farm's unasked Services, are taken as capital too.
    assert_eq!(
        BusinessIncomeSource::Services.for_8615(true, 1000),
        (&BusinessIncomeSource::Capital, true)
    );
    assert_eq!(
        BusinessIncomeSource::Services.for_8615(false, 1000),
        (&BusinessIncomeSource::Services, false)
    );
}

#[test]
fn thirty_percent_answer_without_net_profit_is_taken_as_capital() {
    // A $1,000 Schedule C loss: no net profit, so no 30% ceiling and the
    // answer gives no amount (i8615 "Earned income"). The compensation is
    // not known, so the business is taken as capital, with a warning.
    let mut d = young_interest_return();
    d.people.as_mut().unwrap().taxpayer.dependency = FilerDependency::Independent;
    let mut source = cash_business(-1000);
    let FederalIncome::Business { data } = &mut source else { panic!() };
    data.income_source = BusinessIncomeSource::MixedThirtyPercent;
    add(&mut d, FederalIncomeEntry { label: "Lawn care".into(), ..income(source) });
    let r = supported(&d);
    assert!(noted(&r, "WARN_KIDDIE_SOURCE_ASSUMED_CAPITAL"), "{r:?}");
    assert_eq!(value(&r, &alloc::format!("business_{}_kiddie_earned_income", d.income[1].id)), 0);
    // Boundary: exactly zero profit is still no net profit.
    d.income[1].income = cash_business(0);
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.income_source = BusinessIncomeSource::MixedThirtyPercent;
    assert!(noted(&supported(&d), "WARN_KIDDIE_SOURCE_ASSUMED_CAPITAL"));
    // Entering the amount instead is accepted: with the loss the $6,000
    // compensation is the earned income, with no 30% ceiling.
    d.income[1].income = cash_business(-1000);
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.income_source = BusinessIncomeSource::Mixed { compensation: 600000 };
    let r = supported(&d);
    assert_eq!(
        value(&r, &alloc::format!("business_{}_kiddie_earned_income", d.income[1].id)),
        6000
    );
    // An adult's return never reads the answer.
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.income_source = BusinessIncomeSource::MixedThirtyPercent;
    d.people.as_mut().unwrap().taxpayer.birth_date = 19800615;
    supported(&d);
}

#[test]
fn dependent_standard_deduction_nets_a_business_loss_against_wages() {
    let mut d = young_interest_return();
    add(&mut d, wage(0, Owner::Taxpayer, 500000, 0));
    add(&mut d, income(cash_business(-2000)));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 13000);
    // Earned income is the wages less the loss (5,000 - 2,000), plus 450.
    assert_eq!(return_line(&r, "12e"), 345000);
    assert_eq!(value(&r, "regular_income_tax"), 1832);
    assert_eq!(restart(d.clone()).draft, d);
    // A loss larger than wages leaves negative earned income, so the $1,350 floor applies.
    d.income[2].income = cash_business(-8000);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 7000);
    assert_eq!(return_line(&r, "12e"), 135000);
    // Taxable income 7,000 - 1,350 now bounds the net unearned income.
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 5650);
}
#[test]
fn dependent_standard_deduction_nets_a_business_loss_against_scholarship() {
    let mut d = young_interest_return();
    add(&mut d, income(scholarship(3000, 3000, 0)));
    add(&mut d, income(cash_business(-2000)));
    let r = supported(&d);
    // Scholarship 3,000 less the loss 2,000 is 1,000 of earned income, plus 450.
    assert_eq!(return_line(&r, "12e"), 145000);
    assert_eq!(value(&r, "earned_income_credit"), 0);
}

#[test]
fn kiddie_parent_special_gains_preserve_rate_ceilings() {
    let mut p = ordinary_parent();
    p.status = FederalStatus::Single;
    p.taxable = 22000000;
    p.gains = Some(ParentGains {
        investment_election: 0,
        qualified: 0,
        long_term: 3000000,
        combined: 3000000,
        collectibles: 3000000,
        unrecaptured: 0,
    });
    // Ordinary income 190,000 -> 197,300, below the 24% ceiling. Total income
    // stays above that ceiling, so the extra dollars shift 28%-rate gain.
    assert_eq!(p.calculate(7300, 8650, FederalStatus::Single, [0, 0], 0, [0; 3]).0, 2044);
    p.gains.as_mut().unwrap().collectibles = 0;
    p.gains.as_mut().unwrap().unrecaptured = 3000000;
    assert_eq!(p.calculate(7300, 8650, FederalStatus::Single, [0, 0], 0, [0; 3]).0, 1825);
    let mut d = young_interest_return();
    d.people.as_mut().unwrap().parent_tax = Some(p);
    supported(&d);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn kiddie_family_special_gain_caps_use_original_parent_source_lines() {
    let mut p = ordinary_parent();
    p.status = FederalStatus::Single;
    p.taxable = 22000000;
    p.gains = Some(ParentGains {
        investment_election: 0,
        qualified: 0,
        long_term: 3000000,
        combined: 1000000,
        collectibles: 0,
        unrecaptured: 3000000,
    });
    // The original parent worksheet caps at $10,000; the combined worksheet
    // caps at $20,000. Do not permanently discard the original line 19 amount.
    assert_eq!(
        p.calculate(10000, 10000, FederalStatus::Single, [10000, 10000], 10000, [0; 3]).0,
        2500
    );
    p.gains.as_mut().unwrap().unrecaptured = 0;
    p.gains.as_mut().unwrap().collectibles = 3000000;
    assert_eq!(
        p.calculate(10000, 10000, FederalStatus::Single, [10000, 10000], 10000, [0; 3]).0,
        2800
    );
}

#[test]
fn kiddie_sibling_special_gain_uses_original_25_percent_ceiling() {
    let mut p = ordinary_parent();
    p.status = FederalStatus::Single;
    p.taxable = 25000000;
    p.others.push(OtherChildTax {
        label: "Sibling".into(),
        net: 1460000,
        gains: Some(ChildGains {
            investment_election: 0,
            qualified: 0,
            capital: 1460000,
            special: Some(ChildSpecialGains {
                net_capital: 2000000,
                collectibles: 1000000,
                unrecaptured: 1000000,
            }),
        }),
    });
    // Family worksheet: ordinary tax increases $320 on this child's $1,000.
    // Allocated special pool = $14,600, original line 19 = $10,000:
    // $2,500 at 25% + $1,288 at 28%. Increase $4,108 * .064 = $263.
    assert_eq!(p.calculate(1000, 2350, FederalStatus::Single, [0, 0], 0, [0; 3]), (263, 399));
    assert!(p.valid());
    let mut d = young_interest_return();
    d.people.as_mut().unwrap().parent_tax = Some(p.clone());
    supported(&d);
    assert_eq!(restart(d.clone()).draft, d);
    let special = p.others[0].gains.as_mut().unwrap().special.as_mut().unwrap();
    special.net_capital = 1000000;
    assert!(!p.valid());
}

#[test]
fn kiddie_sibling_special_gain_allocation_uses_original_net_gain() {
    let mut p = ordinary_parent();
    p.status = FederalStatus::Single;
    p.taxable = 25000000;
    p.others.push(OtherChildTax {
        label: "Sibling".into(),
        net: 730000,
        gains: Some(ChildGains {
            investment_election: 0,
            qualified: 0,
            capital: 730000,
            special: Some(ChildSpecialGains {
                net_capital: 2000000,
                collectibles: 1000000,
                unrecaptured: 0,
            }),
        }),
    });
    // $20,000 original net gain, $7,300 allocated: $3,650 at 28%,
    // $3,650 at 15%, plus $320 ordinary tax = $1,890.
    // Child share $1,890 * .120 = $227, plus its $135 remaining tax.
    assert_eq!(p.calculate(1000, 2350, FederalStatus::Single, [0, 0], 0, [0; 3]), (227, 363));
}

#[test]
fn kiddie_child_special_gains_allocate_and_preserve_remaining_tax() {
    for (collectibles, unrecaptured, expected) in [(10000, 0, 2180), (0, 10000, 1961)] {
        let d = young_special_return(collectibles, unrecaptured, 0);
        let r = supported(&d);
        assert_eq!(value(&r, "kiddie_tax_allocated_capital_gain"), 7300);
        assert_eq!(
            value(&r, "kiddie_tax_allocated_28_percent_gain"),
            if collectibles > 0 { 7300 } else { 0 }
        );
        assert_eq!(
            value(&r, "kiddie_tax_allocated_25_percent_gain"),
            if unrecaptured > 0 { 7300 } else { 0 }
        );
        // Parent share uses the special rates; the child's remaining $1,350
        // follows the Tax Table.
        assert_eq!(value(&r, "regular_income_tax"), expected);
        assert_eq!(return_line(&r, "16"), expected * 100);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn kiddie_child_mixed_special_gains_keep_original_unrecaptured_limit() {
    let d = young_special_return(10000, 10000, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "kiddie_tax_allocated_28_percent_gain"), 8650);
    assert_eq!(value(&r, "kiddie_tax_allocated_25_percent_gain"), 8650);
    // Family worksheet line 35 uses original $10,000, not allocated $8,650:
    // $10,000 * .25 + $7,300 * .28 = $4,544. Own remainder adds $135.
    assert_eq!(value(&r, "kiddie_tax_parent_share"), 4544);
    assert_eq!(return_line(&r, "16"), 468000);
}

#[test]
fn kiddie_special_gain_allocation_uses_net_gain_after_short_term_loss() {
    let mut d = young_special_return(10000, 0, 10000);
    add(
        &mut d,
        income(FederalIncome::Sale {
            data: sale_source(native_sale(GainTerm::Short, 100000, Some(600000))),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "kiddie_tax_allocated_capital_gain"), 12300);
    // Short loss reduces the original 28% gain to $5,000 and net gain to
    // $15,000. Allocation is $5,000 * $12,300 / $15,000, not / $20,000.
    assert_eq!(value(&r, "kiddie_tax_allocated_28_percent_gain"), 4100);
    assert_eq!(return_line(&r, "16"), 237800);
}

#[test]
fn kiddie_special_gains_retain_childs_own_tax_minimum() {
    let mut parent = ordinary_parent();
    parent.taxable = 0;
    assert_eq!(
        parent
            .calculate(7300, 240000, FederalStatus::Single, [10000, 7300], 7300, [10000, 10000, 0])
            .1,
        52856
    );
}

#[test]
fn kiddie_personal_taxes_allocate_as_indirect_gain_deductions() {
    let mut d = young_special_return(10000, 0, 0);
    let deductions = d.deductions.as_mut().unwrap();
    itemized(deductions).personal_property_taxes = 400000;
    let r = supported(&d);
    // Personal taxes reduce the $10,000 gain to taxable/allocated $6,000;
    // they do not increase the direct-expense amount on Form 8615 line 2.
    assert_eq!(value(&r, "kiddie_tax_allocated_capital_gain"), 6000);
    assert_eq!(value(&r, "kiddie_tax_allocated_28_percent_gain"), 6000);
    assert_eq!(return_line(&r, "16"), 168000);
    itemized(d.deductions.as_mut().unwrap()).personal_property_taxes = 0;
    supported(&d); // No amount needs classification.
}

#[test]
fn kiddie_investment_property_tax_uses_direct_expense_worksheet() {
    let mut d = young_special_return(10000, 0, 0);
    itemized(d.deductions.as_mut().unwrap()).investment_property_taxes = 400000;
    let r = supported(&d);
    // Line 2 = $1,350 + $4,000. Worksheet #2 allocates all of this against gains.
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 4650);
    assert_eq!(value(&r, "kiddie_tax_allocated_capital_gain"), 4650);
    assert_eq!(value(&r, "kiddie_tax_allocated_28_percent_gain"), 4650);
    assert_eq!(return_line(&r, "16"), 143800);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 10000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn kiddie_direct_expenses_can_end_parent_calculation_without_parent_sources() {
    let mut d = young_interest_return();
    d.people.as_mut().unwrap().parent_tax = None;
    itemized(d.deductions.as_mut().unwrap()).investment_property_taxes = 900000;
    let r = supported(&d);
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 0);
    assert_eq!(return_line(&r, "16"), 10100);
}

#[test]
fn kiddie_direct_and_indirect_expenses_allocate_without_double_deduction() {
    use crate::kiddie::gain_allocation;
    // Worksheet #3: $4,000 directly connected plus $4,000 other deductions.
    // $2,000 taxable; direct dividends/gains $2,400/$1,600, then indirect same.
    assert_eq!(gain_allocation(2000, 10000, 10000, 8000, 6000, 4000, [4000; 2]), [1200, 800]);
    // Under the $1,350 direct-expense floor, Worksheet #1 still applies.
    assert_eq!(gain_allocation(7300, 10000, 10000, 1000, 6000, 4000, [1000; 2]), [4380, 2920]);
}

#[test]
fn independent_multiple_mixed_businesses_share_existing_half_se_allocation() {
    let mut d = young_interest_return();
    d.people.as_mut().unwrap().taxpayer.dependency = FilerDependency::Independent;
    add(&mut d, income(mixed_business(10000, 6000)));
    add(&mut d, income(mixed_business(20000, 12000)));
    add(&mut d, wage(0, Owner::Taxpayer, 5000000, 0));
    let r = supported(&d);
    // Half SE $2,120 is attributed $707/$1,413 to the $10k/$20k businesses.
    for (entry, amount) in d.income[1..3].iter().zip([2788, 5576]) {
        assert_eq!(
            value(&r, &alloc::format!("business_{}_kiddie_earned_income", entry.id)),
            amount
        );
    }
    assert_eq!(value(&r, "agi"), 87880);
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 28936);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn independent_service_profits_and_losses_net_before_mixed_earnings() {
    let mut d = young_interest_return();
    d.people.as_mut().unwrap().taxpayer.dependency = FilerDependency::Independent;
    add(&mut d, income(cash_business(10000)));
    add(&mut d, income(cash_business(-4000)));
    add(&mut d, income(capital_business(20000)));
    add(&mut d, wage(0, Owner::Taxpayer, 5000000, 0));
    // $26,000 combined business profit minus $6,000 net service earnings.
    assert_eq!(value(&supported(&d), "kiddie_tax_net_unearned_income"), 27300);
}

#[test]
fn child_service_business_has_normal_amt_exemption() {
    let mut d = young_interest_return();
    d.people.as_mut().unwrap().taxpayer.dependency = FilerDependency::Independent;
    d.income[0].income = savings_income(30000, 0);
    add(&mut d, income(cash_business(10000)));
    let r = supported(&d);
    // AGI 39,293, QBI 1,859: AMTI 37,434 is below 90,100.
    // Business earned-income classification does not shrink the exemption.
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 37434);
    assert_eq!(value(&r, "regular_income_tax"), 4693);
    assert_eq!(value(&r, "alternative_minimum_tax"), 0);
}

#[test]
fn child_service_receipts_preserve_alternate_loss_worksheet() {
    let mut d = young_interest_return();
    let mut source = cash_business(-1000);
    let FederalIncome::Business { data } = &mut source else { unreachable!() };
    match &mut data.activity {
        BusinessActivity::Ordinary { receipts, .. }
        | BusinessActivity::Gambling { receipts, .. } => *receipts = 100000,
        _ => panic!("expected nonfarm test business"),
    };
    // Preserve the same $1,000 net loss while documenting $1,000 receipts.
    data.expenses[0].amount += 100000;
    add(&mut d, income(source));
    assert_eq!(value(&supported(&d), "kiddie_tax_net_unearned_income"), 7300);
}

#[test]
fn dependent_mixed_and_capital_businesses_with_full_wage_deduction() {
    for source in [mixed_business(10000, 6000), capital_business(10000)] {
        let mut dependent = young_interest_return();
        add(&mut dependent, income(source));
        add(&mut dependent, wage(0, Owner::Taxpayer, 5000000, 0));
        let result = supported(&dependent);
        assert_eq!(return_line(&result, "12e"), 1610000);
        let mut independent = dependent.clone();
        independent.people.as_mut().unwrap().taxpayer.dependency = FilerDependency::Independent;
        let comparison = supported(&independent);
        for key in
            ["kiddie_tax_net_unearned_income", "regular_income_tax", "alternative_minimum_tax"]
        {
            assert_eq!(value(&result, key), value(&comparison, key));
        }
        assert_eq!(restart(dependent.clone()).draft, dependent);
    }
}

#[test]
fn dependent_mixed_business_wage_cap_accounts_for_half_se() {
    let mut d = young_interest_return();
    add(&mut d, income(mixed_business(10000, 6000)));
    // 16,357 wages minus 707 half-SE = 15,650; plus 450 reaches 16,100.
    add(&mut d, wage(0, Owner::Taxpayer, 1635700, 0));
    assert_eq!(return_line(&supported(&d), "12e"), 1610000);
    d.income.pop();
    add(&mut d, wage(0, Owner::Taxpayer, 1635600, 0));
    // The dependent standard deduction counts the business's whole profit
    // (worksheet earned income), so the mixed business changes nothing here and
    // needs no coordination refusal (unit B11 KIDS): wages of $16,356 plus the
    // profit less half of the self-employment tax reach the cap.
    assert_eq!(return_line(&supported(&d), "12e"), 1610000);
}

#[test]
fn investment_election_rejects_overallocated_sources_and_unimplemented_child_coordination() {
    let mut d = elected_portfolio();
    d.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().election =
        InvestmentElection::Allocated { capital: 1100000, dividends: 0 };
    unsupported(&d, "Investment interest");
    d.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().election =
        InvestmentElection::Allocated { capital: 0, dividends: 2000100 };
    unsupported(&d, "Investment interest");
    let mut child = young_interest_return();
    child.income = elected_portfolio().income;
    child.next_id = 20;
    child.deductions = elected_portfolio().deductions;
    unsupported(&child, "Investment tax");
}

#[test]
fn child_investment_interest_uses_deduction_after_section_68() {
    let mut d = young_interest_return();
    d.income[0].income = savings_income(100000, 0);
    add(&mut d, wage(0, Owner::Taxpayer, 80000000, 0));
    investment_borrowing(&mut d, 100000, 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "investment_interest_deduction"), 100000);
    assert_eq!(value(&r, "deduction"), 94595);
    // Form 8615's direct expense is the allowed deduction, not Form 4952's
    // limit before section 68: 100,000 - (1,350 + 94,595) = 4,055.
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 4055);
}

#[test]
fn child_interest_tracing_changes_preferential_allocation_not_income_limit() {
    for (source, expected) in [
        (InvestmentInterestUse::Ordinary, [3730, 5595]),
        (InvestmentInterestUse::Preferential, [1330, 1995]),
        (InvestmentInterestUse::Mixed { paid: 100000, carryover: 200000 }, [2530, 3795]),
    ] {
        let d = child_interest_gains(source);
        let r = supported(&d);
        assert_eq!(value(&r, "investment_interest_deduction"), 6000);
        assert_eq!(value(&r, "investment_interest_carryforward"), 0);
        assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 12650);
        // Worksheet #2: $1,350 * .2/.3 plus traced deductions * .4/.6.
        assert_eq!(value(&r, "kiddie_tax_allocated_dividends"), expected[0]);
        assert_eq!(value(&r, "kiddie_tax_allocated_capital_gain"), expected[1]);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn child_interest_unknown_or_limited_mixed_attribution_stays_explicit() {
    unsupported(&child_interest_gains(InvestmentInterestUse::Unresolved), "Investment tax");
    let mut d = child_interest_gains(InvestmentInterestUse::Mixed { paid: 100000, carryover: 0 });
    d.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().loans[0].paid = 1200000;
    unsupported(&d, "Investment tax");
    for source in [InvestmentInterestUse::Ordinary, InvestmentInterestUse::Preferential] {
        d.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().income_use = source;
        let r = supported(&d);
        assert_eq!(value(&r, "investment_interest_deduction"), 10000);
        assert_eq!(value(&r, "investment_interest_carryforward"), 4000);
    }
}

#[test]
fn child_interest_attribution_validates_each_source_pool() {
    let mut d = child_interest_gains(InvestmentInterestUse::Mixed { paid: 400001, carryover: 0 });
    assert!(!d.valid_stored());
    d.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().income_use =
        InvestmentInterestUse::Mixed { paid: 0, carryover: 200001 };
    assert!(!d.valid_stored());
}

#[test]
fn child_interest_attribution_does_not_block_when_nothing_is_deducted() {
    let mut d = child_interest_gains(InvestmentInterestUse::Unresolved);
    d.income.remove(0); // Only qualified dividends and long-term gains remain.
    let r = supported(&d);
    assert_eq!(value(&r, "investment_interest_deduction"), 0);
    assert_eq!(value(&r, "investment_interest_carryforward"), 6000);
}

#[test]
fn child_gambling_losses_reduce_unearned_income_after_ninety_percent_limit() {
    let d = child_gambling_return(10000, 8000);
    let r = supported(&d);
    assert_eq!(value(&r, "gambling_loss_deduction"), 7200);
    assert_eq!(value(&r, "kiddie_tax_direct_gambling_deduction"), 7200);
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 1450);
    assert_eq!(value(&r, "kiddie_tax_parent_share"), 319);
    assert_eq!(value(&r, "regular_income_tax"), 455);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn child_gambling_winnings_cap_can_eliminate_parent_rate_without_parent_facts() {
    let mut d = child_gambling_return(10000, 20000);
    d.people.as_mut().unwrap().parent_tax = None;
    let r = supported(&d);
    assert_eq!(value(&r, "gambling_loss_deduction"), 10000);
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 0);
    assert_eq!(value(&r, "regular_income_tax"), 0);
}

#[test]
fn child_gambling_expense_does_not_directly_reduce_preferential_income() {
    let mut d = child_interest_gains(InvestmentInterestUse::Ordinary);
    add(&mut d, gambling(10000, 8000));
    let r = supported(&d);
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 15450);
    // Worksheet #2: only the $1,350 allowance allocates .133 to dividends
    // and .200 to gains. Gambling and ordinary-interest expenses don't.
    assert_eq!(value(&r, "kiddie_tax_allocated_dividends"), 3820);
    assert_eq!(value(&r, "kiddie_tax_allocated_capital_gain"), 5730);
}

#[test]
fn child_gambling_deduction_respects_standard_choice_and_section68() {
    let mut d = child_gambling_return(10000, 8000);
    // 2026-11-02: automatic selection keeps the retained eligible itemized claim.
    d.deductions.as_mut().unwrap().itemize_override = None;
    let r = supported(&d);
    assert_eq!(value(&r, "kiddie_tax_direct_gambling_deduction"), 7200);
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 1450);
    let mut d = child_gambling_return(100000, 100000);
    add(&mut d, wage(0, Owner::Taxpayer, 80000000, 0));
    let r = supported(&d);
    assert_eq!(value(&r, "gambling_loss_deduction"), 90000);
    assert_eq!(value(&r, "itemized_deduction_reduction"), 4865);
    assert_eq!(value(&r, "kiddie_tax_direct_gambling_deduction"), 85135);
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 13515);
}

#[test]
fn two_special_joint_children_keep_one_child_shared_limit_independent() {
    for (special_first, expected) in [
        (
            true,
            &[
                ("adoption_child_1_special_needs_deemed_excess", 7670),
                ("adoption_child_2_special_needs_deemed_excess", 4000),
                ("adoption_joint_special_needs_deemed_excess", 11_670),
                ("adoption_benefits_excluded", 29_670),
                ("agi", 108_330),
            ][..],
        ),
        (
            false,
            &[
                ("adoption_child_2_benefits_excluded", 12_000),
                ("adoption_joint_special_needs_deemed_excess", 4000),
                ("agi", 116_000),
            ][..],
        ),
    ] {
        let mut d = if special_first {
            two_special_joint_adoption_return()
        } else {
            ordinary_and_special_joint_adoption_return()
        };
        d.credits.as_mut().unwrap().adoptions[1].limits =
            AdoptionLimits::Shared { credit: 1_200_000, exclusion: 1_200_000, agreed: true };
        let r = supported(&d);
        for (key, amount) in [
            ("adoption_1_exclusion_limit", 17_670),
            ("adoption_2_exclusion_limit", 12_000),
            ("adoption_1_credit_limit", 17_670),
            ("adoption_2_credit_limit", 12_000),
        ] {
            assert_eq!(value(&r, key), amount);
        }
        for &(key, amount) in expected {
            assert_eq!(value(&r, key), amount);
        }
        assert_eq!(return_line(&r, "1f"), if special_first { -1_167_000 } else { -400_000 });
        assert_eq!(
            value(&r, "adoption_refundable_credit")
                + value(&r, "adoption_nonrefundable_credit")
                + value(&r, "adoption_carryforward"),
            if special_first { 29_670 } else { 20_000 }
        );
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn two_special_joint_children_reject_a_reused_source_and_accept_history_over_cap_and_a_third_child()
{
    let d = two_special_joint_adoption_return();
    let mut reused = d.clone();
    joint_adoption_payments_mut(&mut reused, 1)[0].wage = 1;
    unsupported(&reused, "Adoption benefits");
    let mut missing_w2 = d.clone();
    if let FederalIncome::Wage { data } = &mut missing_w2.income[2].income {
        let mut data = edit_wage(data);
        data.medicare_wages = data.wages;
    }
    unsupported(&missing_w2, "Adoption benefits");
    let mut over_cap = d.clone();
    joint_adoption_payments_mut(&mut over_cap, 1)[0].received = 1_600_000;
    set_joint_adoption_w2(&mut over_cap, 2, 2_000_000, 1_600_000);
    // Above the child's $17,670 limit the exclusion is the limit and the rest
    // stays taxable (unit B11 KIDS), with an earlier exclusion lowering it.
    supported(&over_cap);
    let mut prior = d.clone();
    *joint_adoption_history_mut(&mut prior, 1) =
        AdoptionBenefitHistory::Exclusion { used: 100_000 };
    supported(&prior);
    let mut third = d;
    third.credits.as_mut().unwrap().adoptions.push(adoption_child(
        AdoptionKind::Domestic {
            finalized: AdoptionFinalization::Final { year: 2026 },
            special_needs: true,
        },
        &[],
    ));
    supported(&third); // Any number of children (the third has no employer benefits).
}

#[test]
fn ordinary_partial_and_special_deemed_joint_children_reconcile_employee_earnings() {
    for special_first in [false, true] {
        let mut d = if special_first {
            two_special_joint_adoption_return()
        } else {
            ordinary_and_special_joint_adoption_return()
        };
        let payments = joint_adoption_payments_mut(&mut d, 1);
        payments[0].received = 50_000;
        payments[1].received = 50_000;
        // $274,080 W-2 box 1 plus $11,000 code T is $285,080 MAGI: half of
        // each child's own Form 8839 exclusion base remains after phaseout.
        for (index, (wages, code_t)) in
            [(25_208_000, 600_000), (100_000, 400_000), (2_000_000, 50_000), (100_000, 50_000)]
                .into_iter()
                .enumerate()
        {
            set_joint_adoption_w2(&mut d, index, wages, code_t);
        }
        for _ in 0..2 {
            add_child(&mut d);
            let dependent = d.dependents.last_mut().unwrap();
            dependent.set_care_expenses(300_000);
        }
        d.credits.as_mut().unwrap().care = FederalCare::default();
        if special_first {
            // Each child's $17,670 exclusion independently phases to $8,835.
            document_care(&mut d);
            supported(&d);
            set_joint_adoption_exclusion_for_child(&mut d, 0, 8835, 0);
            unsupported(&d, "Adoption benefits"); // Taxpayer received only $6,000 code T.
            set_joint_adoption_exclusion_for_child(&mut d, 0, 4835, 4000);
        } else {
            // Ordinary-child automatic split: $5,000 x 6,000/10,000 = $3,000
            // taxpayer, $2,000 spouse (unit B11 KIDS).
            document_care(&mut d);
            let split = supported(&d);
            for (key, amount) in [
                ("adoption_child_1_employer_excluded_taxpayer", 3000),
                ("adoption_child_1_employer_excluded_spouse", 2000),
                ("adoption_child_1_employer_taxable_taxpayer", 3000),
                ("adoption_child_1_employer_taxable_spouse", 2000),
            ] {
                assert_eq!(value(&split, key), amount);
            }
            set_joint_adoption_exclusion_for_child(&mut d, 0, 1000, 4000);
        }
        let r = {
            document_care(&mut d);
            supported(&d)
        };
        assert_eq!(value(&r, "adoption_exclusion_magi"), 285_080);
        assert_eq!(
            value(&r, "adoption_child_1_benefits_excluded"),
            if special_first { 8835 } else { 5000 }
        );
        assert_eq!(value(&r, "adoption_child_2_benefits_excluded"), 8835);
        assert_eq!(
            value(&r, "adoption_child_1_benefits_taxable"),
            if special_first { 1165 } else { 5000 }
        );
        assert_eq!(value(&r, "adoption_child_2_benefits_taxable"), -7835);
        assert_eq!(value(&r, "adoption_joint_special_needs_deemed_excess"), 7835);
        assert_eq!(
            value(&r, "adoption_benefits_excluded"),
            if special_first { 17_670 } else { 13_835 }
        );
        assert_eq!(
            value(&r, "adoption_wage_adjustment"),
            if special_first { -6670 } else { -2835 }
        );
        assert_eq!(
            value(&r, "adoption_child_1_employer_taxable_taxpayer"),
            if special_first { 1165 } else { 5000 }
        );
        assert_eq!(value(&r, "adoption_child_1_employer_taxable_spouse"), 0);
        assert_eq!(value(&r, "adoption_child_2_employer_taxable_taxpayer"), 0);
        assert_eq!(value(&r, "adoption_child_2_employer_taxable_spouse"), 0);
        assert_eq!(value(&r, "care_earned_spouse"), 2000);
        assert_eq!(value(&r, "care_credit"), 400);
        assert_eq!(return_line(&r, "1a"), 27_408_000);
        assert_eq!(return_line(&r, "1f"), if special_first { -667_000 } else { -283_500 });
        if special_first {
            assert_eq!(value(&r, "adoption_child_2_special_needs_deemed_excess"), 7835);
        }
        assert_eq!(return_line(&r, "25a"), 1_200_000);
        assert_eq!(value(&r, "agi"), if special_first { 267_410 } else { 271_245 });
        let restored = restart(d.clone()).draft;
        assert_eq!(restored, d);
        assert_eq!(supported(&restored).return_lines, r.return_lines);

        let (taxpayer, spouse, adjustment, agi, earnings, credit) = if special_first {
            (6000, 2835, -667_000, 267_410, 3165, 633)
        } else {
            (5000, 0, -283_500, 271_245, 6000, 1200)
        };
        set_joint_adoption_exclusion_for_child(&mut d, 0, taxpayer, spouse);
        let alternative = {
            document_care(&mut d);
            supported(&d)
        };
        assert_eq!(return_line(&alternative, "1f"), adjustment);
        assert_eq!(value(&alternative, "agi"), agi);
        assert_eq!(value(&alternative, "care_earned_spouse"), earnings);
        assert_eq!(value(&alternative, "care_credit"), credit);
    }
}

#[test]
fn ordinary_and_special_joint_children_reject_a_reused_source_and_accept_the_rest() {
    let d = ordinary_and_special_joint_adoption_return();
    let mut reused = d.clone();
    joint_adoption_payments_mut(&mut reused, 1)[0].wage = 1;
    unsupported(&reused, "Adoption benefits");
    let mut pending = d.clone();
    pending.credits.as_mut().unwrap().adoptions[1].kind =
        AdoptionKind::Domestic { finalized: AdoptionFinalization::Pending, special_needs: true };
    supported(&pending); // A pending adoption excludes the actual code T like an ordinary one.
    let mut prior = d.clone();
    *joint_adoption_history_mut(&mut prior, 1) =
        AdoptionBenefitHistory::Exclusion { used: 100_000 };
    supported(&prior);
    let mut over_cap = d.clone();
    joint_adoption_payments_mut(&mut over_cap, 1)[0].received = 1_600_000;
    set_joint_adoption_w2(&mut over_cap, 2, 2_000_000, 1_600_000);
    supported(&over_cap);
    let mut third = d;
    third.credits.as_mut().unwrap().adoptions.push(adoption_child(
        AdoptionKind::Domestic { finalized: AdoptionFinalization::Pending, special_needs: false },
        &[],
    ));
    supported(&third);
}

#[test]
fn two_children_full_phaseout_attributes_each_code_t_owner_once() {
    let mut d = two_child_joint_adoption_return();
    set_joint_adoption_w2(&mut d, 0, 30_000_000, 600_000);
    set_joint_adoption_w2(&mut d, 1, 100_000, 400_000);
    set_joint_adoption_w2(&mut d, 3, 100_000, 300_000);
    add_child(&mut d);
    add_child(&mut d);
    d.dependents[0].set_care_expenses(600_000);
    d.credits.as_mut().unwrap().care = FederalCare::default();
    let r = {
        document_care(&mut d);
        supported(&d)
    };
    assert_eq!(value(&r, "adoption_exclusion_magi"), 340_000);
    assert_eq!(value(&r, "adoption_child_1_benefits_taxable"), 10_000);
    assert_eq!(value(&r, "adoption_child_2_benefits_taxable"), 8000);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 0);
    assert_eq!(value(&r, "adoption_wage_adjustment"), 18_000);
    assert_eq!(value(&r, "care_earned_spouse"), 9000); // $2,000 W-2 plus $7,000 of spouse code T.
    assert_eq!(value(&r, "care_credit"), 1200);
    assert_eq!(value(&r, "adoption_refundable_credit"), 0);
    assert_eq!(return_line(&r, "1a"), 32_200_000);
    assert_eq!(return_line(&r, "1f"), 1_800_000);
    assert_eq!(return_line(&r, "25a"), 1_200_000);
    assert_eq!(value(&r, "agi"), 340_000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn two_children_partly_phased_code_t_reconciles_each_child_and_care_earnings() {
    let mut d = two_child_joint_adoption_return();
    // $267,080 box 1 plus $18,000 code T is $285,080 Form 8839
    // line 25 MAGI, so half of each child's separate exclusion remains.
    set_joint_adoption_w2(&mut d, 0, 24_508_000, 600_000);
    set_joint_adoption_w2(&mut d, 1, 100_000, 400_000);
    set_joint_adoption_w2(&mut d, 3, 100_000, 300_000);
    for _ in 0..2 {
        add_child(&mut d);
        let dependent = d.dependents.last_mut().unwrap();
        dependent.set_care_expenses(300_000);
    }
    d.credits.as_mut().unwrap().care = FederalCare::default();
    // Automatic: each partly excluded child's exclusion follows the code T
    // shares (child 1 $5,000 x 6/10 = $3,000, child 2 $4,000 x 5/8 = $2,500).
    let split = {
        document_care(&mut d);
        supported(&d)
    };
    assert_eq!(value(&split, "adoption_child_1_employer_excluded_taxpayer"), 3000);
    assert_eq!(value(&split, "adoption_child_2_employer_excluded_taxpayer"), 2500);
    assert_eq!(value(&split, "adoption_benefits_excluded"), 9000);
    set_joint_adoption_exclusion_for_child(&mut d, 0, 1000, 4000);
    set_joint_adoption_exclusion_for_child(&mut d, 1, 1000, 3000);
    let r = {
        document_care(&mut d);
        supported(&d)
    };
    assert_eq!(value(&r, "adoption_exclusion_magi"), 285_080);
    assert_eq!(value(&r, "adoption_child_1_benefits_excluded"), 5000);
    assert_eq!(value(&r, "adoption_child_2_benefits_excluded"), 4000);
    assert_eq!(value(&r, "adoption_child_1_benefits_taxable"), 5000);
    assert_eq!(value(&r, "adoption_child_2_benefits_taxable"), 4000);
    assert_eq!(value(&r, "adoption_child_1_employer_excluded_taxpayer"), 1000);
    assert_eq!(value(&r, "adoption_child_1_employer_excluded_spouse"), 4000);
    assert_eq!(value(&r, "adoption_child_2_employer_excluded_taxpayer"), 1000);
    assert_eq!(value(&r, "adoption_child_2_employer_excluded_spouse"), 3000);
    assert_eq!(value(&r, "adoption_child_1_employer_taxable_taxpayer"), 5000);
    assert_eq!(value(&r, "adoption_child_1_employer_taxable_spouse"), 0);
    assert_eq!(value(&r, "adoption_child_2_employer_taxable_taxpayer"), 4000);
    assert_eq!(value(&r, "adoption_child_2_employer_taxable_spouse"), 0);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 9000); // Form 8839 line 30.
    assert_eq!(value(&r, "adoption_wage_adjustment"), 9000); // Form 8839 line 31.
    assert_eq!(value(&r, "care_earned_spouse"), 2000);
    assert_eq!(value(&r, "care_credit"), 400);
    assert_eq!(return_line(&r, "1a"), 26_708_000);
    assert_eq!(return_line(&r, "1f"), 900_000);
    assert_eq!(return_line(&r, "25a"), 1_200_000);
    assert_eq!(value(&r, "agi"), 276_080);
    let restored = restart(d.clone()).draft;
    assert_eq!(restored, d);
    assert_eq!(supported(&restored).return_lines, r.return_lines);

    // A different confirmed split preserves joint Form 8839 and Form 1040
    // totals while changing only the spouse's Form 2441 earned-income limit.
    set_joint_adoption_exclusion_for_child(&mut d, 0, 5000, 0);
    set_joint_adoption_exclusion_for_child(&mut d, 1, 4000, 0);
    let alternative = {
        document_care(&mut d);
        supported(&d)
    };
    assert_eq!(value(&alternative, "adoption_benefits_excluded"), 9000);
    assert_eq!(return_line(&alternative, "1f"), 900_000);
    assert_eq!(value(&alternative, "care_earned_spouse"), 9000);
    assert_eq!(value(&alternative, "care_credit"), 1200);
}

#[test]
fn two_children_partial_exclusions_reject_unconfirmed_or_inconsistent_child_split() {
    let mut d = two_child_joint_adoption_return();
    set_joint_adoption_w2(&mut d, 0, 24_508_000, 600_000);
    set_joint_adoption_w2(&mut d, 1, 100_000, 400_000);
    set_joint_adoption_w2(&mut d, 3, 100_000, 300_000);
    set_joint_adoption_exclusion_for_child(&mut d, 0, 1000, 4000);
    set_joint_adoption_exclusion_for_child(&mut d, 1, 1000, 3000);
    let mut unconfirmed = d.clone();
    if let AdoptionBenefits::JointEmployees {
        allocation: JointAdoptionExclusion::Attributed { confirmed, .. },
        ..
    } = &mut unconfirmed.credits.as_mut().unwrap().adoptions[1].benefits
    {
        *confirmed = false;
    }
    // An unconfirmed split leaves the partial exclusion out.
    let r = supported(&unconfirmed);
    assert!(noted(&r, "WARN_ADOPTION_EXCLUSION_SPLIT_LEFT_OUT"), "{r:?}");
    assert_eq!(value(&r, "adoption_child_2_benefits_excluded"), 0);
    let mut wrong_child = d.clone();
    set_joint_adoption_exclusion_for_child(&mut wrong_child, 1, 2000, 3000);
    unsupported(&wrong_child, "Adoption benefits"); // Cannot borrow child 1's unused exclusion.
    let mut over_spouse = d.clone();
    set_joint_adoption_exclusion_for_child(&mut over_spouse, 0, 0, 5000);
    unsupported(&over_spouse, "Adoption benefits");
    let mut stale = d.clone();
    set_joint_adoption_w2(&mut stale, 0, 24_608_000, 600_000);
    unsupported(&stale, "Adoption benefits"); // MAGI changed; both Form 8839 line 29s did too.
    let mut reused = d;
    joint_adoption_payments_mut(&mut reused, 1)[0].wage = 1;
    unsupported(&reused, "Adoption benefits");
}

#[test]
fn two_children_reject_reused_or_unresolved_code_t_sources_and_accept_the_rest() {
    let d = two_child_joint_adoption_return();
    let mut reused = d.clone();
    joint_adoption_payments_mut(&mut reused, 1)[0].wage = 1;
    unsupported(&reused, "Adoption benefits");
    let mut same_child = d.clone();
    same_child.credits.as_mut().unwrap().adoptions[1].label = "First child".into();
    supported(&same_child); // Distinct W-2 sources are what keeps two children apart.
    let mut prior = d.clone();
    *joint_adoption_history_mut(&mut prior, 1) =
        AdoptionBenefitHistory::Exclusion { used: 100_000 };
    supported(&prior);
    let mut missing = d.clone();
    joint_adoption_payments_mut(&mut missing, 1)[1].wage = 9999;
    unsupported(&missing, "Adoption benefits");
    let mut over_cap = d.clone();
    joint_adoption_payments_mut(&mut over_cap, 1)[0].received = 1_600_000;
    set_joint_adoption_w2(&mut over_cap, 2, 2_000_000, 1_600_000);
    supported(&over_cap); // This child's own cap binds below phaseout.
    let mut middle = d.clone();
    set_joint_adoption_w2(&mut middle, 0, 21_000_000, 600_000);
    supported(&middle); // Shared MAGI partly phases both exclusions; each splits by code T.
    let mut third = d;
    third.credits.as_mut().unwrap().adoptions.push(adoption_child(
        AdoptionKind::Domestic {
            finalized: AdoptionFinalization::Final { year: 2026 },
            special_needs: false,
        },
        &[],
    ));
    supported(&third);
}

#[test]
fn joint_code_t_cap_binding_uses_confirmed_employee_exclusion_once() {
    let mut d = joint_employer_adoption_return();
    joint_adoption_payments_mut(&mut d, 0)[0].received = 1_400_000;
    set_joint_adoption_w2(&mut d, 0, 6_000_000, 1_400_000);
    // $20,000 code T exceeds the child's $17,670 cap: Automatic splits the
    // $17,670 exclusion by code T (taxpayer $16,000 and spouse $4,000 of
    // $20,000: $14,136 and $3,534).
    let split = supported(&d);
    assert_eq!(value(&split, "adoption_employer_excluded_taxpayer"), 14_136);
    assert_eq!(value(&split, "adoption_employer_excluded_spouse"), 3_534);
    set_joint_adoption_exclusion(&mut d, 13_670, 4000);
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_exclusion_magi"), 120_000);
    assert_eq!(value(&r, "adoption_employer_excluded_taxpayer"), 13_670);
    assert_eq!(value(&r, "adoption_employer_excluded_spouse"), 4000);
    assert_eq!(value(&r, "adoption_employer_taxable_taxpayer"), 2330);
    assert_eq!(value(&r, "adoption_employer_taxable_spouse"), 0);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 17_670); // Form 8839 line 29.
    assert_eq!(value(&r, "adoption_wage_adjustment"), 2330); // Form 8839 line 31.
    assert_eq!(return_line(&r, "1a"), 10_000_000); // W-2 box 1 unchanged.
    assert_eq!(return_line(&r, "1f"), 233_000);
    assert_eq!(return_line(&r, "25a"), 950_000); // W-2 box 2 unchanged.
    assert_eq!(value(&r, "agi"), 102_330);
    assert_eq!(value(&r, "adoption_refundable_credit"), 5120); // Separate unreimbursed expenses.
    let restored = restart(d.clone()).draft;
    assert_eq!(restored, d);
    assert_eq!(supported(&restored).return_lines, r.return_lines);
}

#[test]
fn joint_code_t_partial_phaseout_preserves_owner_care_earnings_and_joint_tax() {
    let mut d = joint_employer_adoption_return();
    // $273,080 box 1 wages plus $12,000 code T gives $285,080 Form 8839 MAGI:
    // exactly half of this child's $12,000 potential exclusion remains.
    set_joint_adoption_w2(&mut d, 0, 25_208_000, 600_000);
    set_joint_adoption_w2(&mut d, 2, 100_000, 400_000);
    for _ in 0..2 {
        add_child(&mut d);
        let dependent = d.dependents.last_mut().unwrap();
        dependent.set_care_expenses(300_000);
    }
    d.credits.as_mut().unwrap().care = FederalCare::default();
    // Automatic: the $6,000 left after the phaseout splits by code T, taxpayer
    // $8,000 and spouse $4,000 of $12,000: $4,000 and $2,000.
    let split = {
        document_care(&mut d);
        supported(&d)
    };
    assert_eq!(value(&split, "adoption_employer_excluded_taxpayer"), 4000);
    assert_eq!(value(&split, "adoption_employer_excluded_spouse"), 2000);
    set_joint_adoption_exclusion(&mut d, 4000, 2000);
    let r = {
        document_care(&mut d);
        supported(&d)
    };
    assert_eq!(value(&r, "adoption_exclusion_magi"), 285_080);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 6000);
    assert_eq!(value(&r, "adoption_employer_taxable_taxpayer"), 4000);
    assert_eq!(value(&r, "adoption_employer_taxable_spouse"), 2000);
    assert_eq!(value(&r, "adoption_wage_adjustment"), 6000);
    assert_eq!(value(&r, "care_earned_spouse"), 3000); // Box 1 $1,000 + spouse's taxable code T $2,000.
    assert_eq!(value(&r, "care_credit"), 600);
    assert_eq!(return_line(&r, "1a"), 27_308_000);
    assert_eq!(return_line(&r, "1f"), 600_000);
    assert_eq!(return_line(&r, "25a"), 950_000);
    assert_eq!(value(&r, "agi"), 279_080);
    assert_eq!(value(&r, "adoption_refundable_credit"), 5120); // Credit MAGI uses actual AGI, unlike exclusion MAGI.
    let restored = restart(d.clone()).draft;
    assert_eq!(restored, d);
    assert_eq!(supported(&restored).return_lines, r.return_lines);

    // An equally sized supported attribution changes only owner-sensitive
    // care earnings, not the Form 8839 or joint Form 1040 taxable totals.
    set_joint_adoption_exclusion(&mut d, 6000, 0);
    let other = {
        document_care(&mut d);
        supported(&d)
    };
    assert_eq!(value(&other, "adoption_exclusion_magi"), 285_080);
    assert_eq!(value(&other, "adoption_wage_adjustment"), 6000);
    assert_eq!(return_line(&other, "1f"), 600_000);
    assert_eq!(value(&other, "care_earned_spouse"), 5000);
    assert_eq!(value(&other, "care_credit"), 1000);
}

#[test]
fn joint_code_t_partial_allocation_rejects_unsourced_or_inconsistent_employee_split() {
    let mut d = joint_employer_adoption_return();
    set_joint_adoption_w2(&mut d, 0, 25_208_000, 600_000);
    set_joint_adoption_w2(&mut d, 2, 100_000, 400_000);
    supported(&d); // Automatic splits by each employee's code T, not an ordering.

    set_joint_adoption_exclusion(&mut d, 4000, 2000);
    let mut unconfirmed = d.clone();
    if let AdoptionBenefits::JointEmployees {
        allocation: JointAdoptionExclusion::Attributed { confirmed, .. },
        ..
    } = &mut unconfirmed.credits.as_mut().unwrap().adoptions[0].benefits
    {
        *confirmed = false;
    }
    // An unconfirmed split leaves the partial exclusion out.
    let r = supported(&unconfirmed);
    assert!(noted(&r, "WARN_ADOPTION_EXCLUSION_SPLIT_LEFT_OUT"), "{r:?}");
    assert_eq!(value(&r, "adoption_benefits_excluded"), 0);

    let mut wrong_sum = d.clone();
    set_joint_adoption_exclusion(&mut wrong_sum, 3999, 2000);
    unsupported(&wrong_sum, "Adoption benefits");
    let mut over_employee = d.clone();
    set_joint_adoption_exclusion(&mut over_employee, 1000, 5000);
    unsupported(&over_employee, "Adoption benefits");
    let mut stale = d.clone();
    set_joint_adoption_w2(&mut stale, 0, 25_308_000, 600_000);
    unsupported(&stale, "Adoption benefits"); // The MAGI phaseout now differs by $1,000.

    let mut invalid_cents = d.clone();
    if let AdoptionBenefits::JointEmployees {
        allocation: JointAdoptionExclusion::Attributed { taxpayer, .. },
        ..
    } = &mut invalid_cents.credits.as_mut().unwrap().adoptions[0].benefits
    {
        *taxpayer += 1;
    }
    assert!(!invalid_cents.valid_stored());

    let mut all_excluded = joint_employer_adoption_return();
    set_joint_adoption_exclusion(&mut all_excluded, 8000, 4000);
    unsupported(&all_excluded, "Adoption benefits"); // Automatic is the only all-or-none path.
}

#[test]
fn joint_special_needs_code_t_equal_to_cap_has_no_unsourced_wage_split() {
    let mut d = joint_employer_adoption_return();
    let child = &mut d.credits.as_mut().unwrap().adoptions[0];
    child.kind = AdoptionKind::Domestic {
        finalized: AdoptionFinalization::Final { year: 2026 },
        special_needs: true,
    };
    child.expenses.clear(); // The special-needs credit is deemed, not a second claim for reimbursements.
    if let AdoptionBenefits::JointEmployees { payments, .. } = &mut child.benefits {
        payments[0].received = 1_167_000;
    }
    set_joint_adoption_w2(&mut d, 0, 6_000_000, 1_167_000);
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_employer_payment_1"), 11_670);
    assert_eq!(value(&r, "adoption_exclusion_magi"), 117_670);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 17_670);
    assert_eq!(value(&r, "adoption_wage_adjustment"), 0);
    assert_eq!(return_line(&r, "1f"), 0);
    assert_eq!(value(&r, "agi"), 100_000);
    assert_eq!(value(&r, "adoption_refundable_credit"), 5120);
    assert_eq!(
        value(&r, "adoption_refundable_credit")
            + value(&r, "adoption_nonrefundable_credit")
            + value(&r, "adoption_carryforward"),
        17_670
    );
    assert_eq!(restart(d.clone()).draft, d);
    let mut under = d;
    joint_adoption_payments_mut(&mut under, 0)[0].received -= 100;
    set_joint_adoption_w2(&mut under, 0, 6_000_000, 1_166_900);
    let deemed = supported(&under);
    assert_eq!(value(&deemed, "adoption_joint_special_needs_deemed_excess"), 1);
    assert_eq!(return_line(&deemed, "1f"), -100);
}

#[test]
fn joint_special_needs_deemed_excess_is_negative_line_1f_not_employee_earnings() {
    let mut d = joint_special_needs_below_code_t_cap();
    add_child(&mut d);
    d.dependents[0].set_care_expenses(300_000);
    d.credits.as_mut().unwrap().care = FederalCare::default();
    let r = {
        document_care(&mut d);
        supported(&d)
    };
    assert_eq!(value(&r, "adoption_exclusion_magi"), 112_000);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 17_670); // Form 8839 line 29.
    assert_eq!(value(&r, "adoption_joint_special_needs_deemed_excess"), 5670);
    assert_eq!(value(&r, "adoption_employer_excluded_taxpayer"), 8000);
    assert_eq!(value(&r, "adoption_employer_excluded_spouse"), 4000);
    assert_eq!(value(&r, "adoption_employer_taxable_taxpayer"), 0);
    assert_eq!(value(&r, "adoption_employer_taxable_spouse"), 0);
    assert_eq!(value(&r, "adoption_wage_adjustment"), -5670); // Form 8839 line 31.
    assert_eq!(return_line(&r, "1a"), 10_000_000); // W-2 box 1 is unchanged.
    assert_eq!(return_line(&r, "1f"), -567_000);
    assert_eq!(return_line(&r, "25a"), 950_000); // W-2 box 2 is unchanged.
    assert_eq!(value(&r, "care_earned_spouse"), 20_000); // No deemed amount assigned to spouse.
    assert_eq!(value(&r, "agi"), 94_330);
    assert_eq!(value(&r, "adoption_refundable_credit"), 5120);
    assert_eq!(
        value(&r, "adoption_refundable_credit")
            + value(&r, "adoption_nonrefundable_credit")
            + value(&r, "adoption_carryforward"),
        17_670 // Separate deemed special-needs credit, without duplicate expenses.
    );
    let restored = restart(d.clone()).draft;
    assert_eq!(restored, d);
    assert_eq!(supported(&restored).return_lines, r.return_lines);
}

#[test]
fn joint_special_needs_deemed_excess_phaseout_boundary_preserves_line_31_sign() {
    let mut d = joint_special_needs_below_code_t_cap();
    // Form 8839 exclusion MAGI is box 1 wages plus all $12,000 code T.
    // At $277,914, line 29 is $12,001: one deemed dollar above actual benefits.
    set_joint_adoption_w2(&mut d, 0, 22_591_400, 600_000);
    let below = supported(&d);
    assert_eq!(value(&below, "adoption_exclusion_magi"), 277_914);
    assert_eq!(value(&below, "adoption_benefits_excluded"), 12_001);
    assert_eq!(value(&below, "adoption_joint_special_needs_deemed_excess"), 1);
    assert_eq!(return_line(&below, "1f"), -100);
    assert_eq!(value(&below, "agi"), 265_913);

    set_joint_adoption_w2(&mut d, 0, 22_591_500, 600_000);
    let equal = supported(&d);
    assert_eq!(value(&equal, "adoption_exclusion_magi"), 277_915);
    assert_eq!(value(&equal, "adoption_benefits_excluded"), 12_000);
    assert_eq!(return_line(&equal, "1f"), 0);

    set_joint_adoption_w2(&mut d, 0, 22_591_700, 600_000);
    // One actual code T dollar is now taxable: Automatic splits the $11,999
    // line 29 by code T, taxpayer $11,999 x 8/12 = $7,999 and spouse $4,000.
    let automatic = supported(&d);
    assert_eq!(value(&automatic, "adoption_employer_taxable_taxpayer"), 1);
    assert_eq!(value(&automatic, "adoption_employer_taxable_spouse"), 0);
    set_joint_adoption_exclusion(&mut d, 7999, 4000);
    let above = supported(&d);
    assert_eq!(value(&above, "adoption_benefits_excluded"), 11_999);
    assert_eq!(value(&above, "adoption_employer_taxable_taxpayer"), 1);
    assert_eq!(value(&above, "adoption_employer_taxable_spouse"), 0);
    assert_eq!(return_line(&above, "1f"), 100);
}

#[test]
fn joint_special_needs_deemed_excess_requires_confirmed_source_and_automatic_split() {
    let d = joint_special_needs_below_code_t_cap();
    let mut attributed = d.clone();
    set_joint_adoption_exclusion(&mut attributed, 8000, 4000);
    unsupported(&attributed, "Adoption benefits"); // Deemed excess is not employee-attributed.
    let mut missing_w2 = d.clone();
    joint_adoption_payments_mut(&mut missing_w2, 0)[2].wage = 9999;
    unsupported(&missing_w2, "Adoption benefits");
    let mut missing_fica = d.clone();
    if let FederalIncome::Wage { data } = &mut missing_fica.income[2].income {
        let mut data = edit_wage(data);
        data.medicare_wages = data.wages;
    }
    unsupported(&missing_fica, "Adoption benefits");
    let mut prior = d.clone();
    *joint_adoption_history_mut(&mut prior, 0) =
        AdoptionBenefitHistory::Exclusion { used: 100_000 };
    supported(&prior);
    let mut over_cap = d;
    joint_adoption_payments_mut(&mut over_cap, 0)[0].received = 1_200_000;
    set_joint_adoption_w2(&mut over_cap, 0, 6_000_000, 1_200_000);
    // Above-cap special-needs code T: the limit is excluded and the rest is
    // split in proportion to the code T.
    supported(&over_cap);
}

#[test]
fn joint_employers_share_one_child_exclusion_and_keep_w2_tax_once() {
    let d = joint_employer_adoption_return();
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_employer_payment_1"), 6000);
    assert_eq!(value(&r, "adoption_employer_payment_2"), 2000);
    assert_eq!(value(&r, "adoption_employer_payment_3"), 4000);
    assert_eq!(value(&r, "adoption_1_exclusion_limit"), 17_670);
    assert_eq!(value(&r, "adoption_exclusion_magi"), 112_000);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 12_000);
    assert_eq!(value(&r, "adoption_wage_adjustment"), 0);
    assert_eq!(value(&r, "agi"), 100_000);
    assert_eq!(return_line(&r, "1a"), 10_000_000);
    assert_eq!(return_line(&r, "1f"), 0);
    assert_eq!(return_line(&r, "25a"), 950_000);
    assert_eq!(value(&r, "adoption_refundable_credit"), 5120);
    assert_eq!(value(&r, "adoption_nonrefundable_credit"), 2880);
    assert_eq!(restart(d.clone()).draft, d);
    assert_eq!(supported(&restart(d).draft).return_lines, r.return_lines);
}

#[test]
fn joint_employer_adoption_requires_distinct_w2_sources_and_resolved_history() {
    let d = joint_employer_adoption_return();
    let mut missing = d.clone();
    joint_adoption_payments_mut(&mut missing, 0)[1].wage = 9999;
    unsupported(&missing, "Adoption benefits");

    let mut duplicate = d.clone();
    joint_adoption_payments_mut(&mut duplicate, 0)[1].wage = 1;
    assert!(!duplicate.valid_stored());

    let mut wrong_owner = d.clone();
    joint_adoption_payments_mut(&mut wrong_owner, 0)[0].owner = Owner::Spouse;
    unsupported(&wrong_owner, "Adoption benefits");

    let mut missing_fica_wages = d.clone();
    if let FederalIncome::Wage { data } = &mut missing_fica_wages.income[1].income {
        let mut data = edit_wage(data);
        data.medicare_wages = data.wages;
    }
    unsupported(&missing_fica_wages, "Adoption benefits");

    let mut prior = d.clone();
    *joint_adoption_history_mut(&mut prior, 0) =
        AdoptionBenefitHistory::Exclusion { used: 100_000 };
    supported(&prior);

    let mut unconfirmed = d.clone();
    if let AdoptionBenefits::JointEmployees { ordinary, .. } =
        &mut unconfirmed.credits.as_mut().unwrap().adoptions[0].benefits
    {
        *ordinary = false;
    }
    // Not sure: the benefits are taxable, with an assumption warning.
    let r = supported(&unconfirmed);

    assert_eq!(value(&r, "adoption_benefits_excluded"), 0);

    let mut over_limit = d.clone();
    joint_adoption_payments_mut(&mut over_limit, 0)[0].received = 1_500_000;
    unsupported(&over_limit, "Adoption benefits");

    let mut phased = d.clone();
    phased.income[0] = wage(1, Owner::Taxpayer, 30_000_000, 650_000);
    unsupported(&phased, "Adoption benefits");

    let mut special = d.clone();
    special.credits.as_mut().unwrap().adoptions[0].kind = AdoptionKind::Domestic {
        finalized: AdoptionFinalization::Final { year: 2026 },
        special_needs: true,
    };
    let special_result = supported(&special);
    assert_eq!(return_line(&special_result, "1f"), -567_000);

    let mut two_children = d;
    two_children.credits.as_mut().unwrap().adoptions.push(adoption_child(
        AdoptionKind::Domestic { finalized: AdoptionFinalization::Pending, special_needs: false },
        &[],
    ));
    supported(&two_children);
}

#[test]
fn employer_adoption_exclusion_does_not_duplicate_wages_or_credit_expenses() {
    for (wages, benefits, expense, excluded, magi, agi, wage_adjustment, credit) in [
        (6000000, 4000, 600000, 4000, 64000, 60000, 0, 880),
        (26508000, 20000, 1000000, 8835, 285080, 276245, 1116500, 2089),
    ] {
        let mut d = complete(FederalStatus::Single);
        if wages != 6000000 {
            d.income[0] = wage(1, Owner::Taxpayer, wages, 0);
        }
        employer_adoption(&mut d, benefits, 0, false);
        d.credits.as_mut().unwrap().adoptions[0]
            .expenses
            .push(AdoptionExpense { year: 2026, amount: expense });
        let r = supported(&d);
        assert_eq!(value(&r, "adoption_benefits_excluded"), excluded);
        assert_eq!(value(&r, "adoption_exclusion_magi"), magi);
        assert_eq!(value(&r, "agi"), agi);
        assert_eq!(return_line(&r, "1f"), wage_adjustment);
        assert_eq!(value(&r, "adoption_refundable_credit"), 5120);
        // $10,000 phaseout-row credit expense uses the lower post-exclusion AGI.
        assert_eq!(value(&r, "adoption_nonrefundable_credit"), credit);
        if wages == 6000000 {
            assert_eq!(return_line(&r, "1a"), 6000000);
            assert_eq!(restart(d.clone()).draft, d);
        }
    }
}

#[test]
fn adoption_exclusion_does_not_reduce_social_security_modified_income() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 2000000, 0);
    add(&mut d, income(social_security_net(2000000, 0)));
    employer_adoption(&mut d, 10000, 0, false);
    let r = supported(&d);
    // Provisional income is $20k wages + $10k assistance + $10k half benefits.
    // 0.85 * (40k - 34k) + 4.5k = 9.6k taxable benefits.
    assert_eq!(value(&r, "taxable_social_security"), 9600);
    assert_eq!(value(&r, "adoption_exclusion_magi"), 39600);
    assert_eq!(value(&r, "agi"), 29600);
}

#[test]
fn adoption_credit_phaseout_rounds_reduction_before_subtraction() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 26510000, 0);
    d.credits.as_mut().unwrap().adoptions.push(adoption_child(
        AdoptionKind::Domestic {
            finalized: AdoptionFinalization::Final { year: 2026 },
            special_needs: false,
        },
        &[(2026, 1000)],
    ));
    // Line 10: $1,000 * $20 / $40,000 = $0.50, rounded to $1.
    assert_eq!(value(&supported(&d), "adoption_refundable_credit"), 999);
}

#[test]
fn adoption_exclusion_phaseout_endpoints_and_employee_ownership() {
    for (wages, excluded) in [(255080, 10000), (295080, 0)] {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, wages * 100, 0);
        employer_adoption(&mut d, 10000, 0, false);
        let r = supported(&d);
        assert_eq!(value(&r, "adoption_benefits_excluded"), excluded);
        assert_eq!(return_line(&r, "1f"), (10000 - excluded) * 100);
    }
    let mut d = complete(FederalStatus::Single);
    employer_adoption(&mut d, 10000, 0, false);
    if let AdoptionBenefits::Program { owner, .. } =
        &mut d.credits.as_mut().unwrap().adoptions[0].benefits
    {
        *owner = Owner::Spouse;
    }
    unsupported(&d, "Adoption benefits");
}

#[test]
fn adoption_benefits_combine_different_children_and_employee_owners_on_joint_return() {
    let mut d = complete(FederalStatus::Joint);
    add(&mut d, wage(0, Owner::Spouse, 3000000, 0));
    employer_adoption(&mut d, 20000, 0, false);
    employer_adoption(&mut d, 20000, 0, false);
    let child = &mut d.credits.as_mut().unwrap().adoptions[1];
    child.label = "Second synthetic child".into();
    if let AdoptionBenefits::Program { owner, .. } = &mut child.benefits {
        *owner = Owner::Spouse;
    }
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_exclusion_magi"), 170000);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 35340);
    assert_eq!(return_line(&r, "1f"), 466000);
    assert_eq!(value(&r, "agi"), 134660);
}

#[test]
fn denied_current_adoption_does_not_disallow_an_eligible_prior_carry() {
    let mut d = separated_adoption_return();
    d.credits.as_mut().unwrap().adoptions[0].home_over_half_year = false;
    d.credits.as_mut().unwrap().adoption_carry.push(AdoptionCarry {
        year: 2025,
        amount: 300000,
        eligible: true,
    });
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_nonrefundable_credit"), 3000);
    assert_eq!(value(&r, "adoption_refundable_credit"), 0);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 0);
}

#[test]
fn adoption_shared_credit_and_exclusion_limits_are_independent() {
    let d = shared_adoption_return(false);
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_1_credit_limit"), 8000);
    assert_eq!(value(&r, "adoption_1_exclusion_limit"), 5000);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 5000);
    assert_eq!(value(&r, "adoption_refundable_credit"), 5120);
    assert_eq!(value(&r, "adoption_nonrefundable_credit"), 2880);
    assert_eq!(return_line(&r, "1f"), 500000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn adoption_shared_limits_subtract_prior_claims_after_allocation() {
    let mut d = shared_adoption_return(false);
    let child = &mut d.credits.as_mut().unwrap().adoptions[0];
    child.prior = AdoptionPriorCredit { previous: 100000, expenses: 300000 };
    if let AdoptionBenefits::Program { history, .. } = &mut child.benefits {
        *history = AdoptionBenefitHistory::Exclusion { used: 200000 };
    }
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_refundable_credit"), 4000);
    assert_eq!(value(&r, "adoption_nonrefundable_credit"), 0);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 3000);
    assert_eq!(return_line(&r, "1f"), 700000);
}

#[test]
fn adoption_special_needs_respects_shared_limits_without_actual_payments() {
    let mut d = shared_adoption_return(true);
    d.credits.as_mut().unwrap().adoptions[0].expenses.clear();
    let r = supported(&d);
    assert_eq!(return_line(&r, "1f"), -500000);
    assert_eq!(value(&r, "adoption_refundable_credit"), 5120);
    assert_eq!(value(&r, "adoption_nonrefundable_credit"), 2880);
}

#[test]
fn adoption_zero_credit_share_does_not_remove_exclusion_or_owned_carry() {
    let mut d = shared_adoption_return(false);
    d.credits.as_mut().unwrap().adoptions[0].limits =
        AdoptionLimits::Shared { credit: 0, exclusion: 1767000, agreed: true };
    d.credits.as_mut().unwrap().adoption_carry.push(AdoptionCarry {
        year: 2025,
        amount: 300000,
        eligible: true,
    });
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 10000);
    assert_eq!(value(&r, "adoption_refundable_credit"), 0);
    assert_eq!(value(&r, "adoption_nonrefundable_credit"), 3000);
}

#[test]
fn adoption_shared_limit_agreement_and_amounts_are_verified() {
    let mut d = shared_adoption_return(false);
    d.credits.as_mut().unwrap().adoptions[0].limits =
        AdoptionLimits::Shared { credit: 800000, exclusion: 500000, agreed: false };
    // Not agreed: this child's credit and exclusion are left out.
    let r = supported(&d);
    assert!(noted(&r, "WARN_ADOPTION_SHARED_LIMIT_LEFT_OUT"), "{r:?}");
    assert_eq!(value(&r, "adoption_1_credit_limit"), 0);
}

#[test]
fn personal_long_term_care_checks_qualification_the_person_and_stored_amounts() {
    let mut d = complete(FederalStatus::Single);
    set_age(&mut d, -1, 50);
    add(&mut d, income(care_business(50000, 1000, false)));
    // A premium for someone who is not on the return is refused by name.
    itemized(d.deductions.as_mut().unwrap()).long_term_care = vec![personal_care(99, 100000)];
    assert!(!d.valid_stored());
    assert_eq!(estimate(&d).state, ResultState::Incomplete);
    {
        let qualification = CareContractQualification::Nonqualified;
        let record = &mut itemized(d.deductions.as_mut().unwrap()).long_term_care[0];
        record.person = -1;
        record.qualified = qualification;
        assert!(noted(&supported(&d), "WARN_LTC_PREMIUM_LEFT_OUT"));
    }
    itemized(d.deductions.as_mut().unwrap()).long_term_care[0].qualified =
        CareContractQualification::Qualified;
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn sole_spouse_uniform_lifetime_election_needs_supported_source_facts() {
    let mut d = beneficiary_rmd(19400101, 20251201, 19610101, InheritedIraKind::Traditional, 26500);
    inherited_rmd_data(&mut d).eligibility =
        InheritedEligibility::SoleSpouseUniformLifetime { confirmed: true };
    unsupported(&d, "Inherited required distributions"); // beneficiary is under 73
    d.people.as_mut().unwrap().taxpayer.birth_date = 19530101;
    inherited_rmd_data(&mut d).owner_birth = 19570101;
    inherited_rmd_data(&mut d).died = 20221201;
    unsupported(&d, "Inherited required distributions"); // owner died before beginning
    inherited_rmd_data(&mut d).owner_birth = 19400101;
    unsupported(&d, "Inherited required distributions"); // spouse first RMD year was 2023
    // An election the custodian has not confirmed is not in effect: the
    // spouse's inherited Single Life treatment applies instead.
    inherited_rmd_data(&mut d).eligibility =
        InheritedEligibility::SoleSpouseUniformLifetime { confirmed: false };
    let unconfirmed = estimate(&d);
    inherited_rmd_data(&mut d).eligibility = InheritedEligibility::SoleSpouse;
    let single_life = estimate(&d);
    assert_eq!(
        (unconfirmed.state, unconfirmed.tax, unconfirmed.issues.len()),
        (single_life.state, single_life.tax, single_life.issues.len())
    );
}
#[test]
fn personal_debt_allocated_joint_amounts_are_summed_once_and_dates_validated() {
    let mut d = complete(FederalStatus::Joint);
    let before = supported(&d);
    add(&mut d, canceled_personal_debt(2000, PersonalDebtTreatment::Taxable));
    let mut spouse = canceled_personal_debt(3000, PersonalDebtTreatment::Taxable);
    spouse.owner = crate::DocumentOwner::Spouses;
    add(&mut d, spouse);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), value(&before, "agi") + 5000);
    if let FederalIncome::PersonalDebt { data } = &mut d.income[1].income {
        data.form.date = 20251231;
        assert!(!PersonalDebtCancellation::from(&*data).valid());
    }
}

#[test]
fn student_service_discharge_excludes_income_without_form982_or_ssn_gate() {
    for reason in [StudentDischargeReason::PublicService, StudentDischargeReason::Teacher] {
        let mut d = complete(FederalStatus::Single);
        d.people.as_mut().unwrap().taxpayer.valid_ssn = false;
        let before = supported(&d);
        add(
            &mut d,
            canceled_personal_debt(30000, PersonalDebtTreatment::StudentDischarge { reason }),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), value(&before, "agi"));
        assert_eq!(value(&r, "debt_2_student_exclusion"), 30000);
        assert!(!r.lines.iter().any(|v| v.id == "debt_2_basis_reduction"));
        assert_eq!(restart(d.clone()).draft, d);
    }
}
#[test]
fn student_death_disability_discharge_uses_debt_owners_ssn_not_spouses() {
    let mut d = complete(FederalStatus::Joint);
    let mut entry = canceled_personal_debt(
        20000,
        PersonalDebtTreatment::StudentDischarge {
            reason: StudentDischargeReason::DeathOrDisability,
        },
    );
    entry.owner = crate::DocumentOwner::Spouses;
    add(&mut d, entry);
    supported(&d);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().valid_ssn = false;
    unsupported(&d, "Student-loan discharge");
}

#[test]
fn school_discharge_safe_harbor_excludes_parent_or_student_debt_without_ssn_condition() {
    for reason in
        [StudentDischargeReason::SchoolDischarge, StudentDischargeReason::SchoolSettlement]
    {
        let mut d = complete(FederalStatus::Single);
        d.people.as_mut().unwrap().taxpayer.valid_ssn = false;
        let before = supported(&d);
        add(
            &mut d,
            canceled_personal_debt(42000, PersonalDebtTreatment::StudentDischarge { reason }),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), value(&before, "agi"));
        assert_eq!(value(&r, "total_tax"), value(&before, "total_tax"));
        assert_eq!(value(&r, "debt_2_student_exclusion"), 42000);
        assert!(
            r.review_notes
                .iter()
                .any(|v| v.contains("Rev. Proc. 2020-11") && v.contains("recapture"))
        );
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn qualifying_general_service_and_refinancing_programs_use_student_exception() {
    for lender in [
        StudentServiceLender::Government,
        StudentServiceLender::PublicHospital,
        StudentServiceLender::FundedSchool,
        StudentServiceLender::SchoolProgram,
        StudentServiceLender::RefinancingProgram,
    ] {
        let mut d = complete(FederalStatus::Single);
        let before = supported(&d);
        add(
            &mut d,
            canceled_personal_debt(
                12000,
                PersonalDebtTreatment::StudentDischarge {
                    reason: StudentDischargeReason::Service {
                        lender,
                        eligible_school: true,
                        service_terms: true,
                        prohibited_services: false,
                    },
                },
            ),
        );
        assert_eq!(value(&supported(&d), "agi"), value(&before, "agi"));
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn prior_regular_nol_preserves_current_business_loss_and_independent_amt_history() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(cash_business(-80000)));
    let before = supported(&d);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 5000), (2025, 30000)]);
    d.screening.as_mut().unwrap().atnol = prior_atnol(&[(2025, 20000, false)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 5000);
    assert_eq!(value(&r, "nol_2017_used"), 0);
    assert_eq!(value(&r, "nol_2025_used"), 0);
    assert_eq!(value(&r, "current_net_operating_loss"), 20000);
    assert_eq!(
        value(&r, "current_net_operating_loss"),
        value(&before, "current_net_operating_loss")
    );
    assert_eq!(value(&r, "regular_total_nol_carryforward"), 55000);
    assert_eq!(value(&r, "amt_total_nol_carryforward"), 40000);
}

#[test]
fn prior_nol_refigures_student_loan_deduction_and_each_vintages_absorption() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 9000000, 0)];
    let a = d.adjustments.as_mut().unwrap();
    a.set_student_loan_interest(250000);
    let before = supported(&d);
    assert_eq!(value(&before, "agi"), 88333);
    for old_year in [2016, 2019] {
        d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000), (old_year, 20000)]);
        let r = supported(&d);
        let (deduction, used, carry) =
            if old_year < 2018 { (61786, 40953, 59047) } else { (57786, 36953, 63047) };
        assert_eq!(value(&r, "prior_nol_deduction"), deduction);
        assert_eq!(value(&r, "agi"), 90000 - deduction - 2500);
        assert_eq!(value(&r, "nol_2025_used"), used);
        assert_eq!(value(&r, "nol_2025_carryforward"), carry);
        assert_eq!(value(&r, "current_net_operating_loss"), 0);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn prior_nol_capital_addback_refigures_student_loan_phaseout_for_absorption() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 9000000, 0)];
    add(&mut d, income(bad_debt(10000, 0)));
    let a = d.adjustments.as_mut().unwrap();
    a.set_student_loan_interest(250000);
    // Actual baseline: 87,000 MAGI, 2,167 loan deduction, 68,733 TI.
    // Carryover baseline: 90,000 MAGI, 1,667 loan deduction, 72,233 TI.
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 54986);
    assert_eq!(value(&r, "nol_2025_used"), 58486);
    assert_eq!(value(&r, "nol_2025_carryforward"), 41514);
    assert_eq!(value(&r, "agi"), 29514);
    assert_eq!(value(&r, "capital_short_carryforward"), 7000);
}

#[test]
fn prior_2025_cash_and_hosted_student_keep_class_and_vintage_through_full_return() {
    let mut draft = cash_carry_with_hosted_student(6_000_000, 12, 5_000);
    let result = supported(&draft);
    assert_eq!(value(&result, "agi"), 60_000);
    assert_eq!(value(&result, "charity_carry_2025_cash_sixty_used"), 5_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_deduction"), 5_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_remaining_following_year"), 0);
    assert_eq!(value(&result, "charity_hosted_student_deduction"), 300);
    assert_eq!(value(&result, "charity_hosted_student_return_year_carryforward"), 0);
    assert_eq!(value(&result, "charity_floor_not_carried"), 300);
    assert_eq!(return_line(&result, "12e"), 530_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), result);

    let mut without_prior = draft.clone();
    without_prior.deductions.as_mut().unwrap().charity_carryovers.clear();
    let current_only = supported(&without_prior);
    assert_eq!(return_line(&current_only, "12e"), 30_000);
    assert_eq!(return_line(&current_only, "15") - return_line(&result, "15"), 500_000);
    assert!(result.tax < current_only.tax);

    draft.deductions.as_mut().unwrap().itemize_override = None;
    let standard = supported(&draft);
    assert_eq!(value(&standard, "charity_carry_2025_cash_sixty_used"), 5_000);
    assert_eq!(value(&standard, "charity_prior_cash_sixty_deduction"), 0);
    assert_eq!(value(&standard, "charity_prior_cash_sixty_consumed_without_deduction"), 5_000);
    assert_eq!(value(&standard, "charity_prior_cash_sixty_remaining_following_year"), 0);
    assert_eq!(value(&standard, "charity_hosted_student_deduction"), 0);
    assert_eq!(return_line(&standard, "12f"), 0);
    assert_eq!(estimate(&restart(draft.clone()).draft), standard);
}

#[test]
fn prior_cash_with_hosted_student_charges_floor_to_for_use_class_first() {
    let draft = cash_carry_with_hosted_student(6_000_000, 2, 5_000);
    let result = supported(&draft);
    assert_eq!(value(&result, "charity_hosted_student_deduction"), 0);
    assert_eq!(value(&result, "charity_carry_2025_cash_sixty_used"), 5_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_floor_not_carried"), 200);
    assert_eq!(value(&result, "charity_prior_cash_sixty_deduction"), 4_800);
    assert_eq!(value(&result, "charity_return_year_carryforward"), 0);
    assert_eq!(return_line(&result, "12e"), 480_000);
    assert_eq!(estimate(&restart(draft).draft), result);
}

#[test]
fn prior_cash_with_hosted_student_preserves_current_thirty_percent_excess() {
    let draft = cash_carry_with_hosted_student(100_000, 10, 100);
    let result = supported(&draft);
    assert_eq!(value(&result, "charity_carry_2025_cash_sixty_used"), 100);
    assert_eq!(value(&result, "charity_prior_cash_sixty_remaining_following_year"), 0);
    assert_eq!(value(&result, "charity_hosted_student_deduction"), 295);
    assert_eq!(value(&result, "charity_hosted_student_return_year_carryforward"), 205);
    assert_eq!(value(&result, "charity_return_year_carryforward"), 205);
    assert_eq!(return_line(&result, "12e"), 39_500);
    assert_eq!(estimate(&restart(draft).draft), result);
}

#[test]
fn prior_cash_competes_with_hosted_student_under_amended_b_g_limit() {
    // At $60,000 AGI the old (G) cash competes with (B) student expenses.
    // Section 170(b)(1)(B)(ii) restores up to $6,000 cash to its 50% test.
    for (carry, used, remaining, student, student_carry, automatic_carry) in
        [(35_600, 35_600, 0, 100, 500, 500), (40_000, 36_000, 4_000, 0, 600, 600)]
    {
        let mut draft = cash_carry_with_hosted_student(6_000_000, 12, carry);
        let itemized_result = supported(&draft);
        assert_eq!(value(&itemized_result, "charity_carry_2025_cash_sixty_used"), used);
        assert_eq!(
            value(&itemized_result, "charity_prior_cash_sixty_remaining_following_year"),
            remaining
        );
        assert_eq!(value(&itemized_result, "charity_hosted_student_deduction"), student);
        assert_eq!(
            value(&itemized_result, "charity_hosted_student_return_year_carryforward"),
            student_carry
        );
        if remaining == 0 {
            assert_eq!(value(&itemized_result, "charity_return_year_carryforward"), 500);
        } else {
            assert_eq!(value(&itemized_result, "charity_prior_cash_sixty_deduction"), 35_700);
            assert_eq!(value(&itemized_result, "charity_prior_cash_sixty_floor_not_carried"), 300);
        }
        assert_eq!(return_line(&itemized_result, "12e"), 3_570_000);
        assert_eq!(estimate(&restart(draft.clone()).draft), itemized_result);
        // 2026-11-02: automatic selection retains the same itemized facts and limits.
        draft.deductions.as_mut().unwrap().itemize_override = None;
        let automatic = supported(&draft);
        if remaining == 0 {
            assert_eq!(value(&automatic, "charity_carry_2025_cash_sixty_used"), 35_600);
        }
        assert_eq!(value(&automatic, "charity_prior_cash_sixty_consumed_without_deduction"), 0);
        assert_eq!(
            value(&automatic, "charity_prior_cash_sixty_remaining_following_year"),
            remaining
        );
        assert_eq!(
            value(&automatic, "charity_hosted_student_return_year_carryforward"),
            automatic_carry
        );
        assert_eq!(return_line(&automatic, "12f"), 0);
        assert_eq!(estimate(&restart(draft).draft), automatic);
    }
}

#[test]
fn prior_cash_with_hosted_student_requires_complete_sources() {
    let draft = cash_carry_with_hosted_student(6_000_000, 12, 5_000);
    let mut competing = draft.clone();
    competing.deductions.as_mut().unwrap().charity_carryovers[0].balance = 3_000_000;
    let competing_result = supported(&competing);
    assert_eq!(value(&competing_result, "charity_carry_2025_cash_sixty_used"), 30_000);
    assert_eq!(value(&competing_result, "charity_hosted_student_deduction"), 300);
    assert_eq!(return_line(&competing_result, "12e"), 3_030_000);

    let mut older = draft.clone();
    older.deductions.as_mut().unwrap().charity_carryovers[0].year = 2024;
    assert_eq!(value(&supported(&older), "charity_deduction"), 5_300);

    let mut unconfirmed_student = draft.clone();
    unconfirmed_student.deductions.as_mut().unwrap().hosted_students[0].documented = false;
    assert!(noted(&supported(&unconfirmed_student), "WARN_HOSTED_STUDENT_LEFT_OUT"));

    let mut changed_filer = draft.clone();
    changed_filer.deductions.as_mut().unwrap().charity_carryovers[0].same_filers = false;
    assert!(noted(&supported(&changed_filer), "WARN_CHARITY_CARRY_LEFT_OUT"));

    let mut current_cash = draft.clone();
    current_cash.deductions.as_mut().unwrap().set_cash_charity(100_000);
    supported(&current_cash);

    let mut mixed_property = draft.clone();
    mixed_property
        .deductions
        .as_mut()
        .unwrap()
        .property_gifts
        .push(ordinary_property_for_cash_carry(10_000));
    assert_eq!(value(&supported(&mixed_property), "charity_deduction"), 5_400);

    let mut another_student = draft;
    let second = another_student.deductions.as_ref().unwrap().hosted_students[0].clone();
    another_student
        .deductions
        .as_mut()
        .unwrap()
        .hosted_students
        .push(HostedStudent { label: "Second synthetic hosted student".into(), ..second });
    assert_eq!(value(&supported(&another_student), "charity_deduction"), 5_900);
}

#[test]
fn prior_nol_restores_employer_adoption_exclusion_after_limit_calculation() {
    let mut d = nol_adoption();
    assert_eq!(value(&supported(&d), "adoption_benefits_excluded"), 8835);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 300000)]);
    let r = supported(&d);
    // Before the NOL: $276,245 AGI less $16,100 standard deduction.
    assert_eq!(value(&r, "prior_nol_deduction"), 208116);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 17670);
    assert_eq!(value(&r, "adoption_exclusion_magi"), 76964);
    assert_eq!(return_line(&r, "1f"), 233000);
    assert_eq!(value(&r, "agi"), 59294);
    assert_eq!(value(&r, "nol_2025_carryforward"), 91884);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_adoption_exclusion_refigures_later_vintage_absorption() {
    let mut d = nol_adoption();
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 20000), (2025, 300000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 212116);
    // The earlier loss restores the full exclusion before newer-loss use:
    // $265,080 + $2,330 - $20,000 - $16,100 - $48,029 reserved income.
    assert_eq!(value(&r, "nol_2025_used"), 183281);
    assert_eq!(value(&r, "nol_2025_carryforward"), 116719);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 17670);
    assert_eq!(value(&r, "agi"), 55294);
}

#[test]
fn prior_nol_preserves_special_needs_negative_adoption_wages() {
    let mut d = complete(FederalStatus::Single);
    employer_adoption(&mut d, 0, 0, true);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    let r = supported(&d);
    assert_eq!(return_line(&r, "1f"), -1767000);
    assert_eq!(value(&r, "prior_nol_deduction"), 20984);
    assert_eq!(value(&r, "agi"), 21346);
    assert_eq!(value(&r, "nol_2025_carryforward"), 79016);
    assert_eq!(value(&r, "adoption_refundable_credit"), 5120);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_alimony_recapture_adjustment_precedes_loss_limit() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Alimony { data: recapture_alimony() }));
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "alimony_recapture_deduction"), 1500);
    assert_eq!(value(&r, "prior_nol_deduction"), 56320);
    assert_eq!(value(&r, "agi"), 30180);
    assert_eq!(value(&r, "nol_2025_carryforward"), 43680);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn section_1041_spouse_transfer_does_not_recapture_mortgage_subsidy() {
    let mut d = complete(FederalStatus::Joint);
    let ordinary_return = supported(&d);
    set_mortgage_subsidy(
        &mut d,
        MortgageSubsidyDisposition::ExemptSection1041 {
            recipient: MortgageSubsidySpouseRecipient::CurrentSpouse,
            disposed_on: 20260701,
            qualifies_without_recognized_gain_or_loss: true,
        },
    );
    let spouse = supported(&d);
    assert_eq!(value(&spouse, "mortgage_subsidy_recapture_tax"), 0);
    assert_eq!(spouse.tax, ordinary_return.tax);
    assert_eq!(supported(&restart(d.clone()).draft), spouse);

    if let MortgageSubsidyDisposition::ExemptSection1041 { recipient, .. } =
        mortgage_disposition(&mut d)
    {
        *recipient = MortgageSubsidySpouseRecipient::FormerSpouseIncidentToDivorce;
    } else {
        panic!()
    }
    let former = supported(&d);
    assert_eq!(value(&former, "mortgage_subsidy_recapture_tax"), 0);
    assert_eq!(former.tax, ordinary_return.tax);
    if let MortgageSubsidyDisposition::ExemptSection1041 {
        qualifies_without_recognized_gain_or_loss,
        ..
    } = mortgage_disposition(&mut d)
    {
        *qualifies_without_recognized_gain_or_loss = false;
    }
    unsupported(&d, "Mortgage subsidy recapture");
    if let MortgageSubsidyDisposition::ExemptSection1041 {
        qualifies_without_recognized_gain_or_loss,
        disposed_on,
        ..
    } = mortgage_disposition(&mut d)
    {
        *qualifies_without_recognized_gain_or_loss = true;
        *disposed_on = 20270201;
    }
    assert!(!d.valid_stored());
}

#[test]
fn nonspouse_gift_of_subsidized_home_uses_fair_market_value_for_form_8828() {
    let mut d = complete(FederalStatus::Single);
    let ordinary_return = supported(&d);
    set_mortgage_subsidy(
        &mut d,
        MortgageSubsidyDisposition::Ordinary {
            subsidy: MortgageSubsidyKind::Mcc,
            transfer: MortgageSubsidyTransfer::GiftToOther {
                owners: None,
                fair_market_value: 20_000_000,
                adjusted_basis: 10_000_000,
                no_consideration_or_debt_relief: true,
            },
            facts: ordinary_subsidy_facts(),
        },
    );
    let gifted = supported(&d);
    assert_eq!(value(&gifted, "mortgage_subsidy_1_disposition_gain"), 100000);
    assert_eq!(value(&gifted, "mortgage_subsidy_1_modified_income"), 60000);
    assert_eq!(value(&gifted, "mortgage_subsidy_recapture_tax"), 3750);
    assert_eq!(value(&gifted, "agi"), value(&ordinary_return, "agi"));
    assert_eq!(value(&gifted, "capital_gain"), value(&ordinary_return, "capital_gain"));
    assert_eq!(gifted.tax, ordinary_return.tax.map(|tax| tax + 375_000));
    assert_eq!(supported(&restart(d.clone()).draft), gifted);

    let MortgageSubsidyDisposition::Ordinary {
        transfer: MortgageSubsidyTransfer::GiftToOther { no_consideration_or_debt_relief, .. },
        ..
    } = mortgage_disposition(&mut d)
    else {
        panic!()
    };
    *no_consideration_or_debt_relief = false;
    unsupported(&d, "Mortgage subsidy recapture");
    if let MortgageSubsidyDisposition::Ordinary { facts, .. } = mortgage_disposition(&mut d) {
        facts.original_closed_on = 20170115;
    }
    unsupported(&d, "Mortgage subsidy recapture");
    if let MortgageSubsidyDisposition::Ordinary { facts, .. } = mortgage_disposition(&mut d) {
        facts.original_closed_on = 20200115;
    }
    let MortgageSubsidyDisposition::Ordinary {
        transfer: MortgageSubsidyTransfer::GiftToOther { no_consideration_or_debt_relief, .. },
        ..
    } = mortgage_disposition(&mut d)
    else {
        panic!()
    };
    *no_consideration_or_debt_relief = true;

    let MortgageSubsidyDisposition::Ordinary {
        transfer: MortgageSubsidyTransfer::GiftToOther { fair_market_value, .. },
        ..
    } = mortgage_disposition(&mut d)
    else {
        panic!()
    };
    *fair_market_value = 5_000_000;
    let gift_loss = supported(&d);
    assert_eq!(value(&gift_loss, "mortgage_subsidy_1_disposition_gain"), -50000);
    assert_eq!(value(&gift_loss, "mortgage_subsidy_recapture_tax"), 0);
    assert_eq!(gift_loss.tax, ordinary_return.tax);
}
#[test]
fn mixed_note_gain_rounding_conserves_total_and_is_independent_of_payment_order() {
    let mut source = mixed_note_repayments();
    source.loans[0].principal = 300;
    source.loans[0].basis = 200;
    source.loans[0].repayments = 300;
    let ScorpNoteHolding::Dated { payments, .. } = &mut source.loans[0].repayment_term else {
        panic!()
    };
    payments[0].principal = 100;
    payments[1].principal = 200;
    source.risk.start = ScorpRiskStart::Detailed { capacity: 200, profit: 0, loss: 0 };
    source.amt = Some(amt_records(&source));
    let first = source.loan_gains(&source.basis_result().unwrap()).unwrap();
    // 2026-10-19: loan_gains retains actual cents; annual projection below rounds once.
    assert_eq!(first, [0, 33, 67]);
    let ScorpNoteHolding::Dated { payments, .. } = &mut source.loans[0].repayment_term else {
        panic!()
    };
    payments.reverse();
    let second = source.loan_gains(&source.basis_result().unwrap()).unwrap();
    assert_eq!(first, second);
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Scorp { data: scorp_source(source) }));
    assert_eq!(value(&supported(&d), "capital_gain"), 1);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn former_passive_partial_character_release_uses_independent_amt_income() {
    let mut source = former_passive_property_scorp(8_000, 2_000, 6_000, 14_000);
    let alternate = source.amt.as_mut().unwrap();
    alternate.adjustment = -600_000;
    alternate.section1231.as_mut().unwrap().net = 800_000;
    let regular = source.pool_activity_result().unwrap().characters.unwrap();
    let amt = source
        .amt
        .as_ref()
        .unwrap()
        .source(&source)
        .unwrap()
        .pool_activity_result()
        .unwrap()
        .characters
        .unwrap();
    // 2026-10-19: activity character packets retain actual cents in both systems.
    assert_eq!(regular.prior_released, [600_000, 400_000]);
    assert_eq!(amt.prior_released, [200_000, 800_000]);
}

#[test]
fn former_passive_mixed_sign_release_has_independent_amt_history() {
    let mut source = former_passive_property_scorp(-5_000, 10_000, 6_000, 4_000);
    let amt = source.amt.as_mut().unwrap();
    amt.adjustment = 1_500_000;
    amt.section1231.as_mut().unwrap().net = -500_000;
    let c = source
        .amt
        .as_ref()
        .unwrap()
        .source(&source)
        .unwrap()
        .pool_activity_result()
        .unwrap()
        .characters
        .unwrap();
    // 2026-10-19: activity character packets retain actual cents.
    assert_eq!(c.prior_released, [500_000, 0]);
    assert_eq!(c.passive, [-100_000, -400_000]);
    assert_eq!(c.nonpassive, [500_000, -500_000]);
}

#[test]
fn property_child_income_recapture_reconciles_with_and_without_wages() {
    // Form 8615 line 1 uses AGI without earnings, otherwise total income
    // less earnings. Ordinary section 1231 recapture belongs in both paths.
    for (wages, taxable, tax) in [(0, 18650, 3942), (5000, 19550, 4032)] {
        let mut draft = complete_property_gain_draft(false, 20000);
        let young = young_interest_return();
        draft.people = young.people;
        if let FederalIncome::Wage { data } = &mut draft.income[0].income {
            let mut data = edit_wage(data);
            data.wages = wages * 100;
            data.withholding = 0;
        }
        for entry in &mut draft.income {
            if let FederalIncome::Scorp { data } = &mut entry.income {
                let mut data = edit_scorp(data);
                data.property_qbi = property_qbi(0, 0);
            }
        }
        let r = supported(&draft);
        assert_eq!(value(&r, "agi"), 20000 + wages);
        assert_eq!(value(&r, "taxable_income"), taxable);
        assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 17300);
        // Parent's entire $17,300 increment stays in the 22% joint bracket.
        assert_eq!(value(&r, "kiddie_tax_parent_share"), 3806);
        assert_eq!(value(&r, "regular_income_tax"), tax);
        assert_eq!(return_line(&r, "7a"), 0);
        assert_eq!(return_line(&r, "8"), 2000000);
        assert_eq!(return_line(&r, "16"), tax * 100);
        assert!(r.tax.is_some());
        assert_eq!(restart(draft.clone()).draft, draft);
    }
}

#[test]
fn property_child_income_loss_reduces_unearned_income_once() {
    let mut draft = young_interest_return();
    draft.income.clear();
    add(&mut draft, wage(0, Owner::Taxpayer, 500000, 0));
    add(&mut draft, income(savings_income(30000, 0)));
    let mut source = former_passive_property_scorp(0, -20000, 0, 0);
    source.property_qbi = property_qbi(20000, 0);
    source.qbi_statement = complete_ordinary_scorp(0, true).qbi_statement;
    source.amt = Some(amt_records(&source));
    add(&mut draft, income(FederalIncome::Scorp { data: scorp_source(source) }));
    draft.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let r = supported(&draft);
    assert_eq!(value(&r, "agi"), 15000);
    assert_eq!(value(&r, "taxable_income"), 9550);
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 7300);
    assert_eq!(value(&r, "kiddie_tax_parent_share"), 1606);
    assert_eq!(value(&r, "regular_income_tax"), 1832);
    assert_eq!(return_line(&r, "8"), -2000000);
    assert_eq!(return_line(&r, "16"), 183200);
    assert_eq!(value(&r, "section1231_2026_next_lookback"), 20000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 20000);
    assert!(r.tax.is_some());
    assert_eq!(restart(draft.clone()).draft, draft);
}

#[test]
fn property_excess_loss_reconciles_qbi_nol_and_independent_amt_histories() {
    for (qualified, current_qbi, deferred_qbi) in
        [(400000, 256000, 144000), (200000, 128000, 72000), (0, 0, 0)]
    {
        let draft = property_excess_draft(400000, 300000, 500000, qualified, FederalStatus::Single);
        let r = supported(&draft);
        assert_eq!(value(&r, "agi"), -196000);
        assert_eq!(value(&r, "excess_business_loss_carryforward"), 144000);
        assert_eq!(value(&r, "amt_excess_business_loss_carryforward"), 44000);
        assert_eq!(value(&r, "qbi_loss_carryforward"), current_qbi);
        // Current NOL $196,000 plus each tax system's own excess loss.
        assert_eq!(value(&r, "nol_carryforward"), 340000);
        assert_eq!(value(&r, "amt_nol_carryforward"), 240000);
        assert_eq!(value(&r, "section1231_2026_next_lookback"), 256000);
        assert_eq!(value(&r, "amt_section1231_2026_next_lookback"), 256000);
        assert_eq!(return_line(&r, "8"), -25600000);
        assert_eq!(r.tax, Some(0));
        assert!(r.review_notes.iter().any(|n| n.starts_with(&alloc::format!(
            "K-1 excess-loss character: NOL carryforward $144000; qualified portion ${deferred_qbi}."
        ))));
        assert_eq!(restart(draft.clone()).draft, draft);
    }
}

#[test]
fn property_excess_loss_addback_reaches_child_income_with_wages() {
    for (wages, tax) in [(0, 9222), (5000, 9312)] {
        let mut draft =
            property_excess_draft(400000, 300000, 500000, 400000, FederalStatus::Single);
        draft.people = young_interest_return().people;
        if let FederalIncome::Wage { data } = &mut draft.income[0].income {
            let mut data = edit_wage(data);
            data.wages = wages * 100;
            data.withholding = 0;
        }
        add(&mut draft, income(savings_income(300000, 0)));
        let r = supported(&draft);
        assert_eq!(value(&r, "agi"), 44000 + wages);
        // Form 1040 includes the $144,000 loss addback. Line 1 cannot
        // subtract the gross $400,000 loss again when earnings are present.
        assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 41300);
        assert_eq!(value(&r, "kiddie_tax_parent_share"), 9086);
        assert_eq!(value(&r, "regular_income_tax"), tax);
        assert_eq!(return_line(&r, "16"), tax * 100);
        assert_eq!(r.tax, Some(tax * 100));
    }
}

#[test]
fn multiple_property_excess_losses_check_both_spouses_and_reject_gains() {
    let mut draft = property_excess_draft(400000, 300000, 500000, 400000, FederalStatus::Joint);
    let mut other = property_excess_draft(400000, 300000, 500000, 200000, FederalStatus::Joint);
    let mut entry = other.income.remove(1);
    entry.owner = crate::DocumentOwner::Spouses;
    add(&mut draft, entry);
    let r = supported(&draft);
    assert_eq!(value(&r, "excess_business_loss_carryforward"), 288000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 384000);
    assert_eq!(value(&r, "amt_excess_business_loss_carryforward"), 88000);
    assert_eq!(value(&r, "section1231_2026_next_lookback"), 512000);
    assert_eq!(value(&r, "amt_section1231_2026_next_lookback"), 512000);
    assert_eq!(r.tax, Some(0));
    assert_eq!(restart(draft.clone()).draft, draft);
    {
        let FederalIncome::Scorp { data } = &mut draft.income[2].income else { unreachable!() };
        let mut data = edit_scorp(data);
        data.section1231.as_mut().unwrap().net = 1000000;
        data.property_qbi = property_qbi(0, 10000);
    }
    // The remaining loss still exceeds $512k only with a larger first loss.
    {
        let FederalIncome::Scorp { data } = &mut draft.income[1].income else { unreachable!() };
        let mut data = edit_scorp(data);
        data.section1231.as_mut().unwrap().net = -70000000;
        data.property_qbi = property_qbi(700000, 0);
        data.opening = Some(80000000);
        data.risk.start = ScorpRiskStart::Detailed { capacity: 80000000, profit: 0, loss: 0 };
    }
    unsupported(&draft, "Property excess business loss");
}

#[test]
fn household_return_adds_employer_tax_and_employee_withholding_only_once() {
    for (disability, expected, tax, balance, extra_tax) in [
        (
            false,
            &[("household_0_social_tax", 1240), ("household_0_medicare_tax", 290)][..],
            709500,
            59500,
            207200,
        ),
        (
            true,
            &[
                ("household_0_social_wages", 12000),
                ("household_0_social_tax", 1488),
                ("household_0_medicare_tax", 348),
                ("household_0_state_disability_tax_offset", 153),
            ][..],
            674800,
            24800,
            172500,
        ),
    ] {
        let mut e = household_employer(10000);
        if disability {
            e.workers[0].disability = Some(household_disability(2000, 124, 29));
        } else {
            e.workers[0].withholding = 50000;
        }
        let d = with_household(e);
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 60000);
        assert_eq!(value(&r, "household_0_futa_tax"), 42);
        for &(key, amount) in expected {
            assert_eq!(value(&r, key), amount);
        }
        assert_eq!(return_line(&r, "23"), extra_tax);
        assert_eq!(return_line(&r, "25d"), 650000);
        assert_eq!(r.tax, Some(tax));
        assert_eq!(r.balance, Some(balance));
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn household_cash_thresholds_use_exact_cents_and_prior_quarter_facts() {
    let mut e = household_employer(3000);
    let r = supported(&with_household(e.clone()));
    assert_eq!(value(&r, "household_0_social_tax"), 372);
    assert_eq!(value(&r, "household_0_medicare_tax"), 87);
    e.workers[0].pay[0].cash -= 1;
    let r = supported(&with_household(e.clone()));
    assert_eq!(value(&r, "household_0_social_tax"), 0);
    assert_eq!(value(&r, "household_0_medicare_tax"), 0);
    e.workers[0].pay = [20260331, 20260630, 20260930, 20261231]
        .into_iter()
        .map(|date| household_pay(500, date))
        .collect();
    let r = supported(&with_household(e.clone()));
    assert_eq!(value(&r, "household_0_tax"), 0);
    e.prior_quarters[2].cash = 100000;
    let r = supported(&with_household(e));
    assert_eq!(value(&r, "household_0_futa_tax"), 12);
    assert_eq!(r.tax, Some(503500));
}

#[test]
fn household_relationships_keep_fica_and_futa_exceptions_distinct() {
    for (kind, ss, med, futa) in [
        (HouseholdWageKind::Spouse, 0, 0, 0),
        (HouseholdWageKind::ChildUnder21, 0, 0, 0),
        (HouseholdWageKind::Parent { child_care: true, marital_condition: false }, 0, 0, 0),
        (HouseholdWageKind::Parent { child_care: true, marital_condition: true }, 1240, 290, 0),
        (HouseholdWageKind::Minor { student: true, principal_occupation: true }, 0, 0, 42),
        (HouseholdWageKind::Minor { student: false, principal_occupation: true }, 1240, 290, 42),
    ] {
        let mut e = household_employer(10000);
        e.workers[0].pay[0].treatment = kind;
        let d = with_household(e);
        let r = supported(&d);
        assert_eq!(value(&r, "household_0_social_tax"), ss);
        assert_eq!(value(&r, "household_0_medicare_tax"), med);
        assert_eq!(value(&r, "household_0_futa_tax"), futa);
        assert_eq!(r.tax, Some((5023 + ss + med + futa) * 100));
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn household_wage_caps_apply_per_employee_not_to_employer_earnings() {
    let mut e = household_employer(200000);
    let mut second = e.workers[0].clone();
    second.label = "Second worker".into();
    second.pay[0].cash = 21000000;
    e.workers.push(second);
    let r = supported(&with_household(e));
    assert_eq!(value(&r, "household_0_social_tax"), 45756);
    assert_eq!(value(&r, "household_0_medicare_tax"), 11890);
    assert_eq!(value(&r, "household_0_additional_medicare"), 90);
    assert_eq!(value(&r, "additional_medicare_tax"), 0);
    assert_eq!(value(&r, "household_0_futa_tax"), 84);
    assert_eq!(return_line(&r, "23"), 5782000);
    assert_eq!(r.tax, Some(6284300));
}

#[test]
fn household_late_state_contributions_receive_only_ninety_percent_credit() {
    let mut e = household_employer(10000);
    e.states[0].periods[0].wages = 700000;
    e.states[0].periods[0].rate = Some(2.7);
    e.states[0].required = 18900;
    e.states[0].payments = vec![HouseholdStatePayment { date: 20270415, amount: 18900 }];
    assert_eq!(value(&supported(&with_household(e.clone())), "household_0_futa_tax"), 42);
    e.states[0].payments[0].date = 20270416;
    let r = supported(&with_household(e.clone()));
    // 420 gross - (189 experience credit + round(189 * .90)) = 61.
    assert_eq!(value(&r, "household_0_futa_tax"), 61);
    assert_eq!(r.tax, Some(661400));
    e.states[0].reduction = Some(0.3);
    assert_eq!(value(&supported(&with_household(e)), "household_0_futa_tax"), 82);
}

#[test]
fn household_futa_allocates_first_seven_thousand_across_states_by_payment_date() {
    let mut e = household_employer(10000);
    e.workers[0].pay = vec![household_pay(6000, 20260630), household_pay(4000, 20260331)];
    e.workers[0].pay[0].state = "CA".into();
    e.states[0].periods[0].wages = 400000;
    e.states[0].reduction = Some(0.3);
    let mut ca = e.states[0].clone();
    ca.state = "CA".into();
    ca.periods[0].wages = 600000;
    ca.reduction = Some(1.2); // Synthetic rate, not a claim of final 2026 California law.
    e.states.push(ca);
    let r = supported(&with_household(e.clone()));
    // Base 42 + 4,000 * .003 + 3,000 * .012 = 90.
    assert_eq!(value(&r, "household_0_futa_tax"), 90);
    e.workers[0].pay[0].state_covered = false;
    e.states.pop();
    // Only 4,000 state wages: 216 credit less 12 reduction; 420 - 204 = 216.
    assert_eq!(value(&supported(&with_household(e)), "household_0_futa_tax"), 216);
}

#[test]
fn household_unknown_rates_and_incomplete_records_do_not_silently_mean_zero() {
    let mut e = household_employer(10000);
    e.states[0].reduction = None;
    let conservative = supported(&with_household(e.clone()));
    assert_eq!(value(&conservative, "household_0_futa_tax"), 420);
    assert_eq!(value(&conservative, "household_0_futa_credit"), 0);
    let notes: Vec<_> = conservative
        .review_notes
        .iter()
        .filter(|note| note.display.key == "WARN_HOUSEHOLD_FUTA_CREDIT_ASSUMED_NONE")
        .collect();
    assert_eq!(notes.len(), 1);
    assert_eq!(
        crate::interview::issue_page(
            &with_household(e.clone()),
            notes[0].context.as_ref().unwrap(),
            None
        ),
        Some(TaxSection::OtherTaxes)
    );
    e.states[0].reduction = Some(0.0);
    let final_rate = supported(&with_household(e.clone()));
    assert_eq!(value(&final_rate, "household_0_futa_tax"), 42);
    assert!(!noted(&final_rate, "WARN_HOUSEHOLD_FUTA_CREDIT_ASSUMED_NONE"));
    e.complete = false;
    unsupported(&with_household(e.clone()), "Household employment");
    e.complete = true;
    e.states.clear();
    unsupported(&with_household(e.clone()), "Household employment");
    e.workers[0].pay[0].state_covered = false;
    assert_eq!(value(&supported(&with_household(e)), "household_0_futa_tax"), 420);
}

#[test]
fn household_employer_ownership_and_source_periods_validate_on_restore() {
    let e = household_employer(10000);
    let mut d = with_household(e.clone());
    d.screening.as_mut().unwrap().household_employment[0].owner = Owner::Spouse;
    unsupported(&d, "Household employment");
    let mut joint = complete(FederalStatus::Joint);
    let mut spouse = e.clone();
    spouse.owner = Owner::Spouse;
    joint.screening.as_mut().unwrap().household_employment = vec![e.clone(), spouse];
    let r = supported(&joint);
    assert_eq!(value(&r, "household_0_tax"), 1572);
    assert_eq!(value(&r, "household_1_tax"), 1572);
    assert_eq!(return_line(&r, "23"), 314400);
    assert!(crate::household_employment::valid(core::slice::from_ref(&e)));
    assert!(!crate::household_employment::valid(&[e.clone(), e.clone()]));
    let mut invalid = e;
    let duplicate = invalid.states[0].periods[0].clone();
    invalid.states[0].periods.push(duplicate);
    assert!(!crate::household_employment::valid(&[invalid]));
}

#[test]
fn household_changed_state_rates_and_unassigned_rate_use_source_credits() {
    let mut e = household_employer(10000);
    e.states[0].required = 56700;
    e.states[0].periods = vec![
        HouseholdStatePeriod { starts: 20260101, ends: 20260630, wages: 350000, rate: Some(2.7) },
        HouseholdStatePeriod { starts: 20260701, ends: 20261231, wages: 350000, rate: Some(5.4) },
    ];
    e.states[0].payments = vec![HouseholdStatePayment { date: 20270415, amount: 20000 }];
    // Experience difference 189 - 95 = 94; credit 294; gross 420 -> 126.
    let r = supported(&with_household(e.clone()));
    assert_eq!(value(&r, "household_0_futa_credit"), 294);
    assert_eq!(value(&r, "household_0_futa_tax"), 126);
    e.states[0].periods[0].rate = None;
    // No assigned rate means no invented experience credit, leaving actual 200 paid.
    assert_eq!(value(&supported(&with_household(e)), "household_0_futa_tax"), 220);
}

#[test]
fn household_age_change_and_missing_state_data_below_futa_threshold() {
    let mut e = household_employer(10000);
    let mut before = household_pay(10000, 20260201);
    before.treatment = HouseholdWageKind::ChildUnder21;
    e.workers[0].pay = vec![before, household_pay(3000, 20261101)];
    let r = supported(&with_household(e));
    assert_eq!(value(&r, "household_0_social_wages"), 3000);
    assert_eq!(value(&r, "household_0_social_tax"), 372);
    assert_eq!(value(&r, "household_0_futa_wages"), 3000);
    assert_eq!(value(&r, "household_0_futa_tax"), 18);
    let mut e = household_employer(999);
    e.states.clear();
    // No FUTA in either year and no FICA: unavailable state rates do not affect tax.
    let r = supported(&with_household(e));
    assert_eq!(value(&r, "household_0_tax"), 0);
    assert_eq!(r.tax, Some(502300));
}

#[test]
fn household_single_state_section_a_uses_its_own_rounding() {
    let e = household_employer(1009);
    let r = supported(&with_household(e));
    // Section A: round(1,009 * .006) = 6, not the 7 produced by
    // subtracting separately rounded 6% and 5.4% Section B lines.
    assert_eq!(value(&r, "household_0_futa_tax"), 6);
    assert_eq!(r.tax, Some(502900));
}

#[test]
fn household_disability_payments_share_chronological_futa_base_with_regular_pay() {
    let mut e = household_employer(6000);
    e.workers[0].pay[0].date = 20260630;
    e.workers[0].pay[0].state = "CA".into();
    let mut notice = household_disability(2000, 124, 29);
    reported_sick_pay(&mut notice)[0].date = 20260331;
    e.workers[0].disability = Some(notice);
    e.states[0].periods[0].wages = 200000;
    e.states[0].reduction = Some(0.3);
    let mut ca = e.states[0].clone();
    ca.state = "CA".into();
    ca.periods[0].wages = 600000;
    ca.reduction = Some(1.2); // Synthetic worksheet parameters, not final 2026 rates.
    e.states.push(ca);
    let r = supported(&with_household(e));
    assert_eq!(value(&r, "household_0_futa_wages"), 7000);
    assert_eq!(value(&r, "household_0_futa_credit_reduction"), 66);
    assert_eq!(value(&r, "household_0_futa_tax"), 108);
    assert_eq!(value(&r, "household_0_state_disability_tax_offset"), 153);
    assert_eq!(r.tax, Some(620200));
}

#[test]
fn household_disability_notice_completeness_and_withholding_must_reconcile() {
    let mut e = household_employer(10000);
    let mut notice = household_disability(2000, 124, 29);
    notice.complete = false;
    e.workers[0].disability = Some(notice);
    unsupported(&with_household(e.clone()), "Household disability notice");
    let notice = e.workers[0].disability.as_mut().unwrap();
    notice.complete = true;
    notice.medicare_withheld = 50000;
    unsupported(&with_household(e.clone()), "Household disability notice");
    let notice = e.workers[0].disability.as_mut().unwrap();
    notice.medicare_withheld = 2900;
    notice.social_withheld = 20000;
    unsupported(&with_household(e), "Household disability notice");
}

#[test]
fn household_disability_preserves_actual_per_payment_rounding() {
    let mut e = household_employer(10000);
    let mut payment = household_pay(0, 20260630);
    payment.cash = 61;
    let notice = HouseholdDisability {
        wages: HouseholdSickWages::Reported { pay: vec![payment; 20] },
        social_withheld: 80,
        medicare_withheld: 20,
        complete: true,
    };
    // Twenty 61-cent payments each with 4 cents SS and 1 cent Medicare.
    // Annual multiplication would yield only 76/18 cents; retain the actual $1.
    e.workers[0].disability = Some(notice);
    let r = supported(&with_household(e));
    assert_eq!(value(&r, "household_0_state_disability_tax_offset"), 1);
    assert_eq!(value(&r, "household_0_social_tax"), 1241);
    assert_eq!(value(&r, "household_0_medicare_tax"), 290);
    assert_eq!(r.tax, Some(659500));
}

#[test]
fn historical_childless_eic_worksheet_preserves_earned_income_when_agi_is_lower() {
    assert_eq!(crate::historical_eic::amount(2025, 10000, 5000, false), Some(649));
    assert_eq!(crate::historical_eic::amount(2025, 15000, 11000, false), Some(312));
    assert_eq!(crate::historical_eic::amount(2025, 10000, 15000, false), Some(312));
    assert_eq!(crate::historical_eic::amount(2025, 0, 15000, false), Some(0));
}

#[test]
fn partner_sep_missing_source_keeps_record_identity_and_actual_editor() {
    let mut draft = complete(FederalStatus::Single);
    for label in ["Acme partnership", "Acme partnership"] {
        let mut partner = partner_health_source(0, false);
        partner.pensions = 300000;
        partner.retirement = PartnerRetirement::Sep { rate: 25.0 };
        let mut entry = income(FederalIncome::Partnership { data: partner });
        entry.label = label.into();
        add(&mut draft, entry);
    }
    let mut result = supported(&complete(FederalStatus::Single));
    // The benefit inputs are valid; the missing resolved K-1 earnings source
    // must send the person back to that document, preserving its own identity.
    crate::sep::calculate(&draft, [0; 2], None, &[], &mut result);
    let issues: Vec<_> = result.issues.iter().filter(|issue|
        issue.display.key == "ISSUE_RESOLVE_ACTUAL_PARTNERSHIP_SE_EARNINGS_BEFORE_CALCULATING_PLAN_COMP_84586D00"
    ).collect();
    assert_eq!(issues.len(), 2);
    assert_ne!(issues[0].context.identity(), issues[1].context.identity());
    for (issue, entry) in issues.iter().zip(
        draft
            .income
            .iter()
            .filter(|entry| matches!(entry.income, FederalIncome::Partnership { .. })),
    ) {
        assert_eq!(issue.context.source(), &crate::interview::income_issue_context(entry));
        assert_eq!(
            crate::interview::issue_page(&draft, &issue.context, Some(&issue.display.key)),
            Some(TaxSection::Documents)
        );
        assert_eq!(
            issue.context.label().args,
            alloc::vec![crate::MessageArg::Text("Acme partnership".into())]
        );
    }
}

#[test]
fn adverse_second_household_combines_expenses_with_one_general_limit() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_pay(200000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    // $20,000 at the work location plus $30,000 for a qualifying dependent
    // household: one $39,870 cap and one $21,264 base, not two allowances.
    employee_housing(&mut d, 50000, ForeignHousingLocation::General);
    let ForeignHousing::Claim { household, .. } = &mut claim_mut(&mut d, Owner::Taxpayer).housing
    else {
        unreachable!()
    };
    *household = ForeignHousingHousehold::Adverse { confirmed: true };
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_housing_exclusion"), 18606);
    assert_eq!(value(&r, "agi"), 48494);
    assert_eq!(supported(&restart(d.clone()).draft), r);
    let ForeignHousing::Claim { household, .. } = &mut claim_mut(&mut d, Owner::Taxpayer).housing
    else {
        unreachable!()
    };
    *household = ForeignHousingHousehold::Adverse { confirmed: false };
    // Unconfirmed: the housing claim is left out with a notice.
    let r = supported(&d);
    assert!(noted(&r, "WARN_FOREIGN_HOUSING_LEFT_OUT"), "{r:?}");
    assert_eq!(value(&r, "foreign_housing_exclusion"), 0);
}

#[test]
fn adverse_second_household_in_one_high_cost_locality_uses_one_cap() {
    let mut draft = empty_return(FederalStatus::Single);
    add(&mut draft, income(foreign_pay(200000)));
    set_claim(&mut draft, foreign_claim(Owner::Taxpayer));
    employee_housing(&mut draft, 200000, ForeignHousingLocation::ChinaHongKong);
    let ForeignHousing::Claim { household, .. } =
        &mut claim_mut(&mut draft, Owner::Taxpayer).housing
    else {
        unreachable!()
    };
    *household = ForeignHousingHousehold::AdverseSameLocation {
        second_location: ForeignHousingLocation::ChinaHongKong,
        confirmed: true,
    };

    let report = supported(&draft);
    assert_eq!(value(&report, "foreign_housing_taxpayer_limit"), 114_300);
    assert_eq!(value(&report, "foreign_housing_exclusion"), 93_036);
    assert_eq!(estimate(&restart(draft.clone()).draft), report);

    let ForeignHousing::Claim { household, .. } =
        &mut claim_mut(&mut draft, Owner::Taxpayer).housing
    else {
        unreachable!()
    };
    *household = ForeignHousingHousehold::AdverseSameLocation {
        second_location: ForeignHousingLocation::AustraliaSydney,
        confirmed: true,
    };
    unsupported(&draft, "Foreign housing allocation");
}

#[test]
fn adverse_second_only_household_in_another_listed_locality_uses_one_cap() {
    let mut draft = empty_return(FederalStatus::Single);
    add(&mut draft, income(foreign_pay(200_000)));
    set_claim(&mut draft, foreign_claim(Owner::Taxpayer));
    // No eligible primary-abode costs are included. All $200,000 of entered
    // costs are for the spouse/dependent household in Sydney, not Hong Kong.
    employee_housing(&mut draft, 200_000, ForeignHousingLocation::AustraliaSydney);
    let ForeignHousing::Claim { household, .. } =
        &mut claim_mut(&mut draft, Owner::Taxpayer).housing
    else {
        unreachable!()
    };
    *household = ForeignHousingHousehold::AdverseSecondOnly {
        primary_location: ForeignHousingLocation::ChinaHongKong,
        second_location: ForeignHousingLocation::AustraliaSydney,
        confirmed: true,
    };

    let report = supported(&draft);
    assert_eq!(value(&report, "foreign_housing_taxpayer_limit"), 65_600);
    assert_eq!(value(&report, "foreign_housing_taxpayer_base"), 21_264);
    assert_eq!(value(&report, "foreign_housing_exclusion"), 44_336);
    assert_eq!(value(&report, "agi"), 22_764);
    assert_eq!(supported(&restart(draft.clone()).draft), report);

    let mut invalid = draft.clone();
    let ForeignHousing::Claim { stays, .. } = &mut claim_mut(&mut invalid, Owner::Taxpayer).housing
    else {
        unreachable!()
    };
    stays[0].location = ForeignHousingLocation::ChinaHongKong;
    unsupported(&invalid, "Foreign housing allocation");
    unsupported(&restart(invalid).draft, "Foreign housing allocation");

    let mut duplicate = draft.clone();
    let mut spouse = duplicate.income[0].clone();
    spouse.owner = crate::DocumentOwner::Spouses;
    add(&mut duplicate, spouse);
    let mut spouse_claim = claim_mut(&mut duplicate, Owner::Taxpayer).clone();
    spouse_claim.owner = Owner::Spouse;
    set_claim(&mut duplicate, spouse_claim);
    unsupported(&duplicate, "Foreign housing allocation");
}

#[test]
fn adverse_second_household_rejects_duplicate_spouse_claim_and_unresolved_caps() {
    let mut d = empty_return(FederalStatus::Joint);
    add(&mut d, income(foreign_pay(200000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    employee_housing(&mut d, 50000, ForeignHousingLocation::General);
    let ForeignHousing::Claim { household, .. } = &mut claim_mut(&mut d, Owner::Taxpayer).housing
    else {
        unreachable!()
    };
    *household = ForeignHousingHousehold::Adverse { confirmed: true };
    supported(&d);
    let mut duplicate = d.clone();
    let mut spouse = duplicate.income[0].clone();
    spouse.owner = crate::DocumentOwner::Spouses;
    add(&mut duplicate, spouse);
    let mut spouse_claim = claim_mut(&mut duplicate, Owner::Taxpayer).clone();
    spouse_claim.owner = Owner::Spouse;
    set_claim(&mut duplicate, spouse_claim);
    unsupported(&duplicate, "Foreign housing allocation");
    unsupported(&restart(duplicate).draft, "Foreign housing allocation");
    let ForeignHousing::Claim { stays, .. } = &mut claim_mut(&mut d, Owner::Taxpayer).housing
    else {
        unreachable!()
    };
    stays[0].location = ForeignHousingLocation::ChinaHongKong;
    unsupported(&d, "Foreign housing allocation");
    unsupported(&restart(d).draft, "Foreign housing allocation");
}

#[test]
fn prior_household_furnishing_retains_2025_vintage_and_form_8283_evidence() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    ded.charity_carryovers.push(prior_household_carry());
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let result = supported(&d);
    // Original-lot amounts are retained in historical audit evidence; this page carries the reconciled class/year balance.
    assert_eq!(value(&result, "charity_carry_2025_property_fifty_opening"), 3_000);
    assert_eq!(value(&result, "charity_carry_2025_property_fifty_used"), 3_000);
    assert_eq!(value(&result, "charity_carry_2025_property_fifty_deduction"), 2_700);
    assert_eq!(value(&result, "charity_carry_2025_property_fifty_floor_not_carried"), 300);
    assert_eq!(value(&result, "charity_carry_2025_property_fifty_remaining"), 0);
    assert_eq!(value(&result, "charity_return_year_carryforward"), 0);
    assert_eq!(return_line(&result, "12e"), 270_000);
    assert!(
        result
            .review_notes
            .iter()
            .any(|note| note.contains("expires after 2030") && note.contains("Form 8283"))
    );
    assert_eq!(estimate(&restart(d.clone()).draft), result);
    let mut without_prior = d.clone();
    without_prior
        .deductions
        .as_mut()
        .unwrap()
        .charity_carryovers
        .retain(|c| c.class != CharityClass::Property50);
    let control = supported(&without_prior);
    assert_eq!(return_line(&control, "15") - return_line(&result, "15"), 270_000);
    assert!(result.tax < control.tax);

    d.deductions.as_mut().unwrap().itemize_override = None;
    let standard = supported(&d);
    assert_eq!(value(&standard, "charity_carry_2025_property_fifty_used"), 3_000);
    assert_eq!(value(&standard, "charity_carry_2025_property_fifty_deduction"), 0);
    assert_eq!(
        value(&standard, "charity_carry_2025_property_fifty_consumed_without_deduction"),
        3_000
    );
    assert_eq!(value(&standard, "charity_carry_2025_property_fifty_floor_not_carried"), 0);
    assert_eq!(value(&standard, "charity_carry_2025_property_fifty_remaining"), 0);
    assert_eq!(return_line(&standard, "12f"), 0);
    assert_eq!(estimate(&restart(d.clone()).draft), standard);
}

#[test]
fn current_cash_property_and_stock_precede_prior_household() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    ded.charity_carryovers.push(prior_household_carry());
    ded.set_cash_charity(2_400_000);
    let mut current_household = used_goods(500_000);
    current_household.kind =
        GiftKind::Goods { kind: GoodsKind::Household, condition: GoodsCondition::Good };
    current_household.description = "Synthetic 2026 used household goods".into();
    current_household.basis = 1_000_000;
    ded.property_gifts.push(current_household);
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let partial = supported(&d);
    assert_eq!(value(&partial, "charity_cash_sixty_current_deduction"), 24_000);
    assert_eq!(value(&partial, "charity_property_fifty_current_deduction"), 5_000);
    assert_eq!(value(&partial, "charity_carry_2025_property_fifty_used"), 3_000);
    assert_eq!(value(&partial, "charity_carry_2025_property_fifty_deduction"), 2_700);
    assert_eq!(value(&partial, "charity_carry_2025_property_fifty_floor_not_carried"), 300);
    assert_eq!(value(&partial, "charity_carry_2025_property_fifty_remaining"), 0);
    assert_eq!(value(&partial, "charity_return_year_carryforward"), 0);
    assert_eq!(return_line(&partial, "12e"), 3_170_000);
    assert_eq!(estimate(&restart(d.clone()).draft), partial);

    d.deductions.as_mut().unwrap().set_cash_charity(2_000_000);
    let mut current_stock = public_stock_gift();
    current_stock.value = 500_000;
    current_stock.basis = 100_000;
    d.deductions.as_mut().unwrap().property_gifts.push(current_stock);
    let with_stock = supported(&d);
    assert_eq!(value(&with_stock, "charity_cash_sixty_current_deduction"), 20_000);
    assert_eq!(value(&with_stock, "charity_property_fifty_current_deduction"), 5_000);
    assert_eq!(value(&with_stock, "charity_carry_2025_property_fifty_used"), 3_000);
    assert_eq!(value(&with_stock, "charity_carry_2025_property_fifty_deduction"), 3_000);
    assert_eq!(value(&with_stock, "charity_carry_2025_property_fifty_floor_not_carried"), 0);
    assert_eq!(value(&with_stock, "charity_capital_gain_thirty_current_deduction"), 1_700);
    assert_eq!(value(&with_stock, "charity_capital_gain_thirty_return_year_carryforward"), 3_300);
    assert_eq!(value(&with_stock, "charity_return_year_carryforward"), 3_300);
    assert_eq!(value(&with_stock, "charity_carry_2025_property_fifty_remaining"), 0);
    assert_eq!(value(&with_stock, "form_8283_section_a_value"), 10_000);
    // Original-lot amounts are retained in historical audit evidence; this page carries the reconciled class/year balance.
    assert_eq!(return_line(&with_stock, "12e"), 2_970_000);
    assert_eq!(estimate(&restart(d.clone()).draft), with_stock);

    d.deductions
        .as_mut()
        .unwrap()
        .property_gifts
        .retain(|g| matches!(g.kind, GiftKind::Goods { .. }));
    d.deductions.as_mut().unwrap().set_cash_charity(2_500_000);
    let current_fills_room = supported(&d);
    assert_eq!(value(&current_fills_room, "charity_carry_2025_property_fifty_used"), 3_000);
    assert_eq!(value(&current_fills_room, "charity_carry_2025_property_fifty_remaining"), 0);
    assert_eq!(value(&current_fills_room, "charity_property_fifty_current_deduction"), 5_000);
    assert_eq!(estimate(&restart(d.clone()).draft), current_fills_room);
}

#[test]
fn prior_household_reconciled_class_balance_accepts_cash_stock_and_whaling() {
    let mut d = complete(FederalStatus::Single);
    d.deductions.as_mut().unwrap().charity_carryovers.push(prior_household_carry());
    assert!(d.valid_stored());
    d.deductions.as_mut().unwrap().charity_carryovers[0].same_filers = false;
    assert!(noted(&supported(&d), "WARN_CHARITY_CARRY_LEFT_OUT"));
    d.deductions.as_mut().unwrap().charity_carryovers[0].same_filers = true;
    d.deductions.as_mut().unwrap().charity_carryovers.push(prior_cash_charity(2025, 100));
    supported(&d);
    d.deductions.as_mut().unwrap().charity_carryovers.push(prior_public_stock_carry());
    supported(&d);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 1_000)]);
    unsupported(&d, "Charitable carryover coordination");
    d.screening.as_mut().unwrap().nol = NolHistory::None;
    payee_statement_credit(&mut d, 10_000);
    unsupported(&d, "Charitable carryover coordination");
    d.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::No;
    d.deductions
        .as_mut()
        .unwrap()
        .whaling_captains
        .push(whaling_claim(vec![whaling_expense(WhalingExpenseKind::WeaponsAndGear, 100_000)]));
    supported(&d);
    // Reconciled balances replace original-lot metadata; whole-dollar validity and vintage still bind.
    let c = &mut d.deductions.as_mut().unwrap().charity_carryovers[0];
    c.balance = 300_001;
    assert!(!d.valid_stored());
    d.deductions.as_mut().unwrap().charity_carryovers[0].balance = 300_000;
    d.deductions.as_mut().unwrap().charity_carryovers[0].year = 2026;
    assert!(!d.valid_stored());
    d.deductions.as_mut().unwrap().charity_carryovers[0].year = 2020;
    assert!(!d.valid_stored());
}

#[test]
fn prior_household_blocks_current_operating_loss_carry_coordination() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, wage(0, Owner::Taxpayer, 1_000_000, 0));
    add(&mut d, income(cash_business(-30_000)));
    d.deductions.as_mut().unwrap().charity_carryovers.push(prior_household_carry());
    unsupported(&d, "Charitable carryover current NOL");
}

#[test]
fn qualified_foster_care_service_costs_join_the_cash_charity_limits() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    ded.set_cash_charity(100_000);
    ded.volunteer_expenses.push(VolunteerExpense {
        date: 20260620,
        charity: "Synthetic county foster agency".into(),
        service: "Nonprofit foster care of a placed child".into(),
        kind: VolunteerExpenseKind::FosterCare { amount: 40_000 },
        documented: true,
    });
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let itemized_result = supported(&d);
    assert_eq!(value(&itemized_result, "charity_volunteer_1_expense"), 400);
    assert_eq!(value(&itemized_result, "charity_cash_sixty_current_deduction"), 1_100);
    assert_eq!(estimate(&restart(d.clone()).draft), itemized_result);

    let ded = d.deductions.as_mut().unwrap();
    ded.itemize_override = None;
    ded.set_cash_charity(0);
    let standard_result = supported(&d);
    assert_eq!(return_line(&standard_result, "12f"), 0);
    d.deductions.as_mut().unwrap().volunteer_expenses[0].documented = false;
    assert!(noted(&supported(&d), "WARN_VOLUNTEER_EXPENSE_LEFT_OUT"));
    d.deductions.as_mut().unwrap().volunteer_expenses[0].documented = true;
}

#[test]
fn hosted_student_months_apply_fifteen_day_test_and_fifty_dollar_cap() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    ded.set_cash_charity(100_000);
    ded.hosted_students.push(HostedStudent {
        label: "Synthetic hosted student".into(),
        sponsor: "Synthetic educational charity".into(),
        agreement: "Synthetic written agreement".into(),
        school: "Synthetic high school".into(),
        school_location: "Synthetic City, CA".into(),
        grade: 11,
        joined: 20260601,
        school_start: 20260601,
        school_end: 20260831,
        months: vec![
            HostedStudentMonth { month: 6, qualifying_days: 15, cost: 8_000 },
            HostedStudentMonth { month: 7, qualifying_days: 14, cost: 10_000 },
            HostedStudentMonth { month: 8, qualifying_days: 20, cost: 3_000 },
        ],
        documented: true,
    });
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let itemized_result = supported(&d);
    assert_eq!(value(&itemized_result, "charity_hosted_student_1_eligible"), 80);
    assert_eq!(value(&itemized_result, "charity_cash_sixty_current_deduction"), 780);
    assert_eq!(value(&itemized_result, "charity_hosted_student_deduction"), 0);
    assert!(
        itemized_result.review_notes.iter().any(|note| note.contains("written sponsor agreement"))
    );
    assert_eq!(estimate(&restart(d.clone()).draft), itemized_result);

    d.deductions.as_mut().unwrap().set_cash_charity(3_595_000);
    let near_sixty_percent = supported(&d);
    assert_eq!(value(&near_sixty_percent, "charity_cash_sixty_current_deduction"), 35_700);
    assert_eq!(value(&near_sixty_percent, "charity_hosted_student_deduction"), 0);
    assert_eq!(value(&near_sixty_percent, "charity_hosted_student_return_year_carryforward"), 80);
    assert_eq!(value(&near_sixty_percent, "charity_return_year_carryforward"), 80);

    let ded = d.deductions.as_mut().unwrap();
    ded.itemize_override = None;
    ded.set_cash_charity(0);
    let standard_result = supported(&d);
    assert_eq!(return_line(&standard_result, "12f"), 0);
    d.deductions.as_mut().unwrap().hosted_students[0].documented = false;
    assert!(noted(&supported(&d), "WARN_HOSTED_STUDENT_LEFT_OUT"));
    d.deductions.as_mut().unwrap().hosted_students[0].documented = true;
    d.deductions.as_mut().unwrap().hosted_students[0].joined = 20260620;
    assert!(!d.valid_stored());
    d.deductions.as_mut().unwrap().hosted_students[0].joined = 20260601;
    let duplicate = d.deductions.as_ref().unwrap().hosted_students[0].clone();
    d.deductions.as_mut().unwrap().hosted_students.push(duplicate);
    assert!(!d.valid_stored());
}

#[test]
fn hosted_student_expenses_obey_thirty_percent_limit_and_keep_2026_class() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 2_000_000, 0);
    let ded = d.deductions.as_mut().unwrap();
    for index in 0..11 {
        ded.hosted_students.push(HostedStudent {
            label: alloc::format!("Synthetic student {index}"),
            sponsor: "Synthetic educational charity".into(),
            agreement: "Synthetic written agreement".into(),
            school: "Synthetic year-round high school".into(),
            school_location: "Synthetic City, CA".into(),
            grade: 11,
            joined: 20260101,
            school_start: 20260101,
            school_end: 20261231,
            months: (1..=12)
                .map(|month| HostedStudentMonth { month, qualifying_days: 15, cost: 5_000 })
                .collect(),
            documented: true,
        });
    }
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let result = supported(&d);
    assert_eq!(value(&result, "charity_hosted_student_deduction"), 5_900);
    assert_eq!(value(&result, "charity_hosted_student_return_year_carryforward"), 700);
    assert_eq!(value(&result, "charity_return_year_carryforward"), 700);
    assert_eq!(estimate(&restart(d.clone()).draft), result);

    d.deductions.as_mut().unwrap().itemize_override = None;
    let standard = supported(&d);
    assert_eq!(return_line(&standard, "12f"), 0);
    assert_eq!(value(&standard, "charity_hosted_student_return_year_carryforward"), 600);
    assert_eq!(value(&standard, "charity_return_year_carryforward"), 600);
}

#[test]
fn current_and_prior_cash_share_one_b_limit_for_student_and_whaling() {
    // AGI $100,000: 2026 G $20,000, 2025 G $25,000; B room is
    // $50,000 - $45,000 + the $10,000 cash carve-out = $15,000.
    // The $600 student claim and $30,000 whaling claim share that room and
    // the $500 floor. No source-specific B allocation is asserted.
    let mut draft = with_current_direct_cash(
        cash_carry_with_student_and_whaling(10_000_000, 12, 3_000_000, 25_000),
        2_000_000,
    );
    let itemized = supported(&draft);
    assert_eq!(value(&itemized, "charity_hosted_student_1_eligible"), 600);
    assert_eq!(value(&itemized, "charity_whaling_statutory_claim"), 30_000);
    assert_eq!(value(&itemized, "charity_cash_sixty_current_deduction"), 20_000);
    assert_eq!(value(&itemized, "charity_carry_2025_cash_sixty_used"), 25_000);
    assert_eq!(value(&itemized, "charity_prior_cash_sixty_deduction"), 25_000);
    assert_eq!(value(&itemized, "charity_other_30_percent_deduction"), 14_500);
    assert_eq!(value(&itemized, "charity_other_30_percent_return_year_carryforward"), 16_100);
    assert_eq!(value(&itemized, "charity_return_year_carryforward"), 16_100);
    assert_eq!(return_line(&itemized, "12e"), 5_950_000);
    assert!(!itemized.lines.iter().any(|line| {
        line.id == "charity_hosted_student_deduction" || line.id == "charity_whaling_deduction"
    }));
    assert_eq!(estimate(&restart(draft.clone()).draft), itemized);

    // 2026-11-02: automatic selection retains the same itemized facts and limits.
    draft.deductions.as_mut().unwrap().itemize_override = None;
    let automatic = supported(&draft);
    assert_eq!(value(&automatic, "charity_other_30_percent_deduction"), 14_500);
    assert_eq!(value(&automatic, "charity_other_30_percent_return_year_carryforward"), 16_100);
    assert_eq!(value(&automatic, "charity_prior_cash_sixty_remaining_following_year"), 0);
    assert_eq!(return_line(&automatic, "12f"), 0);
    assert_eq!(estimate(&restart(draft).draft), automatic);
}

#[test]
fn current_and_prior_cash_with_hosted_student_charge_floor_to_b_first() {
    let mut draft =
        with_current_direct_cash(cash_carry_with_hosted_student(6_000_000, 12, 15_600), 2_000_000);
    let itemized = supported(&draft);
    assert_eq!(value(&itemized, "agi"), 60_000);
    assert_eq!(value(&itemized, "charity_cash_sixty_current_deduction"), 20_000);
    assert_eq!(value(&itemized, "charity_carry_2025_cash_sixty_used"), 15_600);
    assert_eq!(value(&itemized, "charity_prior_cash_sixty_deduction"), 15_600);
    assert_eq!(value(&itemized, "charity_hosted_student_deduction"), 100);
    assert_eq!(value(&itemized, "charity_hosted_student_return_year_carryforward"), 500);
    assert_eq!(value(&itemized, "charity_cash_sixty_return_year_carryforward"), 0);
    assert_eq!(value(&itemized, "charity_return_year_carryforward"), 500);
    assert_eq!(return_line(&itemized, "12e"), 3_570_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), itemized);

    // 2026-11-02: automatic selection retains the same itemized facts and limits.
    draft.deductions.as_mut().unwrap().itemize_override = None;
    let automatic = supported(&draft);
    assert_eq!(return_line(&automatic, "12f"), 0);
    assert_eq!(value(&automatic, "charity_carry_2025_cash_sixty_used"), 15_600);
    assert_eq!(value(&automatic, "charity_prior_cash_sixty_consumed_without_deduction"), 0);
    assert_eq!(value(&automatic, "charity_hosted_student_return_year_carryforward"), 500);
    assert_eq!(estimate(&restart(draft).draft), automatic);
}

#[test]
fn joint_captains_each_receive_their_own_fifty_thousand_dollar_cap() {
    let mut draft = complete(FederalStatus::Joint);
    draft.income[0] = wage(1, Owner::Taxpayer, 34_000_000, 0);
    let ded = draft.deductions.as_mut().unwrap();
    ded.whaling_captains.push(whaling_claim(vec![whaling_expense(
        WhalingExpenseKind::CrewAndProvisions,
        6_500_000,
    )]));
    let mut spouse =
        whaling_claim(vec![whaling_expense(WhalingExpenseKind::WeaponsAndGear, 5_500_000)]);
    spouse.owner = Owner::Spouse;
    ded.whaling_captains.push(spouse);
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let result = supported(&draft);
    assert_eq!(value(&result, "charity_whaling_taxpayer_statutory_claim"), 50_000);
    assert_eq!(value(&result, "charity_whaling_spouse_statutory_claim"), 50_000);
    assert_eq!(value(&result, "charity_whaling_statutory_claim"), 100_000);
    assert_eq!(value(&result, "charity_whaling_deduction"), 98_300);
    assert_eq!(value(&result, "charity_whaling_return_year_carryforward"), 0);
    assert_eq!(estimate(&restart(draft.clone()).draft), result);

    draft.deductions.as_mut().unwrap().whaling_captains[1].owner = Owner::Taxpayer;
    assert!(!draft.valid_stored());
    let mut single = complete(FederalStatus::Single);
    let mut spouse =
        whaling_claim(vec![whaling_expense(WhalingExpenseKind::WeaponsAndGear, 100_000)]);
    spouse.owner = Owner::Spouse;
    single.deductions.as_mut().unwrap().whaling_captains.push(spouse);
    unsupported(&single, "Whaling captain charity");
}

#[test]
fn paired_2025_household_and_stock_carries_keep_vintage_evidence_and_standard_use() {
    let mut draft = paired_2025_noncash_carries();
    let deductions = draft.deductions.as_mut().unwrap();
    deductions.itemize_override = Some(ItemizeOverride::Itemize);
    let result = supported(&draft);
    assert_eq!(value(&result, "charity_carry_2025_property_fifty_opening"), 3_000);
    assert_eq!(value(&result, "charity_carry_2025_capital_gain_thirty_opening"), 20_000);
    assert_eq!(value(&result, "charity_carry_2025_property_fifty_used"), 3_000);
    assert_eq!(value(&result, "charity_carry_2025_capital_gain_thirty_used"), 18_000);
    assert_eq!(value(&result, "charity_carry_2025_property_fifty_deduction"), 3_000);
    assert_eq!(value(&result, "charity_carry_2025_capital_gain_thirty_deduction"), 17_700);
    assert_eq!(value(&result, "charity_carry_2025_capital_gain_thirty_floor_not_carried"), 300);
    assert_eq!(value(&result, "charity_carry_2025_property_fifty_floor_not_carried"), 0);
    assert_eq!(value(&result, "charity_carry_2025_property_fifty_remaining"), 0);
    assert_eq!(value(&result, "charity_carry_2025_capital_gain_thirty_remaining"), 2_000);
    assert_eq!(value(&result, "charity_return_year_carryforward"), 0);
    // Original-lot amounts are retained in historical audit evidence; this page carries the reconciled class/year balance.
    assert_eq!(return_line(&result, "12e"), 2_070_000);
    assert!(result.review_notes.iter().any(|note| {
        note.contains("2025 original and intervening contribution worksheets")
            && note.contains("Form 8283")
            && note.contains("expires after 2030")
    }));
    assert_eq!(estimate(&restart(draft.clone()).draft), result);

    let mut control = draft.clone();
    control
        .deductions
        .as_mut()
        .unwrap()
        .charity_carryovers
        .retain(|c| c.class != CharityClass::Property50);
    control
        .deductions
        .as_mut()
        .unwrap()
        .charity_carryovers
        .retain(|c| c.class != CharityClass::CapitalGain30);
    let without_carries = supported(&control);
    assert_eq!(return_line(&without_carries, "15") - return_line(&result, "15"), 2_070_000);
    assert!(result.tax < without_carries.tax);

    // 2026-11-02: automatic selection retains the same itemized facts and limits.
    draft.deductions.as_mut().unwrap().itemize_override = None;
    let automatic = supported(&draft);
    assert_eq!(value(&automatic, "charity_carry_2025_property_fifty_used"), 3_000);
    assert_eq!(value(&automatic, "charity_carry_2025_capital_gain_thirty_used"), 18_000);
    assert_eq!(
        value(&automatic, "charity_carry_2025_property_fifty_consumed_without_deduction"),
        0
    );
    assert_eq!(
        value(&automatic, "charity_carry_2025_capital_gain_thirty_consumed_without_deduction"),
        0
    );
    assert_eq!(value(&automatic, "charity_carry_2025_capital_gain_thirty_remaining"), 2_000);
    assert_eq!(return_line(&automatic, "12f"), 0);
    assert_eq!(estimate(&restart(draft.clone()).draft), automatic);
}

#[test]
fn current_stock_precedes_both_2025_carries_and_prior_household_precedes_prior_stock() {
    let mut draft = paired_2025_noncash_carries();
    let deductions = draft.deductions.as_mut().unwrap();
    deductions.set_cash_charity(2_000_000);
    let mut current_household = used_goods(500_000);
    current_household.kind =
        GiftKind::Goods { kind: GoodsKind::Household, condition: GoodsCondition::Good };
    current_household.basis = 1_000_000;
    deductions.property_gifts.push(current_household);
    let mut current_stock = public_stock_gift();
    current_stock.value = 400_000;
    current_stock.basis = 100_000;
    deductions.property_gifts.push(current_stock);
    deductions.itemize_override = Some(ItemizeOverride::Itemize);

    let with_current_stock = supported(&draft);
    assert_eq!(value(&with_current_stock, "charity_cash_sixty_current_deduction"), 20_000);
    assert_eq!(value(&with_current_stock, "charity_property_fifty_current_deduction"), 5_000);
    assert_eq!(value(&with_current_stock, "charity_capital_gain_thirty_current_deduction"), 1_700);
    assert_eq!(value(&with_current_stock, "charity_carry_2025_property_fifty_used"), 3_000);
    assert_eq!(value(&with_current_stock, "charity_carry_2025_capital_gain_thirty_used"), 0);
    assert_eq!(value(&with_current_stock, "charity_carry_2025_property_fifty_remaining"), 0);
    assert_eq!(
        value(&with_current_stock, "charity_carry_2025_capital_gain_thirty_remaining"),
        20_000
    );
    assert_eq!(value(&with_current_stock, "charity_return_year_carryforward"), 2_300);
    assert_eq!(value(&with_current_stock, "form_8283_section_a_value"), 9_000);
    // Original-lot amounts are retained in historical audit evidence; this page carries the reconciled class/year balance.
    assert_eq!(return_line(&with_current_stock, "12e"), 2_970_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), with_current_stock);

    draft
        .deductions
        .as_mut()
        .unwrap()
        .property_gifts
        .retain(|g| matches!(g.kind, GiftKind::Goods { .. }));
    let without_current_stock = supported(&draft);
    assert_eq!(value(&without_current_stock, "charity_carry_2025_property_fifty_used"), 3_000);
    assert_eq!(value(&without_current_stock, "charity_carry_2025_capital_gain_thirty_used"), 2_000);
    assert_eq!(
        value(&without_current_stock, "charity_carry_2025_capital_gain_thirty_remaining"),
        18_000
    );
    assert_eq!(estimate(&restart(draft.clone()).draft), without_current_stock);

    let deductions = draft.deductions.as_mut().unwrap();
    deductions.property_gifts.clear();
    deductions.set_cash_charity(2_700_000);
    let household_first = supported(&draft);
    assert_eq!(value(&household_first, "charity_carry_2025_property_fifty_used"), 3_000);
    assert_eq!(value(&household_first, "charity_carry_2025_capital_gain_thirty_used"), 0);
    assert_eq!(value(&household_first, "charity_carry_2025_capital_gain_thirty_remaining"), 20_000);
    assert_eq!(value(&household_first, "charity_carry_2025_property_fifty_floor_not_carried"), 300);
    assert_eq!(return_line(&household_first, "12e"), 2_970_000);
}

#[test]
fn paired_2025_floor_can_span_stock_and_household_without_new_vintage() {
    let mut draft = paired_2025_noncash_carries();
    let deductions = draft.deductions.as_mut().unwrap();
    let stock = deductions
        .charity_carryovers
        .iter_mut()
        .find(|c| c.class == CharityClass::CapitalGain30)
        .unwrap();
    stock.balance = 10_000;
    stock.appreciation = Some(5_000);
    deductions.itemize_override = Some(ItemizeOverride::Itemize);
    let result = supported(&draft);
    assert_eq!(value(&result, "charity_carry_2025_capital_gain_thirty_used"), 100);
    assert_eq!(value(&result, "charity_carry_2025_capital_gain_thirty_floor_not_carried"), 100);
    assert_eq!(value(&result, "charity_carry_2025_capital_gain_thirty_deduction"), 0);
    assert_eq!(value(&result, "charity_carry_2025_property_fifty_floor_not_carried"), 200);
    assert_eq!(value(&result, "charity_carry_2025_property_fifty_deduction"), 2_800);
    assert_eq!(value(&result, "charity_return_year_carryforward"), 0);
    assert_eq!(return_line(&result, "12e"), 280_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), result);
}

#[test]
fn public_stock_follows_cash_and_hosted_students_under_2026_limits() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    ded.property_gifts.push(public_stock_gift());
    ded.hosted_students.push(HostedStudent {
        label: "Synthetic hosted student".into(),
        sponsor: "Synthetic educational charity".into(),
        agreement: "Synthetic written agreement".into(),
        school: "Synthetic high school".into(),
        school_location: "Synthetic City, CA".into(),
        grade: 11,
        joined: 20260601,
        school_start: 20260601,
        school_end: 20260630,
        months: vec![HostedStudentMonth { month: 6, qualifying_days: 15, cost: 8_000 }],
        documented: true,
    });
    ded.set_cash_charity(100_000);
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let mixed = supported(&d);
    assert_eq!(value(&mixed, "charity_capital_gain_thirty_current_deduction"), 17_700);
    assert_eq!(value(&mixed, "charity_hosted_student_deduction"), 50);
    assert_eq!(value(&mixed, "charity_cash_sixty_current_deduction"), 1_000);
    assert_eq!(value(&mixed, "charity_return_year_carryforward"), 2_300);
    assert_eq!(estimate(&restart(d.clone()).draft), mixed);

    d.deductions.as_mut().unwrap().set_cash_charity(3_500_000);
    let high_cash = supported(&d);
    assert_eq!(value(&high_cash, "charity_capital_gain_thirty_current_deduction"), 0);
    assert_eq!(value(&high_cash, "charity_hosted_student_deduction"), 0);
    assert_eq!(value(&high_cash, "charity_cash_sixty_current_deduction"), 34_700);
    assert_eq!(value(&high_cash, "charity_capital_gain_thirty_return_year_carryforward"), 20_000);
    assert_eq!(value(&high_cash, "charity_hosted_student_return_year_carryforward"), 50);
    assert_eq!(value(&high_cash, "charity_return_year_carryforward"), 20_050);

    let ded = d.deductions.as_mut().unwrap();
    ded.set_cash_charity(0);
    ded.property_gifts[0].value = 4_000_000;
    let stock_capped = supported(&d);
    assert_eq!(value(&stock_capped, "charity_capital_gain_thirty_current_deduction"), 17_700);
    assert_eq!(value(&stock_capped, "charity_hosted_student_deduction"), 0);
    assert_eq!(value(&stock_capped, "charity_hosted_student_return_year_carryforward"), 50);
    assert_eq!(value(&stock_capped, "charity_return_year_carryforward"), 22_350);
}

#[test]
fn joint_special_needs_deemed_exclusion_and_care_grace_keep_distinct_wage_lines() {
    let mut d = joint_special_needs_below_code_t_cap();
    add_child(&mut d);
    d.dependents[0].set_care_expenses(900_000);
    grace_and_current_benefits(&mut d);
    care_box10(&mut d, 0, 700_000);
    document_care(&mut d);

    let result = supported(&d);
    assert_eq!(value(&result, "adoption_joint_special_needs_deemed_excess"), 5_670);
    assert_eq!(value(&result, "care_form2441_line12"), 7_000);
    assert_eq!(value(&result, "care_form2441_line13"), 2_000);
    assert_eq!(value(&result, "care_benefits_excluded"), 7_500);
    assert_eq!(value(&result, "care_benefits_taxable"), 1_500);
    assert_eq!(value(&result, "care_earned_taxpayer"), 81_500);
    assert_eq!(value(&result, "care_earned_spouse"), 20_000);
    assert_eq!(return_line(&result, "1a"), 10_000_000);
    assert_eq!(return_line(&result, "1e"), 150_000);
    assert_eq!(return_line(&result, "1f"), -567_000);
    assert_eq!(return_line(&result, "25a"), 950_000);
    assert_eq!(value(&result, "agi"), 95_830);
    assert_eq!(estimate(&restart(d).draft), result);
}

// T2-30f: Form 2441 (matrix row 30). Combined joint fixture: two qualifying
// children (one turning 13 mid-2026, still a full-year qualifying person
// because family()'s test in federal_rules.rs is a single January 1, 2026
// `under_13_in_year` test for the whole year, not a per-period apportionment),
// $7,000 of expenses over the $6,000 two-person ceiling, $2,000 of employer
// dependent-care benefits split across both spouses' W-2s (Form 2441 Part
// III, fully excluded), and an AGI of $50,000 that lands the credit at the
// 2026 P.L. 119-21 35% floor rate.
//
// Hand computation (forms/form_2441.rs, care.rs, federal_rules.rs, i2441--dft):
//   Wages: taxpayer $30,000 + spouse $20,000 = $50,000 = AGI (no other
//     income or adjustments).
//   care_rate(50000, true): ceiling($50,000-$15,000, $2,000) = 18 (since
//     $35,000/$2,000 = 17.5 rounds up), 50-18 = 32, floored to 35 by
//     max(35, ...); the $150,000 second phase-down term is 0 (AGI is under
//     it). Rate = 35%.
//   Part III (CareBenefits::JointEmployees, no grace): line 12 = $1,000 +
//     $1,000 = $2,000; line 15 = $2,000 (line 13 = $0, no grace balance).
//     Line 20 = min(line 15 $2,000, incurred $7,000, taxpayer earned
//     $30,000, spouse earned $20,000) = $2,000. Line 21 (no grace) = $7,500.
//     Line 25 excluded = min($2,000, $7,500) = $2,000. Line 26 taxable =
//     $2,000 - $2,000 = $0 (Form 1040 line 1e stays $0).
//   Form 2441 line 3: care_cost = $7,000 - $2,000 excluded = $5,000, capped
//     at the two-person $6,000 ceiling less the same $2,000 excluded =
//     $4,000; the $30,000/$20,000 earned-income caps do not bind.
//   Line 9a/9c = $4,000 x 35% = $1,400 (no line 9b prior-year care).
//   Standard deduction (MFJ 2026, federal_rules.rs standard_base) =
//     $32,200; taxable income = $50,000 - $32,200 = $17,800; 2026 draft Tax
//     Table row 17,800-17,850, MFJ column
//     (tests/fixtures/irs-tax-table-2026.csv) = $1,783.
//   Credit Limit Worksheet B draws care first from the $1,783 of tax: all
//     $1,400 is used, leaving $383 for the nonrefundable child credit.
//   Schedule 8812: two CTC children (both under 17, valid SSNs), MAGI
//     $50,000 far under the $400,000 joint threshold, so potential =
//     2 x $2,200 = $4,400; nonrefundable (line 14) = min($4,400, $383) =
//     $383; unused potential = $4,017; earned-income ceiling (line 20) =
//     15% x ($50,000 - $2,500) = $7,125; ACTC (line 27) = min($4,017,
//     min($3,400, $7,125)) = $3,400.
//   Schedule EIC, 2 children, joint: earned income = AGI = $50,000 falls in
//     the draft Publication 1040 $50,000-$50,050 row
//     (tests/fixtures/irs-eic-2026-draft.csv), joint two-children column =
//     $3,343 (verified against eic_table_2026's own formula by hand); AGI
//     equals earned income so the phase-out lookup does not change it.
//   Line 18 tax before credits = $1,783; line 19 = $383; line 20 (Schedule 3
//     line 2, the Form 2441 credit) = $1,400; line 21 = $1,783; line 22 =
//     $0; line 23 (no SE, Additional Medicare or NIIT at this income) = $0;
//     line 24a = $0.
//   Line 25a withholding = $2,000 + $1,000 = $3,000; line 27a = $3,343;
//     line 28 = $3,400; line 33 = $3,000 + $3,343 + $3,400 = $9,743; line 34
//     (refund) = $9,743.
//
// Not implemented by this card: nothing above is refused. This fixture only
// exercises the calculation directly (as the neighbouring `benefit_return`/
// `joint_employer_care_return` tests do), bypassing the interview's save-time
// check. Fixed (unit T2-30x, 2026-09-26): interview.rs's `dependent_rule`
// (`CARE_NEEDS_UNDER_13_OR_SELF_CARE`) used cutoff `BORN_BEFORE_AGE_13 =
// 20140101`, which refused care costs for any child born in 2013 at all
// through the M2/M3 "Children and people you support" page, even though this
// fixture shows the calculation model (family(), then a `birth_date >
// 20130101` cutoff, now the day-exact `under_13_in_year`) correctly gives
// credit for the part of 2026 before a child's 13th birthday.
// `dependent_rule` now shares the same test (see unit ages, 2026-10-08, in
// interview.rs); see
// tests/interview.rs care_costs_are_accepted_for_a_child_turning_13_in_2026
// and care_costs_are_refused_for_a_child_already_13_on_january_1.
#[test]
fn joint_two_children_one_turning_13_with_employer_care_benefits() {
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 3_000_000, 200_000);
    let id = d.next_id;
    add(&mut d, wage(id, Owner::Spouse, 2_000_000, 100_000));
    add_child(&mut d);
    add_child(&mut d);
    d.dependents[0].birth_date = 20130615; // turns 13 on 2026-06-15
    d.dependents[0].set_care_expenses(350_000);
    d.dependents[1].birth_date = 20200310; // age 6 all year
    d.dependents[1].set_care_expenses(350_000);
    let taxpayer_w2 = w2(&d, 1);
    let spouse_w2 = w2(&d, 2);
    care_w2(&mut d, taxpayer_w2, 100_000, 0, 0, 500_000);
    care_w2(&mut d, spouse_w2, 100_000, 0, 0, 500_000);
    document_care(&mut d);

    let r = supported(&d);
    assert_eq!(value(&r, "care_credit"), 1_400);
    assert_eq!(value(&r, "care_benefits_excluded"), 2_000);
    assert_eq!(value(&r, "care_benefits_taxable"), 0);
    assert_eq!(value(&r, "child_credit"), 383);
    assert_eq!(value(&r, "additional_child_credit"), 3_400);
    assert_eq!(value(&r, "earned_income_credit"), 3_343);
    assert_eq!(return_line(&r, "1a"), 5_000_000);
    assert_eq!(return_line(&r, "1e"), 0);
    assert_eq!(return_line(&r, "15"), 1_780_000);
    assert_eq!(return_line(&r, "16"), 178_300);
    assert_eq!(return_line(&r, "25a"), 300_000);
    assert_eq!(return_line(&r, "33"), 974_300);
    assert_eq!(r.tax, Some(0));
    assert_eq!(r.refund, Some(974_300));
}

// T2-30f boundary (1 of 5): the $3,000/$6,000 expense ceiling. A single
// filer's wages of $70,000 keep AGI in the 2026 35% floor range
// (care_rate: ceiling($70,000-$15,000, $2,000) = 28, 50-28 = 22, floored to
// 35; AGI <= $75,000 keeps the second phase-down term at 0), and produce
// $6,576 of tax on $53,900 of taxable income ($70,000 - the $16,100 single
// standard deduction; 2026 draft Tax Table row 53,900-53,950, single
// column) — far more than either credit below, so Credit Limit Worksheet B
// never limits these amounts and only the expense ceiling is under test.
#[test]
fn care_expense_ceiling_does_not_grow_past_3000_or_6000() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 7_000_000, 0);
    add_child(&mut d);
    // One qualifying person: the $3,000 ceiling.
    d.dependents[0].set_care_expenses(300_000);
    assert_eq!(
        value(
            &{
                document_care(&mut d);
                supported(&d)
            },
            "care_credit"
        ),
        1_050
    ); // $3,000 x 35%
    d.dependents[0].set_care_expenses(300_100); // $3,000.01 paid
    assert_eq!(
        value(
            &{
                document_care(&mut d);
                supported(&d)
            },
            "care_credit"
        ),
        1_050
    ); // unchanged: still $3,000 x 35%
    // A second qualifying person (no care costs of their own) doubles the
    // ceiling to $6,000 (federal_rules.rs family(): the two-person ceiling
    // applies even if expenses were paid for only one person).
    add_child(&mut d);
    d.dependents[0].set_care_expenses(600_000);
    assert_eq!(
        value(
            &{
                document_care(&mut d);
                supported(&d)
            },
            "care_credit"
        ),
        2_100
    ); // $6,000 x 35%
    d.dependents[0].set_care_expenses(600_100); // $6,000.01 paid
    assert_eq!(
        value(
            &{
                document_care(&mut d);
                supported(&d)
            },
            "care_credit"
        ),
        2_100
    ); // unchanged: still $6,000 x 35%
}

// T2-30f boundary (2 of 5): the earned-income limit (line 6), including the
// disabled-spouse deemed-income worksheet. A disabled spouse both adds a
// qualifying person to the credit ceiling (federal_rules.rs: `ctx.care_people
// + i64::from(ctx.disabled_spouse)` turns the one-child $3,000 ceiling into
// the two-person $6,000 ceiling) and raises the monthly deemed-income
// amount to $500/month (care.rs CareEarnings::calculate: `people_count >=
// 2` selects $500 over $250). AGI stays at $100,000 (the taxpayer's default
// wage; the spouse has none), keeping care_rate at the 2026 35% floor
// (first term floors to 35 once AGI is over $43,001; the second
// phase-down needs AGI over $150,000). The taxpayer's own $100,000 of
// earned income is never the binding cap; the disabled spouse's deemed
// income is.
#[test]
fn disabled_spouse_deemed_income_binds_the_earned_income_limit() {
    let mut d = complete(FederalStatus::Joint);
    add_child(&mut d);
    d.dependents[0].set_care_expenses(600_000); // $6,000, the two-person ceiling
    d.credits.as_mut().unwrap().care = FederalCare::default();
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().incapable_self_care = true;
    let mut months = care_months(Owner::Spouse, 0);
    for m in months.iter_mut().take(11) {
        m.incapable = true;
    }
    set_care_months(&mut d, months.clone());
    // 11 incapable months x $500 = $5,500 of deemed spouse earned income,
    // below the $6,000 ceiling, so line 6 caps the credit base there:
    // $5,500 x 35% = $1,925.
    let r = supported(&d);
    assert_eq!(value(&r, "care_earned_spouse"), 5_500);
    assert_eq!(value(&r, "care_credit"), 1_925);
    for m in &mut months {
        m.incapable = true;
    }
    set_care_months(&mut d, months);
    // All 12 months x $500 = $6,000, exactly matching the ceiling: the
    // earned-income limit no longer reduces the credit base.
    let r = supported(&d);
    assert_eq!(value(&r, "care_earned_spouse"), 6_000);
    assert_eq!(value(&r, "care_credit"), 2_100);
}

// T2-30f boundary (3 of 5): the 2026 percentage table by AGI (care_rate,
// forms/form_2441.rs). Its first term floors at 35% once
// ceiling(AGI-$15,000, $2,000) reaches 15 steps, i.e. once AGI-$15,000
// exceeds $28,000: AGI = $43,001 is the first dollar where that happens
// (ceiling($28,001, $2,000) = 15, 50-15 = 35), while $43,000 itself
// (ceiling($28,000, $2,000) = 14, 50-14 = 36) is still one point above the
// floor. $1,000 of care expenses is far under every other cap (a $3,000
// one-person ceiling; $43,000+ of earned income), so only the rate changes
// between the two AGIs.
#[test]
fn care_rate_reaches_the_35_percent_floor_at_43001_agi() {
    let mut d = complete(FederalStatus::Single);
    add_child(&mut d);
    d.dependents[0].set_care_expenses(100_000); // $1,000
    d.income[0] = wage(1, Owner::Taxpayer, 4_300_000, 0); // AGI $43,000
    assert_eq!(
        value(
            &{
                document_care(&mut d);
                supported(&d)
            },
            "care_credit"
        ),
        360
    ); // $1,000 x 36%
    d.income[0] = wage(1, Owner::Taxpayer, 4_300_100, 0); // AGI $43,001
    assert_eq!(
        value(
            &{
                document_care(&mut d);
                supported(&d)
            },
            "care_credit"
        ),
        350
    ); // $1,000 x 35%
}

// T2-30f boundary (4 of 5): the dependent-care benefits exclusion (line 12
// onward), specifically the $7,500 statutory cap on line 21/25 (2026
// increase from $5,000, P.L. 119-21). A single filer's employer-plan
// benefit of exactly $7,500 (at both the plan's own limit and the
// statutory cap) is fully excluded; one more dollar of benefit is not.
#[test]
fn employer_benefit_exclusion_caps_at_7500_and_taxes_the_remainder() {
    let mut d = complete(FederalStatus::Single);
    add_child(&mut d);
    care_costs(&mut d, 800_000);
    let id = w2(&d, 1);
    care_w2(&mut d, id, 750_000, 0, 0, 750_000);

    let r = supported(&d);
    assert_eq!(value(&r, "care_benefits_excluded"), 7_500);
    assert_eq!(value(&r, "care_benefits_taxable"), 0);
    care_box10(&mut d, 0, 750_100); // $7,501
    let r = supported(&d);
    assert_eq!(value(&r, "care_benefits_excluded"), 7_500);
    assert_eq!(value(&r, "care_benefits_taxable"), 1);
}

// T2-30f boundary (5 of 5): the age-13 test. family() (federal_rules.rs)
// tests age 13 with one fixed January 1, 2026 comparison for the whole year,
// not a per-period apportionment. IRC 21(b)(1)(A) needs a dependent who has
// not attained 13, and a child attains an age on the birthday (Rev. Rul.
// 2003-72; the 2026 i2441 "turned 13 during the year"; unit family credits F8,
// 2026-10-10): a child born January 1, 2013 turned 13 on January 1, 2026 and
// was never under 13 during 2026 (care_person false; any claimed expense is
// left out with a WARN_CARE_COSTS_LEFT_OUT notice and the estimate proceeds);
// a child born January 2, 2013 turns 13 on January 2, 2026, was under 13 on
// January 1, and counts as a qualifying person for the whole year's expenses.
#[test]
fn age_13_test_is_a_whole_year_cutoff_at_january_1_2013() {
    let mut d = complete(FederalStatus::Single);
    add_child(&mut d);
    d.dependents[0].set_care_expenses(300_000);
    d.dependents[0].birth_date = 20130101;
    let r = {
        document_care(&mut d);
        supported(&d)
    };
    assert!(noted(&r, "WARN_CARE_COSTS_LEFT_OUT"), "{r:?}");
    assert!(!r.lines.iter().any(|l| l.id == "care_credit" && l.amount > 0));
    for born in [20130102, 20130103] {
        d.dependents[0].birth_date = born;
        assert_eq!(
            value(
                &{
                    document_care(&mut d);
                    supported(&d)
                },
                "care_credit"
            ),
            1_050,
            "{born}"
        ); // $3,000 x 35%
    }
}

// Unit family credits F8 (2026-10-10), replacing unit ages (2026-10-08):
// Rev. Rul. 2003-72 (2003-33 I.R.B. 346) says that for sections 21, 23, 24, 32,
// 129 and 151 "a child attains a given age on the anniversary of the date that
// the child was born ... a child born on January 1, 1987, attains the age of 17
// on January 1, 2004". The day-before rule is written only for age 65, the
// childless EIC age 25, Form 8814 and 8615 and the Form 8880 date. A child born
// on January 1 therefore attains each age on the January 1 that ends the year.
//
// IRC 152(c)(3)(A) and 32(c)(3)(A): a qualifying child is under 19 at the
// close of the year, or under 24 and a student. Born 2008-01-01 the 19th
// birthday is 2027-01-01: still 18, a qualifying child (the 2026 Schedule EIC
// line 3 "If born after 2007"). Born 2007-12-31 the 19th birthday is
// 2026-12-31: 19, not a qualifying child unless a student under 24. Born
// 2003-01-01 the 24th birthday is 2027-01-01: 23, a student qualifying child;
// born 2002-12-31 is 24. The one-child EIC at $20,000 of wages is the 2026
// maximum ($4,427); the no-child EIC at the same wages is the baseline.
#[test]
fn dependent_age_19_and_24_are_attained_on_the_birthday_for_qualifying_child_and_eic() {
    let eic = |born: u32, student: bool, child: bool| {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, 2_000_000, 0);
        if child {
            add_child(&mut d);
            d.dependents[0].birth_date = born;
            d.dependents[0].situation = if student {
                DependentSituation::Student
            } else {
                DependentSituation::ChildUnder19
            };
        }
        value(&supported(&d), "earned_income_credit")
    };
    let none = eic(0, false, false);
    assert!(none < 4427);
    // 2008-01-01: 18 at year end.
    assert_eq!(eic(20080101, false, true), 4427);
    // 2007-12-31: 19, no longer a qualifying child.
    assert_eq!(eic(20071231, false, true), none);
    // ... but a student under 24 qualifies.
    assert_eq!(eic(20071231, true, true), 4427);
    // 2003-01-01: 23 at year end, a student.
    assert_eq!(eic(20030101, true, true), 4427);
    // 2002-12-31: 24, a student no longer qualifies.
    assert_eq!(eic(20021231, true, true), none);
}

// IRC 24(c)(1): the child tax credit needs a qualifying child under 17 at the
// close of the year, and a child attains 17 on the birthday (Rev. Rul.
// 2003-72). Born 2010-01-01 the 17th birthday is 2027-01-01: 16, the $2,200
// CTC. Born 2009-12-31 attains 17 on December 31, 2026, so the child earns the
// $500 credit for other dependents (ODC), not the CTC. Joint wages of $100,000
// give tax far above either credit.
#[test]
fn dependent_age_17_is_attained_on_the_birthday_for_the_child_tax_credit() {
    let credit = |born: u32| {
        let mut d = complete(FederalStatus::Joint);
        add_child(&mut d);
        d.dependents[0].birth_date = born;
        value(&supported(&d), "child_credit")
    };
    assert_eq!(credit(20100101), 2200);
    assert_eq!(credit(20091231), 500);
}

// IRC 21(b)(1)(A): Form 2441 credits care for a dependent who has not
// attained 13, tested for the part of the year before the 13th birthday. Born
// 2014-01-01 the 13th birthday is 2027-01-01 and 13 is attained 2026-12-31,
// so the child is under 13 for all but the last day of 2026 and still
// qualifies (the child is "13 at year end" only on that last day). Born
// 2014-01-02 is the control; $3,000 of expenses x 35% = $1,050. Contrast the
// 2013 boundary in age_13_test_is_a_whole_year_cutoff_at_january_2_2013.
#[test]
fn dependent_age_13_care_credit_counts_a_child_who_turns_13_on_the_last_day_of_2026() {
    for born in [20140101, 20140102] {
        let mut d = complete(FederalStatus::Single);
        add_child(&mut d);
        d.dependents[0].birth_date = born;
        d.dependents[0].set_care_expenses(300_000);
        assert_eq!(
            value(
                &{
                    document_care(&mut d);
                    supported(&d)
                },
                "care_credit"
            ),
            1_050,
            "{born}"
        );
    }
}

// Form 8862 (unit T2-46): the disallowance record of the More credits page.
fn disallowed(year: u32, finding: DisallowanceFinding) -> DisallowanceFacts {
    DisallowanceFacts { year, finding, reinstated: false }
}
fn eic_disallowed(
    facts: DisallowanceFacts,
    cause: EicDisallowanceCause,
    your_us_home_days: Option<u32>,
) -> EicDisallowance {
    EicDisallowance::Disallowed { facts, cause, your_us_home_days, spouse_us_home_days: None }
}
fn after_disallowance(
    earned_income: EicDisallowance,
    child_credits: CreditDisallowance,
    education_credit: CreditDisallowance,
) -> CreditsAfterDisallowance {
    CreditsAfterDisallowance { earned_income, child_credits, education_credit }
}
// Single filer born 1990-04-10 with one child born 2018-06-15 (`child()`),
// $20,000.00 of W-2 wages and $500.00 withheld: the return of the Form 8862
// fixture below and of its ban boundaries.
fn one_child_eic_draft() -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19900410;
    d.income[0] = wage(1, Owner::Taxpayer, 2_000_000, 50_000);
    add_child(&mut d);
    d
}
fn form_8862_issue(d: &FederalDraft, needle: &str) {
    let r = estimate(d);
    let issue = r
        .issues
        .iter()
        .find(|i| {
            i.section == "More credits"
                && i.message.contains("Form 8862")
                && i.message.contains(needle)
        })
        .unwrap_or_else(|| panic!("{needle}: {r:?}"));
    assert!(!issue.display.key.is_empty() && issue.display.key != "TEXT", "{needle}: {issue:?}");
}
fn form_8862_not_needed(r: &FederalResult, needle: &str) {
    let note = r
        .review_notes
        .iter()
        .find(|note| note.starts_with("Form 8862 is not needed") && note.contains(needle))
        .unwrap_or_else(|| panic!("{needle}: {r:?}"));
    assert!({
        let message = &note.display;
        message.key.starts_with("NOTE_8862_")
    });
    assert!(r.lines.iter().all(|l| !l.id.starts_with("form_8862_")));
}

#[test]
fn eic_after_disallowance_return_matches_hand_computation() {
    // Single filer born 1990-04-10 (age 36), one qualifying child born
    // 2018-06-15 (age 8; son, 365 days in the home, valid SSN, U.S. home).
    // One W-2: box 1 wages $20,000.00, box 2 withholding $500.00. The IRS
    // disallowed the filer's 2023 EIC after an examination (not a math error,
    // no reckless-disregard or fraud finding, not reinstated since), so the
    // 2026 claim needs Form 8862 (Instructions for Form 8862, Rev. 12-2025,
    // Who Must File: "your EIC for a year after 1996 was reduced or
    // disallowed for any reason other than a math or clerical error").
    //
    // Form 1040:
    //   Line 1a wages = $20,000.00; line 9 = line 11 AGI = $20,000.00.
    //   Line 12e standard deduction, single, under 65 = $16,100.00 (Rev.
    //     Proc. 2025-32 Table 8, `standard_base()`).
    //   Line 15 taxable income = $20,000.00 - $16,100.00 = $3,900.00.
    //   Line 16 tax: 2026 Tax Table row "at least 3,900 but less than 3,950"
    //     (tests/fixtures/irs-tax-table-2026.csv line 141) = $393.00
    //     (10% of the $3,925 midpoint, $392.50, rounded).
    //   Schedule 8812: line 4 one child under 17 = $2,200.00 (line 5); MAGI
    //     $20,000 is under the $200,000 single threshold, so no phaseout;
    //     line 14 nonrefundable child credit = min($2,200, tax $393) = $393.00;
    //     line 27 additional child tax credit = the smaller of the unused
    //     credit $2,200 - $393 = $1,807 and the lesser of $1,700 x 1 child
    //     and 15% x ($20,000 - $2,500) = $2,625 -> $1,700.00.
    //   Line 19 = $393.00; line 24a total tax = $393.00 - $393.00 = $0.00.
    //   Schedule EIC / EIC Worksheet (tests/fixtures/irs-eic-2026-draft.csv):
    //     one child, single column, earned income $20,000 -> row
    //     "20000,20050,...,4427" = $4,427; AGI $20,000 -> the same row, $4,427;
    //     line 27a = min($4,427, $4,427) = $4,427.00.
    //   Line 25a withholding = $500.00; line 28 = $1,700.00.
    //   Line 33 total payments = $500 + $4,427 + $1,700 = $6,627.00.
    //   Line 34 refund = $6,627.00 - $0.00 = $6,627.00.
    // Form 8862 (Rev. 12-2025), from the return's own facts:
    //   Part I line 1 = 2026; line 2 the Earned Income Credit box only (the
    //     child credits were never disallowed, so Part III is not completed).
    //   Part II line 3 = No (the finding was not only misreported income);
    //     line 4 = No (neither filer is another taxpayer's qualifying child);
    //     Section A line 5a = the child; line 6 = Yes; line 7 child 1 = 365
    //     days; line 8 skipped (born 2018, alive all year). No Section B.
    let mut d = one_child_eic_draft();
    d.credits.as_mut().unwrap().after_disallowance = Some(after_disallowance(
        eic_disallowed(
            disallowed(2023, DisallowanceFinding::Ineligible),
            EicDisallowanceCause::Other,
            None,
        ),
        CreditDisallowance::Never,
        CreditDisallowance::Never,
    ));
    let r = supported(&d);
    assert_eq!(return_line(&r, "1a"), 2_000_000);
    assert_eq!(return_line(&r, "11a / 11b"), 2_000_000);
    assert_eq!(return_line(&r, "12e"), 1_610_000);
    assert_eq!(return_line(&r, "15"), 390_000);
    assert_eq!(return_line(&r, "16"), 39_300);
    assert_eq!(value(&r, "child_credit"), 393);
    assert_eq!(return_line(&r, "24a"), 0);
    assert_eq!(return_line(&r, "25a"), 50_000);
    assert_eq!(return_line(&r, "27a"), 442_700);
    assert_eq!(return_line(&r, "28"), 170_000);
    assert_eq!(return_line(&r, "33"), 662_700);
    assert_eq!(return_line(&r, "34"), 662_700);
    assert_eq!(value(&r, "form_8862_line_1"), 2026);
    assert_eq!(value(&r, "form_8862_line_2_eic"), 1);
    assert!(!r.lines.iter().any(|l| l.id == "form_8862_line_2_child_credits"));
    assert!(!r.lines.iter().any(|l| l.id == "form_8862_line_2_aotc"));
    assert_eq!(value(&r, "form_8862_line_3"), 0);
    assert_eq!(value(&r, "form_8862_line_4"), 0);
    assert_eq!(value(&r, "form_8862_line_5a"), 1);
    assert_eq!(value(&r, "form_8862_line_6"), 1);
    assert_eq!(value(&r, "form_8862_line_7_child_1"), 365);
    assert!(!r.lines.iter().any(|l| l.id.starts_with("form_8862_line_8")));
    assert!(!r.lines.iter().any(|l| l.id.starts_with("form_8862_line_9")));
    assert!(!r.lines.iter().any(|l| l.id.starts_with("form_8862_line_1")
        && l.id != "form_8862_line_1"
        && !l.id.starts_with("form_8862_line_1_")));
    let tagged = r.lines.iter().find(|l| l.id == "form_8862_line_7_child_1").unwrap();
    assert_eq!(tagged.form_line, Some(FormLine::new(TaxForm::Form8862, "7")));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn two_year_ban_runs_through_the_second_year_after_the_disallowed_year() {
    // i8862 "Do not file Form 8862 for the 2 years after the most recent tax
    // year for which there was a final determination that your ... claim was
    // due to reckless or intentional disregard of the rules"; its example
    // bars 2025 and 2026 after a disallowed 2024 claim. So a 2024 reckless
    // finding bars 2026 (the credit is left out with a note), and a 2023
    // finding's ban (2024 and 2025) has ended: Form 8862 is required and the
    // same $4,427 credit computes.
    let mut d = one_child_eic_draft();
    d.credits.as_mut().unwrap().after_disallowance = Some(after_disallowance(
        eic_disallowed(
            disallowed(2024, DisallowanceFinding::RecklessDisregard),
            EicDisallowanceCause::Other,
            None,
        ),
        CreditDisallowance::Never,
        CreditDisallowance::Never,
    ));
    let banned = supported(&d);
    assert_eq!(return_line(&banned, "27a"), 0);
    assert!(noted(&banned, "NOTE_8862_EIC_BANNED"));
    let EicDisallowance::Disallowed { facts, .. } =
        &mut d.credits.as_mut().unwrap().after_disallowance.as_mut().unwrap().earned_income
    else {
        unreachable!()
    };
    facts.year = 2023;
    let r = supported(&d);
    assert_eq!(return_line(&r, "27a"), 442_700);
    assert_eq!(value(&r, "form_8862_line_1"), 2026);
    assert_eq!(value(&r, "form_8862_line_2_eic"), 1);
}

#[test]
fn ten_year_ban_runs_through_the_tenth_year_after_the_disallowed_year() {
    // The fraud ban is 10 years after the disallowed year: a 2016 finding
    // bars 2017 through 2026, a 2015 finding's ban ended with 2025, and the
    // EIC requirement reaches every year after 1996, so 2015 still needs the
    // form. A banned credit the return does not claim anyway (the AOTC here:
    // no education entry) says nothing.
    let mut d = one_child_eic_draft();
    d.credits.as_mut().unwrap().after_disallowance = Some(after_disallowance(
        eic_disallowed(
            disallowed(2016, DisallowanceFinding::Fraud),
            EicDisallowanceCause::Other,
            None,
        ),
        CreditDisallowance::Never,
        CreditDisallowance::Disallowed(disallowed(2016, DisallowanceFinding::Fraud)),
    ));
    let r = supported(&d);
    assert_eq!(return_line(&r, "27a"), 0);
    assert!(noted(&r, "NOTE_8862_EIC_BANNED"));
    assert!(!noted(&r, "NOTE_8862_AOTC_BANNED"));
    let EicDisallowance::Disallowed { facts, .. } =
        &mut d.credits.as_mut().unwrap().after_disallowance.as_mut().unwrap().earned_income
    else {
        unreachable!()
    };
    facts.year = 2015;
    let r = supported(&d);
    assert_eq!(return_line(&r, "27a"), 442_700);
    assert_eq!(value(&r, "form_8862_line_2_eic"), 1);
}

#[test]
fn childless_eic_after_disallowance_needs_line_9_days() {
    // The childless fixture (childless_eic_return_matches_hand_computation:
    // single, born 1996-06-15, $14,000 wages, $200 interest, EIC $407) after
    // a 2022 disallowance. Section B needs line 9a, the days the filer's main
    // home was in the United States: absent, the estimate stands with a note;
    // 183 (the form's own cutoff: "If you enter less than 183 ... you cannot
    // claim the EIC") passes and fills lines 9a = 183, 10a = 30 (age at the
    // end of 2026, born 1996) and 11a = No (not another's dependent); 182
    // contradicts the filer's "main home in the United States more than half
    // the year" answer and is refused.
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19960615;
    d.income[0] = wage(1, Owner::Taxpayer, 1_400_000, 30_000);
    add(&mut d, income(savings_income(200, 0)));
    let claim = |days| {
        Some(after_disallowance(
            eic_disallowed(
                disallowed(2022, DisallowanceFinding::Ineligible),
                EicDisallowanceCause::Other,
                days,
            ),
            CreditDisallowance::Never,
            CreditDisallowance::Never,
        ))
    };
    // Days not entered: the estimate stands, with a note for the form.
    d.credits.as_mut().unwrap().after_disallowance = claim(None);
    assert!(noted(&supported(&d), "WARN_8862_HOME_DAYS_TAXPAYER"));
    d.credits.as_mut().unwrap().after_disallowance = claim(Some(183));
    let r = supported(&d);
    assert_eq!(return_line(&r, "27a"), 40_700);
    assert_eq!(value(&r, "form_8862_line_9a"), 183);
    assert_eq!(value(&r, "form_8862_line_10a"), 30);
    assert_eq!(value(&r, "form_8862_line_11a"), 0);
    assert!(!r.lines.iter().any(|l| l.id.starts_with("form_8862_line_5")));
    assert!(!r.lines.iter().any(|l| l.id == "form_8862_line_9b"));
    d.credits.as_mut().unwrap().after_disallowance = claim(Some(182));
    form_8862_issue(&d, "line 9a: 182 days contradicts");
    let result = estimate(&d);
    let issue = result
        .issues
        .iter()
        .find(|issue| issue.display.key == "ISSUE_8862_HOME_DAYS_CONTRADICTION_TAXPAYER")
        .unwrap();
    assert_eq!(
        crate::interview::issue_pages(&d, &issue.context, Some(&issue.display.key)),
        alloc::vec![TaxSection::MoreCredits, TaxSection::People]
    );
}

#[test]
fn form_8862_is_not_needed_for_a_math_error_reinstatement_or_pre_2016_child_credit() {
    // i8862 exceptions: a math or clerical error never needs the form; a
    // credit allowed again after Form 8862 with no later disallowance does
    // not; the CTC/ACTC/ODC requirement reaches only years after 2015, so a
    // 2015 child-credit disallowance needs no form while a 2016 one fills
    // Part III (line 12a the child, lines 14-17 Yes). The credits compute
    // unchanged in every case ($4,427 EIC, $393 + $1,700 child credits).
    let mut d = one_child_eic_draft();
    let credits = d.credits.as_mut().unwrap();
    credits.after_disallowance = Some(after_disallowance(
        eic_disallowed(
            disallowed(2023, DisallowanceFinding::MathOrClericalError),
            EicDisallowanceCause::Other,
            None,
        ),
        CreditDisallowance::Never,
        CreditDisallowance::Never,
    ));
    let r = supported(&d);
    assert_eq!(return_line(&r, "27a"), 442_700);
    form_8862_not_needed(&r, "math or clerical error");
    let credits = d.credits.as_mut().unwrap();
    credits.after_disallowance = Some(after_disallowance(
        eic_disallowed(
            DisallowanceFacts {
                year: 2020,
                finding: DisallowanceFinding::Ineligible,
                reinstated: true,
            },
            EicDisallowanceCause::Other,
            None,
        ),
        CreditDisallowance::Disallowed(disallowed(2015, DisallowanceFinding::Ineligible)),
        CreditDisallowance::Never,
    ));
    let r = supported(&d);
    assert_eq!(return_line(&r, "27a"), 442_700);
    assert_eq!(return_line(&r, "28"), 170_000);
    form_8862_not_needed(&r, "allowed again after Form 8862");
    form_8862_not_needed(&r, "before the requirement began");
    let credits = d.credits.as_mut().unwrap();
    credits.after_disallowance = Some(after_disallowance(
        EicDisallowance::Never,
        CreditDisallowance::Disallowed(disallowed(2016, DisallowanceFinding::Ineligible)),
        CreditDisallowance::Never,
    ));
    let r = supported(&d);
    assert_eq!(value(&r, "child_credit"), 393);
    assert_eq!(return_line(&r, "28"), 170_000);
    assert_eq!(value(&r, "form_8862_line_1"), 2026);
    assert_eq!(value(&r, "form_8862_line_2_child_credits"), 1);
    assert!(!r.lines.iter().any(|l| l.id == "form_8862_line_2_eic"));
    assert_eq!(value(&r, "form_8862_line_12a"), 1);
    for line in ["14", "15", "16", "17"] {
        assert_eq!(value(&r, &alloc::format!("form_8862_line_{line}_child_1")), 1, "{line}");
    }
    assert!(!r.lines.iter().any(|l| l.id.starts_with("form_8862_line_13")));
}

#[test]
fn childless_eic_exception_and_income_reporting_finding() {
    // i8862: "You are claiming the EIC without a qualifying child and the
    // only reason your EIC was reduced or disallowed in the earlier year was
    // because it was determined that a child listed on Schedule EIC was not
    // your qualifying child" needs no form; the same cause with a qualifying
    // child on this return fills Part II (line 3 = No). Line 3 = Yes when
    // the only earlier
    // finding was misreported earned or investment income, and the rest of
    // Part II is skipped.
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19960615;
    d.income[0] = wage(1, Owner::Taxpayer, 1_400_000, 30_000);
    d.credits.as_mut().unwrap().after_disallowance = Some(after_disallowance(
        eic_disallowed(
            disallowed(2021, DisallowanceFinding::Ineligible),
            EicDisallowanceCause::ChildNotQualifying,
            None,
        ),
        CreditDisallowance::Never,
        CreditDisallowance::Never,
    ));
    let r = supported(&d);
    assert!(return_line(&r, "27a") > 0);
    form_8862_not_needed(&r, "without a qualifying child");
    let mut with_child = one_child_eic_draft();
    with_child.credits.as_mut().unwrap().after_disallowance =
        d.credits.as_ref().unwrap().after_disallowance.clone();
    let r = supported(&with_child);
    assert_eq!(value(&r, "form_8862_line_3"), 0);
    assert_eq!(value(&r, "form_8862_line_7_child_1"), 365);
    with_child.credits.as_mut().unwrap().after_disallowance = Some(after_disallowance(
        eic_disallowed(
            disallowed(2021, DisallowanceFinding::Ineligible),
            EicDisallowanceCause::IncomeReporting,
            None,
        ),
        CreditDisallowance::Never,
        CreditDisallowance::Never,
    ));
    let r = supported(&with_child);
    assert_eq!(value(&r, "form_8862_line_3"), 1);
    assert!(!r.lines.iter().any(|l| l.id == "form_8862_line_4"));
    assert!(!r.lines.iter().any(|l| l.id == "form_8862_line_7_child_1"));
}

#[test]
fn child_credit_and_aotc_parts_list_each_person() {
    // Head of household with one CTC child and one ODC person (the filer's
    // parent, born 1950, supported, no CTC because 76), $2,000 of the filer's
    // own American Opportunity expenses, and a 2019 disallowance of both the
    // child credits and the AOTC: Part III lists the child on line 12a and
    // the parent on line 13a (lines 16 and 17 Yes for both, 14 and 15 only
    // for the child); Part IV lists the filer as student 1 on line 18a with
    // 19a = Yes and 19b = No (the Form 8863 line 23 confirmation covers the
    // four-prior-years rule). Line 8 refusal: a Schedule EIC child claimed
    // under the whole-year exception who was not born in 2026 needs a date
    // of death the return does not carry.
    let mut d = one_child_eic_draft();
    d.setup.as_mut().unwrap().status = filing_choice(FederalStatus::Head);
    let mut parent = child(d.next_id);
    d.next_id += 1;
    parent.first_name = "Parent".into();
    parent.last_name = "Parent".into();
    parent.birth_date = 19500301;
    parent.relationship = Relationship::Parent;
    d.dependents.push(parent);
    let credits = d.credits.as_mut().unwrap();
    let sid0_10788 = -1; // You, the first person of the catalog
    add_education(
        credits,
        native_education(
            sid0_10788,
            education_expenses(200_000),
            education_choice(EducationMethod::AmericanOpportunity, Answer::Yes),
        ),
    );
    credits.after_disallowance = Some(after_disallowance(
        EicDisallowance::Never,
        CreditDisallowance::Disallowed(disallowed(2019, DisallowanceFinding::Ineligible)),
        CreditDisallowance::Disallowed(disallowed(2019, DisallowanceFinding::Ineligible)),
    ));
    let r = supported(&d);
    assert_eq!(value(&r, "form_8862_line_2_child_credits"), 1);
    assert_eq!(value(&r, "form_8862_line_2_aotc"), 1);
    assert_eq!(value(&r, "form_8862_line_12a"), 1);
    assert_eq!(value(&r, "form_8862_line_13a"), 1);
    assert_eq!(value(&r, "form_8862_line_14_child_1"), 1);
    assert_eq!(value(&r, "form_8862_line_15_child_1"), 1);
    assert_eq!(value(&r, "form_8862_line_16_child_1"), 1);
    assert_eq!(value(&r, "form_8862_line_17_child_1"), 1);
    assert_eq!(value(&r, "form_8862_line_16_other_dependent_1"), 1);
    assert_eq!(value(&r, "form_8862_line_17_other_dependent_1"), 1);
    assert!(!r.lines.iter().any(|l| l.id == "form_8862_line_14_other_dependent_1"));
    assert_eq!(value(&r, "form_8862_line_18a"), 1);
    assert_eq!(value(&r, "form_8862_line_19a_student_1"), 1);
    assert_eq!(value(&r, "form_8862_line_19b_student_1"), 0);
    // A whole-year child born before 2026 with no date of death prints no
    // line 8 (the old refusal needed a date of death the record now carries).
    d.credits.as_mut().unwrap().after_disallowance = Some(after_disallowance(
        eic_disallowed(
            disallowed(2019, DisallowanceFinding::Ineligible),
            EicDisallowanceCause::Other,
            None,
        ),
        CreditDisallowance::Never,
        CreditDisallowance::Never,
    ));
    d.dependents[0].residency = DependentResidency::WholeYearException;
    let r = supported(&d);
    assert!(!r.lines.iter().any(|l| l.id.starts_with("form_8862_line_8")));
    d.dependents[0].birth_date = 20260305;
    let r = supported(&d);
    assert_eq!(value(&r, "form_8862_line_7_child_1"), 365);
    assert_eq!(value(&r, "form_8862_line_8_child_1_birth"), 305);
}

// Form 8814 (src/child_election.rs): the map's fixtures A-E. Statement
// amounts are whole dollars, rounded half up at each fraction, and the tax is
// figured from the 2026 Tax Table midpoints the way the other fixtures are.
fn joint_with_child(wages: i64, birth_date: u32) -> (FederalDraft, i64) {
    let mut d = complete(FederalStatus::Joint);
    d.income = vec![wage(1, Owner::Taxpayer, wages * 100, 0)];
    let id = d.next_id;
    add_child(&mut d);
    d.dependents[0].birth_date = birth_date;
    (d, id)
}

#[test]
fn child_election_below_the_no_tax_amount_reports_nothing() {
    // A: joint, wages $90,000, one child (10) with $1,300 of interest.
    // Line 4 = 1,300, not over $1,350: the child need not file and nothing is
    // reported. Taxable 90,000 - 32,200 = 57,800; table midpoint 57,825:
    // 2,480 + 0.12 x 33,025 = 6,443 = line 16.
    let (mut d, id) = joint_with_child(90_000, 20160615);
    add(&mut d, child_election(id, vec![child_interest(1_300)], 0, OtherParentChoice::NotNeeded));
    let r = supported(&d);
    assert!(noted(&r, "NOTE_CHILD_ELECTION_NOT_REQUIRED"));
    assert_eq!(return_line(&r, "16") / 100, 6_443);
    assert_eq!(return_line(&r, "11a / 11b") / 100, 90_000);
    for id in ["child_election_tax", "child_election_other_income"] {
        assert!(r.lines.iter().all(|l| l.id != id), "{id}");
    }
    assert!(r.lines.iter().all(|l| !l.id.starts_with("child_election_")));
}

#[test]
fn child_election_in_the_ten_percent_band_adds_only_the_tax() {
    // B: the same parents, the child's interest $2,000. Line 4 = 2,000, at
    // most $2,700: lines 6-12 are zero, so nothing joins income. Line 14 =
    // 2,000 - 1,350 = 650, under $1,350: line 15 = 10% x 650 = 65.
    // AGI stays 90,000; line 16 = 6,443 + 65 = 6,508.
    let (mut d, id) = joint_with_child(90_000, 20160615);
    add(&mut d, child_election(id, vec![child_interest(2_000)], 0, OtherParentChoice::NotNeeded));
    let r = supported(&d);
    assert!(!noted(&r, "NOTE_CHILD_ELECTION_NOT_REQUIRED"));
    assert_eq!(return_line(&r, "11a / 11b") / 100, 90_000);
    assert_eq!(return_line(&r, "16") / 100, 6_508);
    assert_eq!(value(&r, "child_election_tax"), 65);
    assert_eq!(value(&r, "child_election_other_income"), 0);
    let row = |line: &str| -> i64 {
        let key = r
            .lines
            .iter()
            .find(|l| {
                l.form_line.as_ref().is_some_and(|f| f.form == TaxForm::Form8814 && f.line == line)
            })
            .unwrap_or_else(|| panic!("no Form 8814 line {line}"));
        key.amount / 100
    };
    assert_eq!((row("1a"), row("4"), row("14"), row("15")), (2_000, 2_000, 650, 65));
    // The review shows the form's lines among the return lines, one copy for
    // each child's election.
    let rows = crate::draft_return(&r);
    let shown = |line: &str| {
        rows.iter()
            .find(|row| row.form_line == Some(FormLine::new(TaxForm::Form8814, line)))
            .unwrap_or_else(|| panic!("no Form 8814 line {line} among the return lines"))
            .amount
    };
    assert_eq!((shown("4"), shown("15")), (200_000, 6_500));
    // Lines 6 to 12 have nothing on them and are left off.
    assert!(r.lines.iter().all(|l| {
        l.form_line.as_ref().is_none_or(|f| {
            f.form != TaxForm::Form8814 || !["6", "9", "10", "11", "12"].contains(&f.line.as_str())
        })
    }));
}

#[test]
fn child_election_with_qualified_dividends_matches_the_i8814_example() {
    // C: the i8814 Fred example. Child (6): 1099-DIV box 1a $2,190, box 1b
    // $2,190, box 2a $610. Line 4 = 2,190 + 610 = 2,800; line 6 = 100;
    // line 9 = 100 x 2,190 / 2,800 = 78.2 -> 78; line 10 = 100 x 610 / 2,800
    // = 21.8 -> 22; line 11 = 100; line 12 = 0; line 14 = 1,450, not under
    // 1,350: line 15 = 135.
    // Form 1040: 3a = 3b = 78, 7a = 22, line 8 = 0, AGI 90,100, taxable
    // 57,900. Qualified Dividends and Capital Gain Tax Worksheet: preferential
    // 78 + 22 = 100 at 0%, ordinary 57,800 -> 6,443; line 16 = 6,443 + 135.
    let (mut d, id) = joint_with_child(90_000, 20200615);
    add(
        &mut d,
        child_election(
            id,
            vec![child_dividends(2_190, 2_190, 610)],
            0,
            OtherParentChoice::NotNeeded,
        ),
    );
    let r = supported(&d);
    assert_eq!(return_line(&r, "3a") / 100, 78);
    assert_eq!(return_line(&r, "3b") / 100, 78);
    assert_eq!(return_line(&r, "7a") / 100, 22);
    assert_eq!(return_line(&r, "8") / 100, 0);
    assert_eq!(return_line(&r, "11a / 11b") / 100, 90_100);
    assert_eq!(return_line(&r, "15") / 100, 57_900);
    assert_eq!(return_line(&r, "16") / 100, 6_578);
    assert_eq!(value(&r, "child_election_qualified_dividends"), 78);
    assert_eq!(value(&r, "child_election_capital_gain_distributions"), 22);
    assert_eq!(value(&r, "child_election_other_income"), 0);
    assert_eq!(value(&r, "child_election_tax"), 135);
    // Schedule B is not required: 78 is not over $1,500.
    assert!(r.lines.iter().all(|l| l.id != "schedule_b_ordinary_dividends"));
    // The row's lines are Form 8814 lines.
    let entry = d
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::ChildElection { .. }))
        .unwrap()
        .id;
    assert_eq!(
        printed_line(&r, &alloc::format!("child_election_{entry}_qualified_dividends_reported")),
        (TaxForm::Form8814, "9".into())
    );
    assert_eq!(value(&r, &alloc::format!("child_election_{entry}_reported_total")), 100);
    // The preview is line 6, the income the row adds to the return.
    assert_eq!(
        crate::interview::document_amount(&d.income.last().unwrap().income, false),
        Some(10_000)
    );
}

#[test]
fn child_election_two_children_share_the_28_percent_gain_and_net_with_schedule_d() {
    // D: head of household, wages $60,000, a 2025 long-term capital loss
    // carryover of $1,000 (a $4,000 loss, $3,000 deducted).
    // Child A (10): 1099-INT box 1 $3,000, box 8 $500, box 9 $200.
    //   Line 4 = 3,000; line 6 = 300; no dividends or distributions, so line 9
    //   = line 10 = 0 and line 12 = 300; line 14 = 1,650, not under 1,350:
    //   line 15 = 135. Line 1b = 500 (Pub 596 Worksheet 1 line 2); the $200 of
    //   private-activity interest is a Form 6251 line 2g preference.
    // Child B (15): 1099-INT box 1 $1,000; 1099-DIV box 1a $1,200, box 1b
    //   $800, box 2a $2,000, box 2d $500. Line 4 = 1,000 + 1,200 + 2,000 =
    //   4,200; line 6 = 1,500; line 9 = 1,500 x 800 / 4,200 = 285.7 -> 286;
    //   line 10 = 1,500 x 2,000 / 4,200 = 714.3 -> 714; line 11 = 1,000;
    //   line 12 = 500; line 14 = 2,850 -> line 15 = 135. The 28% share of line
    //   10 = 714 x 500 / 2,000 = 178.5 -> 179.
    // Sums: 3a = 3b = 286; Schedule D line 13 = 714; Schedule 1 line 8z = 300
    // + 500 = 800; line 16 adds 270.
    // Schedule D: line 15 = 714 - 1,000 = -286, so line 7a = -286 and line 18
    // = 0 (179 - 1,000 < 0). Total income = 60,000 + 286 - 286 + 800 = 60,800;
    // taxable 60,800 - 24,150 = 36,650. Worksheet: line 3 = 0 (Schedule D is
    // negative), preferential 286 at 0%, ordinary 36,364 -> table midpoint
    // 36,375: 1,770 + 0.12 x 18,675 = 4,011; line 16 = 4,011 + 270 = 4,281.
    let mut d = complete(FederalStatus::Head);
    d.income = vec![wage(1, Owner::Taxpayer, 6_000_000, 0)];
    let a = d.next_id;
    add_child(&mut d);
    d.dependents[0].birth_date = 20160615;
    let b = d.next_id;
    add_child(&mut d);
    d.dependents[1].birth_date = 20110615;
    let mut history = capital_history();
    history.status = FederalStatus::Head;
    history.agi = 8_000_000;
    history.deductions = 2_000_000;
    history.short = 0;
    history.long = -400_000;
    history.deducted_loss = 300_000;
    history.investment_parts = Some(InvestmentCapitalParts { short: 0, long: 100_000, amt: None });
    d.screening.as_mut().unwrap().capital_history = vec![history];
    let mut interest = child_interest(3_000);
    if let ChildIncome::Interest { data, .. } = &mut interest {
        data.exempt = 50_000;
        data.private_activity = 20_000;
    }
    add(&mut d, child_election(a, vec![interest], 0, OtherParentChoice::LivedApart));
    let mut dividends = child_dividends(1_200, 800, 2_000);
    if let ChildIncome::Dividend { data, .. } = &mut dividends {
        data.form.collectible_distributions = Some(50_000);
    }
    add(
        &mut d,
        child_election(b, vec![child_interest(1_000), dividends], 0, OtherParentChoice::LivedApart),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "child_election_qualified_dividends"), 286);
    assert_eq!(value(&r, "child_election_capital_gain_distributions"), 714);
    assert_eq!(value(&r, "child_election_other_income"), 800);
    assert_eq!(value(&r, "child_election_tax"), 270);
    assert_eq!(value(&r, "child_election_exempt_interest"), 500);
    assert_eq!(return_line(&r, "3a") / 100, 286);
    assert_eq!(return_line(&r, "3b") / 100, 286);
    assert_eq!(return_line(&r, "7a") / 100, -286);
    assert_eq!(value(&r, "schedule_1_other_income"), 800);
    assert_eq!(return_line(&r, "11a / 11b") / 100, 60_800);
    assert_eq!(return_line(&r, "15") / 100, 36_650);
    assert_eq!(return_line(&r, "16") / 100, 4_281);
    // Each child's election is its own copy of Form 8814 in the review.
    let tax_rows: alloc::vec::Vec<i64> = crate::draft_return(&r)
        .iter()
        .filter(|row| row.form_line == Some(FormLine::new(TaxForm::Form8814, "15")))
        .map(|row| row.amount)
        .collect();
    assert_eq!(tax_rows, vec![13_500, 13_500]);
    // Form 8960 line 7 and Form 4952 investment income are line 12 (no Alaska
    // dividend); the earned income credit's investment income adds the
    // tax-exempt interest (500), line 12 (800) and the dividends (286).
    assert_eq!(value(&r, "child_election_investment_income"), 800);
    assert_eq!(value(&r, "eic_investment_income"), 500 + 800 + 286);
    // The 28% share reaches the child B row's Schedule D worksheet inputs.
    let lines_b = match &d.income.last().unwrap().income {
        FederalIncome::ChildElection { data } => data.lines(),
        _ => unreachable!(),
    };
    assert_eq!((lines_b.line_9, lines_b.line_10, lines_b.collectible), (286, 714, 179));
    // The private-activity interest of child A is an AMT preference of $200.
    let mut without = d.clone();
    for entry in &mut without.income {
        if let FederalIncome::ChildElection { data } = &mut entry.income {
            for statement in &mut data.statements {
                if let ChildIncome::Interest { data, .. } = statement {
                    data.private_activity = 0;
                }
            }
        }
    }
    assert_eq!(
        value(&r, "alternative_minimum_taxable_income")
            - value(&supported(&without), "alternative_minimum_taxable_income"),
        200
    );
}

#[test]
fn child_election_alaska_dividend_is_ordinary_but_not_investment_income() {
    // E: single, wages $40,000, a child (12) with $2,000 of interest and a
    // $1,700 Alaska Permanent Fund dividend. Line 2a = 1,700; line 4 =
    // 3,700; line 6 = 1,000; no qualified dividends or distributions, so line
    // 12 = 1,000; line 14 = 2,350 -> line 15 = 135. The Alaska share =
    // 1,000 x 1,700 / (2,000 + 1,700 - 0) = 459.5 -> 459; investment income
    // (Form 8960 line 7, Form 4952, EIC Worksheet 1 line 4) = 1,000 - 459 =
    // 541. AGI 41,000; taxable 41,000 - 16,100 = 24,900 -> midpoint 24,925:
    // 1,240 + 0.12 x 12,525 = 2,743; line 16 = 2,743 + 135 = 2,878.
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 4_000_000, 0)];
    let id = d.next_id;
    add_child(&mut d);
    d.dependents[0].birth_date = 20140615;
    add(
        &mut d,
        child_election(id, vec![child_interest(2_000)], 1_700, OtherParentChoice::LivedApart),
    );
    let r = supported(&d);
    assert_eq!(return_line(&r, "11a / 11b") / 100, 41_000);
    assert_eq!(return_line(&r, "16") / 100, 2_878);
    assert_eq!(value(&r, "child_election_other_income"), 1_000);
    assert_eq!(value(&r, "child_election_investment_income"), 541);
    assert_eq!(value(&r, "eic_investment_income"), 541);
    // The same dividend reported on the child's own Form 1099-DIV would have
    // no share to remove; the share is the Alaska dividend's part of line 12.
    let id = d.income.last().unwrap().id;
    assert_eq!(value(&r, &alloc::format!("child_election_{id}_ordinary_dividends")), 1_700);
}

#[test]
fn child_election_omissions_leave_the_child_out_with_one_notice() {
    // The election is optional, so every omission leaves the parent's return
    // correct without it, with one notice naming the child, and never
    // changes the state of the estimate.
    let (baseline, id) = joint_with_child(90_000, 20160615);
    let base = supported(&baseline);
    let omit = |edit: &dyn Fn(&mut FederalDraft, &mut FederalChildElection), key: &str| {
        let (mut d, id) = joint_with_child(90_000, 20160615);
        add(
            &mut d,
            child_election(id, vec![child_dividends(3_000, 0, 0)], 0, OtherParentChoice::NotNeeded),
        );
        let mut data = child_election_mut(&mut d).clone();
        edit(&mut d, &mut data);
        *child_election_mut(&mut d) = data;
        let r = supported(&d);
        assert!(noted(&r, key), "{key}: {:?}", r.review_notes);
        assert_eq!(
            r.review_notes.iter().filter(|n| n.display.key.contains("CHILD_ELECTION")).count(),
            1,
            "{key}"
        );
        assert_eq!(return_line(&r, "16"), return_line(&base, "16"), "{key} reports nothing");
        assert!(r.lines.iter().all(|l| !l.id.starts_with("child_election_")));
        r
    };
    omit(
        &|_, e| e.statements = vec![child_interest(13_500)],
        "WARN_CHILD_ELECTION_INCOME_TOO_HIGH_LEFT_OUT",
    );
    omit(
        &|_, e| {
            if let ChildIncome::Dividend { data, .. } = &mut e.statements[0] {
                data.form.withholding = Some(100);
            }
        },
        "WARN_CHILD_ELECTION_WITHHOLDING_LEFT_OUT",
    );
    omit(
        &|_, e| {
            if let ChildIncome::Dividend { data, .. } = &mut e.statements[0] {
                data.special = crate::DividendSpecial::Foreign;
            }
        },
        "WARN_CHILD_ELECTION_SPECIAL_LEFT_OUT",
    );
    omit(
        &|_, e| e.facts = crate::ChildElectionFacts::NOMINEE.0,
        "WARN_CHILD_ELECTION_SPECIAL_LEFT_OUT",
    );
    omit(
        &|_, e| e.facts = crate::ChildElectionFacts::OTHER_INCOME.0,
        "WARN_CHILD_ELECTION_INELIGIBLE_LEFT_OUT",
    );
    omit(
        &|_, e| e.facts = crate::ChildElectionFacts::ESTIMATED_PAYMENTS.0,
        "WARN_CHILD_ELECTION_INELIGIBLE_LEFT_OUT",
    );
    // A joint return whose child lived with the couple 150 days: the
    // noncustodial parent cannot elect.
    omit(
        &|d, _| d.dependents[0].residency = DependentResidency::Days { days: 150 },
        "WARN_CHILD_ELECTION_NONCUSTODIAL_LEFT_OUT",
    );
    // A child who is no longer one the return can report.
    omit(&|d, _| d.dependents[0].birth_date = 20070101, "WARN_CHILD_ELECTION_INELIGIBLE_LEFT_OUT");
    omit(
        &|d, _| d.dependents[0].relationship = Relationship::Foster,
        "WARN_CHILD_ELECTION_INELIGIBLE_LEFT_OUT",
    );
    let _ = id;
    // Separate parents: the other parent's return, and not being sure.
    let separate = |choice: OtherParentChoice, days: u32, key: &str| {
        let mut d = complete(FederalStatus::Single);
        d.income = vec![wage(1, Owner::Taxpayer, 6_000_000, 0)];
        let id = d.next_id;
        add_child(&mut d);
        d.dependents[0].birth_date = 20160615;
        d.dependents[0].residency = DependentResidency::Days { days };
        add(&mut d, child_election(id, vec![child_dividends(3_000, 0, 0)], 0, choice));
        let r = supported(&d);
        assert!(noted(&r, key), "{key}");
        assert!(r.lines.iter().all(|l| !l.id.starts_with("child_election_")));
    };
    separate(
        OtherParentChoice::TogetherOtherHigher,
        365,
        "WARN_CHILD_ELECTION_OTHER_PARENT_LEFT_OUT",
    );
    separate(OtherParentChoice::Unresolved, 365, "WARN_CHILD_ELECTION_PARENT_ASSUMED");
    separate(OtherParentChoice::LivedApart, 100, "WARN_CHILD_ELECTION_NONCUSTODIAL_LEFT_OUT");
    // Living together with the higher taxable income keeps the child.
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 6_000_000, 0)];
    let id = d.next_id;
    add_child(&mut d);
    d.dependents[0].birth_date = 20160615;
    add(
        &mut d,
        child_election(
            id,
            vec![child_dividends(3_000, 0, 0)],
            0,
            OtherParentChoice::TogetherYouHigher,
        ),
    );
    let r = supported(&d);
    assert!(r.review_notes.iter().all(|n| !n.display.key.contains("CHILD_ELECTION")));
    assert_eq!(value(&r, "child_election_other_income"), 300);
}

#[test]
fn child_election_foreign_account_keeps_the_election_with_a_note() {
    let (mut d, id) = joint_with_child(90_000, 20160615);
    add(&mut d, child_election(id, vec![child_interest(2_000)], 0, OtherParentChoice::NotNeeded));
    child_election_mut(&mut d).facts = crate::ChildElectionFacts::FOREIGN_ACCOUNT.0;
    let r = supported(&d);
    assert!(noted(&r, "NOTE_CHILD_FOREIGN_ACCOUNT"));
    assert_eq!(value(&r, "child_election_tax"), 65);
}

#[test]
fn child_election_is_independent_of_form_8615() {
    // The parent who elects is not a young filer: no Form 8615 line or parent
    // page is involved, and the election's tax stays out of Form 8615's
    // line 16 inputs. The brothers-and-sisters line of a child's own Form
    // 8615 excludes a child the parent reported on Form 8814.
    let (mut d, id) = joint_with_child(90_000, 20160615);
    add(&mut d, child_election(id, vec![child_interest(4_000)], 0, OtherParentChoice::NotNeeded));
    let r = supported(&d);
    assert_eq!(value(&r, "child_election_tax"), 135);
    assert!(r.lines.iter().all(|l| !l.id.starts_with("kiddie_tax_")));
    assert_ne!(crate::federal_rules::form_8615_applies(&r), Some(true));
    let kiddie = concat!(
        include_str!("../../family/kiddie.rs"),
        include_str!("../../family/parent_tax_source.rs"),
    );
    assert!(
        kiddie
            .matches("a brother or sister whose income your parent reported on Form 8814")
            .count()
            >= 2
    );
}

#[test]
fn child_election_investment_income_reaches_form_8960_and_form_4952() {
    // Single, wages $250,000 (above the $200,000 threshold), a child (8) with
    // $4,000 of interest and a $1,000 Alaska Permanent Fund dividend.
    // Line 4 = 5,000; line 6 = 2,300; line 12 = 2,300 (no qualified dividends
    // or distributions). Alaska share = 2,300 x 1,000 / (4,000 + 1,000) = 460;
    // investment income = 2,300 - 460 = 1,840: Form 8960 line 7, net
    // investment income (3.8% of 1,840 = 69.92) and Form 4952 investment
    // income.
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 25_000_000, 0)];
    let id = d.next_id;
    add_child(&mut d);
    add(
        &mut d,
        child_election(id, vec![child_interest(4_000)], 1_000, OtherParentChoice::LivedApart),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "child_election_other_income"), 2_300);
    assert_eq!(value(&r, "child_election_investment_income"), 1_840);
    assert_eq!(value(&r, "form_8960_other_modifications"), 1_840);
    assert_eq!(value(&r, "form_8960_net_investment_income"), 1_840);
    assert_eq!(value(&r, "net_investment_income_tax"), 70);
    assert_eq!(printed_line(&r, "form_8960_other_modifications"), (TaxForm::Form8960, "7".into()));
    // Form 4952: $5,000 of interest paid is allowed up to the investment
    // income, 1,840, and 3,160 carries forward.
    investment_borrowing(&mut d, 5_000, 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "investment_interest_ordinary_income"), 1_840);
    assert_eq!(value(&r, "investment_interest_deduction"), 1_840);
    assert_eq!(value(&r, "investment_interest_carryforward"), 3_160);
}

#[test]
fn child_election_qualified_dividends_can_be_elected_on_form_4952() {
    // A child's $5,000 of qualified dividends put $2,300 on line 3a (Form 8814 line 6); Pub. 550
    // lets the parents choose to include all or part of it in investment income (F9, II-D1). An
    // election within it is accepted and an election beyond it is refused.
    let (mut d, id) = joint_with_child(90_000, 20200615);
    add(
        &mut d,
        child_election(id, vec![child_dividends(5_000, 5_000, 0)], 0, OtherParentChoice::NotNeeded),
    );
    investment_borrowing(&mut d, 500, 0, 0);
    d.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().election =
        InvestmentElection::CapitalFirst { amount: 500 * 100 };
    let r = estimate(&d);
    assert_ne!(r.state, ResultState::Unsupported, "{:#?}", r.issues);
    assert!(
        !r.issues
            .iter()
            .any(|i| { i.display.key.contains("ELECTED_GAINS_AND_DIVIDENDS_CANNOT_EXCEED") })
    );
    d.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().election =
        InvestmentElection::CapitalFirst { amount: 5_000 * 100 };
    let r = estimate(&d);
    assert!(
        r.issues
            .iter()
            .any(|i| { i.display.key.contains("ELECTED_GAINS_AND_DIVIDENDS_CANNOT_EXCEED") })
    );
}

// Forms 8332 and 2120 unit (2026-10-09). IRC section 152(e) lets a custodial
// parent release the claim to a child to the other parent; IRC section
// 152(d)(3) lets one of several people who together support a relative claim
// them. Fixtures A to E follow the unit's map (form-8332-2120.md, section 6);
// synthetic people, whole dollars, 2026 amounts: standard deduction $16,100
// single and $24,150 head of household, Tax Table midpoints (10% bracket to
// $12,400 single and $17,700 head), CTC $2,200, ODC $500, EIC maximum $4,427
// for one child with a 15.98% phase-out from $23,890, and Form 2441 rate
// max(35, 50 - ceil((AGI - 15,000) / 2,000)) percent. No withholding.
fn released_child(release: CustodyRelease, days: u32) -> FederalDependent {
    let mut c = child(2);
    c.first_name = "Ava".into();
    c.last_name = "Sample".into();
    c.residency = DependentResidency::Days { days };
    c.us_home_over_half_year = days >= 183;
    c.situation = DependentSituation::Supported {
        your_support_over_half: false,
        gross_income: c.gross_income(),
    };
    set_release(&mut c, release);
    c
}
fn single_with_release(wages: i64, c: FederalDependent) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, wages * 100, 0);
    d.dependents.push(c);
    d.next_id = 3;
    d
}
fn head_with_release(wages: i64, c: FederalDependent) -> FederalDraft {
    let mut d = complete(FederalStatus::Head);
    d.income[0] = wage(1, Owner::Taxpayer, wages * 100, 0);
    d.dependents.push(c);
    d.next_id = 3;
    d
}

// Fixture A: the noncustodial parent holding the release. Single, wages
// $40,000; Ava lived with the filer 3 months (90 days) and the other parent
// released the claim to the filer. AGI 40,000; taxable 40,000 - 16,100 =
// 23,900, Tax Table midpoint 23,925: 1,240 + 12% x 11,525 = 2,623. The child
// is a qualifying child for the credit with the residency test waived (section
// 152(e)(1)): CTC 2,200 <= 2,623, so the nonrefundable credit is 2,200 and
// the additional child tax credit 0. She did not live with the filer over half
// the year, so there is no earned income credit (self-only at AGI 40,000: 0).
// Total tax 2,623 - 2,200 = 423, owed 423. Without the release Ava is no one's
// dependent on this return (residency fails) and the tax is 2,623.
#[test]
fn noncustodial_parent_with_a_release_gets_the_child_tax_credit_only() {
    let d = single_with_release(40_000, released_child(CustodyRelease::ReleasedToYou, 90));
    let r = supported(&d);
    assert_eq!(value(&r, "regular_income_tax"), 2623);
    assert_eq!(value(&r, "child_credit"), 2200);
    assert_eq!(value(&r, "additional_child_credit"), 0);
    assert_eq!(value(&r, "earned_income_credit"), 0);
    assert_eq!(r.tax, Some(42_300));
    assert_eq!(r.balance, Some(42_300));
    assert!(noted(&r, "NOTE_FORM_8332_ATTACH"), "{r:?}");
    assert!(!noted(&r, "NOTE_FORM_2120_ATTACH"));
    // The note names the child and belongs to the child's row.
    let note = r.review_notes.iter().find(|n| n.display.key == "NOTE_FORM_8332_ATTACH").unwrap();
    assert_eq!(
        note.display.args,
        vec![crate::MessageArg::Encoded(crate::interview::person_name("Ava", "Sample"))]
    );
    assert_eq!(
        note.context,
        Some(
            crate::IssueContext::Section(TaxSection::Children)
                .item("dependents", 0)
                .named_encoded(&crate::interview::person_name("Ava", "Sample"))
        )
    );
    // Without the release: no dependent, no credit, no note.
    let d = single_with_release(40_000, released_child(CustodyRelease::None, 90));
    let r = supported(&d);
    assert_eq!(value(&r, "child_credit"), 0);
    assert_eq!(r.tax, Some(262_300));
    assert!(!noted(&r, "NOTE_FORM_8332_ATTACH"));
    // Head of household: the child did not live with the filer over half the
    // year, so the release never gives the status (section 2(b)(1)(A)(i)
    // reads section 152(c) without section 152(e)); the same return filed as
    // head of household is refused on the filing status and the people page.
    let d = head_with_release(40_000, released_child(CustodyRelease::ReleasedToYou, 90));
    unsupported(&d, "Filing status");
    unsupported(&d, "Children and people you support");
    // The dependent care credit: the noncustodial parent has no qualifying
    // person (section 21(e)(5)), so $1,000 of care costs for Ava are left out
    // with the existing notice.
    let mut d = single_with_release(40_000, released_child(CustodyRelease::ReleasedToYou, 90));
    d.dependents[0].set_care_expenses(100_000);
    let r = supported(&d);
    assert_eq!(value(&r, "care_credit"), 0);
    assert!(noted(&r, "WARN_CARE_COSTS_LEFT_OUT"), "{r:?}");
    assert_eq!(r.tax, Some(42_300));
}

// Fixture B: the custodial parent who released. Head of household, wages
// $45,000; Ava lived all year, care costs $2,000, YouReleased. AGI 45,000;
// taxable 45,000 - 24,150 = 20,850, midpoint 20,875: 1,770 + 12% x 3,175 =
// 2,151. Head of household stands (section 2(b)(1)(A)(i): the qualifying
// child without section 152(e)); the child is not the filer's dependent, so
// no CTC or ODC. Form 2441: expenses min(2,000, 3,000) = 2,000 at
// max(35, 50 - 15) = 35%: 700 (section 21(e)(5) keeps the custodial parent's
// qualifying individual). Tax 2,151 - 700 = 1,451. EIC with one child at
// AGI 45,000 (table row 45,000-45,050, midpoint 45,025): 4,427 - 15.98% x
// (45,025 - 23,890) = 4,427 - 3,377.4 = 1,050 (the earned income credit
// ignores section 152(e), section 32(c)(3)(A)). Owed 1,451 - 1,050 = 401.
// Without the release: CTC 2,200 is limited to the 1,451 of tax, ACTC
// min(749, 1,700, 15% x (45,000 - 2,500)) = 749, tax 0, refund 749 + 1,050 =
// 1,799.
#[test]
fn custodial_parent_who_released_keeps_head_of_household_eic_and_care() {
    let mut c = released_child(CustodyRelease::YouReleased, 365);
    c.set_care_expenses(200_000);
    let mut d = head_with_release(45_000, c.clone());
    let r = {
        document_care(&mut d);
        supported(&d)
    };
    assert_eq!(value(&r, "deduction"), 24150);
    assert_eq!(value(&r, "regular_income_tax"), 2151);
    assert_eq!(value(&r, "child_credit"), 0);
    assert_eq!(value(&r, "care_credit"), 700);
    assert_eq!(value(&r, "earned_income_credit"), 1050);
    assert_eq!(r.tax, Some(145_100));
    assert_eq!(r.balance, Some(40_100));
    assert!(!noted(&r, "NOTE_FORM_8332_ATTACH"));
    // The same return without the release.
    set_release(&mut c, CustodyRelease::None);
    let mut without_release = head_with_release(45_000, c);
    document_care(&mut without_release);
    let r = supported(&without_release);
    assert_eq!(value(&r, "child_credit"), 2200 - 749);
    assert_eq!(value(&r, "additional_child_credit"), 749);
    assert_eq!(value(&r, "earned_income_credit"), 1050);
    assert_eq!(r.tax, Some(0));
    assert_eq!(r.refund, Some(179_900));
}

// Fixture C: a multiple support agreement. Single, wages $50,000; Dana
// Sample, the filer's parent born 1950-03-10, did not live with the filer,
// the filer did not pay over half of her support, her gross income is
// $3,000, and the filer holds the Form 2120 statements. AGI 50,000; taxable
// 33,900, midpoint 33,925: 1,240 + 12% x 21,525 = 3,823. A qualifying
// relative by section 152(d)(3) (3,000 < 5,300): the credit for other
// dependents 500, so tax 3,323. At $5,300 of gross income she is not a
// dependent: 3,823. As head of household she gives no status (section
// 2(b)(3)(B)(ii): a person claimed under a multiple support agreement).
fn dana(gross_income: i64) -> FederalDependent {
    let mut c = child(2);
    c.first_name = "Dana".into();
    c.last_name = "Sample".into();
    c.relationship = Relationship::Parent;
    c.birth_date = 19500310;
    c.residency = DependentResidency::Days { days: 0 };
    c.us_home_over_half_year = false;
    c.situation = DependentSituation::Supported { your_support_over_half: false, gross_income };
    set_fact(&mut c, PersonFacts::MULTIPLE_SUPPORT, true);
    c
}
#[test]
fn multiple_support_agreement_gives_the_credit_for_other_dependents_not_head_of_household() {
    let r = supported(&single_with_release(50_000, dana(300_000)));
    assert_eq!(value(&r, "regular_income_tax"), 3823);
    assert_eq!(value(&r, "child_credit"), 500);
    assert_eq!(r.tax, Some(332_300));
    assert!(noted(&r, "NOTE_FORM_2120_ATTACH"), "{r:?}");
    assert!(!noted(&r, "NOTE_FORM_8332_ATTACH"));
    // The gross-income test still applies.
    let r = supported(&single_with_release(50_000, dana(530_000)));
    assert_eq!(value(&r, "child_credit"), 0);
    assert_eq!(r.tax, Some(382_300));
    assert!(!noted(&r, "NOTE_FORM_2120_ATTACH"), "{r:?}");
    // Without the agreement the support test fails.
    let mut c = dana(300_000);
    set_fact(&mut c, PersonFacts::MULTIPLE_SUPPORT, false);
    let r = supported(&single_with_release(50_000, c));
    assert_eq!(value(&r, "child_credit"), 0);
    // Head of household: Dana does not live with the filer and the agreement
    // excludes her as a qualifying person.
    unsupported(&head_with_release(50_000, dana(300_000)), "Filing status");
    // Control: a parent the filer supports over half qualifies the filer.
    let mut c = dana(300_000);
    set_fact(&mut c, PersonFacts::MULTIPLE_SUPPORT, false);
    c.situation = DependentSituation::Supported {
        your_support_over_half: true,
        gross_income: c.gross_income(),
    };
    let r = supported(&head_with_release(50_000, c));
    assert_eq!(value(&r, "child_credit"), 500);
}

// Fixture D: a released child aged 17 is the filer's credit for other
// dependents, not the child tax credit. Fixture A with Ava born 2009-03-01
// (17 on December 31, 2026): 2,623 - 500 = 2,123.
#[test]
fn released_child_aged_17_gets_the_credit_for_other_dependents() {
    let mut c = released_child(CustodyRelease::ReleasedToYou, 90);
    c.birth_date = 20090301;
    let r = supported(&single_with_release(40_000, c));
    assert_eq!(value(&r, "regular_income_tax"), 2623);
    assert_eq!(value(&r, "child_credit"), 500);
    assert_eq!(r.tax, Some(212_300));
}

// Fixture E: a released qualifying relative. Fixture A with a child born
// 2005-05-01 (20, not a student, not disabled) who is "Another person you
// support": the filer did not pay over half of her support and her gross
// income is $2,000. She is not a qualifying child (too old), so the release
// waives the support test of the qualifying relative (section 152(e)(1)(A),
// (d)(1)(C)): the credit for other dependents 500, tax 2,623 - 500 = 2,123.
// At the gross-income limit $5,300 she is no one's dependent.
#[test]
fn released_qualifying_relative_aged_20_gets_the_credit_for_other_dependents() {
    let mut c = released_child(CustodyRelease::ReleasedToYou, 90);
    c.birth_date = 20050501;
    c.situation = DependentSituation::Supported {
        your_support_over_half: c.your_support_over_half(),
        gross_income: 200_000,
    };
    let r = supported(&single_with_release(40_000, c.clone()));
    assert_eq!(value(&r, "child_credit"), 500);
    assert_eq!(r.tax, Some(212_300));
    assert!(noted(&r, "NOTE_FORM_8332_ATTACH"), "{r:?}");
    c.situation = DependentSituation::Supported {
        your_support_over_half: c.your_support_over_half(),
        gross_income: 530_000,
    };
    let r = supported(&single_with_release(40_000, c.clone()));
    assert_eq!(value(&r, "child_credit"), 0);
    assert_eq!(r.tax, Some(262_300));
    // A released child who is not related as a child cannot rest on the
    // release for the qualifying-child path; Unrelated needs the whole year.
    c.situation = DependentSituation::Supported {
        your_support_over_half: c.your_support_over_half(),
        gross_income: 200_000,
    };
    c.relationship = Relationship::Unrelated;
    let r = supported(&single_with_release(40_000, c));
    assert_eq!(value(&r, "child_credit"), 0);
}

// A release the filer signed ends the filer's dependent claim whatever else is
// true of the child, and the child who provides over half their own support
// is not released to anyone.
#[test]
fn a_release_never_makes_a_child_who_supports_themselves_a_dependent() {
    let mut c = released_child(CustodyRelease::ReleasedToYou, 90);
    set_fact(&mut c, PersonFacts::OWN_SUPPORT, true);
    let r = supported(&single_with_release(40_000, c));
    assert_eq!(value(&r, "child_credit"), 0);
    assert_eq!(r.tax, Some(262_300));
    // Another test still applies to the released child: a joint return other
    // than for a refund.
    let mut c = released_child(CustodyRelease::ReleasedToYou, 90);
    set_fact(&mut c, PersonFacts::FILED_JOINTLY, true);
    let r = supported(&single_with_release(40_000, c));
    assert_eq!(value(&r, "child_credit"), 0);
    // And the citizen or resident test.
    let mut c = released_child(CustodyRelease::ReleasedToYou, 90);
    set_fact(&mut c, PersonFacts::ABROAD, true);
    let r = supported(&single_with_release(40_000, c));
    assert_eq!(value(&r, "child_credit"), 0);
    assert!(!noted(&r, "NOTE_FORM_8332_ATTACH"), "{r:?}");
}

// A qualifying surviving spouse needs a dependent son or daughter (section
// 2(a)(1)(B)); the child the filer released is not the filer's dependent, and
// no listed exception is a release.
#[test]
fn qualifying_survivor_cannot_rest_on_a_child_the_filer_released() {
    let survivor = |release| {
        let mut d = complete(FederalStatus::Surviving);
        d.setup.as_mut().unwrap().marital_state =
            MaritalState::WidowedPriorYear { could_file_joint: true };
        let mut c = released_child(release, 365);
        c.situation = DependentSituation::Supported {
            your_support_over_half: true,
            gross_income: c.gross_income(),
        };
        d.dependents.push(c);
        d.next_id = 3;
        d
    };
    supported(&survivor(CustodyRelease::None));
    unsupported(&survivor(CustodyRelease::YouReleased), "Filing status");
}

// A married custodial parent who lived apart from the spouse for the last six
// months and released the child can still file head of household: section
// 7703(b)(1) considers the filer unmarried with a child the filer "would be
// so entitled but for section 152(e)". The citizen or resident test of
// section 152(b)(3) applies on that path (it is a test of the section 151
// deduction): the same child who is not a U.S. citizen, national or resident
// does not make the married filer unmarried. On a separate return the Form
// 2441 exception counts the released child as the custodial parent's
// qualifying person: wages $45,000, AGI 45,000, 35% x $2,000 = $700.
#[test]
fn married_custodial_parent_who_released_the_child_is_considered_unmarried() {
    let apart = |status: FilingChoice, c: FederalDependent| {
        let mut d = head_with_release(45_000, c);
        if matches!(status, FilingChoice::Separate { .. }) {
            answer_separate_pages(&mut d);
        }
        d.setup.as_mut().unwrap().status = status;
        d.setup.as_mut().unwrap().marital_state =
            MaritalState::Married { living: SpouseLiving::LastSixMonths };
        document_care(&mut d);
        d
    };
    let c = released_child(CustodyRelease::YouReleased, 365);
    let r = supported(&apart(FilingChoice::Head, c.clone()));
    assert_eq!(value(&r, "deduction"), 24150);
    // Not a citizen: no 152(b)(3) dependent, so the married filer cannot be
    // considered unmarried through the child.
    let mut foreign = c.clone();
    set_fact(&mut foreign, PersonFacts::ABROAD, true);
    unsupported(&apart(FilingChoice::Head, foreign), "Filing status");
    // Separate return: the Form 2441 exception.
    let mut c = c;
    c.set_care_expenses(200_000);
    let r = supported(&apart(
        FilingChoice::Separate { section179: Section179Spouse::NotSupplied },
        c.clone(),
    ));
    assert_eq!(value(&r, "care_credit"), 700);
    // The parent holding the release has no qualifying person for care.
    set_release(&mut c, CustodyRelease::ReleasedToYou);
    c.residency = DependentResidency::Days { days: 90 };
    c.us_home_over_half_year = false;
    let r =
        supported(&apart(FilingChoice::Separate { section179: Section179Spouse::NotSupplied }, c));
    assert_eq!(value(&r, "care_credit"), 0);
}

// Form 8863: only the person who claims the student as a dependent claims the
// credit for the student's expenses (section 25A(g)(3)); the child the filer
// released is not on the filer's dependents list, so an education row naming
// the child is refused, and the parent holding the release may name the child.
#[test]
fn education_credit_follows_the_parent_who_claims_the_child() {
    let with_education = |release, days| {
        let mut d = single_with_release(40_000, released_child(release, days));
        d.dependents[0].birth_date = 20080615;
        d.dependents[0].situation = DependentSituation::Student;
        let child = crate::federal_rules::people_students(&d).last().unwrap().id;
        set_education(
            d.credits.as_mut().unwrap(),
            vec![native_education(
                child,
                education_expenses(300_000),
                EducationChoice::LifetimeLearning,
            )],
        );
        d
    };
    unsupported(&with_education(CustodyRelease::YouReleased, 365), "Education");
    let r = supported(&with_education(CustodyRelease::ReleasedToYou, 90));
    // Lifetime Learning Credit: 20% of $3,000 of tuition (AGI 40,000 is
    // below the phase-out).
    assert_eq!(value(&r, "education_credit"), 600);
    // Control: no release, the child lived with the filer all year.
    supported(&with_education(CustodyRelease::None, 365));
}

// The agi of the fixture return (Arizona, halves) with a $30,000 W-2 divided
// by its community answer, and one more income item.
pub(in crate::tests) fn community_item_agi(
    entry: FederalIncomeEntry,
    owner: Owner,
    share: CommunityShare,
) -> FederalResult {
    let mut d = complete(FederalStatus::Separate);
    d.deductions.as_mut().unwrap().itemize_override = None;
    d.income = vec![wage(1, Owner::Taxpayer, 3_000_000, 300_000)];
    d.next_id = 2;
    d.setup.as_mut().unwrap().community_property = community_arizona();
    d.income[0].community = CommunityShare::Community;
    let mut e = entry;
    e.owner = crate::DocumentOwner::from(owner);
    e.community = share;
    add(&mut d, e);
    estimate(&d)
}
pub(in crate::tests) fn plain_item_agi(entry: FederalIncomeEntry) -> i64 {
    let mut d = complete(FederalStatus::Separate);
    d.deductions.as_mut().unwrap().itemize_override = None;
    d.income = vec![wage(1, Owner::Taxpayer, 3_000_000, 300_000)];
    d.next_id = 2;
    add(&mut d, entry);
    value(&supported(&d), "agi")
}

#[test]
fn fixed_by_law_items_are_the_owners_whatever_the_row_says() {
    // Pub. 555: IRA distributions and Social Security are the owner's own.
    for entry in [
        retirement_entry(Owner::Taxpayer, RetirementKind::Ira, 2000, RetirementTreatment::Ordinary),
        income(social_security_net(2000, 0)),
    ] {
        let plain = plain_item_agi(entry.clone());
        let item = plain - 30_000;
        for share in [
            CommunityShare::Community,
            CommunityShare::Separate,
            CommunityShare::Partly { percent: 40 },
        ] {
            let own = community_item_agi(entry.clone(), Owner::Taxpayer, share);
            // Half of the W-2 in every case; the account's income whole.
            assert_eq!(value(&own, "agi"), 15_000 + item, "{share:?}");
            let theirs = community_item_agi(entry.clone(), Owner::Spouse, share);
            assert_eq!(value(&theirs, "agi"), 15_000, "{share:?}");
        }
    }
}

#[test]
fn a_row_without_an_answer_on_a_community_property_return_is_a_remaining_correction() {
    let mut d = community_pair(CommunityState::Arizona, CommunityRule::Halves, true);
    d.income[2].community = CommunityShare::NotNeeded;
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(r.issues.iter().any(|i| i.display.key == "ISSUE_COMMUNITY_ANSWER_NEEDED"));
}
