use crate::tests::*;

pub(in crate::tests) fn community_w2_draft(
    own: i64,
    spouse: i64,
    own_tax: i64,
    spouse_tax: i64,
) -> FederalDraft {
    let mut d = complete(FederalStatus::Separate);
    d.setup.as_mut().unwrap().community_property = community_arizona();
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    d.income =
        vec![wage(1, Owner::Taxpayer, own, own_tax), wage(2, Owner::Spouse, spouse, spouse_tax)];
    for entry in &mut d.income {
        entry.community = CommunityShare::Community;
    }
    d.next_id = 3;
    d
}

// A married-filing-separately return in Arizona that divides every item by half.
pub(in crate::tests) fn community_arizona() -> CommunityProperty {
    CommunityProperty::Yes { state: CommunityState::Arizona, rule: CommunityRule::Halves }
}

pub(in crate::tests) fn community_multi_w2_draft() -> FederalDraft {
    let mut d = community_w2_draft(4_000_002, 7_000_000, 400_002, 700_000);
    add(&mut d, wage(0, Owner::Taxpayer, 1_999_998, 199_998));
    add(&mut d, wage(0, Owner::Spouse, 3_000_000, 300_000));
    d
}

pub(in crate::tests) fn community_interest_draft(
    own_wages: i64,
    spouse_wages: i64,
    interest: i64,
    statement_addressee: Owner,
) -> FederalDraft {
    let mut d = community_w2_draft(own_wages, spouse_wages, 0, 0);
    let mut statement = income(FederalIncome::Interest { data: native_interest(interest, 0) });
    statement.owner = statement_addressee;
    add(&mut d, statement);
    d
}

pub(in crate::tests) fn community_dividend_draft(
    own_wages: i64,
    spouse_wages: i64,
    ordinary: i64,
    qualified: i64,
    statement_addressee: Owner,
) -> FederalDraft {
    let mut d = community_w2_draft(own_wages, spouse_wages, 0, 0);
    let mut statement = income(FederalIncome::Dividend {
        data: FederalDividend {
            foreign_tax: 0,
            section_1202_gain: 0,
            section_1202_exclusion: None,
            ordinary,
            qualified,
            section199a: ReitDividends::None,
            capital_distributions: 0,
            collectible_distributions: 0,
            unrecaptured_distributions: 0,
            withholding: 0,
            special: crate::DividendSpecial::NoneOfThese,
            exempt_dividends: 0,
            private_activity_dividends: 0,
            after_death: None,

            section_897_ordinary: 0,
            section_897_capital: 0,
            foreign_country: alloc::string::String::new(),

            nominee: None,

            nondividend_distributions: 0,
            cash_liquidation: 0,
            noncash_liquidation: 0,
            capital_returns: Vec::new(),
        },
    });
    statement.owner = statement_addressee;
    add(&mut d, statement);
    d
}

pub(in crate::tests) fn community_joint_portfolio_draft(
    own_wages: i64,
    spouse_wages: i64,
    interest: i64,
    ordinary: i64,
    qualified: i64,
) -> FederalDraft {
    let mut d = community_interest_draft(own_wages, spouse_wages, interest, Owner::Spouse);
    add(
        &mut d,
        income(FederalIncome::Dividend {
            data: FederalDividend {
                foreign_tax: 0,
                section_1202_gain: 0,
                section_1202_exclusion: None,
                ordinary,
                qualified,
                section199a: ReitDividends::None,
                capital_distributions: 0,
                collectible_distributions: 0,
                unrecaptured_distributions: 0,
                withholding: 0,
                special: crate::DividendSpecial::NoneOfThese,
                exempt_dividends: 0,
                private_activity_dividends: 0,
                after_death: None,

                section_897_ordinary: 0,
                section_897_capital: 0,
                foreign_country: alloc::string::String::new(),

                nominee: None,

                nondividend_distributions: 0,
                cash_liquidation: 0,
                noncash_liquidation: 0,
                capital_returns: Vec::new(),
            },
        }),
    );
    d
}

pub(in crate::tests) fn community_portfolio_with_second_payers(
    mut d: FederalDraft,
) -> FederalDraft {
    d.income[2].label = "Bank A".into();
    d.income[3].label = "Fund A".into();
    let mut interest = d.income[2].clone();
    interest.label = "Bank B".into();
    interest.owner = Owner::Taxpayer;
    let FederalIncome::Interest { data } = &mut interest.income else { unreachable!() };
    data.taxable = 300_000;
    add(&mut d, interest);
    let mut dividend = d.income[3].clone();
    dividend.label = "Fund B".into();
    dividend.owner = Owner::Spouse;
    let FederalIncome::Dividend { data } = &mut dividend.income else { unreachable!() };
    data.ordinary = 600_000;
    data.qualified = 100_000;
    add(&mut d, dividend);
    d
}

pub(in crate::tests) fn community_portfolio_with_extra_w2s(mut d: FederalDraft) -> FederalDraft {
    let FederalIncome::Wage { data: own } = &mut d.income[0].income else { unreachable!() };
    own.withholding = 600_002;
    let FederalIncome::Wage { data: spouse } = &mut d.income[1].income else { unreachable!() };
    spouse.withholding = 1_000_000;
    let own_id = d.next_id;
    add(&mut d, wage(own_id, Owner::Taxpayer, 1_999_998, 199_998));
    let spouse_id = d.next_id;
    add(&mut d, wage(spouse_id, Owner::Spouse, 4_000_000, 400_000));
    d
}
