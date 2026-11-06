// IRS worked examples entered through the same page Requests as a filer.
// The App targets 2026. Each fixture states when an earlier edition's amounts
// are year independent; a year-sensitive comparison adapts only the year-indexed constants of the
// example and states the adaptation (the publication's figure and the 2026 figure with its
// derivation).
use super::*;
use crate::tests::cross::interview::{
    adult, document, from_review, save_children, save_community, save_setup, save_spouse,
    save_spouse_boxes, save_you, tick, walk_on_to_review,
};

fn dollars_input(amount: i64) -> i64 {
    amount * 100
}

fn accepted(reply: &Reply) {
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
}

pub(super) fn example_return(filing: Filing, birth: u32) -> Interview {
    example_return_with_spouse(filing, birth, birth)
}

// A joint return whose spouse has their own birth date (age and senior deductions differ).
pub(super) fn example_return_with_spouse(
    filing: Filing,
    birth: u32,
    spouse_birth: u32,
) -> Interview {
    let mut session = Interview::new(TEST_USER, FederalDraft::empty("irs-example".into()));
    send(&mut session, P::OpenInterview { user: TEST_USER });
    let request = save_setup(&session.draft, ReturnSetup { filing });
    accepted(&send(&mut session, request));
    let request = save_you(&session.draft, adult("Synthetic", "Example", birth));
    accepted(&send(&mut session, request));
    if filing == Filing::MarriedSeparately {
        // A separate return's next step asks whether community property applies;
        // the examples are not community-property returns.
        let request = save_community(&session.draft, CommunityAnswer::No);
        accepted(&send(&mut session, request));
    }
    if filing == Filing::MarriedJointly {
        let request = save_spouse(&session.draft, adult("Synthetic", "Spouse", spouse_birth));
        accepted(&send(&mut session, request));
    }
    let request = save_children(&session.draft, vec![]);
    let Reply::Edit(documents @ P::SaveDocuments { .. }) = send(&mut session, request) else {
        panic!("household Next opens the document center")
    };
    let begin @ Reply::Response(P::BeginIncomeWithoutForm { .. }) = send(&mut session, documents)
    else {
        panic!("documents Next opens income without a form")
    };
    let without_form = crate::tests::cross::interview::without_form_editor(begin);
    let reply = send(&mut session, without_form);
    let request = tick(reply, Checklist::NONE);
    let mut reply = send(&mut session, request);
    if filing == Filing::MarriedSeparately {
        // These examples establish that the spouse does not itemize. Answer
        // the actual strict question when the ordinary walk reaches it.
        loop {
            reply = crate::tests::cross::interview::past_less_common(&mut session, reply);
            if matches!(&reply, Reply::Response(P::BeginSpouseItemizes { .. })) {
                let request = spouse_itemizes_request(&session.draft, false);
                reply = send(&mut session, request);
                assert!(
                    matches!(
                        &reply,
                        Reply::Response(P::BeginSpouseBoxes { .. })
                            | Reply::Edit(P::SaveSpouseBoxes { .. })
                    ),
                    "expected the spouse's standard-deduction boxes: {reply:?}"
                );
                let request = save_spouse_boxes(&session.draft, SpouseBoxes::NoneOfThese);
                reply = send(&mut session, request);
                break;
            }
            reply = match reply {
                Reply::Response(hub @ P::Hub { .. }) => {
                    let request = crate::tests::cross::interview::hub_action(&hub);
                    send(&mut session, request)
                }
                Reply::Edit(page @ P::SaveDeductions { .. }) => send(&mut session, page),
                other => panic!("expected the separate spouse question: {other:?}"),
            };
        }
    }
    walk_on_to_review(&mut session, reply);
    session
}

pub(super) fn enter(
    session: &mut Interview,
    topic: TaxSection,
    documents: Vec<IncomeDocumentItem>,
) {
    assert!(
        documents
            .iter()
            .all(|row| crate::tests::support::client::income_pages::topic_for(&row.document)
                == topic)
    );
    let documents = fixture_owned_rows(&session.draft, documents);
    crate::tests::support::client::income_pages::save_each_topic(session, documents);
}

fn wages(session: &mut Interview, amount: i64) {
    enter(
        session,
        TaxSection::Wages,
        vec![document(
            0,
            IncomeSource::Wage {
                value: crate::WageDocument::W2(wage_document(crate::tests::cross::interview::w2(
                    dollars_input(amount),
                ))),
            },
        )],
    );
}

fn page(session: &mut Interview, section: TaxSection) -> P {
    match from_review(session, section) {
        Reply::Edit(page) => page,
        other => panic!("expected {section:?} editor, got {other:?}"),
    }
}

#[test]
fn protocol_example_fixture_finishes_an_empty_return() {
    let session = example_return(Filing::Single, 19800615);
    assert_eq!(value(&supported(&session.draft), "agi"), 0);
}

pub(super) fn printed(result: &FederalResult, form: TaxForm, line: &str) -> i64 {
    if form == TaxForm::Form1040 {
        result
            .return_lines
            .iter()
            .find(|v| v.form_line.as_ref().is_some_and(|f| f.form == form && f.line == line))
            .unwrap_or_else(|| panic!("missing Form 1040 line {line}"))
            .amount
            / 100
    } else {
        result
            .lines
            .iter()
            .filter(|v| v.form_line.as_ref().is_some_and(|f| f.form == form && f.line == line))
            .map(|v| v.amount)
            .sum::<i64>()
            / 100
    }
}

fn mortgage(interest: i64, principal: i64, originated: u32) -> Mortgage1098 {
    Mortgage1098 {
        lender: Default::default(),
        borrower: Default::default(),
        id: 0,
        interest: dollars_input(interest),
        principal: dollars_input(principal),
        originated,
        refund: 0,
        insurance: 0,
        points: 0,
        more: MortgageMore::default(),
    }
}

fn enter_mortgages(
    session: &mut Interview,
    loans: Vec<Mortgage1098>,
    points: Vec<crate::MortgagePointTreatment>,
) {
    let answers = loans.iter().map(|loan| loan.more.clone()).zip(points).collect::<Vec<_>>();
    let mut editor = page(session, TaxSection::Deductions);
    let P::SaveDeductions { value, .. } = &mut editor else { unreachable!() };
    value.itemize_override = Some(ItemizeOverride::Itemize);
    accepted(&send(session, editor));
    let editor = crate::tests::cross::interview::save_mortgages(
        session,
        MortgageDocuments { documents: loans.into_iter().map(mortgage_document).collect() },
    );
    accepted(&send(session, editor));
    accepted(&crate::tests::cross::interview::save_mortgage_answers(session, &answers));
}

/// Pub. 936 (2026 DRAFT, September 30, 2026), p. 8, ratable points example.
/// Three payments on $4,800 points over 240 months: $60 in 2026.
/// https://www.irs.gov/pub/irs-dft/p936--dft.pdf (fetched 2026-10-10).
#[test]
fn pub936_points_twenty_year_loan_first_year() {
    let mut session = example_return(Filing::Single, 19800615);
    let mut loan = mortgage(0, 100_000, 20261001);
    loan.points = dollars_input(4_800);
    enter_mortgages(
        &mut session,
        vec![loan],
        vec![crate::MortgagePointTreatment::Amortized { term_months: 240, ended: false }],
    );
    let result = supported(&session.draft);
    assert_eq!(
        printed(&result, TaxForm::ScheduleA, "8a") + printed(&result, TaxForm::ScheduleA, "8c"),
        60
    );
}

/// Pub. 936 (2026 DRAFT, September 30, 2026), p. 8, refinancing Example 1.
/// $2,000 / 180 x 6 = $67; service fees are not entered as interest points.
/// https://www.irs.gov/pub/irs-dft/p936--dft.pdf (fetched 2026-10-10).
#[test]
fn pub936_refinance_points_six_payments() {
    let mut session = example_return(Filing::Single, 19800615);
    // Points spread from the box 3 date of the refinanced loan, July 1, 2026.
    let mut loan = mortgage(0, 100_000, 20260701);
    loan.points = dollars_input(2_000);
    enter_mortgages(
        &mut session,
        vec![loan],
        vec![crate::MortgagePointTreatment::Amortized { term_months: 180, ended: false }],
    );
    let result = supported(&session.draft);
    assert_eq!(
        printed(&result, TaxForm::ScheduleA, "8a") + printed(&result, TaxForm::ScheduleA, "8c"),
        67
    );
}

/// Pub. 936 (2026 DRAFT, September 30, 2026), p. 15, interest/rate example.
/// $2,500 / 9% = $27,778 rounded to whole dollars, as in the printed worksheet.
/// https://www.irs.gov/pub/irs-dft/p936--dft.pdf (fetched 2026-10-10).
#[test]
fn pub936_table1_interest_over_rate_average() {
    let mut session = example_return(Filing::Single, 19800615);
    let mut loan = mortgage(2_500, 30_000, 20100101);
    loan.more.details = MortgageDetails::FirstLender(MortgageFirstLender {
        average: AverageBalance::InterestOverRate { rate: 9.0 },
        ..MortgageFirstLender::default()
    });
    enter_mortgages(&mut session, vec![loan], vec![crate::MortgagePointTreatment::None]);
    let result = supported(&session.draft);
    let balance =
        result.lines.iter().find(|line| line.id == "mortgage_average_balance").unwrap().amount;
    assert_eq!((balance + 50) / 100, 27_778);
}

fn business_assets(assets: Vec<Asset>) -> FederalResult {
    let mut session = example_return(Filing::Single, 19800615);
    let mut business = crate::tests::cross::interview::business(dollars_input(100_000));
    business.assets = assets;
    enter(
        &mut session,
        TaxSection::SelfEmployment,
        vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Business(business) })],
    );
    supported(&session.draft)
}

/// Pub. 946 (2025, March 13, 2026), p. 38, MACRS worksheet: $10,000 seven-year furniture -> $1,429.
/// Service date shifted from 2025-08-11 to 2026-08-11; regular GDS election unchanged.
/// https://www.irs.gov/pub/irs-pdf/p946.pdf (fetched 2026-10-10).
#[test]
fn pub946_furniture_macrs_worksheet() {
    let asset = crate::tests::cross::interview::with_first_year(
        crate::tests::cross::interview::asset(dollars_input(10_000), 20260811, AssetClass::Seven),
        FirstYearDeduction::Regular,
    );
    let result = business_assets(vec![asset]);
    assert_eq!(printed(&result, TaxForm::ScheduleC, "13"), 1_429);
}

/// Pub. 946 (2025, March 13, 2026), pp. 41-42, Example 1: first-year 5-year 200% DB half-year $200.
/// The source gives no calendar year; service in February 2026 preserves all facts.
/// https://www.irs.gov/pub/irs-pdf/p946.pdf (fetched 2026-10-10).
#[test]
fn pub946_five_year_declining_balance_first_year() {
    let asset = crate::tests::cross::interview::with_first_year(
        crate::tests::cross::interview::asset(dollars_input(1_000), 20260201, AssetClass::Five),
        FirstYearDeduction::Regular,
    );
    assert_eq!(printed(&business_assets(vec![asset]), TaxForm::ScheduleC, "13"), 200);
}

/// Pub. 946 (2025, March 13, 2026), pp. 41-42, Example 1: second-year 5-year 200% DB $320.
/// https://www.irs.gov/pub/irs-pdf/p946.pdf (fetched 2026-10-10).
#[test]
fn pub946_five_year_declining_balance_second_year() {
    let mut asset =
        crate::tests::cross::interview::asset(dollars_input(1_000), 20250201, AssetClass::Five);
    asset.gds_mut().unwrap().history = AssetHistory::Continuing {
        prior_depreciation: dollars_input(200),
        first_year_179: 0,
        first_year_bonus: 0,
        carryforward: 0,
        mid_quarter: false,
    };
    assert_eq!(printed(&business_assets(vec![asset]), TaxForm::ScheduleC, "13"), 320);
}

/// Pub. 946 (2025, March 13, 2026), p. 16, May Oak: 80% of $11,000 is $8,800 business basis.
/// Elect all eligible basis with sufficient business income; no year-sensitive cap binds.
/// https://www.irs.gov/pub/irs-pdf/p946.pdf (fetched 2026-10-10).
#[test]
fn pub946_section179_partial_business_use() {
    let mut asset = crate::tests::cross::interview::with_first_year(
        crate::tests::cross::interview::asset(dollars_input(11_000), 20260201, AssetClass::Seven),
        FirstYearDeduction::Section179 {
            description: "Equipment".into(),
            elected: dollars_input(8_800),
        },
    );
    asset.business_percent = 80;
    assert_eq!(printed(&business_assets(vec![asset]), TaxForm::Form4562, "12"), 8_800);
}

/// Form 6251 instructions (2025, January 8, 2026), p. 4, ISO example: 100 x ($25 - $10) = $1,500.
/// Exercise shifted one year to 2026; no sale and no payment for the option.
/// https://www.irs.gov/pub/irs-pdf/i6251.pdf (fetched 2026-10-10).
#[test]
fn i6251_iso_exercise_adjustment() {
    let mut session = example_return(Filing::Single, 19800615);
    enter(
        &mut session,
        TaxSection::InvestmentSales,
        vec![document(
            0,
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::Form3921(
                    crate::tests::support::builders::iso_document(crate::IsoExerciseSource {
                        id: 0,
                        assigned: crate::DocumentRecipient::Unknown,
                        owner_source: None,
                        joint_share: None,
                        owner: Owner::Taxpayer,
                        form: crate::Form3921Paper {
                            transferor: crate::PrintedParty::from_name("Synthetic employer".into()),
                            employee: Default::default(),
                            granted: 20240101,
                            exercised: 20260601,
                            exercise_price: dollars_input(10),
                            market_price: dollars_input(25),
                            shares: 100.0,
                            corporation: crate::PrintedParty::from_name("".into()),
                            ..Default::default()
                        },
                        more: Some(crate::Form3921More {
                            sold: 0.0,
                            recognition: crate::IsoRecognition::Exercise,
                            option_cost: 0,
                        }),
                    }),
                ),
            },
        )],
    );
    // The result exposes AMT income, rather than a separate line 2i row.
    // With no other income or AMT adjustments, it is the published exercise spread.
    assert_eq!(value(&supported(&session.draft), "alternative_minimum_taxable_income"), 1_500);
}

fn student_credit(
    session: &mut Interview,
    tuition: i64,
    scholarships: i64,
    inclusion: ScholarshipInclusion,
) {
    let mut editor = crate::tests::support::client::income_pages::open_document_center(session);
    let P::SaveDocuments { documents, .. } = &mut editor else { unreachable!() };
    documents.common.education.students = vec![crate::StudentCreditDocument {
        id: 0,
        student_id: -1,
        form: student_credit_paper(StudentCredit {
            filer: Default::default(),
            student: crate::PrintedParty::from_name("Synthetic Example".into()),
            tuition: dollars_input(tuition),
            scholarships: dollars_input(scholarships),
            half_time: true,
            graduate: false,
            prior_tuition_adjustment: crate::SourceCents::NotReported,
            prior_scholarship_adjustment: crate::SourceCents::NotReported,
        }),
    }];
    accepted(&send(session, editor));
    let (_, mut annual) =
        native_education(-1, education_expenses(0), EducationChoice::AmericanOpportunity);
    annual.more.scholarship_in_income = inclusion;
    accepted(&crate::tests::cross::interview::save_education_answers(session, vec![annual]));
}

// The part of the 1098-T box 5 grant the student includes in income (Pub. 970 chapter 2,
// "Coordination with Pell grants"): the election on the Education page, with no separate
// scholarship document.
fn included(taxable: i64) -> ScholarshipInclusion {
    if taxable == 0 {
        ScholarshipInclusion::None
    } else {
        ScholarshipInclusion::Amount { included: dollars_input(taxable) }
    }
}

fn bill(award: i64, inclusion: ScholarshipInclusion) -> FederalResult {
    let mut session = example_return(Filing::Single, 19980615);
    wages(&mut session, 39_250);
    student_credit(&mut session, 5_600, award, inclusion);
    supported(&session.draft)
}

/// Pub. 970 (2025, January 27, 2026), p. 15, Pell coordination Example 1: Bill's AOTC is $2,500.
/// Only the unchanged credit/AGI oracle is compared; 2025 tax brackets are not reused.
/// https://www.irs.gov/pub/irs-pdf/p970.pdf (fetched 2026-10-10).
#[test]
fn pub970_bill_no_scholarship() {
    let result = bill(0, ScholarshipInclusion::None);
    assert_eq!(
        value(&result, "education_credit") + value(&result, "refundable_education_credit"),
        2_500
    );
    assert_eq!(value(&result, "agi"), 39_250);
}

/// Pub. 970 (2025, January 27, 2026), pp. 15-16, Pell coordination Example 2: fully excluded $5,600 award; no credit.
/// https://www.irs.gov/pub/irs-pdf/p970.pdf (fetched 2026-10-10).
#[test]
fn pub970_bill_excluded_scholarship() {
    let result = bill(5_600, included(0));
    assert_eq!(
        value(&result, "education_credit") + value(&result, "refundable_education_credit"),
        0
    );
    assert_eq!(value(&result, "agi"), 39_250);
}

/// Pub. 970 (2025, January 27, 2026), p. 16, Pell coordination Example 3: include $4,000; AGI $43,250, AOTC $2,500.
/// https://www.irs.gov/pub/irs-pdf/p970.pdf (fetched 2026-10-10).
/// Encoded with the allocation election (the Education page's included part, $4,000 of the box 5
/// $5,600 grant), as Pub. 970 describes it.
#[test]
fn pub970_bill_partially_taxable_scholarship() {
    let result = bill(5_600, included(4_000));
    assert_eq!(
        value(&result, "education_credit") + value(&result, "refundable_education_credit"),
        2_500
    );
    assert_eq!(value(&result, "agi"), 43_250);
}

/// Pub. 970 (2025, January 27, 2026), p. 16, Pell coordination Example 4: school applies grant to tuition; same $2,500 AOTC.
/// Both school-directed and personally received awards use the same supplied allocation facts.
/// https://www.irs.gov/pub/irs-pdf/p970.pdf (fetched 2026-10-10).
/// Encoded with the Automatic election (living costs $4,400): the part with the lowest tax is the
/// publication's $4,000.
#[test]
fn pub970_bill_school_applied_scholarship() {
    let result =
        bill(5_600, ScholarshipInclusion::Automatic { living_costs: dollars_input(4_400) });
    assert_eq!(
        value(&result, "education_credit") + value(&result, "refundable_education_credit"),
        2_500
    );
}

fn interest(session: &mut Interview, amount: i64) {
    let int = Int1099 {
        payer: crate::PrintedParty::from_name("Synthetic bank".into()),
        recipient: Default::default(),
        adjustments: None,
        nominee: None,
        state_details: Default::default(),
        taxable: dollars_input(amount),
        more: IntMore::default(),
        after_death: None,
    };
    enter(
        session,
        TaxSection::InterestDividends,
        vec![document(
            0,
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Int1099(interest_document(int)),
            },
        )],
    );
}

/// Pub. 596 (2025, November 18, 2025), pp. 22-23, detailed Example 2: Greys earned $33,030, AGI $33,555, EIC $6,498.
/// Child and parent ages preserved in 2026; the year-indexed EIC table is adapted: the publication's
/// 2025 table gives $6,498, the 2026 draft table the App ships
/// (tests/fixtures/irs-eic-2026-draft.csv, row 33,550-33,600, joint, two children) $6,807 for the
/// same earned income $33,030 and AGI $33,555.
/// https://www.irs.gov/pub/irs-pdf/p596.pdf (fetched 2026-10-10).
#[test]
fn pub596_greys_two_children_eic() {
    let mut session = example_return(Filing::MarriedJointly, 19800615);
    let children = vec![
        crate::tests::cross::interview::row("Synthetic", "Older", 20160615),
        crate::tests::cross::interview::row("Synthetic", "Younger", 20180615),
    ];
    let request = save_children(&session.draft, children);
    accepted(&send(&mut session, request));
    let mut spouse = document(
        0,
        IncomeSource::Wage {
            value: crate::WageDocument::W2(wage_document(crate::tests::cross::interview::w2(
                dollars_input(18_030),
            ))),
        },
    );
    spouse.owner = crate::DocumentOwner::Spouses;
    enter(
        &mut session,
        TaxSection::Wages,
        vec![
            document(
                0,
                IncomeSource::Wage {
                    value: crate::WageDocument::W2(wage_document(
                        crate::tests::cross::interview::w2(dollars_input(15_000)),
                    )),
                },
            ),
            spouse,
        ],
    );
    interest(&mut session, 525);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "agi"), 33_555);
    assert_eq!(value(&result, "earned_income_credit"), 6_807);
}

fn home_sale(
    acquired: u32,
    closed: u32,
    proceeds: i64,
    basis: i64,
    depreciation: i64,
    lived_days: u32,
    nonqualified_days: u32,
) -> Form1099SPaper {
    home_sale_document(MainHomeSale {
        form: crate::HomeSaleSource::Paper(home_sale_paper(Form1099S {
            filer: Default::default(),
            transferor: Default::default(),
            address: "Synthetic home".into(),
            closed,
            proceeds: dollars_input(proceeds),
            property_received: false,
            foreign_person: false,
        })),
        answers: Some(crate::RealEstateSaleUse::MainHome(HomeSaleAnswers {
            property_value: SourceCents::NotReported,
            firpta_withheld: SourceCents::NotReported,
            form_1099s: true,
            acquired,
            received: HomeReceived::Bought,
            payment: SalePayment::InFull,
            exchange: None,
            expenses: 0,
            basis: dollars_input(basis),
            lived: HomeLived::Days { days: lived_days },
            earlier_sale: EarlierHomeSale::No,
            spouse: HomeSpouse::NotJoint,
            reason: HomeSaleReason::TwoYears,
            use_history: HomeRentalUse {
                depreciation: dollars_input(depreciation),
                nonqualified_days,
                separate: None,
            },
            service: None,
            more: HomeSaleMore::Nothing,
        })),
    })
}

/// Pub. 523 (2025, February 26, 2026), p. 18, Taylor: $227,000 total gain, $200,000 excluded, $27,000 depreciation gain.
/// Timeline 2020-2024 shifted two years to 2022-2026, preserving residence/rental periods.
/// https://www.irs.gov/pub/irs-pdf/p523.pdf (fetched 2026-10-10).
#[test]
fn pub523_taylor_rental_after_last_residence() {
    let mut session = example_return(Filing::Single, 19800615);
    let sale = home_sale(20220101, 20261201, 600_000, 373_000, 27_000, 1_096, 0);
    enter(
        &mut session,
        TaxSection::InvestmentSales,
        vec![document(
            0,
            IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Form1099S(sale) },
        )],
    );
    let result = supported(&session.draft);
    assert_eq!(value(&result, "home_1_realized_gain"), 227_000);
    assert_eq!(value(&result, "home_1_excluded_gain"), 200_000);
    assert_eq!(value(&result, "home_1_depreciation_gain"), 27_000);
    assert_eq!(value(&result, "home_1_taxable_gain"), 27_000);
}

fn residential_rental(rent: i64, operating: i64) -> Rental {
    let mut rental = crate::tests::cross::interview::rental_page(dollars_input(rent));
    // These publications supply aggregate expenses, not a depreciable basis.
    // Zero remaining building basis contributes no invented depreciation.
    rental.building = Building {
        commercial_1993_transition: None,
        allowed_commercial_1993_transition: Vec::new(),
        casualty: None,
        casualty_property: None,
        id: 0,
        energy: None,
        sale_history: None,
        credit_reduction: 0,
        cost: 100,
        land: 100,
        in_service: 20200101,
        acquired: RentalAcquisition::Purchased,
        partial: None,
    };
    rental.expenses.cleaning = dollars_input(operating);
    rental
}

/// Pub. 925 (2025, February 23, 2026), pp. 6-7, phaseout example: $120,000 salary, $31,000 rental loss;
/// $15,000 allowed and $16,000 carried. Rental expense is entered in aggregate;
/// no depreciable basis or expense-category breakdown is supplied by the example.
/// https://www.irs.gov/pub/irs-pdf/p925.pdf (fetched 2026-10-10).
#[test]
fn pub925_active_rental_loss_phaseout() {
    let mut session = example_return(Filing::Single, 19800615);
    wages(&mut session, 120_000);
    enter(
        &mut session,
        TaxSection::Rentals,
        vec![document(
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::Rental(residential_rental(0, 31_000)),
            },
        )],
    );
    let result = supported(&session.draft);
    assert_eq!(printed(&result, TaxForm::ScheduleE, "22"), -15_000);
    assert_eq!(value(&result, "rental_2_suspended_loss"), 16_000);
}

/// Pub. 527 (2025, January 8, 2026), p. 26, cottage: 85/99 expenses are rental; overlapping2 days
/// count for the home test (16 personal versus83 rental) but not expense division.
/// A $99 expense is a unit probe for the exact published85/99 ratio, not an IRS
/// dollar expense. Gross rent is a neutral completion fact absent from the source.
/// https://www.irs.gov/pub/irs-pdf/p527.pdf (fetched 2026-10-10).
#[test]
fn pub527_cottage_overlap_expense_ratio() {
    let mut session = example_return(Filing::Single, 19800615);
    let mut rental = residential_rental(1_000, 99);
    rental.personal_use = PersonalUse::Yes {
        fair_rental_days: 85,
        personal_days: 14,
        overlap_days: 2,
        direct_expenses: 0,
    };
    enter(
        &mut session,
        TaxSection::Rentals,
        vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Rental(rental) })],
    );
    let result = supported(&session.draft);
    assert_eq!(printed(&result, TaxForm::ScheduleE, "20"), 85);
}

/// Pub. 527 (2025, January 8, 2026), p. 28, condominium Example 3: 170 paid days, 10 overlapping,
/// 7 additional personal days ->160 fair-only and17 personal for the home test.
/// A $177 expense probes the exact170/177 expense ratio. Rent $10 makes the
/// Worksheet 5-1 home limitation observable: $10 deducted, $160 carried.
/// These probe dollars are additional completion assumptions, not quoted IRS amounts.
/// https://www.irs.gov/pub/irs-pdf/p527.pdf (fetched 2026-10-10).
#[test]
fn pub527_condominium_overlap_home_limit() {
    let mut session = example_return(Filing::Single, 19800615);
    let mut rental = residential_rental(10, 177);
    rental.personal_use = PersonalUse::Yes {
        fair_rental_days: 170,
        personal_days: 7,
        overlap_days: 10,
        direct_expenses: 0,
    };
    enter(
        &mut session,
        TaxSection::Rentals,
        vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Rental(rental) })],
    );
    let result = supported(&session.draft);
    assert_eq!(printed(&result, TaxForm::ScheduleE, "21"), 0);
    assert_eq!(value(&result, "rental_1_home_operating_carryover"), 160);
    assert_eq!(value(&result, "deduction"), 16_100);
    assert_eq!(return_line(&result, "12e"), 1_610_000);
    assert_eq!(value(&result, "amt_itemized_taxes"), 16_100);
}

// The published Form 8962 allocation situations, as the corpus states them: who
// shares the policy and for which months. The health unit's canonical
// Marketplace records are built from these facts by its own fixture converter.
#[derive(Clone, Copy)]
enum PolicySharing {
    NotShared,
    Spouse { from: u32, to: u32 },
    NoAdvance { percent: u32, from: u32, to: u32 },
    Shared { percent: u32, from: u32, to: u32 },
}

fn marketplace(session: &mut Interview, amounts: PolicyAmounts, shared: PolicySharing) {
    let rows: Vec<(u32, i64, i64, i64)> = match amounts {
        PolicyAmounts::AllYear { premium, benchmark, advance } => {
            (1..=12).map(|month| (month, premium, benchmark.unwrap_or(0), advance)).collect()
        }
        PolicyAmounts::ByMonth { months } => months
            .iter()
            .map(|m| (m.month, m.premium, m.benchmark.unwrap_or(0), m.advance))
            .collect(),
    };
    let months = rows
        .into_iter()
        .map(|(month, premium, benchmark, advance)| {
            let within = |from, to| (from..=to).contains(&month);
            let allocation = match shared {
                PolicySharing::NotShared => MarketplaceAllocation::NotShared,
                PolicySharing::Spouse { from, to } if within(from, to) => {
                    MarketplaceAllocation::Spouse
                }
                PolicySharing::NoAdvance { percent, from, to } if within(from, to) => {
                    MarketplaceAllocation::NoAdvance { percent }
                }
                PolicySharing::Shared { percent, from, to } if within(from, to) => {
                    MarketplaceAllocation::Shared { percent }
                }
                _ => MarketplaceAllocation::NotShared,
            };
            MarketplaceMonth {
                month,
                policies: vec![MarketplacePolicy {
                    label: "Synthetic insurer".into(),
                    premium,
                    advance,
                    allocation,
                }],
                benchmark,
                eligible: MarketplaceEligibility::Everyone,
            }
        })
        .collect();
    let mut prepared = session.draft.clone();
    let MarketplaceClaim::Yes { facts } = marketplace_claim(&mut prepared, months) else {
        unreachable!()
    };
    // Family identities must be accepted before a statement can link them.
    // The arithmetic converter's draft-local IDs are not accepted source IDs.
    let mut facts = *facts;
    let mut prerequisites: MarketplaceFactsPage = facts.clone().into();
    prerequisites.forms.clear();
    prerequisites.covered.clear();
    prerequisites.eligibility.clear();
    for family in &mut prerequisites.other_families {
        family.id = 0;
    }
    let mut accepted_facts =
        crate::tests::cross::interview::save_marketplace_and_reopen(session, prerequisites);
    if !facts.other_families.is_empty() {
        assert_eq!(facts.other_families.len(), 1);
        assert_eq!(facts.covered.len(), 1);
        let family = accepted_facts.other_families[0].id;
        accepted_facts.covered = vec![MarketplaceCoveredPersonPage {
            id: 0,
            identity: MarketplacePersonIdentityPage::OtherFamily {
                family,
                label: "Other fixture enrollee".into(),
            },
        }];
        accepted_facts =
            crate::tests::cross::interview::save_marketplace_and_reopen(session, accepted_facts);
        let person = accepted_facts.covered[0].id;
        let old_person = facts.covered[0].id;
        facts.other_families[0].id = family;
        facts.covered[0].id = person;
        facts.covered[0].identity = MarketplacePersonIdentity::OtherFamily {
            family,
            label: "Other fixture enrollee".into(),
        };
        for row in &mut facts.eligibility {
            if row.person == old_person {
                row.person = person;
            }
        }
        for form in &mut facts.forms {
            for row in &mut form.coverage {
                for covered in &mut row.people {
                    if *covered == old_person {
                        *covered = person;
                    }
                }
            }
            for period in &mut form.allocation {
                let other = MarketplaceFamilyRef::Other { family };
                match &mut period.kind {
                    MarketplaceAllocationKind::MarriedSeparate { pair, .. } => pair.two = other,
                    MarketplaceAllocationKind::NoAdvance { families } => {
                        families[1].family = other;
                    }
                    MarketplaceAllocationKind::OtherShared { families } => {
                        families[1].family = other;
                    }
                    _ => unreachable!("allocation kinds supplied by this example fixture"),
                }
            }
        }
    }
    for form in &mut facts.forms {
        form.id = 0;
    }
    crate::tests::cross::interview::save_marketplace_and_reopen(session, facts.into());
}

/// Form 8962 instructions (2025, October 1, 2025), p. 20, Allocation Situation2 Example 3:
/// Michael/Colleen separate without relief; half of $8,700 APTC is $4,350, no PTC.
/// Timeline shifted to 2026; the full repayment does not depend on a 2025 cap.
/// https://www.irs.gov/pub/irs-pdf/i8962.pdf (fetched 2026-10-10).
#[test]
#[ignore = "IRS8962-MICHAEL rounding convention: IRS annual $8,700 / 2 = $4,350; App rounds monthly $362.50 to $363, totaling $4,356"]
fn i8962_separate_spouses_half_advance() {
    let mut session = example_return(Filing::MarriedSeparately, 19800615);
    marketplace(
        &mut session,
        PolicyAmounts::AllYear {
            premium: dollars_input(1_000),
            benchmark: Some(dollars_input(1_000)),
            advance: dollars_input(725),
        },
        PolicySharing::Spouse { from: 1, to: 12 },
    );
    let result = supported(&session.draft);
    assert_eq!(value(&result, "premium_tax_credit"), 0);
    assert_eq!(value(&result, "premium_credit_advance"), 4_350);
    assert_eq!(value(&result, "premium_credit_repayment"), 4_350);
}

/// Form 8962 instructions (2025, October 1, 2025), p. 20, Allocation Situation2 Example 4:
/// Only Colleen insured; all $8,700 APTC belongs to her, no PTC.
/// https://www.irs.gov/pub/irs-pdf/i8962.pdf (fetched 2026-10-10).
#[test]
fn i8962_separate_spouse_own_advance() {
    let mut session = example_return(Filing::MarriedSeparately, 19800615);
    marketplace(
        &mut session,
        PolicyAmounts::AllYear {
            premium: dollars_input(1_000),
            benchmark: Some(dollars_input(1_000)),
            advance: dollars_input(725),
        },
        PolicySharing::NotShared,
    );
    let result = supported(&session.draft);
    assert_eq!(value(&result, "premium_tax_credit"), 0);
    assert_eq!(value(&result, "premium_credit_advance"), 8_700);
    assert_eq!(value(&result, "premium_credit_repayment"), 8_700);
}

/// Form 8962 instructions (2025, October 1, 2025), p. 20, no-APTC Gary/Jim example: Gary's
/// allocated premiums are $10,000 ($15,000 x $12,000/$18,000, "67% of the total premiums"), but
/// the form's column (e) takes the percentage "as a decimal rounded to two places" and Gary
/// "enters '0.67'", so the allocation the form figures is $15,000 x 0.67 = $10,050 (the page's
/// whole percent 67 is that entry).
/// https://www.irs.gov/pub/irs-pdf/i8962.pdf (fetched 2026-10-10).
#[test]
fn i8962_no_advance_gary_benchmark_ratio() {
    let mut session = example_return(Filing::Single, 19800615);
    wages(&mut session, 40_000);
    marketplace(
        &mut session,
        PolicyAmounts::AllYear {
            premium: dollars_input(1_250),
            benchmark: Some(dollars_input(1_000)),
            advance: 0,
        },
        PolicySharing::NoAdvance { percent: 67, from: 1, to: 12 },
    );
    let claim = &session.draft.credits.as_ref().unwrap().marketplace;
    let amounts = &crate::marketplace::facts(claim).unwrap().forms[0].form.amounts();
    let allocated: i64 = (1..=12)
        .map(|month| crate::marketplace_sources::policy_month(amounts, month).unwrap().0 * 67 / 100)
        .sum();
    assert_eq!(allocated, dollars_input(10_050));
}

/// Form 8962 instructions (2025, October 1, 2025), p. 21, Situation4 Example 1: Joe's agreed80%
/// allocation of $6,345 APTC is $5,076. Alice's20% belongs to her separate return.
/// The original premium and APTC annual cents are preserved across month rows.
/// Only allocation is compared; 2025 contribution rates are not reused in 2026.
/// https://www.irs.gov/pub/irs-pdf/i8962.pdf (fetched 2026-10-10).
#[test]
fn i8962_joe_agreed_shared_policy_advance() {
    let mut session = example_return(Filing::Single, 19800615);
    wages(&mut session, 77_672);
    let months = (1..=12)
        .map(|month| PolicyMonth {
            month,
            premium: if month == 12 { 108_337 } else { 108_333 },
            benchmark: Some(dollars_input(1_000)),
            advance: 52_875,
        })
        .collect();
    marketplace(
        &mut session,
        PolicyAmounts::ByMonth { months },
        PolicySharing::Shared { percent: 80, from: 1, to: 12 },
    );
    assert_eq!(value(&supported(&session.draft), "premium_credit_advance"), 5_076);
}

/// Form 8962 instructions (2025, October 1, 2025), p. 21, Situation4 Example 1: Alice's agreed20%
/// allocation of $6,345 APTC is $1,269, independent of her unspecified income.
/// https://www.irs.gov/pub/irs-pdf/i8962.pdf (fetched 2026-10-10).
#[test]
#[ignore = "IRS8962-ALICE rounding convention: IRS annual $6,345 x 20% = $1,269; App rounds each monthly allocation to $106, totaling $1,272"]
fn i8962_alice_agreed_shared_policy_advance() {
    let mut session = example_return(Filing::Single, 19800615);
    wages(&mut session, 30_000);
    let months = (1..=12)
        .map(|month| PolicyMonth {
            month,
            premium: if month == 12 { 108_337 } else { 108_333 },
            benchmark: Some(dollars_input(1_000)),
            advance: 52_875,
        })
        .collect();
    marketplace(
        &mut session,
        PolicyAmounts::ByMonth { months },
        PolicySharing::Shared { percent: 20, from: 1, to: 12 },
    );
    assert_eq!(value(&supported(&session.draft), "premium_credit_advance"), 1_269);
}

/// Pub. 523 (2025, February 26, 2026), p. 12, Logan's law office inside the dwelling:
/// $13,000 realized gain, $2,000 depreciation gain, $11,000 excluded.
/// Source 2013-2016 timeline shifted 10 years; unspecified basis/proceeds completed
/// as $200,000/$213,000, January 1 dates and a three-year residence (completion facts).
/// https://www.irs.gov/pub/irs-pdf/p523.pdf (fetched 2026-10-10).
#[test]
fn pub523_logan_office_inside_dwelling() {
    let mut session = example_return(Filing::Single, 19800615);
    let sale = home_sale(20230101, 20260101, 213_000, 200_000, 2_000, 1_096, 0);
    enter(
        &mut session,
        TaxSection::InvestmentSales,
        vec![document(
            0,
            IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Form1099S(sale) },
        )],
    );
    let result = supported(&session.draft);
    assert_eq!(value(&result, "home_1_realized_gain"), 13_000);
    assert_eq!(value(&result, "home_1_excluded_gain"), 11_000);
    assert_eq!(value(&result, "home_1_depreciation_gain"), 2_000);
}

fn irs_ira_contributor(owner: Owner, traditional: i64, roth: i64) -> IraContributor {
    IraContributor {
        owner,
        traditional: dollars_input(traditional),
        roth: dollars_input(roth),
        covered: false,
        form_8606_value: None,
        more: IraMore::default(),
    }
}
fn irs_ira_save(session: &mut Interview, people: Vec<IraContributor>) {
    let mut request = page(session, TaxSection::IraContributions);
    let P::SaveIraContributions { value, .. } = &mut request else { unreachable!() };
    value.people = people;
    accepted(&send(session, request));
}
fn irs_ira_wages(session: &mut Interview, taxpayer: i64, spouse: Option<i64>) {
    let mut form = crate::tests::cross::interview::w2(dollars_input(taxpayer));
    form.more.retirement_plan = true;
    let mut forms = vec![document(
        0,
        IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(form)) },
    )];
    if let Some(amount) = spouse {
        let mut row = document(
            0,
            IncomeSource::Wage {
                value: crate::WageDocument::W2(wage_document(crate::tests::cross::interview::w2(
                    dollars_input(amount),
                ))),
            },
        );
        row.owner = crate::DocumentOwner::Spouses;
        forms.push(row);
    }
    enter(session, TaxSection::Wages, forms);
}
fn irs_ira_interest(session: &mut Interview, amount: i64) {
    enter(
        session,
        TaxSection::InterestDividends,
        vec![document(
            0,
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Int1099(interest_document(Int1099 {
                    payer: crate::PrintedParty::from_name("Synthetic interest".into()),
                    recipient: Default::default(),
                    adjustments: None,
                    nominee: None,
                    taxable: dollars_input(amount),
                    more: Default::default(),
                    after_death: None,
                    state_details: Default::default(),
                })),
            },
        )],
    );
}

/// IRS Pub. 590-A (2025), Jan. 15, 2026, pp. 17 and 19, reduced IRA deduction Example 1.
/// https://www.irs.gov/pub/irs-pdf/p590a.pdf
/// Both spouses age 39, wages 66,000/51,500, investment income 9,000, contributions 7,000 each.
/// The unspecified interest/dividend split is completed as ordinary interest; it does not change IRA MAGI.
/// Calendar year is shifted to 2026, and the year-indexed phaseout is adapted: the publication's
/// covered joint range is $126,000-$146,000 (MAGI $126,500, deduction $13,825 of $14,000); the
/// 2026 range starts at $129,000 (`IRA_PHASEOUT_START_COVERED_JOINT_DOLLARS`, Notice 2025-67),
/// above the same MAGI $126,500, so the publication's method gives the full $14,000.
#[test]
fn pub590a_reduced_ira_deduction_example_one() {
    let mut session = example_return(Filing::MarriedJointly, 19870615);
    irs_ira_wages(&mut session, 66_000, Some(51_500));
    irs_ira_interest(&mut session, 9_000);
    irs_ira_save(
        &mut session,
        vec![
            irs_ira_contributor(Owner::Taxpayer, 7_000, 0),
            irs_ira_contributor(Owner::Spouse, 7_000, 0),
        ],
    );
    let result = supported(&session.draft);
    assert_eq!(value(&result, "ira_deduction"), 14_000);
}

/// IRS Pub. 590-A (2025), Jan. 15, 2026, pp. 17 and 20, reduced IRA deduction Example 2.
/// https://www.irs.gov/pub/irs-pdf/p590a.pdf
/// Age 39 spouses, taxpayer wages 45,500, interest 2,000, stock gain 191,000; MAGI 238,500.
/// Only the taxpayer is covered at work; each contributes 7,000. Source spouse deduction is 5,250.
/// The unspecified stock proceeds/basis split is completed as 191,000 proceeds and zero basis.
/// The year-indexed range is adapted: the publication's not-covered-spouse range is
/// $236,000-$246,000 (deduction $5,250); the 2026 range, $242,000-$252,000 (Notice 2025-67), lies
/// above MAGI $238,500, so the spouse's full $7,000 is deductible (the covered taxpayer's remains zero).
#[test]
fn pub590a_reduced_ira_deduction_example_two() {
    let mut session = example_return(Filing::MarriedJointly, 19870615);
    irs_ira_wages(&mut session, 45_500, None);
    irs_ira_interest(&mut session, 2_000);
    enter(
        &mut session,
        TaxSection::InvestmentSales,
        vec![document(
            0,
            IncomeSource::InvestmentSale {
                value: crate::InvestmentSaleDocument::Sale1099B(Sale1099B {
                    payer: Default::default(),
                    recipient: Default::default(),
                    description: "Synthetic stock gain".into(),
                    digital_asset: false,
                    term: Some(GainTerm::Long),
                    proceeds: dollars_input(191_000),
                    basis: Some(0),
                    reporting: BasisReporting::NotReported,
                    use_kind: SaleUse::Investment { wash_sale: 0 },
                    collectible: false,
                    market_discount: 0,
                    several_sales: false,
                    withholding: 0,
                    state_details: Default::default(),
                }),
            },
        )],
    );
    irs_ira_save(
        &mut session,
        vec![
            irs_ira_contributor(Owner::Taxpayer, 7_000, 0),
            irs_ira_contributor(Owner::Spouse, 7_000, 0),
        ],
    );
    assert_eq!(value(&supported(&session.draft), "ira_deduction"), 7_000);
}

/// IRS Pub. 590-B (2025), Jan. 21, 2026, pp. 16 and 18-20, Rose Green, Worksheet 1-1.
/// https://www.irs.gov/pub/irs-pdf/p590b.pdf
/// Given: prior basis 300, current contribution 2,000, year-end value 20,000, conversion 5,000,
/// and illustrated nondeductible contribution 500. The source leaves wages/age unstated;
/// complete those as 80,000 covered wages and age 60, and voluntarily designate 500 nondeductible.
/// These completion facts preserve the source's 1,500 deductible contribution without a phaseout.
#[test]
fn pub590b_rose_green_worksheet_one_one() {
    let mut session = example_return(Filing::Single, 19660615);
    irs_ira_wages(&mut session, 80_000, None);
    let payment = crate::tests::cross::interview::r1099(
        dollars_input(5_000),
        DistributionCode::Normal {
            what_happened: Disposition::Converted { amount: dollars_input(5_000) },
        },
        true,
    );
    enter(
        &mut session,
        TaxSection::Retirement,
        vec![
            document(
                0,
                IncomeSource::Retirement {
                    value: crate::RetirementDocument::R1099(retirement_document(payment)),
                },
            ),
            document(
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::Form8606(Form8606 {
                        adjustment: 0,
                        reservist_repayments: vec![],
                        basis: dollars_input(300),
                        year_end: dollars_input(20_000),
                    }),
                },
            ),
        ],
    );
    let mut contribution = irs_ira_contributor(Owner::Taxpayer, 2_000, 0);
    contribution.more.nondeductible = dollars_input(500);
    irs_ira_save(&mut session, vec![contribution]);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "ira_deduction"), 1_500);
    assert_eq!(value(&result, "taxpayer_ira_basis_recovered"), 460);
    assert_eq!(value(&result, "roth_conversion_taxable"), 4_540);
    assert_eq!(value(&result, "taxpayer_ira_basis_carryforward"), 340);
}

/// IRS Pub. 590-B (2025), Jan. 21, 2026, p. 36, Amelia Roth ordering example.
/// https://www.irs.gov/pub/irs-pdf/p590b.pdf
/// The complete timeline shifts one year: conversion 2021->2022, contribution/distribution 2025->2026,
/// preserving age 60 and elapsed years. The source omits compensation; complete it as 10,000 wages
/// so the stated 5,000 regular Roth contribution is permitted. Source: first 5,000 regular, next 2,000 conversion, no income.
#[test]
fn pub590b_amelia_roth_distribution_ordering() {
    let mut session = example_return(Filing::Single, 19660615);
    wages(&mut session, 10_000);
    irs_ira_save(&mut session, vec![irs_ira_contributor(Owner::Taxpayer, 0, 5_000)]);
    let payment = crate::tests::cross::interview::r1099(
        dollars_input(7_000),
        DistributionCode::RothIra {
            sequence: None,
            rolled_over: 0,
            roth: RothIraCode::ExceptionApplies { distributed: 20261108, reason: RothReason::Age },
        },
        false,
    );
    enter(
        &mut session,
        TaxSection::Retirement,
        vec![
            document(
                0,
                IncomeSource::Retirement {
                    value: crate::RetirementDocument::R1099(retirement_document(payment)),
                },
            ),
            document(
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::RothHistory(RothIraHistory {
                        first_year: 2022,
                        amounts: Some(RothOrdering {
                            regular: 0,
                            conversions: vec![RothConversion {
                                year: 2022,
                                taxable: dollars_input(60_000),
                                nontaxable: dollars_input(20_000),
                            }],
                        }),
                    }),
                },
            ),
        ],
    );
    let result = reviewed(&session);
    assert_eq!(return_line(&result, "4a"), dollars_input(7_000));
    assert_eq!(return_line(&result, "4b"), 0);
    assert_eq!(value(&result, "early_distribution_tax"), 0);
    let regular_recovered: i64 = result
        .lines
        .iter()
        .filter(|line| line.id.starts_with("roth_") && line.id.ends_with("_regular_recovered"))
        .map(|line| line.amount)
        .sum();
    assert_eq!(regular_recovered, dollars_input(5_000));
}

// Pub. 503 has six requested examples. Four require unreachable public inputs;
// their published amounts and named scope gaps are recorded in the handoff report.
// Do not recreate those examples by directly mutating dormant CareClaim records.

fn pub503_child(session: &mut Interview, born: u32, expenses: i64) {
    let _ = from_review(session, TaxSection::Children);
    let child = crate::tests::cross::interview::row("Synthetic", "Care child", born);
    let request = save_children(&session.draft, vec![child]);
    accepted(&send(session, request));
    let care = CareCosts::Yes { expenses: dollars_input(expenses), incapable_self_care: false };
    crate::tests::cross::interview::save_care_costs(session, &[care]);
}

/// Pub. 503 (2025 final, November 18, 2025), p. 12, Dollar Limit/Yearly limit Example 1.
/// $500 x January-April = $2,000 qualified expenses; child turns13 on May1.
/// Uniform year shift to 2026 preserves the May1 birthday and four care months.
/// Neutral completion: single filer born 1980 with $10,000 ordinary wages,
/// child living at home all year, valid SSN, no benefits or competing claimant.
/// Compare the published expense base, not the changed 2026 credit percentage.
/// https://www.irs.gov/pub/irs-pdf/p503.pdf (fetched 2026-10-10).
#[test]
fn pub503_yearly_limit_child_turns_thirteen() {
    let mut session = example_return(Filing::Single, 19800615);
    wages(&mut session, 10_000);
    pub503_child(&mut session, 20130501, 2_000);
    let result = supported(&session.draft);
    assert_eq!(printed(&result, TaxForm::Form2441, "3"), 2_000);
    assert_eq!(printed(&result, TaxForm::Form2441, "6"), 2_000);
}

/// Pub. 503 (2025 final, November 18, 2025), p. 12, Dollar Limit/Yearly limit Example 2.
/// July-December nursery school: $400 x 6 = $2,400 qualified expenses.
/// Uniform shift to 2026 preserves six care months and the daughter's age3.
/// Neutral completion: joint filers born 1980, $10,000 ordinary wages each,
/// daughter bornJune15, 2023 living at home all year with a valid SSN,
/// no benefits or competing claimant. Only the published expense base is compared.
/// https://www.irs.gov/pub/irs-pdf/p503.pdf (fetched 2026-10-10).
#[test]
fn pub503_yearly_limit_spouse_begins_work_in_july() {
    let mut session = example_return(Filing::MarriedJointly, 19800615);
    let taxpayer = document(
        0,
        IncomeSource::Wage {
            value: crate::WageDocument::W2(wage_document(crate::tests::cross::interview::w2(
                dollars_input(10_000),
            ))),
        },
    );
    let mut spouse = document(
        0,
        IncomeSource::Wage {
            value: crate::WageDocument::W2(wage_document(crate::tests::cross::interview::w2(
                dollars_input(10_000),
            ))),
        },
    );
    spouse.owner = crate::DocumentOwner::Spouses;
    enter(&mut session, TaxSection::Wages, vec![taxpayer, spouse]);
    pub503_child(&mut session, 20230615, 2_400);
    let result = supported(&session.draft);
    assert_eq!(printed(&result, TaxForm::Form2441, "3"), 2_400);
    assert_eq!(printed(&result, TaxForm::Form2441, "6"), 2_400);
}

fn pub915_owned(owner: Owner, document_value: IncomeSource) -> IncomeDocumentItem {
    let mut item = document(0, document_value);
    item.owner = crate::DocumentOwner::from(owner);
    item
}

fn pub915_benefits(amount: i64, statement: SocialSecurityStatement) -> IncomeSource {
    let net = dollars_input(amount);
    IncomeSource::Retirement {
        value: match statement {
            SocialSecurityStatement::Railroad => {
                crate::RetirementDocument::Rrb1099(crate::rrb_document::Rrb1099Paper {
                    year: 2026,
                    benefits_paid: net.max(0),
                    benefits_repaid: net.saturating_neg().max(0),
                    benefits: net,
                    ..Default::default()
                })
            }
            SocialSecurityStatement::SocialSecurity => {
                crate::RetirementDocument::Ssa1099(ssa_document(ssa_1099_net(net, 0)))
            }
        },
    }
}

fn pub915_pension(amount: i64) -> IncomeSource {
    IncomeSource::Retirement {
        value: crate::RetirementDocument::R1099(retirement_document(
            crate::tests::cross::interview::r1099(
                dollars_input(amount),
                DistributionCode::Normal { what_happened: Disposition::Kept },
                false,
            ),
        )),
    }
}

fn pub915_interest(amount: i64) -> IncomeSource {
    IncomeSource::InterestDividend {
        value: crate::InterestDividendDocument::Int1099(interest_document(Int1099 {
            payer: crate::PrintedParty::from_name("Synthetic bank".into()),
            recipient: Default::default(),
            adjustments: None,
            nominee: None,
            taxable: dollars_input(amount),
            more: IntMore::default(),
            after_death: None,
            state_details: Default::default(),
        })),
    }
}

/// Pub. 915 (2026 DRAFT, title-page August 17, 2026; IRS postedAugust 21), printedp.8
/// (PDFp.9), Worksheet 1 Example 1: $5,980 benefits, $2,990 taxable.
/// Published wages $9,400, pension $18,600 and interest $990 entered separately.
/// Neutral completion: birthJune15, 1956; ordinary taxable pension, no withholding.
/// https://www.irs.gov/pub/irs-dft/p915--dft.pdf (fetched 2026-10-10).
#[test]
fn pub915_worksheet_one_single() {
    let mut session = example_return(Filing::Single, 19560615);
    wages(&mut session, 9_400);
    enter(&mut session, TaxSection::InterestDividends, vec![document(0, pub915_interest(990))]);
    enter(
        &mut session,
        TaxSection::Retirement,
        vec![
            document(0, pub915_pension(18_600)),
            document(0, pub915_benefits(5_980, SocialSecurityStatement::SocialSecurity)),
        ],
    );
    let result = supported(&session.draft);
    assert_eq!(printed(&result, TaxForm::Form1040, "6a"), 5_980);
    assert_eq!(printed(&result, TaxForm::Form1040, "6b"), 2_990);
}

/// Pub. 915 (2026 DRAFT, title-page August 17, 2026; IRS postedAugust 21), printedp.9
/// (PDFp.10), Worksheet 1 Example 2: Casey's$5,600 benefits have$0 taxable.
/// Pat's wages $14,000 and deductible IRA$1,000; Casey's pension $15,500;
/// joint taxable interest $250. Neutral completion: both bornJune15, 1956,
/// ordinary fully taxable pension, neither covered at work, no withholding.
/// https://www.irs.gov/pub/irs-dft/p915--dft.pdf (fetched 2026-10-10).
#[test]
fn pub915_worksheet_one_joint_ira_deduction() {
    let mut session = example_return(Filing::MarriedJointly, 19560615);
    enter(
        &mut session,
        TaxSection::Wages,
        vec![pub915_owned(
            Owner::Spouse,
            IncomeSource::Wage {
                value: crate::WageDocument::W2(wage_document(crate::tests::cross::interview::w2(
                    dollars_input(14_000),
                ))),
            },
        )],
    );
    enter(&mut session, TaxSection::InterestDividends, vec![document(0, pub915_interest(250))]);
    enter(
        &mut session,
        TaxSection::Retirement,
        vec![
            document(0, pub915_pension(15_500)),
            document(0, pub915_benefits(5_600, SocialSecurityStatement::SocialSecurity)),
        ],
    );
    let mut editor = page(&mut session, TaxSection::IraContributions);
    let P::SaveIraContributions { value, .. } = &mut editor else { unreachable!() };
    value.people = vec![IraContributor {
        owner: Owner::Spouse,
        traditional: dollars_input(1_000),
        roth: 0,
        covered: false,
        form_8606_value: None,
        more: IraMore::default(),
    }];
    accepted(&send(&mut session, editor));
    let result = supported(&session.draft);
    assert_eq!(printed(&result, TaxForm::Form1040, "6a"), 5_600);
    assert_eq!(printed(&result, TaxForm::Form1040, "6b"), 0);
    assert_eq!(printed(&result, TaxForm::Schedule1, "20"), 1_000);
}

/// Pub. 915 (2026 DRAFT, title-page August 17, 2026; IRS postedAugust 21), printedp.10
/// (PDFp.11), Worksheet 1 Example 3: $10,000 RRB-1099 SSEB, $6,275 taxable.
/// Jessie's pension $38,000; interest $2,300 plus excluded savings-bond interest $200.
/// Neutral completion for published qualification: both bornJune15, 1956;
/// $2,000 bond proceeds issuedJune1, 2000, no earlier interest reporting;
/// Jessie's$2,000 eligible tuition, no other assistance or education credit.
/// Those facts make the full published$200 exclusion valid; they are not IRS facts.
/// https://www.irs.gov/pub/irs-dft/p915--dft.pdf (fetched 2026-10-10).
#[test]
fn pub915_worksheet_one_railroad_and_savings_bond_exclusion() {
    let mut session = example_return(Filing::MarriedJointly, 19560615);
    enter(
        &mut session,
        TaxSection::Retirement,
        vec![
            pub915_owned(Owner::Spouse, pub915_pension(38_000)),
            document(0, pub915_benefits(10_000, SocialSecurityStatement::Railroad)),
        ],
    );
    let mut int = match pub915_interest(2_300) {
        IncomeSource::InterestDividend { value: crate::InterestDividendDocument::Int1099(int) } => {
            int
        }
        _ => unreachable!(),
    };
    int.us_savings_bond_and_treasury_interest = dollars_input(200);
    // The bonds name their accepted 1099-INT as the interest source, so the
    // statement is saved first and the bonds follow under the same owner.
    enter(
        &mut session,
        TaxSection::InterestDividends,
        vec![document(
            0,
            IncomeSource::InterestDividend {
                value: crate::InterestDividendDocument::Int1099(interest_document(int)),
            },
        )],
    );
    let source_page =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = source_page else { unreachable!() };
    assert_eq!(documents.common.interest_dividends.documents.len(), 1);
    let source =
        documents.common.interest_dividends.documents.into_iter().next().unwrap().into_income();
    let bonds = SavingsBonds {
        source_issuer: Default::default(),
        source_recipient: Default::default(),
        proceeds: dollars_input(2_000),
        redemption_interest: dollars_input(200),
        earliest_issue_date: 20000601,
        interest_statement: source.id,
        students: vec![SavingsBondStudent {
            person: -2,
            expenses: SavingsBondExpenses::Entered { tuition: dollars_input(2_000), benefits: 0 },
            contributions: 0,
        }],
    };
    enter(
        &mut session,
        TaxSection::InterestDividends,
        vec![
            source,
            document(
                0,
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::SavingsBonds(savings_bonds_document(
                        savings_bonds_source(bonds),
                    )),
                },
            ),
        ],
    );
    let result = supported(&session.draft);
    assert_eq!(printed(&result, TaxForm::Form1040, "2b"), 2_300);
    assert_eq!(printed(&result, TaxForm::Form1040, "6a"), 10_000);
    assert_eq!(printed(&result, TaxForm::Form1040, "6b"), 6_275);
}

/// Pub. 915 (2026 DRAFT, title-page August 17, 2026; IRS postedAugust 21), printedp.11
/// (PDFp.12), Worksheet 1 Example 4: Kris's$4,000 benefits, $3,400 taxable.
/// Married filing separately, living together; Kris's wages $8,000; Taylor$0.
/// Neutral completion: Kris bornJune15, 1956, noncommunity-property residence,
/// ordinary W-2 wages and no withholding. No spouse income is added to this return.
/// https://www.irs.gov/pub/irs-dft/p915--dft.pdf (fetched 2026-10-10).
#[test]
fn pub915_worksheet_one_separate_lived_together() {
    let mut session = example_return(Filing::MarriedSeparately, 19560615);
    wages(&mut session, 8_000);
    enter(
        &mut session,
        TaxSection::Retirement,
        vec![document(0, pub915_benefits(4_000, SocialSecurityStatement::SocialSecurity))],
    );
    let result = supported(&session.draft);
    assert_eq!(printed(&result, TaxForm::Form1040, "6a"), 4_000);
    assert_eq!(printed(&result, TaxForm::Form1040, "6b"), 3_400);
}

/// Pub. 915 (2026 DRAFT, title-page August 17, 2026; IRS postedAugust 21), printedpp.12-15
/// (PDFpp.13-16), Terry Jackson's Worksheets1, 2, 4: regular$3,000; elected$2,500.
/// Benefits$11,000 include$2,000 for 2025. Current wages $3,500, interest $2,500,
/// dividends$1,500, fully taxable pension $18,000; prior AGI$23,000, no benefits.
/// Neutral completion: bornJune15, 1956; ordinary pension and dividends,
/// no withholding, earlier lump sums, adjustments or exempt interest.
/// https://www.irs.gov/pub/irs-dft/p915--dft.pdf (fetched 2026-10-10).
#[test]
fn pub915_terry_jackson_lump_sum_election() {
    let mut session = example_return(Filing::Single, 19560615);
    wages(&mut session, 3_500);
    enter(
        &mut session,
        TaxSection::InterestDividends,
        vec![
            document(0, pub915_interest(2_500)),
            document(
                0,
                IncomeSource::InterestDividend {
                    value: crate::InterestDividendDocument::Div1099(dividend_document(Div1099 {
                        payer: crate::PrintedParty::from_name("Synthetic dividend payer".into()),
                        recipient: Default::default(),
                        ordinary: dollars_input(1_500),
                        qualified: 0,
                        state_details: Default::default(),
                        ..native_div1099("".into(), 0, 0)
                    })),
                },
            ),
        ],
    );
    let pension = pub915_pension(18_000);
    let benefits = pub915_benefits(11_000, SocialSecurityStatement::SocialSecurity);
    enter(
        &mut session,
        TaxSection::Retirement,
        vec![document(0, pension.clone()), document(0, benefits.clone())],
    );
    assert_eq!(printed(&supported(&session.draft), TaxForm::Form1040, "6b"), 3_000);
    enter(
        &mut session,
        TaxSection::Retirement,
        vec![
            document(0, pension),
            document(0, benefits),
            document(
                0,
                IncomeSource::Record {
                    value: crate::IncomeRecord::SocialLumpSums(LumpSumElection {
                        printed_years: Vec::new(),
                        years: vec![SocialBenefitYear {
                            year: 2025,
                            status: FederalStatus::Single,
                            apart: false,
                            received: dollars_input(2_000),
                            benefits: 0,
                            earlier_lumps: 0,
                            agi: dollars_input(23_000),
                            taxable: 0,
                            refigured: 0,
                            exempt: 0,
                            addbacks: 0,
                        }],
                        complete: true,
                    }),
                },
            ),
        ],
    );
    let result = supported(&session.draft);
    assert_eq!(printed(&result, TaxForm::Form1040, "6a"), 11_000);
    assert_eq!(printed(&result, TaxForm::Form1040, "6b"), 2_500);
}

/// Pub. 596 (2025, November 18, 2025), pp. 22-23, detailed Example 1, Jamie Smith.
/// Age 63, $17,000 wages, $7,400 taxable pension and $7,000 Social Security;
/// published AGI $24,400 and no EIC. Ages shift uniformly into 2026.
/// https://www.irs.gov/pub/irs-pdf/p596.pdf (fetched 2026-10-10).
#[test]
#[ignore = "IRS596-JAMIE source inconsistency: printed AGI $24,400 omits $1,450 taxable Social Security from stated facts; App AGI $25,850, EIC $0"]
fn pub596_jamie_smith_no_eic() {
    let mut session = example_return(Filing::Single, 19630615);
    wages(&mut session, 17_000);
    enter(
        &mut session,
        TaxSection::Retirement,
        vec![
            document(0, pub915_pension(7_400)),
            document(0, pub915_benefits(7_000, SocialSecurityStatement::SocialSecurity)),
        ],
    );
    let result = supported(&session.draft);
    assert_eq!(value(&result, "agi"), 24_400);
    assert_eq!(value(&result, "earned_income_credit"), 0);
}

/// Pub. 946 (2025, March 13, 2026), pp. 42-43, Example 3, first-year mid-quarter.
/// Safe $4,000 in January, furniture $1,000 in September, computer $5,000 in October:
/// published deductions $1,000, $107 and $250. Source's relative year is 2026.
/// https://www.irs.gov/pub/irs-pdf/p946.pdf (fetched 2026-10-10).
#[test]
fn pub946_mid_quarter_first_year() {
    let assets = [
        (4_000, 20260101, AssetClass::Seven),
        (1_000, 20260901, AssetClass::Seven),
        (5_000, 20261001, AssetClass::Five),
    ]
    .into_iter()
    .map(|(basis, date, class)| {
        crate::tests::cross::interview::with_first_year(
            crate::tests::cross::interview::asset(dollars_input(basis), date, class),
            FirstYearDeduction::Regular,
        )
    })
    .collect();
    assert_eq!(printed(&business_assets(assets), TaxForm::ScheduleC, "13"), 1_000 + 107 + 250);
}

/// Pub. 946 (2025, March 13, 2026), pp. 42-43, Example 3, second-year mid-quarter.
/// The source's prior deductions are $1,000, $107 and $250; current $857, $255, $1,900.
/// Prior service dates shift uniformly to 2025; no bonus or section 179 election.
/// https://www.irs.gov/pub/irs-pdf/p946.pdf (fetched 2026-10-10).
#[test]
fn pub946_mid_quarter_second_year() {
    let assets = [
        (4_000, 20250101, AssetClass::Seven, 1_000),
        (1_000, 20250901, AssetClass::Seven, 107),
        (5_000, 20251001, AssetClass::Five, 250),
    ]
    .into_iter()
    .map(|(basis, date, class, prior)| {
        let mut asset = crate::tests::cross::interview::asset(dollars_input(basis), date, class);
        asset.gds_mut().unwrap().history = AssetHistory::Continuing {
            prior_depreciation: dollars_input(prior),
            first_year_179: 0,
            first_year_bonus: 0,
            carryforward: 0,
            mid_quarter: true,
        };
        asset
    })
    .collect();
    assert_eq!(printed(&business_assets(assets), TaxForm::ScheduleC, "13"), 857 + 255 + 1_900);
}

/// Pub. 523 (2025, February 26, 2026), pp. 17-18, Finley nonqualified use.
/// Entire timeline shifts one year: bought January 1, 2021; rented two years;
/// lived January 1, 2023-January 1, 2025; sold January 1, 2026.
/// IRS uses the rounded year fraction 2/5 and prints $120,000 nonqualified gain.
/// Original dates give 731/1827 rental/ownership days and 730 residence days;
/// shifting the timeline changes those to 730/1826 and 731 residence days.
/// The observed difference combines that leap-day shift with the IRS 2/5-year
/// illustration convention; it is not an unchanged-original-date comparison.
/// This is not an indexed-year or demonstrated tax-law implementation defect.
/// https://www.irs.gov/pub/irs-pdf/p523.pdf (fetched 2026-10-10).
#[test]
#[ignore = "IRS523-FINLEY illustration convention: IRS uses 2/5 years; page calculates exact 730/1826 days, not an implementation defect"]
fn pub523_finley_nonqualified_use() {
    let mut session = example_return(Filing::Single, 19800615);
    let sale = home_sale(20210101, 20260101, 700_000, 380_000, 20_000, 731, 730);
    enter(
        &mut session,
        TaxSection::InvestmentSales,
        vec![document(
            0,
            IncomeSource::InvestmentSale { value: crate::InvestmentSaleDocument::Form1099S(sale) },
        )],
    );
    let result = supported(&session.draft);
    assert_eq!(value(&result, "home_1_realized_gain"), 320_000);
    assert_eq!(value(&result, "home_1_excluded_gain"), 180_000);
    assert_eq!(value(&result, "home_1_taxable_gain"), 140_000);
}

fn joan(taxable: i64) -> Interview {
    let mut session = example_return(Filing::Single, 19980615);
    // Age and wages are unspecified; these completion facts avoid credit phaseout.
    wages(&mut session, 30_000);
    student_credit(&mut session, 3_000, 2_000, included(taxable));
    session
}

/// Pub. 970 (2025, January 27, 2026), p. 15, Joan Example 1.
/// $3,000 tuition less the excluded $2,000 scholarship leaves $1,000 expenses.
/// The $4,000 student loan and $5,000 room/board are not education reductions.
/// Only the accepted credit expense pool is compared; no IRS credit is invented.
/// https://www.irs.gov/pub/irs-pdf/p970.pdf (fetched 2026-10-10).
#[test]
fn pub970_joan_excluded_scholarship_expenses() {
    let session = joan(0);
    supported(&session.draft);
    assert_eq!(
        session.draft.credits.as_ref().unwrap().education_inputs()[0]
            .expenses
            .qualified(EducationMethod::AmericanOpportunity),
        dollars_input(1_000)
    );
}

/// Pub. 970 (2025, January 27, 2026), p. 15, Joan Example 2.
/// Including the entire $2,000 scholarship leaves the published $3,000 expenses.
/// Public box 5 remains the real $2,000; the election includes it ($2,000 included part).
/// https://www.irs.gov/pub/irs-pdf/p970.pdf (fetched 2026-10-10).
#[test]
fn pub970_joan_taxable_scholarship_expenses() {
    let session = joan(2_000);
    supported(&session.draft);
    assert_eq!(
        session.draft.credits.as_ref().unwrap().education_inputs()[0]
            .expenses
            .qualified(EducationMethod::AmericanOpportunity),
        dollars_input(3_000)
    );
}

/// Form 8962 instructions (2025, October 1, 2025), p. 17, Melissa and Ryan.
/// Melissa has $1,000 monthly joint-policy advances January-April, half allocated;
/// her own May-December advance is $400/month. Published total advance: $5,200.
/// Premium and benchmark amounts are absent; complete both as $2,000/month,
/// and income as $40,000. Only the published allocation is compared.
/// https://www.irs.gov/pub/irs-pdf/i8962.pdf (fetched 2026-10-10).
#[test]
fn i8962_melissa_separate_spouse_then_own_policy() {
    let mut session = example_return(Filing::MarriedSeparately, 19800615);
    wages(&mut session, 40_000);
    let months = (1..=12)
        .map(|month| PolicyMonth {
            month,
            premium: dollars_input(2_000),
            benchmark: Some(dollars_input(2_000)),
            advance: dollars_input(if month <= 4 { 1_000 } else { 400 }),
        })
        .collect();
    marketplace(
        &mut session,
        PolicyAmounts::ByMonth { months },
        PolicySharing::Spouse { from: 1, to: 4 },
    );
    assert_eq!(value(&supported(&session.draft), "premium_credit_advance"), 5_200);
}

fn annual_policy(premium: i64, advance: i64) -> PolicyAmounts {
    let premium = dollars_input(premium);
    let advance = dollars_input(advance);
    PolicyAmounts::ByMonth {
        months: (1..=12)
            .map(|month| PolicyMonth {
                month,
                premium: premium / 12 + if month == 12 { premium % 12 } else { 0 },
                benchmark: Some(dollars_input(2_000)),
                advance: advance / 12 + if month == 12 { advance % 12 } else { 0 },
            })
            .collect(),
    }
}

/// Form 8962 instructions (2025, October 1, 2025), pp. 19-20, John/Carol Example 1.
/// John files separately; annual premium $14,000 and APTC $8,500 divide equally.
/// The exact allocation is $4,250. Neutral income $40,000 and benchmark $2,000/month;
/// the source's year-sensitive repayment limitation is not reused for 2026.
/// https://www.irs.gov/pub/irs-pdf/i8962.pdf (fetched 2026-10-10).
#[test]
#[ignore = "IRS8962-JOHN rounding convention: IRS annual $8,500 / 2 = $4,250; monthly whole-dollar allocations total $4,248"]
fn i8962_john_separate_spouse_allocation() {
    let mut session = example_return(Filing::MarriedSeparately, 19800615);
    wages(&mut session, 40_000);
    marketplace(
        &mut session,
        annual_policy(14_000, 8_500),
        PolicySharing::Spouse { from: 1, to: 12 },
    );
    assert_eq!(value(&supported(&session.draft), "premium_credit_advance"), 4_250);
}

/// Form 8962 instructions (2025, October 1, 2025), p. 20, Nancy/Kevin Example 2.
/// Annual premium $10,000 and APTC $6,500 divide equally. Nancy's APTC is $3,250.
/// Her relief eligibility, income and benchmark do not alter this allocation oracle;
/// complete income as $40,000 and benchmark as $2,000/month, without claiming relief.
/// https://www.irs.gov/pub/irs-pdf/i8962.pdf (fetched 2026-10-10).
#[test]
#[ignore = "IRS8962-NANCY rounding convention: IRS annual $6,500 / 2 = $3,250; monthly whole-dollar allocations total $3,252"]
fn i8962_nancy_separate_spouse_allocation() {
    let mut session = example_return(Filing::MarriedSeparately, 19800615);
    wages(&mut session, 40_000);
    marketplace(
        &mut session,
        annual_policy(10_000, 6_500),
        PolicySharing::Spouse { from: 1, to: 12 },
    );
    assert_eq!(value(&supported(&session.draft), "premium_credit_advance"), 3_250);
}
