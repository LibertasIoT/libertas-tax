// Line audit fix F13 (2026-10-10), Forms 4797 and 4684: the source rows of the
// Form 4797 Part I and Part II lines (4797-D4) and the two Form 4684 defects
// (4684-D1, D2). The 4797 asset fixtures are in business.rs. Expected amounts
// are the audit's hand-worked figures (qbi-schedule-f-4797-4684.md section 3)
// from the 2026 draft forms, the 2025 final instructions and 26 U.S.C. 63(b)(8)
// and 165(h) as amended by P.L. 119-108 section 2 (Sept. 11, 2026).
use super::*;

fn line_4797(r: &FederalResult, line: &str) -> i64 {
    let wanted = FormLine::new(TaxForm::Form4797, line);
    r.lines.iter().filter(|l| l.form_line.as_ref() == Some(&wanted)).map(|l| l.amount).sum::<i64>()
        / 100
}

// The farm unit stores a parcel sale against its accepted parcel basis; the
// synthetic sale facts are set through the shared fixture helper.
fn farm_return(sales: Vec<crate::farm_land::ResolvedFarmLandSale>) -> FederalDraft {
    let mut d = conservation_return();
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.conservation = None;
    set_land_sales(data, sales, &mut d.next_id);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    d
}

// 4797-D4, farm land (probe E): bought June 15, 2021, sold June 16, 2026 for
// $150,000 with $10,000 selling expenses and $100,000 basis, $20,000 of prior
// section 175 deductions. Gain $40,000, 80% x $20,000 = $16,000 section 1252
// recapture, $24,000 section 1231 gain on line 2. A parcel sold within a year
// is ordinary on line 11: bought June 15, 2025, sold June 15, 2026, $40,000.
#[test]
fn f13_d4_farm_land_has_its_part_i_and_part_ii_rows() {
    let d =
        farm_return(vec![farm_land_sale(20210615, 20260616), farm_land_sale(20250615, 20260615)]);
    let r = supported(&d);
    let id = d.income[1].id;
    assert_eq!(value(&r, &alloc::format!("farm_{id}_land_0_section1231")), 24_000);
    assert_eq!(
        printed_line(&r, &alloc::format!("farm_{id}_land_0_section1231")),
        (TaxForm::Form4797, "2".into())
    );
    assert_eq!(value(&r, &alloc::format!("farm_{id}_land_1_ordinary")), 40_000);
    assert_eq!(
        printed_line(&r, &alloc::format!("farm_{id}_land_1_ordinary")),
        (TaxForm::Form4797, "11".into())
    );
    // The short parcel has nothing on line 2 and the long parcel's recapture
    // is not a line 11 amount.
    assert_eq!(line_4797(&r, "2"), 24_000);
    assert_eq!(line_4797(&r, "11"), 40_000);
    assert_eq!(value(&r, "form4797_line8"), line_4797(&r, "2"));
}

// 4797-D4, farm-land installment: the same parcel sold with $120,000 received
// in 2026 of the $150,000 price. Gross profit $40,000 x 120/150 = $32,000
// installment income, the first $16,000 is section 1252 recapture and the
// other $16,000 is the section 1231 gain, Form 6252 line 26 on line 4.
#[test]
fn f13_d4_farm_land_installment_gain_is_on_line_4() {
    let mut sale = farm_land_sale(20210615, 20260616);
    sale.payment = crate::SalePayment::Installment {
        assumed_debt: 0,
        payments: 12_000_000,
        interest: crate::NoteInterest::Adequate,
        special: false,
    };
    let d = farm_return(vec![sale]);
    let r = supported(&d);
    let id = d.income[1].id;
    assert_eq!(value(&r, &alloc::format!("farm_{id}_land_0_form6252_gain")), 16_000);
    assert_eq!(
        printed_line(&r, &alloc::format!("farm_{id}_land_0_section1231")),
        (TaxForm::Form4797, "4".into())
    );
    assert_eq!(line_4797(&r, "4"), 16_000);
    assert_eq!(line_4797(&r, "2"), 0);
    assert_eq!(value(&r, "form4797_line8"), 16_000);
}

// 4797-D4, earlier-year installment sale of a rental building: gross profit
// $100,000 of a $250,000 contract price, $30,000 received in 2026: $12,000,
// Form 6252 line 26 on Form 4797 line 4.
#[test]
fn f13_d4_earlier_installment_gain_is_on_line_4() {
    let mut d = complete(FederalStatus::Single);
    let screening = d.screening.as_mut().unwrap();
    screening.installment_payments = vec![crate::EarlierInstallment {
        owner: Owner::Taxpayer,
        sale: crate::InstallmentPayments {
            property: "Rental building".into(),
            sold: 20230601,
            gross_profit: 10_000_000,
            contract_price: 25_000_000,
            earlier_payments: 5_000_000,
            payments: 3_000_000,
            kind: crate::PropertyCharacter::BusinessProperty {
                used: crate::PropertyUse::Rental,
                unrecaptured: 800_000,
            },
            special: false,
            qbi: crate::InstallmentQbi::NotQualified,
        },
    }];
    screening.section1231 = Some(section1231_history(&[], &[]));
    let r = supported(&d);
    assert_eq!(value(&r, "installment_1_section1231"), 12_000);
    assert_eq!(printed_line(&r, "installment_1_section1231"), (TaxForm::Form4797, "4".into()));
    assert_eq!(line_4797(&r, "4"), value(&r, "form4797_line8"));
}

// 4684-D1: a 2026 presidential major-disaster loss (FEMA DR-1234, incident
// period in 2026) is a qualified disaster loss under section 165(h)(6) as
// amended by P.L. 119-108 section 2. Single, AGI $60,000, household property
// basis $30,000 and value before $30,000, nothing left, no reimbursement:
// line 10 $30,000; line 11 $500 (line 10 exceeds the line 4 gains of $0);
// line 12 $29,500; line 14 $29,500; line 15 min($29,500, $29,500) = $29,500 on
// Schedule A line 17b; lines 16-18 $0 (no 10% of AGI reduction). Section
// 63(b)(8) adds the line 15 amount to the standard deduction: Form 1040 line
// 12e $16,100 + $29,500 = $45,600. As an ordinary federal event ($100 and the
// 10% reduction) the same loss is $23,900.
#[test]
fn f13_4684_d1_a_current_year_major_disaster_is_a_qualified_disaster_loss() {
    let mut d = complete(FederalStatus::Single);
    d.deductions.as_mut().unwrap().personal_casualties.push(major_disaster("Hurricane", 3_000_000));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 60_000);
    // Lines 11 and 12 through 14.
    assert_eq!(value(&r, "form_4684_personal_loss_after_events"), 29_500);
    assert_eq!(value(&r, "form_4684_personal_net_after_gains"), 29_500);
    // Line 15 (Schedule A line 17b); lines 16-18 are zero.
    assert_eq!(value(&r, "qualified_net_disaster_loss"), 29_500);
    assert_eq!(value(&r, "form_4684_personal_loss_after_agi"), 0);
    // The 2026 draft Schedule A: line 16 holds only the ordinary casualty loss,
    // line 17b the qualified disaster loss, line 17c the standard deduction
    // beside it (the standard route) and line 18 their sum.
    assert_eq!(value(&r, "personal_casualty_deduction"), 0);
    assert_eq!(printed_line(&r, "qualified_net_disaster_loss"), (TaxForm::ScheduleA, "17b".into()));
    assert_eq!(value(&r, "standard_with_disaster_loss"), 16_100);
    assert_eq!(value(&r, "itemized_deductions_allowed"), 45_600);
    // Standard route: the standard deduction plus line 15.
    assert_eq!(return_line(&r, "12e"), 4_560_000);
    // The same event as an ordinary federally declared emergency: $100 floor,
    // 10% of AGI reduction, itemized only, so the standard deduction is
    // larger.
    d.deductions.as_mut().unwrap().personal_casualties[0].declaration =
        CasualtyDeclaration::Federal { reference: "EM-1234".into() };
    let r = supported(&d);
    assert_eq!(value(&r, "form_4684_personal_loss_after_events"), 29_900);
    assert_eq!(value(&r, "form_4684_personal_loss_after_agi"), 23_900);
    assert_eq!(value(&r, "qualified_net_disaster_loss"), 0);
    assert_eq!(return_line(&r, "12e"), 2_390_000);
}

// 4684-D1, line 11: a qualified event takes $500 only when its line 10 losses
// exceed the line 4 gains of all the events (i4684 line 11); otherwise $100.
// A $1,000 major-disaster loss beside a $4,900 gain from another event takes
// $100: line 12 is $900, not $500.
#[test]
fn f13_4684_d1_the_500_floor_needs_losses_above_the_gains() {
    let mut d = complete(FederalStatus::Single);
    let mut event = major_disaster("Storm", 100_000);
    let mut gain = major_disaster("Theft", 10_000);
    gain.declaration = CasualtyDeclaration::Undeclared;
    gain.zip.clear();
    gain.properties[0].before = 10_000;
    gain.properties[0].reimbursement = 500_000;
    gain.properties[0].received = 500_000;
    event.properties[0].before = 100_000;
    d.deductions.as_mut().unwrap().personal_casualties = vec![event, gain];
    let r = supported(&d);
    assert_eq!(value(&r, "form_4684_personal_gains"), 4_900);
    assert_eq!(value(&r, "form_4684_personal_loss_after_events"), 900);
}

// 4684-D2: a car (basis $10,000, value $12,000 before, nothing after) stolen
// in 2026; the insurer agreed in December to pay $14,000 and paid in January
// 2027. Line 3 is $14,000 (expected), so line 9 is $0; line 4 counts only
// what was received in 2026 ($0), so there is no gain, Schedule D line 11 is
// $0 and AGI stays $60,000. Received in 2026, the $4,000 gain is on line 4.
#[test]
fn f13_4684_d2_line_4_counts_only_reimbursement_received() {
    let mut d = complete(FederalStatus::Single);
    let theft = PersonalCasualty {
        id: 0,
        label: "Stolen car".into(),
        date: 20261215,
        declaration: CasualtyDeclaration::Undeclared,
        zip: "".into(),
        properties: vec![CasualtyProperty {
            id: 0,
            home_site: None,
            business_damage: None,
            label: "Car".into(),
            location: "Synthetic City, CA".into(),
            acquisition: CasualtyAcquisition::Purchased { date: 20200101 },
            basis: 1_000_000,
            before: 1_200_000,
            after: 0,
            reimbursement: 1_400_000,
            received: 0,
        }],
        source_complete: true,
    };
    d.deductions.as_mut().unwrap().personal_casualties = vec![theft];
    let r = supported(&d);
    assert_eq!(value(&r, "form_4684_personal_gains"), 0);
    assert_eq!(value(&r, "capital_gain_11"), 0);
    assert_eq!(value(&r, "agi"), 60_000);
    d.deductions.as_mut().unwrap().personal_casualties[0].properties[0].received = 1_400_000;
    let r = supported(&d);
    assert_eq!(value(&r, "form_4684_personal_gains"), 4_000);
    assert_eq!(value(&r, "capital_gain_11"), 4_000);
    assert_eq!(value(&r, "agi"), 64_000);
}
