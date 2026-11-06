use crate::tests::support::assert::line_amount as cents_of;
// Form 1099-Q (src/education_account.rs): the taxable earnings of 529 plan and
// Coverdell ESA distributions, the Form 5329 Part II additional tax and the
// expenses they share with the education credit. Fixtures F1-F9 and F5/F7 of
// the Form 1099-Q map, with the arithmetic in the comments. Dollars below;
// stored amounts are cents.
use super::*;

pub(in crate::tests) fn cents(dollars: i64) -> i64 {
    dollars * 100
}

// A Form 1099-Q (dollars).
pub(in crate::tests) fn qtp(earnings: i64, basis: i64) -> EducationAccount {
    EducationAccount::Qtp { earnings: cents(earnings), basis: cents(basis) }
}
pub(in crate::tests) fn coverdell(earnings: i64, basis: i64) -> EducationAccount {
    EducationAccount::Coverdell {
        amounts: CoverdellAmounts::Reported { earnings: cents(earnings), basis: cents(basis) },
    }
}
pub(in crate::tests) fn withdrawal() -> Q1099Transfer {
    Q1099Transfer::Withdrawal { reason: DistributionReason::Withdrawal }
}
pub(in crate::tests) fn q1099_form(
    owner: Owner,
    gross: i64,
    account: EducationAccount,
    transfer: Q1099Transfer,
    beneficiary: FederalEducationBeneficiary,
) -> FederalIncomeEntry {
    let mut entry = income(FederalIncome::EducationDistribution {
        data: education_distribution_source(FederalEducationDistribution {
            recipient_not_beneficiary: !matches!(
                &beneficiary,
                FederalEducationBeneficiary::Recipient
            ),
            coverdell_basis_start: None,
            coverdell_contributions: 0,
            redeposited: 0,
            taxable_transfer_part: 0,
            roth_rollover: None,
            late_rollovers: None,
            gross: cents(gross),
            account,
            transfer,
            beneficiary: Some(beneficiary),
        }),
    });
    entry.owner = crate::DocumentOwner::from(owner);
    entry
}
// One student's row of the expense record: every amount zero but those set.
pub(in crate::tests) fn student_row(
    person: i64,
    college: CollegeCosts,
) -> FederalEducationAccountStudent {
    FederalEducationAccountStudent {
        person,
        college,
        other_college: 0,
        k12: 0,
        k12_coverdell: 0,
        apprenticeship: 0,
        loan_payments: 0,
        loan_payments_before: 0,
        academy_costs: 0,
        used_elsewhere: 0,
    }
}
pub(in crate::tests) fn entered(
    tuition_books: i64,
    assistance: i64,
    elsewhere: i64,
) -> CollegeCosts {
    CollegeCosts::Entered {
        tuition_books: cents(tuition_books),
        assistance: cents(assistance),
        credit_elsewhere: cents(elsewhere),
    }
}
pub(in crate::tests) fn expense_record(
    rows: Vec<FederalEducationAccountStudent>,
) -> FederalIncomeEntry {
    income(FederalIncome::EducationAccountExpenses {
        data: FederalEducationAccountExpenses { students: rows },
    })
}
fn recipient() -> FederalEducationBeneficiary {
    FederalEducationBeneficiary::Recipient
}
fn student(person: i64) -> FederalEducationBeneficiary {
    FederalEducationBeneficiary::Student { person }
}
// A return with the first W-2 at this wage (dollars) and one dependent child
// (the student), in the education catalog.
fn education_return(status: FederalStatus, wages: i64) -> FederalDraft {
    let mut d = complete(status);
    d.income[0] = wage(1, Owner::Taxpayer, cents(wages), 0);
    add_child(&mut d);
    d
}
// The cents of a calculation line.
fn missing(r: &FederalResult, id: &str) -> bool {
    r.lines.iter().all(|l| l.id != id)
}
fn form_ids(d: &FederalDraft) -> Vec<i64> {
    d.income
        .iter()
        .filter(|e| matches!(e.income, FederalIncome::EducationDistribution { .. }))
        .map(|e| e.id)
        .collect()
}

// F1: fully qualified (joint, parent recipient, no credit). Wages $250,000.
// Dependent D, a college student. Box 1 $25,000 (earnings $6,000, basis
// $19,000), student D. Costs: tuition and books $20,000, other college $12,000.
//   AQEE = 32,000 >= 25,000, so the earnings are tax-free: taxable 0; no
//   Part II line; AGI 250,000.
#[test]
fn f1_fully_qualified_distribution_is_tax_free() {
    let mut d = education_return(FederalStatus::Joint, 250_000);
    let child = d.dependents[0].id;
    add(
        &mut d,
        q1099_form(Owner::Taxpayer, 25_000, qtp(6_000, 19_000), withdrawal(), student(child)),
    );
    let mut row = student_row(child, entered(20_000, 0, 0));
    row.other_college = cents(12_000);
    add(&mut d, expense_record(vec![row.clone()]));
    let r = supported(&d);
    let id = form_ids(&d)[0];
    assert_eq!(cents_of(&r, &alloc::format!("education_account_{id}_taxable")), 0);
    assert_eq!(cents_of(&r, &alloc::format!("education_account_{id}_earnings")), 600_000);
    assert_eq!(cents_of(&r, "education_account_taxable_earnings"), 0);
    assert_eq!(value(&r, "account_additional_tax"), 0);
    assert!(missing(&r, "account_taxpayer_taxable"));
    assert_eq!(value(&r, "agi"), 250_000);
    assert!(!noted(&r, "WARN_EDUCATION_ACCOUNT_EXPENSES_ASSUMED_NONE"));
    // F1b: other college $5,000 makes AQEE exactly $25,000 = the distribution.
    let mut exact = row;
    exact.other_college = cents(5_000);
    d.income.pop();
    add(&mut d, expense_record(vec![exact.clone()]));
    let r = supported(&d);
    assert_eq!(cents_of(&r, "education_account_taxable_earnings"), 0);
    // $1,000 short of the distribution leaves part of the earnings taxable.
    exact.other_college = cents(4_000);
    d.income.pop();
    add(&mut d, expense_record(vec![exact]));
    let r = supported(&d);
    // AQEE = 20,000 + 4,000 = 24,000: tax-free = 600,000 x 2,400,000 /
    // 2,500,000 = 576,000; taxable 24,000 cents = $240.
    assert_eq!(cents_of(&r, "education_account_taxable_earnings"), 24_000);
    // Part II: line 5 $240, no exception (no credit, no assistance), line 8 $24.
    assert_eq!(value(&r, "account_taxpayer_taxable"), 240);
    assert_eq!(value(&r, "account_taxpayer_subject"), 240);
    assert_eq!(value(&r, "account_additional_tax"), 24);
}

// F2: partly qualified with AOTC coordination (Pub 970 ch. 7 Example 2).
// Joint, wages $120,000. Dependent S: AOTC entry with tuition $7,100 and
// scholarships $3,100, so qualified expenses 4,000 and the credit uses 4,000.
// Box 1 $5,300 (earnings $950, basis $4,350), student S; the expense row takes
// the credit entry and $1,200 of other college costs.
//   QHEE = 7,100 + 1,200 = 8,300; R = 3,100 + 4,000 = 7,100; AQEE = 1,200.
//   Tax-free = 95,000 x 120,000 / 530,000 = 21,509; taxable 73,491 = $735.
//   Part II: T_nc = 95,000 - 95,000 x 520,000 / 530,000 = 1,792 -> X5 = 71,699;
//   XD = 10,000, S = 310,000 -> X34 = 1,792; line 6 = 735, line 7 = 0, tax 0.
#[test]
fn f2_credit_coordination_reproduces_publication_970_example_2() {
    let build = |tuition: i64, scholarships: i64| {
        let mut d = education_return(FederalStatus::Joint, 120_000);
        let child = d.dependents[0].id;
        let mut expenses = education_expenses(cents(tuition));
        expenses.assistance = cents(scholarships);
        let sid2_190 = sid(&d, 2);
        add_education(d.credits.as_mut().unwrap(), {
            let mut pair = native_education(
                sid2_190,
                expenses,
                education_choice(EducationMethod::AmericanOpportunity, Answer::Yes),
            );
            pair.0.id = 0;
            pair.0.half_time = true;
            pair.0.graduate = false;
            pair.1.more.american_opportunity_used_up = Some(false);
            pair
        });
        add(
            &mut d,
            q1099_form(Owner::Taxpayer, 5_300, qtp(950, 4_350), withdrawal(), student(child)),
        );
        let mut row = student_row(child, CollegeCosts::EducationCredit);
        row.other_college = cents(1_200);
        add(&mut d, expense_record(vec![row]));
        d
    };
    let d = build(7_100, 3_100);
    let r = supported(&d);
    let id = form_ids(&d)[0];
    assert_eq!(
        cents_of(&r, &alloc::format!("education_account_{id}_adjusted_expenses")),
        cents(1_200)
    );
    assert_eq!(cents_of(&r, &alloc::format!("education_account_{id}_taxable")), 73_491);
    assert_eq!(cents_of(&r, &alloc::format!("education_account_{id}_exceptions")), 73_491);
    assert_eq!(cents_of(&r, "education_account_taxable_earnings"), 73_491);
    assert_eq!(value(&r, "account_taxpayer_taxable"), 735);
    assert_eq!(value(&r, "account_taxpayer_exceptions"), 735);
    assert_eq!(value(&r, "account_taxpayer_subject"), 0);
    assert_eq!(value(&r, "account_taxpayer_tax"), 0);
    assert_eq!(value(&r, "account_additional_tax"), 0);
    // The credit stays: AGI 120,735 is below the phaseout; 2,000 + 25% of 2,000.
    assert_eq!(value(&r, "agi"), 120_735);
    assert_eq!(value(&r, "education_credit") + value(&r, "refundable_education_credit"), 2_500);
    // F2b: no scholarship (tuition $4,000): the same taxable $735, but the whole
    // credit part is excepted and nothing is covered by a scholarship.
    //   X5 = 71,699, S = 0, so line 6 = 717, line 7 = 18, line 8 = 10% of 18 = 2.
    let d = build(4_000, 0);
    let r = supported(&d);
    let id = form_ids(&d)[0];
    assert_eq!(cents_of(&r, &alloc::format!("education_account_{id}_taxable")), 73_491);
    assert_eq!(cents_of(&r, &alloc::format!("education_account_{id}_exceptions")), 71_699);
    assert_eq!(value(&r, "account_taxpayer_taxable"), 735);
    assert_eq!(value(&r, "account_taxpayer_exceptions"), 717);
    assert_eq!(value(&r, "account_taxpayer_subject"), 18);
    assert_eq!(value(&r, "account_taxpayer_tax"), 2);
    assert_eq!(value(&r, "account_additional_tax"), 2);
    assert!(r.lines.iter().any(|l| l.id == "account_additional_tax" && l.amount == 200));
}

// F3: nonqualified, scholarship exception (single, the student is the
// recipient). Wages $30,000. Box 1 $10,000 (earnings $4,000, basis $6,000);
// costs: tuition $6,000 with $6,000 of tax-free assistance.
//   AQEE = 0 -> taxable 4,000 (Schedule 1 line 8z; AGI 34,000).
//   X5 = 0; XD = 10,000; S = 6,000 -> X34 = 400,000 x 600,000 / 1,000,000 =
//   240,000; line 5 = 4,000, line 6 = 2,400, line 7 = 1,600, line 8 = 160.
#[test]
fn f3_scholarship_covered_part_is_excepted_from_the_additional_tax() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, cents(30_000), 0);
    add(&mut d, q1099_form(Owner::Taxpayer, 10_000, qtp(4_000, 6_000), withdrawal(), recipient()));
    add(&mut d, expense_record(vec![student_row(-1, entered(6_000, 6_000, 0))]));
    let r = supported(&d);
    assert_eq!(cents_of(&r, "education_account_taxable_earnings"), cents(4_000));
    assert_eq!(value(&r, "agi"), 34_000);
    assert_eq!(value(&r, "account_taxpayer_taxable"), 4_000);
    assert_eq!(value(&r, "account_taxpayer_exceptions"), 2_400);
    assert_eq!(value(&r, "account_taxpayer_subject"), 1_600);
    assert_eq!(value(&r, "account_taxpayer_tax"), 160);
    assert_eq!(value(&r, "account_additional_tax"), 160);
    // The additional tax is part of the return's tax.
    let without = {
        let mut other = d.clone();
        other.income.truncate(1);
        supported(&other)
    };
    assert!(r.tax > without.tax);
    // F3b: a disability (box 7 code 4) excepts all of it.
    let FederalIncome::EducationDistribution { data } = &mut d.income[1].income else {
        unreachable!()
    };
    data.form.trustee_to_trustee = false;
    data.form.qtp_to_roth_ira = false;
    data.form.distribution_code = Some(crate::Form1099QDistributionCode::Disability);
    data.redeposited = 0;
    let r = supported(&d);
    assert_eq!(value(&r, "account_taxpayer_exceptions"), 4_000);
    assert_eq!(value(&r, "account_additional_tax"), 0);
    assert_eq!(cents_of(&r, "education_account_taxable_earnings"), cents(4_000));
}

// F4: the 529 K-12 limit (joint, state 529). Dependent K. Box 1 $25,000
// (earnings $5,000, basis $20,000), student K; K-12 costs $25,000.
//   AQEE = min(25,000, 20,000 cap) = 20,000; tax-free = 500,000 x 2,000,000 /
//   2,500,000 = 400,000; taxable 1,000; line 5 = 1,000, line 6 = 0, line 8 = 100.
#[test]
fn f4_k12_costs_are_capped_for_a_529_only() {
    let mut d = education_return(FederalStatus::Joint, 150_000);
    let child = d.dependents[0].id;
    let mut row = student_row(child, entered(0, 0, 0));
    row.k12 = cents(25_000);
    add(&mut d, expense_record(vec![row]));
    add(
        &mut d,
        q1099_form(Owner::Taxpayer, 25_000, qtp(5_000, 20_000), withdrawal(), student(child)),
    );
    let r = supported(&d);
    assert_eq!(cents_of(&r, "education_account_taxable_earnings"), cents(1_000));
    assert_eq!(value(&r, "account_taxpayer_taxable"), 1_000);
    assert_eq!(value(&r, "account_taxpayer_exceptions"), 0);
    assert_eq!(value(&r, "account_additional_tax"), 100);
    // F4b: the same form as a Coverdell has no K-12 limit: taxable 0.
    let FederalIncome::EducationDistribution { data } = &mut d.income[2].income else {
        unreachable!()
    };
    data.form.distribution_from = crate::Form1099QDistributionFrom::CoverdellEsa;
    let r = supported(&d);
    assert_eq!(cents_of(&r, "education_account_taxable_earnings"), 0);
    assert_eq!(value(&r, "account_additional_tax"), 0);
    // F4c: two 529 forms of $12,500 (earnings $2,500 each): the cap is per
    // student, not per form. AQEE 20,000 splits 10,000 / 10,000, so each form
    // has 2,500 - 2,500 x 1,000,000 / 1,250,000 = 500 taxable.
    let FederalIncome::EducationDistribution { data } = &mut d.income[2].income else {
        unreachable!()
    };
    *data = education_distribution_source(FederalEducationDistribution {
        recipient_not_beneficiary: !matches!(
            &student(child),
            FederalEducationBeneficiary::Recipient
        ),
        coverdell_basis_start: None,
        coverdell_contributions: 0,
        redeposited: 0,
        taxable_transfer_part: 0,
        roth_rollover: None,
        late_rollovers: None,
        gross: cents(12_500),
        account: qtp(2_500, 10_000),
        transfer: withdrawal(),
        beneficiary: Some(student(child)),
    });
    add(
        &mut d,
        q1099_form(Owner::Taxpayer, 12_500, qtp(2_500, 10_000), withdrawal(), student(child)),
    );
    let r = supported(&d);
    for id in form_ids(&d) {
        assert_eq!(
            cents_of(&r, &alloc::format!("education_account_{id}_adjusted_expenses")),
            cents(10_000)
        );
        assert_eq!(cents_of(&r, &alloc::format!("education_account_{id}_taxable")), cents(500));
    }
    assert_eq!(cents_of(&r, "education_account_taxable_earnings"), cents(1_000));
    assert_eq!(value(&r, "account_additional_tax"), 100);
}

// F5: a Coverdell with only a box 7 value (Pub 970 ch. 6, credit example).
// Single student filer; the parents claimed the American opportunity credit on
// $4,000. Box 1 $1,000, box 7 value $1,800, basis at the end of 2025 $2,100
// and 2026 contributions $400; costs: tuition and books $5,800, assistance
// $1,500, expenses another person's credit used $4,000. AQEE = 300.
//   Worksheet 6-3: line 3 = 2,500; 8 = 2,800; 10 = 100,000 x 250,000 /
//   280,000 = 89,286; 11 = 10,714; 13 = 10,714 x 30,000 / 100,000 = 3,214;
//   14 = 7,500 = $75; 15 = 250,000 - 89,286 = 160,714.
//   Part II: with the parents' credit left out AQEE = 4,300 >= 1,000, so
//   T_nc = 0 and X5 = $75: line 7 = 0.
#[test]
fn f5_coverdell_with_only_a_year_end_value_uses_worksheet_6_3() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, cents(20_000), 0);
    add(&mut d, {
        let mut value = q1099_form(
            Owner::Taxpayer,
            1_000,
            EducationAccount::Coverdell {
                amounts: CoverdellAmounts::ValueOnly { value: cents(1_800) },
            },
            withdrawal(),
            recipient(),
        );
        let FederalIncome::EducationDistribution { data } = &mut value.income else {
            unreachable!()
        };
        data.coverdell_basis_start = Some(cents(2_100));
        data.coverdell_contributions = cents(400);
        data.late_rollovers = Some(0);
        value
    });
    add(&mut d, expense_record(vec![student_row(-1, entered(5_800, 1_500, 4_000))]));
    let r = supported(&d);
    let id = form_ids(&d)[0];
    assert_eq!(cents_of(&r, &alloc::format!("education_account_{id}_earnings")), 10_714);
    assert_eq!(cents_of(&r, &alloc::format!("education_account_{id}_taxable")), 7_500);
    assert_eq!(cents_of(&r, &alloc::format!("education_account_{id}_basis_remaining")), 160_714);
    assert_eq!(
        cents_of(&r, &alloc::format!("education_account_{id}_adjusted_expenses")),
        cents(300)
    );
    assert_eq!(value(&r, "account_taxpayer_taxable"), 75);
    assert_eq!(value(&r, "account_taxpayer_exceptions"), 75);
    assert_eq!(value(&r, "account_taxpayer_subject"), 0);
    assert_eq!(value(&r, "account_additional_tax"), 0);
}

#[test]
fn coverdell_paper_fmv_and_known_late_rollovers_supply_the_original_worksheet_input() {
    let mut d = complete(FederalStatus::Single);
    let mut entry = q1099_form(
        Owner::Taxpayer,
        1_000,
        EducationAccount::Coverdell {
            amounts: CoverdellAmounts::ValueOnly { value: cents(1_800) },
        },
        withdrawal(),
        recipient(),
    );
    let FederalIncome::EducationDistribution { data } = &mut entry.income else { unreachable!() };
    data.coverdell_basis_start = Some(cents(2_100));
    data.coverdell_contributions = cents(400);
    data.late_rollovers = Some(0);
    add(&mut d, entry);
    let expected = supported(&d);
    let id = form_ids(&d)[0];
    let FederalIncome::EducationDistribution { data } =
        &mut d.income.iter_mut().find(|entry| entry.id == id).unwrap().income
    else {
        unreachable!()
    };
    data.form.distribution_from = crate::Form1099QDistributionFrom::CoverdellEsa;
    data.form.earnings = 0;
    data.form.basis = 0;
    data.form.fair_market_value = Some(crate::Form1099QFairMarketValue { value: cents(1_700) });
    data.late_rollovers = Some(cents(100));
    let actual = supported(&d);
    for suffix in ["earnings", "taxable", "basis_remaining"] {
        let line = alloc::format!("education_account_{id}_{suffix}");
        assert_eq!(cents_of(&actual, &line), cents_of(&expected, &line));
    }
    let restored = FederalDraft::from_avro(&d.to_avro()).unwrap();
    assert_eq!(restored, d);
    let FederalIncome::EducationDistribution { data } =
        &mut d.income.iter_mut().find(|entry| entry.id == id).unwrap().income
    else {
        unreachable!()
    };
    data.late_rollovers = None;
    assert!(FederalEducationDistribution::from(&*data).valid());
    assert_eq!(
        FederalEducationDistribution::from(&*data).answer_issue().unwrap().key,
        "Q1099_LATE_ROLLOVERS_NEEDED"
    );
    assert!(
        estimate(&d).issues.iter().any(|issue| issue.display.key == "Q1099_LATE_ROLLOVERS_NEEDED")
    );
}

#[test]
fn paper_box6_and_retained_student_answers_keep_unknowns_and_conflicts_distinct() {
    let entry = q1099_form(Owner::Taxpayer, 1_000, qtp(200, 800), withdrawal(), recipient());
    let FederalIncome::EducationDistribution { mut data } = entry.income else { unreachable!() };
    data.beneficiary = None;
    assert_eq!(
        FederalEducationDistribution::from(&data).effective_beneficiary(),
        Some(&FederalEducationBeneficiary::Recipient)
    );
    assert!(data.beneficiary.is_none());
    assert!(FederalEducationDistribution::from(&data).answer_issue().is_none());
    data.form.recipient_not_beneficiary = true;
    assert!(FederalEducationDistribution::from(&data).valid());
    assert!(FederalEducationDistribution::from(&data).effective_beneficiary().is_none());
    assert_eq!(
        FederalEducationDistribution::from(&data).answer_issue().unwrap().key,
        "Q1099_BENEFICIARY_NEEDED"
    );
    data.beneficiary = Some(FederalEducationBeneficiary::OtherStudent {
        expenses: 0,
        assistance: 0,
        credit_expenses: 0,
    });
    assert!(FederalEducationDistribution::from(&data).answer_issue().is_none());
    data.form.recipient_not_beneficiary = false;
    assert!(FederalEducationDistribution::from(&data).valid());
    assert_eq!(
        FederalEducationDistribution::from(&data).answer_issue().unwrap().key,
        "Q1099_BENEFICIARY_BOX6_CONFLICT"
    );
    assert!(matches!(data.beneficiary, Some(FederalEducationBeneficiary::OtherStudent { .. })));
}

#[test]
fn paper_box6_sets_the_matched_recipient_and_corrections_reopen_only_the_unknown_student() {
    use crate::tests::support::assert::accepted;
    use crate::tests::support::builders::native_income::owned;
    use crate::tests::support::client::income_pages::{
        open_document_center, open_topic, save_documents,
    };

    let entry = q1099_form(Owner::Spouse, 1_000, qtp(200, 800), withdrawal(), recipient());
    let FederalIncome::EducationDistribution { data } = entry.income else { unreachable!() };
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    let request = save_documents(
        &mut session,
        vec![owned(
            Owner::Spouse,
            0,
            IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Q1099(data.form) },
        )],
    );
    accepted(&send(&mut session, request));
    let entry = session
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::EducationDistribution { .. }))
        .unwrap();
    let id = entry.id;
    assert_eq!(entry.owner, crate::DocumentOwner::Spouses);
    let FederalIncome::EducationDistribution { data } = &entry.income else { unreachable!() };
    assert_eq!(data.beneficiary, Some(FederalEducationBeneficiary::Recipient));
    assert_eq!(entry.printed_names().unwrap().assigned, crate::DocumentRecipient::Spouse);
    let P::SaveEducation { followups, .. } = open_topic(&mut session, TaxSection::Education) else {
        unreachable!()
    };
    assert!(!followups.iter().any(|question| {
        matches!(
            question.answer,
            crate::document_center::EducationDocumentAnswer::EducationBeneficiary { .. }
        )
    }));

    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let row = documents.other.other_income.documents.iter_mut().find(|row| row.id == id).unwrap();
    let crate::OtherIncomeDocument::Q1099(form) = &mut row.document else { unreachable!() };
    form.recipient_not_beneficiary = true;
    accepted(&send(&mut session, request));
    let entry = session.draft.income.iter().find(|entry| entry.id == id).unwrap();
    let FederalIncome::EducationDistribution { data } = &entry.income else { unreachable!() };
    assert!(data.beneficiary.is_none());
    assert_eq!(
        FederalEducationDistribution::from(data).answer_issue().unwrap().key,
        "Q1099_BENEFICIARY_NEEDED"
    );
    let mut request = open_topic(&mut session, TaxSection::Education);
    let P::SaveEducation { followups, .. } = &mut request else { unreachable!() };
    let question = followups
        .iter_mut()
        .find(|question| {
            matches!(
                question.answer,
                crate::document_center::EducationDocumentAnswer::EducationBeneficiary { .. }
            )
        })
        .unwrap();
    assert_eq!(question.key, alloc::format!("income/{id}/beneficiary"));
    question.answer = crate::document_center::EducationDocumentAnswer::EducationBeneficiary {
        value: Some(Q1099Beneficiary::Student { person: -1 }),
    };
    accepted(&send(&mut session, request));
    let request = open_document_center(&mut session);
    accepted(&send(&mut session, request));
    let entry = session.draft.income.iter().find(|entry| entry.id == id).unwrap();
    let FederalIncome::EducationDistribution { data } = &entry.income else { unreachable!() };
    assert_eq!(data.beneficiary, Some(FederalEducationBeneficiary::Student { person: -1 }));

    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let row = documents.other.other_income.documents.iter_mut().find(|row| row.id == id).unwrap();
    let crate::OtherIncomeDocument::Q1099(form) = &mut row.document else { unreachable!() };
    form.recipient_not_beneficiary = false;
    accepted(&send(&mut session, request));
    let mut restored = restart(session.draft);
    let entry = restored.draft.income.iter().find(|entry| entry.id == id).unwrap();
    assert_eq!(entry.owner, crate::DocumentOwner::Spouses);
    let FederalIncome::EducationDistribution { data } = &entry.income else { unreachable!() };
    assert_eq!(data.beneficiary, Some(FederalEducationBeneficiary::Recipient));
    assert!(FederalEducationDistribution::from(data).answer_issue().is_none());
    let P::SaveEducation { followups, .. } = open_topic(&mut restored, TaxSection::Education)
    else {
        unreachable!()
    };
    assert!(!followups.iter().any(|question| {
        matches!(
            question.answer,
            crate::document_center::EducationDocumentAnswer::EducationBeneficiary { .. }
        )
    }));
}

// F6: a 529 to a Roth IRA (box 4b). The existing rollover facts are now the
// 1099-Q's: box 1 $6,000 (earnings $2,000, basis $4,000). No income, no
// additional tax; the IRA rules read boxes 2 and 3.
#[test]
fn f6_roth_transfer_is_a_form_1099_q_with_no_taxable_earnings() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, cents(30_000), 0);
    d.adjustments.as_mut().unwrap().iras = ira_facts(0, 100_000, Answer::No);
    let ordinary = supported(&d);
    set_qtp_roth(&mut d, Owner::Taxpayer, &[qtp_roth_rollover()]);
    let r = supported(&d);
    assert_eq!(value(&r, "taxpayer_529_roth_form_5498_box10"), 6_000);
    assert_eq!(value(&r, "taxpayer_529_roth_source_contribution_basis"), 4_000);
    assert_eq!(value(&r, "agi"), value(&ordinary, "agi"));
    assert_eq!(r.tax, ordinary.tax);
    assert_eq!(cents_of(&r, "education_account_taxable_earnings"), 0);
    assert_eq!(value(&r, "account_additional_tax"), 0);
    // Left unconfirmed, the transfer is not a qualified rollover: the form is an ordinary 529
    // withdrawal, with its note, once, on the 1099-Q itself.
    let mut transfer = qtp_roth_rollover();
    transfer.rollover.confirmed = false;
    set_qtp_roth(&mut d, Owner::Taxpayer, &[transfer]);
    let r = supported(&d);
    let notes: Vec<_> =
        r.review_notes.iter().filter(|n| n.display.key == "NOTE_QTP_ROTH_NOT_QUALIFIED").collect();
    assert_eq!(notes.len(), 1);
    let id = form_ids(&d)[0];
    assert!(matches!(
        notes[0].context.as_ref().unwrap().source(),
        IssueContext::Income { id: noted, .. } if *noted == id
    ));
    assert!(missing(&r, "taxpayer_529_roth_form_5498_box10"));
    let FederalIncome::EducationDistribution { data } =
        &mut d.income.iter_mut().find(|entry| entry.id == id).unwrap().income
    else {
        unreachable!()
    };
    data.roth_rollover = None;
    assert!(FederalEducationDistribution::from(&*data).valid());
    assert_eq!(
        FederalEducationDistribution::from(&*data).answer_issue().unwrap().key,
        "Q1099_ROTH_RECORDS_NEEDED"
    );
    let pending = estimate(&d);
    assert!(pending.issues.iter().any(|issue| issue.display.key == "Q1099_ROTH_RECORDS_NEEDED"));
    assert!(
        !pending.review_notes.iter().any(|note| note.display.key == "NOTE_QTP_ROTH_NOT_QUALIFIED")
    );
}

// F7: a 529 and a Coverdell for one student (Pub 970 ch. 6 Example 2). The
// student files. Coverdell box 1 $1,800 (earnings $300, basis $1,500); 529 box 1
// $3,200 (earnings $800, basis $2,400). Costs: tuition and books $3,000 and
// $1,000 of Coverdell-only K-12 costs.
//   The Coverdell takes its $1,000; the remaining 800 / 3,200 share the 3,000:
//   Coverdell 600, 529 2,400 (Pub 970: $600 / $2,400).
//   Coverdell taxable = 30,000 - 30,000 x 160,000 / 180,000 = 3,333;
//   529 taxable = 80,000 - 60,000 = 20,000; line 8z 23,333 -> $233, line 8 = 23.
#[test]
fn f7_529_and_coverdell_share_a_students_costs() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, cents(20_000), 0);
    add(
        &mut d,
        q1099_form(Owner::Taxpayer, 1_800, coverdell(300, 1_500), withdrawal(), recipient()),
    );
    add(&mut d, q1099_form(Owner::Taxpayer, 3_200, qtp(800, 2_400), withdrawal(), recipient()));
    let mut row = student_row(-1, entered(3_000, 0, 0));
    row.k12_coverdell = cents(1_000);
    add(&mut d, expense_record(vec![row]));
    let r = supported(&d);
    let ids = form_ids(&d);
    assert_eq!(
        cents_of(&r, &alloc::format!("education_account_{}_adjusted_expenses", ids[0])),
        cents(1_600)
    );
    assert_eq!(
        cents_of(&r, &alloc::format!("education_account_{}_adjusted_expenses", ids[1])),
        cents(2_400)
    );
    assert_eq!(cents_of(&r, &alloc::format!("education_account_{}_taxable", ids[0])), 3_333);
    assert_eq!(cents_of(&r, &alloc::format!("education_account_{}_taxable", ids[1])), 20_000);
    assert_eq!(cents_of(&r, "education_account_taxable_earnings"), 23_333);
    assert_eq!(value(&r, "account_taxpayer_taxable"), 233);
    assert_eq!(value(&r, "account_taxpayer_subject"), 233);
    assert_eq!(value(&r, "account_additional_tax"), 23);
    assert_eq!(value(&r, "agi"), 20_233);
}

// F9: loan payments, the $10,000 lifetime limit and the reductions of the
// educator expenses and the student loan interest. A single student filer who
// teaches: wages $40,000, Form 1098-E interest $2,500, educator expenses $300.
// Box 1 $10,000 (earnings $2,000, basis $8,000); costs: $10,000 of payments on
// the student's own loans.
//   AQEE = 10,000 -> tax-free 2,000, all paid to loans: the student loan
//   interest becomes 2,500 - 2,000 = 500 and the educator expenses 300 - 2,000
//   -> 0.
#[test]
fn f9_loan_payments_reduce_the_educator_and_student_loan_deductions() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, cents(40_000), 0);
    d.adjustments.as_mut().unwrap().student_loan.student_loan_interest = cents(2_500);
    d.adjustments.as_mut().unwrap().set_educator_taxpayer(cents(300));
    add(&mut d, q1099_form(Owner::Taxpayer, 10_000, qtp(2_000, 8_000), withdrawal(), recipient()));
    let mut row = student_row(-1, entered(0, 0, 0));
    row.loan_payments = cents(10_000);
    add(&mut d, expense_record(vec![row.clone()]));
    let before = {
        let mut other = d.clone();
        other.income.truncate(1);
        supported(&other)
    };
    assert_eq!(value(&before, "schedule_1_student_loan_interest"), 2_500);
    assert_eq!(value(&before, "schedule_1_educator_expenses"), 300);
    let r = supported(&d);
    assert_eq!(cents_of(&r, "education_account_taxable_earnings"), 0);
    assert_eq!(cents_of(&r, "education_account_loan_interest_removed"), cents(2_000));
    assert_eq!(value(&r, "schedule_1_student_loan_interest"), 500);
    assert!(missing(&r, "schedule_1_educator_expenses"));
    assert_eq!(value(&r, "agi"), 39_500);
    // F9b: $6,000 of the limit was used before 2026: counted min(10,000,
    // 4,000) = 4,000, so tax-free = 200,000 x 400,000 / 1,000,000 = 80,000 and
    // taxable $1,200 (additional tax $120); loans share 800 of the exclusion.
    row.loan_payments_before = cents(6_000);
    d.income.pop();
    add(&mut d, expense_record(vec![row]));
    let r = supported(&d);
    assert_eq!(cents_of(&r, "education_account_taxable_earnings"), cents(1_200));
    assert_eq!(cents_of(&r, "education_account_loan_interest_removed"), cents(800));
    assert_eq!(value(&r, "schedule_1_student_loan_interest"), 1_700);
    assert!(missing(&r, "schedule_1_educator_expenses"));
    assert_eq!(value(&r, "account_additional_tax"), 120);
    assert_eq!(value(&r, "agi"), 40_000 + 1_200 - 1_700);
}

// An educator is reduced only by the excluded earnings: $350 of expenses less
// $100 of tax-free earnings leaves $250; the itemized remainder follows.
#[test]
fn excluded_earnings_reduce_the_educator_expenses_before_the_cap() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, cents(40_000), 0);
    d.adjustments.as_mut().unwrap().set_educator_taxpayer(cents(350));
    add(&mut d, q1099_form(Owner::Taxpayer, 1_000, qtp(100, 900), withdrawal(), recipient()));
    add(&mut d, expense_record(vec![student_row(-1, entered(1_000, 0, 0))]));
    let r = supported(&d);
    assert_eq!(value(&r, "schedule_1_educator_expenses"), 250);
}

// A distribution whose student has no expense row assumes no expenses (all of
// the earnings taxable) with one warning.
#[test]
fn a_student_without_an_expense_row_is_assumed_to_have_no_expenses() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, cents(30_000), 0);
    add(&mut d, q1099_form(Owner::Taxpayer, 10_000, qtp(4_000, 6_000), withdrawal(), recipient()));
    let r = supported(&d);
    assert_eq!(cents_of(&r, "education_account_taxable_earnings"), cents(4_000));
    assert_eq!(value(&r, "account_additional_tax"), 400);
    assert_eq!(
        r.review_notes
            .iter()
            .filter(|n| n.display.key == "WARN_EDUCATION_ACCOUNT_EXPENSES_ASSUMED_NONE")
            .count(),
        1
    );
}

// Direct transfers, redeposits, returned excess and the refused code: the part
// put back within 60 days is not a distribution and keeps its pro rata
// earnings; a transfer's non-qualifying part is all taxable and penalized.
#[test]
fn transfers_redeposits_and_codes_follow_the_form() {
    let base = || {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, cents(30_000), 0);
        d
    };
    // Redeposited $4,000 of $10,000 (earnings $5,000): distribution 6,000,
    // earnings 3,000, no expenses: taxable 3,000.
    let mut d = base();
    add(&mut d, {
        let mut value = q1099_form(
            Owner::Taxpayer,
            10_000,
            qtp(5_000, 5_000),
            Q1099Transfer::Withdrawal { reason: DistributionReason::Withdrawal },
            recipient(),
        );
        let FederalIncome::EducationDistribution { data } = &mut value.income else {
            unreachable!()
        };
        data.redeposited = cents(4_000);
        value
    });
    let r = supported(&d);
    assert_eq!(cents_of(&r, "education_account_taxable_earnings"), cents(3_000));
    // A trustee transfer with a $2,000 non-qualifying part: its earnings
    // (5,000 x 2,000 / 10,000 = 1,000) are taxable and fully penalized.
    let mut d = base();
    add(&mut d, {
        let mut value = q1099_form(
            Owner::Taxpayer,
            10_000,
            qtp(5_000, 5_000),
            Q1099Transfer::TrusteeTransfer,
            recipient(),
        );
        let FederalIncome::EducationDistribution { data } = &mut value.income else {
            unreachable!()
        };
        data.taxable_transfer_part = cents(2_000);
        value
    });
    let r = supported(&d);
    assert_eq!(cents_of(&r, "education_account_taxable_earnings"), cents(1_000));
    assert_eq!(value(&r, "account_taxpayer_exceptions"), 0);
    assert_eq!(value(&r, "account_additional_tax"), 100);
    // A fully qualifying transfer has no income.
    let FederalIncome::EducationDistribution { data } = &mut d.income[1].income else {
        unreachable!()
    };
    data.form.trustee_to_trustee = true;
    data.form.qtp_to_roth_ira = false;
    data.taxable_transfer_part = 0;
    let r = supported(&d);
    assert_eq!(cents_of(&r, "education_account_taxable_earnings"), 0);
    // A Coverdell excess returned in time (code 2): the earnings are taxed
    // this year and never penalized.
    let mut d = base();
    add(
        &mut d,
        q1099_form(
            Owner::Taxpayer,
            2_000,
            coverdell(50, 1_950),
            Q1099Transfer::ExcessThisYear,
            recipient(),
        ),
    );
    let r = supported(&d);
    assert_eq!(cents_of(&r, "education_account_taxable_earnings"), cents(50));
    assert_eq!(value(&r, "account_taxpayer_exceptions"), 50);
    assert_eq!(value(&r, "account_additional_tax"), 0);
    // Code 3 belongs to the earlier return: nothing here, one note.
    let FederalIncome::EducationDistribution { data } = &mut d.income[1].income else {
        unreachable!()
    };
    data.form.distribution_code = Some(crate::Form1099QDistributionCode::ExcessLastYear);
    let r = supported(&d);
    assert_eq!(cents_of(&r, "education_account_taxable_earnings"), 0);
    assert!(noted(&r, "NOTE_COVERDELL_EXCESS_PRIOR_RETURN"));
    // Code 6 (a prohibited transaction): the account ceased to be a Coverdell
    // ESA on January 1 and its value that day is a withdrawal no education
    // expenses cover (IRC 530(e), 408(e)(2)): the $50 of earnings is taxable
    // and the 10% additional tax is $5, with no exception.
    let FederalIncome::EducationDistribution { data } = &mut d.income[1].income else {
        unreachable!()
    };
    data.form.distribution_code = Some(crate::Form1099QDistributionCode::ProhibitedTransaction);
    let r = supported(&d);
    assert_eq!(cents_of(&r, "education_account_taxable_earnings"), cents(50));
    assert_eq!(value(&r, "account_taxpayer_exceptions"), 0);
    assert_eq!(value(&r, "account_additional_tax"), 5);
    // A loss (a final-year 529) is never taxable.
    let mut d = base();
    add(&mut d, q1099_form(Owner::Taxpayer, 1_000, qtp(-200, 1_200), withdrawal(), recipient()));
    let r = supported(&d);
    assert_eq!(cents_of(&r, "education_account_taxable_earnings"), 0);
    assert_eq!(value(&r, "agi"), 30_000);
}

// The student must be on the return and not the form's own recipient; the
// college costs come from the credit entry only if there is one, and from the
// record only if there is none.
#[test]
fn student_and_college_cost_consistency_issues_block_the_estimate() {
    let mut d = education_return(FederalStatus::Single, 60_000);
    let child = d.dependents[0].id;
    add(&mut d, q1099_form(Owner::Taxpayer, 5_000, qtp(1_000, 4_000), withdrawal(), student(-1)));
    add(&mut d, expense_record(vec![student_row(-1, entered(5_000, 0, 0))]));
    let r = estimate(&d);
    assert!(r.issues.iter().any(|i| i.display.key == "ISSUE_Q1099_STUDENT_IS_THE_RECIPIENT"));
    // A student who is not on the return.
    d.income[1].income = FederalIncome::EducationDistribution {
        data: education_distribution_source(FederalEducationDistribution {
            recipient_not_beneficiary: !matches!(
                &student(child + 100),
                FederalEducationBeneficiary::Recipient
            ),
            coverdell_basis_start: None,
            coverdell_contributions: 0,
            redeposited: 0,
            taxable_transfer_part: 0,
            roth_rollover: None,
            late_rollovers: None,
            gross: cents(5_000),
            account: qtp(1_000, 4_000),
            transfer: withdrawal(),
            beneficiary: Some(student(child + 100)),
        }),
    };
    let r = estimate(&d);
    assert!(r.issues.iter().any(|i| {
        i.display.key
            == "ISSUE_THE_SELECTED_STUDENT_MUST_BE_A_FILER_OR_AN_ELIGIBLE_DEPENDENT_ON_TH_879AF16D"
    }));
    // The credit entry's costs without a credit entry.
    d.income[1].income = FederalIncome::EducationDistribution {
        data: education_distribution_source(FederalEducationDistribution {
            recipient_not_beneficiary: !matches!(
                &student(child),
                FederalEducationBeneficiary::Recipient
            ),
            coverdell_basis_start: None,
            coverdell_contributions: 0,
            redeposited: 0,
            taxable_transfer_part: 0,
            roth_rollover: None,
            late_rollovers: None,
            gross: cents(5_000),
            account: qtp(1_000, 4_000),
            transfer: withdrawal(),
            beneficiary: Some(student(child)),
        }),
    };
    d.income[2].income = FederalIncome::EducationAccountExpenses {
        data: FederalEducationAccountExpenses {
            students: vec![student_row(child, CollegeCosts::EducationCredit)],
        },
    };
    let r = estimate(&d);
    assert!(r.issues.iter().any(|i| {
        i.display.key == "ISSUE_EDUCATION_ACCOUNT_STUDENT_HAS_NO_EDUCATION_CREDIT_ENTRY"
    }));
    // Entered costs beside a credit entry for the same student.
    let sid1_880 = sid(&d, 1);
    add_education(d.credits.as_mut().unwrap(), {
        let mut pair = native_education(
            sid1_880,
            education_expenses(cents(5_000)),
            education_choice(EducationMethod::AmericanOpportunity, Answer::Yes),
        );
        pair.0.id = 0;
        pair.0.half_time = true;
        pair.0.graduate = false;
        pair.1.more.american_opportunity_used_up = Some(false);
        pair
    });
    d.income[2].income = FederalIncome::EducationAccountExpenses {
        data: FederalEducationAccountExpenses {
            students: vec![student_row(child, entered(5_000, 0, 0))],
        },
    };
    let r = estimate(&d);
    assert!(
        r.issues.iter().any(|i| i.display.key == "ISSUE_EDUCATION_ACCOUNT_EXPENSES_ENTERED_TWICE")
    );
}

// A student the return does not carry has the costs assigned on the form:
// the form's own expenses, assistance and credit expenses.
#[test]
fn a_student_not_on_the_return_takes_the_costs_assigned_to_the_form() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, cents(60_000), 0);
    add(
        &mut d,
        q1099_form(
            Owner::Taxpayer,
            10_000,
            qtp(2_000, 8_000),
            withdrawal(),
            FederalEducationBeneficiary::OtherStudent {
                expenses: cents(9_000),
                assistance: cents(1_000),
                credit_expenses: cents(2_000),
            },
        ),
    );
    // AQEE = 9,000 - 1,000 - 2,000 = 6,000: tax-free 2,000 x 6,000 / 10,000 =
    // 1,200, taxable 800. T_nc: AQEE 8,000 -> taxable 400, so X5 = 400; XD =
    // 10,000 - 8,000 = 2,000 and S = 1,000 -> X34 = 400 x 1,000 / 2,000 = 200.
    let r = supported(&d);
    assert_eq!(cents_of(&r, "education_account_taxable_earnings"), cents(800));
    assert_eq!(value(&r, "account_taxpayer_exceptions"), 600);
    assert_eq!(value(&r, "account_taxpayer_subject"), 200);
    assert_eq!(value(&r, "account_additional_tax"), 20);
}

// Form 8815 line 3 counts the expenses the 529 exclusion used (this unit),
// never both that and the amount the credit entry allocates to a 529.
//   Single; wages $60,000; 1099-INT box 3 $3,000; bonds: proceeds $12,000,
//   interest $3,000, owner born 1980-05-01, issued 2010-06-01. The child has an
//   AOTC entry: tuition $14,000, so the credit uses 4,000 and line 2 = 10,000.
//   The child's 529 pays $6,000 (earnings $1,500) of costs not used by the
//   credit: expenses used = 6,000 -> line 3 = 6,000, line 4 = 4,000, line 7 =
//   4,000 / 12,000 and line 8 = 1,000. Without the 1099-Q the same entry's
//   tax-free 529 amount (a person-entered $2,500 here) would be line 3.
#[test]
fn form_8815_counts_the_expenses_the_529_exclusion_used_once() {
    use crate::tests::investment::investment::{bond_record, bond_return, bond_student};
    let mut d = bond_return(FederalStatus::Single, 60_000, 3_000);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19800501;
    let child = d.dependents[0].id;
    let mut expenses = education_expenses(cents(14_000));
    expenses.other_benefits = cents(2_500);
    let sid1_952 = sid(&d, 1);
    add_education(d.credits.as_mut().unwrap(), {
        let mut pair = native_education(
            sid1_952,
            expenses,
            education_choice(EducationMethod::AmericanOpportunity, Answer::Yes),
        );
        pair.0.id = 0;
        pair.0.half_time = true;
        pair.0.graduate = false;
        pair.1.more.american_opportunity_used_up = Some(false);
        pair
    });
    let mut record =
        bond_record(Owner::Taxpayer, 12_000, 3_000, 20100601, vec![bond_student(child, 0, 0)]);
    let FederalIncome::SavingsBonds { data } = &mut record.income else { unreachable!() };
    data.answers.as_mut().unwrap().students[0].expenses =
        FederalSavingsBondExpenses::EducationCredit;
    add(&mut d, record);
    // Without a 1099-Q: the entry's own allocation (2,500) is line 3.
    // Qualified = 14,000 - 2,500 = 11,500, the credit uses 4,000.
    let r = supported(&d);
    assert_eq!(value(&r, "form_8815_qualified_expenses"), 10_000);
    assert_eq!(value(&r, "form_8815_educational_benefits"), 2_500);
    // With the 1099-Q: this unit's computed expenses used replace it.
    // College pool = 14,000 - credit 4,000 = 10,000; the 529 distributes 6,000.
    let mut row = student_row(child, CollegeCosts::EducationCredit);
    row.other_college = 0;
    add(&mut d, expense_record(vec![row]));
    add(
        &mut d,
        q1099_form(Owner::Taxpayer, 6_000, qtp(1_500, 4_500), withdrawal(), student(child)),
    );
    let r = supported(&d);
    assert_eq!(cents_of(&r, "education_account_taxable_earnings"), 0);
    assert_eq!(value(&r, "form_8815_educational_benefits"), 6_000);
    assert_eq!(value(&r, "form_8815_net_expenses"), 4_000);
    assert_eq!(value(&r, "form_8815_eligible_interest"), 1_000);
}

// The credit takes its expenses first even where it computes to zero; the
// withdrawal then cannot use them (with a note).
#[test]
fn a_phased_out_credit_still_consumes_its_expenses_with_a_note() {
    let mut d = education_return(FederalStatus::Joint, 200_000);
    let child = d.dependents[0].id;
    let sid2_1006 = sid(&d, 2);
    add_education(d.credits.as_mut().unwrap(), {
        let mut pair = native_education(
            sid2_1006,
            education_expenses(cents(4_000)),
            education_choice(EducationMethod::AmericanOpportunity, Answer::Yes),
        );
        pair.0.id = 0;
        pair.0.half_time = true;
        pair.0.graduate = false;
        pair.1.more.american_opportunity_used_up = Some(false);
        pair
    });
    add(
        &mut d,
        q1099_form(Owner::Taxpayer, 4_000, qtp(1_000, 3_000), withdrawal(), student(child)),
    );
    add(&mut d, expense_record(vec![student_row(child, CollegeCosts::EducationCredit)]));
    let r = supported(&d);
    // The credit is phased out at $200,000, yet its $4,000 of tuition is used:
    // nothing is left for the 529, so all $1,000 of earnings is taxable.
    assert_eq!(value(&r, "education_credit"), 0);
    assert_eq!(cents_of(&r, "education_account_taxable_earnings"), cents(1_000));
    assert!(noted(&r, "NOTE_EDUCATION_ACCOUNT_CREDIT_EXPENSES"));
    // Part II: the credit alone caused the inclusion, so it is excepted.
    assert_eq!(value(&r, "account_taxpayer_exceptions"), 1_000);
    assert_eq!(value(&r, "account_additional_tax"), 0);
}

// Form 4952 and Form 8615 style totals read the earnings like ABLE: the
// income joins other income on the return.
#[test]
fn taxable_earnings_join_total_income_and_the_schedule_1_other_income() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, cents(30_000), 0);
    add(&mut d, q1099_form(Owner::Taxpayer, 10_000, qtp(4_000, 6_000), withdrawal(), recipient()));
    let r = supported(&d);
    assert_eq!(cents_of(&r, "schedule_1_other_income"), cents(4_000));
    assert_eq!(printed_line(&r, "schedule_1_other_income"), (TaxForm::Schedule1, "8z".into()));
    assert_eq!(value(&r, "agi"), 34_000);
}

// The student of a 1099-Q whose recipient is not the beneficiary is a required
// question with no default: asked first on its own page, never offered as an
// option meaning "not answered yet", and only for a distribution that needs it.
#[test]
fn education_student_is_a_required_question_asked_only_when_it_applies() {
    use crate::tests::support::assert::accepted;
    use crate::tests::support::builders::native_income::owned;
    use crate::tests::support::client::income_pages::{
        open_document_center, open_topic, save_documents,
    };
    use crate::tests::support::client::own_questions::{
        answer_questions, save_questions, wire, wire_page,
    };
    let entry = q1099_form(Owner::Taxpayer, 1_000, qtp(200, 800), withdrawal(), recipient());
    let FederalIncome::EducationDistribution { data } = entry.income else { unreachable!() };
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = save_documents(
        &mut session,
        vec![owned(
            Owner::Taxpayer,
            0,
            IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Q1099(data.form) },
        )],
    );
    accepted(&send(&mut session, request));
    let id = session.draft.income.last().unwrap().id;
    // The recipient is the beneficiary: nothing is asked, the page opens.
    let page = open_topic(&mut session, TaxSection::Education);
    let Reply::Edit(P::SaveEducation { followups, questions, .. }) =
        wire_page(&mut session, Reply::Edit(page))
    else {
        panic!("a settled student opens the education page")
    };
    assert!(questions.is_empty());
    assert!(followups.iter().all(|row| !matches!(
        row.answer,
        crate::document_center::EducationDocumentAnswer::EducationBeneficiary { .. }
    )));

    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let row = documents.other.other_income.documents.iter_mut().find(|row| row.id == id).unwrap();
    let crate::OtherIncomeDocument::Q1099(form) = &mut row.document else { unreachable!() };
    form.recipient_not_beneficiary = true;
    accepted(&send(&mut session, request));

    // The student is not known: the questions page comes first.
    let page = open_topic(&mut session, TaxSection::Education);
    let begin = match wire_page(&mut session, Reply::Edit(page)) {
        Reply::Response(begin @ P::BeginDocumentQuestions { .. }) => begin,
        other => panic!("expected the questions page, got {other:?}"),
    };
    let P::BeginDocumentQuestions { topic, questions, .. } = &begin else { unreachable!() };
    assert_eq!(*topic, TaxSection::Education);
    assert_eq!(questions.len(), 1);
    assert_eq!(questions[0].key, alloc::format!("income/{id}/beneficiary"));
    // The question blocks Save: nothing is stored without an answer.
    let before = session.draft.clone();
    assert!(matches!(
        save_questions(&mut session, &begin, Vec::new()),
        Reply::Response(P::Problem { .. })
    ));
    assert_eq!(session.draft, before);

    // The answer is stored once and the education page opens with it.
    let answer = crate::DocumentStrictAnswer::EducationBeneficiary {
        value: Q1099Beneficiary::Student { person: -1 },
    };
    let reply = answer_questions(&mut session, &begin, vec![answer.clone()]);
    let Reply::Edit(page @ P::SaveEducation { .. }) = reply else {
        panic!("expected the education page, got {reply:?}")
    };
    let P::SaveEducation { followups, questions, .. } = &page else { unreachable!() };
    assert!(followups.iter().all(|row| !matches!(
        row.answer,
        crate::document_center::EducationDocumentAnswer::EducationBeneficiary { .. }
    )));
    assert_eq!(questions.len(), 1);
    assert_eq!(crate::OwnAsked::parts(questions[0].clone()).1, answer);
    let entry = session.draft.income.iter().find(|entry| entry.id == id).unwrap();
    let FederalIncome::EducationDistribution { data } = &entry.income else { unreachable!() };
    assert_eq!(data.beneficiary, Some(FederalEducationBeneficiary::Student { person: -1 }));

    // The reopened page keeps the answer across a save and a restart.
    accepted(&wire(&mut session, page));
    let mut restored = restart(session.draft);
    let page = open_topic(&mut restored, TaxSection::Education);
    let Reply::Edit(P::SaveEducation { questions, .. }) =
        wire_page(&mut restored, Reply::Edit(page))
    else {
        panic!("a settled student opens the education page")
    };
    assert_eq!(crate::OwnAsked::parts(questions[0].clone()).1, answer);
}
