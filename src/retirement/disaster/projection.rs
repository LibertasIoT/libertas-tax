// Form8915-F PRINT: project the existing designated/repayment/spread owners.
// Tax-year2026 draft PartI uses5a(total),5b(nonqualified),5c(qualified).
// The earlier source IDs remain untagged; their combined IRA taxable row is
// not actual line20. This projection splits18/19/20 and preserves their21 sum.
use super::Form;
use crate::kernel::lines::{LineValue, emit as emit_lines, row};
use crate::{FederalDraft, FederalIncome, FederalIncomeEntry, FederalResult, FormLine, TaxForm};
use alloc::collections::BTreeMap;

#[derive(Default)]
pub(super) struct Projection {
    // Plan, traditional IRA, Roth IRA gross designated amounts.
    amount: [i64; 3],
    traditional_basis: i64,
    roth_basis: i64,
    has_8606: bool,
    pub(super) disasters: BTreeMap<u32, (i64, [i64; 3])>,
    home: [i64; 3],
    home_remaining: [i64; 3],
    home_cost: i64,
    home_repaid: i64,
    pub(super) home_8606: bool,
    home_sources: BTreeMap<i64, (usize, i64, i64, i64)>,
}
fn category(entry: &FederalIncomeEntry) -> usize {
    match &entry.income {
        FederalIncome::Roth { .. } => 2,
        FederalIncome::Retirement { data } if data.kind.ira() => 1,
        _ => 0,
    }
}
impl Projection {
    pub(super) fn record(
        &mut self,
        d: &FederalDraft,
        r: &FederalResult,
        entry: &FederalIncomeEntry,
        amount: i64,
        taxable: i64,
    ) {
        let disaster = match &entry.income {
            FederalIncome::Roth { data } => &data.disaster,
            FederalIncome::Retirement { data } => &data.disaster,
            _ => unreachable!(),
        };
        let crate::DisasterDistribution::Qualified { fema, earlier, .. } = disaster else {
            unreachable!();
        };
        let category = category(entry);
        self.amount[category] += amount;
        self.disasters.entry(*fema).or_insert((*earlier, [0; 3])).1[category] += amount;
        if category == 0 {
            return;
        }
        let required = match crate::inherited_ira::link_of(d, entry) {
            Some(crate::inherited_ira::Link::Record(group)) => {
                let kind = if category == 2 { "roth" } else { "ira" };
                let first = if category == 2 { "19" } else { "15a" };
                r.lines.iter().any(|line| {
                    line.id == alloc::format!("inherited_{kind}_{group}_form_line{first}")
                })
            }
            Some(_) => false,
            None => {
                let who = if entry.owner == crate::Owner::Spouse { "spouse" } else { "taxpayer" };
                let kind = if category == 2 { "roth" } else { "ira" };
                let first = if category == 2 { "19" } else { "15a" };
                let basis_method = category == 2
                    || matches!(&entry.income, FederalIncome::Retirement { data } if data.taxable == crate::RetirementTaxable::IraBasis);
                basis_method
                    && r.lines
                        .iter()
                        .any(|line| line.id == alloc::format!("{who}_{kind}_form_line{first}"))
            }
        };
        self.has_8606 |= required;
        if required {
            if category == 2 {
                self.roth_basis += taxable;
            } else {
                self.traditional_basis += taxable;
            }
        }
    }
    pub(super) fn home(&mut self, entry: &FederalIncomeEntry, gross: i64, cost: i64, repaid: i64) {
        self.home_sources.insert(entry.id, (category(entry), gross, cost, repaid));
        self.home[category(entry)] += gross;
        self.home_cost += cost;
        self.home_repaid += repaid;
        self.home_remaining[category(entry)] += crate::kernel::carry::native_remaining::<
            crate::kernel::carry::Cents,
        >(gross, cost + repaid)
        .get();
    }
}

// The accepted1099-R gross sources, once each. An inherited account groups
// its linked payments; counting that account here would count them twice.
pub(super) fn available(d: &FederalDraft) -> [[i64; 3]; 2] {
    let mut total = [[0; 3]; 2];
    for entry in &d.income {
        let gross = match &entry.income {
            FederalIncome::Retirement { data } => data.gross,
            FederalIncome::Roth { data } => data.gross,
            _ => continue,
        };
        total[usize::from(entry.owner == crate::Owner::Spouse)][category(entry)] += gross;
    }
    total
}

pub(super) fn emit(
    r: &mut FederalResult,
    owner: usize,
    year: u32,
    form: &Form,
    available: &mut [i64; 3],
    later: bool,
) {
    let who = if owner == 0 { "taxpayer" } else { "spouse" };
    let prefix = alloc::format!("form_8915_{who}_{year}_");
    let mut values: alloc::vec::Vec<(alloc::string::String, alloc::string::String, LineValue)> = alloc::vec![
        ("A".into(), "A".into(), LineValue::Year(2026)),
        ("B".into(), "B".into(), LineValue::Year(i64::from(year))),
    ];
    let mut money = |number: &str, amount| {
        values.push((number.into(), number.into(), LineValue::MoneyCents(amount)))
    };
    let p = &form.projection;
    if !p.disasters.is_empty() {
        // A current-year disaster cannot have appeared on a prior-year form.
        // A positive accepted prior amount also establishes a prior claim.
        // Other cases use the expressly permitted Worksheet1B, preserving the
        // actual per-FEMA allocation without guessing whether a zero prior
        // amount meant the disaster appeared on a prior form's itemC.
        if p.disasters.len() == 1 && (year == 2026 || p.disasters.values().next().unwrap().0 > 0) {
            let earlier = p.disasters.values().next().unwrap().0;
            if earlier > 0 {
                money("1a", super::LIMIT);
                money("1b", earlier);
                money(
                    "1c",
                    crate::kernel::carry::native_remaining::<crate::kernel::carry::Cents>(
                        super::LIMIT,
                        earlier,
                    )
                    .get(),
                );
                money("1d", 0);
            }
            money(
                "1e",
                crate::kernel::carry::native_remaining::<crate::kernel::carry::Cents>(
                    super::LIMIT,
                    earlier,
                )
                .get(),
            );
        }
        for (i, number) in ["2", "3", "4"].iter().enumerate() {
            money(&alloc::format!("{number}(a)"), available[i]);
            money(&alloc::format!("{number}(b)"), p.amount[i]);
        }
        let gross: i64 = available.iter().sum();
        let qualified: i64 = p.amount.iter().sum();
        money("5a", gross);
        money("5b", gross - qualified);
        money("5c(a)", qualified);
        money("5c(b)", qualified);
        money("6", qualified);
        // Oldest to newest forms carry the unused gross to the next PartI.
        // Only the final PartI reports that remaining gross; PartIV's actual
        // non-8606 qualified gross is separately excluded from this line7.
        money(
            "7",
            if later {
                0
            } else {
                crate::kernel::carry::native_remaining::<crate::kernel::carry::Cents>(
                    gross,
                    qualified + p.home.iter().sum::<i64>(),
                )
                .get()
            },
        );
    }
    // Consume the unchanged native results, including death acceleration and
    // prior-form repayment carries, rather than recalculating their amounts.
    let source = |number: &str| {
        r.lines
            .iter()
            .find(|line| line.id == alloc::format!("{prefix}line_{number}"))
            .map_or(0, |line| line.amount)
    };
    if form.plan_amount > 0 {
        for number in ["8", "9", "10", "11"] {
            money(number, source(number));
        }
    }
    if form.plan_amount != 0
        || form.plan_carried != 0
        || form.plan_repaid != 0
        || form.plan_excess_in != 0
    {
        for number in ["12", "13", "14", "15"] {
            money(number, source(number));
        }
    }
    if form.ira_amount > 0 {
        if p.has_8606 {
            money("18", p.traditional_basis);
            money("19", p.roth_basis);
        }
        money("20", form.ira_taxable - p.traditional_basis - p.roth_basis);
        money("21", form.ira_taxable);
        money("22", source("22"));
    }
    if form.ira_amount != 0
        || form.ira_carried != 0
        || form.ira_repaid != 0
        || form.ira_excess_in != 0
    {
        for number in ["23", "24", "25", "26"] {
            money(number, source(number));
        }
    }
    let terminal_plan = source("13") - source("11") - source("12");
    let terminal_ira = source("24") - source("22") - source("23");
    let home_gross: i64 = p.home.iter().sum();
    if home_gross > 0 {
        money("28", home_gross);
        money("29", p.home_cost);
        money(
            "30",
            crate::kernel::carry::native_remaining::<crate::kernel::carry::Cents>(
                home_gross,
                p.home_cost,
            )
            .get(),
        );
        money("31", p.home_repaid);
        money("32", if later { 0 } else { p.home_remaining.iter().sum() });
    }
    emit_lines(r, &mut (), values, |(suffix, number, value), _, _| {
        Some(row(
            Some(FormLine::new(TaxForm::Form8915F, &number)),
            &alloc::format!("{prefix}print_{suffix}"),
            value,
            &["income.retirement.disaster", "screening.disaster_carries"],
            "https://www.irs.gov/pub/irs-dft/f8915f--dft.pdf",
            msg!("EXPLAIN_FORM8915_PROJECTED_LINE"),
        ))
    });
    // The death instruction adds the remaining future installment directly
    // to13/24. Keep12/23 as their actual prior-form worksheets, with the
    // extra terminal-return amount shown once as a supporting component.
    emit_lines(
        r,
        &mut (),
        [("plan", "13", terminal_plan), ("ira", "24", terminal_ira)],
        |(kind, number, amount), _, _| {
            if amount == 0 {
                return None;
            }
            Some(row(
                Some(FormLine::subrow(TaxForm::Form8915F, number)),
                &alloc::format!("{prefix}terminal_{kind}"),
                LineValue::MoneyCents(amount),
                &["screening.disaster_carries", "people"],
                "https://www.irs.gov/instructions/i8915f",
                msg!("EXPLAIN_FORM8915_TERMINAL_REMAINDER"),
            ))
        },
    );
    let ira_section = form.ira_amount != 0
        || form.ira_carried != 0
        || form.ira_repaid != 0
        || form.ira_excess_in != 0;
    let plan_section = form.plan_amount != 0
        || form.plan_carried != 0
        || form.plan_repaid != 0
        || form.plan_excess_in != 0;
    let choices = [
        ("8", false, plan_section && form.plan_amount == 0),
        ("16", form.ira_amount > 0, ira_section),
        ("17", p.has_8606, form.ira_amount > 0),
        ("27", p.home_8606, home_gross > 0 || p.home_8606),
    ];
    emit_lines(r, &mut (), choices, |(number, answer, applies), _, _| {
        if !applies {
            return None;
        }
        Some(row(
            Some(FormLine::new(TaxForm::Form8915F, number)),
            &alloc::format!("{prefix}choice_{number}_{}", if answer { "yes" } else { "no" }),
            LineValue::Text,
            &["income.retirement.disaster"],
            "https://www.irs.gov/pub/irs-dft/f8915f--dft.pdf",
            msg!("EXPLAIN_FORM8915_PROJECTED_LINE"),
        ))
    });
    emit_lines(
        r,
        &mut (),
        [("11", form.plan_amount > 0), ("22", form.ira_amount > 0)],
        |(number, applies), _, _| {
            if !applies {
                return None;
            }
            let election = if form.spread == Some(crate::DisasterSpread::AllInReturnYear) {
                "all"
            } else {
                "spread"
            };
            Some(row(
                Some(FormLine::subrow(TaxForm::Form8915F, number)),
                &alloc::format!("{prefix}election_{number}_{election}"),
                LineValue::Text,
                &["income.retirement.disaster"],
                "https://www.irs.gov/instructions/i8915f",
                msg!("EXPLAIN_FORM8915_ELECTION"),
            ))
        },
    );
    let worksheet = !p.disasters.is_empty()
        && !(p.disasters.len() == 1
            && (year == 2026 || p.disasters.values().next().unwrap().0 > 0));
    emit_lines(r, &mut (), [worksheet], |used, _, _| {
        if !used {
            return None;
        }
        Some(row(
            Some(FormLine::subrow(TaxForm::Form8915F, "1")),
            &alloc::format!("{prefix}choice_1_worksheet"),
            LineValue::Text,
            &["income.retirement.disaster"],
            "https://www.irs.gov/instructions/i8915f",
            msg!("EXPLAIN_FORM8915_DISASTER_LIMIT"),
        ))
    });
    // Supporting per-FEMA cap arithmetic, including Worksheet1B when used.
    // These rows never act as extra printed totals.
    for (fema, (prior, amounts)) in &p.disasters {
        emit_lines(
            r,
            &mut (),
            [
                ("prior", *prior),
                (
                    "limit",
                    crate::kernel::carry::native_remaining::<crate::kernel::carry::Cents>(
                        super::LIMIT,
                        *prior,
                    )
                    .get(),
                ),
                ("qualified", amounts.iter().sum()),
            ],
            |(kind, amount), _, _| {
                Some(row(
                    Some(FormLine::subrow(TaxForm::Form8915F, "1")),
                    &alloc::format!("{prefix}worksheet_{kind}_{fema}"),
                    LineValue::MoneyCents(amount),
                    &["income.retirement.disaster"],
                    "https://www.irs.gov/instructions/i8915f",
                    msg!("EXPLAIN_FORM8915_DISASTER_LIMIT"),
                ))
            },
        );
    }
    // Per-document components retain category attribution through PartIV's
    // transfer to a later PartI; these are supporting rows, not extra totals.
    for (id, (category, gross, cost, repaid)) in &p.home_sources {
        let category = ["plan", "ira", "roth"][*category];
        emit_lines(
            r,
            &mut (),
            [("gross", "28", *gross), ("cost", "29", *cost), ("repaid", "31", *repaid)],
            |(kind, number, value), _, _| {
                Some(row(
                    Some(FormLine::subrow(TaxForm::Form8915F, number)),
                    &alloc::format!("{prefix}home_{category}_{id}_{kind}"),
                    LineValue::MoneyCents(value),
                    &["income.retirement.disaster"],
                    "https://www.irs.gov/instructions/i8915f",
                    msg!("EXPLAIN_FORM8915_PROJECTED_LINE"),
                ))
            },
        );
    }
    for (i, available) in available.iter_mut().enumerate() {
        *available = crate::kernel::carry::native_remaining::<crate::kernel::carry::Cents>(
            *available,
            p.amount[i] + p.home[i],
        )
        .get();
        if later {
            *available += p.home_remaining[i];
        }
    }
}
