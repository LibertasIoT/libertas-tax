// Schedule K-1 box items in the return's totals: the amounts the
// per-code records and the year-end events of a partnership or S corporation
// K-1 bring to Schedule 1 lines 4, 8b, 8c, 8z and 18, Schedule A, Schedule D,
// Form 4797 Part II line 11 (2026), Form 4952, Form 8949 boxes C and F, the 28% and
// unrecaptured section 1250 worksheets, Schedule 2 line 13h and Form 8960.
// The amounts are those the basis and at-risk limits allow (K1Items::flows);
// the passive stage cuts the activity items afterward (extended_rules.rs).
use crate::federal_rules::Totals;
use crate::k1_items::{Disposal, K1ItemFlows};
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::checked_sum;
use crate::line_emission::emit as emit_lines;
use crate::line_emission::push_line;
use crate::{
    FederalIncome, FederalIncomeEntry, FederalResult, FormLine, GainTerm, Message, MessageArg,
    ScorpTradeKind, TaxForm,
};

// What one K-1 brings, in both tax systems.
pub(crate) struct Facts<'a> {
    pub entry: &'a FederalIncomeEntry,
    pub partner: bool,
    pub flows: K1ItemFlows,
    pub amt_flows: K1ItemFlows,
    pub disposal: Disposal,
    pub amt_disposal: Disposal,
    // Whether the K-1's activity is passive or trading, so that its gains are
    // net investment income; false for a nonpassive business that is not
    // trading (section 1411(c)(2)(A) and (c)(4)).
    pub nii_activity: bool,
    // Whether the activity is investment property for Form 4952 (section
    // 163(d)(5)(A)(ii)): trading in which the owner does not materially
    // participate.
    pub investment_activity: bool,
    pub term: Option<GainTerm>,
    // Box 20 codes AC and AD, cents.
    pub collectibles: i64,
    pub unrecaptured: i64,
    pub reit_dividends: i64,
    pub deferred_tax: i64,
    // The section 199A statement's qualified PTP items belong to the QBI
    // stage; nothing here.
    pub source: &'static str,
    // The K-1 has a Schedule K-3: its box 11 (10) items are inside the K-3 Part II line 24 gross
    // income, which Form 1116 line 3e takes in their place (the amounts stay in AGI), so none
    // of them enters line 3e a second time. A sale of the interest or a distribution above
    // basis is the owner's own gain, outside the entity's K-3.
    pub k3: bool,
}

impl Facts<'_> {
    fn prefix(&self) -> &'static str {
        if self.partner { "partnership" } else { "scorp" }
    }
}

// The material-participation facts that decide the two classifications.
pub(crate) fn classify(
    trade: ScorpTradeKind,
    material: crate::participation::Participation,
) -> (bool, bool) {
    let nii = !(trade == ScorpTradeKind::Ordinary
        && material == crate::participation::Participation::Yes);
    let investment = trade == ScorpTradeKind::FinancialTrading
        && material == crate::participation::Participation::No;
    (nii, investment)
}

fn term_index(term: Option<GainTerm>) -> usize {
    usize::from(term == Some(GainTerm::Long))
}

// The one place a K-1 item reaches Schedule D lines 5 and 12: through
// `add_short` / `add_long`, so Form 1116 line 3e (`capital_gross`) sees it; a box 11 (10) item
// of a K-1 with a Schedule K-3 goes through the outside-line-3e twin (its gross is in the K-3).
fn add_capital(t: &mut Totals, term: usize, amount: i64, outside_line_3e: bool) -> Option<()> {
    t.try_add_k1_capital(amount, term != 0, outside_line_3e)
}

// The cents of capital gain (loss) each holding period gets in one tax
// system, and the part that is investment income.
fn capital(f: &Facts, flows: &K1ItemFlows, disposal: &Disposal) -> Option<([i64; 2], [i64; 2])> {
    let mut total = flows.checked_capital()?;
    let mut investment = flows.preferred;
    if f.investment_activity {
        for (amount, nonportfolio) in investment.iter_mut().zip(flows.nonportfolio) {
            *amount = checked_sum([*amount, nonportfolio])?;
        }
    }
    let term = term_index(f.term);
    let events = checked_sum([disposal.distribution_gain, disposal.capital()])?;
    total[term] = checked_sum([total[term], events])?;
    if f.investment_activity {
        investment[term] = checked_sum([investment[term], events])?;
    }
    Some((total, investment))
}

fn ordinary(f: &Facts, flows: &K1ItemFlows, disposal: &Disposal) -> Option<i64> {
    let _ = f;
    checked_sum([flows.ordinary, disposal.ordinary])
}

// Gross portfolio royalties are the same in both tax systems; the allowed
// code I deduction can differ because each system owns its loss ledger.
fn royalties(f: &Facts) -> i64 {
    match &f.entry.income {
        FederalIncome::Partnership { data } => data.separate.as_ref().map_or(0, |v| v.royalties),
        FederalIncome::Scorp { data } => {
            let data = &crate::ScorpK1::from(data);
            data.royalties
        }
        _ => 0,
    }
}

pub(crate) fn apply(f: &Facts, t: &mut Totals, r: &mut FederalResult) {
    if apply_checked(f, t, r).is_none() {
        let (key, _message) = crate::partnership::PartnerPlan::RANGE;
        crate::issue_localized!(
            r,
            &crate::interview::income_issue_context(f.entry),
            "K-1 income",
            key,
            _message
        );
    }
}

fn apply_checked(f: &Facts, t: &mut Totals, r: &mut FederalResult) -> Option<()> {
    let flows = &f.flows;
    let (regular, regular_investment) = capital(f, flows, &f.disposal)?;
    let (alternative, alternative_investment) = capital(f, &f.amt_flows, &f.amt_disposal)?;
    let events = checked_sum([f.disposal.distribution_gain, f.disposal.capital()])?;
    let amt_events = checked_sum([f.amt_disposal.distribution_gain, f.amt_disposal.capital()])?;
    // At most seven signed64 operands per result: keep integral cancellation
    // inside this owner, then check its native-cent output once.
    let mut net = i128::from(t.amt_k1_capital_net_adjustment);
    let mut gross = i128::from(t.amt_k1_capital_gross_adjustment)
        + i128::from(phase::floor(amt_events, 0))
        - i128::from(phase::floor(events, 0));
    for term in 0..2 {
        net += i128::from(alternative[term]) - i128::from(regular[term]);
        if !f.k3 {
            gross += i128::from(phase::floor(f.amt_flows.capital()[term], 0))
                - i128::from(phase::floor(flows.capital()[term], 0));
        }
    }
    let net = i64::try_from(net).ok()?;
    let gross = i64::try_from(gross).ok()?;
    t.amt_k1_capital_net_adjustment = net;
    t.amt_k1_capital_gross_adjustment = gross;
    let gross = royalties(f);
    let net = i64::try_from(i128::from(gross) - i128::from(flows.royalty_deductions)).ok()?;
    t.royalties += crate::kernel::ratio::exact::whole(net);
    t.k1_royalty_gross = checked_sum([t.k1_royalty_gross, gross])?;
    t.k1_royalty_deductions = checked_sum([t.k1_royalty_deductions, flows.royalty_deductions])?;
    t.k1_royalty_losses = checked_sum([t.k1_royalty_losses, phase::cap(net, [0])])?;
    t.amt_royalty_adjustment = i64::try_from(
        i128::from(t.amt_royalty_adjustment) + i128::from(flows.royalty_deductions)
            - i128::from(f.amt_flows.royalty_deductions),
    )
    .ok()?;
    t.other_8z = checked_sum([t.other_8z, flows.portfolio, flows.recovery])?;
    t.k1_line_7 = checked_sum([
        t.k1_line_7,
        flows.portfolio,
        flows.recovery,
        if f.nii_activity { flows.debt } else { 0 },
    ])?;
    t.k1_investment_income = checked_sum([t.k1_investment_income, flows.portfolio])?;
    t.personal_debt = checked_sum([t.personal_debt, flows.debt])?;
    t.gambling = checked_sum([t.gambling, flows.winnings])?;
    t.gambling_losses = checked_sum([t.gambling_losses, flows.gambling_losses])?;
    for class in 0..3 {
        t.k1_charity[class] = checked_sum([t.k1_charity[class], flows.charity[class]])?;
    }
    t.k1_charity_cash30 = checked_sum([t.k1_charity_cash30, flows.charity_cash30])?;
    let investment_interest = flows.investment_interest;
    t.k1_interest[0] = checked_sum([t.k1_interest[0], investment_interest])?;
    t.k1_interest[1] = checked_sum([t.k1_interest[1], f.amt_flows.investment_interest])?;
    let owner = usize::from(f.entry.owner == crate::DocumentOwner::Spouses);
    t.injured_niit_deduction_cents.k1_interest[owner] =
        checked_sum([t.injured_niit_deduction_cents.k1_interest[owner], investment_interest])?;
    t.early_penalty = checked_sum([t.early_penalty, flows.early_penalty])?;
    t.k1_deferred_tax = checked_sum([t.k1_deferred_tax, f.deferred_tax])?;
    t.reit_dividends = checked_sum([t.reit_dividends, f.reit_dividends])?;
    t.retain_reit_owner(f.entry, f.reit_dividends);
    // Form 1116 line 3e counts each gain whole: the box 11 (10) items and the owner's own
    // gain on the interest are separate contributions.
    let event_term = term_index(f.term);
    for term in 0..2 {
        add_capital(t, term, flows.checked_capital()?[term], f.k3)?;
        // PRINT trace follows the admitted K-1 code owner. Own-interest
        // disposition events below stay on Form8949/ScheduleD3 or10.
        t.schedule_d_k1[term] =
            checked_sum([t.schedule_d_k1[term], flows.checked_capital()?[term]])?;
        add_capital(t, term, if term == event_term { events } else { 0 }, false)?;
        t.investment_gains[term] =
            checked_sum([t.investment_gains[term], regular_investment[term]])?;
        t.amt_sale_adjustment[term] = i64::try_from(
            i128::from(t.amt_sale_adjustment[term]) + i128::from(alternative[term])
                - i128::from(regular[term]),
        )
        .ok()?;
        t.amt_investment_gain_adjustment[term] = i64::try_from(
            i128::from(t.amt_investment_gain_adjustment[term])
                + i128::from(alternative_investment[term])
                - i128::from(regular_investment[term]),
        )
        .ok()?;
    }
    let regular_ordinary = ordinary(f, flows, &f.disposal)?;
    let alternative_ordinary = ordinary(f, &f.amt_flows, &f.amt_disposal)?;
    let research = t.research_items.entry(f.entry.id).or_default();
    research.business_loss = checked_sum([
        research.business_loss,
        flows.ordinary.min(0).checked_neg()?,
        f.disposal.ordinary.min(0).checked_neg()?,
    ])?;
    let flow_capital = flows.checked_capital()?;
    research.capital_loss = checked_sum([
        research.capital_loss,
        flow_capital[0].min(0).checked_neg()?,
        flow_capital[1].min(0).checked_neg()?,
        events.min(0).checked_neg()?,
    ])?;
    research.income = checked_sum([
        research.income,
        net,
        flows.portfolio,
        flows.recovery,
        flows.debt,
        flows.winnings,
        regular[0],
        regular[1],
        regular_ordinary,
        flows.early_penalty.checked_neg()?,
    ])?;
    for (class, amount) in [2, 0, 3].into_iter().zip(flows.charity) {
        research.charity[class] = checked_sum([research.charity[class], amount])?;
    }
    research.investment_interest =
        checked_sum([research.investment_interest, flows.investment_interest])?;
    // The code R gain whole (not for a K-3 K-1) and the section 751 gain whole.
    if !f.k3 {
        t.ordinary_gain_gross =
            checked_sum([t.ordinary_gain_gross, phase::floor(flows.ordinary, 0)])?;
    }
    t.ordinary_gain_gross =
        checked_sum([t.ordinary_gain_gross, phase::floor(f.disposal.ordinary, 0)])?;
    if f.k3 {
        t.k3_agi_items = checked_sum([
            t.k3_agi_items,
            flows.portfolio,
            flows.recovery,
            flows.debt,
            flows.winnings,
        ])?;
    }
    t.k1_ordinary[0] = checked_sum([t.k1_ordinary[0], regular_ordinary])?;
    t.k1_ordinary[1] = checked_sum([t.k1_ordinary[1], alternative_ordinary])?;
    if f.investment_activity {
        t.k1_investment_ordinary[0] = checked_sum([t.k1_investment_ordinary[0], regular_ordinary])?;
        t.k1_investment_ordinary[1] =
            checked_sum([t.k1_investment_ordinary[1], alternative_ordinary])?;
    }
    if !f.nii_activity {
        t.k1_nii_excluded = i64::try_from(
            i128::from(t.k1_nii_excluded)
                - i128::from(flows.ordinary)
                - i128::from(flows.nonportfolio[0])
                - i128::from(flows.nonportfolio[1]),
        )
        .ok()?;
    }
    let disposal = &f.disposal;
    let term = term_index(f.term);
    // Form 8949 box C or F: the distribution gain and the capital part of
    // the sale (proceeds less the section 751 part, less the basis).
    let events = checked_sum([disposal.distribution_gain, disposal.capital()])?;
    if events != 0 || disposal.sold {
        // Schedule D line 3 or 10 and its columns (d) and (e): the distribution above basis is
        // proceeds with no basis, the sale its proceeds and basis.
        let (line, total) =
            if term == 0 { (3, &mut t.sale_line_3) } else { (7, &mut t.sale_line_10) };
        *total = checked_sum([*total, events])?;
        let sold = if disposal.sold { (disposal.proceeds, disposal.basis) } else { (0, 0) };
        t.add_columns(line, disposal.distribution_gain + sold.0, sold.1);
    }
    // Collectibles and unrecaptured section 1250 gain are parts of the
    // capital gain of a sale of an interest held more than one year (Reg.
    // 1.1(h)-1(b)(2)(ii) and (b)(3)(ii)); ignored when held one year or less.
    let sale_gain = disposal.capital();
    if term == 1 && sale_gain > 0 {
        let collectibles = phase::cap(f.collectibles, [sale_gain]);
        t.collectible_long = checked_sum([t.collectible_long, collectibles])?;
        t.unrecaptured_long = checked_sum([
            t.unrecaptured_long,
            phase::cap(f.unrecaptured, [sale_gain - collectibles]),
        ])?;
    }
    // Section 1411(c)(4): a gain on the interest of a business the owner
    // materially participates in, other than trading, is net investment
    // income only to the extent of the deemed-sale gain (Form 8960 line 5c).
    if !f.nii_activity {
        t.k1_business_gain = checked_sum([
            t.k1_business_gain,
            disposal.distribution_gain,
            phase::floor(disposal.sale_gain(), 0),
        ])?;
    }
    report(f, r);
    Some(())
}

fn report(f: &Facts, r: &mut FederalResult) {
    let id = f.entry.id;
    let prefix = f.prefix();
    let inputs: [&str; 1] = [if f.partner { "income.partnership" } else { "income.scorp" }];
    for (row, kind, cents) in &f.flows.rows {
        let Some((form, line, explain)) = kind.row(row) else {
            continue;
        };
        // The aggregate rows of Schedule 1, Schedule A, Schedule D and Form
        // 4952 carry the printed line (the explanation names it); a second
        // row of the same line would count it twice. Form 6781 line 1 has one
        // row for each source.
        push_line(
            r,
            (*kind == crate::k1_items::K1ItemKind::Section1256).then(|| FormLine::new(form, line)),
            &alloc::format!("{prefix}_{id}_item_{row}"),
            *cents,
            &inputs,
            f.source,
            Message::new(explain, alloc::vec![]),
        );
    }
    let gross = royalties(f);
    if gross != 0
        || f.flows
            .rows
            .iter()
            .any(|(_, kind, _)| *kind == crate::k1_items::K1ItemKind::RoyaltyDeductions)
    {
        for (amt, flows) in [(false, &f.flows), (true, &f.amt_flows)] {
            let amt_prefix = if amt { "amt_" } else { "" };
            emit_lines(
                r,
                &mut (),
                [
                    ("4", "line4_royalties", gross),
                    ("19", "line19_royalty_deductions", flows.royalty_deductions),
                    ("20", "line20_royalty_expenses", flows.royalty_deductions),
                    ("21", "line21_royalty_income", gross - flows.royalty_deductions),
                ],
                |(line, suffix, amount), _, _| {
                    Some(calculation_row(
                        (!amt).then(|| FormLine::new(TaxForm::ScheduleE, line)),
                        &alloc::format!("{amt_prefix}{prefix}_{id}_{suffix}"),
                        LineValue::MoneyCents(amount),
                        &inputs,
                        f.source,
                        Message::new("EXPLAIN_K1_ROYALTIES", alloc::vec![]),
                    ))
                },
            );
        }
    }
    let disposal = &f.disposal;
    let box_name = if term_index(f.term) == 0 { "C" } else { "F" };
    if disposal.distribution_gain != 0 && f.partner {
        push_line(
            r,
            Some(FormLine::new(TaxForm::Form8949, box_name)),
            &alloc::format!("{prefix}_{id}_distribution_gain"),
            disposal.distribution_gain,
            &inputs,
            "https://www.irs.gov/instructions/i1065sk1",
            Message::new("EXPLAIN_K1_DISTRIBUTION_GAIN", alloc::vec![]),
        );
    }
    // An S corporation's rows keep the names they had: the regular amount and
    // the AMT amount by holding period.
    if !f.partner {
        let term_name = if term_index(f.term) == 0 { "short" } else { "long" };
        for (suffix, amount) in
            [("gain", f.disposal.distribution_gain), ("amt_gain", f.amt_disposal.distribution_gain)]
        {
            if amount != 0 {
                push_line(
                    r,
                    Some(FormLine::new(TaxForm::Form8949, box_name)),
                    &alloc::format!("scorp_{id}_distribution_{term_name}_{suffix}"),
                    amount,
                    &["income.scorp.distributions", "income.scorp.amt"],
                    "https://www.irs.gov/instructions/i7203",
                    Message::new("EXPLAIN_SCORP_N_DISTRIBUTION_N_N", alloc::vec![]),
                );
            }
        }
    }
    if disposal.sold {
        if disposal.ordinary != 0 {
            push_line(
                r,
                None,
                &alloc::format!("{prefix}_{id}_sale_ordinary"),
                disposal.ordinary,
                &inputs,
                "https://www.irs.gov/publications/p541",
                Message::new("EXPLAIN_K1_SALE_ORDINARY", alloc::vec![]),
            );
        }
        push_line(
            r,
            Some(FormLine::new(TaxForm::Form8949, box_name)),
            &alloc::format!("{prefix}_{id}_sale_gain"),
            disposal.capital(),
            &inputs,
            "https://www.irs.gov/publications/p541",
            Message::new(
                "EXPLAIN_K1_SALE_GAIN",
                alloc::vec![
                    MessageArg::usd_cents(disposal.proceeds),
                    MessageArg::usd_cents(disposal.basis),
                ],
            ),
        );
    }
    // The holding period of the gain is the person's own answer; an unanswered
    // one is taken as one year or less, which never taxes a gain less.
    if f.term.is_none() && (disposal.distribution_gain != 0 || disposal.sold) {
        crate::federal_rules::note_once_scoped(
            r,
            Message::new(
                "WARN_K1_TERM_ASSUMED",
                alloc::vec![crate::interview::income_display_arg(f.entry)],
            ),
            crate::interview::income_issue_context(f.entry),
        );
    }
}

#[cfg(test)]
pub(crate) const DISPOSITION_OTHER: &str = crate::fixture_english!(
    "A part sale, gift, transfer at death, sale to a related person, installment sale, liquidation with property, redemption, carried interest (section 1061) or interest held partly one year or less is not calculated."
);

// The refusals of the codes and the disposition that decide on a K-1 alone:
// a disposition other than a sale or a liquidation for cash, and investment
// interest (box 13 code H) with a loss the at-risk limit holds back, which
// needs Form 4952 and Form 6198 each after the other (i6198 line 4).
pub(crate) const DISPOSITION_NEEDED: (&str, &str) = (
    "DOCUMENT_K1_DISPOSITION",
    crate::fixture_english!("Answer what happened to this interest during the year."),
);

pub(crate) fn refusals(
    entry: &FederalIncomeEntry,
    items: &crate::k1_items::K1Items,
    disposition: &Option<crate::K1Disposition>,
    ledgers: [Option<&crate::k1_loss::LossLedger>; 2],
    r: &mut FederalResult,
) {
    let context = crate::interview::income_issue_context(entry);
    if disposition.is_none() {
        crate::issue_localized!(
            r,
            &context,
            "K-1 disposition",
            "DOCUMENT_K1_DISPOSITION",
            DISPOSITION_NEEDED.1
        );
    }
    if *disposition == Some(crate::K1Disposition::Other) {
        crate::issue_localized!(
            r,
            &context,
            "K-1 disposition",
            "ISSUE_K1_DISPOSITION_OTHER",
            DISPOSITION_OTHER,
        );
    }
    if items.investment_interest() > 0
        && ledgers.into_iter().flatten().any(|ledger| ledger.risk_suspended.any())
    {
        let message = Message::new(
            "ISSUE_K1_INVESTMENT_INTEREST_AT_RISK",
            alloc::vec![crate::interview::income_display_arg(entry)],
        );
        crate::federal_rules::issue_message(
            r,
            &context,
            #[cfg(test)]
            "K-1 investment interest",
            #[cfg(test)]
            message.english(),
            message,
        );
    }
}

// The gain above the basis (a distribution or the sale) is taken as not
// passive, which never lowers tax; the entity's allocation among its activities
// may make part of it passive income that frees another passive activity's
// loss (Reg. 1.469-2T(e)(3); Pub 925). The estimate says so only when another
// passive activity of the return has an unallowed loss this year.
pub(crate) fn gain_not_passive(
    d: &crate::FederalDraft,
    rental: &crate::extended_rules::RentalResult,
    r: &mut FederalResult,
) {
    for entry in &d.income {
        let (gain, passive, own) = match &entry.income {
            crate::FederalIncome::Partnership { data } => {
                let (_, disposal) = match data.outcome(&data.regular) {
                    Ok(outcome) => outcome,
                    Err((key, _message)) => {
                        crate::issue_localized!(
                            r,
                            &crate::interview::income_issue_context(entry),
                            "K-1 income",
                            key,
                            _message
                        );
                        continue;
                    }
                };
                (
                    disposal.activity_gain(),
                    data.items_in_pool()
                        || data.material_for_losses() != crate::participation::Participation::Yes,
                    rental
                        .partners
                        .iter()
                        .filter(|v| v.id == entry.id)
                        .map(|v| v.suspended.get())
                        .sum::<i64>(),
                )
            }
            crate::FederalIncome::Scorp { data } => {
                let data = &crate::ScorpK1::from(data);
                let (_, disposal) = data.outcome();
                (
                    disposal.activity_gain(),
                    data.nonpassive() != crate::participation::Participation::Yes,
                    rental
                        .scorp_suspensions
                        .iter()
                        .filter(|(id, _)| *id == entry.id)
                        .map(|(_, suspended)| suspended.get())
                        .sum::<i64>(),
                )
            }
            _ => continue,
        };
        if gain > 0 && passive && rental.passive_loss - crate::kernel::ratio::dollars(own) > 0 {
            crate::federal_rules::note_once_scoped(
                r,
                Message::new(
                    "WARN_K1_GAIN_NOT_PASSIVE",
                    alloc::vec![crate::interview::income_display_arg(entry)],
                ),
                crate::interview::income_issue_context(entry),
            );
        }
    }
}
