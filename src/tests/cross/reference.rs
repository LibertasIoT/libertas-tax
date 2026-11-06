//! External-reference cases: `tests/fixtures/reference/<source>/<case>.json`, schema in that
//! directory's README.md. Each case enters its pages through the native client Requests (the
//! `irs_examples` helpers), then compares every listed line and the state against figures
//! printed by a source outside the App, collecting all mismatches before failing once.
extern crate std;
use super::interview::{
    accepted, adult, business, cash_farm, document, from_review, no_expenses, row, save_care_costs,
    save_children, save_more_credits, save_you, with_more,
};
use super::irs_examples::{enter, example_return_with_spouse, printed};
use super::reference_pages::{Doc, Loan, documents, office_site, section};
use super::*;
use alloc::string::String;

#[derive(serde::Deserialize)]
struct Case {
    filing: String,
    birth: u32,
    // Defaults to `birth` when the source gives only one date.
    spouse_birth: Option<u32>,
    // The filer is a dependent child who can be claimed on a parent's return (Form 8615).
    #[serde(default)]
    dependent: bool,
    pages: Vec<PageSpec>,
    expected: Vec<Line>,
    expected_state: String,
    // Notice keys that must appear in the review notes (known gaps).
    #[serde(default)]
    expected_notes: Vec<String>,
    // Issue keys that must appear when the App refuses an item.
    #[serde(default)]
    expected_issues: Vec<String>,
    // Gaps excluded from `expected`: documentation only, each names its ledger row.
    #[serde(default)]
    #[allow(dead_code)]
    known_gaps: Vec<serde_json::Value>,
    // Printed lines the App is known to miss: reported, not failed (triage class B/C/D/E).
    #[serde(default)]
    expect_mismatch: Vec<Mismatch>,
}

#[derive(serde::Deserialize)]
#[serde(tag = "page", rename_all = "snake_case")]
enum PageSpec {
    Children {
        children: Vec<(String, String, u32)>,
    },
    // Children with the care expenses paid for them: `[first, last, yyyymmdd, dollars]`.
    CareChildren {
        children: Vec<(String, String, u32, i64)>,
    },
    // Nieces or nephews who live in Mexico, are not U.S. citizens, nationals or resident aliens,
    // and are supported by the filer.
    ForeignRelatives {
        children: Vec<(String, String, u32)>,
    },
    Wages {
        w2: Vec<Wage>,
    },
    // The documents of one topic page (everything the page holds).
    Documents {
        topic: String,
        docs: Vec<Doc>,
    },
    // A simplified home office for the business just entered: `(month, days, square feet)`.
    HomeOffice {
        months: Vec<(u32, u32, u32)>,
    },
    // Regular IRA contributions retained for 2026.
    IraContributions {
        traditional: i64,
        roth: i64,
        #[serde(default)]
        covered: bool,
    },
    // Itemizing, with these Form 1098 loans.
    Mortgages {
        loans: Vec<Loan>,
    },
    // The parent's return for Form 8615: a joint return with this taxable income.
    Parent {
        taxable: i64,
    },
    // The taxpayer as the one student (1098-T box 1 only), AOTC years used up or not.
    Education {
        tuition: i64,
        used_up: bool,
    },
    // The dependent care page of a W-2 with box 10: the plan maximum and the part forfeited.
    Care {
        plan_maximum: i64,
        forfeited: i64,
    },
    // One Schedule C business: whole-dollar receipts, expense lines by field name, and `other`
    // expense rows (description, amount). `special` declares a Form 7205 or timber treatment the
    // App leaves out.
    Business {
        receipts: i64,
        #[serde(default)]
        expenses: std::collections::BTreeMap<String, i64>,
        #[serde(default)]
        other: Vec<(String, i64)>,
        #[serde(default)]
        special: bool,
        // Section 195 start-up costs: (whole-dollar costs, yyyymmdd the business began).
        #[serde(default)]
        startup: Option<(i64, u32)>,
    },
    // A purchased advanced manufacturing production credit (Form 7207 through a Schedule A
    // (Form 3800) transfer election statement), whole dollars. Section 6418 transferred credits
    // are passive for a transferee with no interest in the transferor's business
    // (26 CFR 1.6418-2(f)(3)(ii)).
    BoughtProductionCredit {
        amount: i64,
    },
    // 1099-DIV: ordinary, qualified and capital gain distributions (boxes 1a, 1b, 2a).
    Dividends {
        div: Vec<Dividend>,
    },
    // One Schedule F farm, whole dollars, with the Form 4136 gasoline lines it claims.
    Farm(FarmSpec),
    // 1098-VLI interest on new, U.S.-assembled vehicles (Schedule 1-A Part IV).
    CarLoans {
        interest: Vec<i64>,
    },
}

#[derive(serde::Deserialize)]
struct Dividend {
    ordinary: i64,
    qualified: i64,
    capital_gain: i64,
}

#[derive(serde::Deserialize)]
struct FarmSpec {
    raised: i64,
    program: i64,
    crop_insurance: i64,
    other_income: i64,
    // Schedule F expense lines by field name; any other name is an "other expenses" row.
    expenses: std::collections::BTreeMap<String, i64>,
    // Form 4136 line 1 uses: (line, gallons, actual cost).
    gasoline: Vec<(String, u32, i64)>,
}

#[derive(serde::Deserialize)]
struct Wage {
    wages: f64,
    withholding: f64,
    social_wages: f64,
    social_tax: f64,
    medicare_wages: f64,
    medicare_tax: f64,
    // Box 12 code TT, qualified overtime compensation.
    #[serde(default)]
    overtime: f64,
    // Box 10, dependent care benefits.
    #[serde(default)]
    care_benefits: f64,
}

#[derive(serde::Deserialize)]
struct Line {
    form: String,
    line: String,
    amount: i64,
}

#[derive(serde::Deserialize)]
struct Mismatch {
    form: String,
    line: String,
    class: String,
    triage: String,
}

fn farm(spec: &FarmSpec) -> IncomeDocumentItem {
    let c = |v: i64| v * 100;
    let mut farm = cash_farm(c(spec.raised), 0);
    farm.accounting = FarmMethod::Cash {
        resale: 0,
        resale_cost: 0,
        raised: c(spec.raised),
        program: c(spec.program),
        custom_hire: 0,
        other: c(spec.other_income),
    };
    farm.insurance = CropInsurance::Payments {
        physical: vec![CropLoss {
            crop: "Grain".into(),
            amount: c(spec.crop_insurance),
            year: 2026,
        }],
        price_only: 0,
        from_prior_year: 0,
        withholding: 0,
    };
    let e = &mut farm.expenses;
    for (line, amount) in &spec.expenses {
        let amount = c(*amount);
        match line.as_str() {
            "chemicals" => e.chemicals = amount,
            "equipment_rent" => e.equipment_rent = amount,
            "fertilizers" => e.fertilizers = amount,
            "freight" => e.freight = amount,
            "fuel" => e.fuel = amount,
            "insurance" => e.insurance = amount,
            "mortgage_interest" => e.mortgage_interest = amount,
            "other_rent" => e.other_rent = amount,
            "repairs" => e.repairs = amount,
            "seeds" => e.seeds = amount,
            "storage" => e.storage = amount,
            "supplies" => e.supplies = amount,
            "taxes" => e.taxes = amount,
            "utilities" => e.utilities = amount,
            other => e.other.push(OtherExpense { description: other.into(), amount }),
        }
    }
    let mut fuels = FederalFuelLines::default();
    for (line, gallons, cost) in &spec.gasoline {
        let used = Some(FederalFuelUse {
            gallons: f64::from(*gallons),
            actual_cost: c(*cost),
            qualified: true,
            not_claimed_elsewhere: true,
        });
        match line.as_str() {
            "off_highway" => fuels.gasoline_off_highway = used,
            "farm" => fuels.gasoline_farm = used,
            other => panic!("unknown gasoline line {other}"),
        }
    }
    let farm = with_more(farm, |more| {
        // The credit's tax was kept out of the expense lines, so no income inclusion follows.
        more.fuel_tax_credit = Some(FederalFuelTaxCredit {
            fuels: fuels.into(),
            expense_treatment: FederalFuelExpenseTreatment::TaxExcludedFromExpenses,
            records_complete: true,
        });
    });
    document(0, IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) })
}

// A printed line; a Form 1040 line the return does not show is zero.
fn line(result: &FederalResult, form: TaxForm, line: &str) -> i64 {
    let shown = result
        .return_lines
        .iter()
        .any(|v| v.form_line.as_ref().is_some_and(|f| f.form == form && f.line == line));
    if form == TaxForm::Form1040 && !shown { 0 } else { printed(result, form, line) }
}

// A calculation line the App does not tag with a form line, by id, in whole dollars (half up).
fn calc(result: &FederalResult, id: &str) -> i64 {
    let cents: i64 = result.lines.iter().filter(|l| l.id == id).map(|l| l.amount).sum();
    (cents + 50).div_euclid(100)
}

fn form(name: &str) -> TaxForm {
    match name {
        "1040" => TaxForm::Form1040,
        "8812" => TaxForm::Schedule8812,
        "EIC" => TaxForm::ScheduleEic,
        "Sch1" => TaxForm::Schedule1,
        "Sch2" => TaxForm::Schedule2,
        "SchC" => TaxForm::ScheduleC,
        "SchSE" => TaxForm::ScheduleSe,
        "Sch3" => TaxForm::Schedule3,
        "SchA" => TaxForm::ScheduleA,
        "2441" => TaxForm::Form2441,
        "5329" => TaxForm::Form5329,
        "8615" => TaxForm::Form8615,
        "8863" => TaxForm::Form8863,
        "SchF" => TaxForm::ScheduleF,
        "3800" => TaxForm::Form3800,
        "4562-B" => TaxForm::Form4562B,
        other => panic!("unknown form {other}"),
    }
}

fn check(name: &str, case: &Case, out: &mut Vec<String>) {
    let filing = match case.filing.as_str() {
        "Single" => Filing::Single,
        "MarriedJointly" => Filing::MarriedJointly,
        other => panic!("{name}: unknown filing {other}"),
    };
    let mut session =
        example_return_with_spouse(filing, case.birth, case.spouse_birth.unwrap_or(case.birth));
    if case.dependent {
        let you = Filer {
            details: FilerDetails {
                claimable: Claimable::Possibly {
                    another_eitc_child: false,
                    dependency: FilerDependency::CanBeClaimed { claimed: true },
                    us_home_over_half_year: true,
                    full_time_student: false,
                    disabled: false,
                    incapable_self_care: false,
                    support_share: SupportShare::LessThanHalf,
                    parent_alive: true,
                },
                ..adult("Synthetic", "Example", case.birth).details
            },
            ..adult("Synthetic", "Example", case.birth)
        };
        let request = save_you(&session.draft, you);
        accepted(&send(&mut session, request));
    }
    // The care paid, entered on the care page: with its plan answers when the
    // case has them, otherwise on its own.
    let mut care: Vec<CareCosts> = Vec::new();
    let care_page = case.pages.iter().any(|p| matches!(p, PageSpec::Care { .. }));
    for page in &case.pages {
        match page {
            PageSpec::Children { children } => {
                let people = children.iter().map(|(f, l, b)| row(f, l, *b)).collect();
                let request = save_children(&session.draft, people);
                accepted(&send(&mut session, request));
            }
            PageSpec::Wages { w2 } => {
                let documents = w2
                    .iter()
                    .map(|w| {
                        let c = |v: f64| (v * 100.0).round() as i64;
                        let base = super::interview::w2(0);
                        let mut more = base.more.clone();
                        let mut answers = crate::W2Answers::default();
                        more.care_benefits = c(w.care_benefits);
                        if w.overtime > 0.0 {
                            more.box12.push(Box12Entry { code: W2Code::TT, amount: c(w.overtime) });
                            // The source deducts the TT amount on Schedule 1-A line 16.
                            answers.overtime_flsa_required =
                                Some(WageDeductionEligibility::Eligible);
                        }
                        document(
                            0,
                            IncomeSource::Wage {
                                value: crate::WageDocument::W2(wage_document(crate::W2Source {
                                    form: wage_document(W2 {
                                        more,
                                        wages: c(w.wages),
                                        withholding: c(w.withholding),
                                        social_wages: c(w.social_wages),
                                        social_tax: c(w.social_tax),
                                        medicare_wages: c(w.medicare_wages),
                                        medicare_tax: c(w.medicare_tax),
                                        ..base
                                    }),
                                    answers,
                                })),
                            },
                        )
                    })
                    .collect();
                enter(&mut session, TaxSection::Wages, documents);
            }
            PageSpec::CareChildren { children } => {
                let people = children.iter().map(|(f, l, b, _)| row(f, l, *b)).collect();
                let request = save_children(&session.draft, people);
                accepted(&send(&mut session, request));
                care = children
                    .iter()
                    .map(|(.., expenses)| CareCosts::Yes {
                        expenses: expenses * 100,
                        incapable_self_care: false,
                    })
                    .collect();
                if !care_page {
                    save_care_costs(&mut session, &care);
                }
            }
            PageSpec::ForeignRelatives { children } => {
                let people = children
                    .iter()
                    .map(|(f, l, b)| Dependent {
                        relationship: Relationship::NieceNephew,
                        lived: LivedWithYou::PartOfYear { months: 0 },
                        situation: DependentSituation::Supported {
                            your_support_over_half: true,
                            gross_income: 0,
                        },
                        facts: PersonFacts::ABROAD.0 | PersonFacts::RESIDENT_CANADA_MEXICO.0,
                        ..row(f, l, *b)
                    })
                    .collect();
                let request = save_children(&session.draft, people);
                accepted(&send(&mut session, request));
            }
            PageSpec::Documents { topic, docs } => {
                enter(&mut session, section(topic), documents(docs));
            }
            PageSpec::HomeOffice { months } => {
                let id = session.draft.income.last().expect("a business").id;
                let request = crate::tests::support::client::save_home_offices(
                    &session.draft,
                    vec![office_site(id, months)],
                );
                accepted(&send(&mut session, request));
            }
            PageSpec::IraContributions { traditional, roth, covered } => {
                let Reply::Edit(mut page) = from_review(&mut session, TaxSection::IraContributions)
                else {
                    panic!("{name}: no IRA page")
                };
                let P::SaveIraContributions { value, .. } = &mut page else { unreachable!() };
                value.people = vec![IraContributor {
                    owner: Owner::Taxpayer,
                    traditional: traditional * 100,
                    roth: roth * 100,
                    covered: *covered,
                    form_8606_value: None,
                    more: IraMore::default(),
                }];
                accepted(&send(&mut session, page));
            }
            PageSpec::Mortgages { loans } => {
                let Reply::Edit(mut page) = from_review(&mut session, TaxSection::Deductions)
                else {
                    panic!("{name}: no deductions page")
                };
                let P::SaveDeductions { value, .. } = &mut page else { unreachable!() };
                value.itemize_override = Some(ItemizeOverride::Itemize);
                accepted(&send(&mut session, page));
                let page = super::interview::save_mortgages(
                    &mut session,
                    MortgageDocuments {
                        documents: loans.iter().map(Loan::record).map(mortgage_document).collect(),
                    },
                );
                accepted(&send(&mut session, page));
                let answers = loans
                    .iter()
                    .map(|loan| (loan.record().more, loan.points_answer()))
                    .collect::<Vec<_>>();
                accepted(&super::interview::save_mortgage_answers(&mut session, &answers));
            }
            PageSpec::Parent { taxable } => {
                let value = crate::tests::federal::losses::parent(FederalStatus::Joint, *taxable);
                accepted(&crate::tests::federal::losses::save_parent(&mut session, value));
            }
            PageSpec::Education { tuition, used_up } => {
                let mut page =
                    crate::tests::support::client::income_pages::open_document_center(&mut session);
                let P::SaveDocuments { documents, .. } = &mut page else { unreachable!() };
                documents.common.education.students = vec![crate::StudentCreditDocument {
                    id: 0,
                    student_id: -1,
                    form: student_credit_paper(StudentCredit {
                        filer: Default::default(),
                        student: crate::PrintedParty::from_name("Synthetic Example".into()),
                        tuition: tuition * 100,
                        scholarships: 0,
                        half_time: true,
                        graduate: false,
                        prior_tuition_adjustment: crate::SourceCents::NotReported,
                        prior_scholarship_adjustment: crate::SourceCents::NotReported,
                    }),
                }];
                accepted(&send(&mut session, page));
                let method = if *used_up {
                    EducationChoice::LifetimeLearning
                } else {
                    EducationChoice::AmericanOpportunity
                };
                let (_, mut annual) = native_education(-1, education_expenses(0), method);
                annual.more.american_opportunity_used_up = Some(*used_up);
                accepted(&super::interview::save_education_answers(&mut session, vec![annual]));
            }
            PageSpec::Care { plan_maximum, forfeited } => {
                let mut page = super::interview::open_credits_topic(&mut session, TaxSection::Care);
                super::interview::put_care_costs(&mut page, &care);
                let P::SaveCare { value, .. } = &mut page else { unreachable!() };
                value.plan_maximum = CarePlanMaximum::Plans { maximum: plan_maximum * 100 };
                for row in &mut value.benefits {
                    row.forfeited = (*forfeited > 0).then_some(forfeited * 100);
                }
                accepted(&send(&mut session, page));
            }
            PageSpec::Business { receipts, expenses, other, special, startup } => {
                let c = |v: i64| v * 100;
                let mut b = business(c(*receipts));
                let mut lines = no_expenses();
                for (name, amount) in expenses {
                    let slot = match name.as_str() {
                        "insurance" => &mut lines.insurance,
                        "legal_and_professional" => &mut lines.legal_and_professional,
                        "office" => &mut lines.office,
                        "other_rent" => &mut lines.other_rent,
                        "supplies" => &mut lines.supplies,
                        "taxes_and_licenses" => &mut lines.taxes_and_licenses,
                        other => panic!("{name}: unknown expense line {other}"),
                    };
                    *slot = c(*amount);
                }
                lines.other = other
                    .iter()
                    .map(|(d, a)| crate::OtherExpense { description: d.clone(), amount: c(*a) })
                    .collect();
                b.expenses = lines;
                b.more.special = *special;
                if let Some((costs, started)) = startup {
                    b.more.startup = crate::BusinessStartup::Costs {
                        costs: c(*costs),
                        started: *started,
                        capitalize: false,
                        eligible: true,
                    };
                }
                let item =
                    document(0, IncomeSource::Record { value: crate::IncomeRecord::Business(b) });
                enter(&mut session, TaxSection::SelfEmployment, vec![item]);
            }
            PageSpec::Dividends { div } => {
                let documents = div
                    .iter()
                    .map(|d| {
                        let mut value =
                            native_div1099("".into(), d.ordinary * 100, d.qualified * 100);
                        value.capital_distributions = d.capital_gain * 100;
                        document(
                            0,
                            IncomeSource::InterestDividend {
                                value: crate::InterestDividendDocument::Div1099(dividend_document(
                                    value,
                                )),
                            },
                        )
                    })
                    .collect();
                enter(&mut session, TaxSection::InterestDividends, documents);
            }
            PageSpec::Farm(spec) => {
                enter(&mut session, TaxSection::Farming, vec![farm(spec)]);
                if spec.crop_insurance > 0 {
                    let activity = crate::farm::sources(&session.draft).last().unwrap().entry.id;
                    let request = P::SaveCropInsuranceTiming {
                        user: TEST_USER,
                        cookie: session.draft.cookie.clone(),
                        revision: session.draft.revision,
                        activity,
                        timing: CropPaperTiming::Include,
                    };
                    accepted(&send(&mut session, request));
                }
            }
            PageSpec::CarLoans { interest } => {
                let mut page =
                    crate::tests::support::client::income_pages::open_document_center(&mut session);
                let P::SaveDocuments { documents, .. } = &mut page else { unreachable!() };
                // The 1098-VLI prints its borrower, the filer's own loan; a
                // blank block would leave the owner question unanswered.
                let borrower =
                    crate::tests::support::builders::fixture_document_names(&session.draft, -1)
                        .unwrap()
                        .recipient;
                documents.other.vehicle_loans.documents = interest
                    .iter()
                    .map(|i| crate::Form1098VliDocument {
                        id: 0,
                        form: vehicle_paper(crate::FederalVehicleInterest {
                            lender: crate::PrintedParty::from_name("Synthetic Credit Union".into()),
                            borrower: borrower.clone(),
                            refund: 0,
                            interest: i * 100,
                            original_use: true,
                            assembled_in_us: true,
                        }),
                    })
                    .collect();
                accepted(
                    &crate::tests::support::client::income_pages::save_source_fixture_request(
                        &mut session,
                        page,
                    ),
                );
            }
            PageSpec::BoughtProductionCredit { amount } => {
                let history = crate::BusinessCreditHistory {
                    credits: vec![crate::EnteredBusinessCredit {
                        credit: crate::BusinessCredit::AdvancedManufacturingProduction,
                        source: crate::CreditSource::Bought,
                        amount: amount * 100,
                        election: crate::CreditElection::None,
                        carbon_oxide_origin: None,
                        renewable_facility_origin: None,
                    }],
                    carryforwards: vec![],
                    carrybacks: vec![],
                    spouse_without_credits: false,
                };
                let value = crate::MoreCredits {
                    answers: Vec::new(),
                    credits: vec![crate::MoreCredit::BusinessCredits(history.into())],
                    documents: vec![],
                };
                let request = save_more_credits(&session.draft, value);
                accepted(&send(&mut session, request));
            }
        }
    }
    let result = estimate(&session.draft);
    if std::env::var("REF_DUMP").is_ok_and(|v| name.contains(&v)) {
        std::eprintln!("CASE {name}");
        std::eprintln!("STATE {:?}", result.state);
        for v in &result.return_lines {
            if let Some(f) = &v.form_line {
                std::eprintln!("LINE {:?} {} {}", f.form, f.line, v.amount / 100);
            }
        }
        for v in &result.lines {
            match &v.form_line {
                Some(f) => std::eprintln!("LINE {:?} {} {}", f.form, f.line, v.amount / 100),
                None => std::eprintln!("VALUE {} {}", v.id, v.amount / 100),
            }
        }
        for i in &result.issues {
            std::eprintln!("ISSUE {}", i.display.key);
        }
        for n in &result.review_notes {
            std::eprintln!("NOTE {}", n.display.key);
        }
    }
    let state = alloc::format!("{:?}", result.state);
    if state != case.expected_state {
        out.push(alloc::format!("{name}: state {state}, expected {}", case.expected_state));
    }
    for key in &case.expected_issues {
        if !result.issues.iter().any(|i| &i.display.key == key) {
            out.push(alloc::format!("{name}: missing issue {key}"));
        }
    }
    for key in &case.expected_notes {
        if !result.review_notes.iter().any(|n| &n.display.key == key) {
            out.push(alloc::format!("{name}: missing review note {key}"));
        }
    }
    for want in &case.expected {
        let got = if want.form == "calc" {
            calc(&result, &want.line)
        } else {
            line(&result, form(&want.form), &want.line)
        };
        let known =
            case.expect_mismatch.iter().find(|m| m.form == want.form && m.line == want.line);
        match known {
            Some(m) if got != want.amount => std::eprintln!(
                "{name}: {} line {}: App {got}, source {} (class {}, {})",
                want.form,
                want.line,
                want.amount,
                m.class,
                m.triage
            ),
            Some(m) => out.push(alloc::format!(
                "{name}: {} line {} now matches ({got}); remove expect_mismatch {}",
                want.form,
                want.line,
                m.triage
            )),
            None if got != want.amount => out.push(alloc::format!(
                "{name}: {} line {}: App {got}, source {}",
                want.form,
                want.line,
                want.amount
            )),
            None => {}
        }
    }
}

#[test]
fn reference_cases_match_their_sources() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/reference");
    let mut out = Vec::new();
    for source in std::fs::read_dir(&root).unwrap() {
        let source = source.unwrap().path();
        if !source.is_dir() {
            continue;
        }
        for file in std::fs::read_dir(&source).unwrap() {
            let path = file.unwrap().path();
            let raw = std::fs::read_to_string(&path).unwrap();
            let case: Case = serde_json::from_str(&raw).unwrap_or_else(|e| panic!("{path:?}: {e}"));
            check(&path.file_stem().unwrap().to_string_lossy(), &case, &mut out);
        }
    }
    assert!(out.is_empty(), "{}", out.join("\n"));
}
