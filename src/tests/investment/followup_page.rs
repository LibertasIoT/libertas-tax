// The income topic page as the person sees it (shape
// followup-questions-shape-2026-10-31): questions a printed box calls for are
// required per paper, rare situations are asked once per page, and a page with
// an unanswered question opens through BeginIncomes.
use super::*;
use crate::tests::cross::interview::{accepted, document, rejected_with};
use crate::tests::support::client::income_pages::{open_topic, save_documents};
use crate::{
    DocumentSituation, DocumentSituationAnswer as S, DocumentSituations, DocumentStrictAnswer as T,
    DocumentStrictQuestion,
};

// The request and reply as the endpoint receives and sends them.
fn wire(s: &mut Interview, request: P) -> Reply {
    let (request, targets) = match s.wire_request(request) {
        Ok(request) => request,
        Err(reply) => return *reply,
    };
    let reply = send(s, request);
    s.wire_reply(reply, targets.as_ref())
}

fn wire_page(s: &mut Interview) -> Reply {
    let page = open_topic(s, TaxSection::InterestDividends);
    s.wire_reply(Reply::Edit(page), None)
}

fn fund(name: &str) -> IncomeSource {
    IncomeSource::InterestDividend {
        value: crate::InterestDividendDocument::Div1099(crate::Form1099DivPaper {
            calendar_year: Some(2026),
            capital_distributions: Some(10_000),
            section_1202_gain: Some(2_000),
            section199a: Some(20_000),
            ..div1099_source(native_div1099(name.into(), 100_000, 0)).form
        }),
    }
}

fn bank(name: &str) -> IncomeSource {
    IncomeSource::InterestDividend {
        value: crate::InterestDividendDocument::Int1099(interest_document(native_int1099(
            name.into(),
            21_000,
        ))),
    }
}

fn question(key: &str, answer: T) -> DocumentStrictQuestion {
    DocumentStrictQuestion {
        key: key.into(),
        prompt: Vec::new(),
        allowed_answers: Vec::new(),
        allowed_values: Vec::new(),
        ccc_lots: Vec::new(),
        health_plans: Vec::new(),
        espp_acquisitions: Vec::new(),
        interest_statements: Vec::new(),
        people: Vec::new(),
        credit_income_sources: Vec::new(),
        answer,
    }
}

// The client's questions page, built from BeginDocumentQuestions, with the
// box-triggered answers and the situations answer.
fn answered(begin: &P, fund: i64, situations: Vec<DocumentSituation>) -> P {
    let P::BeginDocumentQuestions {
        cookie,
        revision,
        page,
        previous,
        review,
        progress,
        topic,
        allowed_situations,
        situation_answers,
        situation_documents,
        disaster_choices,
        inherited_iras,
        businesses,
        people,
        inherited_roth_iras,
        wage_scorporations,
        ..
    } = begin.clone()
    else {
        panic!("the questions page opens through BeginDocumentQuestions")
    };
    P::SaveDocumentQuestions {
        user: TEST_USER,
        cookie,
        revision,
        page,
        previous,
        review,
        estimate: None,
        progress,
        topic,
        questions: vec![
            question(
                &alloc::format!("income/{fund}/reit"),
                T::DividendEligibility { value: crate::DividendEligibility::Yes },
            ),
            question(
                &alloc::format!("income/{fund}/exclusion"),
                T::DividendFundExclusion { value: crate::FundQsbsExclusion::None },
            ),
        ],
        situations: DocumentSituations::These { situations },
        allowed_situations,
        situation_answers,
        situation_documents,
        disaster_choices,
        inherited_iras,
        businesses,
        people,
        inherited_roth_iras,
        wage_scorporations,
    }
}

fn ids(s: &Interview) -> (i64, i64) {
    let id = |kind: fn(&FederalIncome) -> bool| {
        s.draft.income.iter().find(|entry| kind(&entry.income)).unwrap().id
    };
    (
        id(|income| matches!(income, FederalIncome::Dividend { .. })),
        id(|income| matches!(income, FederalIncome::Interest { .. })),
    )
}

#[test]
fn income_page_asks_box_questions_per_paper_and_rare_situations_once() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = save_documents(
        &mut s,
        vec![document(0, fund("Example Orchard Fund")), document(0, bank("Example Bank"))],
    );
    accepted(&send(&mut s, request));
    let (fund_id, bank_id) = ids(&s);

    // First visit: boxes 5 and 2c ask their questions unanswered, and the
    // situations question starts unanswered; nothing else is asked.
    let begin = match wire_page(&mut s) {
        Reply::Response(begin @ P::BeginDocumentQuestions { .. }) => begin,
        other => panic!("a page with unanswered questions asks them first, got {other:?}"),
    };
    let P::BeginDocumentQuestions {
        questions,
        situations,
        allowed_situations,
        situation_documents,
        ..
    } = &begin
    else {
        unreachable!()
    };
    let keys: Vec<_> = questions.iter().map(|q| q.key.clone()).collect();
    assert_eq!(
        keys,
        [alloc::format!("income/{fund_id}/reit"), alloc::format!("income/{fund_id}/exclusion")]
    );
    assert!(situations.is_empty(), "the situations question is not answered yet");
    assert_eq!(situation_documents.len(), 2);
    use libertas_macros::variant_index as v;
    let mut expected: Vec<i32> = [
        v!(S::InterestNominee),
        v!(S::DividendNominee),
        v!(S::ReportingAdjustments),
        v!(S::AccruedInterest),
        v!(S::InterestTreaty),
        v!(S::ForeignDividendTreatment),
        v!(S::AnotherTreatment),
    ]
    .into_iter()
    .map(|i| i as i32)
    .collect();
    expected.sort_unstable();
    assert_eq!(*allowed_situations, expected);
    assert!(crate::interview::situations_pending(&s));

    // A situation the paper cannot have is refused on the page's own field.
    let wrong = vec![DocumentSituation { situation: S::InterestTreaty, document: fund_id }];
    let reply = wire(&mut s, answered(&begin, fund_id, wrong));
    let Reply::Response(P::Problem { field, .. }) = &reply else { panic!("{reply:?}") };
    assert_eq!(field.as_deref(), Some("[situations/0/document]"));
    rejected_with(reply, "DOCUMENT_SITUATION_NOT_ON_DOCUMENT");

    // One situation on the bank's 1099-INT; None of these for every other. The
    // reply continues on the topic page, which asks nothing more.
    let treaty = vec![DocumentSituation { situation: S::InterestTreaty, document: bank_id }];
    let reply = wire(&mut s, answered(&begin, fund_id, treaty.clone()));
    let Reply::Edit(P::SaveIncomes { questions, situations, followups, .. }) = reply else {
        panic!("the saved questions page continues on its topic page, got {reply:?}")
    };
    assert_eq!(questions.len(), 2);
    assert_eq!(situations, DocumentSituations::These { situations: treaty.clone() });
    assert!(followups.is_empty(), "every row is a box question or a rare situation");
    let FederalIncome::Dividend { data } =
        &s.draft.income.iter().find(|e| e.id == fund_id).unwrap().income
    else {
        unreachable!()
    };
    assert_eq!(data.reit_eligible, Some(crate::DividendEligibility::Yes));
    assert_eq!(data.section_1202_exclusion, Some(crate::FundQsbsExclusion::None));
    assert_eq!(data.nominee, None);
    assert!(s.draft.situations_unanswered.is_empty());
    assert!(!crate::interview::situations_pending(&s));

    // Revisit: nothing starts unanswered, so the page is the saved data.
    let Reply::Edit(P::SaveIncomes { questions, situations, .. }) = wire_page(&mut s) else {
        panic!("an answered page opens by EditRequest")
    };
    assert_eq!(questions.len(), 2);
    assert_eq!(situations, DocumentSituations::These { situations: treaty });

    // A later paper asks the situations question again, and only its own
    // box questions; the earlier answers keep their saved values.
    let request = save_documents(
        &mut s,
        vec![
            document(fund_id, fund("Example Orchard Fund")),
            document(bank_id, bank("Example Bank")),
            document(0, fund("Example Second Fund")),
        ],
    );
    accepted(&send(&mut s, request));
    let Reply::Response(P::BeginDocumentQuestions { questions, situations, .. }) =
        wire_page(&mut s)
    else {
        panic!("a new paper reopens the questions")
    };
    assert_eq!(questions.len(), 2, "only the new fund's box 5 and box 2c questions");
    assert!(questions.iter().all(|q| !q.key.contains(&alloc::format!("/{fund_id}/"))));
    assert!(situations.is_empty());
    assert!(crate::interview::situations_pending(&s));
}

#[test]
fn wages_page_asks_box_12_tt_per_w2_and_rare_situations_once() {
    let mut draft = complete(FederalStatus::Single);
    let FederalIncome::Wage { data: WageSource::W2(source) } = &mut draft.income[0].income else {
        panic!("complete fixture has its wage source")
    };
    source.form.box12 = vec![crate::PrintedBox12Entry { code: W2Code::TT, amount: Some(100) }];
    let id = draft.income[0].id;
    // As a Document center Save records a new paper.
    draft.situations_unanswered = vec![id];
    let mut s = Interview::new(TEST_USER, draft);
    let page = open_topic(&mut s, TaxSection::Wages);
    let begin = match s.wire_reply(Reply::Edit(page), None) {
        Reply::Response(begin @ P::BeginDocumentQuestions { .. }) => begin,
        other => panic!("the W-2's TT question is asked first, got {other:?}"),
    };
    let P::BeginDocumentQuestions { questions, situations, allowed_situations, topic, .. } = &begin
    else {
        unreachable!()
    };
    assert_eq!(*topic, TaxSection::Wages);
    let key = alloc::format!("income/{id}/wage/overtime");
    assert_eq!(
        questions.iter().map(|q| q.key.clone()).collect::<Vec<_>>(),
        core::slice::from_ref(&key)
    );
    assert!(situations.is_empty(), "the situations question is not answered yet");
    use libertas_macros::variant_index as v;
    assert!(allowed_situations.contains(&(v!(S::InmatePay) as i32)));
    assert!(allowed_situations.contains(&(v!(S::MedicaidWaiver) as i32)));
    assert!(!allowed_situations.contains(&(v!(S::DailyTips) as i32)), "no tips are reported");

    let mut request = answered(
        &begin,
        0,
        vec![DocumentSituation { situation: S::InmatePay { value: 50_000 }, document: id }],
    );
    let P::SaveDocumentQuestions { questions, .. } = &mut request else { unreachable!() };
    *questions =
        vec![question(&key, T::WageOvertime { value: crate::WageDeductionEligibility::Eligible })];
    let reply = wire(&mut s, request);
    let Reply::Edit(P::SaveWagesAnswers { questions, situations, wages, .. }) = reply else {
        panic!("the saved questions continue on the wages page, got {reply:?}")
    };
    assert!(wages.is_empty(), "the W-2's answers are regrouped into questions and situations");
    assert_eq!(questions.len(), 1);
    assert_eq!(
        situations,
        DocumentSituations::These {
            situations: vec![DocumentSituation {
                situation: S::InmatePay { value: 50_000 },
                document: id
            }]
        }
    );
    let P::SaveWagesAnswers { wages, .. } = open_topic(&mut s, TaxSection::Wages) else {
        unreachable!()
    };
    assert_eq!(
        wages[0].answers.overtime_flsa_required,
        Some(crate::WageDeductionEligibility::Eligible)
    );
    assert_eq!(wages[0].answers.inmate_pay, 50_000);
    assert_eq!(wages[0].answers.medicaid_waiver, 0);
    assert!(s.draft.situations_unanswered.is_empty());
}

#[test]
fn w2_box14_unknown_label_saves_on_the_document_center_and_is_asked_on_wages() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let FederalIncome::Wage { data: WageSource::W2(source) } = &s.draft.income[0].income else {
        panic!("complete fixture has its wage source")
    };
    let mut paper = source.form.clone();
    let id = s.draft.income[0].id;
    paper.box14a = vec![W2OtherItem {
        label: "Employer widget".into(),
        value: W2OtherValue::Money { amount: 5_000 },
    }];
    let row = document(id, IncomeSource::Wage { value: crate::WageDocument::W2(paper) });
    let request = save_documents(&mut s, vec![row]);
    // The Document Center only collects: the save is accepted, the kind stays open.
    accepted(&send(&mut s, request));
    let index = s
        .draft
        .income
        .iter()
        .position(|entry| matches!(&entry.income, FederalIncome::Wage { data: WageSource::W2(w) } if !w.form.box14a.is_empty()))
        .expect("the saved W-2");
    let id = s.draft.income[index].id;
    assert_eq!(
        crate::tests::core_tests::native_edit::stored_w2_answers(&s.draft, index).box14[0].kind,
        None
    );
    // The Wages page asks once, as a keyed question of that W-2.
    let page = open_topic(&mut s, TaxSection::Wages);
    let begin = match s.wire_reply(Reply::Edit(page), None) {
        Reply::Response(begin @ P::BeginDocumentQuestions { .. }) => begin,
        other => panic!("the box 14 question is asked first, got {other:?}"),
    };
    let P::BeginDocumentQuestions { questions, .. } = &begin else { unreachable!() };
    let key = alloc::format!("income/{id}/wage/box14/0");
    assert!(questions.iter().any(|q| q.key == key));
    let mut request = answered(&begin, 0, Vec::new());
    let P::SaveDocumentQuestions { questions, .. } = &mut request else { unreachable!() };
    *questions =
        vec![question(&key, T::WageBox14 { kind: crate::W2Box14Kind::Other, state: None })];
    let reply = wire(&mut s, request);
    assert!(matches!(reply, Reply::Edit(P::SaveWagesAnswers { .. })), "{reply:?}");
    let meaning =
        &crate::tests::core_tests::native_edit::stored_w2_answers(&s.draft, index).box14[0];
    assert_eq!(meaning.kind, Some(crate::W2Box14Kind::Other));
}

fn capital_return_paper(name: &str, boxes: (i64, i64, i64)) -> IncomeSource {
    IncomeSource::InterestDividend {
        value: crate::InterestDividendDocument::Div1099(crate::Form1099DivPaper {
            calendar_year: Some(2026),
            nondividend_distributions: Some(boxes.0),
            cash_liquidation: Some(boxes.1),
            noncash_liquidation: Some(boxes.2),
            ..div1099_source(native_div1099(name.into(), 100_000, 0)).form
        }),
    }
}

// Round 9 Smith walk: three plain owned 1099-DIVs, one each with box 3, 9
// and 10. Their lots are asked on the topic page, every pending reconcile item
// links to its row there, and the answered lots let the page save.
#[test]
fn capital_returns_are_asked_on_the_topic_page_and_save() {
    use crate::{
        CapitalDistributionKind as K, CapitalReturn1099Div, GainTerm, LiquidationProperty,
    };
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = save_documents(
        &mut s,
        vec![
            document(0, capital_return_paper("Fund One", (18_000, 0, 0))),
            document(0, capital_return_paper("Fund Two", (0, 60_000, 0))),
            document(0, capital_return_paper("Fund Three", (0, 0, 25_000))),
        ],
    );
    accepted(&send(&mut s, request));
    let ids: Vec<i64> = s
        .draft
        .income
        .iter()
        .filter(|entry| matches!(entry.income, FederalIncome::Dividend { .. }))
        .map(|entry| entry.id)
        .collect();
    assert_eq!(ids.len(), 3);

    let begin = match wire_page(&mut s) {
        Reply::Response(begin @ P::BeginDocumentQuestions { .. }) => begin,
        other => panic!("the situations question opens first, got {other:?}"),
    };
    let P::BeginDocumentQuestions { estimate: Some(estimate), .. } = &begin else {
        panic!("the questions page carries its estimate")
    };
    // Every reconcile item links to its document's capital-returns row.
    let fields: Vec<_> = estimate
        .remaining_items
        .iter()
        .filter(|item| item.item.windows(29).any(|w| w == b"DIV_CAPITAL_RETURNS_RECONCILE"))
        .map(|item| item.field.clone())
        .collect();
    assert_eq!(fields.len(), 3);
    for id in &ids {
        let row = alloc::format!("income/{id}/capital_returns");
        assert!(
            fields.iter().any(|field| field.as_deref().is_some_and(|f| f.contains(&row))),
            "{row} is linked, got {fields:?}"
        );
    }

    // The questions page saves and continues on the topic page, which asks
    // for each document's lots.
    let P::BeginDocumentQuestions {
        cookie,
        revision,
        page,
        previous,
        review,
        progress,
        topic,
        allowed_situations,
        situation_answers,
        situation_documents,
        disaster_choices,
        inherited_iras,
        businesses,
        people,
        inherited_roth_iras,
        wage_scorporations,
        ..
    } = begin
    else {
        unreachable!()
    };
    let reply = wire(
        &mut s,
        P::SaveDocumentQuestions {
            user: TEST_USER,
            cookie,
            revision,
            page,
            previous,
            review,
            estimate: None,
            progress,
            topic,
            questions: Vec::new(),
            situations: DocumentSituations::NoneOfThese,
            allowed_situations,
            situation_answers,
            situation_documents,
            disaster_choices,
            inherited_iras,
            businesses,
            people,
            inherited_roth_iras,
            wage_scorporations,
        },
    );
    let Reply::Edit(mut page @ P::SaveIncomes { .. }) = reply else {
        panic!("the saved questions page continues on its topic page, got {reply:?}")
    };
    let P::SaveIncomes { followups, .. } = &mut page else { unreachable!() };
    let lot = |kind, amount, property: Vec<LiquidationProperty>| CapitalReturn1099Div {
        kind,
        amount,
        property,
        basis: 100_000,
        term: GainTerm::Long,
        special: false,
    };
    let answers = [
        lot(K::Nondividend, 18_000, Vec::new()),
        lot(K::InterimLiquidation, 60_000, Vec::new()),
        lot(
            K::InterimLiquidation,
            0,
            vec![LiquidationProperty { label: "Property".into(), value: 25_000 }],
        ),
    ];
    for (id, answer) in ids.iter().zip(answers) {
        let row = followups
            .iter_mut()
            .find(|row| row.key == alloc::format!("income/{id}/capital_returns"))
            .expect("the topic page asks for this document's lots");
        row.answer = crate::DocumentAnswer::DividendCapitalReturns { value: vec![answer] };
    }
    let reply = wire(&mut s, page);
    // The page saves and the walk continues.
    assert!(matches!(reply, Reply::Edit(_)), "the page saves, got {reply:?}");
    for id in &ids {
        let FederalIncome::Dividend { data } =
            &s.draft.income.iter().find(|entry| entry.id == *id).unwrap().income
        else {
            unreachable!()
        };
        assert_eq!(data.capital_returns.len(), 1);
    }
}

// Savings-bond records are a must-answer strict question: asked unanswered
// with the interest statements and students it chooses from, blocking Save
// until answered, and stored as the same value the canonical row holds.
#[test]
fn savings_bond_records_are_a_required_strict_question() {
    use crate::tests::investment::investment::{bond_record, bond_return, bond_student};
    use libertas_macros::variant_index as v;
    let mut d = bond_return(FederalStatus::Single, 60_000, 3_000);
    let child = d.dependents()[0].id;
    let mut record = bond_record(Owner::Taxpayer, 12_000, 3_000, 20100601, Vec::new());
    let FederalIncome::SavingsBonds { data } = &mut record.income else { unreachable!() };
    data.answers = None;
    add(&mut d, record);
    let bond_id = d.income.last().unwrap().id;
    let mut s = Interview::new(TEST_USER, d);

    let begin = match wire_page(&mut s) {
        Reply::Response(begin @ P::BeginDocumentQuestions { .. }) => begin,
        other => panic!("an unanswered bond record asks its question first, got {other:?}"),
    };
    let P::BeginDocumentQuestions { questions, .. } = &begin else { unreachable!() };
    let key = alloc::format!("income/{bond_id}/savings_bonds_records");
    let asked = questions.iter().find(|q| q.key == key).unwrap_or_else(|| {
        panic!(
            "the bond question is asked, got {:?}",
            questions.iter().map(|q| &q.key).collect::<Vec<_>>()
        )
    });
    assert_eq!(asked.allowed_answers, [v!(T::SavingsBondsRecords) as i32]);
    assert!(!asked.interest_statements.is_empty());
    assert!(asked.people.iter().any(|p| p.id == child));

    // Unanswered, Save is refused.
    let mut blocked = answered(&begin, bond_id, Vec::new());
    let P::SaveDocumentQuestions { questions, .. } = &mut blocked else { unreachable!() };
    questions.clear();
    let reply = wire(&mut s, blocked);
    rejected_with(reply, "DOCUMENT_QUESTIONS_CHANGED");

    // Answered, it round trips to the canonical row's value.
    let statement = asked.interest_statements[0].id;
    let view = crate::SavingsBondAnswersView {
        interest_statement: statement,
        students: vec![bond_student(child, 4_000, 0)],
    };
    let mut answer = answered(&begin, bond_id, Vec::new());
    let P::SaveDocumentQuestions { questions, .. } = &mut answer else { unreachable!() };
    questions.clear();
    questions.push(question(&key, T::SavingsBondsRecords { value: view.clone().into() }));
    let _ = wire(&mut s, answer);
    let FederalIncome::SavingsBonds { data } =
        &s.draft.income.iter().find(|e| e.id == bond_id).unwrap().income
    else {
        unreachable!()
    };
    assert_eq!(data.answers.as_ref().map(Into::into), Some(view));
}
