// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::support::assert::{accepted, rejected_with};
use super::*;
use crate::interview::estimate_text;
// Shared native producers and page traversal; keep established topic-call paths.
use super::support::client::income_pages::save_topic;
use crate::tests::cross::interview::{
    PAGE_CREDITS, PAGE_OTHER_TAXES, PAGE_PAYMENTS, at_the_payments_page, back, figure_with,
    navigate, no_payments, open_payments, page_paid, paid, payments_value, refusal, save_payments,
    save_setup, shown, submit, value_of,
};
const PAGE_INCOME_BY_PERIOD: i32 = TaxSection::IncomeByPeriod as i32;

#[test]
fn dated_payments_total_the_estimated_payments() {
    let d = complete(FederalStatus::Single);
    let baseline = supported(&d);
    let (mut s, page) = at_the_payments_page(d.clone());
    // The page follows the credits hub and opens complete: no payments, the
    // IRS figures the penalty, and the withholding of Form 1040 lines
    // 25a-25d (complete()'s one W-2, box 2 $6,500.00).
    let P::SavePayments { page: section, previous, progress, .. } = &page else { unreachable!() };
    // M3-P19: Back opens the previous shown section, other taxes (every
    // return walks it, gate fixes batch A); setup, you, people, the
    // checklist, income, adjustments, credits, other taxes, payments and
    // the review.
    assert_eq!((*section, *previous), (PAGE_PAYMENTS, PAGE_OTHER_TAXES));
    assert!(shown(progress).contains("section 9 of 10"));
    let (value, _, _, _) = payments_value(&page);
    assert_eq!(
        value,
        &Payments {
            credit_capacity: baseline.credit_capacity,
            withheld: Some(Withheld { w2: 650_000, forms_1099: 0, other: 0, total: 650_000 }),
            ..no_payments()
        }
    );
    // Two dated payments and an extension payment. By hand: estimated =
    // $1,200.00 + $800.00 = $2,000.00 (Form 1040 line 26); with the
    // $300.00 extension payment the payments rise by $2,300.00.
    let entered = Payments {
        estimated: vec![page_paid(20260415, 120_000), page_paid(20260915, 80_000)],
        extension: 30_000,
        extension_payer: Some(PaymentPayer::Joint),
        ..value.clone()
    };
    match submit(&mut s, entered.clone()) {
        Reply::Response(P::Review { result, answers, .. }) => {
            assert_eq!(result.state, ResultState::EstimatedForSupportedScope);
            assert_eq!(result.payments, baseline.payments.map(|v| v + 230_000));
            assert_eq!(return_line(&result, "26"), 200_000);
            let row = answers.iter().find(|r| r.section == TaxSection::Payments).unwrap();
            assert_eq!(status_text(&row.status), "3 payments");
        }
        other => panic!("expected the review, got {other:?}"),
    }
    let pay = s.draft.payments.clone().unwrap();
    assert_eq!(pay.estimated, Some(200_000));
    assert_eq!(
        pay.estimated_payments,
        entered.estimated.iter().map(InjuredDatedPayment::accepted).collect::<Vec<_>>()
    );
    assert_eq!(pay.extension, 30_000);
    assert_eq!(pay.prior_year, None);
    assert!(s.draft.finished_sections.contains(&TaxSection::Payments));
    // It reopens as entered, and an unchanged save stores no revision.
    let revision = s.draft.revision;
    let page = open_payments(&mut s);
    assert_eq!(
        payments_value(&page).0,
        &Payments { credit_capacity: Some(377_700), ..entered.clone() }
    );
    submit(&mut s, entered);
    assert_eq!(s.draft.revision, revision);
    // Back returns to the credits hub.
    let request = back(&s.draft, PAGE_CREDITS);
    assert!(matches!(send(&mut s, request), Reply::Response(P::Hub { page: PAGE_CREDITS, .. })));
    // The running estimate shows the page's answers before they are saved.
    let request = P::EstimatePayments {
        allowed_refund_management: vec![],

        user: TEST_USER,
        allowed_prior_returns: vec![0],
        allowed_farmer: vec![0],
        allowed_farmer_next_year: vec![0, 1, 2],
        value: Payments { estimated: vec![page_paid(20260115, 100_000)], ..no_payments() }.into(),
    };
    let Reply::Response(P::PaymentsEstimate { estimate: text, .. }) = send(&mut s, request) else {
        panic!("expected the running estimate")
    };
    let mut expected = s.draft.clone();
    expected.payments = Some(FederalPayments {
        estimated: Some(100_000),
        estimated_payments: vec![paid(20260115, 100_000)],
        ..FederalPayments::none()
    });
    assert_eq!(text, estimate_text(&estimate(&expected)));
}

#[test]
fn irs_figures_leaves_no_prior_year() {
    let (mut s, _) = at_the_payments_page(complete(FederalStatus::Single));
    // Figuring the penalty stores the 2025 facts with every Form 2210
    // answer: box D, box C, the farmer rule and a box A waiver.
    let figured = Payments {
        estimated: vec![page_paid(20260415, 100_000)],
        penalty: PenaltyChoice::Figure {
            prior_tax: 500_000,
            prior_agi: Some(6_000_000),
            full_year: true,
            prior_return: PriorReturn::SameReturn,
            withholding: WithholdingTiming::ByPeriod {
                first: 150_000,
                second: 150_000,
                third: 150_000,
                fourth: 200_000,
            },
            income: IncomeTiming::Evenly,
            farmer: Farmer::No,
            waiver: Waiver::Full {
                reason: PenaltyWaiverReason::CasualtyOrDisasterOrUnusual,
                explanation: "House fire in March".into(),
            },
        },
        ..no_payments()
    };
    accepted(&submit(&mut s, figured.clone()));
    let pay = s.draft.payments.clone().unwrap();
    assert_eq!(
        pay.prior_year,
        Some(PriorYearTax { full_year: true, tax: 500_000, agi: Some(6_000_000), spouse_tax: 0 })
    );
    assert!(pay.withholding_by_date);
    assert_eq!(
        pay.withholding_periods,
        Some(PeriodWithholding {
            first: 150_000,
            second: 150_000,
            third: 150_000,
            fourth: 200_000,
        })
    );
    assert!(!pay.annualized && pay.annualized_income.is_none());
    assert!(matches!(pay.waiver, Some(PenaltyWaiver::Full { .. })));
    // Form 2210 runs: the estimate carries its line 38.
    assert!(
        estimate(&s.draft)
            .return_lines
            .iter()
            .any(|l| status_text(&l.label).starts_with("1040 38 —"))
    );
    let page = open_payments(&mut s);
    assert_eq!(payments_value(&page).0.penalty, figured.penalty, "reopens as entered");
    // Letting the IRS figure it erases every Form 2210 fact and keeps the
    // payments.
    let irs = Payments { penalty: PenaltyChoice::IrsFigures, ..figured };
    accepted(&submit(&mut s, irs));
    let pay = s.draft.payments.clone().unwrap();
    assert_eq!(
        pay,
        FederalPayments {
            credit_applied_next_year: 0,
            estimated: Some(100_000),
            estimated_payments: vec![paid(20260415, 100_000)],
            ..FederalPayments::none()
        }
    );
    assert!(
        !estimate(&s.draft)
            .return_lines
            .iter()
            .any(|l| status_text(&l.label).starts_with("1040 38 —"))
    );
    // Box C is stored as the annualized election and opens the income by
    // period page (M3-P14); until its periods are saved the estimate
    // refuses it by name. Back to Evenly erases it.
    let box_c = Payments {
        penalty: PenaltyChoice::Figure {
            prior_tax: 500_000,
            prior_agi: Some(6_000_000),
            full_year: true,
            prior_return: PriorReturn::SameReturn,
            withholding: WithholdingTiming::Evenly,
            income: IncomeTiming::ByPeriod,
            farmer: Farmer::No,
            waiver: Waiver::None,
        },
        ..no_payments()
    };
    assert!(matches!(submit(&mut s, box_c.clone()), Reply::Edit(P::SaveIncomeByPeriod { .. })));
    let pay = s.draft.payments.clone().unwrap();
    assert!(pay.annualized && pay.annualized_income.is_none());
    let result = estimate(&s.draft);
    assert!(
        result.issues.iter().any(|i| i.message.contains("Form 2210 Part II box C")),
        "{:?}",
        result.issues
    );
    assert_eq!(payments_value(&open_payments(&mut s)).0.penalty, box_c.penalty);
    accepted(&submit(
        &mut s,
        Payments {
            penalty: figure_with(
                PriorReturn::SameReturn,
                WithholdingTiming::Evenly,
                Farmer::No,
                Waiver::None,
            ),
            ..no_payments()
        },
    ));
    assert!(!s.draft.payments.as_ref().unwrap().annualized);
    // A waiver with a blank explanation is refused on the server too.
    let blank = Payments {
        penalty: figure_with(
            PriorReturn::SameReturn,
            WithholdingTiming::Evenly,
            Farmer::No,
            Waiver::Full {
                reason: PenaltyWaiverReason::RetirementOrDisability,
                explanation: "  ".into(),
            },
        ),
        ..no_payments()
    };
    let before = s.draft.clone();
    error(send(&mut s, save_payments(&before, blank)));
    assert_eq!(s.draft, before);
}

#[test]
fn box_d_periods_must_total_the_withholding() {
    assert!(APP_STRINGS.iter().any(|(key, _)| *key == "WITHHOLDING_PERIODS_NOT_TOTAL"));
    // Server side: the four periods must add up to line 25d, $6,500.00 on
    // complete()'s one W-2. One cent short is refused with the same key,
    // on save and in the running estimate.
    let (mut s, _) = at_the_payments_page(complete(FederalStatus::Single));
    let by_period = |fourth: i64| Payments {
        penalty: figure_with(
            PriorReturn::SameReturn,
            WithholdingTiming::ByPeriod { first: 150_000, second: 150_000, third: 150_000, fourth },
            Farmer::No,
            Waiver::None,
        ),
        ..no_payments()
    };
    let before = s.draft.clone();
    rejected_with(
        send(&mut s, save_payments(&before, by_period(199_999))),
        "WITHHOLDING_PERIODS_NOT_TOTAL",
    );
    assert_eq!(s.draft, before);
    rejected_with(
        send(
            &mut s,
            P::EstimatePayments {
                allowed_refund_management: vec![],

                user: TEST_USER,
                allowed_prior_returns: vec![0],
                allowed_farmer: vec![0],
                allowed_farmer_next_year: vec![0, 1, 2],
                value: by_period(199_999).into(),
            },
        ),
        "WITHHOLDING_PERIODS_NOT_TOTAL",
    );
    // A cent over is refused as well; the exact total is stored.
    rejected_with(
        send(&mut s, save_payments(&before, by_period(200_001))),
        "WITHHOLDING_PERIODS_NOT_TOTAL",
    );
    accepted(&send(&mut s, save_payments(&before, by_period(200_000))));
    let pay = s.draft.payments.as_ref().unwrap();
    assert!(pay.withholding_by_date);
    assert_eq!(pay.withholding_periods.as_ref().map(|p| p.fourth), Some(200_000));
    // The withholding a client sends is ignored: the service compares with
    // the return's own line 25d.
    let (mut s, _) = at_the_payments_page(complete(FederalStatus::Single));
    let forged = Payments {
        withheld: Some(Withheld { w2: 649_999, forms_1099: 0, other: 0, total: 649_999 }),
        ..by_period(199_999)
    };
    let before = s.draft.clone();
    rejected_with(send(&mut s, save_payments(&before, forged)), "WITHHOLDING_PERIODS_NOT_TOTAL");
}

#[test]
fn box_e_is_offered_only_on_joint() {
    let separate = |spouse_tax| Payments {
        penalty: figure_with(
            PriorReturn::SeparateReturns { spouse_tax },
            WithholdingTiming::Evenly,
            Farmer::No,
            Waiver::None,
        ),
        ..no_payments()
    };
    // A single return offers only the one 2025 return, and refuses box E
    // on save whatever list the client echoes.
    let (mut s, page) = at_the_payments_page(complete(FederalStatus::Single));
    assert_eq!(payments_value(&page).1, &[0]);
    let before = s.draft.clone();
    error(send(&mut s, save_payments(&before, separate(300_000))));
    assert_eq!(s.draft, before);
    // A joint return offers box E and stores the spouse's separate tax on
    // the prior-year record.
    let (mut s, page) = at_the_payments_page(complete(FederalStatus::Joint));
    assert_eq!(payments_value(&page).1, &[0, 1]);
    accepted(&submit(&mut s, separate(300_000)));
    assert_eq!(
        s.draft.payments.as_ref().and_then(|p| p.prior_year.as_ref()).map(|p| p.spouse_tax),
        Some(300_000)
    );
    let page = open_payments(&mut s);
    assert_eq!(payments_value(&page).0.penalty, separate(300_000).penalty);
    // Leaving the joint return erases the payments entered for it, box E
    // with them (walk-owned data: they were the couple's).
    let request = save_setup(&s.draft, ReturnSetup { filing: Filing::Single });
    send(&mut s, request);
    let pay = s.draft.payments.as_ref().unwrap();
    assert!(pay.prior_year.is_none() && pay.nothing_entered() && !pay.joint);
}

#[test]
fn payment_date_outside_2026_is_rejected() {
    // Client side: the date field carries the window as its range.
    // 2026-10-16: required cap-reason comments do not change the declared date window.
    let source = include_str!("../../payments/underpayment.rs")
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(source.contains(
        "#[libertas_date_only]\n    #[libertas_number(min = $ESTIMATED_PAYMENT_START_DATE, max = $ESTIMATED_PAYMENT_LAST_DATE)]\n    pub date: u32,"
    ));
    assert_eq!(crate::year_constants::ESTIMATED_PAYMENT_START_DATE, 20260101);
    assert_eq!(crate::year_constants::ESTIMATED_PAYMENT_LAST_DATE, 20270415);
    // Server side: January 1, 2026 through April 15, 2027 inclusive; the
    // day before, the day after and an impossible date are refused.
    let (mut s, _) = at_the_payments_page(complete(FederalStatus::Single));
    let dated = |date| Payments { estimated: vec![page_paid(date, 50_000)], ..no_payments() };
    let before = s.draft.clone();
    for date in [20251231, 20270416, 20260230] {
        rejected_with(
            send(&mut s, save_payments(&before, dated(date))),
            "PAYMENT_DATE_OUTSIDE_2026",
        );
        assert_eq!(s.draft, before, "{date}");
    }
    accepted(&send(&mut s, save_payments(&before, dated(20260101))));
    assert_eq!(s.draft.payments.as_ref().unwrap().estimated_payments, vec![paid(20260101, 50_000)]);
    let later = s.draft.clone();
    accepted(&send(&mut s, save_payments(&later, dated(20270415))));
    assert_eq!(s.draft.payments.as_ref().unwrap().estimated, Some(50_000));
}

// M3-P14: the income by period page.

// Figure the penalty with the income answer given, otherwise as
// figure_with's defaults.
fn figure_income(income: IncomeTiming) -> PenaltyChoice {
    PenaltyChoice::Figure {
        prior_tax: 500_000,
        prior_agi: Some(6_000_000),
        full_year: true,
        prior_return: PriorReturn::SameReturn,
        withholding: WithholdingTiming::Evenly,
        income,
        farmer: Farmer::No,
        waiver: Waiver::None,
    }
}
fn box_c_payments() -> Payments {
    Payments { penalty: figure_income(IncomeTiming::ByPeriod), ..no_payments() }
}
fn save_periods(d: &FederalDraft, value: IncomeByPeriod, review: bool) -> P {
    P::SaveIncomeByPeriod {
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
        page: PAGE_INCOME_BY_PERIOD,
        previous: PAGE_PAYMENTS,
        review,
        estimate: vec![],
        warnings: vec![],
        progress: vec![],
        value,
    }
}
fn periods_page(reply: Reply) -> IncomeByPeriod {
    match reply {
        Reply::Edit(P::SaveIncomeByPeriod { value, .. }) => value,
        other => panic!("expected the income by period page, got {other:?}"),
    }
}
// A period with only its adjusted gross income, in cents.
fn agi_only(agi: i64) -> AnnualizedPeriod {
    AnnualizedPeriod {
        agi,
        itemized: 0,
        qbi: 0,
        additional: 0,
        self_employment: 0,
        social_wages: 0,
    }
}
fn stored_periods(s: &Interview) -> Option<AnnualizedIncome> {
    s.draft.payments.as_ref().and_then(|p| p.annualized_income.clone())
}
fn line_in_cents(r: &FederalResult, id: &str) -> i64 {
    r.lines.iter().find(|l| l.id == id).map_or_else(|| panic!("no line {id}"), |l| l.amount)
}

#[test]
fn income_by_period_opens_only_for_box_c() {
    // Even income (no box C): the payments page goes on to the review, and
    // the page is neither offered nor opened nor saved.
    let (mut s, _) = at_the_payments_page(complete(FederalStatus::Single));
    match submit(&mut s, Payments { penalty: figure_income(IncomeTiming::Evenly), ..no_payments() })
    {
        Reply::Response(P::Review { allowed_sections, answers, .. }) => {
            assert!(!allowed_sections.contains(&PAGE_INCOME_BY_PERIOD));
            assert!(!answers.iter().any(|r| r.section == TaxSection::IncomeByPeriod));
        }
        other => panic!("expected the review, got {other:?}"),
    }
    let request = navigate(&s.draft, TaxSection::IncomeByPeriod);
    assert!(refusal(send(&mut s, request)).contains("This section is not available yet."));
    let before = s.draft.clone();
    let forged = IncomeByPeriod {
        through_march: agi_only(100_000),
        through_may: agi_only(200_000),
        through_august: agi_only(300_000),
        full_year: agi_only(6_000_000),
    };
    error(send(&mut s, save_periods(&before, forged, false)));
    assert_eq!(s.draft, before);

    // Box C opens the page next, in the payments section with Back to the
    // payments page. complete()'s return: one W-2 of $60,000.00, no
    // adjustments, the standard deduction, no self-employment. By hand the
    // full year is AGI $60,000.00 and nothing else; the first open spreads
    // it evenly: 3/12 = $15,000.00, 5/12 = $25,000.00, 8/12 = $40,000.00.
    let (mut s, _) = at_the_payments_page(complete(FederalStatus::Single));
    let reply = submit(&mut s, box_c_payments());
    let Reply::Edit(P::SaveIncomeByPeriod { page, previous, ref progress, .. }) = reply else {
        panic!("expected the income by period page, got {reply:?}")
    };
    assert_eq!((page, previous), (PAGE_INCOME_BY_PERIOD, PAGE_PAYMENTS));
    // M3-P19: setup, you, people, the checklist, income, adjustments,
    // credits, other taxes, payments and the review.
    assert!(shown(progress).contains("section 9 of 10"));
    let even = periods_page(reply);
    assert_eq!(
        even,
        IncomeByPeriod {
            through_march: agi_only(1_500_000),
            through_may: agi_only(2_500_000),
            through_august: agi_only(4_000_000),
            full_year: agi_only(6_000_000),
        }
    );
    // Until the periods are saved the page is the open item: the estimate
    // refuses box C by name, resume opens the page and the review suggests it.
    assert_eq!(stored_periods(&s), None);
    let result = estimate(&s.draft);
    assert!(
        result.issues.iter().any(|i| i.message.contains("Form 2210 Part II box C")),
        "{:?}",
        result.issues
    );
    assert_eq!(periods_page(send(&mut s, P::OpenInterview { user: TEST_USER })), even);
    let request = back(&s.draft, PAGE_PAYMENTS);
    assert!(matches!(send(&mut s, request), Reply::Edit(P::SavePayments { .. })));
    let Reply::Response(P::Review { allowed_sections, answers, suggested, .. }) =
        send(&mut s, P::ReviewAccepted { user: TEST_USER })
    else {
        panic!("expected the review")
    };
    assert!(allowed_sections.contains(&PAGE_INCOME_BY_PERIOD));
    assert_eq!(suggested, TaxSection::IncomeByPeriod);
    let row = answers.iter().find(|r| r.section == TaxSection::IncomeByPeriod).unwrap();
    assert_eq!(status_text(&row.status), "Not answered yet");
    // The running estimate calculates the page before it is saved.
    let request = P::EstimateIncomeByPeriod { user: TEST_USER, value: even.clone() };
    assert!(matches!(send(&mut s, request), Reply::Response(P::RunningEstimate { .. })));
    // Saving it stores Schedule AI and continues at the review; the return
    // is supported with column (a)'s line 1 as entered.
    let request = save_periods(&s.draft, even.clone(), false);
    match send(&mut s, request) {
        Reply::Response(P::Review { answers, .. }) => {
            let row = answers.iter().find(|r| r.section == TaxSection::IncomeByPeriod).unwrap();
            assert_eq!(status_text(&row.status), "Entered");
        }
        other => panic!("expected the review, got {other:?}"),
    }
    let stored = stored_periods(&s).unwrap();
    assert_eq!(stored.through_march, even.through_march);
    assert_eq!(stored.full_year, even.full_year);
    let result = supported(&s.draft);
    assert_eq!(line_in_cents(&result, "form_2210_ai_line_1_a"), 1_500_000);
    assert_eq!(line_in_cents(&result, "form_2210_ai_line_1_d"), 6_000_000);
    // Answered, the interview resumes at the review; from the review an
    // unchanged save returns there without a new revision.
    assert!(matches!(
        send(&mut s, P::OpenInterview { user: TEST_USER }),
        Reply::Response(P::Review { .. })
    ));
    let revision = s.draft.revision;
    let request = navigate(&s.draft, TaxSection::IncomeByPeriod);
    let page = periods_page(send(&mut s, request));
    let request = save_periods(&s.draft, page, true);
    assert!(matches!(send(&mut s, request), Reply::Response(P::Review { .. })));
    assert_eq!(s.draft.revision, revision);
}

#[test]
fn full_year_column_is_the_return() {
    // complete()'s $60,000.00 W-2 (box 3 $60,000.00) plus a $20,000.00
    // Schedule C profit, single, standard deduction. By hand:
    // Schedule SE: 92.35% x $20,000 = $18,470.00; 15.3% = $2,825.91, shown
    // $2,826; half, $1,413, is the Schedule 1 adjustment.
    // AGI (line 11) = $60,000 + $20,000 - $1,413 = $78,587.
    // QBI (line 13b): 20% x ($20,000 - $1,413) = $3,717.40, below 20% of
    // $78,587 - $16,100 = $62,487 ($12,497.40), so $3,717.
    // Column (d): AGI $78,587.00, itemized 0, QBI $3,717.00, Schedule 1-A 0,
    // net profit $20,000.00 and Social Security wages $60,000.00 of the
    // taxpayer, the person with self-employment tax.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(cash_business(20_000)));
    let (mut s, _) = at_the_payments_page(d);
    let page = periods_page(submit(&mut s, box_c_payments()));
    let full = AnnualizedPeriod {
        agi: 7_858_700,
        itemized: 0,
        qbi: 371_700,
        additional: 0,
        self_employment: 2_000_000,
        social_wages: 6_000_000,
    };
    assert_eq!(page.full_year, full);
    // A column (d) the client sends is ignored: the stored column is the
    // return's, and Schedule AI reconciles to it (taxable income
    // $78,587 - $16,100 - $3,717 = $58,770; SE tax within $1).
    let entered = IncomeByPeriod {
        through_march: agi_only(1_000_000),
        through_may: agi_only(2_000_000),
        through_august: agi_only(4_000_000),
        full_year: agi_only(1),
    };
    let request = save_periods(&s.draft, entered, false);
    accepted(&send(&mut s, request));
    assert_eq!(stored_periods(&s).unwrap().full_year, full);
    let result = supported(&s.draft);
    assert_eq!(value_of(&result, "taxable_income"), 58_770);
    assert_eq!(line_in_cents(&result, "form_2210_ai_line_1_d"), 7_858_700);
    // Column (d) follows the return: removing the business leaves the W-2
    // alone, so AGI $60,000.00 and nothing else.
    let request = save_topic(&s.draft, TaxSection::SelfEmployment, vec![]);
    send(&mut s, request);
    assert_eq!(stored_periods(&s).unwrap().full_year, agi_only(6_000_000));
    let request = navigate(&s.draft, TaxSection::IncomeByPeriod);
    let page = periods_page(send(&mut s, request));
    assert_eq!(page.full_year, agi_only(6_000_000));
    assert_eq!(page.through_may, agi_only(2_000_000), "entered periods are kept");
    supported(&s.draft);
}

#[test]
fn box_c_back_to_evenly_erases_the_periods() {
    let (mut s, _) = at_the_payments_page(complete(FederalStatus::Single));
    let even = periods_page(submit(&mut s, box_c_payments()));
    let entered = IncomeByPeriod {
        through_march: agi_only(500_000),
        through_may: agi_only(1_000_000),
        through_august: agi_only(2_000_000),
        ..even.clone()
    };
    let request = save_periods(&s.draft, entered.clone(), false);
    accepted(&send(&mut s, request));
    assert!(stored_periods(&s).is_some());
    // The page reopens as entered.
    let request = navigate(&s.draft, TaxSection::IncomeByPeriod);
    assert_eq!(periods_page(send(&mut s, request)), entered);
    // Box C back to Evenly erases the periods and goes on to the review;
    // the page is no longer offered.
    open_payments(&mut s);
    assert!(matches!(
        submit(&mut s, Payments { penalty: figure_income(IncomeTiming::Evenly), ..no_payments() },),
        Reply::Response(P::Review { .. })
    ));
    let pay = s.draft.payments.clone().unwrap();
    assert!(!pay.annualized);
    assert_eq!(pay.annualized_income, None);
    assert!(pay.prior_year.is_some());
    let request = navigate(&s.draft, TaxSection::IncomeByPeriod);
    assert!(refusal(send(&mut s, request)).contains("This section is not available yet."));
    // Choosing box C again starts over from the even spread.
    assert_eq!(periods_page(submit(&mut s, box_c_payments())), even);
    let request = save_periods(&s.draft, entered, false);
    accepted(&send(&mut s, request));
    // Letting the IRS figure the penalty erases them as well.
    accepted(&submit(&mut s, no_payments()));
    let pay = s.draft.payments.clone().unwrap();
    assert!(!pay.annualized);
    assert_eq!(pay.annualized_income, None);
    assert_eq!(pay.prior_year, None);
}

#[test]
fn applied_credit_capacity_refreshes_and_native_errors_locate_the_answer() {
    let (mut session, page) = at_the_payments_page(complete(FederalStatus::Single));
    let mut value = payments_value(&page).0.clone();
    let before = session.draft.clone();
    let capacity = value.credit_capacity.unwrap();
    value.extension = 100_000;
    value.extension_payer = Some(PaymentPayer::Taxpayer);
    let request = P::EstimatePayments {
        user: TEST_USER,
        allowed_prior_returns: vec![],
        allowed_farmer: vec![],
        allowed_farmer_next_year: vec![],
        allowed_refund_management: vec![],
        value: value.clone().into(),
    };
    let Reply::Response(P::PaymentsEstimate { credit_capacity, .. }) = send(&mut session, request)
    else {
        panic!("payments calculation")
    };
    assert_eq!(credit_capacity, Some(capacity + 100_000));
    assert_eq!(session.draft, before);
    for (amount, recipient, key, field) in [
        (-1, None, "PAYMENT_APPLIED_CREDIT_NEGATIVE", "[value/credit_applied_next_year]"),
        (1, None, "PAY_CREDIT_RECIPIENT_NEEDED", "[value/credit_recipient]"),
    ] {
        value.credit_applied_next_year = amount;
        value.credit_recipient = recipient;
        let request = save_payments(&before, value.clone());
        let Reply::Response(P::Problem { message, field: located }) = send(&mut session, request)
        else {
            panic!("invalid election must be refused")
        };
        assert_eq!(message, Message::new(key, vec![]).encode());
        assert_eq!(located.as_deref(), Some(field));
        assert_eq!(session.draft, before);
    }
}

#[test]
fn applied_credit_unknown_capacity_saves_intent_without_admitting_cash_or_forecast_credit() {
    let mut draft = complete(FederalStatus::Single);
    let FederalIncome::EmployeeWages { mut data } = omitted_employee_wages(7_500) else {
        unreachable!()
    };
    data.complete = false;
    add(&mut draft, income(FederalIncome::EmployeeWages { data }));
    let baseline = supported(&draft);
    let (mut session, page) = at_the_payments_page(draft);
    let mut value = payments_value(&page).0.clone();
    assert_eq!(value.credit_capacity, None);
    value.credit_applied_next_year = 100_050;
    value.credit_recipient = Some(OverpaymentRecipient::TaxpayerAccount);
    accepted(&submit(&mut session, value));
    let result = estimate(&session.draft);
    assert_eq!((result.credit_capacity, result.refund), (None, None));
    assert_eq!(result.state, baseline.state);
    assert_eq!(
        estimate_text(&result),
        Message::new("PAYMENT_APPLIED_CREDIT_UNAVAILABLE", vec![]).encode()
    );
    assert_eq!(
        (result.tax, result.payments, result.balance),
        (baseline.tax, baseline.payments, baseline.balance)
    );
    assert_eq!(result.lines, baseline.lines);
    let ordinary = |r: &FederalResult| {
        r.return_lines
            .iter()
            .filter(|row| {
                row.form_line
                    .as_ref()
                    .is_none_or(|line| !matches!(line.line.as_str(), "35a" | "36"))
            })
            .cloned()
            .collect::<Vec<_>>()
    };
    assert_eq!(ordinary(&result), ordinary(&baseline));
    assert!(
        result.review_notes.iter().any(|n| n.display.key == "PAYMENT_APPLIED_CREDIT_UNAVAILABLE")
    );
    assert!(!result.return_lines.iter().any(|row| {
        row.form_line.as_ref().is_some_and(|line| matches!(line.line.as_str(), "35a" | "36"))
    }));
    assert!(!result.lines.iter().any(|l| l.id.starts_with("form_1040_es_planned_cash_")));
    assert!(!result.review_notes.iter().any(|n| n.display.key == "NOTE_APPLIED_CREDIT_PLAN"));
    assert_eq!(payments_value(&open_payments(&mut session)).0.credit_applied_next_year, 100_050);
}

#[test]
fn applied_credit_intent_survives_a_household_change_and_rechecks_capacity() {
    let mut draft = complete(FederalStatus::Joint);
    draft.income = vec![wage(1, Owner::Taxpayer, 6_000_000, 650_000)];
    let (mut session, _) = at_the_payments_page(draft);
    let value = Payments {
        credit_applied_next_year: 250_050,
        credit_recipient: Some(OverpaymentRecipient::SpouseAccount),
        extension: 1_000_000,
        extension_payer: Some(PaymentPayer::Joint),
        ..no_payments()
    };
    accepted(&submit(&mut session, value));
    let request = save_setup(&session.draft, ReturnSetup { filing: Filing::Single });
    send(&mut session, request);
    let pay = session.draft.payments.as_ref().unwrap();
    assert_eq!(
        (pay.credit_applied_next_year, pay.credit_recipient),
        (250_050, Some(OverpaymentRecipient::SpouseAccount))
    );
    assert!(!pay.joint);
    assert!(!session.draft.finished_sections.contains(&TaxSection::Payments));
    let page = open_payments(&mut session);
    let reopened = payments_value(&page).0;
    assert_eq!(reopened.credit_applied_next_year, 250_050);
    // Single-return tax $5,023 against $6,500 withholding: $1,477 remains.
    assert_eq!(reopened.credit_capacity, Some(147_700));
}

#[test]
fn applied_credit_defaults_to_zero_on_the_normal_final_payments_page() {
    let mut draft = complete(FederalStatus::Single);
    draft.payments = None;
    let (mut session, page) = at_the_payments_page(draft);
    assert_eq!(payments_value(&page).0.credit_applied_next_year, 0);
    let value = payments_value(&page).0.clone();
    accepted(&submit(&mut session, value));
    assert_eq!(session.draft.payments.as_ref().unwrap().credit_applied_next_year, 0);
}

#[test]
fn applied_credit_stored_validity_checks_intrinsics_without_freezing_capacity() {
    for (amount, recipient, valid) in [
        (-1, Some(OverpaymentRecipient::TaxpayerAccount), false),
        (1, None, false),
        (0, None, true),
        (100_050, Some(OverpaymentRecipient::SpouseAccount), true),
        // Changing financial facts may leave a retained excess for correction.
        (i64::MAX, Some(OverpaymentRecipient::TaxpayerAccount), true),
    ] {
        let mut draft = complete(FederalStatus::Single);
        let pay = draft.payments.as_mut().unwrap();
        pay.credit_applied_next_year = amount;
        pay.credit_recipient = recipient;
        assert_eq!(draft.valid_stored(), valid);
    }
}
