use crate::installment::{
    CurrentSchedule, InstallmentPayments, InstallmentQbi, InstallmentUse,
    InvestmentInstallmentSale, PropertyCharacter, PropertyUse, SecondDisposition,
};
use crate::kernel::installment::{PriorSaleCents, Schedule, payment_year};
use crate::kernel::ratio::dollars;
use crate::line_emission::push_line;
use crate::{FederalDraft, FederalResult, FormLine, Owner, TaxForm};
use alloc::{format, vec::Vec};

impl InvestmentInstallmentSale {
    pub(crate) fn term(&self) -> crate::GainTerm {
        if self.sold > self.acquired + 10000 {
            crate::GainTerm::Long
        } else {
            crate::GainTerm::Short
        }
    }

    // The same actual sale, paid in full, enters the existing capital-sale owner.
    pub(crate) fn full_sale(&self) -> crate::FederalSale {
        crate::FederalSale {
            term: self.term(),
            collectible: self.collectible,
            proceeds: self.price,
            basis: Some(self.basis),
            amt: crate::SaleAmt::SameAsRegular,
            withholding: 0,
            use_kind: match self.use_kind {
                InstallmentUse::Investment => crate::SaleUse::Investment { wash_sale: 0 },
                InstallmentUse::Personal => crate::SaleUse::Personal,
            },
            special: crate::SaleSpecial::NoneOfThese,
            adjustment: crate::SaleAdjustment::SellingExpenses { amount: self.selling_expenses },
            basis_reporting: crate::BasisReporting::NoForm1099B,
            digital_asset: false,
            market_discount: 0,
            several_sales: false,
        }
    }
}

// Section 453A(b)(2): nondealer notes from 2026 sales above $150,000
// (farm property excepted) outstanding at the end of the year.
pub(crate) fn section_453a(draft: &FederalDraft) -> i64 {
    let mut total = 0;
    let mid_quarter = crate::depreciation::mid_quarter(draft);
    for entry in &crate::community::property_entries(draft) {
        match &entry.income {
            crate::FederalIncome::Business { .. } | crate::FederalIncome::FarmRental { .. } => {
                let assets = match &entry.income {
                    crate::FederalIncome::Business { data } => &data.assets,
                    crate::FederalIncome::FarmRental { data } => &data.assets,
                    _ => continue,
                };
                for asset in assets {
                    if let Some(v) = asset.sale_for(mid_quarter, false)
                        && let Some(form) = v.installment
                        && form.price > 150_000
                    {
                        total += form.outstanding();
                    }
                }
            }
            crate::FederalIncome::Rental { data } => {
                if let Ok(Some(v)) = crate::asset_sale::rental_sale(data, false)
                    && let Some(form) = v.installment
                    && form.price > 150_000
                {
                    total += form.outstanding();
                }
            }
            crate::FederalIncome::InstallmentSale { data: sale } => {
                if sale.issue().is_none()
                    && sale.refusal().is_none()
                    && matches!(sale.use_kind, InstallmentUse::Investment)
                    && let Ok(Some(current)) = sale.calculation()
                    && current.form.price > 150_000
                {
                    total += current.form.outstanding();
                }
            }
            _ => {}
        }
    }
    total
}

const SOURCE: &str = "https://www.irs.gov/pub/irs-dft/f6252--dft.pdf";

fn row(
    r: &mut FederalResult,
    line: &str,
    id: &str,
    amount: i64,
    input: &str,
    explanation: crate::Message,
) {
    push_line(
        r,
        Some(FormLine::new(TaxForm::Form6252, line)),
        id,
        amount * 100,
        &[input],
        SOURCE,
        explanation,
    );
}

// One year-of-sale Form 6252's rows, and line 26's destination.
pub(crate) fn report_sale(
    r: &mut FederalResult,
    prefix: &str,
    input: &str,
    form: &Schedule,
    unrecaptured: i64,
    section1231: bool,
) {
    let line13 = form.cost - form.depreciation + form.expenses + form.recapture;
    for (line, suffix, amount, explanation) in [
        ("5", "price", form.price, msg!("EXPLAIN_INSTALLMENT_N")),
        ("6", "assumed_debt", form.debt, msg!("EXPLAIN_INSTALLMENT_N_2")),
        ("7", "net_price", form.price - form.debt, msg!("EXPLAIN_INSTALLMENT_N_3")),
        ("8", "cost", form.cost, msg!("EXPLAIN_INSTALLMENT_N_4")),
        ("9", "depreciation", form.depreciation, msg!("EXPLAIN_INSTALLMENT_N_5")),
        ("10", "adjusted_basis", form.cost - form.depreciation, msg!("EXPLAIN_INSTALLMENT_N_6")),
        ("11", "expenses", form.expenses, msg!("EXPLAIN_INSTALLMENT_N_7")),
        ("12", "recapture", form.recapture, msg!("EXPLAIN_INSTALLMENT_N_8")),
        ("13", "total", line13, msg!("EXPLAIN_INSTALLMENT_N_9")),
        ("14", "profit", form.price - line13, msg!("EXPLAIN_INSTALLMENT_PROFIT_BEFORE_EXCLUSION")),
        ("16", "gross_profit", form.gross_profit, msg!("EXPLAIN_INSTALLMENT_N_10")),
        ("17", "excess_debt", form.excess_debt, msg!("EXPLAIN_INSTALLMENT_N_11")),
        ("18", "contract_price", form.contract, msg!("EXPLAIN_INSTALLMENT_N_12")),
        ("20", "deemed_payment", form.excess_debt, msg!("EXPLAIN_INSTALLMENT_N_13")),
        ("21", "payments", form.payments, msg!("EXPLAIN_INSTALLMENT_N_14")),
        (
            "22",
            "total_payments",
            form.excess_debt + form.payments,
            msg!("EXPLAIN_INSTALLMENT_N_15"),
        ),
    ] {
        row(r, line, &format!("{prefix}_{suffix}"), amount, input, explanation);
    }
    r.lines.push(crate::kernel::lines::row(
        Some(FormLine::new(TaxForm::Form6252, "19")),
        &format!("{prefix}_percentage"),
        crate::kernel::lines::LineValue::RatioTenThousandths(crate::kernel::ratio::fraction(
            form.gross_profit,
            10_000,
            form.contract,
        )),
        &[input],
        SOURCE,
        msg!("EXPLAIN_INSTALLMENT_GROSS_PROFIT_RATIO"),
    ));
    report_income(r, prefix, input, form, unrecaptured, section1231);
}

pub(crate) fn report_current_adjustments(
    r: &mut FederalResult,
    id: i64,
    sale: &InvestmentInstallmentSale,
    current: &CurrentSchedule,
) {
    let prefix = format!("installment_sale_{id}");
    let input = "income.installment_sale";
    row(
        r,
        "15",
        &format!("{prefix}_excluded"),
        0,
        input,
        msg!("EXPLAIN_INSTALLMENT_NO_HOME_EXCLUSION"),
    );
    if current.pledged > 0 {
        push_line(
            r,
            None,
            &format!("{prefix}_pledged"),
            current.pledged,
            &[input],
            SOURCE,
            msg!("EXPLAIN_INSTALLMENT_PLEDGE"),
        );
    }
    if let Some(acceleration) = current.acceleration
        && let SecondDisposition::Resold { amount_realized, .. } = sale.second_disposition
    {
        for (line, suffix, amount) in [
            ("30", "resale_realized", dollars(amount_realized)),
            ("31", "resale_contract_price", current.form.contract),
            ("32", "lesser", dollars(acceleration.limited_realized_cents)),
            ("33", "payments_to_date", current.form.excess_debt + current.form.payments),
            ("34", "accelerated", dollars(acceleration.deemed_cents)),
            ("35", "resale_income", acceleration.income_dollars),
            ("36", "resale_ordinary", 0),
            ("37", "resale_gain", acceleration.income_dollars),
        ] {
            row(
                r,
                line,
                &format!("{prefix}_{suffix}"),
                amount,
                input,
                msg!("EXPLAIN_INSTALLMENT_RELATED_ACCELERATION"),
            );
        }
    }
}

fn report_income(
    r: &mut FederalResult,
    prefix: &str,
    input: &str,
    form: &Schedule,
    unrecaptured: i64,
    section1231: bool,
) {
    row(r, "24", &format!("{prefix}_income"), form.income, input, msg!("EXPLAIN_N_INCOME"));
    row(r, "25", &format!("{prefix}_ordinary"), form.ordinary, input, msg!("EXPLAIN_N_ORDINARY"));
    row(
        r,
        "26",
        &format!("{prefix}_gain"),
        form.gain,
        input,
        if section1231 { msg!("EXPLAIN_N_GAIN") } else { msg!("EXPLAIN_N_GAIN_2") },
    );
    if section1231 && unrecaptured > 0 {
        push_line(
            r,
            None,
            &format!("{prefix}_unrecaptured_1250"),
            unrecaptured * 100,
            &[input],
            "https://www.irs.gov/instructions/i1040sd",
            msg!("EXPLAIN_N_UNRECAPTURED_N_2"),
        );
    }
}

fn earlier(draft: &FederalDraft) -> impl Iterator<Item = (usize, &InstallmentPayments)> {
    draft
        .screening
        .iter()
        .flat_map(|s| s.installment_payments.iter())
        .map(|v| &v.sale)
        .enumerate()
        .filter(|(_, v)| v.supported())
}

// Earlier-year sales' Schedule D gains in cents, [short-term, long-term].
// Same recognized amounts with their retained filer, before aggregation.
pub(crate) fn earlier_capital_sources(draft: &FederalDraft) -> ([i64; 2], [[i64; 2]; 2]) {
    let mut total = [0, 0];
    let mut owners = [[0; 2]; 2];
    for record in
        draft.screening.iter().flat_map(|s| &s.installment_payments).filter(|v| v.sale.supported())
    {
        let sale = &record.sale;
        let (gain, _) = sale.recognized();
        let term = match sale.kind {
            PropertyCharacter::ShortTermCapital => 0,
            PropertyCharacter::LongTermCapital => 1,
            PropertyCharacter::BusinessProperty { .. } => continue,
        };
        total[term] += gain * 100;
        owners[usize::from(record.owner == Owner::Spouse)][term] += gain * 100;
    }
    (total, owners)
}

// Positive recognized capital amounts before netting separate retained sources.
// The original recognition owner supplies the gain; proceeds are never gross income.
pub(crate) fn earlier_capital_gross(draft: &FederalDraft) -> Option<i64> {
    draft
        .screening
        .iter()
        .flat_map(|screen| &screen.installment_payments)
        .filter(|record| record.sale.supported())
        .filter(|record| {
            matches!(
                record.sale.kind,
                PropertyCharacter::ShortTermCapital | PropertyCharacter::LongTermCapital
            )
        })
        .try_fold(0_i64, |gross, record| {
            let gain =
                crate::kernel::ratio::positive(record.sale.recognized().0).checked_mul(100)?;
            gross.checked_add(gain)
        })
}

// The synthetic source id of an earlier-year sale; never an income entry's.
fn source_id(index: usize) -> i64 {
    -1 - index as i64
}

// Resolve the native source identity against its retained seller. This is
// metadata only: do not run Form6252 again to establish an owner's amount.
pub(crate) fn earlier_source_owner(draft: &FederalDraft, id: i64) -> Option<Owner> {
    draft
        .screening
        .as_ref()?
        .installment_payments
        .iter()
        .enumerate()
        .find(|(index, record)| {
            source_id(*index) == id
                && record.sale.supported()
                && matches!(record.sale.kind, PropertyCharacter::BusinessProperty { .. })
        })
        .map(|(_, record)| record.owner)
}

// Earlier-year business and rental property sales as section 1231 sources
// (Form 4797 line 4), with whether each is a business you materially
// participated in (not net investment income, not passive).
pub(crate) fn earlier_sources(
    draft: &FederalDraft,
) -> Vec<(i64, crate::farm_land::LandGain, bool)> {
    earlier(draft)
        .filter_map(|(index, sale)| {
            let PropertyCharacter::BusinessProperty { used, .. } = &sale.kind else {
                return None;
            };
            let (gain, unrecaptured) = sale.recognized();
            Some((
                source_id(index),
                crate::farm_land::LandGain {
                    gain,
                    percent: 0,
                    recapture: 0,
                    ordinary: 0,
                    section1231: gain,
                    unrecaptured,
                },
                *used == PropertyUse::Business,
            ))
        })
        .collect()
}

// https://www.irs.gov/publications/p505#en_US_2026_publink1000194740
// Pub. 505 (2026), Farming and Fishing, includes recognized draft, breeding,
// dairy and sporting livestock gains. The payment-year owner already computes
// Form 6252 line 26 once in whole dollars; neither proceeds nor the 1231 net
// supplies this numerator. Nullable is only the source/UI carrier: availability
// is resolved by the shared SourceFact owner.
pub(crate) fn farmer_livestock_gross(draft: &FederalDraft) -> Result<i64, &'static str> {
    use crate::kernel::source_fact::SourceFact;
    earlier(draft).try_fold(0_i64, |gross, (_, sale)| {
        let gain = sale.recognized().0;
        if gain <= 0 || !matches!(sale.kind, PropertyCharacter::BusinessProperty { .. }) {
            return Ok(gross);
        }
        let fact = match sale.livestock {
            // Pub. 225, chapter 15, includes farm machinery gains. Not livestock
            // does not establish whether the earlier property was farm machinery.
            None | Some(crate::Livestock::No) => SourceFact::Unavailable,
            Some(livestock) => SourceFact::Reported(livestock),
        };
        let Some(_) = fact.require(|| "ISSUE_FARMER_GROSS_SOURCE_UNAVAILABLE")? else {
            return Ok(gross);
        };
        gain.checked_mul(100)
            .and_then(|amount| gross.checked_add(amount))
            .ok_or("ISSUE_FARMER_GROSS_AMOUNT_UNAVAILABLE")
    })
}

// The part of an earlier-year installment gain a section 1231 lookback makes
// ordinary (cents-free whole dollars): qualified business income of the
// business that sold the property.

fn qbi_part_with_claims(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    character: Option<&crate::section1231::Result1231>,
    index: usize,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<i64, crate::section1231::SourceError> {
    let Some(character) = character else {
        return Ok(0);
    };
    let Some((_, sale)) = earlier(draft).find(|(i, _)| *i == index) else {
        return Ok(0);
    };
    let form = payment_year(PriorSaleCents {
        gross_profit: sale.gross_profit,
        contract_price: sale.contract_price,
        earlier_payments: sale.earlier_payments,
        payments: sale.payments,
    });
    if form.gain <= 0 || character.net <= 0 || character.ordinary == 0 {
        return Ok(0);
    }
    Ok(
        match crate::section1231::activity_character_with_claims(
            draft,
            pool,
            character,
            source_id(index),
            false,
            source_claims,
        )? {
            crate::section1231::PropertyCharacter::Capital => 0,
            crate::section1231::PropertyCharacter::Ordinary => form.gain,
            crate::section1231::PropertyCharacter::Recaptured(amount) => amount,
        },
    )
}

// The qualified business income of earlier-year installment sales by the
// business named for each, in whole dollars: `Some(id)` joins that business,
// `None` is a business no longer on the return (a stand-alone activity with
// no W-2 wages and no qualified property).
pub(crate) type QbiShares = Vec<(Option<i64>, i64, Owner)>;

pub(crate) fn qbi_shares_with_claims(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    character: Option<&crate::section1231::Result1231>,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<QbiShares, crate::section1231::SourceError> {
    let mut shares = Vec::new();
    for (index, sale) in earlier(draft) {
        let (
            PropertyCharacter::BusinessProperty { used: PropertyUse::Business, .. },
            InstallmentQbi::Qualified { business },
        ) = (&sale.kind, &sale.qbi)
        else {
            continue;
        };
        let part = qbi_part_with_claims(draft, pool, character, index, source_claims)?;
        if part == 0 {
            continue;
        }
        // A business the return no longer holds is the same declared state.
        let held = business.filter(|id| {
            draft
                .income
                .iter()
                .any(|e| e.id == *id && matches!(&e.income, crate::FederalIncome::Business { .. }))
        });
        shares.push((
            held,
            part,
            draft.screening.as_ref().unwrap().installment_payments[index].owner,
        ));
    }
    Ok(shares)
}

// The earlier-year sales' rows and refusals, and the return-level section
// 453A limit on 2026 sales. `pool` is the regular-tax passive pool;
// `character` the regular section 1231 result.

pub(crate) fn report_with_claims(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    character: Option<&crate::section1231::Result1231>,
    r: &mut FederalResult,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) {
    let outstanding = section_453a(draft);
    if outstanding > 5_000_000 {
        crate::issue_localized!(
            r,
            &crate::IssueContext::Section1231,
            "Installment sales",
            "ISSUE_INSTALLMENT_NOTES_FROM_RETURN_YEAR_SALES_ABOVE_150_000_TOTAL_MORE_THAN_5_0_5149A91B",
            "Installment notes from 2026 sales above $150,000 total more than $5,000,000 at the end of 2026, so interest is owed on the deferred tax (section 453A, Form 6252 and Publication 537), which is not calculated.",
        );
    }
    let Some(screening) = draft.screening.as_ref() else {
        return;
    };
    for (index, value) in screening.installment_payments.iter().enumerate() {
        let sale = &value.sale;
        #[cfg(test)]
        let section = format!("Installment payments: {}", sale.property);
        if !sale.valid() {
            crate::issue_localized!(
                r,
                &crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
                &section,
                "ISSUE_ENTER_A_SALE_DATE_BEFORE_RETURN_YEAR_A_GROSS_PROFIT_NO_LARGER_THAN_THE_CON_7BB8E34C",
                "Enter a sale date before 2026, a gross profit no larger than the contract price, and payments that do not exceed the contract price (Form 6252 lines 16, 18, 21 and 23).",
            );
            continue;
        }
        if sale.special {
            crate::issue_localized!(
                r,
                &crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
                &section,
                "ISSUE_THIS_EARLIER_YEAR_INSTALLMENT_SALE_IS_NOT_CALCULATED_A_RELATED_PART_7D8BAB22",
                "This earlier-year installment sale is not calculated: a related-party sale, a dealer sale, a contingent price, a pledged or disposed note, remaining recapture, collectibles, small business stock, unstated interest, a different AMT gain or suspended passive losses need their own worksheets (Form 6252 lines 3, 4, 25 and Part III; Publication 537).",
            );
            continue;
        }
        let form = payment_year(PriorSaleCents {
            gross_profit: sale.gross_profit,
            contract_price: sale.contract_price,
            earlier_payments: sale.earlier_payments,
            payments: sale.payments,
        });
        let (_, unrecaptured) = sale.recognized();
        let prefix = format!("installment_{}", index + 1);
        let input = "screening.installment_payments";
        row(
            r,
            "21",
            &format!("{prefix}_payments"),
            form.payments,
            input,
            msg!("EXPLAIN_N_PAYMENTS"),
        );
        row(
            r,
            "22",
            &format!("{prefix}_total_payments"),
            form.payments,
            input,
            msg!("EXPLAIN_N_TOTAL_PAYMENTS"),
        );
        row(
            r,
            "23",
            &format!("{prefix}_earlier_payments"),
            form.earlier_payments,
            input,
            msg!("EXPLAIN_N_EARLIER_PAYMENTS"),
        );
        let business = matches!(sale.kind, PropertyCharacter::BusinessProperty { .. });
        report_income(r, &prefix, input, &form, unrecaptured, business);
        // Form 4797 line 4: an earlier sale of business or rental property
        // brings its Form 6252 line 26 gain into the section 1231 netting.
        if business {
            push_line(
                r,
                Some(FormLine::new(TaxForm::Form4797, "4")),
                &format!("{prefix}_section1231"),
                form.gain * 100,
                &[input],
                "https://www.irs.gov/instructions/i4797",
                msg!("EXPLAIN_INSTALLMENT_N_SECTION1231"),
            );
        }
        match &sale.kind {
            PropertyCharacter::BusinessProperty { used: PropertyUse::Rental, .. }
                if form.gain > 0 && pool.passive_loss > 0 =>
            {
                crate::issue_localized!(
                    r,
                    &crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
                    &section,
                    "ISSUE_INSTALLMENT_GAIN_FROM_AN_EARLIER_SALE_OF_RENTAL_OR_PASSIVE_ACTIVITY_13F232BB",
                    "Installment gain from an earlier sale of rental or passive-activity property is passive income that allows part of this return's unallowed passive losses (Form 8582 lines 1 through 4), which is not calculated.",
                )
            }
            // The part a section 1231 lookback makes ordinary is qualified
            // business income of the business that sold the property
            // (Reg. 1.199A-3(b)(2)(ii)(A)); the figure is the activity the
            // person named, if any.
            PropertyCharacter::BusinessProperty { used: PropertyUse::Business, .. }
                if matches!(sale.qbi, InstallmentQbi::Qualified { .. })
                    && form.gain > 0
                    && character.is_some_and(|v| v.net > 0 && v.ordinary > 0) =>
            {
                match qbi_part_with_claims(draft, pool, character, index, source_claims) {
                    Ok(part) => row(
                        r,
                        "4",
                        &format!("{prefix}_qualified_income"),
                        part,
                        input,
                        msg!("EXPLAIN_INSTALLMENT_N_QUALIFIED_INCOME"),
                    ),
                    Err(error) => error.report_at(
                        r,
                        &crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
                        #[cfg(test)]
                        &section,
                    ),
                }
            }
            _ => {}
        }
    }
}
