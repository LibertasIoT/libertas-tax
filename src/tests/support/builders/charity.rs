use crate::tests::*;

pub(in crate::tests) fn gift_appraisal(id: i64, report_date: u32) -> Appraisal {
    Appraisal {
        id,
        appraiser: "Synthetic qualified appraiser".into(),
        declaration_date: report_date,
        report_date,
        effective_date: report_date,
        qualified: true,
        declaration_signed: true,
    }
}
pub(in crate::tests) fn set_gift_appraisal(ded: &mut FederalDeductions, index: usize, id: i64) {
    let date = ded.property_gifts[index].given;
    ded.property_gifts[index].appraisal = Some(id);
    ded.property_gifts[index].donee_acknowledged = true;
    if !ded.appraisals.iter().any(|a| a.id == id) {
        ded.appraisals.push(gift_appraisal(id, date));
    }
}
pub(in crate::tests) fn used_goods(value: i64) -> PropertyGift {
    PropertyGift {
        description: "Used coat".into(),
        charity: "Synthetic public charity".into(),
        donee: DoneeKind::PublicCharity,
        given: 20261115,
        how_acquired: AcquiredBy::Purchase,
        acquired: 20200110,
        basis: 100_000,
        value,
        value_method: ValueMethod::ThriftShopValue,
        benefit_received: None,
        appraisal: None,
        in_scope: true,
        donee_acknowledged: true,
        kind: GiftKind::Goods { kind: GoodsKind::Clothing, condition: GoodsCondition::Good },
    }
}

pub(in crate::tests) fn prior_household_carry() -> CharityCarryover {
    CharityCarryover {
        research_parts: None,
        year: 2025,
        class: CharityClass::Property50,
        balance: 300_000,
        appreciation: None,
        same_filers: true,
    }
}

pub(in crate::tests) fn donated_personal_vehicle(treatment: VehicleUse) -> PropertyGift {
    PropertyGift {
        description: "2020 synthetic sedan, good condition, 60,000 miles".into(),
        charity: "Synthetic public charity".into(),
        donee: DoneeKind::PublicCharity,
        given: 20260910,
        how_acquired: AcquiredBy::Purchase,
        acquired: 20200101,
        basis: 3_500_000,
        value: 2_000_000,
        value_method: ValueMethod::VehicleGuide,
        benefit_received: None,
        appraisal: None,
        in_scope: true,
        donee_acknowledged: true,
        kind: GiftKind::Vehicle { vin: "TESTVIN1234567890".into(), treatment },
    }
}

pub(in crate::tests) fn volunteer_travel(
    car_cost: VolunteerCarCost,
    parking_tolls: i64,
) -> VolunteerExpense {
    VolunteerExpense {
        date: 20260620,
        charity: "Synthetic public charity".into(),
        service: "Delivered meals to homebound neighbors".into(),
        kind: VolunteerExpenseKind::Car { car_cost, parking_tolls },
        documented: true,
    }
}

pub(in crate::tests) fn whaling_expense(kind: WhalingExpenseKind, paid: i64) -> WhalingExpense {
    WhalingExpense {
        paid_date: 20260615,
        place: "Synthetic North Slope activity area, Alaska".into(),
        description: "Reasonable necessary sanctioned subsistence whaling expense".into(),
        kind,
        paid,
        report_reference: "synthetic-contemporaneous-expense-log".into(),
        payment_evidence: Some(WhalingPaymentEvidence::Original {
            record_reference: "synthetic-receipt".into(),
        }),
    }
}

pub(in crate::tests) fn whaling_claim(expenses: Vec<WhalingExpense>) -> WhalingCaptainClaim {
    WhalingCaptainClaim {
        owner: Owner::Taxpayer,
        captain_record_reference: "synthetic-AEWC-captain-recognition".into(),
        aewc_report_reference: "synthetic-2026-AEWC-reports".into(),
        trip_log_reference: "synthetic-2026-departure-return-area-log".into(),
        expenses,
        eligibility_and_records_confirmed: true,
    }
}

pub(in crate::tests) fn cash_carry_with_whaling(
    agi: i64,
    whaling_paid: i64,
    prior_cash: i64,
) -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    draft.income = vec![wage(1, Owner::Taxpayer, agi, 650_000)];
    let deductions = draft.deductions.as_mut().unwrap();
    deductions.charity_carryovers = vec![prior_cash_charity(2025, prior_cash)];
    deductions.whaling_captains.push(whaling_claim(vec![whaling_expense(
        WhalingExpenseKind::CrewAndProvisions,
        whaling_paid,
    )]));
    deductions.choice = DeductionSelection::Itemized { facts: itemized(deductions).clone() };
    draft
}

pub(in crate::tests) fn cash_carry_with_student_and_whaling(
    agi: i64,
    months: u32,
    whaling_paid: i64,
    prior_cash: i64,
) -> FederalDraft {
    let mut draft = cash_carry_with_whaling(agi, whaling_paid, prior_cash);
    let student = cash_carry_with_hosted_student(agi, months, prior_cash)
        .deductions
        .unwrap()
        .hosted_students
        .remove(0);
    draft.deductions.as_mut().unwrap().hosted_students.push(student);
    draft
}

pub(in crate::tests) fn with_current_direct_cash(
    mut draft: FederalDraft,
    current_cash: i64,
) -> FederalDraft {
    let deductions = draft.deductions.as_mut().unwrap();
    deductions.set_cash_charity(current_cash);
    draft
}

pub(in crate::tests) fn public_stock_gift() -> PropertyGift {
    PropertyGift {
        description: "Synthetic publicly traded common shares".into(),
        charity: "Synthetic U.S. public charity".into(),
        donee: DoneeKind::PublicCharity,
        given: 20260615,
        how_acquired: AcquiredBy::Purchase,
        acquired: 20240101,
        basis: 500_000,
        value: 2_000_000,
        value_method: ValueMethod::MarketQuotation,
        benefit_received: None,
        appraisal: None,
        in_scope: true,
        donee_acknowledged: true,
        kind: GiftKind::PublicStock { family_over_ten_percent: false },
    }
}

pub(in crate::tests) fn prior_public_stock_carry() -> CharityCarryover {
    CharityCarryover {
        research_parts: None,
        year: 2025,
        class: CharityClass::CapitalGain30,
        balance: 1_000_000,
        appreciation: Some(750_000),
        same_filers: true,
    }
}

pub(in crate::tests) fn paired_2025_noncash_carries() -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    let deductions = draft.deductions.as_mut().unwrap();
    deductions.charity_carryovers.push(prior_household_carry());
    let mut stock = prior_public_stock_carry();
    // Both original gifts could carry only if the 2025 50% room was full;
    // the 2025 stock lot must then remain entirely undeducted.
    stock.balance = 2_000_000;
    stock.appreciation = Some(1_500_000);
    deductions.charity_carryovers.push(stock);
    draft
}
