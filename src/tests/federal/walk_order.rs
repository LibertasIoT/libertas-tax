// The guided walk of a Smith-shaped joint return (acceptance round 3, G-45 and
// G-47): from the first page, the service's next step opens every section whose
// documents or answers make it apply, in the sequence's order, and every page
// offers that next step.
use super::*;
use crate::tests::cross::interview::{go_to, hub_action, tick};

// The documents of the Smith walk that the guided order skipped (G-47): Jane's
// 1099-NEC and 1099-K, a W-2G, a farm cooperative's 1099-PATR, a 1099-LTC, a
// 1099-MISC with other income and a 1098-E; the walk passes the itemized hub
// when nothing is left in it (G-45).
fn smith_shaped() -> FederalDraft {
    let mut d = complete(FederalStatus::Joint);
    let spouse = |value: FederalIncome| {
        let mut entry = income(value);
        entry.owner = crate::DocumentOwner::Spouses;
        entry
    };
    add(
        &mut d,
        spouse(FederalIncome::Nec1099 {
            data: nec_source(FederalForm1099Nec {
                cash_tips: 0,
                tip_occupation_codes: Default::default(),
                overtime: 0,
                compensation: 1_200_000,
                withholding: 0,
                golden_parachute: 0,
                belongs: None,
            }),
        }),
    );
    // The NEC's receiving activity: the walk's Self-employment page refuses a
    // saved paper with no business that links it.
    let nec = crate::related_forms::catalog(&d)
        .into_iter()
        .find(|row| row.kind == crate::related_forms::DocumentKind::Form1099Nec)
        .unwrap()
        .id;
    let mut business = spouse(cash_business(0));
    let FederalIncome::Business { data } = &mut business.income else { unreachable!() };
    data.related = vec![nec];
    add(&mut d, business);
    add(
        &mut d,
        spouse(FederalIncome::K1099 {
            data: k_source(FederalForm1099K {
                cash_tips: 0,
                tip_occupation_codes: Default::default(),
                gross: 800_000,
                withholding: 0,
                parts: vec![FederalKPart { id: 0, amount: 800_000, belongs: None, refunded: 0 }],
            }),
        }),
    );
    add(
        &mut d,
        income(FederalIncome::Gambling {
            data: gambling_source(FederalGambling {
                winnings: 200_000,
                losses: 0,
                withholding: 48_000,
                own_records: false,
                ordinary: true,
            }),
        }),
    );
    // Gambling losses are a Schedule A fact, so the return itemizes by its
    // facts (no stored route) and the deductions page and itemized hub apply.
    add(&mut d, gambling_losses(100));
    let mut patr = FederalCooperative { patronage: 150_000, ..Default::default() };
    patr.allocations.push(crate::cooperative::automatic(&patr));
    add(&mut d, income(FederalIncome::Cooperative { data: cooperative_source(patr) }));
    // The farm that receives the PATR: the Farming page refuses it without one.
    add(&mut d, income(crate::tests::support::builders::related_forms::receiving_farm()));
    add(&mut d, spouse(ltc_form(3_000, None)));
    let misc = FederalForm1099Misc {
        cash_tips: 0,
        tip_occupation_codes: Default::default(),
        overtime: 0,
        crop_insurance: 0,
        component_ids: Default::default(),
        rents: 0,
        royalties: 0,
        other: 70_000,
        other_use: None,
        withholding: 0,
        fishing: 0,
        medical: 0,
        substitute: 0,
        attorney: 0,
        client_funds: 0,
        fish_sales: 0,
        deferred: 0,
    };
    add(&mut d, spouse(FederalIncome::Misc1099 { data: misc_source(misc) }));
    d.adjustments.as_mut().unwrap().student_loan.forms.push(box_1(120_000));
    // Only the household is behind: the walk starts at the Document center.
    d.finished_sections = vec![TaxSection::Children];
    d
}

// The section a reply opens and the request its forward action sends.
fn forward(s: &Interview, reply: Reply) -> Option<(TaxSection, P)> {
    let section = match &reply {
        Reply::Response(P::Review { .. }) => return None,
        Reply::Response(hub @ P::Hub { page, primary_actions, .. }) => {
            assert!(!primary_actions.is_empty(), "the hub {page} offers no next step");
            let section = *crate::interview::SEQUENCE.iter().find(|s| **s as i32 == *page).unwrap();
            // hub_action asserts the hub offers its own Continue (or Review) request.
            return Some((section, hub_action(hub)));
        }
        // The unsaved opening checklist is answered as a client builds it.
        Reply::Response(P::BeginChecklist { .. }) => {
            return Some((TaxSection::Checklist, tick(reply, Checklist::NONE)));
        }
        Reply::Response(P::BeginOftenMissed { .. }) => {
            let request = crate::tests::support::client::answer_often_missed(
                reply,
                crate::OftenMissed::default(),
            );
            return Some((TaxSection::OftenMissed, request));
        }
        Reply::Response(P::BeginPayments { .. }) => {
            let Reply::Edit(request) = crate::tests::support::client::answer_no_estimates(reply)
            else {
                panic!("the payments page answers")
            };
            return Some((TaxSection::Payments, request));
        }
        Reply::Edit(request) => match request {
            P::SaveDocuments { .. } => TaxSection::Documents,
            P::SaveIncomeWithoutForm { .. } => TaxSection::IncomeWithoutForm,
            P::SaveChecklist { .. } => TaxSection::Checklist,
            P::SaveIncomes { topic, .. } => *topic,
            P::SaveWagesAnswers { .. } => TaxSection::Wages,
            P::SaveHomeOffices { .. } => TaxSection::HomeOffices,
            P::SaveHsa { .. } => TaxSection::Hsa,
            P::SaveSelfEmployedBenefits { .. } => TaxSection::SelfEmployedBenefits,
            P::SaveIraContributions { .. } => TaxSection::IraContributions,
            P::SaveEducator { .. } => TaxSection::Educator,
            P::SaveStudentLoan { .. } => TaxSection::StudentLoan,
            P::SaveEmployeeExpenses { .. } => TaxSection::EmployeeExpenses,
            P::SaveMoreAdjustments { .. } => TaxSection::MoreAdjustments,
            P::SaveDeductions { .. } => TaxSection::Deductions,
            P::SaveMedical { .. } => TaxSection::Medical,
            P::SaveTaxes { .. } => TaxSection::Taxes,
            P::SaveMortgages { .. } => TaxSection::Mortgage,
            P::SaveGiftCarryovers { .. } => TaxSection::GiftCarryovers,
            P::SaveOtherItemized { .. } => TaxSection::OtherItemized,
            P::SaveCare { .. } => TaxSection::Care,
            P::SaveEducation { .. } => TaxSection::Education,
            P::SaveSaver { .. } => TaxSection::Saver,
            P::SaveMarketplace { .. } => TaxSection::Marketplace,
            P::SaveMoreCredits { .. } => TaxSection::MoreCredits,
            P::SaveOtherTaxes { .. } => TaxSection::OtherTaxes,
            P::SavePayments { .. } => TaxSection::Payments,
            P::SaveOftenMissed { .. } => TaxSection::OftenMissed,
            other => panic!("unexpected walk page {other:?}"),
        },
        other => panic!("a walk page without a next step: {other:?} ({:?})", s.draft.revision),
    };
    let Reply::Edit(request) = reply else { unreachable!() };
    Some((section, request))
}

#[test]
fn smith_walk_opens_every_section_with_facts_in_order_and_always_offers_next() {
    let mut s = restart(smith_shaped());
    let mut reply = send(&mut s, P::OpenInterview { user: TEST_USER });
    let mut opened: Vec<TaxSection> = Vec::new();
    // Each page's Save (or a hub's Continue) opens the walk's next step.
    while !matches!(reply, Reply::Response(P::Review { .. })) {
        let (section, request) = forward(&s, reply).expect("a walk page");
        assert!(opened.len() < 80, "the walk does not end: {opened:?}");
        opened.push(section);
        reply = send(&mut s, request);
        if let Reply::Response(P::Problem { .. }) = &reply {
            panic!("{section:?} refused its own next step: {reply:?}");
        }
    }
    let with_facts = [
        TaxSection::Documents,
        TaxSection::Checklist,
        TaxSection::SelfEmployment,
        TaxSection::OtherIncome,
        TaxSection::LongTermCareBenefits,
        TaxSection::Gambling,
        TaxSection::Farming,
        TaxSection::StudentLoan,
        TaxSection::Deductions,
        TaxSection::Itemized,
    ];
    for section in with_facts {
        assert!(opened.contains(&section), "{section:?} was never opened: {opened:?}");
    }
    // Hubs are summaries a walk may return to; every other page opens in order.
    let hubs =
        [TaxSection::Income, TaxSection::Adjustments, TaxSection::Itemized, TaxSection::Credits];
    let rank = |section: &TaxSection| {
        crate::interview::SEQUENCE.iter().position(|s| s == section).unwrap()
    };
    let pages: Vec<usize> = opened.iter().filter(|s| !hubs.contains(s)).map(rank).collect();
    assert!(pages.windows(2).all(|pair| pair[0] < pair[1]), "{opened:?}");
    // The review (G-56, G-58): each remaining item once, under one page; a
    // finishing row names its page; saved sections are answered.
    let Reply::Response(P::Review { remaining_items, answers, .. }) = reply else { unreachable!() };
    for (index, item) in remaining_items.iter().enumerate() {
        let text = status_text(&item.status);
        assert!(
            !remaining_items[..index].iter().any(|other| other.status == item.status),
            "listed twice: {text}"
        );
        assert!(!text.starts_with("Complete the remaining applicable pages"), "{text}");
    }
    for row in answers.iter().filter(|row| {
        matches!(row.section, TaxSection::Deductions | TaxSection::Itemized | TaxSection::Credits)
    }) {
        assert_ne!(status_text(&row.status), "Not answered yet", "{:?}", row.section);
    }
}

// G-45 (walk groups 7 to 9): the itemized hub whose walk has nothing left
// is passed like the other hubs. A page saved after a jump ahead from it
// opens the walk's next step, never that hub again, through to the review.
#[test]
fn a_jump_ahead_from_the_itemized_hub_continues_the_walk_without_returning_to_it() {
    let mut s = restart(smith_shaped());
    let mut reply = send(&mut s, P::OpenInterview { user: TEST_USER });
    loop {
        if let Reply::Response(P::Hub { page, .. }) = &reply
            && *page == TaxSection::Itemized as i32
        {
            break;
        }
        let (_, request) = forward(&s, reply).expect("the walk reaches the itemized hub");
        reply = send(&mut s, request);
    }
    let request = go_to(&s.draft, TaxSection::Itemized as i32, TaxSection::OtherTaxes);
    let Reply::Edit(page @ P::SaveOtherTaxes { .. }) = send(&mut s, request) else {
        panic!("Go to opens other taxes")
    };
    let mut reply = send(&mut s, page);
    let mut after = Vec::new();
    while let Some((section, request)) = forward(&s, reply) {
        assert!(after.len() < 40, "the walk does not end: {after:?}");
        assert_ne!(section, TaxSection::Itemized, "the save returned to the itemized hub");
        after.push(section);
        reply = send(&mut s, request);
    }
    assert!(!after.contains(&TaxSection::OtherTaxes), "{after:?}");
}
