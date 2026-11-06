// Fix unit F9 (2026-10-10): the line audit of Forms 6251, 8801, 4952, 8960, 8959 and 1116.
use super::*;
use crate::tests::cross::interview::{
    accepted, rejected_with, save_whole_return, save_your_situations,
};

// A printed form line's amount in whole dollars.
fn printed(r: &FederalResult, form: TaxForm, line: &str) -> i64 {
    r.lines
        .iter()
        .find(|l| l.form_line.as_ref().is_some_and(|f| f.form == form && f.line == line))
        .unwrap_or_else(|| panic!("no {form:?} line {line}"))
        .amount
        / 100
}

fn employment(income: i64, tax: i64) -> ForeignTaxClaim {
    ForeignTaxClaim::Ordinary {
        history: ForeignCreditHistory::None,
        carryback: ForeignCarryback::Unreviewed,
        interest_allocation: ForeignInterestAllocation::GrossIncome,
        investment_assets: ForeignInvestmentAssets::Unreviewed,
        loss_history: ForeignLossHistory::None,
        state_taxes: ForeignStateTaxes::Unreviewed,
        amt_limitation: ForeignAmtLimitation::Normal,
        sources: vec![ForeignOrdinaryTax {
            country: "ZZ".into(),
            owner: Owner::Taxpayer,
            label: "Country A employer".into(),
            category: ForeignTaxCategory::Employment,
            income: income * 100,
            tax: tax * 100,
        }],
        basis: ForeignTaxBasis::Paid,
        situations: vec![],
        rate_adjustment: ForeignRateAdjustment::Exception,
    }
}

fn history(
    status: FederalStatus,
    base: i64,
    regular: i64,
    amt: i64,
    carry: i64,
) -> MinimumTaxHistory {
    MinimumTaxHistory {
        return_form: crate::MinimumTaxReturnForm::Form1040,
        gains: None,
        status,
        base: base * 100,
        exclusions: vec![],
        regular: regular * 100,
        amt: amt * 100,
        carry: carry * 100,
        regular_nol: 0,
        mtcnol: crate::MinimumTaxLoss::None,
        foreign_earned: None,
        foreign_credit: crate::MinimumTaxForeignCredit::None,
        old_vehicle_credit: 0,
    }
}

fn property_tax_refund_history(property_refunded: i64) -> TaxRefundHistory {
    TaxRefundHistory::IncomeTax {
        deduction_year: 2025,
        income_paid: 600_000,
        sales_available: 0,
        salt_paid: 1_400_000,
        salt_deducted: 1_000_000,
        itemized: 2_000_000,
        standard: 1_575_000,
        overall_limit: None,
        property_refunded,
        ordinary: true,
    }
}

#[test]
fn amt_d1_property_tax_refund_on_line_8z_is_subtracted_on_line_2b() {
    // Form 6251 line 2b (section 56(b)(1)(C)): the refund of real estate tax that Worksheet 2a
    // sends to Schedule 1 line 8z comes out of alternative minimum taxable income beside the
    // line 1 share. Single, W-2 $60,000; 1099-G $1,500 of 2025 income tax and $5,000 of real
    // estate tax refunded: line 1 $577 and line 8z $1,923; AMTI $60,000.
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::GovernmentPayment {
            data: crate::GovernmentPaymentSource {
                form: refund_paper(Refund1099G {
                    payer: Default::default(),
                    recipient: Default::default(),
                    state_details: Default::default(),
                    amount: 150_000,
                    year: 2025,
                    withholding: 0,
                }),
                history: property_tax_refund_history(500_000),
                ..GovernmentPaymentSource::printed(Default::default())
            },
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 62_500);
    assert_eq!(value(&r, "taxable_state_tax_refund"), 577);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 60_000);
    // With a $200,000 ISO spread: line 4 $260,000, line 6 $169,900, line 10 $5,323, line 11 $38,851.
    d.screening.as_mut().unwrap().iso_exercises =
        vec![iso_exercise("Synthetic ISO", 1.0, 0, 200_000 * 100)];
    let r = supported(&d);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 260_000);
    assert_eq!(value(&r, "alternative_minimum_tax"), 38_851);
}

#[test]
fn mtc_d1_form_8814_tax_is_in_the_minimum_tax_credit_limit() {
    // Form 8801 line 22 starts from Form 1040 line 16, which carries the Form 8814 tax. MFJ,
    // W-2 $90,000, one child with Form 8814 dividends of $2,190 (all qualified) and $610: line 16
    // $6,578, child tax credit $2,200, line 22 $4,378, line 25 $4,378, carryforward $5,622, total
    // tax $0.
    let mut d = complete(FederalStatus::Joint);
    d.income = vec![wage(1, Owner::Taxpayer, 90_000 * 100, 0)];
    let id = d.next_id;
    add_child(&mut d);
    d.dependents[0].birth_date = 20200615;
    add(
        &mut d,
        child_election(
            id,
            vec![child_dividends(2_190, 2_190, 610)],
            0,
            OtherParentChoice::NotNeeded,
        ),
    );
    d.credits.as_mut().unwrap().minimum_tax = Some(crate::MinimumTaxHistories::from_current_unit(
        history(FederalStatus::Joint, 200_000, 30_000, 10_000, 0),
    ));
    let r = supported(&d);
    assert_eq!(return_line(&r, "16"), 6_578 * 100);
    assert_eq!(value(&r, "minimum_tax_credit_limit"), 4_378);
    assert_eq!(value(&r, "minimum_tax_credit"), 4_378);
    assert_eq!(value(&r, "minimum_tax_credit_carryforward"), 5_622);
    assert_eq!(return_line(&r, "22"), 0);
    assert_eq!(r.tax, Some(0));
}

#[test]
fn ftc_d1_form_8814_tax_is_in_the_foreign_tax_credit_limit() {
    // Form 1116 line 20 is Form 1040 line 16 (with Schedule 2 line 1z): the Form 8814 tax is in
    // it. MFJ, W-2 $40,000, foreign pay $60,000 taxed $30,000, a child with $2,700 of interest
    // (tax $135): line 16 $7,778, line 21 = 7,778 x 0.6 = $4,667, line 22 $911 after the $2,200
    // child tax credit.
    let mut d = complete(FederalStatus::Joint);
    d.income = vec![wage(1, Owner::Taxpayer, 40_000 * 100, 0)];
    let id = d.next_id;
    add_child(&mut d);
    d.dependents[0].birth_date = 20160615;
    add(&mut d, income(foreign_pay(60_000)));
    d.credits.as_mut().unwrap().foreign_tax = employment(60_000, 30_000);
    add(&mut d, child_election(id, vec![child_interest(2_700)], 0, OtherParentChoice::NotNeeded));
    let r = supported(&d);
    assert_eq!(return_line(&r, "16"), 7_778 * 100);
    assert_eq!(value(&r, "foreign_tax_credit"), 4_667);
    assert_eq!(return_line(&r, "22"), 911 * 100);
}

#[test]
fn ii_d1_child_qualified_dividends_can_be_elected_on_line_4g() {
    // Pub. 550: the parents may choose all or part of a child's Form 8814 qualified dividends as
    // investment income. MFJ, W-2 $90,000, itemizing, a child with $5,000 of qualified dividends
    // (line 6 $2,300 on Form 1040 lines 3a and 3b, tax $135); investment interest $2,000 and the
    // election of $2,000: line 4g $2,000, line 8 $2,000, line 7 $0.
    let mut d = complete(FederalStatus::Joint);
    d.income = vec![wage(1, Owner::Taxpayer, 90_000 * 100, 0)];
    let id = d.next_id;
    add_child(&mut d);
    d.dependents[0].birth_date = 20200615;
    add(
        &mut d,
        child_election(id, vec![child_dividends(5_000, 5_000, 0)], 0, OtherParentChoice::NotNeeded),
    );
    investment_borrowing(&mut d, 2_000, 0, 0);
    let without = supported(&d);
    assert_eq!(value(&without, "investment_interest_deduction"), 0);
    d.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().election =
        InvestmentElection::CapitalFirst { amount: 2_000 * 100 };
    let r = supported(&d);
    assert_eq!(value(&r, "investment_interest_deduction"), 2_000);
    assert_eq!(value(&r, "investment_interest_carryforward"), 0);
}

#[test]
fn ii_d2_publicly_traded_partnership_net_passive_income_is_investment_income() {
    // 2026 draft i4952 line 4a: net passive income from a passive activity of a publicly traded
    // partnership is investment income. Single, W-2 $60,000, a PTP's K-1 box 1 of $5,000 and
    // $5,000 of investment interest: line 4a $5,000, line 8 $5,000, line 7 $0.
    let mut d = complete(FederalStatus::Single);
    let mut data = ordinary_partner_source(5_000, false);
    data.publicly_traded = true;
    add(&mut d, income(FederalIncome::Partnership { data }));
    investment_borrowing(&mut d, 5_000, 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "investment_interest_ordinary_income"), 5_000);
    assert_eq!(value(&r, "investment_interest_deduction"), 5_000);
    assert_eq!(value(&r, "investment_interest_carryforward"), 0);
}

fn farm_rental_data(rent: i64) -> FederalFarmRental {
    FederalFarmRental {
        improvement_buildings: Vec::new(),
        interest_limit_election: None,
        farm: FarmIncome {
            related: vec![],
            specials: farm_specials(),
            land_basis: vec![],
            header: None,
            accounting: FarmAccounting::Cash {
                receipts: vec![FarmReceipt::Raised { amount: rent * 100 }],
            },
            insurance_papers: Default::default(),
            insurance: FarmInsuranceRecords::None,
            ccc: None,
            crp: vec![],
            supplies: None,
            conservation: None,
            land_sales: vec![],
        },
        expenses: vec![],
        vehicles: vec![],
        assets: vec![],
        active_participation: true,
        at_risk: Some(crate::RentalAtRisk::All),
        history: RentalHistory::NONE,
        qbi_business: RentalBusiness::No,
        qbi_aggregation_group: None,
        payroll: BusinessPayroll::No,
    }
}

fn press_sold(proceeds: i64) -> BusinessAsset {
    BusinessAsset {
        cost: 1_000_000,
        acquired: 20240301,
        placed: 20240301,
        history: EquipmentHistory::Continuing {
            initial_section179: 0,
            carryforward: 0,
            prior_depreciation: 520_000,
            amt: EquipmentAmtHistory::Same,
            mid_quarter: false,
            schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
        },
        disposal: crate::AssetDisposal::Disposed {
            date: 20260701,
            how: crate::DisposalKind::Sold {
                livestock: crate::SoldLivestock::No,
                proceeds: proceeds * 100,
                selling_expenses: 0,
                payment: crate::SalePayment::InFull,
                buyer: crate::Buyer::Unrelated,
            },
        },
        ..bonus_equipment(0, 100)
    }
}

#[test]
fn niit_d1_farm_rental_form_4797_recapture_is_net_investment_income() {
    // Form 8960 line 5b removes only gain on property of a non-section-1411 trade or business.
    // A Form 4835 farm rental is a passive rental activity. Single, W-2 $300,000, crop-share rent
    // $20,000, a five-year press (cost $10,000, prior depreciation $5,200) sold for $9,000: Form
    // 4835 line 32 $19,040, section 1245 gain $5,160. Form 8960: 4a 19,040; 5a 5,160; 5b 0;
    // line 8 24,200; line 17 $920.
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, wage(0, Owner::Taxpayer, 30_000_000, 0));
    let mut data = farm_rental_data(20_000);
    data.assets = vec![press_sold(9_000)];
    add(&mut d, income(FederalIncome::FarmRental { data }));
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let r = supported(&d);
    assert_eq!(printed(&r, TaxForm::Form8960, "5a"), 5_160);
    assert_eq!(printed(&r, TaxForm::Form8960, "5b"), 0);
    assert_eq!(printed(&r, TaxForm::Form8960, "8"), 24_200);
    assert_eq!(printed(&r, TaxForm::Form8960, "17"), 920);
    assert_eq!(printed(&r, TaxForm::Schedule2, "6"), 920);
}

#[test]
fn niit_d3_form_8960_is_printed_whenever_modified_agi_exceeds_the_threshold() {
    // Single, W-2 $300,000 and a materially participating Schedule C ($20,000 receipts) that
    // sold the press for $9,000: Schedule C $19,040, half of the self-employment tax $255, AGI
    // $323,945. Nothing is investment income, yet modified AGI is above $200,000, so Form 8960
    // is attached: line 13 $323,945, line 15 $123,945, line 17 $0.
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, wage(0, Owner::Taxpayer, 30_000_000, 0));
    let mut b = cash_business(20_000);
    if let FederalIncome::Business { data } = &mut b {
        data.assets = vec![press_sold(9_000)];
    }
    add(&mut d, income(b));
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let r = supported(&d);
    assert_eq!(printed(&r, TaxForm::Form8960, "8"), 0);
    assert_eq!(printed(&r, TaxForm::Form8960, "13"), 323_945);
    assert_eq!(printed(&r, TaxForm::Form8960, "15"), 123_945);
    assert_eq!(printed(&r, TaxForm::Form8960, "17"), 0);
    // Below the threshold no form is printed.
    let below = supported(&complete(FederalStatus::Single));
    assert!(
        !below
            .lines
            .iter()
            .any(|l| { l.form_line.as_ref().is_some_and(|f| f.form == TaxForm::Form8960) })
    );
}

#[test]
fn niit_d2_personal_casualty_loss_reduces_form_8960_line_7() {
    // i8960 line 7: casualty and theft losses reported on Schedule A are entered as a negative
    // amount. Single, W-2 $300,000, bank interest $60,000, one declared-disaster loss of
    // personal property with $80,000 of basis: Form 4684 $79,900 less 10% of AGI $360,000 leaves
    // $43,900 on Schedule A. Form 8960: line 1 $60,000, line 7 ($43,900), line 8 $16,100,
    // line 17 = $16,100 x 3.8% = $612.
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, wage(0, Owner::Taxpayer, 30_000_000, 0));
    add(&mut d, income(savings_income(60_000, 0)));
    d.deductions
        .as_mut()
        .unwrap()
        .personal_casualties
        .push(personal_disaster("Storm", 80_000 * 100));
    let r = supported(&d);
    assert_eq!(value(&r, "personal_casualty_deduction"), 43_900);
    assert_eq!(printed(&r, TaxForm::Form8960, "1"), 60_000);
    assert_eq!(printed(&r, TaxForm::Form8960, "7"), -43_900);
    assert_eq!(printed(&r, TaxForm::Form8960, "8"), 16_100);
    assert_eq!(printed(&r, TaxForm::Form8960, "17"), 612);
}

#[test]
fn ftc_d2_mortgage_interest_worksheet_leaves_out_form_2555_income() {
    // Form 1116 line 4a: the Worksheet for Home Mortgage Interest enters gross income without
    // the income excluded on Form 2555. Single, foreign pay $200,000 (bona fide residence, the
    // $132,900 exclusion), foreign tax $60,000, U.S. W-2 $100,000, general sales tax $10,000 and
    // mortgage interest $20,000: AGI $167,100, tax $38,775; 4a = 67,100 / 167,100 x 20,000 =
    // $8,032; line 7 $52,401; credit $14,820; total tax $23,955.
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_pay(200_000)));
    add(&mut d, wage(9, Owner::Taxpayer, 100_000 * 100, 0));
    let mut claim = foreign_claim(Owner::Taxpayer);
    claim.qualification = ForeignQualification::Residence {
        began: 20220110,
        ended: None,
        basis: crate::ResidenceBasis::Citizen,
        nonresident_statement: false,
    };
    set_claim(&mut d, claim);
    d.credits.as_mut().unwrap().foreign_tax = employment(200_000, 60_000);
    foreign_medical_itemizing(&mut d, 0, 10_000, 0);
    foreign_mortgage(&mut d, 20_000);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 167_100);
    assert_eq!(value(&r, "foreign_tax_credit"), 14_820);
    assert_eq!(r.tax, Some(23_955 * 100));
}

#[test]
fn amed_d1_additional_medicare_explanations_name_the_2026_form_8959_parts() {
    // The 2026 draft Form 8959: Part I wages (line 7), Part II RRTA, Part III line 12 to Schedule 2
    // line 17b, Part IV self-employment (line 18) to Schedule 2 line 11. i8959's Kathleen and Liam
    // example: wages $130,000 and a spouse's self-employment of $140,000: line 18 is $180.
    let mut d = empty_return(FederalStatus::Joint);
    add(&mut d, wage(0, Owner::Taxpayer, 13_000_000, 0));
    let mut e = income(cash_business(151_597));
    e.owner = crate::DocumentOwner::Spouses;
    add(&mut d, e);
    let r = supported(&d);
    assert_eq!(value(&r, "additional_medicare_self_employment"), 180);
    let text = |id: &str| {
        r.lines
            .iter()
            .find(|l| l.id == id)
            .unwrap_or_else(|| panic!("no {id}"))
            .explanation
            .english()
    };
    assert!(
        text("additional_medicare_self_employment").contains("Form 8959 Part IV line 18"),
        "{}",
        text("additional_medicare_self_employment")
    );
    assert!(
        text("additional_medicare_wages").contains("Form 8959 Part III line 12"),
        "{}",
        text("additional_medicare_wages")
    );
}

fn nonresident_spouse_return(elected: bool, spouse_wages: i64) -> FederalDraft {
    // A section 6013(g) joint return: the U.S. citizen taxpayer has W-2 $100,000 and bank
    // interest $100,000; the spouse (formerly a nonresident alien) has W-2 $50,000.
    let mut d = empty_return(FederalStatus::Joint);
    add(&mut d, wage(0, Owner::Taxpayer, 10_000_000, 0));
    add(&mut d, income(savings_income(100_000, 0)));
    add(&mut d, wage(0, Owner::Spouse, spouse_wages * 100, 0));
    d.set_situations(SpouseSituations::NONRESIDENT_SPOUSE.0, true);
    if elected {
        d.set_situations(SpouseSituations::NIIT_JOINT_ELECTION.0, true);
    }
    d
}

#[test]
fn niit_d4_nonresident_spouse_return_defaults_to_the_separate_treatment() {
    // i8960 "Election To File Jointly With Nonresident Spouse": without the NIIT election the
    // U.S. spouse is treated as married filing separately: modified AGI $200,000 less $125,000,
    // net investment income $100,000: 75,000 x 3.8% = $2,850.
    let r = supported(&nonresident_spouse_return(false, 50_000));
    assert_eq!(printed(&r, TaxForm::Form8960, "13"), 200_000);
    assert_eq!(printed(&r, TaxForm::Form8960, "14"), 125_000);
    assert_eq!(printed(&r, TaxForm::Form8960, "17"), 2_850);
    assert_eq!(printed(&r, TaxForm::Schedule2, "6"), 2_850);
    assert!(noted(&r, "NOTE_NIIT_NONRESIDENT_SPOUSE_SEPARATE"));
    // With the nonresident spouse the taxpayer: the spouse's own figures (here none) apply.
    let mut d = nonresident_spouse_return(false, 50_000);
    d.set_situations(SpouseSituations::YOU_WERE_THE_NONRESIDENT.0, true);
    let r = estimate(&d);
    assert_eq!(printed(&r, TaxForm::Schedule2, "6"), 0);
}

#[test]
fn niit_d4_joint_election_uses_the_joint_amounts() {
    // With the election, spouse W-2 $50,000: joint MAGI $250,000 is not above $250,000: no tax and
    // no form. With $60,000: MAGI $260,000, excess $10,000 x 3.8% = $380.
    let r = supported(&nonresident_spouse_return(true, 50_000));
    assert!(
        !r.lines
            .iter()
            .any(|l| { l.form_line.as_ref().is_some_and(|f| f.form == TaxForm::Form8960) })
    );
    let r = supported(&nonresident_spouse_return(true, 60_000));
    assert_eq!(printed(&r, TaxForm::Form8960, "13"), 260_000);
    assert_eq!(printed(&r, TaxForm::Form8960, "14"), 250_000);
    assert_eq!(printed(&r, TaxForm::Schedule2, "6"), 380);
    assert!(noted(&r, "NOTE_NIIT_NONRESIDENT_SPOUSE_ELECTED"));
}

#[test]
fn niit_d4_items_need_the_resident_choice_and_a_joint_return() {
    use crate::interview::Interview;
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    // The election rides on the resident choice.
    let before = s.draft.checklist;
    rejected_with(
        save_whole_return(&mut s, SpouseSituations::NIIT_JOINT_ELECTION.0),
        "WHOLE_RETURN_NIIT_NEEDS_THE_RESIDENT_CHOICE",
    );
    assert_eq!(s.draft.checklist, before);
    accepted(&save_whole_return(
        &mut s,
        SpouseSituations::NONRESIDENT_SPOUSE.0 | SpouseSituations::NIIT_JOINT_ELECTION.0,
    ));
    assert!(noted(&estimate(&s.draft), "NOTE_NIIT_NONRESIDENT_SPOUSE_ELECTED"));
    // A return that is not joint has no spouse page: About you does not
    // offer the items.
    let mut single = Interview::new(TEST_USER, complete(FederalStatus::Single));
    rejected_with(
        save_your_situations(
            &mut single,
            SpouseSituations::NONRESIDENT_SPOUSE.0 | SpouseSituations::NIIT_JOINT_ELECTION.0,
        ),
        "MESSAGE_CHOOSE_ONLY_THE_LISTED_ITEMS",
    );
}

#[test]
fn credits_d1_form_3800_line_7_leaves_out_the_education_credit_recapture() {
    // 2026 draft i3800 line 7: "Don't include any recapture of an education credit". Single,
    // W-2 $60,000 (Tax Table $5,023), a $10,000 Form 8881 Part I carryforward from 2020 and a $250
    // American opportunity credit recapture: line 7 $5,023, line 38 $5,023, Form 1040 line 16
    // $5,273 so line 22 and total tax $250, carryforward $4,977.
    let mut d = complete(FederalStatus::Single);
    let credits = d.credits.as_mut().unwrap();
    credits.business_credits = Some(BusinessCreditHistory {
        credits: Vec::new(),
        carryforwards: vec![BusinessCreditCarryforward {
            filer: crate::CarryFiler::SameFilers,
            research_source: None,
            credit: BusinessCredit::PensionStartup,
            origin_year: 2020,
            amount: 10_000 * 100,
            passive: None,
        }],
        carrybacks: vec![],
        spouse_without_credits: false,
    });
    credits.education_recapture = Some(PriorYearEducationCreditRecapture {
        method: EducationMethod::AmericanOpportunity,
        filed_qualified_expenses: 700_000,
        after_filing_reduction: 400_000,
        filed_education_magi: 5_000_000,
        filed_joint: false,
        filed_credit_used: 250_000,
        ordinary_prior_return: true,
    });
    let r = supported(&d);
    assert_eq!(value(&r, "form_3800_line_7"), 5_023);
    assert_eq!(value(&r, "form_3800_line_38"), 5_023);
    assert_eq!(return_line(&r, "16"), 5_273 * 100);
    assert_eq!(return_line(&r, "22"), 250 * 100);
    assert_eq!(r.tax, Some(250 * 100));
    // The $10,000 carryforward is used up to line 7: $5,023 used, $4,977 carried to 2027.
    let used = r
        .lines
        .iter()
        .find(|l| l.id.starts_with("business_credit_") && l.id.ends_with("_used"))
        .expect("the carryforward's used row");
    assert_eq!(used.amount / 100, 5_023);
}
