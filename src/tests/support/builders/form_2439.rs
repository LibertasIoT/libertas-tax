use crate::tests::*;

fn form2439_paper_native(form: Form2439) -> Form2439Paper {
    let Form2439 { fund, shareholder, gain, unrecaptured, small_business, collectibles, tax_paid } =
        form;
    Form2439Paper {
        ric_or_reit: fund,
        shareholder,
        total_undistributed_long_term_capital_gains: gain,
        unrecaptured_section_1250_gain: unrecaptured,
        section_1202_gain: small_business,
        collectibles_28_percent_gain: collectibles,
        tax_paid_by_ric_or_reit: tax_paid,
        calendar_year: 2026,
        ..Default::default()
    }
}

paper_adapter!(form2439_paper, form2439_paper_native, FundBoxFixture, Form2439, Form2439Paper);
