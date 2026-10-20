use crate::kernel::carry::{Amount, Cents};
use crate::kernel::loss_limits::vintages;
use crate::scorp_basis::{BasisError, LossRelease, LossVintage};
use crate::tests::*;

// Dollar-denominated fixture facts enter the actual typed source-cent owners.
pub(in crate::tests) fn k1_amount(dollars: i64) -> Amount<Cents> {
    Amount::new(dollars.checked_mul(100).expect("finite fixture dollars")).unwrap()
}

pub(in crate::tests) fn prior_atnol(years: &[(u32, i64, bool)]) -> AtnolHistory {
    AtnolHistory::Records {
        years: years
            .iter()
            .map(|&(year, balance, special_exception)| AtnolYear {
                year,
                balance: balance * 100,
                special_exception,
            })
            .collect(),
        confirmed: true,
    }
}

pub(in crate::tests) fn prior_nol(years: &[(u32, i64)]) -> NolHistory {
    NolHistory::Records {
        years: years
            .iter()
            .map(|&(year, amount)| NolYear {
                ptp_ebl: crate::PtpEblNol::NoPtpEbl,
                year,
                balance: amount * 100,
                injured_owners: None,
                research_parts: None,
            })
            .collect(),
        confirmed: true,
    }
}

pub(in crate::tests) fn nol_cash_gift(years: &[(u32, i64)]) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    ded.choice = DeductionSelection::Itemized { facts: itemized(ded).clone() };
    ded.set_cash_charity(4000000);
    d.screening.as_mut().unwrap().nol = prior_nol(years);
    d
}

pub(in crate::tests) fn prior_cash_charity(year: u32, dollars: i64) -> CharityCarryover {
    CharityCarryover {
        research_parts: None,
        year,
        class: CharityClass::Cash60,
        balance: dollars * 100,
        appreciation: None,
        same_filers: true,
    }
}

pub(in crate::tests) fn ordinary_property_for_cash_carry(value: i64) -> PropertyGift {
    let mut gift = used_goods(value);
    gift.description = "Synthetic personally purchased used watch".into();
    gift.given = 20260615;
    gift.kind = GiftKind::Goods { kind: GoodsKind::Jewelry, condition: GoodsCondition::Good };
    gift.basis = value + 100_000;
    gift.value_method =
        if value > 500_000 { ValueMethod::Appraisal } else { ValueMethod::ComparableSales };
    gift.appraisal = (value > 500_000).then_some(2);
    gift
}

pub(in crate::tests) fn cash_carry_with_current_property(
    agi: i64,
    gift: i64,
    prior_cash: i64,
) -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    draft.income = vec![wage(1, Owner::Taxpayer, agi, 650_000)];
    let deductions = draft.deductions.as_mut().unwrap();
    deductions.charity_carryovers = vec![prior_cash_charity(2025, prior_cash)];
    deductions.property_gifts.push(ordinary_property_for_cash_carry(gift));
    if gift > 500_000 {
        deductions.appraisals.push(gift_appraisal(2, 20260615));
        draft.next_id = 3;
    }
    deductions.choice = DeductionSelection::Itemized { facts: itemized(deductions).clone() };
    draft
}

pub(in crate::tests) fn direct_cash_prior_cash_and_property(
    agi: i64,
    current_cash: i64,
    prior_cash: i64,
    property: i64,
) -> FederalDraft {
    let mut draft = cash_carry_with_current_property(agi, property, prior_cash);
    let deductions = draft.deductions.as_mut().unwrap();
    deductions.set_cash_charity(current_cash);
    draft
}

pub(in crate::tests) fn cash_carry_with_hosted_student(
    agi: i64,
    months: u32,
    prior_cash: i64,
) -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    draft.income = vec![wage(1, Owner::Taxpayer, agi, 650_000)];
    let deductions = draft.deductions.as_mut().unwrap();
    deductions.charity_carryovers = vec![prior_cash_charity(2025, prior_cash)];
    deductions.hosted_students.push(HostedStudent {
        label: "Synthetic hosted student".into(),
        sponsor: "Synthetic educational charity".into(),
        agreement: "Synthetic written education agreement".into(),
        school: "Synthetic year-round high school".into(),
        school_location: "Synthetic City, CA".into(),
        grade: 11,
        joined: 20260101,
        school_start: 20260101,
        school_end: 20261231,
        months: (1..=months)
            .map(|month| HostedStudentMonth { month, qualifying_days: 15, cost: 5_000 })
            .collect(),
        documented: true,
    });
    deductions.choice = DeductionSelection::Itemized { facts: itemized(deductions).clone() };
    draft
}

pub(in crate::tests) fn cash_carry_with_appreciated_stock(
    agi: i64,
    gift: i64,
    prior_cash: i64,
) -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    draft.income = vec![wage(1, Owner::Taxpayer, agi, 650_000)];
    let deductions = draft.deductions.as_mut().unwrap();
    deductions.charity_carryovers = vec![prior_cash_charity(2025, prior_cash)];
    let mut stock = public_stock_gift();
    stock.value = gift;
    stock.basis = (gift / 4).min(500_000);
    deductions.property_gifts.push(stock);
    deductions.choice = DeductionSelection::Itemized { facts: itemized(deductions).clone() };
    draft
}

pub(in crate::tests) fn current_and_prior_cash_with_appreciated_stock(
    agi: i64,
    current_cash: i64,
    prior_cash: i64,
    stock_gift: i64,
) -> FederalDraft {
    let mut draft = cash_carry_with_appreciated_stock(agi, stock_gift, prior_cash);
    let deductions = draft.deductions.as_mut().unwrap();
    deductions.set_cash_charity(current_cash);
    draft
}

pub(in crate::tests) fn cash_carry_with_vehicle(
    treatment: VehicleUse,
    prior_cash: i64,
) -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    draft.income = vec![wage(1, Owner::Taxpayer, 10_000_000, 650_000)];
    let deductions = draft.deductions.as_mut().unwrap();
    deductions.charity_carryovers = vec![prior_cash_charity(2025, prior_cash)];
    deductions.property_gifts.push(donated_personal_vehicle(treatment));
    deductions.choice = DeductionSelection::Itemized { facts: itemized(deductions).clone() };
    draft
}

pub(in crate::tests) fn sold_vehicle_for_cash_carry(prior_cash: i64) -> FederalDraft {
    cash_carry_with_vehicle(VehicleUse::Sold { sold: 20261001, proceeds: 1_500_000 }, prior_cash)
}

pub(in crate::tests) fn nol_social() -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 4000000, 0);
    add(&mut d, income(social_security_net(2000000, 0)));
    d
}

pub(in crate::tests) fn nol_work_deductions(
    wages: i64,
    tips: i64,
    overtime: i64,
    interest: i64,
) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, wages * 100, 0);
    let FederalIncome::Wage { data } = &mut d.income[0].income else { unreachable!() };
    data.work_deductions = work_deductions(tips * 100, overtime * 100);
    d.deductions.as_mut().unwrap().vehicle_loans = vec![FederalVehicleInterest {
        label: "Synthetic vehicle".into(),
        interest: interest * 100,
        business: 0,
        original_use: true,
        assembled_in_us: true,
    }];
    d
}

pub(in crate::tests) fn nol_rental(wages: i64) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, wages * 100, 0);
    add(&mut d, income(FederalIncome::Rental { data: rental(9000, 30000) }));
    d
}

pub(in crate::tests) fn nol_adoption() -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 26508000, 0);
    employer_adoption(&mut d, 20000, 0, false);
    d
}

pub(in crate::tests) fn amt_only_capital_history() -> PriorCapitalReturn {
    let mut history = capital_history();
    history.agi = 10000000;
    history.deductions = 0;
    history.short = 0;
    history.long = 0;
    history.deducted_loss = 0;
    history.amt = Some(PriorAmtCapitalReturn {
        taxable_income: 10000000,
        short: -1300000,
        long: 0,
        deducted_loss: 300000,
    });
    history.investment_parts = Some(InvestmentCapitalParts {
        short: 0,
        long: 0,
        amt: Some(InvestmentAmtCapitalParts { short: 1_000_000, long: 0 }),
    });
    history
}

pub(in crate::tests) fn minimum_tax_history() -> MinimumTaxHistory {
    MinimumTaxHistory {
        return_form: crate::MinimumTaxReturnForm::Form1040,
        gains: None,
        status: FederalStatus::Single,
        base: 4000000,
        exclusions: vec![MinimumTaxExclusion {
            source: "2025 Form 6251 line 2a".into(),
            amount: 2000000,
        }],
        regular: 500000,
        amt: 1000000,
        carry: 500000,
        regular_nol: 0,
        mtcnol: crate::MinimumTaxLoss::None,
        foreign_earned: None,
        foreign_credit: crate::MinimumTaxForeignCredit::None,
        old_vehicle_credit: 0,
    }
}

pub(in crate::tests) fn minimum_tax_gains(
    preferential: i64,
    combined: i64,
    zero: i64,
    fifteen: i64,
) -> MinimumTaxRateFacts {
    MinimumTaxGains {
        preferential: preferential * 100,
        combined: combined * 100,
        regular_zero: zero * 100,
        regular_fifteen: fifteen * 100,
    }
    .into()
}

pub(in crate::tests) fn minimum_tax_exclusion_base(history: &mut MinimumTaxHistory, base: i64) {
    // A taxable state-tax refund can occupy regular brackets and then be
    // removed by the negative exclusion on Form 6251 line 2b. Keep prior
    // regular income sufficient for the supplied regular worksheet amounts.
    history.base =
        (base * 100).max(history.gains.as_ref().map_or(0, |g| g.normalized().regular_zero));
    history.exclusions = vec![MinimumTaxExclusion {
        source: "2025 Form 6251 line 2b".into(),
        amount: base * 100 - history.base,
    }];
}

pub(in crate::tests) fn source_owned_nol(draft: &mut FederalDraft, amount: i64, owner: Owner) {
    let mut shares = [0; 2];
    shares[usize::from(owner == Owner::Spouse)] = amount;
    draft.screening.as_mut().unwrap().nol = NolHistory::Records {
        confirmed: true,
        years: vec![NolYear {
            ptp_ebl: crate::PtpEblNol::NoPtpEbl,
            research_parts: None,
            year: 2017,
            balance: amount,
            injured_owners: Some(InjuredNolOwners::Shares {
                taxpayer: shares[0],
                spouse: shares[1],
            }),
        }],
    };
}

// Direct transient owner packets only: MAX-sized seeds are not native-valid
// source documents and make no claim about admitting a giant return.
pub(in crate::tests) fn retained_loss_projection(
    balances: &[i64],
) -> crate::estate_trust::LossProjection {
    crate::estate_trust::pooled_loss_vintages(
        balances
            .iter()
            .enumerate()
            .map(|(index, balance)| crate::estate_trust::LossVintage {
                origin: Some(index as i64 + 1),
                year: 2018,
                post_2017: true,
                balance: crate::kernel::ratio::exact::whole(*balance),
                special: false,
                expiration: None,
                period: Some((20180101, 20181231)),
            })
            .collect(),
    )
}

// Project the existing source release into the shared next-limit owner.
pub(in crate::tests) fn next_loss_limit(
    released: &LossRelease,
    allowed: i64,
) -> Result<(i64, Option<LossVintage>), BasisError> {
    let release = vintages::VintageRelease::<_, Cents> {
        amount: released.amount,
        qualified: released.qbi,
        remaining: released
            .remaining
            .iter()
            .map(|v| {
                Ok(vintages::LossVintage {
                    key: &v.year,
                    original: v.original,
                    original_qualified: v.original_qbi,
                    remaining: v.remaining,
                    remaining_qualified: v.remaining_qbi,
                })
            })
            .collect::<Result<_, BasisError>>()?,
    };
    let next = release
        .after_next_limit(
            Amount::new(allowed.checked_mul(100).ok_or(BasisError::Invalid)?)
                .map_err(|_| BasisError::Invalid)?,
            2026,
        )
        .map_err(|_| BasisError::Invalid)?;
    Ok((
        crate::kernel::ratio::dollars(next.allowed_qualified.get()),
        next.held.map(|v| LossVintage {
            year: v.key,
            original: v.original,
            original_qbi: v.original_qualified,
            remaining: v.remaining,
            remaining_qbi: v.remaining_qualified,
        }),
    ))
}

pub(in crate::tests) fn four_classes(values: [i64; 4]) -> LossAmounts {
    LossAmounts::of(&[
        (K1LossClass::Ordinary, Amount::<Cents>::new(values[0] * 100).unwrap()),
        (K1LossClass::ShortTerm, Amount::<Cents>::new(values[1] * 100).unwrap()),
        (K1LossClass::LongTerm, Amount::<Cents>::new(values[2] * 100).unwrap()),
        (K1LossClass::Collectibles, Amount::<Cents>::new(values[3] * 100).unwrap()),
    ])
}

pub(in crate::tests) fn five_classes(values: [i64; 5]) -> LossAmounts {
    four_classes([values[0], values[1], values[2], values[3]]).plus(&LossAmounts::of(&[(
        K1LossClass::Section1231,
        Amount::<Cents>::new(values[4] * 100).unwrap(),
    )]))
}

// Native-cent source facts, through the ordinary dividend and NOL owners.
pub(in crate::tests) fn ptp_ebl_nol_draft(
    status: FederalStatus,
    wages: i64,
    reit: i64,
    history: NolHistory,
) -> FederalDraft {
    let mut draft = complete(status);
    draft.income = vec![wage(1, Owner::Taxpayer, wages, 0)];
    let mut dividend = native_dividend(reit, 0, 0);
    dividend.section199a = ReitDividends::Reported { amount: reit, eligible: true };
    add(&mut draft, income(FederalIncome::Dividend { data: dividend }));
    draft.screening.as_mut().unwrap().nol = history;
    draft
}
