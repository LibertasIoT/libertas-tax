// Form 8824 (unit B2): a rental exchanged for other real property, and an
// earlier-year related-party exchange. Fixtures are synthetic, 2026 Form 8824
// draft lines, hand-computed in the comments.
use super::*;
use crate::{
    EarlierRelatedExchange, ExchangeParty, ExchangeReplacement, PropertyCharacter, PropertyUse,
    RelatedDisposal, RelatedException, RelatedExchange, RentalExchange, ReplacementUse,
};

// The rental given up: one building, land inside its cost, dollars.
fn given_up(
    kind: RentalPropertyType,
    cost: i64,
    land: i64,
    in_service: u32,
    rent: i64,
    expenses: i64,
) -> FederalRental {
    let mut v = rental(rent, expenses);
    v.property_type = kind;
    v.buildings = vec![purchased_rental_building(cost * 100, land * 100, in_service)];
    v
}
fn rental_use(kind: RentalPropertyType, in_service: u32) -> ReplacementUse {
    ReplacementUse::Rental { kind, in_service }
}
// An exchange of unrelated parties with no money fields; the caller sets
// them. Dollars for the values.
fn exchange(
    transferred: u32,
    identified: u32,
    received: u32,
    value: i64,
    land: i64,
    used: ReplacementUse,
    land_value: i64,
) -> RentalExchange {
    RentalExchange {
        transferred,
        identified,
        received,
        extended: None,
        replacement: ExchangeReplacement {
            address: "9 Oak Lane".into(),
            value: value * 100,
            land: land * 100,
            used,
        },
        cash_received: 0,
        other_received: 0,
        debt_relieved: 0,
        debt_assumed: 0,
        cash_paid: 0,
        expenses: 0,
        land_value: land_value * 100,
        party: ExchangeParty::Unrelated,
        special: false,
    }
}
// A single filer with the wages and the rental; returns the rental's id.
fn exchange_return(wages: i64, property: FederalRental) -> (FederalDraft, i64) {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, wages * 100, 0)];
    add(&mut d, income(FederalIncome::Rental { data: property }));
    d.screening.as_mut().unwrap().section1231 =
        Some(Section1231History { regular: vec![], amt: vec![] });
    let id = d.income.last().unwrap().id;
    (d, id)
}
fn exchanged(mut property: FederalRental, x: RentalExchange) -> FederalRental {
    property.sale = crate::RentalSale::Exchanged(x);
    property
}
fn row_value(r: &FederalResult, id: i64, suffix: &str) -> i64 {
    value(r, &alloc::format!("rental_{id}_exchange_{suffix}"))
}
fn has_row(r: &FederalResult, id: &str) -> bool {
    r.lines.iter().any(|l| l.id == id)
}

// Fixture A. Single, W-2 $70,000. Rental (single-family) cost $300,000, land
// $60,000, first available 2016-03-15; rent $30,000 and expenses $12,000.
// Exchanged 2026-06-20; identified 2026-07-25 (day 35); received 2026-09-10
// (day 82); the property received is worth $520,000 ($120,000 land), rented
// from 2026-09-15; debt relieved $150,000, taken on $200,000, expenses
// $28,000, land part $90,000. No boot.
fn fixture_a() -> (FederalDraft, i64) {
    let mut x = exchange(
        20260620,
        20260725,
        20260910,
        520_000,
        120_000,
        rental_use(RentalPropertyType::SingleFamily, 20260915),
        90_000,
    );
    x.debt_relieved = 150_000 * 100;
    x.debt_assumed = 200_000 * 100;
    x.expenses = 28_000 * 100;
    exchange_return(
        70_000,
        exchanged(
            given_up(RentalPropertyType::SingleFamily, 300_000, 60_000, 20160315, 30_000, 12_000),
            x,
        ),
    )
}

#[test]
fn exchange_no_boot_carries_basis_and_depreciates_the_excess() {
    let (d, id) = fixture_a();
    let r = supported(&d);
    // Depreciation given up, 27.5-year mid-month on $240,000: 2016 19
    // half-months $6,909.09, 2017-2025 nine years $8,727.27 each, 2026
    // through June (11 half-months) $4,000.00: $89,454.55 -> $89,455.
    // B = 60,000 + (240,000 - 89,455) = $210,545.
    // 15 = positive(0 + 0 + positive(150,000 - 200,000) - 28,000) = 0
    // 16 = 520,000; 17 = 520,000
    // 18 = 210,545 + (200,000 - 150,000) + 28,000 = $288,545
    // 19 = 231,455; 20 = min(0, 231,455) = 0; 21 = 0; 22 = 0; 23 = 0
    // 24 = 231,455 deferred; 25 = 288,545 + 0 - 0 = $288,545 (= 520,000 - 231,455)
    // 25a = 288,545 x 400,000 / 520,000 = $221,958.
    for (suffix, amount) in [
        ("line15", 0),
        ("line16", 520_000),
        ("line17", 520_000),
        ("line18", 288_545),
        ("line19", 231_455),
        ("line20", 0),
        ("line21", 0),
        ("line22", 0),
        ("line23", 0),
        ("line24", 231_455),
        ("line25", 288_545),
        ("line25a", 221_958),
    ] {
        assert_eq!(row_value(&r, id, suffix), amount, "{suffix}");
    }
    assert_eq!(
        printed_line(&r, &alloc::format!("rental_{id}_exchange_line25a")),
        (TaxForm::Form8824, "25a".into())
    );
    assert!(!has_row(&r, &alloc::format!("rental_{id}_exchange_section1231")));
    assert!(!noted(&r, "WARN_EXCHANGE_TAXED_AS_SALE"));
    // E = min(221,958, G = 150,545) = 150,545 exchanged basis; excess 71,413.
    // Replacement from 2026-09-15 (7 half-months): carryover 150,545 x 7 /
    // (660 - 246) = $2,545.45 (= 240,000 x 7 / 660, as if not transferred);
    // excess 71,413 x 7 / 660 = $757.41. Schedule E: 4,000.00 + 2,545.45 +
    // 757.41 = 7,302.86 -> $7,303.
    // 2026-10-19, D-2026-10-19-01: exact cents; whole-dollar form output.
    assert_eq!(line_amount(&r, &alloc::format!("rental_{id}_depreciation")), 730_286);
    assert_eq!(displayed_line_amount(&r, &alloc::format!("rental_{id}_depreciation")), 730_300);
    assert_eq!(value(&r, &alloc::format!("rental_{id}_replacement_carryover_depreciation")), 2_545);
    assert_eq!(value(&r, &alloc::format!("rental_{id}_replacement_excess_depreciation")), 757);
    assert_eq!(
        printed_line(&r, &alloc::format!("rental_{id}_replacement_carryover_depreciation")),
        (TaxForm::Form4562, "17".into())
    );
    assert_eq!(
        printed_line(&r, &alloc::format!("rental_{id}_replacement_excess_depreciation")),
        (TaxForm::Form4562, "19i".into())
    );
    // Schedule E net = 30,000 - 12,000 - 7,303 = $10,697; AGI = $80,697.
    // 1040 line 15 = 80,697 - 16,100 = $64,597; Tax Table row 64,550-64,600:
    // 1,240 + 4,560 + 22% x (64,575 - 50,400) = $8,919.
    assert_eq!(value(&r, "agi"), 80_697);
    assert_eq!(r.tax, Some(891_900));
    // The §469(g) release does not run and nothing reaches Form 4797.
    assert_eq!(value(&r, "form4797_line8"), 0);
    // Reopening the stored return shows the same figures.
    assert_eq!(supported(&restart(d.clone()).draft), r);
}

// Fixture B. Single, W-2 $50,000. Rental (single-family) cost $200,000, land
// $40,000, first available 2012-01-10; rent $24,000, expenses $6,000.
// Exchanged 2026-04-15; received 2026-05-20 (day 35, so identified the same
// day); the property received is worth $300,000 ($75,000 land), a
// multi-family rental from 2026-06-01; cash $25,000, debt relieved $120,000,
// taken on $100,000, expenses $15,000, land part $70,000.
fn fixture_b(
    transferred: u32,
    identified: u32,
    received: u32,
    extended: Option<bool>,
) -> (FederalDraft, i64) {
    let mut x = exchange(
        transferred,
        identified,
        received,
        300_000,
        75_000,
        rental_use(RentalPropertyType::MultiFamily, received.max(20260601)),
        70_000,
    );
    x.extended = extended;
    x.cash_received = 25_000 * 100;
    x.debt_relieved = 120_000 * 100;
    x.debt_assumed = 100_000 * 100;
    x.expenses = 15_000 * 100;
    exchange_return(
        50_000,
        exchanged(
            given_up(RentalPropertyType::SingleFamily, 200_000, 40_000, 20120110, 24_000, 6_000),
            x,
        ),
    )
}

#[test]
fn exchange_boot_with_liabilities_is_unrecaptured_section_1250_gain() {
    // The property received is first rented 2026-06-01.
    let (d, id) = fixture_b(20260415, 20260520, 20260520, None);
    let r = supported(&d);
    // Depreciation given up: 160,000 x (23 + 13 x 24 + 7) / 660 = $82,909
    // (2026 part $1,697). B = 40,000 + 77,091 = $117,091.
    // Net liabilities positive(120,000 - 100,000) = 20,000.
    // 15 = 25,000 + 0 + 20,000 - 15,000 = $30,000 (expenses used 15,000)
    // 16 = 300,000; 17 = 330,000; 18 = 117,091 + 0 + 0 = $117,091
    // 19 = 212,909; 20 = min(30,000, 212,909) = 30,000; 21 = 0; 22 = 30,000
    // 23 = 30,000; 24 = 182,909; 25 = 117,091 + 30,000 - 30,000 = $117,091
    // (= 300,000 - 182,909). 25a = 117,091 x 225,000 / 300,000 = $87,818.
    for (suffix, amount) in [
        ("line15", 30_000),
        ("line16", 300_000),
        ("line17", 330_000),
        ("line18", 117_091),
        ("line19", 212_909),
        ("line20", 30_000),
        ("line21", 0),
        ("line22", 30_000),
        ("line23", 30_000),
        ("line24", 182_909),
        ("line25", 117_091),
        ("line25a", 87_818),
    ] {
        assert_eq!(row_value(&r, id, suffix), amount, "{suffix}");
    }
    // Form 4797 line 5: the section 1231 gain; the unrecaptured §1250 gain is
    // min(30,000, min(82,909, building gain 260,000 - 77,091 = 182,909)).
    assert_eq!(
        printed_line(&r, &alloc::format!("rental_{id}_exchange_section1231")),
        (TaxForm::Form4797, "5".into())
    );
    assert_eq!(row_value(&r, id, "section1231"), 30_000);
    assert_eq!(row_value(&r, id, "unrecaptured_1250"), 30_000);
    assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 30_000);
    // With no prior section 1231 losses the gain is net section 1231 gain.
    assert_eq!(value(&r, "form4797_line8"), 30_000);
    // E = min(87,818, 77,091) = 77,091; excess 10,727. From 2026-06-01 (13
    // half-months): carryover 77,091 x 13 / (660 - 342) = $3,151.52, excess
    // 10,727 x 13 / 660 = $211.29. Schedule E: 1,696.97 + 3,151.52 + 211.29
    // = 5,059.78 -> $5,060.
    // 2026-10-19, D-2026-10-19-01: exact cents; whole-dollar form output.
    assert_eq!(line_amount(&r, &alloc::format!("rental_{id}_depreciation")), 505_978);
    assert_eq!(displayed_line_amount(&r, &alloc::format!("rental_{id}_depreciation")), 506_000);
    // AGI = 50,000 + (24,000 - 6,000 - 5,060) + 30,000 = $92,940.
    assert_eq!(value(&r, "agi"), 92_940);
}

#[test]
fn exchange_commercial_building_that_lost_value_has_no_unrecaptured_gain() {
    // Single. Commercial rental cost $500,000, land $100,000, available
    // 2019-07-01; rent $50,000, expenses $20,000. Exchanged 2026-03-10;
    // identified 2026-04-20 (day 41); received 2026-08-01; the property
    // received is worth $450,000 ($150,000 land), a commercial rental from
    // 2026-08-01; cash $120,000, expenses $20,000, land part $220,000.
    let mut x = exchange(
        20260310,
        20260420,
        20260801,
        450_000,
        150_000,
        rental_use(RentalPropertyType::Commercial, 20260801),
        220_000,
    );
    x.cash_received = 120_000 * 100;
    x.expenses = 20_000 * 100;
    let (d, id) = exchange_return(
        60_000,
        exchanged(
            given_up(RentalPropertyType::Commercial, 500_000, 100_000, 20190701, 50_000, 20_000),
            x,
        ),
    );
    let r = supported(&d);
    // Depreciation given up (Table A-7a): 400,000 x (1.177% + 6 x 2.564% +
    // 2.564% x 5/24) = $68,381 (2026 part $2,137). B = 100,000 + 331,619 =
    // $431,619.
    // 15 = 120,000 - 20,000 = 100,000; 16 = 450,000; 17 = 550,000; 18 =
    // 431,619; 19 = 118,381; 20 = 100,000.
    // 21: a 39-year straight-line building has no additional depreciation.
    // 22 = 100,000 -> Form 4797 line 5; 23 = 100,000; 24 = 18,381;
    // 25 = 431,619 (= 450,000 - 18,381).
    // Amount realized 550,000; building part 550,000 - 220,000 = 330,000;
    // building gain 330,000 - 331,619 = -1,619, so the unrecaptured §1250
    // gain is max(0, min(68,381, -1,619)) = 0.
    for (suffix, amount) in [
        ("line15", 100_000),
        ("line16", 450_000),
        ("line17", 550_000),
        ("line18", 431_619),
        ("line19", 118_381),
        ("line20", 100_000),
        ("line21", 0),
        ("line22", 100_000),
        ("line23", 100_000),
        ("line24", 18_381),
        ("line25", 431_619),
        ("line25a", 287_746),
        ("section1231", 100_000),
        ("unrecaptured_1250", 0),
    ] {
        assert_eq!(row_value(&r, id, suffix), amount, "{suffix}");
    }
    assert_eq!(
        printed_line(&r, &alloc::format!("rental_{id}_exchange_line21")),
        (TaxForm::Form8824, "21".into())
    );
    assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 0);
    // 25a = 431,619 x 300,000 / 450,000 = 287,746; E = min(287,746, 331,619)
    // = 287,746 (limited, no excess). From 2026-08-01 (9 half-months) over
    // 936 - 160 = 776: 287,746 x 9 / 776 = $3,337.26. Schedule E: 2,136.67 +
    // 3,337.26 = 5,473.93 -> $5,474.
    // 2026-10-19, D-2026-10-19-01: exact cents; whole-dollar form output.
    assert_eq!(line_amount(&r, &alloc::format!("rental_{id}_depreciation")), 547_393);
    assert_eq!(
        displayed_line_amount(&r, &alloc::format!("rental_{id}_line18_depreciation")),
        547_400
    );
}

// The year-of-exchange related-party cases on fixture A.
fn fixture_a_with(party: ExchangeParty) -> (FederalDraft, i64) {
    let (mut d, id) = fixture_a();
    if let FederalIncome::Rental { data } = &mut d.income[1].income
        && let crate::RentalSale::Exchanged(x) = &mut data.sale
    {
        x.party = party;
    }
    (d, id)
}

#[test]
fn exchange_related_party_boundaries_and_exceptions() {
    let (base_d, base_id) = fixture_a();
    let base = supported(&base_d);
    // Both kept: fixture A plus the two-year note.
    let (d, id) = fixture_a_with(ExchangeParty::Related { disposal: RelatedDisposal::Kept });
    let r = supported(&d);
    assert!(noted(&r, "NOTE_RELATED_EXCHANGE_TWO_YEARS"));
    assert_eq!(row_value(&r, id, "line24"), 231_455);
    assert_eq!(r.tax, base.tax);
    // Each exception reproduces fixture A; the no-tax-avoidance one adds a
    // warning that the IRS must agree.
    for (exception, warned) in [
        (RelatedException::Death, false),
        (RelatedException::InvoluntaryConversion, false),
        (RelatedException::NoTaxAvoidance, true),
    ] {
        let (d, _) = fixture_a_with(ExchangeParty::Related {
            disposal: RelatedDisposal::TheyDisposed { exception },
        });
        let r = supported(&d);
        assert_eq!(noted(&r, "WARN_EXCHANGE_NO_TAX_AVOIDANCE"), warned);
        assert!(!noted(&r, "WARN_EXCHANGE_TAXED_AS_SALE"));
        assert_eq!(r.tax, base.tax);
        let (d, _) = fixture_a_with(ExchangeParty::RelatedSeller { exception });
        assert_eq!(supported(&d).tax, base.tax);
    }
    // The related person disposed of theirs with no exception: taxed as a
    // sale. Recognized = line 19 = $231,455: amount realized 442,000, building
    // part 442,000 - 90,000 = 352,000 less the building's 150,545 adjusted
    // basis = 201,455 (unrecaptured min(201,455, 89,455) = 89,455), land
    // 90,000 - 60,000 = 30,000. The property received costs $520,000: its
    // building 400,000 from 2026-09-15 depreciates 400,000 x 7 / 660 =
    // $4,242.42, so Schedule E depreciation is 4,000.00 + 4,242.42 = $8,242.
    let (d, id) = fixture_a_with(ExchangeParty::Related {
        disposal: RelatedDisposal::TheyDisposed { exception: RelatedException::None },
    });
    let r = supported(&d);
    assert!(noted(&r, "WARN_EXCHANGE_TAXED_AS_SALE"));
    assert_eq!(value(&r, &alloc::format!("rental_{id}_sale_building_gain")), 201_455);
    assert_eq!(value(&r, &alloc::format!("rental_{id}_sale_unrecaptured_1250")), 89_455);
    assert_eq!(value(&r, &alloc::format!("rental_{id}_sale_land")), 30_000);
    // Part III stays so that line 24 is reported this year; no basis lines.
    assert_eq!(row_value(&r, id, "line24"), 231_455);
    assert!(!has_row(&r, &alloc::format!("rental_{id}_exchange_line25")));
    assert_eq!(value(&r, &alloc::format!("rental_{id}_depreciation")), 8_242);
    // A related seller with no exception is the same sale, without Part III.
    let (d, id) =
        fixture_a_with(ExchangeParty::RelatedSeller { exception: RelatedException::None });
    let r = supported(&d);
    assert!(noted(&r, "WARN_EXCHANGE_TAXED_AS_SALE"));
    assert!(!has_row(&r, &alloc::format!("rental_{id}_exchange_line19")));
    assert_eq!(value(&r, &alloc::format!("rental_{id}_sale_building_gain")), 201_455);
    // Not section 469(g): a related buyer's sale releases no suspended loss.
    let (mut d, id) = fixture_a_with(ExchangeParty::Related {
        disposal: RelatedDisposal::TheyDisposed { exception: RelatedException::None },
    });
    if let FederalIncome::Rental { data } = &mut d.income[1].income {
        data.history =
            RentalHistory { regular: 400_000 * 100, amt: 400_000 * 100, ..RentalHistory::NONE };
    }
    let r = supported(&d);
    // Own net 30,000 - 12,000 - 8,242 = 9,758, less the 400,000 prior loss,
    // plus the 231,455 gain is still negative (-158,787): passive, so only
    // the $25,000 allowance... the loss above the allowance stays suspended.
    assert!(value(&r, &alloc::format!("rental_{id}_suspended_loss")) > 0);
    // Your own disposal of the property you received is refused by name.
    let (d, _) = fixture_a_with(ExchangeParty::Related { disposal: RelatedDisposal::YouDisposed });
    let r = estimate(&d);
    assert!(r.issues.iter().any(|i| i.display.key == "ISSUE_EXCHANGE_REPLACEMENT_DISPOSED"));
    let _ = base_id;
}

fn related_return(wages: i64, exchanges: Vec<EarlierRelatedExchange>) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, wages * 100, 0)];
    let screening = d.screening.as_mut().unwrap();
    screening.related_exchanges = exchanges;
    screening.section1231 = Some(Section1231History { regular: vec![], amt: vec![] });
    d
}
fn earlier_exchange(disposed: u32, exception: RelatedException) -> EarlierRelatedExchange {
    EarlierRelatedExchange {
        owner: Owner::Taxpayer,
        exchange: RelatedExchange {
            property: "12 Elm St".into(),
            last_transfer: 20240501,
            disposed,
            exception,
            deferred: 90_000 * 100,
            kind: PropertyCharacter::BusinessProperty {
                used: PropertyUse::Rental,
                unrecaptured: 35_000 * 100,
            },
            special: false,
        },
    }
}

#[test]
fn exchange_earlier_year_related_follow_up_two_year_boundary() {
    // $70,000 W-2. Last transfer 2024-05-01; the period ends before
    // 2026-05-01. Disposed 2026-04-30: the deferred $90,000 is recognized as
    // section 1231 gain on Form 4797 line 5 with min(35,000, 90,000) =
    // $35,000 unrecaptured §1250 gain; AGI = 70,000 + 90,000 = $160,000.
    let d = related_return(70_000, vec![earlier_exchange(20260430, RelatedException::None)]);
    let r = supported(&d);
    assert_eq!(value(&r, "related_exchange_1_gain"), 90_000);
    assert_eq!(value(&r, "related_exchange_1_section1231"), 90_000);
    assert_eq!(value(&r, "related_exchange_1_unrecaptured_1250"), 35_000);
    assert_eq!(printed_line(&r, "related_exchange_1_section1231"), (TaxForm::Form4797, "5".into()));
    assert_eq!(printed_line(&r, "related_exchange_1_gain"), (TaxForm::Form8824, "24".into()));
    assert_eq!(value(&r, "form4797_line8"), 90_000);
    assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 35_000);
    assert_eq!(value(&r, "agi"), 160_000);
    // The anniversary itself is outside the period: nothing is recognized.
    let r = supported(&related_return(
        70_000,
        vec![earlier_exchange(20260501, RelatedException::None)],
    ));
    assert!(!has_row(&r, "related_exchange_1_gain"));
    assert_eq!(value(&r, "agi"), 70_000);
    // An exception: death, involuntary conversion or no tax avoidance.
    for exception in [
        RelatedException::Death,
        RelatedException::InvoluntaryConversion,
        RelatedException::NoTaxAvoidance,
    ] {
        let r = supported(&related_return(70_000, vec![earlier_exchange(20260430, exception)]));
        assert!(!has_row(&r, "related_exchange_1_gain"), "{exception:?}");
    }
    // A deferred loss (zero) recognizes nothing.
    let mut loss = earlier_exchange(20260430, RelatedException::None);
    loss.exchange.deferred = 0;
    let r = supported(&related_return(70_000, vec![loss]));
    assert!(!has_row(&r, "related_exchange_1_gain"));
    // Capital property: long-term to Schedule D line 11, short-term to 4.
    let mut capital = earlier_exchange(20260430, RelatedException::None);
    capital.exchange.kind = PropertyCharacter::LongTermCapital;
    let r = supported(&related_return(70_000, vec![capital.clone()]));
    assert_eq!(value(&r, "capital_gain_11"), 90_000);
    assert_eq!(value(&r, "agi"), 160_000);
    capital.exchange.kind = PropertyCharacter::ShortTermCapital;
    let r = supported(&related_return(70_000, vec![capital]));
    assert_eq!(value(&r, "capital_gain_4"), 90_000);
    // Something else applies: refused by name.
    let mut special = earlier_exchange(20260430, RelatedException::None);
    special.exchange.special = true;
    let r = estimate(&related_return(70_000, vec![special]));
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(r.issues.iter().any(|i| i.display.key == "ISSUE_RELATED_EXCHANGE_SOMETHING_ELSE"));
    // A rental gain with unallowed passive losses elsewhere needs Form 8582.
    let mut rental_gain = earlier_exchange(20260430, RelatedException::None);
    rental_gain.exchange.kind =
        PropertyCharacter::BusinessProperty { used: PropertyUse::Rental, unrecaptured: 0 };
    let mut d = related_return(70_000, vec![rental_gain]);
    let mut losing = rental(1_000, 40_000);
    losing.participation = crate::RentalParticipation::NotActive;
    add(&mut d, income(FederalIncome::Rental { data: losing }));
    let r = estimate(&d);
    assert!(
        r.issues.iter().any(|i| i.display.key
            == "ISSUE_RELATED_EXCHANGE_GAIN_FROM_A_RENTAL_WITH_UNALLOWED_PASSIVE_LOSSES"),
        "{:?}",
        r.issues
    );
}

#[test]
fn exchange_deadlines_forty_five_one_eighty_and_the_extension() {
    // Fixture B with transfer 2026-04-15: identified 2026-05-30 (day 45) and
    // received 2026-10-12 (day 180) qualify.
    let (d, id) = fixture_b(20260415, 20260530, 20261012, None);
    let r = supported(&d);
    assert!(!noted(&r, "WARN_EXCHANGE_TAXED_AS_SALE"));
    assert_eq!(row_value(&r, id, "line19"), 212_909);
    assert_eq!(row_value(&r, id, "line23"), 30_000);
    // Identified day 46 and received day 47: taxed as a sale. Recognized
    // $212,909 = building 182,909 (unrecaptured $82,909) + land 30,000
    // (70,000 - 40,000). The property received costs $300,000: its building
    // 225,000 from 2026-06-01 depreciates 225,000 x 13 / 660 = $4,431.82.
    let (d, id) = fixture_b(20260415, 20260531, 20260601, None);
    let r = supported(&d);
    assert!(noted(&r, "WARN_EXCHANGE_TAXED_AS_SALE"));
    assert_eq!(value(&r, &alloc::format!("rental_{id}_sale_building_gain")), 182_909);
    assert_eq!(value(&r, &alloc::format!("rental_{id}_sale_unrecaptured_1250")), 82_909);
    assert_eq!(value(&r, &alloc::format!("rental_{id}_sale_land")), 30_000);
    assert!(!has_row(&r, &alloc::format!("rental_{id}_exchange_line19")));
    // 1,696.97 + 4,431.82 = 6,128.79 -> $6,129.
    // 2026-10-19, D-2026-10-19-01: exact cents; whole-dollar form output.
    assert_eq!(line_amount(&r, &alloc::format!("rental_{id}_depreciation")), 612_879);
    assert_eq!(displayed_line_amount(&r, &alloc::format!("rental_{id}_depreciation")), 612_900);
    // Received day 181 (2026-10-13), identified in time: taxed as a sale.
    let (d, _) = fixture_b(20260415, 20260530, 20261013, None);
    assert!(noted(&supported(&d), "WARN_EXCHANGE_TAXED_AS_SALE"));
    // Transfer 2026-11-20, received 2027-04-20 (day 151): the extension
    // question is required, a Problem on the page and a refusal in the
    // estimate when missing.
    let (d, _) = fixture_b(20261120, 20261220, 20270420, None);
    let x = rental_exchange(&d);
    assert_eq!(x.rule(), Some("EXCHANGE_SAY_WHETHER_YOU_EXTEND"));
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported);
    let (d, _) = fixture_b(20261120, 20261220, 20270420, Some(false));
    assert_eq!(rental_exchange(&d).rule(), None);
    assert!(noted(&supported(&d), "WARN_EXCHANGE_TAXED_AS_SALE"));
    let (d, id) = fixture_b(20261120, 20261220, 20270420, Some(true));
    let r = supported(&d);
    assert!(!noted(&r, "WARN_EXCHANGE_TAXED_AS_SALE"));
    // Depreciation to November (21 half-months of 2026): 160,000 x 356 / 660 =
    // $86,303, B = 40,000 + 73,697 = 113,697, line 19 = 330,000 - 113,697.
    assert_eq!(row_value(&r, id, "line19"), 216_303);
    assert_eq!(row_value(&r, id, "line23"), 30_000);
    // On the last day before the due date no answer is needed.
    let (d, _) = fixture_b(20261120, 20261220, 20270415, None);
    assert_eq!(rental_exchange(&d).rule(), None);
    // A 2027 receipt leaves 2026 depreciation to the property given up.
    let (d, id) = fixture_b(20261120, 20261220, 20270420, Some(true));
    let r = supported(&d);
    assert!(value(&r, &alloc::format!("rental_{id}_depreciation")) > 0);
}

fn rental_exchange(d: &FederalDraft) -> RentalExchange {
    let FederalIncome::Rental { data } = &d.income[1].income else {
        panic!("a rental");
    };
    let crate::RentalSale::Exchanged(x) = &data.sale else {
        panic!("an exchange");
    };
    x.clone()
}
fn mutate(d: &mut FederalDraft, change: impl FnOnce(&mut FederalRental, &mut RentalExchange)) {
    let FederalIncome::Rental { data } = &mut d.income[1].income else {
        panic!("a rental");
    };
    let mut x = match core::mem::replace(&mut data.sale, crate::RentalSale::No) {
        crate::RentalSale::Exchanged(x) => x,
        _ => panic!("an exchange"),
    };
    change(data, &mut x);
    data.sale = crate::RentalSale::Exchanged(x);
}
fn refused_with(d: &FederalDraft, key: &str) {
    let r = estimate(d);
    assert_eq!(r.state, ResultState::Unsupported, "{key}");
    assert!(r.issues.iter().any(|i| i.display.key == key), "{key}: {:?}", r.issues);
}

#[test]
fn exchange_consistency_checks_and_refusals() {
    let (base, id) = fixture_a();
    // The land part above everything received: the page rule and the
    // estimate's refusal.
    let mut d = base.clone();
    mutate(&mut d, |_, x| x.land_value = 800_000 * 100);
    assert_eq!(rental_exchange(&d).rule(), Some("EXCHANGE_LAND_VALUE_ABOVE_TOTAL"));
    refused_with(&d, "ISSUE_EXCHANGE_DATES_OR_AMOUNTS_INVALID");
    // Land above the amount realized after expenses (still below the total).
    let mut d = base.clone();
    mutate(&mut d, |_, x| x.land_value = 600_000 * 100);
    refused_with(&d, "ISSUE_EXCHANGE_DATES_OR_AMOUNTS_INVALID");
    // The other page rules.
    let mut d = base.clone();
    mutate(&mut d, |_, x| x.identified = 20260911);
    assert_eq!(rental_exchange(&d).rule(), Some("EXCHANGE_IDENTIFIED_AFTER_RECEIVED"));
    let mut d = base.clone();
    mutate(&mut d, |_, x| x.replacement.land = 520_001 * 100);
    assert_eq!(rental_exchange(&d).rule(), Some("EXCHANGE_LAND_ABOVE_VALUE"));
    let mut d = base.clone();
    mutate(&mut d, |_, x| x.replacement.land = 520_000 * 100);
    assert_eq!(rental_exchange(&d).rule(), Some("EXCHANGE_RENTAL_WITHOUT_BUILDING"));
    let mut d = base.clone();
    mutate(&mut d, |_, x| {
        x.replacement.used = rental_use(RentalPropertyType::SingleFamily, 20260909)
    });
    assert_eq!(rental_exchange(&d).rule(), Some("EXCHANGE_RENTED_BEFORE_RECEIVED"));
    // Receipt before the transfer is a reverse exchange.
    let mut d = base.clone();
    mutate(&mut d, |_, x| {
        x.identified = 20260601;
        x.received = 20260610;
        x.replacement.used = rental_use(RentalPropertyType::SingleFamily, 20260615);
    });
    refused_with(&d, "ISSUE_EXCHANGE_RECEIVED_BEFORE_TRANSFER");
    // Something else applies.
    let mut d = base.clone();
    mutate(&mut d, |_, x| x.special = true);
    refused_with(&d, "ISSUE_EXCHANGE_SOMETHING_ELSE");
    // Use in the filer's own business is refused by name.
    let mut d = base.clone();
    mutate(&mut d, |_, x| x.replacement.used = ReplacementUse::Business);
    refused_with(&d, "ISSUE_EXCHANGE_REPLACEMENT_IN_BUSINESS");
    // A home or foreign property is taxed as a sale; land held for investment
    // is an exchange whose suspended losses stay.
    for (used, reason) in [
        (ReplacementUse::Personal, "EXCHANGE_REASON_PERSONAL"),
        (ReplacementUse::Foreign, "EXCHANGE_REASON_FOREIGN"),
    ] {
        let mut d = base.clone();
        mutate(&mut d, |_, x| x.replacement.used = used.clone());
        let r = supported(&d);
        let note = r
            .review_notes
            .iter()
            .find(|n| n.display.key == "WARN_EXCHANGE_TAXED_AS_SALE")
            .expect("the exchange is taxed as a sale");
        assert!(
            note.display.args.contains(&crate::MessageArg::Resource(reason.into())),
            "{reason}"
        );
        assert_eq!(value(&r, &alloc::format!("rental_{id}_depreciation")), 4_000);
    }
    let mut d = base.clone();
    mutate(&mut d, |rental, x| {
        x.replacement.used = ReplacementUse::Investment;
        rental.history =
            RentalHistory { regular: 1_000 * 100, amt: 1_000 * 100, ..RentalHistory::NONE };
    });
    let r = supported(&d);
    assert!(noted(&r, "NOTE_EXCHANGE_LOSSES_STAY_SUSPENDED"));
    assert_eq!(value(&r, &alloc::format!("rental_{id}_depreciation")), 4_000);
    // Vacation property with ordinary stays follows the same residential rules.
    let mut d = base.clone();
    mutate(&mut d, |_, x| {
        x.replacement.used = rental_use(
            RentalPropertyType::VacationShortTerm { stays: ShortTermStays::OverSevenDays },
            20260915,
        )
    });
    supported(&d);
    // A rental with personal use and a building held one year or less keep
    // the rental sale refusals.
    let mut d = base.clone();
    mutate(&mut d, |rental, _| {
        rental.personal_use = crate::extended_model::RentalPersonalUse::Yes {
            fair_rental_days: 200,
            personal_days: 30,
            overlap_days: 0,
            direct_expenses: 0,
        };
    });
    refused_with(&d, "ISSUE_RENTAL_EXCHANGE_WITH_PERSONAL_USE");
    let mut d = base.clone();
    mutate(&mut d, |rental, _| {
        rental.buildings.push(purchased_rental_building(1_000_000, 0, 20250901))
    });
    // ... and a building held one year or less is calculated (Form 8824 defers
    // its gain like the others').
    supported(&d);
}

#[test]
fn exchange_multiple_buildings_keep_each_remaining_recovery_period() {
    // Fixture A's rental with a $60,000 improvement available 2022-07-01. The
    // exchanged basis divides by each building's adjusted basis after the
    // 2026 depreciation to the transfer month, and each keeps its own
    // remaining recovery period. Half-months through June 2026: the house 246
    // (19 + 9 x 24 + 11), the improvement 94 (11 + 3 x 24 + 11).
    let (mut d, id) = fixture_a();
    mutate(&mut d, |rental, _| {
        rental.buildings.push(purchased_rental_building(60_000 * 100, 0, 20220701))
    });
    let r = supported(&d);
    let allowed = |b: &RentalBuilding| {
        crate::asset_sale::accumulated(b, false, &RentalPropertyType::SingleFamily, 20260620)
            .unwrap()
    };
    let FederalIncome::Rental { data } = &d.income[1].income else { unreachable!() };
    let adjusted: Vec<i64> = data.buildings.iter().map(|b| b.cost - b.land - allowed(b)).collect();
    let depreciation: i64 = data.buildings.iter().map(allowed).sum();
    // B = land 60,000 + buildings 300,000 - depreciation; 18 = B + 50,000 +
    // 28,000; 19 = 520,000 - 18; 25 = 18 (no boot); 25a = 25 x 400/520.
    let basis = 60_000 + 300_000 - crate::kernel::ratio::dollars(depreciation);
    let line18 = basis + 50_000 + 28_000;
    assert_eq!(row_value(&r, id, "line18"), line18);
    let line25a = line18 * 400 / 520;
    let g = crate::kernel::ratio::dollars(adjusted[0] + adjusted[1]);
    let exchanged = line25a.min(g);
    let carry = |index: usize, elapsed: i64| {
        let part = exchanged * 100 * adjusted[index] / (adjusted[0] + adjusted[1]);
        part * 7 / (660 - elapsed)
    };
    let expected = carry(0, 246) + carry(1, 94);
    let got = value(&r, &alloc::format!("rental_{id}_replacement_carryover_depreciation"));
    assert!((got - expected / 100).abs() <= 1, "{got} {expected}");
}

#[test]
fn exchange_pre_1999_building_refigures_the_alternative_minimum_tax_basis() {
    // A building first available 1995-05-10 (27.5-year regular, 40-year ADS
    // for the alternative minimum tax) exchanged without boot for a $400,000
    // single-family rental ($100,000 land) first rented 2026-09-15. The
    // regular and AMT depreciation of the property given up differ, so the
    // exchanged basis and the carried-over depreciation differ too.
    let mut x = exchange(
        20260620,
        20260725,
        20260910,
        400_000,
        100_000,
        rental_use(RentalPropertyType::SingleFamily, 20260915),
        80_000,
    );
    x.expenses = 10_000 * 100;
    let (d, id) = exchange_return(
        90_000,
        exchanged(
            given_up(RentalPropertyType::SingleFamily, 300_000, 60_000, 19950510, 30_000, 12_000),
            x.clone(),
        ),
    );
    let r = supported(&d);
    let FederalIncome::Rental { data } = &d.income[1].income else { unreachable!() };
    let regular = x.form(data, false).unwrap();
    let alternative = x.form(data, true).unwrap();
    // Regular: 27.5 years from May 1995 end in 2022, so the building is fully
    // depreciated; AMT: 40-year ADS (960 half-months) runs to 2035.
    assert_eq!(regular.depreciation, 240_000);
    assert!(alternative.depreciation < regular.depreciation);
    assert_eq!(regular.exchanged_basis, 0);
    assert!(alternative.exchanged_basis > 0);
    // A fully depreciated building leaves nothing to carry over: only the
    // excess basis is depreciated on the regular return.
    let carryover = x.replacement_depreciation(data, false).unwrap();
    assert_eq!(carryover.carryover, 0);
    assert!(carryover.excess > 0);
    let amt = x.replacement_depreciation(data, true).unwrap();
    assert!(amt.carryover > 0);
    assert_eq!(
        // 2026-10-19, D-2026-10-19-01: preserve the exact source sum.
        line_amount(&r, &alloc::format!("rental_{id}_depreciation")),
        crate::extended_rules::depreciation_for_year(
            &data.buildings[0],
            false,
            (&(data).property_type, &crate::RentalLocation::UnitedStates),
            2026,
            data.sale.date(),
            crate::kernel::source_fact::SourceFact::NotReported
        )
        .unwrap()
            + carryover.total()
    );
    assert_eq!(displayed_line_amount(&r, &alloc::format!("rental_{id}_depreciation")), 55_700);
}

#[test]
fn exchange_recognized_gain_is_passive_income_and_releases_no_suspended_loss() {
    // Fixture B with $100,000 of prior suspended passive losses on the rental.
    // Own net 24,000 - 6,000 - 5,060 = $12,940, less the $100,000 loss, plus
    // the $30,000 recognized gain = -$57,060. An exchange is not a fully
    // taxable disposition (section 469(g)(1)(A)), so the loss stays passive:
    // the $25,000 special allowance (MAGI $50,000, below $100,000) allows
    // $25,000 and $32,060 stays suspended. Schedule E carries -$55,000 and
    // Form 4797 +$30,000, so AGI = 50,000 - 25,000 = $25,000.
    let (mut d, id) = fixture_b(20260415, 20260520, 20260520, None);
    mutate(&mut d, |rental, _| {
        rental.history =
            RentalHistory { regular: 100_000 * 100, amt: 100_000 * 100, ..RentalHistory::NONE }
    });
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("rental_{id}_suspended_loss")), 32_060);
    assert_eq!(value(&r, "form4797_line8"), 30_000);
    assert_eq!(value(&r, "agi"), 25_000);
    // The same facts as a sale of the rental release the loss in full.
    let (mut sold, id) = fixture_b(20260415, 20260520, 20260520, None);
    mutate(&mut sold, |rental, x| {
        rental.history =
            RentalHistory { regular: 100_000 * 100, amt: 100_000 * 100, ..RentalHistory::NONE };
        // Taxed as a sale: the property received is a home.
        x.replacement.used = ReplacementUse::Personal;
    });
    let r = supported(&sold);
    assert_eq!(value(&r, &alloc::format!("rental_{id}_suspended_loss")), 0);
}

#[test]
fn exchange_qbi_unadjusted_basis_carries_over_with_the_excess() {
    // Fixture A: the exchanged basis equals the building's adjusted basis, so
    // the whole $240,000 unadjusted basis carries over and the $71,413 excess
    // basis is new.
    let (d, _) = fixture_a();
    let FederalIncome::Rental { data } = &d.income[1].income else { unreachable!() };
    let x = rental_exchange(&d);
    assert_eq!(x.ubia(data).unwrap(), 24_000_000 + 7_141_300);
    // A failed exchange has the property received at cost: $400,000 building.
    let mut failed = x.clone();
    failed.replacement.used = rental_use(RentalPropertyType::SingleFamily, 20260915);
    failed.party = ExchangeParty::RelatedSeller { exception: RelatedException::None };
    assert_eq!(failed.ubia(data).unwrap(), 40_000_000);
    // Land held for investment has none.
    failed.replacement.used = ReplacementUse::Investment;
    assert_eq!(failed.ubia(data).unwrap(), 0);
}

// A building first available to rent in 2027 (a replacement property received
// in 2027) is a valid record with no 2026 depreciation; the service date still
// has to be a real calendar date, and no later than 2027. 2028 and February 30
// are rejected as building records.
#[test]
fn depreciation_accepts_a_building_first_available_in_2027() {
    let building = |in_service| purchased_rental_building(20_000_000, 4_000_000, in_service);
    let kind = RentalPropertyType::SingleFamily;
    let year = |b: &RentalBuilding, y| {
        crate::extended_rules::depreciation_for_year(
            b,
            false,
            (&kind, &crate::RentalLocation::UnitedStates),
            y,
            None,
            crate::kernel::source_fact::SourceFact::NotReported,
        )
    };
    // Not yet in service in 2026: zero.
    assert_eq!(year(&building(20270301), 2026), Ok(0));
    // 2027: basis 160,000.00; March (month 3) first-year half-months 25 - 6 = 19 of 660:
    // 160,000.00 x 19 / 660 = 4,606.06.
    assert_eq!(year(&building(20270301), 2027), Ok(460_606));
    assert!(year(&building(20271231), 2026).is_ok());
    assert!(year(&building(20280101), 2026).is_err());
}
