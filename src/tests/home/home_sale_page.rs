use crate::tests::support::assert::line_amount as cents;
use crate::tests::support::assert::{accepted, printed_line, rejected_with, return_line};
// u6-home: the sale of your main home through its page (Investment sales,
// kind "Sale of your main home (1099-S)"), and Form 8828's sale path. Every
// expected amount is worked by hand in the comment beside it from Pub. 523
// (Worksheets 1-3), the Schedule D and Form 8949 instructions (2025 drafts,
// the 2026 amounts the App uses: $16,100 single and $32,200 joint standard
// deduction; 10% to $12,400 / $24,800, 12% to $50,400 / $100,800, 22% to
// $105,700, 24% above; capital gain 0% up to $49,450 / $98,900; Tax Table
// rows $50 wide taxed at their midpoint) and Form 8828.
use super::*;
use crate::tests::cross::interview::{
    document, open_other_taxes, other_taxes_request, page_documents, save_other_taxes,
};
use crate::tests::support::client::income_pages::{open_document_center, save_each_topic};
use alloc::{format, string::String};

fn has_line(r: &FederalResult, id: &str) -> bool {
    r.lines.iter().any(|l| l.id == id)
}
fn has_issue(r: &FederalResult, key: &str) -> bool {
    r.issues.iter().any(|i| i.display.key == key)
}

// The sale as its page once held it in one record: tests state it flat and the
// stored source is the printed 1099-S beside the home answers.
#[derive(Clone, Debug)]
pub(in crate::tests) struct HomePage {
    pub filer: crate::PrintedParty,
    pub transferor: crate::PrintedParty,
    pub address: alloc::string::String,
    pub closed: u32,
    pub proceeds: i64,
    pub property_received: bool,
    pub property_value: SourceCents,
    pub foreign_person: bool,
    pub firpta_withheld: SourceCents,
    pub form_1099s: bool,
    pub acquired: u32,
    pub received: HomeReceived,
    pub payment: SalePayment,
    pub exchange: Option<HomeExchange>,
    pub expenses: i64,
    pub basis: i64,
    pub lived: HomeLived,
    pub earlier_sale: EarlierHomeSale,
    pub spouse: HomeSpouse,
    pub reason: HomeSaleReason,
    pub use_history: HomeRentalUse,
    pub service: Option<HomeService>,
    pub more: HomeSaleMore,
}

impl HomePage {
    pub(in crate::tests) fn source(&self) -> MainHomeSale {
        let HomePage {
            filer,
            transferor,
            address,
            closed,
            proceeds,
            property_received,
            property_value,
            foreign_person,
            firpta_withheld,
            form_1099s,
            acquired,
            received,
            payment,
            exchange,
            expenses,
            basis,
            lived,
            earlier_sale,
            spouse,
            reason,
            use_history,
            service,
            more,
        } = self.clone();
        let form = Form1099S {
            filer,
            transferor,
            address,
            closed,
            proceeds,
            property_received,
            foreign_person,
        };
        let answers = HomeSaleAnswers {
            property_value,
            firpta_withheld,
            form_1099s,
            acquired,
            received,
            expenses,
            payment,
            exchange,
            basis,
            lived,
            earlier_sale,
            spouse,
            reason,
            service,
            use_history,
            more,
        };
        if form_1099s {
            MainHomeSale {
                form: crate::HomeSaleSource::Paper(form_1099s_paper(form)),
                answers: Some(crate::RealEstateSaleUse::MainHome(answers)),
            }
        } else {
            MainHomeSale::records(form, answers)
        }
    }
    fn record(&self) -> HomeSale {
        HomeSale::from(&self.source())
    }
}

// The stored source of the standard single filer's sale (or a joint one).
pub(in crate::tests) fn home(joint: bool) -> MainHomeSale {
    page(joint).source()
}

// A single filer's home, bought 2015-03-01 and sold 2026-06-15 for $450,000
// with $25,000 of selling expenses and a $200,000 basis, lived in the whole
// time, with a 1099-S.
fn page(joint: bool) -> HomePage {
    HomePage {
        filer: Default::default(),
        transferor: Default::default(),
        address: "12 Synthetic Lane".into(),
        closed: 20260615,
        proceeds: 45_000_000,
        property_received: false,
        property_value: SourceCents::NotReported,
        foreign_person: false,
        firpta_withheld: SourceCents::NotReported,
        form_1099s: true,
        acquired: 20150301,
        received: HomeReceived::Bought,
        payment: SalePayment::InFull,
        exchange: None,
        expenses: 2_500_000,
        basis: 20_000_000,
        lived: HomeLived::WholeTime,
        earlier_sale: EarlierHomeSale::No,
        spouse: if joint {
            HomeSpouse::Joint { lived: HomeLived::WholeTime, earlier_sale: EarlierHomeSale::No }
        } else {
            HomeSpouse::NotJoint
        },
        reason: HomeSaleReason::TwoYears,
        use_history: HomeRentalUse { depreciation: 0, nonqualified_days: 0, separate: None },
        service: None,
        more: HomeSaleMore::Nothing,
    }
}
// The paper, then its answers, as the client saves them; the first refusal is the reply.
fn save_homes(s: &mut Interview, homes: Vec<HomePage>) -> Reply {
    if homes.is_empty() {
        let mut request = open_document_center(s);
        let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
        documents.common.investment_sales.documents.clear();
        return send(s, request);
    }
    let rows: Vec<IncomeDocumentItem> = homes
        .into_iter()
        .map(|h| {
            document(
                0,
                IncomeSource::InvestmentSale {
                    value: crate::InvestmentSaleDocument::Form1099S(home_sale_document(h.source())),
                },
            )
        })
        .collect();
    let request = crate::tests::support::client::income_pages::save_documents(s, rows.clone());
    let reply = send(s, request);
    if matches!(reply, Reply::Response(P::Problem { .. })) {
        return reply;
    }
    let request =
        crate::tests::support::client::income_pages::information_return_answers_for_topic(
            s,
            &rows,
            TaxSection::InvestmentSales,
        )
        .expect("the sale has its answers page");
    send(s, request)
}
fn home_id(d: &FederalDraft) -> i64 {
    d.income.iter().find(|e| matches!(e.income, FederalIncome::HomeSale { .. })).unwrap().id
}
// The sale saved alone on a fresh return of this status, and its result.
fn sold(status: FederalStatus, sale: HomePage) -> (Interview, FederalResult) {
    sold_with(status, sale, vec![])
}
// The same with other documents of the page saved beside it.
fn sold_with(
    status: FederalStatus,
    sale: HomePage,
    others: Vec<IncomeSource>,
) -> (Interview, FederalResult) {
    let mut s = Interview::new(TEST_USER, complete(status));
    let documents = [home_sale_income(sale.source())].into_iter().chain(others);
    save_each_topic(&mut s, documents.map(|d| document(0, d)).collect());
    let r = estimate(&s.draft);
    labelled(&r);
    (s, r)
}
// The Review's label path: every calculation id and printed row of the result
// has its label resource (the figures alone are read without it).
fn labelled(r: &FederalResult) {
    let _ = FederalEstimate::from(r.clone());
    let _ = crate::draft_return(r);
}
// The page reopens with the sale as saved, and saving it unchanged after a
// restart keeps the stored return.
fn round_trip(s: &Interview, sale: &HomePage) {
    let stored = s.draft.clone();
    let mut again = restart(stored.clone());
    let page = if sale.form_1099s {
        open_document_center(&mut again)
    } else {
        crate::tests::support::client::income_pages::open_topic(
            &mut again,
            TaxSection::InvestmentSales,
        )
    };
    let rows = page_documents(&page).to_vec();
    assert!(
        rows.iter().any(|row| row.document
            == expected_named_source(&s.draft, Owner::Taxpayer, home_sale_income(sale.source()))),
        "{rows:?}"
    );
    accepted(&send(&mut again, page));
    assert_eq!(again.draft, stored);
}

#[test]
fn fully_excluded_single_sale_through_the_page() {
    // Pub. 523 Worksheet 2: $450,000 - $25,000 - $200,000 = $225,000 gain.
    // Owned and lived in 2 of the 5 years, no earlier sale: Worksheet 1
    // limit $250,000, so all $225,000 is excluded and nothing is taxable.
    // With a 1099-S the sale is still reported (Schedule D instructions):
    // Form 8949 Part II box F, d $450,000, e $200,000, g -$250,000
    // ($25,000 expenses + $225,000 exclusion, codes EH), h $0.
    let baseline = supported(&complete(FederalStatus::Single));
    let sale = page(false);
    let (s, r) = sold(FederalStatus::Single, sale.clone());
    let id = home_id(&s.draft);
    assert_eq!(id, 2);
    let entry = &s.draft.income[1];
    assert_eq!(entry.label, "12 Synthetic Lane");
    let FederalIncome::HomeSale { data } = &entry.income else { panic!() };
    let data = &HomeSale::from(data);
    assert_eq!(data.term, GainTerm::Long);
    assert!(matches!(
        data.exclusion,
        HomeExclusion::Full {
            taxpayer: HomeUse { owned: 1826, lived: 1826, prior_exclusion: false, .. },
            spouse: None,
        }
    ));
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{r:?}");
    assert_eq!(cents(&r, "home_2_realized_gain"), 22_500_000);
    assert_eq!(cents(&r, "home_2_excluded_gain"), 22_500_000);
    assert_eq!(cents(&r, "home_2_taxable_gain"), 0);
    assert_eq!(cents(&r, "home_2_form_8949_proceeds"), 45_000_000);
    assert_eq!(cents(&r, "home_2_form_8949_basis"), 20_000_000);
    assert_eq!(cents(&r, "home_2_form_8949_adjustment"), -25_000_000);
    assert_eq!(cents(&r, "home_2_form_8949_gain"), 0);
    let explanation =
        &r.lines.iter().find(|l| l.id == "home_2_form_8949_adjustment").unwrap().explanation;
    assert!(
        explanation.english().contains("Part II box F")
            && explanation.english().contains("code EH")
    );
    assert_eq!(value(&r, "agi"), value(&baseline, "agi"));
    assert_eq!(r.tax, baseline.tax);
    round_trip(&s, &sale);

    // Without a 1099-S a fully excluded sale is not reported at all.
    let mut quiet = page(false);
    quiet.form_1099s = false;
    let (s, r) = sold(FederalStatus::Single, quiet.clone());
    assert_eq!(cents(&r, "home_2_excluded_gain"), 22_500_000);
    assert_eq!(cents(&r, "home_2_taxable_gain"), 0);
    assert!(!has_line(&r, "home_2_form_8949_proceeds"));
    assert!(
        r.review_notes.iter().any(|n| n.contains("not reported on Form 8949")),
        "{:?}",
        r.review_notes
    );
    assert_eq!(r.tax, baseline.tax);
    round_trip(&s, &quiet);
}

#[test]
fn accepted_1099s_establishes_reporting_without_a_topic_answer() {
    // 2026-10-26: Pub. 523, Reporting Your Home Sale: an actual Form 1099-S
    // requires reporting even when all gain is excluded. A false topic answer
    // cannot turn the accepted paper into a sale without a Form 1099-S.
    let mut source = home(false);
    source.facts_mut().form_1099s = false;
    let supplied = source.answers.clone();
    assert!(source.answers().unwrap().form_1099s);
    let natural = HomeSale::from(&source);
    assert!(natural.reported(0));
    source.set_use(supplied);
    assert!(source.answers().unwrap().form_1099s);
    let mut corrected = MainHomeSale::printed(source.form.paper().unwrap().clone());
    corrected.keep_answers(&source);
    assert_eq!(corrected.answers, source.answers);

    // Personal records retain their actual source even with copied home answers.
    let records = MainHomeSale::records(source.form(), source.answers().unwrap());
    assert!(!records.answers().unwrap().form_1099s);
    let natural = HomeSale::from(&records);
    assert!(!natural.reported(0));
    assert!(natural.reported(1));
}

#[test]
fn joint_sale_uses_the_500000_exclusion() {
    // Joint, both lived there the whole time, bought 2010-05-01, sold
    // 2026-07-01 for $1,200,000 with $60,000 expenses and a $500,000
    // basis: gain $640,000. Worksheet 1: both meet residence and look-back,
    // one meets ownership: $500,000. Taxable long-term gain $140,000.
    // AGI $100,000 + $140,000 = $240,000 (below the $250,000 NIIT
    // threshold); taxable income $240,000 - $32,200 = $207,800, ordinary
    // $67,800. QDCG worksheet: 0% on $98,900 - $67,800 = $31,100; 15% on
    // $108,900 = $16,335; Tax Table (joint) on $67,800: $7,643. Tax
    // $23,978.
    let mut sale = page(true);
    sale.acquired = 20100501;
    sale.closed = 20260701;
    sale.proceeds = 120_000_000;
    sale.expenses = 6_000_000;
    sale.basis = 50_000_000;
    let (s, r) = sold(FederalStatus::Joint, sale.clone());
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{r:?}");
    assert_eq!(cents(&r, "home_2_excluded_gain"), 50_000_000);
    assert_eq!(cents(&r, "home_2_taxable_gain"), 14_000_000);
    assert_eq!(value(&r, "capital_gain_10"), 140_000);
    assert_eq!(value(&r, "agi"), 240_000);
    assert_eq!(r.tax, Some(2_397_800));
    round_trip(&s, &sale);

    // Only one spouse meets the residence test: that spouse's $250,000
    // (section 121(b)(2)(B)), taxable $390,000.
    let mut one = sale.clone();
    one.spouse = HomeSpouse::Joint {
        lived: HomeLived::Days { days: 400 },
        earlier_sale: EarlierHomeSale::No,
    };
    let (_, r) = sold(FederalStatus::Joint, one);
    assert_eq!(cents(&r, "home_2_excluded_gain"), 25_000_000);
    assert_eq!(cents(&r, "home_2_taxable_gain"), 39_000_000);
}

#[test]
fn partial_exclusion_for_a_work_move() {
    // Single, bought 2025-06-01, sold 2026-06-01 (365 days, held exactly
    // one year: short-term) after a new job 70 miles from the home (the old
    // one was 10 miles away: at least 50 miles farther). Gain $500,000 -
    // $30,000 - $320,000 = $150,000. Worksheet 1 Section B: the shortest of
    // residence (365), ownership (365) and time since an earlier sale (none)
    // is 365; 365 / 730 x $250,000 = $125,000. Taxable short-term gain
    // $25,000; AGI $85,000; taxable income $68,900, all ordinary; Tax Table
    // (single) $9,876.
    let mut sale = page(false);
    sale.acquired = 20250601;
    sale.closed = 20260601;
    sale.proceeds = 50_000_000;
    sale.expenses = 3_000_000;
    sale.basis = 32_000_000;
    sale.reason = HomeSaleReason::Event {
        reason: HomeSaleTrigger::WorkMove { old_miles: 10, new_miles: 70 },
    };
    let (s, r) = sold(FederalStatus::Single, sale.clone());
    let FederalIncome::HomeSale { data } = &s.draft.income[1].income else { panic!() };
    let data = &HomeSale::from(data);
    assert_eq!(data.term, GainTerm::Short);
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{r:?}");
    assert_eq!(cents(&r, "home_2_excluded_gain"), 12_500_000);
    assert_eq!(cents(&r, "home_2_taxable_gain"), 2_500_000);
    assert_eq!(value(&r, "capital_gain_3"), 25_000);
    assert_eq!(value(&r, "agi"), 85_000);
    assert_eq!(r.tax, Some(987_600));
    let explanation =
        &r.lines.iter().find(|l| l.id == "home_2_form_8949_adjustment").unwrap().explanation;
    assert!(explanation.english().contains("Part I box C"));
    round_trip(&s, &sale);

    // A new job only 49 miles farther is not the work-move safe harbor.
    let mut near = sale.clone();
    near.reason = HomeSaleReason::Event {
        reason: HomeSaleTrigger::WorkMove { old_miles: 10, new_miles: 59 },
    };
    let (_, r) = sold(FederalStatus::Single, near);
    assert_eq!(r.state, ResultState::Unsupported);
}

#[test]
fn gain_above_the_exclusion_with_depreciation_and_nonqualified_use() {
    // Single, bought 2012-01-10, sold 2026-04-20: 5,214 days owned. Rented
    // part of the time: $20,000 straight-line depreciation after May 6,
    // 1997 and 869 days of nonqualified use after 2008. Gain $700,000 -
    // $40,000 - $280,000 = $380,000. Pub. 523 Worksheet 3: the $20,000
    // depreciation is taxable (unrecaptured section 1250 gain); of the other
    // $360,000, 869 / 5,214 = 1/6 ($60,000) is nonqualified; $300,000 could
    // be excluded, limited to $250,000. Taxable $130,000 long-term.
    // AGI $190,000; taxable income $173,900. Schedule D Tax Worksheet:
    // line 10 $130,000, line 11 $20,000, line 13 $110,000, line 14 $63,900,
    // lines 16-17 $49,450, line 18 $43,900, line 21 $63,900, line 22 $0,
    // line 30 $110,000 x 15% = $16,500, line 39 $0 (line 38 $20,000), Tax
    // Table on $63,900 $8,776: line 45 $25,276, below $34,334 on line 46.
    let mut sale = page(false);
    sale.acquired = 20120110;
    sale.closed = 20260420;
    sale.proceeds = 70_000_000;
    sale.expenses = 4_000_000;
    sale.basis = 28_000_000;
    sale.use_history =
        HomeRentalUse { depreciation: 2_000_000, nonqualified_days: 869, separate: None };
    let (s, r) = sold(FederalStatus::Single, sale.clone());
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{r:?}");
    assert_eq!(cents(&r, "home_2_depreciation_gain"), 2_000_000);
    assert_eq!(cents(&r, "home_2_nonqualified_gain"), 6_000_000);
    assert_eq!(cents(&r, "home_2_excluded_gain"), 25_000_000);
    assert_eq!(cents(&r, "home_2_taxable_gain"), 13_000_000);
    assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 20_000);
    assert_eq!(value(&r, "agi"), 190_000);
    assert_eq!(r.tax, Some(2_527_600));
    round_trip(&s, &sale);
}

// The Form 4797 lookback history of a return with no business-property loss.
fn no_property_losses() -> Vec<IncomeSource> {
    vec![IncomeSource::Record {
        value: crate::IncomeRecord::BusinessPropertyLosses(BusinessPropertyLosses {
            regular: vec![],
            amt: vec![],
        }),
    }]
}

fn duty_service(suspended_days: u32, added_lived: u32, spouse: Option<u32>) -> HomeService {
    HomeService {
        election: HomeServiceElection {
            service: HomeDutyKind::Uniformed {
                duty: QualifiedHomeDuty { extended_orders: true, remote_or_quarters: true },
            },
            one_property: true,
            suspended_days,
        },
        added_lived,
        added_care: None,
        spouse_added_lived: spouse,
        spouse_added_care: None,
    }
}

#[test]
fn separate_rental_space_is_a_form_4797_sale_and_the_rest_is_the_home() {
    // Single, bought 2015-03-01, sold 2026-06-15 for $600,000 with $30,000
    // of expenses and a $360,000 adjusted basis (after $40,000 of
    // straight-line depreciation on the rented unit). The unit is 1,000 of
    // 4,000 square feet (Pub. 523, Business Use or Rental of Home).
    // Unit: price $150,000, expenses $7,500, cost ($360,000 + $40,000) / 4 =
    // $100,000, basis $60,000, gain $82,500 on Form 4797 line 2 with $40,000
    // of unrecaptured section 1250 gain. Home: price $450,000, expenses
    // $22,500, basis $300,000, gain $127,500, all excluded.
    // AGI $60,000 + $82,500 = $142,500; taxable income $126,400. Schedule D
    // Tax Worksheet: ordinary $43,900 plus the $40,000 unrecaptured gain is
    // $83,900, Tax Table (midpoint $83,925) $5,800 + 22% x $33,525 =
    // $13,176; the other $42,500 at 15% is $6,375: tax $19,551, below the
    // $22,934 rate-schedule tax on $126,400.
    let mut sale = page(false);
    sale.proceeds = 60_000_000;
    sale.expenses = 3_000_000;
    sale.basis = 36_000_000;
    sale.use_history.separate = Some(SeparateHomePart {
        kind: SeparateUse::Rental { carried_loss: 0 },
        area: 1000,
        whole_area: 4000,
        depreciation: SourceCents::Reported { amount: 4_000_000 },
    });
    let none = no_property_losses;
    let (s, r) = sold_with(FederalStatus::Single, sale.clone(), none());
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{r:?}");
    assert_eq!(cents(&r, "home_2_separate_price"), 15_000_000);
    assert_eq!(cents(&r, "home_2_separate_expenses"), 750_000);
    assert_eq!(cents(&r, "home_2_separate_basis"), 6_000_000);
    assert_eq!(cents(&r, "home_2_separate_gain"), 8_250_000);
    assert_eq!(cents(&r, "home_2_realized_gain"), 12_750_000);
    assert_eq!(cents(&r, "home_2_excluded_gain"), 12_750_000);
    assert_eq!(cents(&r, "home_2_taxable_gain"), 0);
    assert_eq!(cents(&r, "home_2_form_8949_proceeds"), 45_000_000);
    assert_eq!(printed_line(&r, "home_2_separate_gain"), (TaxForm::Form4797, "2".into()));
    assert_eq!(value(&r, "form4797_line8"), 82_500);
    assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 40_000);
    assert_eq!(value(&r, "agi"), 142_500);
    assert_eq!(r.tax, Some(1_955_100));
    round_trip(&s, &sale);

    // Without the depreciation known the share of the price cannot be split.
    let mut unknown = sale.clone();
    unknown.use_history.separate.as_mut().unwrap().depreciation = SourceCents::Unavailable;
    let (_, r) = sold_with(FederalStatus::Single, unknown, none());
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(has_issue(&r, "ISSUE_HOME_SALE_SEPARATE_DEPRECIATION_UNKNOWN"), "{r:?}");

    // A carried passive loss is released by the sale (Form 8582); a business
    // space has none to carry.
    let mut carried = sale.clone();
    carried.use_history.separate.as_mut().unwrap().kind = SeparateUse::Rental { carried_loss: 100 };
    let (_, r) = sold_with(FederalStatus::Single, carried, none());
    assert!(has_issue(&r, "ISSUE_HOME_SALE_SEPARATE_PASSIVE_LOSS"), "{r:?}");
    let mut business = sale.clone();
    business.use_history.separate.as_mut().unwrap().kind = SeparateUse::Business;
    let (_, r) = sold_with(FederalStatus::Single, business, none());
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{r:?}");
    assert_eq!(value(&r, "form4797_line8"), 82_500);

    // A loss on the unit is a section 1231 loss, deducted in full: a $400,000
    // price with $20,000 of expenses and a $600,000 basis gives the unit a
    // $100,000 price, $5,000 of expenses and ($600,000 + $40,000) / 4 -
    // $40,000 = $120,000 of basis, a $25,000 loss. The home's own loss is not deductible. AGI $60,000 - $25,000.
    let mut loss = sale;
    loss.proceeds = 40_000_000;
    loss.expenses = 2_000_000;
    loss.basis = 60_000_000;
    let (_, r) = sold_with(FederalStatus::Single, loss, none());
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{r:?}");
    assert_eq!(cents(&r, "home_2_separate_gain"), -2_500_000);
    assert_eq!(value(&r, "form4797_line8"), -25_000);
    assert_eq!(value(&r, "agi"), 35_000);
}

#[test]
fn service_duty_suspends_the_five_year_test_through_the_page() {
    // Single, bought 2012-01-10, sold 2026-06-15 (5,270 days owned) for
    // $600,000 with $25,000 of expenses and a $300,000 basis: gain $275,000.
    // Lived there 900 days in the 1,096 days before the 5 years and none
    // since, on qualified official extended duty. Section 121(d)(9): with
    // the 1,096-day suspension the test period reaches back to those days,
    // so ownership (2,922) and residence (900) meet 730 and $250,000 is
    // excluded; $25,000 is taxable long-term gain.
    let mut sale = page(false);
    sale.acquired = 20120110;
    sale.proceeds = 60_000_000;
    sale.expenses = 2_500_000;
    sale.basis = 30_000_000;
    sale.lived = HomeLived::Days { days: 0 };
    sale.service = Some(duty_service(1096, 900, None));
    let (s, r) = sold(FederalStatus::Single, sale.clone());
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{r:?}");
    assert_eq!(cents(&r, "home_2_excluded_gain"), 25_000_000);
    assert_eq!(cents(&r, "home_2_taxable_gain"), 2_500_000);
    round_trip(&s, &sale);

    // A prior property already used the suspension: only the ordinary
    // 5 years count, in which nobody lived there.
    let mut other = sale.clone();
    other.service.as_mut().unwrap().election.one_property = false;
    let (_, r) = sold(FederalStatus::Single, other);
    assert_eq!(r.state, ResultState::Unsupported);

    // Joint, a work move: each spouse lived there 400 of the added days, so
    // the reduced exclusion is 400 / 730 x $250,000 each, $273,972.60. The
    // gain is $300,000 ($625,000 - $25,000 - $300,000); $26,027.40 is taxable.
    let mut joint = sale;
    joint.proceeds = 62_500_000;
    joint.spouse =
        HomeSpouse::Joint { lived: HomeLived::Days { days: 0 }, earlier_sale: EarlierHomeSale::No };
    joint.reason = HomeSaleReason::Event {
        reason: HomeSaleTrigger::WorkMove { old_miles: 10, new_miles: 70 },
    };
    joint.service = Some(duty_service(1096, 400, Some(400)));
    let (s, r) = sold(FederalStatus::Joint, joint.clone());
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{r:?}");
    assert_eq!(cents(&r, "home_2_excluded_gain"), 27_397_260);
    assert_eq!(cents(&r, "home_2_taxable_gain"), 2_602_740);
    round_trip(&s, &joint);
}

#[test]
fn a_loss_on_the_home_is_not_deductible() {
    // $300,000 - $20,000 - $350,000: a $70,000 loss. Nothing is deducted;
    // with a 1099-S, Form 8949 shows (d) $300,000, e $350,000 and (g)
    // +$50,000 (codes EL), h $0.
    let baseline = supported(&complete(FederalStatus::Single));
    let mut sale = page(false);
    sale.proceeds = 30_000_000;
    sale.expenses = 2_000_000;
    sale.basis = 35_000_000;
    let (_, r) = sold(FederalStatus::Single, sale);
    assert_eq!(cents(&r, "home_2_realized_gain"), -7_000_000);
    assert_eq!(cents(&r, "home_2_taxable_gain"), 0);
    assert_eq!(cents(&r, "home_2_form_8949_adjustment"), 5_000_000);
    let explanation =
        &r.lines.iter().find(|l| l.id == "home_2_form_8949_adjustment").unwrap().explanation;
    assert!(explanation.english().contains("code EL"), "{}", explanation.english());
    assert_eq!(r.tax, baseline.tax);
}

#[test]
fn ownership_and_use_boundary_is_exactly_two_years() {
    // Bought 2024-06-01, sold 2026-06-01: 730 days (2024 is a leap year),
    // exactly 2 years: the full exclusion. Bought a day later: 729 days,
    // no full exclusion without a reason, named by the estimate.
    let mut sale = page(false);
    sale.acquired = 20240601;
    sale.closed = 20260601;
    let (s, r) = sold(FederalStatus::Single, sale.clone());
    let FederalIncome::HomeSale { data } = &s.draft.income[1].income else { panic!() };
    let data = &HomeSale::from(data);
    assert!(matches!(
        data.exclusion,
        HomeExclusion::Full { taxpayer: HomeUse { owned: 730, lived: 730, .. }, .. }
    ));
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{r:?}");
    assert_eq!(cents(&r, "home_2_taxable_gain"), 0);
    sale.acquired = 20240602;
    let (_, r) = sold(FederalStatus::Single, sale.clone());
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(
        r.issues.iter().any(|i| i.message.contains("full ownership/residence exclusion")),
        "{r:?}"
    );
    // Lived in it 730 of the days owned (bought earlier): full; 729: not.
    sale.acquired = 20200101;
    sale.lived = HomeLived::Days { days: 730 };
    assert_eq!(
        sold(FederalStatus::Single, sale.clone()).1.state,
        ResultState::EstimatedForSupportedScope
    );
    sale.lived = HomeLived::Days { days: 729 };
    assert_eq!(sold(FederalStatus::Single, sale).1.state, ResultState::Unsupported);
}

#[test]
fn exclusion_limit_boundary_is_250000_dollars() {
    // Gain exactly $250,000.00: all excluded. $250,000.01: one cent taxable.
    let mut sale = page(false);
    sale.proceeds = 47_500_000;
    let (_, r) = sold(FederalStatus::Single, sale.clone());
    assert_eq!(cents(&r, "home_2_realized_gain"), 25_000_000);
    assert_eq!(cents(&r, "home_2_taxable_gain"), 0);
    sale.proceeds = 47_500_001;
    let (_, r) = sold(FederalStatus::Single, sale.clone());
    assert_eq!(cents(&r, "home_2_excluded_gain"), 25_000_000);
    assert_eq!(cents(&r, "home_2_taxable_gain"), 1);
    // Joint: $500,000.00 excluded, $500,000.01 leaves one cent.
    let mut sale = page(true);
    sale.proceeds = 72_500_000;
    let (_, r) = sold(FederalStatus::Joint, sale.clone());
    assert_eq!(cents(&r, "home_2_taxable_gain"), 0);
    sale.proceeds = 72_500_001;
    let (_, r) = sold(FederalStatus::Joint, sale);
    assert_eq!(cents(&r, "home_2_taxable_gain"), 1);
}

#[test]
fn earlier_excluded_sale_boundary_is_two_years() {
    // An earlier excluded sale 730 days before this one (2024-06-15 to
    // 2026-06-15) is outside the look-back: full exclusion. 729 days: the
    // look-back fails; with a qualifying event the reduced exclusion uses
    // 729 / 730 of $250,000 = $249,657.53 (rounded to the cent).
    let mut sale = page(false);
    sale.proceeds = 55_000_000; // gain $325,000
    sale.earlier_sale = EarlierHomeSale::Yes { sold: 20240615 };
    let (_, r) = sold(FederalStatus::Single, sale.clone());
    assert_eq!(cents(&r, "home_2_excluded_gain"), 25_000_000);
    sale.earlier_sale = EarlierHomeSale::Yes { sold: 20240616 };
    let (_, r) = sold(FederalStatus::Single, sale.clone());
    assert_eq!(r.state, ResultState::Unsupported);
    sale.reason = HomeSaleReason::Event { reason: HomeSaleTrigger::MedicalCare };
    let (_, r) = sold(FederalStatus::Single, sale);
    assert_eq!(cents(&r, "home_2_excluded_gain"), 24_965_753);
}

#[test]
fn holding_period_boundary_is_more_than_one_year() {
    // Sold on the first anniversary: short-term; a day later: long-term.
    let mut sale = page(false);
    sale.acquired = 20250615;
    sale.reason = HomeSaleReason::Event { reason: HomeSaleTrigger::Unemployment };
    let term = |sale: &HomePage| sale.record().term;
    assert_eq!(term(&sale), GainTerm::Short);
    sale.acquired = 20250614;
    assert_eq!(term(&sale), GainTerm::Long);
}

#[test]
fn surviving_spouse_and_decline_through_the_page() {
    // A widow (single return) sells within 2 years of the death, both met
    // the tests just before it: $500,000. Gain $325,000 is all excluded.
    let mut sale = page(false);
    sale.proceeds = 55_000_000;
    sale.more = HomeSaleMore::SurvivingSpouse {
        died: 20250301,
        both_qualified: true,
        spouse_earlier_sale: false,
        late_acquired: None,
        late_lived: 0,
    };
    let (s, r) = sold(FederalStatus::Single, sale.clone());
    assert_eq!(cents(&r, "home_2_excluded_gain"), 32_500_000);
    round_trip(&s, &sale);
    // Reporting the whole gain instead: $325,000 taxable.
    sale.more = HomeSaleMore::Decline;
    let (s, r) = sold(FederalStatus::Single, sale.clone());
    assert_eq!(cents(&r, "home_2_taxable_gain"), 32_500_000);
    round_trip(&s, &sale);
}

#[test]
fn an_inherited_gift_or_spouse_transfer_is_long_term_from_the_earlier_owner() {
    // Received 2026-02-01 and sold 2026-06-15 for $450,000 with a $430,000
    // basis, reporting the whole $20,000 gain (no two-year test is met): a
    // purchase is short-term, but an inherited home is always long-term
    // (section 1223(9)) and a gift or spouse transfer adds the earlier
    // owner's holding period (section 1223(2)).
    let sold_as = |received| {
        let mut sale = page(false);
        sale.acquired = 20260201;
        sale.proceeds = 45_000_000;
        sale.expenses = 0;
        sale.basis = 43_000_000;
        sale.received = received;
        sale.more = HomeSaleMore::Decline;
        let (s, r) = sold(FederalStatus::Single, sale.clone());
        let FederalIncome::HomeSale { data } = &s.draft.income[1].income else { panic!() };
        let data = &HomeSale::from(data);
        assert_eq!(cents(&r, "home_2_taxable_gain"), 2_000_000);
        round_trip(&s, &sale);
        data.term
    };
    assert_eq!(sold_as(HomeReceived::Bought), GainTerm::Short);
    assert_eq!(sold_as(HomeReceived::Inherited), GainTerm::Long);
    assert_eq!(
        sold_as(HomeReceived::Gift {
            given_acquired: 20200101,
            got_by_exchange: PriorExchange::No
        }),
        GainTerm::Long
    );
    assert_eq!(
        sold_as(HomeReceived::FromSpouse {
            spouse_acquired: 20200101,
            agreement_days: 0,
            got_by_exchange: PriorExchange::No
        }),
        GainTerm::Long
    );
}

#[test]
fn a_transfer_from_a_spouse_counts_their_ownership_and_agreed_use() {
    // Transferred by a former spouse on 2024-09-01, who bought it 2010-05-01;
    // sold 2026-06-15 for $500,000 with $20,000 of expenses and a $200,000
    // carryover basis: gain $280,000. Section 121(d)(3): the former spouse's
    // ownership counts as yours (the 5-year window is whole), and the days the
    // divorce agreement let them live there count as your use. 600 days of
    // your own + 130 = 730 meets the 2-year test: $250,000 excluded and $30,000
    // taxable; with 129 days it does not.
    let mut sale = page(false);
    sale.acquired = 20240901;
    sale.proceeds = 50_000_000;
    sale.expenses = 2_000_000;
    sale.basis = 20_000_000;
    sale.lived = HomeLived::Days { days: 600 };
    sale.received = HomeReceived::FromSpouse {
        spouse_acquired: 20100501,
        agreement_days: 130,
        got_by_exchange: PriorExchange::No,
    };
    let (s, r) = sold(FederalStatus::Single, sale.clone());
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{r:?}");
    assert_eq!(cents(&r, "home_2_excluded_gain"), 25_000_000);
    assert_eq!(cents(&r, "home_2_taxable_gain"), 3_000_000);
    round_trip(&s, &sale);
    sale.received = HomeReceived::FromSpouse {
        spouse_acquired: 20100501,
        agreement_days: 129,
        got_by_exchange: PriorExchange::No,
    };
    let (_, r) = sold(FederalStatus::Single, sale);
    assert_eq!(r.state, ResultState::Unsupported);
}

#[test]
fn a_surviving_spouse_counts_the_late_spouses_ownership_and_use() {
    // The late spouse bought the home 2012-01-10 and died 2025-03-01, leaving
    // it to the widow, who sells 2026-06-15 for $550,000 with $25,000 of
    // expenses and a $200,000 basis: gain $325,000. Section 121(d)(2): the
    // late spouse's ownership counts (window whole), and their 1,111 days there
    // before the widow moved in add to her 714: 1,825 days. Sold within 2 years
    // of the death with both tests met just before it: $500,000 excluded.
    // Counting only her own 471 days of ownership, she does not qualify.
    let mut sale = page(false);
    sale.acquired = 20250301;
    sale.proceeds = 55_000_000;
    sale.basis = 20_000_000;
    sale.lived = HomeLived::Days { days: 714 };
    sale.received = HomeReceived::Inherited;
    sale.more = HomeSaleMore::SurvivingSpouse {
        died: 20250301,
        both_qualified: true,
        spouse_earlier_sale: false,
        late_acquired: Some(20120110),
        late_lived: 1111,
    };
    let (s, r) = sold(FederalStatus::Single, sale.clone());
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{r:?}");
    assert_eq!(cents(&r, "home_2_excluded_gain"), 32_500_000);
    round_trip(&s, &sale);
    sale.more = HomeSaleMore::SurvivingSpouse {
        died: 20250301,
        both_qualified: true,
        spouse_earlier_sale: false,
        late_acquired: None,
        late_lived: 0,
    };
    sale.lived = HomeLived::WholeTime;
    let (_, r) = sold(FederalStatus::Single, sale);
    assert_eq!(r.state, ResultState::Unsupported);
}

#[test]
fn installment_sale_of_a_home_reports_form_6252_after_the_exclusion() {
    // Sold 2026-06-15 for $600,000 with $30,000 of expenses and a $200,000
    // basis after $30,000 of straight-line depreciation; the buyer paid
    // $100,000 in 2026 and owes the rest. Gain $370,000: the $30,000 of
    // depreciation cannot be excluded, and of the other $340,000 the
    // exclusion is $250,000 (Pub. 523, Installment Sale). Form 6252: line 8
    // $230,000, line 9 $30,000, line 13 $230,000, line 14 $370,000, line 15
    // $250,000, line 16 $120,000, line 18 contract price $600,000; the gross
    // profit percentage is 20%, so line 24 is $100,000 x 20% = $20,000 on
    // Schedule D line 11, all unrecaptured section 1250 gain (recognized
    // first). AGI $60,000 + $20,000.
    let mut sale = page(false);
    sale.proceeds = 60_000_000;
    sale.expenses = 3_000_000;
    sale.basis = 20_000_000;
    sale.use_history =
        HomeRentalUse { depreciation: 3_000_000, nonqualified_days: 0, separate: None };
    sale.payment = SalePayment::Installment {
        assumed_debt: 0,
        payments: 10_000_000,
        interest: NoteInterest::Adequate,
        special: false,
    };
    let (s, r) = sold(FederalStatus::Single, sale.clone());
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{r:?}");
    assert_eq!(cents(&r, "home_2_excluded_gain"), 25_000_000);
    assert_eq!(value(&r, "home_2_form6252_cost"), 230_000);
    assert_eq!(value(&r, "home_2_form6252_exclusion"), 250_000);
    assert_eq!(printed_line(&r, "home_2_form6252_profit"), (TaxForm::Form6252, "14".into()));
    assert_eq!(
        crate::tests::support::assert::displayed_line_amount(&r, "home_2_form6252_profit"),
        37_000_000,
    );
    crate::tests::support::assert::assert_displayed_line_ratio(
        &r,
        "home_2_form6252_percentage",
        2_000,
        "0.2000",
    );
    assert_eq!(printed_line(&r, "home_2_form6252_percentage"), (TaxForm::Form6252, "19".into()));
    assert_eq!(value(&r, "home_2_form6252_gross_profit"), 120_000);
    assert_eq!(value(&r, "home_2_form6252_contract_price"), 600_000);
    assert_eq!(value(&r, "home_2_form6252_income"), 20_000);
    assert_eq!(cents(&r, "home_2_taxable_gain"), 2_000_000);
    assert_eq!(printed_line(&r, "home_2_form6252_exclusion"), (TaxForm::Form6252, "15".into()));
    assert_eq!(
        r.lines
            .iter()
            .filter(|line| { line.form_line == Some(FormLine::new(TaxForm::Form6252, "15")) })
            .count(),
        1,
    );
    assert_eq!(
        crate::tests::support::assert::displayed_line_amount(&r, "home_2_form6252_exclusion"),
        25_000_000,
    );
    assert_eq!(value(&r, "capital_gain_11"), 20_000);
    assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 20_000);
    assert_eq!(value(&r, "agi"), 80_000);
    assert!(!has_line(&r, "home_2_form_8949_proceeds"));
    round_trip(&s, &sale);

    // A note below the applicable federal rate has unstated interest.
    let SalePayment::Installment { interest, .. } = &mut sale.payment else { panic!() };
    *interest = NoteInterest::Low;
    let (_, r) = sold(FederalStatus::Single, sale);
    assert!(has_issue(&r, "ISSUE_INSTALLMENT_UNSTATED_INTEREST"), "{r:?}");
}

#[test]
fn a_home_got_in_a_like_kind_exchange_has_no_exclusion_within_five_years() {
    // Got by exchange 2021-06-15 and sold for $450,000 with $25,000 of
    // expenses and a $200,000 basis: gain $225,000. Section 121(d)(10) bars the
    // 5-year period beginning with the acquisition, which ends 2026-06-14: sold
    // that day none is excluded; on the fifth anniversary all of it is.
    let sold_on = |closed| {
        let mut sale = page(false);
        sale.acquired = 20210615;
        sale.closed = closed;
        sale.proceeds = 45_000_000;
        sale.expenses = 2_500_000;
        sale.basis = 20_000_000;
        sale.received = HomeReceived::LikeKindExchange { given_up_acquired: 20100101 };
        let (s, r) = sold(FederalStatus::Single, sale.clone());
        round_trip(&s, &sale);
        r
    };
    let r = sold_on(20260614);
    assert_eq!(cents(&r, "home_2_excluded_gain"), 0);
    assert_eq!(cents(&r, "home_2_taxable_gain"), 22_500_000);
    assert!(noted(&r, "NOTE_HOME_SALE_LIKE_KIND_NO_EXCLUSION"), "{:?}", r.review_notes);
    let r = sold_on(20260615);
    assert_eq!(cents(&r, "home_2_excluded_gain"), 22_500_000);
    assert_eq!(cents(&r, "home_2_taxable_gain"), 0);
    assert_eq!(r.tax, supported(&complete(FederalStatus::Single)).tax);
}

#[test]
fn a_gift_or_spouse_transfer_of_an_exchanged_home_keeps_the_five_year_bar() {
    // Section 121(d)(10) reaches a home whose basis comes from property the
    // giver or spouse got in a like-kind exchange: sold 2026-06-15 for
    // $450,000 with $25,000 of expenses and a $200,000 basis (gain $225,000),
    // received 2025-01-01 from a spouse who got it by exchange on 2023-06-15
    // (within 5 years): nothing is excluded. Their exchange on 2021-06-15 is
    // 5 years back to the day, outside the period: all is. Unknown within the 5 years is a named
    // issue, never assumed; unknown outside them does not matter.
    let sold_after = |exchanged: u32, answer| {
        let mut sale = page(false);
        sale.acquired = 20250101;
        sale.lived = HomeLived::Days { days: 400 };
        sale.proceeds = 45_000_000;
        sale.expenses = 2_500_000;
        sale.basis = 20_000_000;
        sale.received = HomeReceived::FromSpouse {
            spouse_acquired: exchanged,
            agreement_days: 400,
            got_by_exchange: answer,
        };
        let (s, r) = sold(FederalStatus::Single, sale.clone());
        round_trip(&s, &sale);
        r
    };
    let r = sold_after(20230615, PriorExchange::Yes);
    assert_eq!(cents(&r, "home_2_excluded_gain"), 0);
    assert_eq!(cents(&r, "home_2_taxable_gain"), 22_500_000);
    assert!(noted(&r, "NOTE_HOME_SALE_LIKE_KIND_NO_EXCLUSION"), "{:?}", r.review_notes);
    let r = sold_after(20210615, PriorExchange::Yes);
    assert_eq!(cents(&r, "home_2_excluded_gain"), 22_500_000);
    assert!(has_issue(
        &sold_after(20230615, PriorExchange::Unknown),
        "ISSUE_HOME_SALE_PRIOR_EXCHANGE_UNKNOWN"
    ));
    assert_eq!(
        cents(&sold_after(20210615, PriorExchange::Unknown), "home_2_excluded_gain"),
        22_500_000
    );
}

#[test]
fn a_foreign_sellers_withholding_is_a_credit_on_line_25c() {
    // Price $450,000 and basis $150,000: gain $300,000, $250,000 excluded,
    // $50,000 taxable. Box 7: the buyer withheld 15% of $450,000 = $67,500
    // (Form 8288-A), a credit on Form 1040 line 25c.
    let mut sale = page(false);
    sale.proceeds = 45_000_000;
    sale.expenses = 0;
    sale.basis = 15_000_000;
    sale.foreign_person = true;
    sale.firpta_withheld = SourceCents::Reported { amount: 6_750_000 };
    let baseline = supported(&complete(FederalStatus::Single));
    let (s, r) = sold(FederalStatus::Single, sale.clone());
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{r:?}");
    assert_eq!(cents(&r, "home_2_taxable_gain"), 5_000_000);
    assert_eq!(cents(&r, "home_2_firpta_withholding"), 6_750_000);
    assert_eq!(return_line(&r, "25c"), return_line(&baseline, "25c") + 6_750_000);
    round_trip(&s, &sale);

    // Unknown is not zero: the credit stays open.
    sale.firpta_withheld = SourceCents::Unavailable;
    let (_, r) = sold(FederalStatus::Single, sale);
    assert!(has_issue(&r, "ISSUE_HOME_SALE_FIRPTA_WITHHOLDING_UNKNOWN"), "{r:?}");
}

#[test]
fn property_received_is_counted_once_beside_the_box_2a_price() {
    // 1099-S box 2a shows $400,000, which leaves out property and services
    // (Form 1099-S instructions); box 6 is checked for $50,000 of property. The
    // amount realized is $450,000, not $500,000: with a $150,000 basis the gain
    // is $300,000 and $50,000 is taxable (without the property, $250,000 all
    // excluded).
    let mut sale = page(false);
    sale.proceeds = 40_000_000;
    sale.expenses = 0;
    sale.basis = 15_000_000;
    sale.property_received = true;
    sale.property_value = SourceCents::Reported { amount: 5_000_000 };
    let (s, r) = sold(FederalStatus::Single, sale.clone());
    let FederalIncome::HomeSale { data } = &s.draft.income[1].income else { panic!() };
    let data = &HomeSale::from(data);
    assert_eq!(data.price, 45_000_000);
    assert_eq!(cents(&r, "home_2_realized_gain"), 30_000_000);
    assert_eq!(cents(&r, "home_2_taxable_gain"), 5_000_000);
    assert_eq!(cents(&r, "home_2_form_8949_proceeds"), 45_000_000);
    round_trip(&s, &sale);

    // Unknown is not zero: the value stays open.
    sale.property_value = SourceCents::Unavailable;
    let (_, r) = sold(FederalStatus::Single, sale);
    assert!(has_issue(&r, "ISSUE_HOME_SALE_PROPERTY_VALUE_UNKNOWN"), "{r:?}");
}

#[test]
fn exchanging_a_former_home_for_like_kind_property_taxes_only_the_boot_after_the_exclusion() {
    // A former home, rented since 2024 and exchanged 2026-06-15 for property
    // worth $650,000 plus $350,000 in cash: price $1,000,000, $20,000 of
    // exchange expenses, a $300,000 basis after $30,000 of depreciation, and
    // 1,100 days lived in it in the 5 years. Gain $680,000; the $30,000 of
    // depreciation cannot be excluded, so $250,000 is. Form 8824: line 15
    // $350,000 - $20,000 = $330,000, line 16 $650,000, line 18 $300,000,
    // line 19 $680,000, line 20 $330,000 - $250,000 = $80,000 recognized
    // (Rev. Proc. 2005-14), line 24 $600,000 deferred, line 25 $300,000 + $80,000
    // - $330,000 + $250,000 = $300,000 of basis. A rental's $80,000 is a section
    // 1231 gain with $30,000 of unrecaptured section 1250 gain.
    let mut sale = page(false);
    sale.proceeds = 100_000_000;
    sale.expenses = 2_000_000;
    sale.basis = 30_000_000;
    sale.lived = HomeLived::Days { days: 1100 };
    sale.use_history =
        HomeRentalUse { depreciation: 3_000_000, nonqualified_days: 0, separate: None };
    sale.exchange = Some(HomeExchange { value: 65_000_000, held_for: ExchangeUse::Rental });
    let (s, r) = sold_with(FederalStatus::Single, sale.clone(), no_property_losses());
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{r:?}");
    assert_eq!(cents(&r, "home_2_excluded_gain"), 25_000_000);
    for (line, expected) in [
        ("15", 330_000),
        ("16", 650_000),
        ("18", 300_000),
        ("19", 680_000),
        ("20", 80_000),
        ("24", 600_000),
        ("25", 300_000),
    ] {
        assert_eq!(
            value(&r, &alloc::format!("home_2_form8824_line{line}")),
            expected,
            "line {line}"
        );
    }
    assert_eq!(printed_line(&r, "home_2_form8824_line20"), (TaxForm::Form8824, "20".into()));
    // The rental part's recognized gain is the Form 4797 line 5 row.
    assert_eq!(value(&r, "home_2_exchange_section1231"), 80_000);
    assert_eq!(printed_line(&r, "home_2_exchange_section1231"), (TaxForm::Form4797, "5".into()));
    assert_eq!(value(&r, "form4797_line8"), 80_000);
    assert_eq!(value(&r, "form4797_line10"), 80_000);
    assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 30_000);
    assert_eq!(value(&r, "agi"), 140_000);
    round_trip(&s, &sale);

    // Held for investment, the same $80,000 is a long-term capital gain.
    sale.exchange = Some(HomeExchange { value: 65_000_000, held_for: ExchangeUse::Investment });
    let (_, r) = sold(FederalStatus::Single, sale);
    assert_eq!(value(&r, "capital_gain_11"), 80_000);
    assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 30_000);
    assert_eq!(value(&r, "agi"), 140_000);
}

#[test]
fn rare_parts_are_left_out_with_a_notice_and_expatriation_refused() {
    // Answers and unknowns: a named situation the page does not calculate
    // leaves the sale out with its notice; the expatriation tax, an added
    // tax, is refused by name.
    let refused = HomePage {
        more: HomeSaleMore::Refused { situation: HomeSaleSpecial::Expatriate },
        ..page(false)
    };
    let (s, r) = sold(FederalStatus::Single, refused.clone());
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(r.issues.iter().any(|i| i.message.contains("Form 8854")), "{r:?}");
    round_trip(&s, &refused);
    let situation =
        |situation| HomePage { more: HomeSaleMore::Refused { situation }, ..page(false) };
    for (sale, key) in [
        (
            situation(HomeSaleSpecial::NonStraightLineDepreciation),
            "WARN_HOME_SALE_DEPRECIATION_LEFT_OUT",
        ),
        (situation(HomeSaleSpecial::Other), "WARN_HOME_SALE_OTHER_LEFT_OUT"),
    ] {
        let (s, r) = sold(FederalStatus::Single, sale.clone());
        assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{key}: {r:?}");
        assert!(noted(&r, key), "{key}: {r:?}");
        assert!(!r.lines.iter().any(|l| l.id.starts_with("home_2_")), "{key}");
        // The page states the situation in its facts; it never stores a
        // Not sure placeholder in the sale's ordinary-treatment answer.
        let FederalIncome::HomeSale { data } = &s.draft.income.last().unwrap().income else {
            panic!()
        };
        let data = &HomeSale::from(data);
        assert!(data.ordinary);
        round_trip(&s, &sale);
    }
}

#[test]
fn home_sale_page_rules_are_checked_on_save() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let bad = |f: &dyn Fn(&mut HomePage)| {
        let mut v = page(false);
        f(&mut v);
        v
    };
    for (sale, key) in [
        (bad(&|v| v.closed = 20251231), "HOME_SALE_NOT_IN_CURRENT_YEAR"),
        (bad(&|v| v.acquired = 20260615), "HOME_ACQUIRED_AFTER_SALE"),
        (
            bad(&|v| {
                v.acquired = 20250615;
                v.lived = HomeLived::Days { days: 366 }
            }),
            "HOME_LIVED_ABOVE_OWNED",
        ),
        (
            bad(&|v| {
                v.lived = HomeLived::CareFacility {
                    home_days: 1000,
                    facility_days: 827,
                    owned_residence_days: 1000,
                }
            }),
            "HOME_LIVED_ABOVE_OWNED",
        ),
        (
            bad(&|v| v.earlier_sale = EarlierHomeSale::Yes { sold: 20260615 }),
            "HOME_EARLIER_SALE_AFTER_SALE",
        ),
        (
            bad(&|v| {
                v.acquired = 20250615;
                v.use_history =
                    HomeRentalUse { depreciation: 0, nonqualified_days: 366, separate: None }
            }),
            "HOME_NONQUALIFIED_ABOVE_OWNED",
        ),
        (
            bad(&|v| {
                v.more = HomeSaleMore::SurvivingSpouse {
                    died: 20260616,
                    both_qualified: true,
                    spouse_earlier_sale: false,
                    late_acquired: None,
                    late_lived: 0,
                }
            }),
            "HOME_DEATH_AFTER_SALE",
        ),
        (
            bad(&|v| {
                v.more = HomeSaleMore::SurvivingSpouse {
                    died: 20250301,
                    both_qualified: true,
                    spouse_earlier_sale: false,
                    late_acquired: None,
                    late_lived: 0,
                };
                v.reason = HomeSaleReason::Event { reason: HomeSaleTrigger::Divorce }
            }),
            "HOME_SURVIVING_WITH_REASON",
        ),
        (
            bad(&|v| {
                v.use_history.separate = Some(SeparateHomePart {
                    kind: SeparateUse::Business,
                    area: 4000,
                    whole_area: 4000,
                    depreciation: SourceCents::NotReported,
                })
            }),
            "HOME_SEPARATE_AREA_ABOVE_WHOLE",
        ),
        (
            bad(&|v| {
                v.use_history.separate = Some(SeparateHomePart {
                    kind: SeparateUse::Business,
                    area: 1000,
                    whole_area: 4000,
                    depreciation: SourceCents::Reported { amount: 100_000_000 },
                })
            }),
            "HOME_SEPARATE_DEPRECIATION_ABOVE_COST",
        ),
        (
            bad(&|v| v.service = Some(duty_service(1096, 0, Some(0)))),
            "HOME_SERVICE_SPOUSE_NOT_JOINT",
        ),
        (
            bad(&|v| {
                v.acquired = 20250615;
                v.service = Some(duty_service(1096, 1, None))
            }),
            "HOME_SERVICE_LIVED_ABOVE_OWNED",
        ),
        (
            bad(&|v| {
                v.received = HomeReceived::Gift {
                    given_acquired: 20150302,
                    got_by_exchange: PriorExchange::No,
                }
            }),
            "HOME_PRIOR_OWNER_AFTER_ACQUIRED",
        ),
        (
            bad(&|v| {
                v.lived = HomeLived::Days { days: 1800 };
                v.received = HomeReceived::FromSpouse {
                    spouse_acquired: 20100501,
                    agreement_days: 100,
                    got_by_exchange: PriorExchange::No,
                }
            }),
            "HOME_LIVED_ABOVE_OWNED",
        ),
        (page(true), "HOME_SPOUSE_NOT_JOINT"),
    ] {
        let reply = save_homes(&mut s, vec![sale]);
        rejected_with(reply, key);
    }
    let mut joint = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    rejected_with(save_homes(&mut joint, vec![page(false)]), "HOME_SPOUSE_NEEDED");
    // The page offers exactly the spouse answer the return asks.
    let P::SaveDocuments { allowed_home_spouse, allowed_documents, .. } =
        open_document_center(&mut joint)
    else {
        unreachable!()
    };
    assert_eq!(allowed_home_spouse, HomeSpouse::allowed(true));
    assert!(
        allowed_documents.contains(
            &(libertas_macros::variant_index!(crate::IncomeSourceKind::Form1099S) as i32)
        )
    );
    let P::SaveDocuments { allowed_home_spouse, .. } = open_document_center(&mut s) else {
        unreachable!()
    };
    assert_eq!(allowed_home_spouse, HomeSpouse::allowed(false));
}

#[test]
fn home_sale_is_no_longer_a_less_common_refusal() {
    assert!(!Unprepared::all().iter().any(|item| item.refused().lines.contains("main home")),);
}

#[test]
fn leaving_joint_erases_the_spouses_home_answers_and_joining_reopens_the_page() {
    let mut d = complete(FederalStatus::Joint);
    let mut s = Interview::new(TEST_USER, d.clone());
    let mut sale = page(true);
    sale.spouse = HomeSpouse::Joint {
        lived: HomeLived::WholeTime,
        earlier_sale: EarlierHomeSale::Yes { sold: 20250101 },
    };
    let mut service = duty_service(1096, 365, Some(365));
    service.added_care =
        Some(crate::home_sale::HomeServiceCare { facility_days: 365, owned_residence_days: 365 });
    service.spouse_added_care = service.added_care.clone();
    sale.service = Some(service);
    accepted(&save_homes(&mut s, vec![sale]));
    d = crate::interview::with_ticks(s.draft.clone());
    d.finished_sections.push(TaxSection::InvestmentSales);
    let mut stored = d.clone();
    assert_eq!(crate::interview::prune(&mut stored), Default::default());
    d.setup.as_mut().unwrap().status = filing_choice(FederalStatus::Single);
    d.people.as_mut().unwrap().spouse = SpouseFiler::No;
    let pruned = crate::interview::prune(&mut d);
    // The spouse's home answers go with the spouse; the sale itself stays.
    assert_eq!(pruned.removed.len(), 1, "{pruned:?}");
    let mut expected = stored.income.clone();
    for entry in &mut expected {
        if let FederalIncome::HomeSale { data } = &mut entry.income {
            let answers = data.facts_mut();
            answers.spouse = HomeSpouse::NotJoint;
            let service = answers.service.as_mut().unwrap();
            service.spouse_added_lived = None;
            service.spouse_added_care = None;
        }
    }
    assert_eq!(d.income, expected);
    for entry in &mut d.income {
        if let FederalIncome::HomeSale { data } = &mut entry.income {
            assert!(data.answers_issue(false).is_none());
            assert!(!data.drop_spouse());
        }
    }
    // Becoming joint keeps the paper and the taxpayer's answers, but the
    // spouse's home history is now owed on Investment sales.
    let joint = complete(FederalStatus::Joint);
    d.setup = joint.setup;
    d.people = joint.people;
    let income = d.income.clone();
    crate::interview::prune(&mut d);
    assert_eq!(d.income, income);
    let result = estimate(&d);
    assert!(result.tax.is_none());
    assert!(has_issue(
        &result,
        "ISSUE_HOME_SALE_PROVIDE_BOTH_SPOUSES__HOME_HISTORIES_FOR_325E7C69",
    ));
    let mut s = Interview::new(TEST_USER, d.clone());
    let mut request = crate::tests::support::client::income_pages::open_topic(
        &mut s,
        TaxSection::InvestmentSales,
    );
    rejected_with(send(&mut s, request.clone()), "HOME_SPOUSE_NEEDED");
    assert_eq!(s.draft, d);
    let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
    let answers = followups
        .iter_mut()
        .find_map(|row| match &mut row.answer {
            crate::DocumentAnswer::HomeSale { value } => {
                value.as_mut().map(crate::RealEstateSaleUse::facts_mut)
            }
            _ => None,
        })
        .expect("the home facts reopen with the spouse's history unanswered");
    answers.spouse =
        HomeSpouse::Joint { lived: HomeLived::WholeTime, earlier_sale: EarlierHomeSale::No };
    rejected_with(send(&mut s, request), "HOME_SERVICE_SPOUSE_NEEDED");
    assert_eq!(s.draft, d);
}

// ---------------------------------------------------------------------------
// Form 8828 on a sale: the Other taxes item's hidden home sale ID is filled
// from the return's home sales.

fn subsidized_sale() -> MortgageSubsidyEntry {
    MortgageSubsidyEntry {
        label: "Maple Street".into(),
        disposition: MortgageSubsidyDisposition::Ordinary {
            subsidy: MortgageSubsidyKind::Mcc,
            transfer: MortgageSubsidyTransfer::Sale {
                home_sale_income_id: 0,
                qsml_gain_share: None,
            },
            facts: ordinary_subsidy_facts(),
        },
    }
}
// Bought 2020-01-15 with the subsidized loan, sold 2026-07-01 (the Form
// 8828 disposition date) for $400,000 with $20,000 expenses and a $300,000
// basis: an $80,000 gain, all excluded.
fn subsidized_home() -> HomePage {
    HomePage {
        closed: 20260701,
        acquired: 20200115,
        proceeds: 40_000_000,
        expenses: 2_000_000,
        basis: 30_000_000,
        ..page(false)
    }
}
fn bound_id(d: &FederalDraft) -> i64 {
    d.credits.as_ref().unwrap().mortgage_subsidies[0].home_sale_income_id().unwrap()
}

#[test]
fn form_8828_recapture_on_a_sale_through_both_pages() {
    // Form 8828: line 13 gain $80,000 (the section 121 exclusion does not
    // reduce it); line 14 $40,000; line 15 modified AGI $60,000 (the W-2;
    // the excluded gain is not in AGI); line 16 $50,000; line 17 $10,000;
    // line 18 $10,000 / $5,000 = 100% (capped); line 19 $100,000 x 6.25% =
    // $6,250; line 20 60% -> line 21 $3,750; line 22 $3,750; line 23 the
    // smaller of $40,000 and $3,750: $3,750 to Schedule 2 line 13b.
    let baseline = supported(&complete(FederalStatus::Single));
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    accepted(&save_homes(&mut s, vec![subsidized_home()]));
    accepted(&save_other_taxes(
        &mut s,
        vec![OtherTax::MortgageSubsidyRecapture { homes: vec![subsidized_sale().into()] }],
    ));
    assert_eq!(bound_id(&s.draft), home_id(&s.draft));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "mortgage_subsidy_1_disposition_gain"), 80_000);
    assert_eq!(value(&r, "mortgage_subsidy_1_modified_income"), 60_000);
    assert_eq!(value(&r, "mortgage_subsidy_recapture_tax"), 3_750);
    assert_eq!(r.tax, baseline.tax.map(|t| t + 375_000));
    // Round trip of both pages.
    let stored = s.draft.clone();
    let mut again = restart(stored.clone());
    let page = open_other_taxes(&mut again);
    let P::SaveOtherTaxes { value: shown, .. } = &page else { unreachable!() };
    let request = other_taxes_request(&page, &again.draft.clone(), shown.taxes.clone());
    accepted(&send(&mut again, request));
    assert_eq!(again.draft, stored);
    round_trip(&again, &subsidized_home());
    // A closing date other than the disposition date is named.
    let mut other_day = subsidized_home();
    other_day.closed = 20260630;
    accepted(&save_homes(&mut s, vec![other_day]));
    let r = estimate(&s.draft);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(r.issues.iter().any(|i| i.section == "Mortgage subsidy recapture"), "{r:?}");
}

#[test]
fn form_8828_sale_entered_before_the_home_sale_binds_on_the_next_save() {
    // The Form 8828 row saved first has no home sale (ID 0, named by the
    // estimate); saving the home sale binds it without reopening Other
    // taxes, and removing the home sale unbinds it.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    accepted(&save_other_taxes(
        &mut s,
        vec![OtherTax::MortgageSubsidyRecapture { homes: vec![subsidized_sale().into()] }],
    ));
    assert_eq!(bound_id(&s.draft), 0);
    assert_eq!(estimate(&s.draft).state, ResultState::Unsupported);
    accepted(&save_homes(&mut s, vec![subsidized_home()]));
    assert_eq!(bound_id(&s.draft), home_id(&s.draft));
    assert_eq!(value(&supported(&s.draft), "mortgage_subsidy_recapture_tax"), 3_750);
    accepted(&save_homes(&mut s, vec![]));
    assert_eq!(bound_id(&s.draft), 0);
}

// G-13 (Smith round 3): the printed Form 1099-S of 1099s-main-home.pdf exactly
// as AI Import fills it. The form never states how the property was used:
// blank boxes stay blank (zero, empty text, no box 8d date).
fn smith_1099s() -> crate::Form1099SPaper {
    crate::Form1099SPaper {
        calendar_year: 2026,
        corrected: false,
        filer: crate::PrintedParty {
            name: "Example Grove Settlement Services".into(),
            federal_id: "00-0000209".into(),
            address: "8 Example Trust Plaza\nExample Grove, VA US 00000".into(),
        },
        filer_telephone: "202-555-0109".into(),
        transferor: crate::PrintedParty {
            name: "John Smith".into(),
            federal_id: "987-65-4320".into(),
            address: "100 Example Orchard Lane\nExample Grove, VA US 00000".into(),
        },
        account_number: "CLOSE-2026-01".into(),
        box_1_date_of_closing: 20260615,
        box_2a_total_gross_proceeds: 42_000_000,
        box_2b_cash_gross_proceeds: 42_000_000,
        box_2c_digital_asset_gross_proceeds: 0,
        box_3_address_or_legal_description: "90 Example Grove Lane, Example Grove, VA 00000".into(),
        box_4_buyers_part_of_real_estate_tax: 120_000,
        box_6_property_or_services: false,
        box_7_foreign_person: false,
        box_8a_digital_asset_code: String::new(),
        box_8b_digital_asset_name: String::new(),
        box_8c_digital_asset_units: 0.0,
        box_8d_digital_asset_date: None,
    }
}

// The Document Center collects the paper alone; Investment sales then asks
// what was sold. Returns the session and the topic page with that question.
// The household's taxpayer stands in for the printed transferor John Smith.
fn smith_paper(d: &FederalDraft) -> crate::Form1099SPaper {
    let mut paper = smith_1099s();
    paper.transferor.name =
        crate::tests::support::builders::fixture_document_names(d, -1).unwrap().recipient.name;
    paper
}
fn smith_collected() -> (Interview, P) {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let row = document(
        0,
        IncomeSource::InvestmentSale {
            value: crate::InvestmentSaleDocument::Form1099S(smith_paper(&s.draft)),
        },
    );
    let request = crate::tests::support::client::income_pages::save_documents(&mut s, vec![row]);
    accepted(&send(&mut s, request));
    let FederalIncome::HomeSale { data } = &s.draft.income[home_index(&s.draft)].income else {
        unreachable!()
    };
    assert_eq!(data.form.paper(), Some(&smith_paper(&s.draft)), "the paper is kept as printed");
    assert_eq!(data.answers, None, "the Document Center asks nothing about the sale");
    let page = crate::tests::support::client::income_pages::open_topic(
        &mut s,
        TaxSection::InvestmentSales,
    );
    (s, page)
}
fn home_index(d: &FederalDraft) -> usize {
    d.income.iter().position(|e| matches!(e.income, FederalIncome::HomeSale { .. })).unwrap()
}
// The page with what was sold answered.
fn sold_as(mut page: P, sold: crate::RealEstateSaleUse) -> P {
    let P::SaveIncomes { followups, .. } = &mut page else { unreachable!() };
    let row = followups
        .iter_mut()
        .find(|row| matches!(row.answer, crate::DocumentAnswer::HomeSale { .. }))
        .expect("Investment sales asks what the Form 1099-S sale was");
    row.answer = crate::DocumentAnswer::HomeSale { value: Some(sold) };
    page
}
fn smith_home_facts() -> HomeSaleAnswers {
    let crate::RealEstateSaleUse::MainHome(answers) = page(false).source().answers.unwrap() else {
        unreachable!()
    };
    answers
}

#[test]
fn smith_1099s_is_collected_as_printed_and_its_use_asked_on_investment_sales() {
    // The import adapter's classifier: every 1099-S is the neutral Form1099S
    // kind, and neither the kind nor the paper asks the AI to decide its use.
    let center = include_str!("../../federal/document_center.rs");
    assert!(center.contains(
        "Identify every actual Form 1099-S as Form1099S, whatever property it reports; the form does not state how the property was used, so never decide or infer its use."
    ));
    assert!(!center.contains("main-home 1099-S"));
    let kinds = include_str!("../../federal/income_editor.rs");
    let kind = &kinds[..kinds.find("    Form1099S(crate::Form1099SPaper),").unwrap()];
    let kind = &kind[kind.rfind("    /// Form 1099-S\n").unwrap()..];
    assert!(kind.contains("Proceeds from a real estate transaction, as printed"), "{kind}");
    assert!(!kind.contains("lived in"), "{kind}");
    let paper = include_str!("../../home/home_sale.rs");
    assert!(paper.contains("Do not sum distinct forms, derive amounts, or infer basis, expenses, residence, ownership or tax treatment."));

    let baseline = supported(&complete(FederalStatus::Single));
    let (mut s, page) = smith_collected();
    let id = s.draft.income[home_index(&s.draft)].id;
    assert_eq!(s.draft.income[home_index(&s.draft)].owner, crate::DocumentOwner::Yours);
    // Unanswered: a pending question on Investment sales, nothing estimated.
    let P::SaveIncomes { followups, .. } = &page else { unreachable!() };
    assert!(
        followups.iter().any(|row| row.answer == crate::DocumentAnswer::HomeSale { value: None })
    );
    let r = estimate(&s.draft);
    assert!(
        crate::interview::remaining_items(&s.draft, &r, TaxSection::InvestmentSales)
            .iter()
            .any(|row| row.section == TaxSection::InvestmentSales && row.field.is_some()),
        "the unanswered use is pending on its page"
    );
    assert_eq!(value(&r, "agi"), value(&baseline, "agi"));

    // Your main home: the Pub. 523 worksheet. $420,000 - $25,000 - $200,000
    // = $195,000 gain, all excluded; the 1099-S still puts it on Form 8949.
    let main = sold_as(page.clone(), crate::RealEstateSaleUse::MainHome(smith_home_facts()));
    let mut home = restart(s.draft.clone());
    accepted(&send(&mut home, main));
    let r = supported(&home.draft);
    assert_eq!(cents(&r, &format!("home_{id}_realized_gain")), 19_500_000);
    assert_eq!(cents(&r, &format!("home_{id}_taxable_gain")), 0);
    assert_eq!(cents(&r, &format!("home_{id}_form_8949_proceeds")), 42_000_000);
    assert_eq!(r.tax, baseline.tax);
    let mut again = restart(home.draft.clone());
    let reopened = crate::tests::support::client::income_pages::open_topic(
        &mut again,
        TaxSection::InvestmentSales,
    );
    accepted(&send(&mut again, reopened));
    assert_eq!(again.draft, home.draft);

    // Land held for investment: Form 8949 box F, $420,000 less the $300,000
    // basis and $20,000 selling expenses (code E) is a $100,000 long-term gain.
    let land = crate::OtherRealEstateSale {
        held: crate::RealEstateHolding::Investment,
        term: GainTerm::Long,
        basis: 30_000_000,
        expenses: 2_000_000,
    };
    accepted(&send(&mut s, sold_as(page.clone(), crate::RealEstateSaleUse::Other(land.clone()))));
    let r = supported(&s.draft);
    assert_eq!(cents(&r, &format!("real_estate_{id}_gain")), 10_000_000);
    assert_eq!(value(&r, "capital_gain"), 100_000);
    assert!(!has_line(&r, &format!("home_{id}_realized_gain")));
    // A second home sold at a loss: not deductible.
    let second = crate::OtherRealEstateSale {
        held: crate::RealEstateHolding::Personal,
        basis: 45_000_000,
        ..land
    };
    let page = crate::tests::support::client::income_pages::open_topic(
        &mut s,
        TaxSection::InvestmentSales,
    );
    accepted(&send(&mut s, sold_as(page, crate::RealEstateSaleUse::Other(second))));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "capital_gain"), 0);
    assert_eq!(value(&r, "agi"), value(&baseline, "agi"));
}

#[test]
fn smith_1099s_for_a_rental_is_counted_on_the_rental_page() {
    // A rental's 1099-S adds nothing here; its sale on the Rentals page at
    // the box 2a price is the one Form 4797 sale.
    let (mut s, page) = smith_collected();
    accepted(&send(&mut s, sold_as(page, crate::RealEstateSaleUse::Business)));
    let r = estimate(&s.draft);
    assert!(has_issue(&r, "ISSUE_REAL_ESTATE_SALE_PRICE"), "{:?}", r.issues);
    let mut rental_only = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let rental = crate::tests::cross::interview::rental_sold_page(20260615, 42_000_000);
    crate::tests::support::assert::accepted(&crate::tests::cross::interview::save_rentals(
        &mut rental_only,
        vec![rental.clone()],
    ));
    crate::tests::support::assert::accepted(&crate::tests::cross::interview::save_rentals(
        &mut s,
        vec![rental],
    ));
    let r = estimate(&s.draft);
    assert!(!has_issue(&r, "ISSUE_REAL_ESTATE_SALE_PRICE"), "{:?}", r.issues);
    assert_eq!(r.tax, estimate(&rental_only.draft).tax);
}
