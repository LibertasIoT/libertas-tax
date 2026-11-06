use crate::tests::support::assert::payroll::form8919_base;
use crate::tests::support::assert::{accepted, rejected_with};
use crate::tests::support::builders::survey::survey_uncatalogued_document as unanswered_document;
// Relief payments and other income: the nine document
// kinds the Wages, Canceled debt and Other income topics offer (Form 8919,
// health-service loan repayment, scholarship, jury duty, hobby sales, wildfire
// relief, disaster relief, living-expense insurance and hazard mitigation),
// professional gambling on the Business page, and the 1099-R code 3 minimum
// retirement age. Every amount is worked by hand from the 2026 drafts of
// Form 8919 and Schedule 1, Pub. 970 Worksheet 1-1, Pub. 547, Pub. 525 and
// Pub. 596 with the 2026 amounts the App uses ($16,100 single and $32,200
// joint standard deduction; single 10% to $12,400 and 12% to $50,400; joint
// 10% to $24,800 and 12% to $100,800; Tax Table rows $50 wide taxed at their
// midpoint). `complete(Single)` holds one W-2 of $60,000 and
// `complete(Joint)` one of $100,000.
use super::*;
use crate::tests::cross::interview::{business, business_details, from_review, r1099, save_topic};
use crate::tests::support::client::income_pages::{
    open_documents, page_documents, save_documents, topic_for,
};
use alloc::string::String;

fn single() -> Interview {
    Interview::new(TEST_USER, complete(FederalStatus::Single))
}
fn joint() -> Interview {
    Interview::new(TEST_USER, complete(FederalStatus::Joint))
}
// A topic page as the review opens it, with the documents it shows.
fn page(s: &mut Interview, topic: TaxSection) -> P {
    match from_review(s, topic) {
        Reply::Edit(request @ (P::SaveIncomes { .. } | P::SaveWagesAnswers { .. })) => request,
        other => panic!("expected the {topic:?} topic, got {other:?}"),
    }
}
// Saves added sources in their center category and records in their owning topic.
fn save_with(s: &mut Interview, topic: TaxSection, added: Vec<IncomeSource>) -> Reply {
    if topic == TaxSection::Retirement {
        return crate::tests::support::client::save_retirement_fixture(
            s,
            added
                .into_iter()
                .map(|document| unanswered_document(Owner::Taxpayer, 0, document))
                .collect(),
        );
    }

    let (records, sources): (Vec<_>, Vec<_>) = added
        .into_iter()
        .map(|document| unanswered_document(Owner::Taxpayer, 0, document))
        .partition(|row| matches!(row.document, IncomeSource::Record { .. }));
    if !sources.is_empty() {
        let center = open_documents(s);
        let mut rows = page_documents(&center)
            .into_iter()
            .filter(|row| {
                sources.iter().any(|source| {
                    core::mem::discriminant(&source.document)
                        == core::mem::discriminant(&row.document)
                })
            })
            .collect::<Vec<_>>();
        rows.extend(sources);
        let request = save_documents(s, rows);
        let reply = send(s, request);
        if records.is_empty() || matches!(&reply, Reply::Response(P::Problem { .. })) {
            return reply;
        }
    }
    let mut request = page(s, topic);
    match &mut request {
        P::SaveWagesAnswers { records: target, .. } => {
            target.documents.extend(records.into_iter().map(|row| {
                crate::IncomeRecordItem::from_income(row)
                    .and_then(crate::WagesRecordItem::from_income)
                    .unwrap()
            }))
        }
        P::SaveIncomes { documents, .. } => documents.documents.extend(
            records.into_iter().map(|row| crate::IncomeRecordItem::from_income(row).unwrap()),
        ),
        _ => unreachable!(),
    }
    send(s, request)
}
// Replaces source categories through the center and record rows through their topic.
fn save_only(s: &mut Interview, topic: TaxSection, only: Vec<IncomeSource>) -> Reply {
    if topic == TaxSection::Retirement {
        return crate::tests::support::client::save_retirement_fixture(
            s,
            only.into_iter()
                .map(|document| unanswered_document(Owner::Taxpayer, 0, document))
                .collect(),
        );
    }

    let (records, sources): (Vec<_>, Vec<_>) = only
        .into_iter()
        .map(|document| unanswered_document(Owner::Taxpayer, 0, document))
        .partition(|row| matches!(row.document, IncomeSource::Record { .. }));
    if !sources.is_empty() {
        let request = save_documents(s, sources);
        let reply = send(s, request);
        if records.is_empty() || matches!(&reply, Reply::Response(P::Problem { .. })) {
            return reply;
        }
    }
    let request = save_topic(s, topic, records);
    send(s, request)
}
fn shown(s: &mut Interview, topic: TaxSection) -> Vec<IncomeDocumentItem> {
    let mut rows = page_documents(&open_documents(s))
        .into_iter()
        .filter(|row| topic_for(&row.document) == topic)
        .collect::<Vec<_>>();
    rows.extend(page_documents(&page(s, topic)));
    rows
}

fn agi(s: &Interview) -> i64 {
    return_line(&supported(&s.draft), "11a / 11b")
}
fn line16(s: &Interview) -> i64 {
    return_line(&supported(&s.draft), "16")
}
// Stored, restarted and reopened, the page shows the same documents and
// saving them unchanged keeps the return.
fn assert_round_trip(s: &Interview, topic: TaxSection) {
    let saved = s.draft.clone();
    let mut again = restart(saved.clone());
    let center = open_documents(&mut again);
    let has_sources = page_documents(&center).iter().any(|row| topic_for(&row.document) == topic);
    if has_sources {
        accepted(&send(&mut again, center));
    }
    let request = page(&mut again, topic);
    if !has_sources || !page_documents(&request).is_empty() {
        accepted(&send(&mut again, request));
    }
    assert_eq!(again.draft, saved);
}

fn pay(wages: i64, reason: EmployeeWageReason, form1099: bool) -> EmployeeWages {
    EmployeeWages {
        source_issuer: Default::default(),
        source_recipient: Default::default(),
        wages,
        withholding: 0,
        form1099,
        payroll: crate::EmployeePayrollSource::Form8919 { reason },
    }
}
fn code_a() -> EmployeeWageReason {
    EmployeeWageReason::Ss8Determination { date: 20260501 }
}

// ---- A: Form 8919 ---------------------------------------------------------

#[test]
fn a_form_8919_pay_reaches_line_1g_and_the_uncollected_tax_by_hand() {
    // Form 8919 (2026 draft), reason A, pay $5,000, no withholding:
    //   line 6 = 5,000 -> Form 1040 line 1g
    //   line 8 = 60,000 (W-2 box 3); line 9 = 184,500 - 60,000 = 124,500
    //   line 10 = min(5,000, 124,500) = 5,000
    //   line 11 = 6.2% x 5,000 = 310; line 12 = 1.45% x 5,000 = 72.50
    //   (the App rounds each line to whole dollars: 73); line 13 = 383
    //   AGI = 60,000 + 5,000 = 65,000; taxable = 65,000 - 16,100 = 48,900
    //   line 16: midpoint 48,925: 1,240 + 12% x (48,925 - 12,400) = 5,623
    let mut s = single();
    accepted(&save_with(
        &mut s,
        TaxSection::Wages,
        vec![IncomeSource::Wage {
            value: crate::WageDocument::Form8919(pay(500_000, code_a(), true)),
        }],
    ));
    let stored: Vec<_> = s
        .draft
        .income
        .iter()
        .filter(|e| matches!(e.income, FederalIncome::EmployeeWages { .. }))
        .collect();
    assert_eq!(stored.len(), 1);
    // An unnamed entry stores no name; it displays by kind and id.
    assert_eq!(stored[0].label, "");
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "1g"), 500_000);
    assert_eq!(return_line(&r, "1z"), 6_500_000);
    assert_eq!(value(&r, "employee_wages_0_pay"), 5000);
    assert_eq!(value(&r, "employee_wages_0_social_base"), 5000);
    assert_eq!(value(&r, "employee_wages_0_social_tax"), 310);
    assert_eq!(value(&r, "employee_wages_0_medicare_tax"), 73);
    assert_eq!(value(&r, "employee_wages_0_tax"), 383);
    assert_eq!(return_line(&r, "11a / 11b"), 6_500_000);
    assert_eq!(return_line(&r, "15"), 4_890_000);
    assert_eq!(return_line(&r, "16"), 562_300);
    form8919_base(&r, 0, 60_000, 124_500);
    // The other lines complete the same printed Form 8919.
    for (id, line) in [
        ("employee_wages_0_pay", "6"),
        ("employee_wages_0_social_base", "10"),
        ("employee_wages_0_social_tax", "11"),
        ("employee_wages_0_medicare_tax", "12"),
        ("employee_wages_0_tax", "13"),
    ] {
        let found = r.lines.iter().find(|l| l.id == id).unwrap();
        assert_eq!(found.form_line, Some(FormLine::new(TaxForm::Form8919, line)), "{id}");
    }
    // The row shows the pay: the supported estimate's working copy.
    let rows = shown(&mut s, TaxSection::Wages);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[1].preview, Some(500_000));
    assert_round_trip(&s, TaxSection::Wages);
    assert_eq!(supported(&restart(s.draft).draft), r);
}

#[test]
fn a_form_8919_social_base_stops_at_the_wage_base() {
    // The W-2 box 3 wages are $180,000: line 9 = 184,500 - 180,000 = 4,500;
    // line 10 = min(5,000, 4,500) = 4,500; line 11 = 6.2% x 4,500 = 279;
    // line 12 = 1.45% x 5,000 = 73 (72.50); line 13 = 352.
    for (wages, used, left, ss_tax) in [
        (0, 0, 184_500, 310),
        (18_000_000, 180_000, 4_500, 279),
        (18_000_049, 180_000, 4_500, 279),
        (18_000_050, 180_001, 4_499, 279),
        (18_450_000, 184_500, 0, 0),
    ] {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, wages, 0);
        if wages == 0 {
            d.income.clear();
        }
        let mut s = Interview::new(TEST_USER, d);
        accepted(&save_with(
            &mut s,
            TaxSection::Wages,
            vec![IncomeSource::Wage {
                value: crate::WageDocument::Form8919(pay(500_000, code_a(), true)),
            }],
        ));
        let r = supported(&s.draft);
        form8919_base(&r, 0, used, left);
        assert_eq!(value(&r, "employee_wages_0_social_base"), left.min(5_000));
        assert_eq!(value(&r, "employee_wages_0_social_tax"), ss_tax);
        assert_eq!(value(&r, "employee_wages_0_medicare_tax"), 73);
        assert_eq!(value(&r, "employee_wages_0_tax"), ss_tax + 73);
    }
}

#[test]
fn a_form_8919_community_income_keeps_the_physical_employees_base() {
    let mut d = community_w2_draft(18_000_000, 6_000_000, 0, 0);
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut source = income(omitted_employee_wages(5_000));
        source.owner = crate::DocumentOwner::from(owner);
        source.community = CommunityShare::Community;
        add(&mut d, source);
    }
    let r = supported(&d);
    // Income is divided; only this filer's own $180,000 exhausts their base.
    assert_eq!(return_line(&r, "1a"), 12_000_000);
    assert_eq!(return_line(&r, "1g"), 500_000);
    form8919_base(&r, 0, 180_000, 4_500);
    assert_eq!(value(&r, "employee_wages_0_social_base"), 4_500);
    assert_eq!(value(&r, "employee_wages_0_social_tax"), 279);
    assert_eq!(value(&r, "employee_wages_0_medicare_tax"), 73);
    assert!(
        !r.lines
            .iter()
            .any(|row| { row.id.starts_with("employee_wages_1_") && row.form_line.is_some() })
    );
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn a_form_8919_firm_outside_ordinary_coverage_retains_pay_with_a_notice() {
    // 2026-10-19 actual-source split: unavailable payroll coverage is not zero tax.
    let mut s = single();
    let mut firm = pay(500_000, code_a(), true);
    firm.payroll = crate::EmployeePayrollSource::Unavailable;
    accepted(&save_with(
        &mut s,
        TaxSection::Wages,
        vec![IncomeSource::Wage { value: crate::WageDocument::Form8919(firm) }],
    ));
    let r = estimate(&s.draft);
    assert!(noted(&r, "WARN_EMPLOYEE_WAGES_PAYROLL_TAX_LEFT_OUT"));
    assert_eq!(value(&r, "agi"), 65_000);
    assert_eq!(r.state, ResultState::Incomplete);
    assert_eq!(r.tax, None);
    let source = s.draft.income.last().unwrap();
    assert!(
        r.issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_EMPLOYEE_PAYROLL_COVERAGE_UNAVAILABLE"
                && issue.context == crate::interview::income_issue_context(source))
    );
    let rows = shown(&mut s, TaxSection::Wages);
    assert_eq!(rows[1].preview, Some(500_000));
    assert_round_trip(&s, TaxSection::Wages);
}

#[test]
fn a_form_8919_code_h_needs_the_form_1099_and_withholding_stays_within_the_pay() {
    let mut s = single();
    let before = s.draft.clone();
    rejected_with(
        save_with(
            &mut s,
            TaxSection::Wages,
            vec![IncomeSource::Wage {
                value: crate::WageDocument::Form8919(pay(
                    500_000,
                    EmployeeWageReason::SameFirmForms,
                    false,
                )),
            }],
        ),
        "FORM_8919_CODE_H_NEEDS_1099",
    );
    let mut firm = pay(500_000, code_a(), true);
    firm.withholding = 500_001;
    rejected_with(
        save_with(
            &mut s,
            TaxSection::Wages,
            vec![IncomeSource::Wage { value: crate::WageDocument::Form8919(firm) }],
        ),
        "FORM_8919_WITHHOLDING_ABOVE_PAY",
    );
    assert_eq!(s.draft, before);
    // Code H with the Form 1099 is accepted; withholding of a 1099 is a
    // Form 1040 line 25b credit.
    let mut firm = pay(500_000, EmployeeWageReason::SameFirmForms, true);
    firm.withholding = 50_000;
    accepted(&save_with(
        &mut s,
        TaxSection::Wages,
        vec![IncomeSource::Wage { value: crate::WageDocument::Form8919(firm) }],
    ));
    assert_eq!(return_line(&supported(&s.draft), "25b"), 50_000);
}

#[test]
fn a_form_8919_import_reads_the_1099_nec_and_1099_misc_boxes_only() {
    let source = include_str!("../../income/employee_wages.rs");
    let head = &source[..source.find("pub struct EmployeeWages {").unwrap()];
    let action = &head[head.rfind("#[libertas_llm_doc_trans(").unwrap()..];
    // 2026-10-27: EmployeeWages imports established payroll facts alongside
    // printed amounts, and leaves facts the supplied documents cannot settle unavailable.
    assert!(action.contains("For a supplied Form 1099-NEC read box 1 as wages"));
    assert!(action.contains("for Form 1099-MISC read box 3 as wages"));
    assert!(action.contains("a blank required amount is an error"));
    assert!(action.contains("Read box 4 federal income tax withholding"));
    assert!(action.contains("a clearly blank withholding box is zero"));
    assert!(action.contains("Set form1099 true for either 1099"));
    assert!(action.contains(
        "A 1099 alone does not establish employee status, payroll coverage or a Form 8919 reason"
    ));
    assert!(action.contains(
        "Fill payroll from the actual reason in column (c) and its required date only when the supplied records establish both"
    ));
    assert!(action.contains("do not invent a filing or determination date"));
    assert!(action.contains(
        "Fill payroll only from the payroll system and coverage established by the supplied document; use Unavailable when those facts are not settled"
    ));
    assert!(action.contains("Do not infer exemption from foreign work or a government employer"));
    assert!(action.contains("validation = \"$.document_facts\""));
    let page = &source[source.find("pub struct EmployeeWages {").unwrap()..];
    let page = &page[..page.find("\n}").unwrap()];
    let imported: Vec<&str> = marked_fields(page);
    // These are the same typed fields a person can fill from the documents.
    assert_eq!(
        imported,
        vec!["source_issuer", "source_recipient", "wages", "withholding", "form1099", "payroll"]
    );
}

// The fields a struct body marks for AI import.
fn marked_fields(body: &str) -> Vec<&str> {
    body.lines()
        .scan(false, |marked, line| {
            let text = line.trim();
            if text.starts_with("#[libertas_llm_doc_field") {
                *marked = true;
                return Some(None);
            }
            if *marked && text.starts_with("pub ") {
                *marked = false;
                return Some(text.trim_start_matches("pub ").split_once(':').map(|(n, _)| n));
            }
            Some(None)
        })
        .flatten()
        .collect()
}

#[test]
fn the_w2_and_1099_r_pages_take_a_form_4852_substitute() {
    let model = crate::federal_model::HUB_SOURCE;
    // The W-2 import reads a Form 4852 line 7 as the W-2 boxes and nothing else.
    let w2 = &model[..model.find("pub struct W2 {").unwrap()];
    let action = &w2[w2.rfind("#[libertas_llm_doc_trans(").unwrap()..];
    assert!(action.contains("If the file is a Form 4852, read line 7a as box 1"));
    assert!(action.contains("7e as box 2"));
    assert!(action.contains("7b as box 3"));
    assert!(action.contains("7h as box 4"));
    assert!(action.contains("7c as box 5"));
    assert!(action.contains("7i as box 6"));
    assert!(action.contains("7d as box 7"));
    assert!(action.contains("report the employer name from line 5; read no other line."));
    let heading = &w2[w2.rfind("/// W-2\n").unwrap()..];
    assert!(
        heading.contains("If your employer gave you no W-2, enter the amounts from your Form 4852")
    );
    // The 1099-R page names Form 4852 line 8.
    let r = &model[..model.find("pub struct R1099 {").unwrap()];
    let heading = &r[r.rfind("/// 1099-R\n").unwrap()..];
    assert!(heading.contains("Form 4852 line 8"));
    // A Form 4852 is no stored kind.
    assert!(!model.contains("Form4852("));
}

// ---- B: health-service loan repayment --------------------------------------

fn health_loan(amount: i64, interest: i64, confirmed: bool) -> crate::HealthLoanAssistance {
    crate::HealthLoanAssistance {
        program: HealthLoanProgram::National,
        amount,
        interest,
        confirmed,
    }
}

#[test]
fn b_health_service_loan_repayment_is_excluded_and_its_interest_is_not_deducted_twice() {
    // The 1098-E interest is $1,500; the NHSC program paid $20,000 of the
    // loan, $900 of it interest. Confirmed: the $20,000 is excluded; the
    // deductible interest is 1,500 - 900 = 600 (MAGI 60,000 < 85,000).
    //   AGI = 60,000 - 600 = 59,400; taxable = 59,400 - 16,100 = 43,300
    //   line 16: midpoint 43,325: 1,240 + 12% x (43,325 - 12,400) = 4,951
    let mut d = complete(FederalStatus::Single);
    d.adjustments.as_mut().unwrap().set_student_loan_interest(150_000);
    let mut s = Interview::new(TEST_USER, d);
    accepted(&save_with(
        &mut s,
        TaxSection::CanceledDebt,
        vec![IncomeSource::Record {
            value: crate::IncomeRecord::HealthLoanRepayment(health_loan(2_000_000, 90_000, true)),
        }],
    ));
    let r = supported(&s.draft);
    assert_eq!(agi(&s), 5_940_000);
    assert_eq!(return_line(&r, "16"), 495_100);
    assert_eq!(value(&r, "personal_debt_cancellation"), 0);
    // An excluded repayment previews zero.
    let rows = shown(&mut s, TaxSection::CanceledDebt);
    assert_eq!(rows[0].preview, Some(0));
    assert_round_trip(&s, TaxSection::CanceledDebt);
    // Not sure: the program is assumed not to qualify; the $20,000 is income
    // (Schedule 1 line 8z other income per Pub. 970 ch. 5, not canceled debt
    // on line 8c), the interest stays deductible (MAGI 80,000 <
    // 85,000), AGI = 60,000 + 20,000 - 1,500 = 78,500, with a note.
    let FederalIncome::HealthLoanAssistance { data } = &mut s.draft.income[1].income else {
        unreachable!()
    };
    data.confirmed = false;
    assert_eq!(agi(&s), 7_850_000);
    let r = supported(&s.draft);
    assert!(noted(&r, "WARN_HEALTH_LOAN_ASSISTANCE_ASSUMED_TAXABLE"));
    assert_eq!(value(&r, "personal_debt_cancellation"), 0);
    assert_eq!(value(&r, "taxable_health_loan_assistance"), 20_000);
    assert_eq!(value(&r, "schedule_1_other_income"), 20_000);
    // The row now previews the whole payment.
    let rows = shown(&mut s, TaxSection::CanceledDebt);
    assert_eq!(rows[0].preview, Some(2_000_000));
}

#[test]
fn b_health_loan_interest_above_the_assistance_or_the_1098_e_total_is_refused() {
    let mut d = complete(FederalStatus::Single);
    d.adjustments.as_mut().unwrap().set_student_loan_interest(150_000);
    let mut s = Interview::new(TEST_USER, d);
    // The page rule: interest paid through the program cannot exceed the
    // assistance.
    rejected_with(
        save_with(
            &mut s,
            TaxSection::CanceledDebt,
            vec![IncomeSource::Record {
                value: crate::IncomeRecord::HealthLoanRepayment(health_loan(
                    100_000, 100_001, true,
                )),
            }],
        ),
        "HEALTH_LOAN_INTEREST_ABOVE_ASSISTANCE",
    );
    // Interest $1,600 above the 1098-E's $1,500 is the existing consistency
    // issue, offered with the Student loan page.
    accepted(&save_with(
        &mut s,
        TaxSection::CanceledDebt,
        vec![IncomeSource::Record {
            value: crate::IncomeRecord::HealthLoanRepayment(health_loan(2_000_000, 160_000, true)),
        }],
    ));
    unsupported(&s.draft, "Health-service");
}

// ---- C: wildfire relief -----------------------------------------------------

fn wildfire(declared: u32, qualified: bool) -> crate::FederalWildfireRelief {
    crate::FederalWildfireRelief {
        source_issuer: Default::default(),
        source_recipient: Default::default(),
        paid: 20260310,
        declared,
        reference: "FEMA-FM-5000".into(),
        amount: 2_500_000,
        withholding: 50_000,
        qualified,
    }
}

#[test]
fn c_wildfire_relief_is_excluded_when_confirmed_and_income_otherwise() {
    // $25,000 paid 2026-03-10 for a fire declared 2025-01-08 (section 139M,
    // P.L. 119-108). Confirmed: excluded, nothing on the return.
    //   AGI = 100,000; taxable = 100,000 - 32,200 = 67,800
    //   line 16: midpoint 67,825: 2,480 + 12% x (67,825 - 24,800) = 7,643
    // Not sure: taxed on Schedule 1 line 8z, AGI = 125,000; taxable 92,800;
    //   midpoint 92,825: 2,480 + 12% x 68,025 = 10,643, a difference of
    //   3,000 = 12% x 25,000.
    // The 1099-MISC box 4 withholding of $500 is a line 25b credit either way.
    let mut s = joint();
    accepted(&save_with(
        &mut s,
        TaxSection::OtherIncome,
        vec![IncomeSource::OtherIncome {
            value: crate::OtherIncomeDocument::WildfireRelief(wildfire(20250108, true)),
        }],
    ));
    let r = supported(&s.draft);
    assert_eq!(agi(&s), 10_000_000);
    assert_eq!(return_line(&r, "16"), 764_300);
    assert_eq!(return_line(&r, "25b"), 50_000);

    let rows = shown(&mut s, TaxSection::OtherIncome);
    assert_eq!(rows[0].preview, Some(0));
    assert_round_trip(&s, TaxSection::OtherIncome);
    let FederalIncome::WildfireRelief { data } = &mut s.draft.income[1].income else {
        unreachable!()
    };
    data.qualified = false;
    let r = supported(&s.draft);
    assert_eq!(agi(&s), 12_500_000);
    assert_eq!(return_line(&r, "16"), 1_064_300);
    assert_eq!(return_line(&r, "25b"), 50_000);
    assert_eq!(value(&r, "schedule_1_other_income"), 25_000);

    let rows = shown(&mut s, TaxSection::OtherIncome);
    assert_eq!(rows[0].preview, Some(2_500_000));
    // No: taxed as well, without the note.
    let FederalIncome::WildfireRelief { data } = &mut s.draft.income[1].income else {
        unreachable!()
    };
    data.qualified = false;
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "16"), 1_064_300);
}

#[test]
fn c_wildfire_declaration_must_be_from_2015_through_2026() {
    let mut s = joint();
    let before = s.draft.clone();
    for declared in [20270104, 20141231] {
        rejected_with(
            save_with(
                &mut s,
                TaxSection::OtherIncome,
                vec![IncomeSource::OtherIncome {
                    value: crate::OtherIncomeDocument::WildfireRelief(wildfire(declared, true)),
                }],
            ),
            "WILDFIRE_DECLARED_OUTSIDE_2015_2026",
        );
    }
    // A payment not received in 2026, or withholding above the payment.
    let mut late = wildfire(20250108, true);
    late.paid = 20251231;
    rejected_with(
        save_with(
            &mut s,
            TaxSection::OtherIncome,
            vec![IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::WildfireRelief(late),
            }],
        ),
        "RELIEF_RECEIVED_OUTSIDE_2026",
    );
    let mut over = wildfire(20250108, true);
    over.withholding = over.amount + 1;
    rejected_with(
        save_with(
            &mut s,
            TaxSection::OtherIncome,
            vec![IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::WildfireRelief(over),
            }],
        ),
        "RELIEF_WITHHOLDING_ABOVE_PAYMENT",
    );
    assert_eq!(s.draft, before);
    // The boundary days are accepted.
    for declared in [20150101, 20261231] {
        accepted(&save_with(
            &mut s,
            TaxSection::OtherIncome,
            vec![IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::WildfireRelief(wildfire(declared, true)),
            }],
        ));
    }
}

#[test]
fn c_wildfire_and_disaster_imports_read_boxes_3_4_and_6_only() {
    let model = crate::federal_model::HUB_SOURCE;
    for (name, boxes) in [
        (
            "pub struct FederalWildfireRelief {",
            &["box 3 other income", "box 4 federal income tax withheld"][..],
        ),
        (
            "pub struct FederalDisasterRelief {",
            &[
                "Form 1099-MISC or Form 1099-G",
                "for a 1099-G read box 6 taxable grants",
                "box 4 federal income tax withheld",
            ][..],
        ),
    ] {
        let start = model.find(name).unwrap();
        let head = &model[..start];
        let action = &head[head.rfind("#[libertas_llm_doc_trans(").unwrap()..];
        for text in boxes {
            assert!(action.contains(text), "{name}: {text}");
        }
        assert!(action.contains("Do not read any other box."));
        assert!(action.contains("validation = \"$.document_facts\""));
        let body = &model[start..];
        let body = &body[..body.find("\n}").unwrap()];
        // Printed identity, amount and withholding: dates, references, bases,
        // purposes and the qualification are the filer's.
        assert_eq!(
            marked_fields(body),
            vec!["source_issuer", "source_recipient", "amount", "withholding"],
            "{name}"
        );
    }
    // The 1099-G page sends a qualified grant here.
    assert!(
        model.contains("Leave out a qualified disaster relief grant; enter it under Other income.")
    );
    assert!(model.contains("leave it out of that 1099-G's box 6"));
}

// ---- D: disaster relief -----------------------------------------------------

fn disaster(
    qualification: DisasterQualification,
    purpose: DisasterReliefPurpose,
    qualified: bool,
) -> crate::FederalDisasterRelief {
    crate::FederalDisasterRelief {
        source_issuer: Default::default(),
        source_recipient: Default::default(),
        received: 20260612,
        qualification,
        purpose,
        amount: 800_000,
        withholding: 0,
        qualified,
    }
}
fn declared() -> DisasterQualification {
    DisasterQualification::FederalDeclaration { reference: "DR-4800".into() }
}

#[test]
fn d_disaster_relief_is_excluded_when_confirmed_and_income_otherwise() {
    // $8,000 from a state government after a federally declared disaster.
    // Confirmed: excluded, tax 7,643 (see C). No: Schedule 1 line 8z 8,000,
    //   AGI 108,000; taxable 75,800; midpoint 75,825:
    //   2,480 + 12% x (75,825 - 24,800) = 2,480 + 6,123 = 8,603.
    let mut s = joint();
    accepted(&save_with(
        &mut s,
        TaxSection::OtherIncome,
        vec![IncomeSource::OtherIncome {
            value: crate::OtherIncomeDocument::DisasterRelief(disaster(
                declared(),
                DisasterReliefPurpose::GovernmentWelfare,
                true,
            )),
        }],
    ));
    assert_eq!(line16(&s), 764_300);
    let rows = shown(&mut s, TaxSection::OtherIncome);
    assert_eq!(rows[0].preview, Some(0));
    assert_round_trip(&s, TaxSection::OtherIncome);
    let FederalIncome::DisasterRelief { data } = &mut s.draft.income[1].income else {
        unreachable!()
    };
    data.qualified = false;
    let r = supported(&s.draft);
    assert_eq!(agi(&s), 10_800_000);
    assert_eq!(return_line(&r, "16"), 860_300);
    assert_eq!(value(&r, "schedule_1_other_income"), 8000);

    let FederalIncome::DisasterRelief { data } = &mut s.draft.income[1].income else {
        unreachable!()
    };
    data.qualified = false;
}

#[test]
fn d_a_carrier_accident_or_government_assistance_needs_the_matching_purpose() {
    let mut s = joint();
    let before = s.draft.clone();
    rejected_with(
        save_with(
            &mut s,
            TaxSection::OtherIncome,
            vec![IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::DisasterRelief(disaster(
                    DisasterQualification::CarrierAccident { event: "Rail accident".into() },
                    DisasterReliefPurpose::PersonalExpense,
                    true,
                )),
            }],
        ),
        "DISASTER_CARRIER_ACCIDENT_NEEDS_CARRIER_PAYMENT",
    );
    rejected_with(
        save_with(
            &mut s,
            TaxSection::OtherIncome,
            vec![IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::DisasterRelief(disaster(
                    DisasterQualification::GovernmentAssistance {
                        event: "Flood".into(),
                        authority: "State".into(),
                        payer: "County".into(),
                    },
                    DisasterReliefPurpose::PersonalExpense,
                    true,
                )),
            }],
        ),
        "DISASTER_ASSISTANCE_NEEDS_GOVERNMENT_PAYMENT",
    );
    assert_eq!(s.draft, before);
    // The matching purposes are accepted.
    accepted(&save_with(
        &mut s,
        TaxSection::OtherIncome,
        vec![IncomeSource::OtherIncome {
            value: crate::OtherIncomeDocument::DisasterRelief(disaster(
                DisasterQualification::CarrierAccident { event: "Rail accident".into() },
                DisasterReliefPurpose::CommonCarrier {
                    carrier: "Rail Co".into(),
                    loss: DisasterCarrierLoss::PhysicalInjury,
                },
                true,
            )),
        }],
    ));
}

// ---- E: living-expense insurance --------------------------------------------

fn living(
    federal_area: bool,
    regained: Option<u32>,
    confirmed: bool,
) -> crate::FederalCasualtyLivingInsurance {
    crate::FederalCasualtyLivingInsurance {
        event_reference: "Kitchen fire".into(),
        loss_of_use: FederalHomeLossOfUse::Casualty,
        lost_use_on: 20260601,
        regained_use_on: regained,
        payments: vec![FederalCasualtyLivingPayment {
            insurer: "Synthetic Mutual".into(),
            claim_reference: "CLM-1".into(),
            payment_reference: "PAY-1".into(),
            received_on: 20260610,
            amount: 220_000,
            withholding: 0,
        }],
        actual_living_expenses: 385_000,
        normal_living_expenses: 190_000,
        prior_taxable_reported: 0,
        federal_disaster_area: federal_area,
        federal_declaration_reference: if federal_area { "DR-4801".into() } else { String::new() },
        confirmed,
    }
}

#[test]
fn e_living_expense_insurance_taxes_only_the_excess_over_the_temporary_increase() {
    // Pub. 547's example with 2026 dates: lost use 2026-06-01, regained
    // 2026-07-01; one payment of $2,200; actual expenses 3,850 less normal
    // 1,900 = a temporary increase of 1,950. Not a federal disaster area:
    // taxable = 2,200 - 1,950 = 250 on Schedule 1 line 8z.
    //   AGI = 100,250; taxable = 100,250 - 32,200 = 68,050
    //   line 16: midpoint 68,075: 2,480 + 12% x 43,275 = 7,673
    let mut s = joint();
    accepted(&save_with(
        &mut s,
        TaxSection::OtherIncome,
        vec![IncomeSource::Record {
            value: crate::IncomeRecord::CasualtyLivingInsurance(living(
                false,
                Some(20260701),
                true,
            )),
        }],
    ));
    let r = supported(&s.draft);
    assert_eq!(agi(&s), 10_025_000);
    assert_eq!(return_line(&r, "16"), 767_300);
    assert_eq!(value(&r, "schedule_1_other_income"), 250);
    let rows = shown(&mut s, TaxSection::OtherIncome);
    assert_eq!(rows[0].preview, Some(25_000));
    assert_round_trip(&s, TaxSection::OtherIncome);
    // A federally declared disaster area: nothing is taxable (Pub. 547).
    let FederalIncome::CasualtyLivingInsurance { data } = &mut s.draft.income[1].income else {
        unreachable!()
    };
    *data = living(true, Some(20260701), true);
    assert_eq!(agi(&s), 10_000_000);
    assert_eq!(line16(&s), 764_300);
    // Still unusable at the end of 2026: no income yet.
    let FederalIncome::CasualtyLivingInsurance { data } = &mut s.draft.income[1].income else {
        unreachable!()
    };
    *data = living(false, None, true);
    assert_eq!(agi(&s), 10_000_000);
    // Not confirmed: every 2026 payment is taxable, with a note.
    let FederalIncome::CasualtyLivingInsurance { data } = &mut s.draft.income[1].income else {
        unreachable!()
    };
    *data = living(false, Some(20260701), false);
    let r = supported(&s.draft);
    assert_eq!(agi(&s), 10_220_000);
    assert!(noted(&r, "WARN_CASUALTY_LIVING_PAYMENTS_TAXED"), "{r:?}");
}

#[test]
fn e_the_fema_reference_goes_with_a_disaster_area_and_use_is_regained_after_it_was_lost() {
    let mut s = joint();
    let before = s.draft.clone();
    let mut v = living(true, Some(20260701), true);
    v.federal_declaration_reference = String::new();
    rejected_with(
        save_with(
            &mut s,
            TaxSection::OtherIncome,
            vec![IncomeSource::Record { value: crate::IncomeRecord::CasualtyLivingInsurance(v) }],
        ),
        "CASUALTY_LIVING_DISASTER_NEEDS_REFERENCE",
    );
    let mut v = living(false, Some(20260701), true);
    v.federal_declaration_reference = "DR-4801".into();
    rejected_with(
        save_with(
            &mut s,
            TaxSection::OtherIncome,
            vec![IncomeSource::Record { value: crate::IncomeRecord::CasualtyLivingInsurance(v) }],
        ),
        "CASUALTY_LIVING_REFERENCE_WITHOUT_DISASTER",
    );
    rejected_with(
        save_with(
            &mut s,
            TaxSection::OtherIncome,
            vec![IncomeSource::Record {
                value: crate::IncomeRecord::CasualtyLivingInsurance(living(
                    false,
                    Some(20260531),
                    true,
                )),
            }],
        ),
        "CASUALTY_LIVING_REGAINED_BEFORE_LOST",
    );
    assert_eq!(s.draft, before);
}

// ---- F: hazard mitigation ---------------------------------------------------

fn mitigation(qualified: bool) -> crate::FederalDisasterMitigation {
    crate::FederalDisasterMitigation {
        received: 20260415,
        award_reference: "HMGP-12".into(),
        program: DisasterMitigationProgram::StaffordAct,
        property: "12 Elm Street".into(),
        amount: 1_500_000,
        qualified,
    }
}

#[test]
fn f_a_hazard_mitigation_grant_is_excluded_or_left_out_and_previews_zero() {
    // $15,000 Stafford Act grant received 2026-04-15. Qualified: excluded,
    // tax 7,643 (see C), preview 0. Not qualified: left out with a notice,
    // the same tax, preview 0.
    let mut s = joint();
    accepted(&save_with(
        &mut s,
        TaxSection::OtherIncome,
        vec![IncomeSource::Record {
            value: crate::IncomeRecord::DisasterMitigation(mitigation(true)),
        }],
    ));
    assert_eq!(line16(&s), 764_300);
    assert!(!noted(&supported(&s.draft), "WARN_DISASTER_MITIGATION_LEFT_OUT"));
    assert_eq!(shown(&mut s, TaxSection::OtherIncome)[0].preview, Some(0));
    assert_round_trip(&s, TaxSection::OtherIncome);
    let FederalIncome::DisasterMitigation { data } = &mut s.draft.income[1].income else {
        unreachable!()
    };
    data.qualified = false;
    let r = supported(&s.draft);
    assert!(noted(&r, "WARN_DISASTER_MITIGATION_LEFT_OUT"), "{r:?}");
    assert_eq!(return_line(&r, "16"), 764_300);
    assert_eq!(shown(&mut s, TaxSection::OtherIncome)[0].preview, Some(0));
    rejected_with(
        save_with(
            &mut s,
            TaxSection::OtherIncome,
            vec![IncomeSource::Record {
                value: crate::IncomeRecord::DisasterMitigation(crate::FederalDisasterMitigation {
                    received: 20270101,
                    ..mitigation(true)
                }),
            }],
        ),
        "RELIEF_RECEIVED_OUTSIDE_2026",
    );
}

// ---- G: scholarship ---------------------------------------------------------

fn award(
    received: i64,
    restricted: i64,
    qualified_paid: i64,
    services: ScholarshipService,
) -> Scholarship {
    Scholarship::Education {
        data: crate::EducationAward {
            received,
            restricted,
            qualified_paid,
            degree_candidate: true,
            services,
            ordinary: true,
        },
    }
}

#[test]
fn g_scholarship_follows_pub_970_worksheet_1_1() {
    // Degree candidate; award 10,000, restricted to room and board 2,000,
    // 3,000 pays for required services, 5,000 spent on qualified expenses
    // (qualified education expenses 9,000). Worksheet 1-1:
    //   1 = 10,000; 2 = 3,000; 3 = 7,000; 4 = 2,000; 5 = 5,000; 6 = 9,000
    //   7 = min(5, 6) = 5,000; 8 = 0; 9 = 3,000 + 2,000 + (5,000 - 5,000)
    //   = 5,000 = 10,000 - 5,000 -> Schedule 1 line 8r
    //   AGI = 65,000; taxable = 48,900; line 16: 5,623 (see A)
    let mut s = single();
    accepted(&save_with(
        &mut s,
        TaxSection::OtherIncome,
        vec![IncomeSource::Record {
            value: crate::IncomeRecord::Scholarship(award(
                1_000_000,
                200_000,
                500_000,
                ScholarshipService::PaidServices { amount: 300_000 },
            )),
        }],
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "taxable_scholarships"), 5000);
    assert_eq!(agi(&s), 6_500_000);
    assert_eq!(return_line(&r, "16"), 562_300);
    assert_eq!(
        r.lines.iter().find(|l| l.id == "taxable_scholarships").unwrap().form_line,
        Some(FormLine::new(TaxForm::Schedule1, "8r"))
    );
    // The row previews the taxable amount.
    assert_eq!(shown(&mut s, TaxSection::OtherIncome)[0].preview, Some(500_000));
    assert_round_trip(&s, TaxSection::OtherIncome);
    // Without services and with 7,000 spent (<= 10,000 - 2,000): taxable
    // 3,000; AGI 63,000; taxable income 46,900; midpoint 46,925:
    // 1,240 + 12% x 34,525 = 5,383.
    accepted(&save_only(
        &mut s,
        TaxSection::OtherIncome,
        vec![IncomeSource::Record {
            value: crate::IncomeRecord::Scholarship(award(
                1_000_000,
                200_000,
                700_000,
                ScholarshipService::None,
            )),
        }],
    ));
    assert_eq!(agi(&s), 6_300_000);
    assert_eq!(line16(&s), 538_300);
    // A candidate for no degree is taxed on the whole award.
    let mut grant = award(1_000_000, 200_000, 700_000, ScholarshipService::None);
    if let Scholarship::Education { data: crate::EducationAward { degree_candidate, .. } } =
        &mut grant
    {
        *degree_candidate = false;
    }
    accepted(&save_only(
        &mut s,
        TaxSection::OtherIncome,
        vec![IncomeSource::Record { value: crate::IncomeRecord::Scholarship(grant) }],
    ));
    assert_eq!(agi(&s), 7_000_000);
}

#[test]
fn g_scholarship_spending_services_and_restriction_stay_within_the_award() {
    // 6,000 spent + 2,000 restricted + 3,000 for services > 10,000.
    let mut s = single();
    let before = s.draft.clone();
    rejected_with(
        save_with(
            &mut s,
            TaxSection::OtherIncome,
            vec![IncomeSource::Record {
                value: crate::IncomeRecord::Scholarship(award(
                    1_000_000,
                    200_000,
                    600_000,
                    ScholarshipService::PaidServices { amount: 300_000 },
                )),
            }],
        ),
        "SCHOLARSHIP_SPENDING_ABOVE_AWARD",
    );
    // Spending and restriction alone above the award.
    rejected_with(
        save_with(
            &mut s,
            TaxSection::OtherIncome,
            vec![IncomeSource::Record {
                value: crate::IncomeRecord::Scholarship(award(
                    1_000_000,
                    200_000,
                    800_001,
                    ScholarshipService::None,
                )),
            }],
        ),
        "SCHOLARSHIP_SPENDING_ABOVE_AWARD",
    );
    assert_eq!(s.draft, before);
    // Exactly the award is accepted.
    accepted(&save_with(
        &mut s,
        TaxSection::OtherIncome,
        vec![IncomeSource::Record {
            value: crate::IncomeRecord::Scholarship(award(
                1_000_000,
                200_000,
                500_000,
                ScholarshipService::PaidServices { amount: 300_000 },
            )),
        }],
    ));
}

#[test]
fn g_a_graduate_stipend_is_ira_compensation() {
    // A graduate non-tuition stipend of $30,000 is compensation for an IRA
    // (section 219(f)(1), Pub. 590-A): a $7,500 traditional IRA contribution
    // is deductible beside it, and not beside a stipend not confirmed as
    // eligible. It is not EIC earned income.
    let mut d = empty_return(FederalStatus::Single);
    d.adjustments.as_mut().unwrap().iras = ira_facts(750_000, 0, Answer::No);
    let mut s = Interview::new(TEST_USER, d);
    for eligible in [true, false] {
        accepted(&save_only(
            &mut s,
            TaxSection::OtherIncome,
            vec![IncomeSource::Record {
                value: crate::IncomeRecord::Scholarship(Scholarship::GraduateStipend {
                    received: 3_000_000,
                    eligible,
                }),
            }],
        ));
        if eligible {
            let r = supported(&s.draft);
            assert_eq!(value(&r, "ira_deduction"), 7500);
            assert_eq!(value(&r, "earned_income_credit"), 0);
        } else {
            // Taxed without counting as compensation: the contribution is
            // an excess the estimate asks the year-end values for.
            unsupported(&s.draft, "IRA contributions");
        }
    }
}

#[test]
fn g_a_scholarship_row_is_missing_note_follows_the_1098_t_box_5() {
    // A filer's 1098-T box 5 ($9,000) above the tuition entered ($6,000)
    // with no scholarship row: one review note, offered with Other income.
    let mut d = complete(FederalStatus::Single);
    let sid0_1032 = -1; // You, the first person of the catalog
    set_education(
        d.credits.as_mut().unwrap(),
        vec![{
            let mut pair = native_education(
                sid0_1032,
                FederalEducationExpenses {
                    tuition: 600_000,
                    school_materials: 0,
                    other_materials: 0,
                    assistance: 900_000,
                    refunds: 0,
                    other_benefits: 0,
                    scholarship_in_income: crate::ScholarshipInclusion::None,
                },
                EducationChoice::AmericanOpportunity,
            );
            pair.0.id = 0;
            pair.0.half_time = true;
            pair.0.graduate = false;
            pair.1.more.american_opportunity_used_up = Some(false);
            pair
        }],
    );
    let mut s = Interview::new(TEST_USER, d);
    let r = supported(&s.draft);
    assert!(noted(&r, "NOTE_SCHOLARSHIP_MAY_BE_TAXABLE"), "{r:?}");
    let note =
        r.review_notes.iter().find(|n| n.display.key == "NOTE_SCHOLARSHIP_MAY_BE_TAXABLE").unwrap();
    assert_eq!(note.context, Some(crate::IssueContext::Section(TaxSection::OtherIncome)));
    // Entering the scholarship for that person clears it.
    accepted(&save_with(
        &mut s,
        TaxSection::OtherIncome,
        vec![IncomeSource::Record {
            value: crate::IncomeRecord::Scholarship(award(
                900_000,
                0,
                600_000,
                ScholarshipService::None,
            )),
        }],
    ));
    assert!(!noted(&supported(&s.draft), "NOTE_SCHOLARSHIP_MAY_BE_TAXABLE"));
    // Box 5 within the tuition raises no note.
    let mut d = complete(FederalStatus::Single);
    let sid0_1078 = -1; // You, the first person of the catalog
    set_education(
        d.credits.as_mut().unwrap(),
        vec![{
            let mut pair = native_education(
                sid0_1078,
                FederalEducationExpenses {
                    tuition: 600_000,
                    school_materials: 0,
                    other_materials: 0,
                    assistance: 600_000,
                    refunds: 0,
                    other_benefits: 0,
                    scholarship_in_income: crate::ScholarshipInclusion::None,
                },
                EducationChoice::AmericanOpportunity,
            );
            pair.0.id = 0;
            pair.0.half_time = true;
            pair.0.graduate = false;
            pair.1.more.american_opportunity_used_up = Some(false);
            pair
        }],
    );
    assert!(!noted(&supported(&d), "NOTE_SCHOLARSHIP_MAY_BE_TAXABLE"));
}

// ---- H: jury duty -----------------------------------------------------------

#[test]
fn h_jury_pay_is_income_and_the_remitted_part_is_an_adjustment() {
    // Received 1,000, 600 turned over to the employer: Schedule 1 line 8h
    // 1,000 and line 24a 600. AGI = 60,000 + 1,000 - 600 = 60,400; taxable
    // = 44,300; midpoint 44,325: 1,240 + 12% x 31,925 = 5,071. No
    // self-employment tax and no earned income.
    let mut s = single();
    accepted(&save_with(
        &mut s,
        TaxSection::OtherIncome,
        vec![IncomeSource::Record {
            value: crate::IncomeRecord::JuryPay(JuryPay { received: 100_000, remitted: 60_000 }),
        }],
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "jury_pay"), 1000);
    assert_eq!(value(&r, "jury_pay_remitted"), 600);
    assert_eq!(agi(&s), 6_040_000);
    assert_eq!(return_line(&r, "16"), 507_100);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    for (id, line) in [("jury_pay", "8h"), ("jury_pay_remitted", "24a")] {
        assert_eq!(
            r.lines.iter().find(|l| l.id == id).unwrap().form_line,
            Some(FormLine::new(TaxForm::Schedule1, line))
        );
    }
    assert_eq!(shown(&mut s, TaxSection::OtherIncome)[0].preview, Some(100_000));
    assert_round_trip(&s, TaxSection::OtherIncome);
    // Turning over more than was received is refused.
    let before = s.draft.clone();
    rejected_with(
        save_with(
            &mut s,
            TaxSection::OtherIncome,
            vec![IncomeSource::Record {
                value: crate::IncomeRecord::JuryPay(JuryPay {
                    received: 100_000,
                    remitted: 120_000,
                }),
            }],
        ),
        "JURY_REMITTED_ABOVE_RECEIVED",
    );
    assert_eq!(s.draft, before);
}

// ---- I: hobby sales ---------------------------------------------------------

fn hobby(receipts: i64, ordinary: bool) -> HobbySales {
    HobbySales {
        related: crate::RelatedForms::default(),
        receipts,
        inventory: BusinessInventory::Cost {
            method: crate::InventoryMethod::Cost,
            opening: 0,
            purchases: 800_000,
            personal: 0,
            labor: 0,
            materials: 0,
            other: 0,
            closing: 200_000,
            eligible: true,
        },
        ordinary,
    }
}

#[test]
fn i_hobby_sales_are_receipts_less_cost_of_goods_sold_without_a_loss() {
    // Receipts 10,000, purchases 8,000, closing inventory 2,000: cost of
    // goods sold 6,000, so Schedule 1 line 8j = 4,000.
    //   AGI = 64,000; taxable = 47,900; midpoint 47,925:
    //   1,240 + 12% x 35,525 = 5,503; no SE tax, no QBI.
    let mut s = single();
    accepted(&save_with(
        &mut s,
        TaxSection::OtherIncome,
        vec![IncomeSource::Record {
            value: crate::IncomeRecord::HobbySales(hobby(1_000_000, true)),
        }],
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "hobby_sales_income"), 4000);
    assert_eq!(
        r.lines.iter().find(|l| l.id == "hobby_sales_income").unwrap().form_line,
        Some(FormLine::new(TaxForm::Schedule1, "8j"))
    );
    assert_eq!(agi(&s), 6_400_000);
    assert_eq!(return_line(&r, "16"), 550_300);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(value(&r, "qbi_deduction"), 0);
    // The row previews the income, not the receipts.
    assert_eq!(shown(&mut s, TaxSection::OtherIncome)[0].preview, Some(400_000));
    assert_round_trip(&s, TaxSection::OtherIncome);
    // Receipts 1,000 against 6,000 of cost: no loss (floor zero).
    accepted(&save_only(
        &mut s,
        TaxSection::OtherIncome,
        vec![IncomeSource::Record { value: crate::IncomeRecord::HobbySales(hobby(100_000, true)) }],
    ));
    assert_eq!(agi(&s), 6_000_000);
    // Not an ordinary hobby: left out with a notice and previews nothing.
    accepted(&save_only(
        &mut s,
        TaxSection::OtherIncome,
        vec![IncomeSource::Record {
            value: crate::IncomeRecord::HobbySales(hobby(1_000_000, false)),
        }],
    ));
    let r = supported(&s.draft);
    assert!(noted(&r, "WARN_HOBBY_SALES_LEFT_OUT"));
    assert_eq!(agi(&s), 6_000_000);
    assert_eq!(shown(&mut s, TaxSection::OtherIncome)[0].preview, Some(0));
}

// ---- J: professional gambling -----------------------------------------------

fn gambling_business(
    winnings: i64,
    losses: i64,
    withholding: i64,
    travel: i64,
    eligible: bool,
) -> Business {
    let mut b = business(winnings);
    b.expenses.travel = travel;
    b.more = BusinessDetails {
        tips_amount_kinds: alloc::vec![1],
        overtime_amount_kinds: alloc::vec![1],
        ministerial: false,
        notary_profit: 0,
        notary_gross: None,
        gambling: Some(ProfessionalGambling { losses, withholding, eligible }),
        ..business_details()
    };
    b
}

#[test]
fn j_professional_gambling_is_a_business_record_with_the_90_percent_limit() {
    // No W-2. Line 1 gross winnings 100,000; losing wagers 90,000; travel
    // 10,000 on Part II; W-2G withholding 240. Section 165(d) as amended by
    // P.L. 119-21: the deduction is min(100,000, 90% x (90,000 + 10,000))
    // = 90,000, so net profit 10,000.
    //   SE tax = 10,000 x 92.35% x 15.3% = 1,413 (1,412.96); half 707
    //   AGI = 10,000 - 707 = 9,293; taxable 0; line 25c = 240
    let d = empty_return(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, d);
    accepted(&save_only(
        &mut s,
        TaxSection::SelfEmployment,
        vec![IncomeSource::Record {
            value: crate::IncomeRecord::Business(gambling_business(
                10_000_000, 9_000_000, 24_000, 1_000_000, true,
            )),
        }],
    ));
    assert!(matches!(
        s.draft.income[0].income,
        FederalIncome::Business {
            data: FederalBusiness {
                interest_limit_election: None,
                ministerial: false,
                notary_profit: 0,
                notary_gross: None,
                activity: BusinessActivity::Gambling { .. },
                ..
            }
        }
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "self_employment_tax"), 1413);
    assert_eq!(agi(&s), 929_300);
    assert_eq!(return_line(&r, "15"), 0);
    assert_eq!(return_line(&r, "25c"), 24_000);
    assert_round_trip(&s, TaxSection::SelfEmployment);
    // The business reopens as the gambling record, not as ordinary.
    let rows = shown(&mut s, TaxSection::SelfEmployment);
    let IncomeSource::Record { value: crate::IncomeRecord::Business(reopened) } = &rows[0].document
    else {
        panic!("expected the business")
    };
    assert_eq!(
        reopened.more.gambling,
        Some(ProfessionalGambling { losses: 9_000_000, withholding: 24_000, eligible: true })
    );
}

#[test]
fn j_professional_gambling_losses_never_make_a_schedule_c_loss() {
    // Losses 120,000: the deduction is min(100,000, 90% x 130,000 =
    // 117,000) = 100,000; net 0, no SE tax and no loss.
    let d = empty_return(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, d);
    accepted(&save_only(
        &mut s,
        TaxSection::SelfEmployment,
        vec![IncomeSource::Record {
            value: crate::IncomeRecord::Business(gambling_business(
                10_000_000, 12_000_000, 24_000, 1_000_000, true,
            )),
        }],
    ));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(agi(&s), 0);
}

#[test]
fn j_the_accounting_method_round_trips_and_a_not_eligible_business_is_left_out() {
    let d = empty_return(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, d);
    let mut accrual = gambling_business(10_000_000, 9_000_000, 24_000, 1_000_000, true);
    accrual.accounting = BusinessAccounting::Accrual;
    accepted(&save_only(
        &mut s,
        TaxSection::SelfEmployment,
        vec![IncomeSource::Record { value: crate::IncomeRecord::Business(accrual) }],
    ));
    let rows = shown(&mut s, TaxSection::SelfEmployment);
    let IncomeSource::Record { value: crate::IncomeRecord::Business(reopened) } = &rows[0].document
    else {
        panic!("expected the business")
    };
    assert_eq!(reopened.accounting, BusinessAccounting::Accrual);
    assert_round_trip(&s, TaxSection::SelfEmployment);
    // Eligible No: the business is left out with its notice.
    accepted(&save_only(
        &mut s,
        TaxSection::SelfEmployment,
        vec![IncomeSource::Record {
            value: crate::IncomeRecord::Business(gambling_business(
                10_000_000, 9_000_000, 24_000, 1_000_000, false,
            )),
        }],
    ));
    let r = supported(&s.draft);
    assert!(noted(&r, "WARN_GAMBLING_BUSINESS_LEFT_OUT"), "{r:?}");
    assert_eq!(agi(&s), 0);
}

#[test]
fn j_w2g_withholding_stays_within_the_winnings() {
    let d = empty_return(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, d);
    rejected_with(
        save_only(
            &mut s,
            TaxSection::SelfEmployment,
            vec![IncomeSource::Record {
                value: crate::IncomeRecord::Business(gambling_business(
                    10_000_000, 9_000_000, 12_000_000, 1_000_000, true,
                )),
            }],
        ),
        "GAMBLING_BUSINESS_WITHHOLDING_ABOVE_WINNINGS",
    );
    // Exactly the winnings is accepted.
    accepted(&save_only(
        &mut s,
        TaxSection::SelfEmployment,
        vec![IncomeSource::Record {
            value: crate::IncomeRecord::Business(gambling_business(
                10_000_000, 9_000_000, 10_000_000, 1_000_000, true,
            )),
        }],
    ));
}

#[test]
fn j_a_casual_w2g_left_out_sends_the_person_to_self_employment() {
    let model = crate::interview::resource_text("WARN_GAMBLING_LEFT_OUT");
    assert!(
        model.contains("enter a gambling business on Self-employment, or your tax may be higher.")
    );
    // The Business page's special question no longer names gambling.
    let federal = include_str!("../../business/schedule_c_model.rs");
    assert!(federal.contains("/// Timber depletion (Form T), a Form 7205 deduction or another unlisted special treatment applies"));
    assert!(!federal.contains("Professional gambling, a foreign business"));
}

// ---- K: disability pension in the 1099-R ------------------------------------

fn disability_1099r(gross: i64, wages: DisabilityWages, ira: bool) -> R1099Answered {
    r1099(gross, DistributionCode::Disability { began: 20240301, wages }, ira)
}
fn disabled_single() -> Interview {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.disabled = true;
    d.income.clear();
    Interview::new(TEST_USER, d)
}

#[test]
fn k_a_code_3_pension_before_minimum_retirement_age_is_wages_on_line_1h() {
    // 1099-R box 1 and 2a $30,000, code 3 (disabled 2024-03-01), all
    // payments before the plan's minimum retirement age: Form 1040 line 1h
    // = 30,000 and line 5b = 0. No W-2.
    //   AGI = 30,000; taxable = 30,000 - 16,100 = 13,900
    //   line 16: midpoint 13,925: 1,240 + 12% x 1,525 = 1,423
    let mut s = disabled_single();
    accepted(&save_only(
        &mut s,
        TaxSection::Retirement,
        vec![IncomeSource::Retirement {
            value: crate::RetirementDocument::R1099(retirement_document(disability_1099r(
                3_000_000,
                DisabilityWages::All,
                false,
            ))),
        }],
    ));
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "1h"), 3_000_000);
    assert_eq!(return_line(&r, "5a"), 0);
    assert_eq!(return_line(&r, "5b"), 0);
    assert_eq!(return_line(&r, "16"), 142_300);
    assert_round_trip(&s, TaxSection::Retirement);
    // The part reached during 2026: 18,000 through the day the age was
    // reached on line 1h, the other 12,000 on line 5b (and 5a); the tax is
    // the same 1,423.
    accepted(&save_only(
        &mut s,
        TaxSection::Retirement,
        vec![IncomeSource::Retirement {
            value: crate::RetirementDocument::R1099(retirement_document(disability_1099r(
                3_000_000,
                DisabilityWages::Part { amount: 1_800_000 },
                false,
            ))),
        }],
    ));
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "1h"), 1_800_000);
    assert_eq!(return_line(&r, "5a"), 1_200_000);
    assert_eq!(return_line(&r, "5b"), 1_200_000);
    assert_eq!(return_line(&r, "16"), 142_300);
    assert_round_trip(&s, TaxSection::Retirement);
    // The minimum age reached before 2026: pension income on line 5b (the
    // defect that put every code 3 pension there is fixed only for the
    // pre-minimum-age part), the same tax, and no earned income.
    accepted(&save_only(
        &mut s,
        TaxSection::Retirement,
        vec![IncomeSource::Retirement {
            value: crate::RetirementDocument::R1099(retirement_document(disability_1099r(
                3_000_000,
                DisabilityWages::None,
                false,
            ))),
        }],
    ));
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "1h"), 0);
    assert_eq!(return_line(&r, "5b"), 3_000_000);
    assert_eq!(return_line(&r, "16"), 142_300);
    assert_round_trip(&s, TaxSection::Retirement);
}

#[test]
fn k_the_wages_part_is_earned_income_but_not_ira_compensation() {
    // Pub. 596: a disability pension on line 1h is earned income for the
    // EIC until minimum retirement age; Pub. 590-A: a pension is not IRA
    // compensation. A $17,000 pension earns the same credit as $17,000 of
    // W-2 wages when it is wages and none when it is pension income; a
    // $2,000 IRA contribution beside it alone has no compensation.
    let build = |wages: DisabilityWages, ira: bool| {
        let mut d = complete(FederalStatus::Single);
        d.people.as_mut().unwrap().taxpayer.disabled = true;
        d.income.clear();
        if ira {
            d.adjustments.as_mut().unwrap().iras = ira_facts(200_000, 0, Answer::No);
        }
        let mut s = Interview::new(TEST_USER, d);
        crate::tests::support::client::admit_native_fixture_facts(&mut s);
        accepted(&save_only(
            &mut s,
            TaxSection::Retirement,
            vec![IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(disability_1099r(
                    1_700_000, wages, false,
                ))),
            }],
        ));
        s.draft
    };
    let eic = |d: &FederalDraft| value(&supported(d), "earned_income_credit");
    let wages_eic = eic(&build(DisabilityWages::All, false));
    assert!(wages_eic > 0, "{wages_eic}");
    assert_eq!(eic(&build(DisabilityWages::None, false)), 0);
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 1_700_000, 0);
    assert_eq!(eic(&d), wages_eic);
    unsupported(&build(DisabilityWages::All, true), "IRA contributions");
    unsupported(&build(DisabilityWages::None, true), "IRA contributions");
}

#[test]
fn k_the_wages_part_counts_as_earned_income_for_a_dependent_standard_deduction() {
    // A filer who can be claimed as a dependent: a standard deduction of
    // the earned income plus $450, up to the full amount (Pub. 501). Wages
    // of $30,000 give the full $16,100; a pension past the minimum age
    // gives only the unearned-income minimum.
    let deduction = |wages: DisabilityWages| {
        let mut d = complete(FederalStatus::Single);
        d.people.as_mut().unwrap().taxpayer.disabled = true;
        d.people.as_mut().unwrap().taxpayer.dependency =
            FilerDependency::CanBeClaimed { claimed: true };
        d.income.clear();
        let mut s = Interview::new(TEST_USER, d);
        accepted(&save_only(
            &mut s,
            TaxSection::Retirement,
            vec![IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(disability_1099r(
                    3_000_000, wages, false,
                ))),
            }],
        ));
        value(&supported(&s.draft), "deduction")
    };
    assert_eq!(deduction(DisabilityWages::All), 16100);
    assert!(deduction(DisabilityWages::None) < 16100);
}

#[test]
fn k_an_ira_carries_no_wages_and_the_part_stays_within_the_taxable_amount() {
    let mut s = disabled_single();
    let before = s.draft.clone();
    rejected_with(
        save_only(
            &mut s,
            TaxSection::Retirement,
            vec![IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(disability_1099r(
                    3_000_000,
                    DisabilityWages::All,
                    true,
                ))),
            }],
        ),
        "R_IRA_DISABILITY_NOT_WAGES",
    );
    rejected_with(
        save_only(
            &mut s,
            TaxSection::Retirement,
            vec![IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(disability_1099r(
                    3_000_000,
                    DisabilityWages::Part { amount: 3_100_000 },
                    false,
                ))),
            }],
        ),
        "R_DISABILITY_WAGES_ABOVE_TAXABLE",
    );
    assert_eq!(s.draft, before);
    // An IRA with None, and the part equal to the taxable amount, are accepted.
    accepted(&save_only(
        &mut s,
        TaxSection::Retirement,
        vec![IncomeSource::Retirement {
            value: crate::RetirementDocument::R1099(retirement_document(disability_1099r(
                3_000_000,
                DisabilityWages::None,
                true,
            ))),
        }],
    ));
    accepted(&save_only(
        &mut s,
        TaxSection::Retirement,
        vec![IncomeSource::Retirement {
            value: crate::RetirementDocument::R1099(retirement_document(disability_1099r(
                3_000_000,
                DisabilityWages::Part { amount: 3_000_000 },
                false,
            ))),
        }],
    ));
}

#[test]
fn k_the_separate_disability_pension_kind_is_gone() {
    let model = crate::federal_model::HUB_SOURCE;
    assert!(!model.contains("DisabilityPension"));
    assert!(!include_str!("../../core/unknowns.rs").contains("DisabilityPension"));
}

// ---- L: foreign services (the Form 2555 unit's boundary) ---------------------

#[test]
fn l_foreign_services_boundary_figures_for_the_form_2555_unit() {
    // A U.S. citizen consulting wholly in France, present all of 2026, no
    // capital: Schedule C receipts 100,000, expenses 20,000, net 80,000.
    //   SE = 80,000 x 92.35% x 15.3% = 11,303.64 -> 11,304; half 5,652
    //   Form 2555 line 20a = 100,000 (services only); the limit 132,900
    //   line 44 = 20,000 + 5,652 = 25,652; line 45 = 100,000 - 25,652 =
    //   74,348 -> Schedule 1 line 8d
    //   AGI = 80,000 - 5,652 - 74,348 = 0; income tax 0; total tax = SE
    // This unit does not change the stored BusinessSource (the Form 2555
    // unit owns it); the figures are the agreed boundary case.
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(excluded_foreign_business(100_000, 20_000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 11304);
    assert_eq!(value(&r, "foreign_earned_exclusion"), 100_000);
    assert_eq!(value(&r, "foreign_business_taxpayer_disallowed_deductions"), 25_652);
    assert_eq!(value(&r, "foreign_business_taxpayer_net_exclusion"), 74_348);
    assert_eq!(value(&r, "agi"), 0);
    assert_eq!(r.tax, Some(1_130_400));
}

// ---- Less common income and the page text -------------------------------------

#[test]
fn less_common_income_no_longer_names_relief_payments_or_the_nine_kinds() {
    assert!(
        Unprepared::from(RareSituation::ForeignCorporation)
            .refused()
            .lines
            .contains("Schedule 1 lines 8n and 8o")
    );
    let model = crate::federal_model::HUB_SOURCE;
    assert!(!model.contains("ReliefPayments"));
    // Other income that no page owns became the Other income page's own kind.
    assert!(!model.contains("/// Other income not listed elsewhere (Schedule 1 line 8z)"));
}

#[test]
fn the_topics_offer_the_new_kinds_and_east_palestine_is_page_text_only() {
    let kinds = |topic: TaxSection| {
        let mut s = single();
        let allowed_documents = match page(&mut s, topic) {
            P::SaveIncomes { allowed_documents, .. } => allowed_documents,
            // The Wages topic's record kinds are its typed Union; printed
            // wage forms are offered by the Document center below.
            P::SaveWagesAnswers { .. } => Vec::new(),
            _ => unreachable!(),
        };
        let P::SaveDocuments { allowed_documents: source_kinds, .. } = open_documents(&mut s)
        else {
            unreachable!()
        };
        let mut kinds = allowed_documents;
        kinds.extend(source_kinds);
        kinds
    };
    let index = |name: i32| name;
    let wages = kinds(TaxSection::Wages);
    assert!(wages.contains(&index(
        libertas_macros::variant_index!(crate::IncomeSourceKind::Form8919) as i32
    )));
    let debt = kinds(TaxSection::CanceledDebt);
    assert!(debt.contains(
        &(libertas_macros::variant_index!(crate::IncomeSourceKind::HealthLoanRepayment) as i32)
    ));
    let other = kinds(TaxSection::OtherIncome);
    for kind in [
        libertas_macros::variant_index!(crate::IncomeSourceKind::Scholarship) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::JuryPay) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::HobbySales) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::WildfireRelief) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::DisasterRelief) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::CasualtyLivingInsurance) as i32,
        libertas_macros::variant_index!(crate::IncomeSourceKind::DisasterMitigation) as i32,
    ] {
        assert!(other.contains(&kind), "{kind}");
    }
    // No kind for the East Palestine payments: the disaster relief text says
    // they need not be entered.
    let model = include_str!("../../federal/income_editor.rs");
    assert!(model.contains(
        "East Palestine train derailment payments are not taxable and need not be entered."
    ));
    assert!(!model.contains("EastPalestine"));
}

#[test]
fn actual_state_local_coverage_calculates_each_independent_employee_tax() {
    use crate::{EmployeeCoverage as C, EmployeePayrollSource as S};
    // IRC3101 imposes SS and Medicare independently. $5,000: $310 SS,
    // $72.50 -> $73 Medicare; ordinary income tax on $65,000 is $5,623.
    for (social, medicare, ss_tax, med_tax) in [
        (C::Covered, C::Covered, 310, 73),
        (C::Covered, C::NotCovered, 310, 0),
        (C::NotCovered, C::Covered, 0, 73),
        (C::NotCovered, C::NotCovered, 0, 0),
    ] {
        let mut d = complete(FederalStatus::Single);
        let FederalIncome::EmployeeWages { mut data } = omitted_employee_wages(5_000) else {
            unreachable!()
        };
        data.payroll = S::Government { social, medicare, reason: None };
        add(&mut d, income(FederalIncome::EmployeeWages { data }));
        let r = estimate(&d);
        assert_eq!(
            r.state,
            ResultState::EstimatedForSupportedScope,
            "{social:?}/{medicare:?}: {:?}",
            r.issues
        );
        assert_eq!(value(&r, "employee_coverage_0_social_tax"), ss_tax);
        assert_eq!(value(&r, "employee_coverage_0_medicare_tax"), med_tax);
        assert_eq!(r.tax, Some((5_623 + ss_tax + med_tax) * 100));
        assert_eq!(return_line(&r, "1h"), 500_000);
        assert_eq!(return_line(&r, "1g"), 0);
        assert!(!r.lines.iter().any(|line| {
            line.form_line.as_ref().is_some_and(|form| form.form == TaxForm::Form8919)
        }));
        assert_eq!(estimate(&restart(d.clone()).draft), r);
    }
}

#[test]
fn government_both_with_real_reason_uses_existing_8919_and_common_ss_base_once() {
    use crate::{EmployeeCoverage as C, EmployeePayrollSource as S};
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 18_000_000, 0);
    let mut ordinary = pay(300_000, code_a(), true);
    ordinary.payroll =
        S::Government { social: C::Covered, medicare: C::Covered, reason: Some(code_a()) };
    add(&mut d, income(FederalIncome::EmployeeWages { data: ordinary }));
    let mut partial = pay(500_000, code_a(), false);
    partial.payroll = S::Government { social: C::Covered, medicare: C::NotCovered, reason: None };
    add(&mut d, income(FederalIncome::EmployeeWages { data: partial }));
    let r = estimate(&d);
    // Form8919 first: $3,000 at6.2%=$186,Medicare=$44.
    // Other actualSS pay then consumes only the remaining$1,500: $93.
    assert_eq!(value(&r, "employee_wages_0_social_tax"), 186);
    assert_eq!(value(&r, "employee_wages_0_medicare_tax"), 44);
    assert_eq!(value(&r, "employee_coverage_0_social_tax"), 93);
    assert_eq!(value(&r, "employee_coverage_0_medicare_tax"), 0);
    assert_eq!(return_line(&r, "1g"), 300_000);
    assert_eq!(return_line(&r, "1h"), 500_000);
    assert_eq!(value(&r, "schedule_2_uncollected_wage_tax"), 230);
    assert!(
        r.lines
            .iter()
            .find(|line| line.id == "schedule_2_total_additional_taxes")
            .unwrap()
            .form_line
            .is_none()
    );
}

#[test]
fn independent_unknown_coverage_preserves_known_medicare_and_saturated_ss_endpoints() {
    use crate::{EmployeeCoverage as C, EmployeePayrollSource as S};
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 20_000_000, 0);
    if let FederalIncome::Wage { data } = &mut d.income[0].income {
        let mut data = edit_wage(data);
        data.social_wages = 10_000_000;
        data.social_tax = 620_000;
    }
    let mut employee = pay(1_000_000, code_a(), false);
    employee.payroll = S::Government { social: C::Unavailable, medicare: C::Covered, reason: None };
    add(&mut d, income(FederalIncome::EmployeeWages { data: employee }));
    let r = estimate(&d);
    assert_eq!(r.tax, None);
    assert_eq!(r.state, ResultState::Incomplete);
    assert_eq!(value(&r, "employee_coverage_0_medicare_tax"), 145);
    assert_eq!(value(&r, "additional_medicare_employee_wages"), 0);
    assert_eq!(value(&r, "additional_medicare_other_employee_wages"), 10_000);
    assert_eq!(value(&r, "additional_medicare_payroll_wages"), 90);
    let source = d.income.last().unwrap();
    assert!(
        r.issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_EMPLOYEE_PAYROLL_COVERAGE_UNAVAILABLE"
                && issue.context == crate::interview::income_issue_context(source))
    );
    // The same missing SS answer is immaterial once reportedSS reaches itscap.
    if let FederalIncome::Wage { data } = &mut d.income[0].income {
        let mut data = edit_wage(data);
        data.social_wages = 18_450_000;
        data.social_tax = 1_143_900;
    }
    let complete = estimate(&d);
    assert_eq!(complete.state, ResultState::EstimatedForSupportedScope, "{:?}", complete.issues);
    assert_eq!(value(&complete, "employee_coverage_0_medicare_tax"), 145);
    assert_eq!(value(&complete, "additional_medicare_payroll_wages"), 90);
    // Zero actual pay requires no missing payroll answer.
    if let FederalIncome::EmployeeWages { data } = &mut d.income.last_mut().unwrap().income {
        data.wages = 0;
        data.payroll = S::Unavailable;
    }
    assert_eq!(estimate(&d).state, ResultState::EstimatedForSupportedScope);
}

#[test]
fn actual_totalization_certificate_differs_from_foreign_employment_label() {
    use crate::EmployeePayrollSource as S;
    let mut d = complete(FederalStatus::Single);
    let mut employee = pay(500_000, code_a(), false);
    employee.payroll = S::TotalizationCertificate;
    add(&mut d, income(FederalIncome::EmployeeWages { data: employee }));
    let exempt = estimate(&d);
    assert_eq!(exempt.state, ResultState::EstimatedForSupportedScope);
    assert_eq!(exempt.tax, Some(562_300));
    assert_eq!(return_line(&exempt, "1h"), 500_000);
    if let FederalIncome::EmployeeWages { data } = &mut d.income.last_mut().unwrap().income {
        data.payroll = S::ForeignGovernment;
    }
    let foreign = estimate(&d);
    assert_eq!(foreign.tax, None);
    assert_eq!(foreign.state, ResultState::Unsupported);
    assert!(!foreign.lines.iter().any(|line| line.id == "agi"));
    assert!(
        foreign
            .issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_EMPLOYEE_FOREIGN_GOVERNMENT_PAYROLL")
    );
}

#[test]
fn actual_rrta_uses_independent_8959_threshold_actual_withholding_and_ss_coordination() {
    use crate::EmployeePayrollSource as S;
    for representative in [false, true] {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, 21_000_000, 0);
        let mut employee = pay(if representative { 0 } else { 21_000_000 }, code_a(), false);
        let reported = employee_railroad_source(
            [21_000_000, 1_143_900, 0, 304_500, 12_300]
                .map(|amount| SourceCents::Reported { amount }),
        );
        employee.payroll = if representative {
            S::RailroadRepresentative { reported }
        } else {
            S::Railroad { reported }
        };
        add(&mut d, income(FederalIncome::EmployeeWages { data: employee }));
        add(&mut d, income(omitted_employee_wages(5_000)));
        let r = estimate(&d);
        // Separate $200,000 thresholds: ordinary$215,000 ->$135;
        // RRTA$210,000 ->$90, actual Additional withholding$123.
        assert_eq!(value(&r, "additional_medicare_payroll_wages"), 135);
        assert_eq!(value(&r, "additional_medicare_rrta_tax"), 90);
        assert_eq!(value(&r, "additional_medicare_rrta_withheld"), 123);
        // Both actual payroll sources consume the physical employee's base.
        form8919_base(&r, 0, 369_000, 0);
        assert_eq!(value(&r, "employee_wages_0_social_base"), 0);
        assert_eq!(value(&r, "employee_wages_0_social_tax"), 0);
        assert_eq!(value(&r, "employee_wages_0_medicare_tax"), 73);
        assert_eq!(r.tax, None);
        assert_eq!(r.state, ResultState::Unsupported);
        if representative {
            assert!(!r.lines.iter().any(|line| line.id == "agi"));
        }
        assert!(
            r.issues
                .iter()
                .any(|issue| issue.display.key == "ISSUE_EMPLOYEE_RAILROAD_ORDINARY_TAX")
        );
        assert_eq!(estimate(&restart(d.clone()).draft), r);

        // Missing RRTA compensation withholds dependent bases, while the
        // annual statutory wage base and ordinary Medicare remain known.
        let FederalIncome::EmployeeWages { data } = &mut d.income[1].income else { unreachable!() };
        let (S::Railroad { reported } | S::RailroadRepresentative { reported }) = &mut data.payroll
        else {
            unreachable!()
        };
        reported.compensation = SourceCents::Unavailable;
        let unknown = estimate(&d);
        assert_eq!(value(&unknown, "employee_wages_0_wage_base"), 184_500);
        assert_eq!(
            printed_line(&unknown, "employee_wages_0_wage_base"),
            (TaxForm::Form8919, "7".into())
        );
        assert_eq!(value(&unknown, "employee_wages_0_medicare_tax"), 73);
        assert_eq!(
            crate::tests::support::assert::displayed_line_amount(
                &unknown,
                "employee_wages_0_wage_base"
            ),
            18_450_000
        );
        assert!(!crate::draft_return(&unknown).iter().any(|row| {
            row.form_line.as_ref().is_some_and(|line| {
                line.form == TaxForm::Form8919 && matches!(line.line.as_str(), "8" | "9")
            })
        }));
        for suffix in ["used_base", "remaining_base", "social_base", "social_tax", "tax"] {
            let id = alloc::format!("employee_wages_0_{suffix}");
            assert!(!unknown.lines.iter().any(|row| row.id == id), "{id}");
        }
        assert_eq!(unknown.tax, None);
        assert_eq!(estimate(&restart(d).draft), unknown);
    }
}

#[test]
fn actual_payroll_physical_owner_survives_community_income_sharing_and_spouse_swap() {
    use crate::{EmployeeCoverage as C, EmployeePayrollSource as S};
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut d = complete(FederalStatus::Separate);
        d.setup.as_mut().unwrap().community_property = CommunityProperty::Yes {
            state: CommunityState::California,
            rule: CommunityRule::Halves,
        };
        d.income.clear();
        let mut employee = pay(1_000_000, code_a(), false);
        employee.payroll = S::Government { social: C::Covered, medicare: C::Covered, reason: None };
        let mut source = income(FederalIncome::EmployeeWages { data: employee });
        source.owner = crate::DocumentOwner::from(owner);
        source.community = CommunityShare::Community;
        add(&mut d, source);
        let r = estimate(&d);
        assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{:?}", r.issues);
        assert_eq!(return_line(&r, "1h"), 500_000);
        assert_eq!(
            value(&r, "employee_coverage_0_tax"),
            if owner == Owner::Taxpayer { 765 } else { 0 }
        );
        assert_eq!(value(&r, "employee_coverage_1_tax"), 0);
        assert_eq!(estimate(&restart(d.clone()).draft), r);
    }
}

#[test]
fn actual_payroll_source_native_save_reopen_keyed_restart_and_located_correction() {
    use crate::{EmployeeCoverage as C, EmployeePayrollSource as S};
    let mut s = single();
    let mut employee = pay(500_000, code_a(), false);
    employee.payroll = S::Government { social: C::NotCovered, medicare: C::Covered, reason: None };
    accepted(&save_with(
        &mut s,
        TaxSection::Wages,
        vec![IncomeSource::Wage { value: crate::WageDocument::Form8919(employee) }],
    ));
    let saved = s.draft.clone();
    let id = saved.income.last().unwrap().id;
    assert_eq!(estimate(&saved).tax, Some(569_600));
    s = restart(saved.clone()); // shared client performs actual keyed record encoding/decoding.
    assert_eq!(s.draft, saved);
    let mut request = open_documents(&mut s);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let item = documents.common.wages.documents.iter_mut().find(|item| item.id == id).unwrap();
    let crate::WageDocument::Form8919(employee) = &mut item.document else { unreachable!() };
    employee.payroll = S::Government { social: C::Covered, medicare: C::NotCovered, reason: None };
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.income.last().unwrap().id, id);
    assert_eq!(estimate(&s.draft).tax, Some(593_300));
    assert_round_trip(&s, TaxSection::Wages);
}

#[test]
fn unknown_medicare_compares_final_8959_self_employment_offset() {
    use crate::{EmployeeCoverage as C, EmployeePayrollSource as S};
    // IRS8959 single threshold$200,000; actual SE net rounds565*.9235=$522.
    // UnknownMed$34 ordinarytax rounds.493 to0. WagePartI0 in bothcases,
    // but SE remainingthreshold500 vs466 makes22*.009=0 vs56*.009=1.
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 19_950_000, 0);
    add(&mut d, income(cash_business(565)));
    let mut employee = pay(3_400, code_a(), false);
    employee.payroll =
        S::Government { social: C::NotCovered, medicare: C::Unavailable, reason: None };
    add(&mut d, income(FederalIncome::EmployeeWages { data: employee }));
    let unknown = estimate(&d);
    assert_eq!(unknown.tax, None);
    assert_eq!(unknown.state, ResultState::Incomplete);
    assert_eq!(value(&unknown, "agi"), 200_091);
    assert!(!unknown.lines.iter().any(|line| line.id == "regular_income_tax"));
    assert_eq!(value(&unknown, "employee_coverage_0_medicare_tax"), 0);
    assert!(
        unknown
            .issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_EMPLOYEE_PAYROLL_COVERAGE_UNAVAILABLE")
    );
    for (medicare, tax) in [(C::NotCovered, 0), (C::Covered, 1)] {
        if let FederalIncome::EmployeeWages { data } = &mut d.income.last_mut().unwrap().income {
            data.payroll = S::Government { social: C::NotCovered, medicare, reason: None };
        }
        let known = estimate(&d);
        assert_eq!(known.state, ResultState::EstimatedForSupportedScope, "{:?}", known.issues);
        assert_eq!(value(&known, "additional_medicare_payroll_wages"), 0);
        assert_eq!(value(&known, "additional_medicare_se_tax"), tax);
        assert_eq!(value(&known, "employee_coverage_0_medicare_tax"), 0);
    }
}

#[test]
fn unknown_social_security_compares_actual_church_self_employment_owner() {
    use crate::{EmployeeCoverage as C, EmployeePayrollSource as S};
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 18_400_000, 0);
    let mut church = wage(2, Owner::Taxpayer, 56_500, 0);
    if let FederalIncome::Wage { data } = &mut church.income {
        let mut data = edit_wage(data);
        data.coverage = W2Coverage::ChurchEmployee;
        data.social_wages = 0;
        data.social_tax = 0;
        data.medicare_wages = 0;
        data.medicare_tax = 0;
    }
    add(&mut d, church);
    let mut employee = pay(800, code_a(), false);
    employee.payroll =
        S::Government { social: C::Unavailable, medicare: C::NotCovered, reason: None };
    add(&mut d, income(FederalIncome::EmployeeWages { data: employee }));
    let unknown = estimate(&d);
    assert_eq!(unknown.tax, None);
    assert_eq!(unknown.state, ResultState::Incomplete);
    assert!(!unknown.lines.iter().any(|line| line.id == "agi"));
    assert!(!unknown.lines.iter().any(|line| line.id == "regular_income_tax"));
    assert_eq!(value(&unknown, "employee_coverage_0_social_tax"), 0);
    assert!(
        unknown
            .issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_EMPLOYEE_PAYROLL_COVERAGE_UNAVAILABLE")
    );
    // Actual church565 yieldsnet522: SSremaining500→tax77; remaining492→76.
    for (social, tax) in [(C::NotCovered, 77), (C::Covered, 76)] {
        if let FederalIncome::EmployeeWages { data } = &mut d.income.last_mut().unwrap().income {
            data.payroll = S::Government { social, medicare: C::NotCovered, reason: None };
        }
        let known = estimate(&d);
        assert_eq!(known.state, ResultState::EstimatedForSupportedScope, "{:?}", known.issues);
        assert_eq!(value(&known, "self_employment_tax"), tax);
        assert_eq!(value(&known, "employee_coverage_0_social_tax"), 0);
    }
}

#[test]
fn railroad_source_components_stay_independently_available() {
    use crate::EmployeePayrollSource as S;
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 21_000_000, 0);
    let mut employee = pay(21_000_000, code_a(), false);
    employee.payroll = S::Railroad {
        reported: employee_railroad_source([
            SourceCents::Reported { amount: 21_000_000 },
            SourceCents::Unavailable,
            SourceCents::NotReported,
            SourceCents::Unavailable,
            SourceCents::Unavailable,
        ]),
    };
    add(&mut d, income(FederalIncome::EmployeeWages { data: employee }));
    let known_compensation = estimate(&d);
    assert_eq!(value(&known_compensation, "additional_medicare_payroll_wages"), 90);
    assert_eq!(value(&known_compensation, "additional_medicare_rrta_tax"), 90);
    assert!(noted(&known_compensation, "WARN_EMPLOYEE_RAILROAD_WITHHOLDING_UNAVAILABLE"));
    let FederalIncome::EmployeeWages { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    let S::Railroad { reported } = &mut data.payroll else { unreachable!() };
    reported.compensation = SourceCents::Unavailable;
    reported.additional_medicare = SourceCents::Reported { amount: 12_300 };
    let unknown_compensation = estimate(&d);
    assert_eq!(value(&unknown_compensation, "additional_medicare_payroll_wages"), 90);
    assert_eq!(value(&unknown_compensation, "additional_medicare_rrta_withheld"), 123);
    assert!(
        unknown_compensation
            .issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_EMPLOYEE_RRTA_COMPENSATION_UNAVAILABLE")
    );
    assert_eq!(unknown_compensation.tax, None);
    assert_eq!(estimate(&restart(d.clone()).draft), unknown_compensation);
}

#[test]
fn unknown_coverage_compares_combined_native_component_rounding_boundaries() {
    use crate::{EmployeeCoverage as C, EmployeePayrollSource as S};
    // Neither missing operand alone rounds to tax. Full same-owner bases do:
    // SS(4+5)*.062=.558→1; Medicare(1+34)*.0145=.5075→1.
    for (social_axis, known_pay, unknown_pay, component) in [
        (true, 400, 500, "employee_coverage_0_social_tax"),
        (false, 100, 3_400, "employee_coverage_0_medicare_tax"),
    ] {
        let mut d = complete(FederalStatus::Single);
        for (wages, coverage) in [(known_pay, C::Covered), (unknown_pay, C::Unavailable)] {
            let mut employee = pay(wages, code_a(), false);
            employee.payroll = S::Government {
                social: if social_axis { coverage } else { C::NotCovered },
                medicare: if social_axis { C::NotCovered } else { coverage },
                reason: None,
            };
            add(&mut d, income(FederalIncome::EmployeeWages { data: employee }));
        }
        let unknown = estimate(&d);
        assert_eq!(unknown.state, ResultState::Incomplete);
        assert_eq!(unknown.tax, None);
        assert!(!unknown.lines.iter().any(|line| line.id == component));
        assert!(
            unknown
                .issues
                .iter()
                .any(|issue| issue.display.key == "ISSUE_EMPLOYEE_PAYROLL_COVERAGE_UNAVAILABLE")
        );
        for (coverage, expected) in [(C::NotCovered, 0), (C::Covered, 1)] {
            if let FederalIncome::EmployeeWages { data } = &mut d.income.last_mut().unwrap().income
            {
                data.payroll = S::Government {
                    social: if social_axis { coverage } else { C::NotCovered },
                    medicare: if social_axis { C::NotCovered } else { coverage },
                    reason: None,
                };
            }
            let known = estimate(&d);
            assert_eq!(known.state, ResultState::EstimatedForSupportedScope);
            assert_eq!(value(&known, component), expected);
        }
    }
}

#[test]
fn other_spouse_rrta_unknown_withholding_does_not_remove_this_filers_capacity() {
    use crate::EmployeePayrollSource as S;
    let mut d = complete(FederalStatus::Separate);
    d.setup.as_mut().unwrap().community_property =
        CommunityProperty::Yes { state: CommunityState::California, rule: CommunityRule::Halves };
    for entry in &mut d.income {
        entry.community = CommunityShare::Community;
    }
    let mut employee = pay(1_000_000, code_a(), false);
    employee.payroll = S::Railroad {
        reported: employee_railroad_source([
            SourceCents::Reported { amount: 21_000_000 },
            SourceCents::Unavailable,
            SourceCents::Unavailable,
            SourceCents::Unavailable,
            SourceCents::Unavailable,
        ]),
    };
    let mut source = income(FederalIncome::EmployeeWages { data: employee });
    source.owner = crate::DocumentOwner::Spouses;
    source.community = CommunityShare::Community;
    add(&mut d, source);
    let result = estimate(&d);
    assert_eq!(result.state, ResultState::EstimatedForSupportedScope, "{:?}", result.issues);
    assert!(result.financial_source_scope_complete);
    assert!(result.credit_capacity.is_some());
    assert!(!result.issues.iter().any(|issue| issue.display.key.starts_with("ISSUE_EMPLOYEE_")));
    assert_eq!(value(&result, "employee_coverage_0_medicare_tax"), 0);
}

#[test]
fn payroll_partial_does_not_present_provisional_nol_agi() {
    use crate::{EmployeeCoverage as C, EmployeePayrollSource as S};
    let mut d = complete(FederalStatus::Single);
    let mut employee = pay(500_000, code_a(), false);
    employee.payroll = S::Government { social: C::Unavailable, medicare: C::Covered, reason: None };
    add(&mut d, income(FederalIncome::EmployeeWages { data: employee }));
    let ordinary = estimate(&d);
    assert_eq!(value(&ordinary, "agi"), 65_000);
    d.screening.as_mut().unwrap().nol =
        crate::tests::support::builders::prior_nol(&[(2025, 1_000)]);
    let provisional = estimate(&d);
    assert_eq!(provisional.state, ResultState::Incomplete);
    assert!(!provisional.lines.iter().any(|line| line.id == "agi"));
    assert_eq!(value(&provisional, "employee_coverage_0_medicare_tax"), 73);
    assert!(provisional.tax.is_none());
    assert!(provisional.return_lines.is_empty());
}

#[test]
fn unrelated_mandatory_failure_keeps_full_required_source_cleanup() {
    use crate::{EmployeeCoverage as C, EmployeePayrollSource as S};
    let mut d = complete(FederalStatus::Single);
    let mut employee = pay(500_000, code_a(), false);
    employee.payroll = S::Government { social: C::Unavailable, medicare: C::Covered, reason: None };
    add(&mut d, income(FederalIncome::EmployeeWages { data: employee }));
    let scoped = estimate(&d);
    assert_eq!(scoped.state, ResultState::Incomplete);
    assert_eq!(value(&scoped, "employee_coverage_0_medicare_tax"), 73);
    let mut bond_failure = d.clone();
    let mut bond = crate::tests::support::builders::direct_tax_credit_bond(
        "PARTIAL",
        crate::FederalTaxCreditBondKind::QualifiedSchoolConstruction,
        10_000,
    );
    bond.direct_scope_mut().unwrap().ordinary_investment_interest = false;
    bond_failure.credits.as_mut().unwrap().tax_credit_bonds.push(bond);
    if let FederalIncome::Wage { data } = &mut d.income[0].income {
        let mut data = edit_wage(data);
        data.medicare_tax = 0;
    }
    for (draft, key) in [
        (d, "ISSUE_MEDICARE_WITHHOLDING_BELOW_ORDINARY_PAYROLL_TAX_REQUIRES_SPECIAL_PA_3993480B"),
        (bond_failure, "ISSUE_TAX_CREDIT_BOND_INCOME_UNKNOWN"),
    ] {
        let unrelated = estimate(&draft);
        assert_eq!(unrelated.state, ResultState::Unsupported);
        assert_eq!(unrelated.tax, None);
        assert!(
            !unrelated
                .lines
                .iter()
                .any(|line| line.id == "employee_coverage_0_medicare_tax" || line.id == "agi")
        );
        assert!(unrelated.return_lines.is_empty());
        assert!(unrelated.issues.iter().any(|issue| issue.display.key == key));
    }
}
