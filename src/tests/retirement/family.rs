// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::*;

#[test]
fn saver_age_student_and_dependent_eligibility() {
    use crate::federal_rules::saver_potential;
    let mut p = complete(FederalStatus::Single).people.unwrap();
    let f = saver(200_000, 0);
    p.taxpayer.birth_date = 20090101;
    assert_eq!(saver_potential(&f, &p, FederalStatus::Single, 20000), Ok(1000));
    p.taxpayer.birth_date = 20090102;
    assert_eq!(saver_potential(&f, &p, FederalStatus::Single, 20000), Ok(0));
    p.taxpayer.birth_date = 19800615;
    p.taxpayer.full_time_student = true;
    assert_eq!(saver_potential(&f, &p, FederalStatus::Single, 20000), Ok(0));
    p.taxpayer.full_time_student = false;
    p.taxpayer.dependency = FilerDependency::CanBeClaimed { claimed: true };
    assert_eq!(saver_potential(&f, &p, FederalStatus::Single, 20000), Ok(0));
}

#[test]
fn saver_follows_education_care_and_precedes_child_credit() {
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 2_500_000, 0);
    let id = d.next_id;
    add(&mut d, wage(id, Owner::Spouse, 2_500_000, 0));
    add_child(&mut d);
    let without = {
        document_care(&mut d);
        supported(&d)
    };
    assert_eq!(value(&without, "child_credit"), 1783);
    assert_eq!(value(&without, "additional_child_credit"), 417);
    d.credits.as_mut().unwrap().saver = (Some(saver(200_000, 200_000))).into();
    let r = {
        document_care(&mut d);
        supported(&d)
    };
    assert_eq!(value(&r, "saver_credit"), 800);
    assert_eq!(value(&r, "child_credit"), 983);
    assert_eq!(value(&r, "additional_child_credit"), 1217);
    let sid0_1802 = -1; // You, the first person of the catalog
    d.credits.as_mut().unwrap().education.push(native_education(
        sid0_1802,
        education_expenses(500_000),
        education_choice(EducationMethod::LifetimeLearning, Answer::No),
    ));
    d.dependents[0].set_care_expenses(100_000);
    let r = {
        document_care(&mut d);
        supported(&d)
    };
    assert_eq!(value(&r, "education_credit"), 1000);
    assert_eq!(value(&r, "care_credit"), 350);
    assert_eq!(value(&r, "saver_credit"), 433);
    assert_eq!(value(&r, "child_credit"), 0);
    assert_eq!(value(&r, "additional_child_credit"), 1700);
    assert_eq!(r.tax, Some(0));
}

#[test]
fn saver_retained_spouse_answers_block_estimate() {
    let mut d = complete(FederalStatus::Single);
    d.credits.as_mut().unwrap().saver = (Some(saver(200_000, 0))).into();
    let f = d.credits.as_mut().unwrap().saver.as_mut().unwrap();
    f.spouse_contributions = 1;
    unsupported(&d, "Saver");
    let f = d.credits.as_mut().unwrap().saver.as_mut().unwrap();
    f.spouse_contributions = 0;
    f.distributions.push(FederalSaverDistribution {
        owner: Owner::Spouse,
        amount: 100,
        year: distribution_period(SaverDistributionYear::PriorOne, Answer::No),
    });
    unsupported(&d, "Saver");
}

#[test]
fn public_safety_long_term_care_limit_is_shared_across_spouses_and_names_a_person_on_the_return() {
    let mut d = complete(FederalStatus::Joint);
    set_age(&mut d, -1, 50);
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        add(
            &mut d,
            retirement_entry(owner, RetirementKind::Pension, 10000, RetirementTreatment::Ordinary),
        );
        let id = d.income.last().unwrap().id;
        public_safety(&mut d, id, 5000, 3000, PublicSafetyCoverage::LongTermCare { person: -1 });
    }
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 114000);
    assert_eq!(value(&r, "public_safety_medical_before_coordination"), 930);
    // A policy for someone who is not on the return is refused by name; the
    // age is the person's own, so there is no second age to disagree.
    let PublicSafetyCoverage::LongTermCare { person } =
        &mut d.screening.as_mut().unwrap().public_safety[1].coverage
    else {
        unreachable!()
    };
    *person = 99;
    unsupported(&d, "Long-term-care insurance");
}
#[test]
fn nondeductible_allocation_uses_independent_amt_capacity() {
    let mut source = sole_qbi_loss(18_500, 20_000, 18_500);
    source.nondeductible = 150_000;
    source.risk.start = ScorpRiskStart::Detailed { capacity: 1_000_000, profit: 0, loss: 0 };
    let mut amt = amt_records(&source);
    amt.risk.start = ScorpRiskStart::Detailed { capacity: 1_200_000, profit: 0, loss: 0 };
    source.amt = Some(amt);
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Scorp { data: source }));
    let r = supported(&d);
    amt_k1_note(&r, -11_100, -9_250);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 9_250);
    // 2026-10-19: the owning usd_cents message renders Currency2 amounts.
    assert!(r.review_notes.iter().any(|n| n.contains(
        "nondeductible expenses absorbed: $900.00; still suspended by at-risk: $600.00; ending at-risk investment: $0.00."
    )));
}

#[test]
fn care_distribution_spouse_coverage_requires_joint_return() {
    let mut d = care_return(care_claim(3000, 100000, true), 20000);
    let no_joint = supported(&d);
    assert_eq!(value(&no_joint, "care_distribution_2_exception"), 0);
    assert_eq!(value(&no_joint, "early_distribution_tax"), 1600);
    assert_eq!(no_joint.tax, Some(2297400));
    let joint = complete(FederalStatus::Joint);
    d.setup = joint.setup;
    d.people = joint.people;
    let r = supported(&d);
    assert_eq!(value(&r, "care_distribution_2_exception"), 2600);
    assert_eq!(value(&r, "early_distribution_tax"), 1340);
    assert_eq!(value(&r, "agi"), 136000);
}

#[test]
fn automatic_enrollment_saver_joint_return_offsets_both_columns() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut d = complete(FederalStatus::Joint);
        d.income[0] = wage(1, Owner::Taxpayer, 4800000, 0);
        add(&mut d, automatic_enrollment_income(owner, 2000, false));
        d.credits.as_mut().unwrap().saver = Some(saver(300000, 300000)).into();
        let r = supported(&d);
        assert_eq!(value(&r, "income_tax"), 1783);
        assert_eq!(value(&r, "saver_credit"), 400);
        assert_eq!(value(&r, "early_distribution_tax"), 0);
        assert_eq!(r.tax, Some(138300));
    }
}
