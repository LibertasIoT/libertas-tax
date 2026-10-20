use super::*;

#[test]
fn unit13_k1_portfolio_niit_partner_taxpayer() {
    assert_k1_portfolio_owners(K1PortfolioNiitSource::Partner, Owner::Taxpayer, false, false);
}

#[test]
fn unit13_k1_portfolio_niit_partner_spouse() {
    assert_k1_portfolio_owners(K1PortfolioNiitSource::Partner, Owner::Spouse, false, false);
}

#[test]
fn unit13_k1_portfolio_niit_scorp_taxpayer() {
    assert_k1_portfolio_owners(K1PortfolioNiitSource::Scorp, Owner::Taxpayer, false, false);
}

#[test]
fn unit13_k1_portfolio_niit_scorp_spouse() {
    assert_k1_portfolio_owners(K1PortfolioNiitSource::Scorp, Owner::Spouse, false, false);
}

#[test]
fn unit13_k1_portfolio_niit_estate_taxpayer() {
    assert_k1_portfolio_owners(K1PortfolioNiitSource::Estate, Owner::Taxpayer, false, false);
}

#[test]
fn unit13_k1_portfolio_niit_estate_spouse() {
    assert_k1_portfolio_owners(K1PortfolioNiitSource::Estate, Owner::Spouse, false, false);
}

#[test]
fn unit13_k1_portfolio_niit_partner_code_a_taxpayer() {
    assert_k1_portfolio_owners(K1PortfolioNiitSource::Partner, Owner::Taxpayer, true, false);
}

#[test]
fn unit13_k1_portfolio_niit_partner_code_a_spouse() {
    assert_k1_portfolio_owners(K1PortfolioNiitSource::Partner, Owner::Spouse, true, false);
}

#[test]
fn unit13_k1_portfolio_niit_scorp_code_a_taxpayer() {
    assert_k1_portfolio_owners(K1PortfolioNiitSource::Scorp, Owner::Taxpayer, true, false);
}

#[test]
fn unit13_k1_portfolio_niit_scorp_code_a_spouse() {
    assert_k1_portfolio_owners(K1PortfolioNiitSource::Scorp, Owner::Spouse, true, false);
}

#[test]
fn unit13_k1_portfolio_niit_estate_final67e_taxpayer() {
    assert_k1_portfolio_owners(K1PortfolioNiitSource::Estate, Owner::Taxpayer, false, true);
}

#[test]
fn unit13_k1_portfolio_niit_estate_final67e_spouse() {
    assert_k1_portfolio_owners(K1PortfolioNiitSource::Estate, Owner::Spouse, false, true);
}

#[test]
fn unit13_k1_portfolio_niit_and_additional_medicare_taxpayer() {
    assert_k1_portfolio_owners_context(
        K1PortfolioNiitSource::Partner,
        Owner::Taxpayer,
        false,
        false,
        true,
    );
}

#[test]
fn unit13_k1_portfolio_niit_and_additional_medicare_spouse() {
    assert_k1_portfolio_owners_context(
        K1PortfolioNiitSource::Partner,
        Owner::Spouse,
        false,
        false,
        true,
    );
}
