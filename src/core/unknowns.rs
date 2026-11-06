// Answers and unknowns: the items a scope confirmation's No, or
// a kept Not sure that assumes the no-benefit path, leaves out of the
// estimate. `supported` returns a working copy of the return without them,
// or None when nothing is left out, and adds each item's notice once. The
// accepted return itself is never changed; rules that keep part of an item
// (a casualty's gain, a distribution without its exception) decide in place.
use crate::kernel::phase;
use crate::{
    BusinessActivity, FederalDraft, FederalIncome, IssueContext, Message, MessageArg, TaxSection,
};
use alloc::vec::Vec;

/// Existing direct-business risk admission, with expenses supplied by their
/// actual stage (before home use in source admission, after it in HOME).
pub(crate) fn business_loss_needs_risk_with_claims(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    id: i64,
    business: &crate::FederalBusiness,
    total_expense: i64,
) -> Result<bool, crate::Section179PreparedIssue> {
    let d = form_context.draft;
    let source_claims = form_context.claims;
    let participation = form_context.participation();
    Ok(!business.current_risk_supported(participation.get(id))
        && crate::federal_rules::farm_receipts_with_claims(d, id, business, source_claims)?
            < total_expense)
}

struct ScopeNotes<'a> {
    review: &'a mut Vec<crate::federal_model::ReviewNote>,
    financial_omission: bool,
    source_claims: &'a alloc::collections::BTreeMap<i64, i64>,
}

pub(crate) struct SupportedScope {
    pub working: Option<FederalDraft>,
    pub financial_source_scope_complete: bool,
}

fn information(
    notes: &mut ScopeNotes<'_>,
    key: &str,
    label: Option<MessageArg>,
    context: &IssueContext,
) {
    let context = match &label {
        Some(MessageArg::Text(name)) => context.clone().named(name),
        Some(MessageArg::Encoded(name)) if !name.is_empty() => context.clone().named_encoded(name),
        _ => context.clone(),
    };
    let message = Message::new(key, label.into_iter().collect());
    let note = crate::federal_model::ReviewNote::scoped(
        #[cfg(test)]
        message.english(),
        message,
        context,
    );
    if !notes.review.iter().any(|old| old.same_notice(&note)) {
        notes.review.push(note);
    }
}
fn information_args(
    notes: &mut ScopeNotes<'_>,
    key: &str,
    args: Vec<MessageArg>,
    context: &IssueContext,
) {
    let message = Message::new(key, args);
    let note = crate::federal_model::ReviewNote::scoped(
        #[cfg(test)]
        message.english(),
        message,
        context.clone(),
    );
    if !notes.review.iter().any(|old| old.same_notice(&note)) {
        notes.review.push(note);
    }
}
// An omission notice describes a financial source/benefit this estimate
// cannot establish. Informational notices use the same presentation without
// changing that outcome; neither message keys nor generic review-note presence
// determine financial availability.
fn notice(
    notes: &mut ScopeNotes<'_>,
    key: &str,
    label: Option<MessageArg>,
    context: &IssueContext,
) {
    notes.financial_omission = true;
    information(notes, key, label, context);
}
fn notice_args(
    notes: &mut ScopeNotes<'_>,
    key: &str,
    args: Vec<MessageArg>,
    context: &IssueContext,
) {
    notes.financial_omission = true;
    information_args(notes, key, args, context);
}

// Form 3468: an investment credit whose eligibility is not confirmed is left
// out with its basis reduction, so the property is depreciated on its full
// cost. A rehabilitation record of a property placed in service before 2026
// keeps its history (its basis reduction and any recapture) and loses only the
// 2026 share.
fn investment_credits(d: &mut FederalDraft, notes: &mut ScopeNotes<'_>) {
    use crate::InvestmentCredit;
    for entry in &mut d.income {
        let context = crate::interview::income_issue_context(entry);
        let label = || Some(MessageArg::Nested(alloc::boxed::Box::new(context.label())));
        match &mut entry.income {
            FederalIncome::Business { data } => {
                for asset in &mut data.assets {
                    match &asset.investment_credit {
                        InvestmentCredit::NewEnergy { confirmed: false, .. } => {
                            notice(notes, "WARN_ENERGY_CREDIT_LEFT_OUT", label(), &context);
                            asset.investment_credit = InvestmentCredit::No;
                        }
                        InvestmentCredit::Rehabilitation(claim) if !claim.confirmed => {
                            notice(notes, "WARN_REHABILITATION_CREDIT_LEFT_OUT", label(), &context);
                            // A building placed in service in 2026 has no
                            // credit determined: its record goes.
                            if asset.placed / 10000 >= 2026 {
                                asset.investment_credit = InvestmentCredit::No;
                            }
                        }
                        _ => {}
                    }
                }
            }
            FederalIncome::Rental { data } => {
                for building in &mut data.buildings {
                    if let Some(claim) = &building.rehabilitation
                        && !claim.confirmed
                    {
                        notice(notes, "WARN_REHABILITATION_CREDIT_LEFT_OUT", label(), &context);
                        if building.in_service / 10000 >= 2026 {
                            building.rehabilitation = None;
                        }
                    }
                }
            }
            FederalIncome::Partnership { data } => {
                k1_investment_credits(
                    &mut data.investment_property,
                    data.investment_confirmed,
                    notes,
                    label(),
                    &context,
                );
                if data.investment_property.is_empty() {
                    data.investment_confirmed = None;
                }
            }
            FederalIncome::Scorp { data } => {
                let mut properties = crate::ScorpK1::from(&*data).investment_property;
                k1_investment_credits(
                    &mut properties,
                    data.personal
                        .investment_confirmed
                        .map(crate::K1InvestmentConfirmation::confirmed),
                    notes,
                    label(),
                    &context,
                );
                data.statements
                    .investment_property
                    .retain(|property| properties.iter().any(|kept| kept.id() == property.id()));
                data.personal
                    .investment_forms
                    .retain(|answer| properties.iter().any(|kept| kept.id() == answer.id));
                if properties.is_empty() {
                    data.personal.investment_confirmed = None;
                }
            }
            FederalIncome::PartnershipActivities { data } => {
                k1_investment_credits(
                    &mut data.entity.investment_property,
                    data.entity.investment_confirmed,
                    notes,
                    label(),
                    &context,
                );
                if let crate::K1ComponentStatement::Entered { source } = &mut data.activities
                    && let Some(rows) = &mut source.investment_property_sources
                {
                    rows.retain(|row| {
                        data.entity.investment_property.iter().any(|p| p.id() == row.property)
                    });
                }
                if data.entity.investment_property.is_empty() {
                    data.entity.investment_confirmed = None;
                }
            }
            FederalIncome::ScorpActivities { data } => {
                k1_investment_credits(
                    &mut data.entity.investment_property,
                    data.entity.investment_confirmed,
                    notes,
                    label(),
                    &context,
                );
                if let crate::K1ComponentStatement::Entered { source } = &mut data.activities
                    && let Some(rows) = &mut source.investment_property_sources
                {
                    rows.retain(|row| {
                        data.entity.investment_property.iter().any(|p| p.id() == row.property)
                    });
                }
                if data.entity.investment_property.is_empty() {
                    data.entity.investment_confirmed = None;
                }
            }
            _ => {}
        }
    }
}
fn k1_investment_credits(
    items: &mut Vec<crate::K1InvestmentProperty>,
    confirmed: Option<bool>,
    notes: &mut ScopeNotes<'_>,
    label: Option<MessageArg>,
    context: &IssueContext,
) {
    let mut automatic_left_out = false;
    let mut own_left_out = false;
    items.retain(|item| {
        if let Some(own) = item.own_form() {
            own_left_out |= !own.reduced;
            own.reduced
        } else {
            automatic_left_out |= confirmed == Some(false);
            confirmed != Some(false)
        }
    });
    if automatic_left_out {
        notice(notes, "WARN_K1_INVESTMENT_CREDIT_LEFT_OUT", label, context);
    }
    if own_left_out {
        notice(
            notes,
            "WARN_ENTERED_BUSINESS_CREDIT_LEFT_OUT",
            Some(MessageArg::Resource("FORM_FORM3468".into())),
            context,
        );
    }
}

fn retain_scoped<T>(
    values: &mut Vec<T>,
    parent: &IssueContext,
    collection: &'static str,
    mut keep: impl FnMut(&mut T, &IssueContext) -> bool,
) {
    let mut position = 0;
    values.retain_mut(|value| {
        let context = parent.item(collection, position);
        position += 1;
        keep(value, &context)
    });
}
// The insured person of a long-term-care premium, as the people page labels
// them.
fn insured_arg(people: &[crate::PersonChoice], person: i64) -> MessageArg {
    MessageArg::Encoded(insured_label(people, person))
}
fn insured_label(people: &[crate::PersonChoice], person: i64) -> Vec<u8> {
    people.iter().find(|p| p.id == person).map(|p| p.label.clone()).unwrap_or_default()
}

fn display_label(context: &IssueContext, name: &str) -> MessageArg {
    if name.trim().is_empty() {
        MessageArg::Nested(alloc::boxed::Box::new(context.label()))
    } else {
        MessageArg::Text(name.into())
    }
}

pub(crate) fn supported(
    d: &FederalDraft,
    notes: &mut Vec<crate::federal_model::ReviewNote>,
) -> Option<FederalDraft> {
    let claims = crate::related_forms::effective_claimants(d);
    supported_with_scope(crate::related_forms::RelatedFormContext::original(d, &claims), notes)
        .working
}

pub(crate) fn supported_with_scope(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    review: &mut Vec<crate::federal_model::ReviewNote>,
) -> SupportedScope {
    let d = form_context.draft;
    let participation = form_context.participation();
    let macrs = form_context.macrs();
    let source_claims = form_context.claims;
    let notes = &mut ScopeNotes { review, financial_omission: false, source_claims };
    // Check the accepted source graph before optional facts are omitted; a
    // removed incoming contribution/conversion must not manufacture support.
    let mut earlier_roth: Vec<_> = [crate::Owner::Taxpayer, crate::Owner::Spouse]
        .into_iter()
        .flat_map(|owner| {
            crate::roth::earlier_funding(d, owner)
                .into_iter()
                .flat_map(move |ordered| ordered.basis.into_keys().map(move |id| (owner, id)))
        })
        .collect();
    for entry in &d.income {
        if matches!(&entry.income, FederalIncome::InheritedIra { data } if data.roth())
            && let Some(ordered) = crate::roth::inherited_earlier_funding(d, entry.id)
        {
            earlier_roth.extend(ordered.basis.into_keys().map(|id| (entry.owner.owned(), id)));
        }
    }
    let mut working = d.clone();
    crate::savings_bonds::apply_redemptions(&mut working.income);
    for entry in &mut working.income {
        if matches!(entry.income, FederalIncome::EsppSaleSource { .. })
            && let Ok(data) = crate::espp_return::resolve(d, entry)
        {
            entry.income = FederalIncome::EsppSale { data };
        } else if matches!(entry.income, FederalIncome::IsoSaleSource { .. })
            && let Ok(data) = crate::iso_return::resolve(d, entry)
        {
            entry.income = FederalIncome::IsoSale { data };
        } else if matches!(entry.income, FederalIncome::NuaSaleSource { .. })
            && let Ok(data) = crate::nua_sale::resolve(d, entry)
        {
            entry.income = FederalIncome::NuaSale { data };
        }
        // A Form 1099-NEC answered as employee pay is figured as Form 8919 pay.
        if let FederalIncome::Nec1099 { data } = &entry.income
            && let Some(data) = data.employee_wages()
        {
            entry.income = FederalIncome::EmployeeWages { data };
        }
    }
    name_destinations(&mut working);
    screening(&mut working, notes);
    income(&mut working, notes);
    broker_sources(&working, notes);
    farm_weather_classes(&mut working, notes);
    farm_named_benefits(&mut working, notes);
    estate_loss_sources(&mut working, notes);
    notes.financial_omission |= crate::after_death::apply(&mut working, notes.review);
    notes.financial_omission |= crate::investment_statements::apply(&mut working, notes.review);
    community_items(&mut working, notes);
    if matches!(
        working.adjustments.as_ref().map(|a| &a.social_election),
        Some(crate::SocialElection::Compare { complete: true, .. })
    ) && let Some(id) = crate::social_documents::unallocated_railroad_years(&working)
    {
        let entry = working.income.iter().find(|entry| entry.id == id).unwrap();
        notice(
            notes,
            "RRB_EARLIER_YEAR_ALLOCATION_NEEDED",
            None,
            &crate::interview::income_issue_context(entry),
        );
        working.adjustments.as_mut().unwrap().social_election = crate::SocialElection::Regular;
    }
    adjustments(&mut working, notes, &earlier_roth);
    deductions(&mut working, notes);
    credits(&mut working, notes);
    group4(&mut working, notes);
    inherited_links(&working, notes);
    lump_sums(&mut working, notes);
    special_code_notes(&working, notes);
    group1a(&mut working, notes, &participation, &macrs);
    group1b(&mut working, notes);
    k1_items_left_out(&mut working, notes);
    foreign_claims(&mut working, notes);
    ltc_links(&working, notes);
    method_changes(&mut working, notes);
    business_sales(&mut working, notes);
    macrs_source_benefits(&working, &macrs, notes);
    destinations_left_out(d, &mut working, notes);
    notes.financial_omission |= crate::statutory_wages::omitted(d, &mut working, notes.review);
    crate::statutory_wages::route_with_claims(&mut working, notes.source_claims);
    crate::information_returns::route(&mut working, notes.source_claims);
    SupportedScope {
        working: (working != *d).then_some(working),
        financial_source_scope_complete: !notes.financial_omission,
    }
}

// Recovery owners omit only a current deduction whose source is unavailable.
// Keep the actual asset, elections and history for class-wide rules, source
// associations, and mandatory disposition/recapture checks.
fn macrs_source_benefits(
    d: &FederalDraft,
    source_view: &crate::macrs_source::MacrsElectionView,
    notes: &mut ScopeNotes<'_>,
) {
    fn asset_note(
        asset: &crate::BusinessAsset,
        context: IssueContext,
        invocation: crate::macrs_source::AdsInvocation<'_>,
        notes: &mut ScopeNotes<'_>,
    ) {
        // Invalid entered facts belong to Save validation, not an assumption.
        if asset.macrs_source_fact_issue().is_some() {
            return;
        }
        if let Some(reason) = asset.macrs_source_issue_in_context(invocation) {
            let label = display_label(&context, &asset.description);
            notice_args(
                notes,
                "WARN_ASSET_MACRS_RECOVERY_LEFT_OUT",
                alloc::vec![label, MessageArg::Resource(reason.into())],
                &context,
            );
        }
    }
    for entry in &d.income {
        let Some(assets) = crate::nonmacrs::recovery_assets(&entry.income) else {
            continue;
        };
        let context = crate::interview::income_issue_context(entry);
        for (index, asset) in assets.iter().enumerate() {
            let invocation = source_view.context(entry.owner.owned(), Some(entry.id), asset);
            asset_note(asset, context.item("assets", index), invocation, notes);
        }
    }
    for (index, claim) in d.adjustments.iter().flat_map(|a| &a.employee_expenses).enumerate() {
        let context = claim.context(index);
        for (field, assets) in [
            ("vehicle_assets", &claim.vehicle_assets),
            ("equipment_assets", &claim.equipment_assets),
        ] {
            for (asset_index, asset) in assets.iter().enumerate() {
                let invocation = source_view.context(claim.owner, None, asset);
                asset_note(asset, context.item(field, asset_index), invocation, notes);
            }
        }
    }
}

// A class spanning independently entered farms needs one taxpayer-wide class
// statement and attributable source income. Do not repeat separate elections.
// This baseline retains current known income and omits only that benefit.
fn farm_weather_classes(d: &mut FederalDraft, notes: &mut ScopeNotes<'_>) {
    let classes: Vec<_> = crate::farm::sources(d)
        .flat_map(|source| {
            source
                .farm
                .specials
                .weather_sales
                .iter()
                .map(|sale| (source.entry.id, sale.class.clone()))
        })
        .collect();
    for entry in &mut d.income {
        let context = crate::interview::income_issue_context(entry);
        let farm = match &mut entry.income {
            FederalIncome::Business { data } => match &mut data.activity {
                BusinessActivity::Farming { data } => Some(data),
                _ => None,
            },
            FederalIncome::FarmRental { data } => Some(&mut data.farm),
            _ => None,
        };
        if let Some(farm) = farm {
            for sale in &mut farm.specials.weather_sales {
                if sale.elect
                    && classes.iter().any(|(id, class)| *id != entry.id && *class == sale.class)
                {
                    sale.elect = false;
                    notice(notes, "WARN_FARM_WEATHER_SHARED_CLASS", None, &context);
                }
            }
        }
    }
}

// Form 8594: a whole-business sale whose allocation includes purchased
// section 197 intangibles, an installment sale of these amounts or an
// allocation the buyer reports differently is not calculated; its record is
// left out with one notice (the assets' own rows stay).
fn business_sales(d: &mut FederalDraft, notes: &mut ScopeNotes<'_>) {
    for entry in &mut d.income {
        let context = crate::interview::income_issue_context(entry);
        let FederalIncome::Business { data } = &mut entry.income else {
            continue;
        };
        if data.sale.as_ref().is_some_and(|sale| sale.special) {
            data.sale = None;
            notice(
                notes,
                "WARN_BUSINESS_SALE_LEFT_OUT",
                Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                &context,
            );
        }
    }
}

// Each Form 1099 amount that belongs to a business, rental, royalty, equipment
// or hobby record names its record before anything is left out (the form's
// choice, or the owner's only such record), so a record the estimate leaves out
// takes the amounts routed to it. A destination no record can receive (none,
// or several and none chosen) stays unnamed; the estimate raises it.
fn name_destinations(d: &mut FederalDraft) {
    for index in 0..d.income.len() {
        let owner = d.income[index].owner;
        for flow in crate::information_returns::flows(&d.income[index].income) {
            if flow.choice.is_some() || flow.kind.is_none() {
                continue;
            }
            if let crate::information_returns::Lookup::Found(id) =
                crate::information_returns::lookup(
                    d,
                    owner.owned(),
                    flow.kind.expect("explicit destination kind"),
                    None,
                )
            {
                crate::information_returns::name_record(&mut d.income[index].income, flow.slot, id);
            }
        }
    }
}

// The amounts of a form whose destination was left out leave with it, one
// notice for the form; a form with no amount left is removed with its
// withholding (as a farm's cooperative statements are).
fn destinations_left_out(
    original: &FederalDraft,
    d: &mut FederalDraft,
    notes: &mut ScopeNotes<'_>,
) {
    use crate::information_returns::{candidates, clear, flows, nothing_left};
    let claims = notes.source_claims;
    let mut remove = Vec::new();
    for index in 0..d.income.len() {
        let owner = d.income[index].owner;
        let mut dropped = false;
        for flow in flows(&d.income[index].income) {
            let (kind, id) = if let Some(kind) = flow.kind {
                let Some(id) = flow.choice else { continue };
                (kind, id)
            } else {
                let Some(target) = crate::information_returns::resolve_flow(
                    original,
                    &d.income[index],
                    &flow,
                    claims,
                ) else {
                    continue;
                };
                target
            };
            if candidates(d, owner.owned(), kind).contains(&id)
                || !candidates(original, owner.owned(), kind).contains(&id)
            {
                continue;
            }
            clear(&mut d.income[index].income, flow.slot);
            dropped = true;
        }
        if dropped {
            let context = crate::interview::income_issue_context(&d.income[index]);
            notice(
                notes,
                "WARN_FORM_1099_DESTINATION_LEFT_OUT",
                Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                &context,
            );
            if nothing_left(&d.income[index].income) {
                remove.push(d.income[index].id);
            }
        }
    }
    d.income.retain(|entry| !remove.contains(&entry.id));
}

// Form 8958: an item of a kind whose amounts are figured from its own record
// (rentals, pass-through K-1s) that belongs wholly to the spouse is not on this
// separate return; it is left out of the working copy with a notice.
fn community_items(d: &mut FederalDraft, notes: &mut ScopeNotes<'_>) {
    let Some(setup) = d.setup.clone() else {
        return;
    };
    d.income.retain(|entry| {
        let gone = crate::community::self_divided(&entry.income)
            && crate::community::share(&setup, entry.owner.owned(), entry.community, &entry.income)
                .is_some_and(|share| share.none());
        if gone {
            let context = crate::interview::income_issue_context(entry);
            information_args(
                notes,
                "NOTE_COMMUNITY_ITEM_IS_THE_SPOUSES",
                alloc::vec![MessageArg::Nested(alloc::boxed::Box::new(context.label()))],
                &context,
            );
        }
        !gone
    });
}

// Section 481(a) adjustments of a business, farm or rental. A record whose
// AMT amount or consent terms the App does not calculate (`confirmed` No), and
// every record of a rental with personal use (the Worksheet 5-1 allocation
// is not designed for it), is left out with its notice: `confirmed` is
// cleared so the calculation skips it while the page's record positions
// stay. The rest of a multi-year increase is taken in 2026 when the activity
// stops (Rev. Proc. 2015-13 section 7.03(4)(a)): a rental sold in 2026 and an
// owner who died in 2026, both facts the return already holds.
fn method_changes(d: &mut FederalDraft, notes: &mut ScopeNotes<'_>) {
    let died: Vec<i64> = d
        .income
        .iter()
        .filter(|entry| crate::farm::owner_death_date(d, entry.id).is_some())
        .map(|entry| entry.id)
        .collect();
    for entry in &mut d.income {
        let context = crate::interview::income_issue_context(entry);
        let stopped = died.contains(&entry.id);
        let (changes, stopped, personal) = match &mut entry.income {
            FederalIncome::Business { data } => (&mut data.method_changes, stopped, false),
            FederalIncome::Rental { data } => (
                &mut data.method_changes,
                stopped
                    || matches!(
                        data.sale,
                        crate::RentalSale::Sold { .. } | crate::RentalSale::Condemned { .. }
                    ),
                !matches!(data.personal_use, crate::extended_model::RentalPersonalUse::No),
            ),
            _ => continue,
        };
        for (position, change) in changes.iter_mut().enumerate() {
            if !personal {
                if stopped {
                    change.accelerate();
                }
                continue;
            }
            notice(
                notes,
                "WARN_METHOD_CHANGE_LEFT_OUT",
                Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                &context.item("method_changes", position),
            );
        }
        if personal {
            changes.clear();
        }
    }
}

fn screening(d: &mut FederalDraft, notes: &mut ScopeNotes<'_>) {
    let averaging_limit = d
        .screening
        .as_ref()
        .and_then(|screen| screen.farm_averaging.as_ref())
        .and_then(|facts| facts.unavailable(d));
    let Some(screening) = d.screening.as_mut() else {
        return;
    };
    if !screening.nol.confirmed() {
        notice(
            notes,
            "WARN_NOL_LEFT_OUT",
            None,
            &IssueContext::Section(TaxSection::SelfEmployment),
        );
        screening.nol = crate::NolHistory::None;
    }
    if let Some(key) = averaging_limit {
        notice(notes, key, None, &IssueContext::Section(TaxSection::Farming));
        screening.farm_averaging = None;
    }
}

// Keep the document and its withholding even when only one component can
// be calculated. The supported native views omit special components without
// rewriting paper or the person's independent treatment answers.
fn broker_sources(d: &FederalDraft, notes: &mut ScopeNotes<'_>) {
    for entry in &d.income {
        let FederalIncome::Broker { data } = &entry.income else { continue };
        let context = crate::interview::income_issue_context(entry);
        let label = || Some(MessageArg::Nested(alloc::boxed::Box::new(context.label())));
        let view = data.projection();
        for key in [view.sale_boundary, view.contracts_boundary].into_iter().flatten() {
            notice(notes, key, None, &context);
        }
        if let Some(receipt) = view.receipt.filter(|receipt| receipt.amount != 0) {
            notice(notes, receipt.boundary(), None, &context);
        }
        if let Some(key) = view.sale.as_ref().and_then(|sale| sale.special.notice()) {
            notice(notes, key, label(), &context);
        }
        if view.contracts.as_ref().is_some_and(|contracts| contracts.special_treatment) {
            notice(notes, "WARN_SECTION_1256_LEFT_OUT", label(), &context);
        }
    }
}

fn income(d: &mut FederalDraft, notes: &mut ScopeNotes<'_>) {
    let joint = d.setup.as_ref().is_some_and(|s| s.filing_status() == crate::FederalStatus::Joint);
    d.income.retain(|entry| { let context = crate::interview::income_context(entry, &d.dependents); match &entry.income {
        // A child's election the return cannot report: the parent's return
        // without it is correct, and the child files their own.
        FederalIncome::ChildElection { data } => match data.omission(&d.dependents, joint) {
            Some(omission) => {
                let context = if omission.alternatives() {
                    context.alternatives(&[TaxSection::InterestDividends, TaxSection::Children])
                } else {
                    context
                };
                notice(notes, omission.key(), Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))), &context);
                // An unresolved choice of return is assumed the other
                // parent's; every other omission leaves the child out.
                false
            }
            None => true,
        },
        // A 529-to-Roth transfer whose records are not confirmed is not a qualified rollover:
        // the form is an ordinary 529 withdrawal (education_account.rs) and the deposit is a
        // regular Roth contribution the person enters on the IRA page.
        FederalIncome::EducationDistribution { data } => {
            let data = &crate::FederalEducationDistribution::from(data);
            if matches!(data.transfer, crate::Q1099Transfer::RothIra)
                && let Some(rollover) = &data.roth_rollover
                && !rollover.confirmed
            {
                let context = context
                    .clone()
                    .alternatives(&[TaxSection::OtherIncome, TaxSection::IraContributions]);
                notice(
                    notes,
                    "NOTE_QTP_ROTH_NOT_QUALIFIED",
                    Some(display_label(&context, &rollover.account_reference)),
                    &context,
                );
            }
            true
        }
        FederalIncome::IsoSale { data } if !data.confirmed => {
            notice(notes, "WARN_ISO_SALE_LEFT_OUT", Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))), &context);
            false
        }
        FederalIncome::Partnership { data }
            if !data.complete()
                || core::iter::once(&data.regular)
                    .chain(data.alternative.iter())
                    .any(|record| !record.basis.complete || !record.risk.complete) =>
        {
            notice(notes, "WARN_PARTNERSHIP_K1_LEFT_OUT", Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))), &context);
            false
        }
        FederalIncome::Partnership { data } if matches!(&data.participation, Some(crate::PartnerParticipation::General { facts }) if facts.left_out()) => {
            notice(notes, "WARN_PARTICIPATION_LEFT_OUT", Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))), &context);
            false
        }
        FederalIncome::Scorp { data } if crate::ScorpK1::from(data).participation.as_ref().is_some_and(|p| p.left_out()) => {
            notice(notes, "WARN_PARTICIPATION_LEFT_OUT", Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))), &context);
            false
        }
        // Other property is figured as its own sale, or on its own page.
        FederalIncome::HomeSale { data } if data.other_property() => true,
        FederalIncome::HomeSale { data } if !crate::HomeSale::from(data).ordinary => {
            notice(notes, "WARN_HOME_SALE_LEFT_OUT", Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))), &context);
            false
        }
        FederalIncome::HomeSale { data } => match crate::HomeSale::from(data).facts.as_ref().and_then(|f| f.left_out()) {
            Some(key) => {
                notice(notes, key, Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))), &context);
                false
            }
            None => true,
        },
        // Preserve income and source guidance. The payroll owner decides whether
        // unavailable coverage can change tax, after all physical bases are known;
        // an immaterial unknown must not remove a known payment capacity.
        FederalIncome::EmployeeWages { data } if data.wages > 0 && matches!(data.payroll, crate::EmployeePayrollSource::Unavailable) => {
            information(notes, "WARN_EMPLOYEE_WAGES_PAYROLL_TAX_LEFT_OUT", Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))), &context);
            true
        }
        FederalIncome::EmployeeWages { data } if data.railroad().is_some_and(|source| source.known()[1..].iter().any(Option::is_none)) => {
            if joint || entry.owner == crate::DocumentOwner::Yours {
            notice(notes, "WARN_EMPLOYEE_RAILROAD_WITHHOLDING_UNAVAILABLE", Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))), &context);
            } else {
            information(notes, "WARN_EMPLOYEE_RAILROAD_WITHHOLDING_UNAVAILABLE", Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))), &context);
            }
            true
        }
        // A recovery that needs the earlier year's tax refigured (unused
        // credits, AMT, a 0% capital gain rate, a dependent filer, an earlier
        // refund of the same deduction) is not calculated.
        FederalIncome::OtherIncome { data } if data.left_out() => {
            notice(notes, "WARN_RECOVERY_LEFT_OUT", Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))), &context);
            false
        }
        FederalIncome::HobbySales { data } if !data.ordinary => {
            notice(notes, "WARN_HOBBY_SALES_LEFT_OUT", Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))), &context);
            false
        }
        FederalIncome::QualifiedSmallBusinessSale { data } if data.special == Some(true) => {
            notice(notes, "WARN_QSBS_SALE_LEFT_OUT", Some(display_label(&context, &data.issuer)), &context);
            false
        }
        FederalIncome::SmallBusinessStock { data } if !data.ownership.supported() => {
            notice(notes, "WARN_SMALL_BUSINESS_STOCK_LEFT_OUT", Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))), &context);
            false
        }
        // Established full section267 disallowance: no current loss, without an unknown-scope notice.
        FederalIncome::SmallBusinessStock { data }
            if data.ownership == crate::StockLossOwnership::RelatedPartyRestriction => false,
        // Worthlessness not established in 2026: no loss this year.
        FederalIncome::SmallBusinessStock { data } if data.event.unconfirmed().is_some() => {

            false
        }
        FederalIncome::Scorp { data } if !crate::ScorpK1::from(data).risk.complete => {
            notice(notes, "WARN_SCORP_K1_LEFT_OUT", Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))), &context);
            false
        }
        FederalIncome::Retirement { data: source } => {
let data = &crate::FederalRetirement::from(source);
match &data.taxable {
            Some(crate::RetirementTaxable::PropertyProceeds { data }) if !data.confirmed => {
                notice(notes, "WARN_PROPERTY_ROLLOVER_LEFT_OUT", Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))), &context);
                false
            }
            _ => true,
        }
},
        _ => true,
    } });
    for entry in &mut d.income {
        let context = crate::interview::income_issue_context(entry);
        match &mut entry.income {
            FederalIncome::Partnership { data } => {
                if let Some(claim) = &data.fuel_tax_credit {
                    let farm_unsupported =
                        claim.fuels.has_farm_operator_use() && !claim.farm_activity;
                    if !claim.statement_complete || farm_unsupported {
                        notice(
                            notes,
                            "WARN_PARTNER_FUEL_CREDIT_LEFT_OUT",
                            Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                            &context,
                        );
                        data.fuel_tax_credit = None;
                    }
                }

                if let Some(prior) = &data.prior_fuel_credit
                    && !prior.in_scope()
                {
                    // Already in another entry: not added again, no notice.
                    if !prior.filed_in_return_year || !prior.cash_method {
                        notice(
                            notes,
                            "WARN_PARTNER_FUEL_RECOVERY_LEFT_OUT",
                            Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                            &context,
                        );
                    }
                    data.prior_fuel_credit = None;
                }
            }
            FederalIncome::Retirement { data: source } => {
                use crate::RetirementTaxable as T;
                // An unconfirmed deferral or basis-recovery method is left out:
                // the whole distribution is taxed as the payer's amount.
                let key = match crate::FederalRetirement::from(&*source).taxable {
                    Some(T::CivilServiceDeath {
                        data: crate::SurvivorLumpSum { eligible: false, .. },
                    })
                    | Some(T::FullDischarge {
                        data: crate::ContractDischarge { eligible: false, .. },
                    })
                    | Some(T::QualifiedWithdrawal {
                        data: crate::PlanDistribution { eligible: false, .. },
                    })
                    | Some(T::DeferredWithdrawal {
                        data: crate::AnnuityDistribution { eligible: false, .. },
                    }) => Some("WARN_RETIREMENT_BASIS_LEFT_OUT"),
                    Some(T::Simplified { data: basis }) if !basis.eligible => {
                        Some("WARN_RETIREMENT_BASIS_LEFT_OUT")
                    }
                    Some(T::GeneralAnnuity { data: annuity }) if !annuity.eligible => {
                        Some("WARN_RETIREMENT_BASIS_LEFT_OUT")
                    }
                    Some(T::VariableAnnuity { data: annuity }) if !annuity.eligible => {
                        Some("WARN_RETIREMENT_BASIS_LEFT_OUT")
                    }
                    Some(T::Railroad { basis: Some(basis), .. }) if !basis.eligible => {
                        notice(
                            notes,
                            "WARN_RETIREMENT_BASIS_LEFT_OUT",
                            Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                            &context,
                        );
                        source.clear_railroad_basis();
                        None
                    }
                    _ => None,
                };
                if let Some(key) = key {
                    notice(
                        notes,
                        key,
                        Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                        &context,
                    );
                    source.tax_payer_amount();
                }
            }
            FederalIncome::Scorp { data }
                if crate::ScorpK1::from(&*data).amt.as_ref().is_some_and(|amt| !amt.complete) =>
            {
                notice(
                    notes,
                    "WARN_SCORP_AMT_LEFT_OUT",
                    Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                    &context,
                );
                data.personal.amt = crate::ScorpK1::from(&*data)
                    .opening
                    .map(|_| crate::ScorpAmtRecords::SameAsRegular);
            }
            _ => {}
        }
    }
}

// The farm or farm rental each Form 1099-PATR and each Form 1099-G with a box 7
// payment is routed to (its chosen farm, or its owner's only one), by document
// identity, taken before any farm is left out.
fn routed_targets(
    d: &FederalDraft,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Vec<(i64, i64)> {
    d.income
        .iter()
        .filter_map(|entry| {
            let choice = match &entry.income {
                FederalIncome::GovernmentPayment { data }
                    if crate::FederalUnemployment::from(data).agriculture_payment != 0 =>
                {
                    data.component_ids.agriculture
                }
                _ => return None,
            };
            let owner = usize::from(entry.owner == crate::DocumentOwner::Spouses);
            crate::federal_rules::routed_farm(d, owner, choice, source_claims)
                .ok()
                .map(|farm| (entry.id, farm))
        })
        .collect()
}

// A farm or farm rental left out takes the documents routed to it with it:
// the 1099-PATR statements and the 1099-G agriculture payments.
fn farm_documents(
    d: &mut FederalDraft,
    targets: &[(i64, i64)],
    removed: &[i64],
    _notes: &mut ScopeNotes<'_>,
) {
    let gone =
        |id: i64| targets.iter().any(|(document, farm)| *document == id && removed.contains(farm));
    // Cooperative components keep their printed facts and withholding. A
    // missing recipient is a required placement source, not erased income.
    for entry in &mut d.income {
        let left_out = gone(entry.id);
        if let FederalIncome::GovernmentPayment { data } = &mut entry.income
            && left_out
        {
            data.form.agriculture_payments = Some(0);
        }
    }
}

fn adjustments(
    d: &mut FederalDraft,
    notes: &mut ScopeNotes<'_>,
    earlier_roth: &[(crate::Owner, i64)],
) {
    let Some(adjustments) = d.adjustments.as_mut() else {
        return;
    };
    // A move that is neither an Armed Forces nor an intelligence-community
    // move is not deductible (section 217(k)): left out, one notice for each
    // owner on More adjustments.
    adjustments.moving.retain(|claim| {
        // Not manual: the note opens More adjustments, where the reason is changed.
        let context = IssueContext::owner(TaxSection::MoreAdjustments, claim.owner);
        if !claim.record.qualifies() {
            notice(notes, "WARN_MOVING_LEFT_OUT", None, &context);
        }
        claim.record.qualifies()
    });
    for hsa in &mut adjustments.hsa {
        let owner = hsa.owner;
        let context = IssueContext::owner(TaxSection::Hsa, hsa.owner).manual();
        retain_scoped(&mut hsa.funding, &context, "hsa.funding", |transfer, context| {
            let supported = transfer.supported()
                || (matches!(transfer.basis, crate::HsaFundingBasis::Roth { .. })
                    && earlier_roth.contains(&(owner, transfer.income)));
            if !supported {
                notice(notes, "WARN_HSA_FUNDING_LEFT_OUT", None, context);
            }
            supported
        });
    }
}

fn deductions(d: &mut FederalDraft, notes: &mut ScopeNotes<'_>) {
    let allocated_entries = crate::community::entries(d);
    let unresolved_estate_investment = allocated_entries.iter().filter(|entry| matches!(&entry.income, FederalIncome::EstateTrustPortfolio { data } if { let data = &crate::FederalEstateTrustPortfolioK1::from(data); crate::estate_trust::investment_adjustment(data).is_none()
        || crate::estate_trust::capital_parts(data, false, crate::estate_trust::CapitalLossUse::Investment).is_none()
        || crate::estate_trust::capital_parts(data, true, crate::estate_trust::CapitalLossUse::Investment).is_none() || data.unentered.contains(&crate::EstateTrustK1Item::InvestmentIncome) })).map(|entry| crate::interview::income_issue_context(entry)).collect::<Vec<_>>();
    drop(allocated_entries);
    // A final-year box 11 code B investment interest deduction is Form 4952
    // line 1 interest too, limited by the same investment income.
    let estate_interest = d.income.iter().any(|entry| {
        matches!(&entry.income, FederalIncome::EstateTrustPortfolio { data }
            if data.statements.final_character.as_ref().is_some_and(|f| f.additional.iter().any(|item|
                matches!(item, crate::EstateFinalDeduction::InvestmentInterest { .. }))))
    });
    if (estate_interest
        || d.deductions
            .as_ref()
            .is_some_and(|deductions| deductions.investment_interest().is_some()))
        && !unresolved_estate_investment.is_empty()
    {
        for context in unresolved_estate_investment {
            notice(notes, "WARN_ESTATE_INVESTMENT_INTEREST_LEFT_OUT", None, &context);
        }
        if let Some(deductions) = d.deductions.as_mut() {
            deductions.investment_interest = None;
        }
        for entry in &mut d.income {
            if let FederalIncome::EstateTrustPortfolio { data } = &mut entry.income
                && let Some(finals) = &mut data.statements.final_character
            {
                finals.additional.retain(|item| {
                    !matches!(item, crate::EstateFinalDeduction::InvestmentInterest { .. })
                });
            }
        }
    }

    let context = IssueContext::Section(TaxSection::Deductions);
    let Some(deductions) = d.deductions.as_mut() else {
        return;
    };
    retain_scoped(
        &mut deductions.hosted_students,
        &IssueContext::Section(TaxSection::GiftCarryovers),
        "deductions.hosted_students",
        |student, context| {
            if !student.documented {
                notice(
                    notes,
                    "WARN_HOSTED_STUDENT_LEFT_OUT",
                    Some(display_label(context, &student.label)),
                    context,
                );
            }
            student.documented
        },
    );
    let mut no_repayment_benefit = false;
    if let crate::RepaymentClaim::Claim { confirmed, years, .. } = &deductions.repayment {
        if !*confirmed {
            // No claim-of-right benefit: neither the deduction nor the credit.
            notice(notes, "WARN_REPAYMENT_LEFT_OUT", None, &context);
            deductions.repayment = crate::RepaymentClaim::None;
            no_repayment_benefit = true;
        } else if !years.iter().all(crate::RepaymentTaxYear::records_supported) {
            notice(notes, "WARN_REPAYMENT_CREDIT_LEFT_OUT", None, &context);
            deductions.repayment.select(crate::RepaymentMethod::Deduction);
        }
    }
    if no_repayment_benefit {
        // The earlier-year repayments have no current income effect; without
        // a benefit they are simply not claimed.
        for entry in &mut d.income {
            match &mut entry.income {
                FederalIncome::GovernmentPayment { data } => {
                    data.repayments.retain(|v| v.received_year >= 2026)
                }
                FederalIncome::Retirement { data } => data.clear_railroad_repayments(),
                _ => (),
            }
        }
    }
    retain_scoped(
        &mut deductions.volunteer_expenses,
        &IssueContext::Section(TaxSection::GiftCarryovers),
        "deductions.volunteer_expenses",
        |expense, context| {
            if !expense.documented {
                notice(
                    notes,
                    "WARN_VOLUNTEER_EXPENSE_LEFT_OUT",
                    Some(display_label(context, &expense.charity)),
                    context,
                );
            }
            expense.documented
        },
    );
    retain_scoped(
        &mut deductions.whaling_captains,
        &IssueContext::Section(TaxSection::GiftCarryovers),
        "deductions.whaling_captains",
        |claim, context| {
            if !claim.eligibility_and_records_confirmed {
                notice(notes, "WARN_WHALING_EXPENSES_LEFT_OUT", None, context);
            }
            claim.eligibility_and_records_confirmed
        },
    );
}

// Schedule K-3 and K-1 foreign taxes (unit k3). A K-1 box 21 / box 16 code F
// tax without a K-3 is never credited; a K-3 with an item outside the Form 1116
// calculation drops the claim (the K-1 stays); a K-3 tax with no claim says how
// to claim it. Returns whether the claim is dropped.
fn schedule_k3(d: &FederalDraft, notes: &mut ScopeNotes<'_>) -> bool {
    use crate::ForeignTaxClaim as F;
    let claim = d.credits.as_ref().map(|c| &c.foreign_tax);
    let mut drop_claim = false;
    for entry in &d.income {
        let scorp = match &entry.income {
            FederalIncome::Scorp { data } => Some(crate::ScorpK1::from(data)),
            _ => None,
        };
        let (k3, foreign_taxes) = match &entry.income {
            FederalIncome::Partnership { data } => (
                data.schedule_k3.as_ref(),
                data.separate.as_ref().map_or(0, |items| items.foreign_taxes),
            ),
            FederalIncome::Scorp { .. } => {
                let data = scorp.as_ref().unwrap();
                (data.schedule_k3.as_ref(), data.foreign_taxes)
            }
            _ => continue,
        };
        let context = crate::interview::income_issue_context(entry);
        let name = || Some(MessageArg::Nested(alloc::boxed::Box::new(context.label())));
        let Some(k3) = k3 else {
            if foreign_taxes > 0 {
                notice(notes, "WARN_K1_FOREIGN_TAX_WITHOUT_K3", name(), &context);
            }
            continue;
        };
        let claim_context = context.clone().alternatives(&[TaxSection::MoreCredits]);
        let name = || Some(MessageArg::Nested(alloc::boxed::Box::new(claim_context.label())));
        match claim {
            Some(F::Ordinary { .. } | F::Statements { .. }) if k3.credit_left_out() => {
                notice(notes, "WARN_K3_FOREIGN_TAX_CREDIT_NOT_CLAIMED", name(), &claim_context);
                drop_claim = true;
            }
            Some(F::No) | None if k3.taxes() > 0 => {
                notice(notes, "WARN_K3_FOREIGN_TAX_NOT_CLAIMED", name(), &claim_context);
            }
            _ => {}
        }
    }
    drop_claim
}

// Whether a Schedule K-3 row is a Form 1116 source, so an Ordinary claim with
// no source of its own is kept.
fn k3_sources(d: &FederalDraft) -> bool {
    crate::schedule_k3::attached(d).iter().any(|k3| !k3.countries.is_empty())
}

// A running Form 8862 ban on the American opportunity credit (section
// 25A(b)(4)) names only that credit: its students are figured with the
// lifetime learning credit, which the section does not name.
fn aotc_ban(d: &mut FederalDraft, notes: &mut ScopeNotes<'_>) {
    use crate::forms::form_8862::{CreditGroup, ban_message, bans};
    let Some(ban) = bans(d.credits.as_ref().and_then(|c| c.after_disallowance.as_ref())).aotc
    else {
        return;
    };
    let students = crate::federal_rules::people_students(d);
    let Some(credits) = d.credits.as_mut() else {
        return;
    };
    let mut switched = false;
    for entry in &mut credits.students {
        if entry.method != Some(crate::EducationChoice::AmericanOpportunity) {
            continue;
        }
        entry.method = Some(crate::EducationChoice::LifetimeLearning);
        switched = true;
        let name = insured_label(&students, entry.student);
        information_args(
            notes,
            "NOTE_8862_AOTC_BANNED_LIFETIME_LEARNING_USED",
            alloc::vec![MessageArg::Encoded(name.clone())],
            &IssueContext::Section(TaxSection::Education).named_encoded(&name),
        );
    }
    if switched {
        information_args(
            notes,
            "NOTE_8862_AOTC_BANNED",
            ban_message(CreditGroup::Education, ban).args,
            &IssueContext::Section(TaxSection::MoreCredits),
        );
    }
}

fn credits(d: &mut FederalDraft, notes: &mut ScopeNotes<'_>) {
    aotc_ban(d, notes);
    let context = IssueContext::Section(TaxSection::MoreCredits);
    let k3_drops_claim = schedule_k3(d, notes);
    let Some(credits) = d.credits.as_mut() else {
        return;
    };
    if k3_drops_claim {
        credits.foreign_tax = crate::ForeignTaxClaim::No;
    }
    if credits.bba_audit_interest.as_ref().is_some_and(|source| source.model().is_err()) {
        notice(notes, "WARN_BBA_AUDIT_LEFT_OUT", None, &context);
        credits.bba_audit_interest = None;
    }
    retain_scoped(
        &mut credits.adoption_carry,
        &context,
        "credits.adoption_carry",
        |carry, context| {
            let kept = carry.eligible || carry.amount == 0;
            if !kept {
                notice(notes, "WARN_ADOPTION_CARRY_LEFT_OUT", None, context);
            }
            kept
        },
    );
    if let crate::ForeignTaxClaim::Ordinary { history, carryback, loss_history, .. } =
        &mut credits.foreign_tax
    {
        if matches!(
            history,
            crate::ForeignCreditHistory::Retained(records) if !records.same_filing_unit
        ) {
            notice(
                notes,
                "WARN_FOREIGN_CREDIT_CARRYOVER_LEFT_OUT",
                None,
                &IssueContext::Section(TaxSection::MoreCredits),
            );
            *history = crate::ForeignCreditHistory::None;
        }
        // Unconfirmed 2025 worksheets: the carryback stays unreviewed, with
        // its own review note.
        if let crate::ForeignCarryback::PriorReturn { confirmed, .. }
        | crate::ForeignCarryback::DeductionYear { confirmed, .. } = carryback
            && !*confirmed
        {
            *carryback = crate::ForeignCarryback::Unreviewed;
        }
        if matches!(
            loss_history,
            crate::ForeignLossHistory::Retained { same_filing_unit: false, .. }
        ) {
            notice(
                notes,
                "WARN_FOREIGN_LOSS_ACCOUNTS_LEFT_OUT",
                None,
                &IssueContext::Section(TaxSection::MoreCredits),
            );
            *loss_history = crate::ForeignLossHistory::None;
        }
    }
    retain_scoped(
        &mut credits.tax_credit_bonds,
        &context,
        "credits.tax_credit_bonds",
        |bond, context| {
            if bond.dividend_income() > 0 {
                information(notes, "WARN_TAX_CREDIT_RIC_ORDINARY", None, context);
            }
            if !bond.supported() {
                notice(
                    notes,
                    if bond
                        .answers
                        .origin
                        .as_ref()
                        .and_then(crate::TaxCreditOrigin::entity)
                        .is_some_and(|e| e.income_kind == crate::TaxCreditIncomeKind::Interest)
                    {
                        "WARN_TAX_CREDIT_K1_INTERFACE_LEFT_OUT"
                    } else {
                        "WARN_TAX_CREDIT_BOND_LEFT_OUT"
                    },
                    Some(display_label(context, &bond.form.issuer.name)),
                    context,
                );
            }

            // An explicitly unconfirmed carry is not claimed. An unanswered
            // confirmation remains unknown and cannot supply a zero carry.
            if bond.answers.prior_carry_confirmed == Some(false) {
                bond.answers.prior_carry_dollars = 0;
            }
            true
        },
    );
}

// Inherited IRA payments (inherited_ira.rs): a Form 1099-R or Roth withdrawal that names no
// record, and has no only record to use, is taxed in full with no basis (the group needs the
// record's basis); a Roth record whose five-year period is not over and that holds no basis
// recovers none. Each is a note, the calculation itself falls back the same way.
fn inherited_links(d: &FederalDraft, notes: &mut ScopeNotes<'_>) {
    for entry in &d.income {
        let context = crate::interview::income_issue_context(entry);
        let label = Some(MessageArg::Nested(alloc::boxed::Box::new(context.label())));
        match &entry.income {
            FederalIncome::InheritedIra { data }
                if data.roth()
                    && !data.roth_qualified()
                    && data.roth_basis().is_none()
                    && crate::inherited_ira::gross(d, entry.id) > 0 =>
            {
                notice(notes, "WARN_INHERITED_ROTH_BASIS_ASSUMED_NONE", label, &context);
            }
            _ => {
                if crate::inherited_ira::link_of(d, entry)
                    == Some(crate::inherited_ira::Link::Missing)
                {
                    notice(notes, "WARN_INHERITED_IRA_RECORD_MISSING", label, &context);
                }
            }
        }
    }
}

// Form 1099-LTC (care_benefit_return.rs): a per diem payment with no Long-term care periods record to
// place it (none entered, or several and none chosen) is taxed in full, with a notice; the
// calculation itself falls back the same way.
fn ltc_links(d: &FederalDraft, notes: &mut ScopeNotes<'_>) {
    for entry in &d.income {
        if crate::care_benefit_return::link_of(d, entry)
            == Some(crate::care_benefit_return::Link::Missing)
        {
            let context = crate::interview::income_issue_context(entry);
            notice(
                notes,
                "WARN_LTC_PERIODS_MISSING",
                Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                &context,
            );
        }
    }
}

// Form 1099-R codes N, P and R (the retirement-special-codes map, R8): the amount is not 2026
// income, so the estimate adds nothing for it and says where it belongs. Its withholding is
// still a 2026 credit.
fn special_code_notes(d: &FederalDraft, notes: &mut ScopeNotes<'_>) {
    for entry in &d.income {
        let Some(data) = entry.income.retirement() else {
            continue;
        };
        let context = crate::interview::income_issue_context(entry);
        let label = Some(MessageArg::Nested(alloc::boxed::Box::new(context.label())));
        if data.box7 & (crate::DistributionCodes::CODE_P.0 | crate::DistributionCodes::CODE_R.0)
            != 0
        {
            information(notes, "NOTE_1099R_EARLIER_YEAR", label, &context);
        } else if data.box7 & crate::DistributionCodes::CODE_N.0 != 0 {
            information(notes, "NOTE_1099R_RECHARACTERIZED", label, &context);
        }
    }
}

// Form 4972 (lump_sum.rs): an election whose conditions are not met (the filer's No, the entire
// balance, the qualifying event and earlier payments) or whose participant was born on or after
// 1936-01-02 is taxed with the other income, with a notice. The death benefit exclusion (line 9)
// and the estate tax (line 18, divided by the box 9a percentage for a shared lump sum) are
// calculated from the amounts the filer enters.
fn lump_sums(d: &mut FederalDraft, notes: &mut ScopeNotes<'_>) {
    let births = d
        .people
        .as_ref()
        .map(|p| [Some(p.taxpayer.birth_date), p.spouse.as_ref().map(|s| s.birth_date)]);
    for entry in &mut d.income {
        let context = crate::interview::income_issue_context(entry);
        let label = Some(MessageArg::Nested(alloc::boxed::Box::new(context.label())));
        let owner = usize::from(entry.owner == crate::DocumentOwner::Spouses);
        let FederalIncome::Retirement { data: source } = &mut entry.income else {
            continue;
        };
        let Some(crate::RetirementTreatment::LumpSum { data: election }) =
            crate::FederalRetirement::from(&*source).ordinary_distribution
        else {
            continue;
        };
        let own_birth = births.and_then(|b| b[owner]);
        let unresolved = !election.qualifies(own_birth);
        let key = if unresolved {
            "WARN_LUMP_SUM_LEFT_OUT"
        } else if !election.birth(own_birth).is_some_and(crate::Form4972Election::born_early_enough)
        {
            "NOTE_LUMP_SUM_PARTICIPANT_BORN_AFTER_1935"
        } else {
            continue;
        };
        source.lump_sum_with_other_income();
        if unresolved {
            notice(notes, key, label, &context);
        } else {
            information(notes, key, label, &context);
        }
    }
}

// Form 2555 records (unit B7b). A tax home or residence answered No takes no
// exclusion or housing amount for that person silently.
// Fewer than 330 full days fails the physical presence test with its own
// note. A housing claim that is not confirmed, or whose spouse answered No to
// the shared commuting condition, is left out; a record left with no choice,
// or with no foreign earned income and no usable 2025 carryover, claims
// nothing and is removed with a note. Runs after the foreign pay and the
// foreign-services businesses that are not confirmed were left out.
fn foreign_claims(d: &mut FederalDraft, notes: &mut ScopeNotes<'_>) {
    use crate::{ForeignHousing, ForeignHousingHousehold, ForeignQualification};
    let mut earned = [0i64; 2];
    for entry in &d.income {
        let i = usize::from(entry.owner == crate::DocumentOwner::Spouses);
        earned[i] += match &entry.income {
            FederalIncome::ForeignEmployment { data } => data.wages,
            FederalIncome::Wage { data } => {
                let data = &crate::FederalWage::from(data);
                data.abroad
                    + crate::statutory_wages::foreign_gross_with_claims(
                        d,
                        entry,
                        data,
                        notes.source_claims,
                    )
            }
            FederalIncome::Business { data }
                if matches!(data.source, crate::BusinessSource::ForeignServices { .. }) =>
            {
                data.receipts()
            }
            _ => 0,
        };
    }
    let Some(screening) = d.screening.as_mut() else {
        return;
    };
    let separate = |h: &ForeignHousing, qualified: bool| {
        matches!(
            h,
            ForeignHousing::Claim {
                household: ForeignHousingHousehold::Separate { qualified: q },
                ..
            } if *q == qualified
        )
    };
    // The spouses' separate-households commuting condition is one shared
    // fact: when either spouse answers No, both separate claims are left out.
    let commuting_no = screening.foreign_earned.iter().any(|claim| separate(&claim.housing, false));
    retain_scoped(
        &mut screening.foreign_earned,
        &IssueContext::Section(TaxSection::WorkAbroad),
        "screening.foreign_earned",
        |claim, _| {
            let context = IssueContext::owner(TaxSection::WorkAbroad, claim.owner);
            let label = Some(MessageArg::Nested(alloc::boxed::Box::new(context.label())));
            if matches!(claim.qualification, ForeignQualification::PriorYearOnly) {
                return claim.exclusion && claim.prior_year_history.is_some();
            }
            if !claim.tax_home {
                return false;
            }
            if matches!(
                claim.qualification,
                ForeignQualification::Residence { nonresident_statement: true, .. }
            ) {
                information(notes, "WARN_FOREIGN_RESIDENCE_STATEMENT", None, &context);
                return false;
            }
            if let ForeignQualification::Presence { foreign_days, .. } = claim.qualification
                && foreign_days < 330
            {
                information_args(
                    notes,
                    "WARN_FOREIGN_PRESENCE_TEST_NOT_MET",
                    label
                        .into_iter()
                        .chain([MessageArg::Number(i64::from(foreign_days))])
                        .collect(),
                    &context,
                );
                return false;
            }
            if !claim.housing.supported() || (commuting_no && separate(&claim.housing, true)) {
                notice(notes, "WARN_FOREIGN_HOUSING_LEFT_OUT", label.clone(), &context);
                claim.housing = ForeignHousing::None;
            }
            if !claim.exclusion && claim.housing == ForeignHousing::None {
                return false;
            }
            let carryover = matches!(
                &claim.housing,
                ForeignHousing::Claim { prior: Some(p), .. } if p.amount > p.deducted
            );
            let i = usize::from(claim.owner == crate::Owner::Spouse);
            if earned[i] == 0 && !carryover {
                information(notes, "WARN_FORM_2555_NO_FOREIGN_INCOME", label, &context);
                return false;
            }
            true
        },
    );
}

// Audit group 4 (batch 4): retirement, rental, business, farm, foreign,
// fuel-credit, health, Roth, refund and Trump-account items.
fn group4(d: &mut FederalDraft, notes: &mut ScopeNotes<'_>) {
    let special_espp: Vec<i64> = d
        .income
        .iter()
        .filter(|entry| crate::espp_return::is_special(d, entry.id))
        .map(|entry| entry.id)
        .collect();
    // The approved partial worksheet does not establish cooperative patron
    // wage allocation. Omit only its new optional capacity on the working
    // copy; the accepted statements, expenses and credits remain untouched.
    let cooperative_businesses: Vec<i64> =
        crate::cooperative::sources_with_claims(d, notes.source_claims)
            .iter()
            .filter_map(|source| source.activity)
            .collect();
    for entry in &mut d.income {
        let context = crate::interview::income_issue_context(entry);
        let payroll = match &mut entry.income {
            FederalIncome::Business { data } => Some(&mut data.payroll),
            FederalIncome::FarmRental { data } => Some(&mut data.payroll),
            _ => None,
        };
        if let Some(crate::BusinessPayroll::Yes { attribution, .. }) = payroll
            && matches!(attribution, crate::PayrollQbiAttribution::Allocated { .. })
            && cooperative_businesses.contains(&entry.id)
        {
            *attribution = crate::PayrollQbiAttribution::Unavailable;
            notice(
                notes,
                "WARN_QBI_PARTIAL_COOP_WAGES_LEFT_OUT",
                Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                &context,
            );
        }
    }
    // The insured people a long-term-care premium names, for its notices.
    let people = crate::federal_rules::people_students(d);
    let health_people = crate::health::person_eligibility(d);
    use crate::RetirementTaxable as T;
    if let Some(screening) = d.screening.as_mut() {
        if !screening.atnol.confirmed() {
            notice(
                notes,
                "WARN_ATNOL_LEFT_OUT",
                None,
                &IssueContext::Section(TaxSection::SelfEmployment),
            );
            screening.atnol = crate::AtnolHistory::None;
        }
        retain_scoped(
            &mut screening.capital_history,
            &IssueContext::Section(TaxSection::InvestmentSales),
            "screening.capital_history",
            |history, context| {
                if !history.ordinary_history {
                    notice(notes, "WARN_CAPITAL_LOSS_CARRYOVER_LEFT_OUT", None, context);
                }
                history.ordinary_history
            },
        );
    }
    // A government's one paper can contain independent payment and refund
    // components. Leaving one unsupported component out never removes its
    // other amounts or the paper's withholding from the working projection.
    for entry in &mut d.income {
        let context = crate::interview::income_issue_context(entry);
        if let FederalIncome::GovernmentPayment { data } = &mut entry.income {
            if data.form.state_or_local_income_tax_refunds != 0 && !data.history.supported() {
                notice(
                    notes,
                    "WARN_TAX_REFUND_LEFT_OUT",
                    Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                    &context,
                );
                data.form.state_or_local_income_tax_refunds = 0;
                data.history = crate::TaxRefundHistory::NotDeducted;
            }
            if data.special && (data.form.has_payments() || !data.repayments.is_empty()) {
                notice(
                    notes,
                    "WARN_UNEMPLOYMENT_LEFT_OUT",
                    Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                    &context,
                );
                data.form.unemployment_compensation = 0;
                data.form.rtaa_payments = Some(0);
                data.form.taxable_grants = Some(0);
                data.form.agriculture_payments = Some(0);
                data.form.market_gain = Some(0);
                data.form.family_leave_benefits = 0;
                data.repayments.clear();
                data.special = false;
            }
        }
    }
    d.income.retain(|entry| {
        let context = crate::interview::income_issue_context(entry);
        let key = match &entry.income {
            FederalIncome::EsppSaleSource { .. } if special_espp.contains(&entry.id) => {
                "WARN_ESPP_SALE_LEFT_OUT"
            }
            FederalIncome::EsppSaleSource { .. } => "ESPP_ACQUISITION_NEEDED",
            // A printed ISO or NUA sale still waiting for its answers.
            FederalIncome::IsoSaleSource { .. } => "ISO_SALE_ANSWERS_NEEDED",
            FederalIncome::NuaSaleSource { .. } => "NUA_SALE_ANSWERS_NEEDED",
            FederalIncome::EsppSale { data } if data.special => "WARN_ESPP_SALE_LEFT_OUT",
            FederalIncome::Rental { data } if !data.ordinary_rental => "WARN_RENTAL_LEFT_OUT",
            // A direct transfer to a plan that does not account for the after-tax part
            // separately leaves 2026 income unchanged (the after-tax part is untaxed either
            // way): the entry computes as confirmed and the note says what to ask the plan.
            FederalIncome::Retirement { data: source }
                if {
                    let data = &crate::FederalRetirement::from(source);
                    matches!(
                        data.ordinary_distribution,
                        Some(crate::RetirementTreatment::Rollover {
                            destination: crate::RolloverDestination::DirectEmployerPlan {
                                confirmed: false
                            },
                            ..
                        })
                    )
                } =>
            {
                notice(
                    notes,
                    "WARN_DIRECT_PLAN_BASIS_NOT_TRACKED",
                    Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                    &context,
                );
                return true;
            }
            _ => return true,
        };
        notice(
            notes,
            key,
            Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
            &context,
        );
        false
    });
    for entry in &mut d.income {
        let context = crate::interview::income_issue_context(entry);
        let label = entry.label.as_str();
        // No: no Schedule R credit on this disability income.
        let disability = match &mut entry.income {
            FederalIncome::Wage { data } => {
                data.answers_mut().and_then(|answers| answers.disability.as_mut())
            }
            _ => None,
        };
        if let Some(disability) = disability
            && let crate::DisabilityIncome::Qualifying { eligible, .. } = disability
            && !*eligible
        {
            *disability = crate::DisabilityIncome::None;
        }
        if let FederalIncome::Retirement { data } = &mut entry.income
            && let crate::DisabilityIncome::Qualifying { eligible: false, .. } =
                crate::FederalRetirement::from(&*data).disability
        {
            data.leave_out_disability();
        }
        match &mut entry.income {
            FederalIncome::Retirement { data: source } => {
                if let Some(T::LegacyAnnuity { data: legacy }) =
                    &crate::FederalRetirement::from(&*source).taxable
                    && !legacy.confirmed
                {
                    notice(
                        notes,
                        "WARN_RETIREMENT_BASIS_LEFT_OUT",
                        Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                        &context,
                    );
                    source.tax_payer_amount();
                }
            }
            FederalIncome::InheritedIra { data } => {
                // A basis or estate-tax record the filer could not confirm
                // takes no estate-tax deduction this year, with a note.
                if let crate::InheritedIraKindFacts::Traditional { estate_tax, .. } = &mut data.kind
                    && estate_tax.as_ref().is_some_and(|tax| !tax.confirmed)
                {
                    notice(notes, "WARN_IRD_ESTATE_TAX_NOT_DEDUCTED", None, &context);
                    *estate_tax = None;
                }
            }
            FederalIncome::FarmRental { data } if data.payroll.qbi_unavailable() => {
                notice(
                    notes,
                    "WARN_QBI_WAGES_LEFT_OUT",
                    Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                    &context,
                );
            }
            FederalIncome::Rental { data } => {
                if matches!(data.sale, crate::RentalSale::Sold { .. })
                    && data.statement.as_ref().is_some_and(|s| s.box_6_property_or_services)
                {
                    notice(
                        notes,
                        "WARN_RENTAL_SALE_PROPERTY_RECEIVED",
                        Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                        &context,
                    );
                }
                if data.payroll.qbi_unavailable() {
                    notice(
                        notes,
                        "WARN_QBI_WAGES_LEFT_OUT",
                        Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                        &context,
                    );
                }
            }
            FederalIncome::Business { data } => {
                if data.payroll.qbi_unavailable() {
                    notice(
                        notes,
                        "WARN_QBI_WAGES_LEFT_OUT",
                        Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                        &context,
                    );
                }
                if matches!(data.inventory, crate::BusinessInventory::Cost { eligible: false, .. })
                {
                    notice(
                        notes,
                        "WARN_INVENTORY_LEFT_OUT",
                        Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                        &context,
                    );
                    data.inventory = crate::BusinessInventory::None;
                }
                health_plan(
                    &mut data.health_insurance,
                    label,
                    &people,
                    &health_people,
                    notes,
                    &context,
                );
                if let Some(credit) = &mut data.fuel_tax_credit
                    && fuel_lines(&mut credit.fuels, notes, &context)
                {
                    data.fuel_tax_credit = None;
                }
                if let BusinessActivity::Farming { data: farm } = &mut data.activity
                    && farm.supplies.as_ref().is_some_and(|supplies| {
                        supplies.advances.iter().any(|advance| !advance.purchase.eligible())
                    })
                {
                    notice(
                        notes,
                        "WARN_FARM_SUPPLIES_LEFT_OUT",
                        Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                        &context,
                    );
                }
            }
            FederalIncome::Scorp { data } => {
                health_plan(
                    &mut data.personal.health_insurance,
                    label,
                    &people,
                    &health_people,
                    notes,
                    &context,
                );
            }
            FederalIncome::Partnership { data } => {
                health_plan(
                    &mut data.health_insurance,
                    label,
                    &people,
                    &health_people,
                    notes,
                    &context,
                );
                if let Some(credit) = &mut data.fuel_tax_credit
                    && fuel_lines(&mut credit.fuels, notes, &context)
                {
                    data.fuel_tax_credit = None;
                }
            }
            _ => {}
        }
    }
    if let Some(deductions) = d.deductions.as_mut() {
        let facts = &mut deductions.itemized;
        retain_scoped(
            &mut facts.long_term_care,
            &IssueContext::Section(TaxSection::Medical),
            "facts.long_term_care",
            |premium, context| {
                // Preserve invalid accepted references for normal graph rejection.
                // Omission of an optional benefit cannot repair a dangling ID.
                if !health_people.contains_key(&premium.person) {
                    return true;
                }
                if !premium.qualified.qualified()
                    || !health_people.get(&premium.person).is_some_and(|v| v.medical)
                {
                    notice(
                        notes,
                        "WARN_LTC_PREMIUM_LEFT_OUT",
                        Some(insured_arg(&people, premium.person)),
                        context,
                    );
                }
                // Keep source costs for entity-payment reconciliation. The health
                // owner omits the unavailable benefit using these same facts.
                true
            },
        );
    }
    let k3_rows = k3_sources(d);
    let estate_foreign = d.income.iter().any(|entry| crate::estate_foreign::admitted(d, entry.id));
    if d.credit_refusals.contains(&crate::MoreCreditRefusal::OtherCredit) {
        notice(
            notes,
            "WARN_OTHER_CREDIT_LEFT_OUT",
            None,
            &IssueContext::Section(TaxSection::MoreCredits),
        );
        d.credit_refusals.clear();
    }
    if let Some(credits) = d.credits.as_mut() {
        use crate::ForeignTaxClaim as F;
        if let F::Ordinary { sources, situations, history, loss_history, amt_limitation, .. } =
            &mut credits.foreign_tax
        {
            // No: the election is not in force, so the normal method applies.
            if matches!(
                amt_limitation,
                crate::ForeignAmtLimitation::Simplified { confirmed: false, .. }
            ) {
                *amt_limitation = crate::ForeignAmtLimitation::Normal;
            }
            if sources.is_empty()
                && !k3_rows
                && !estate_foreign
                && !history.present()
                && !loss_history.present()
            {
                credits.foreign_tax = F::No;
            } else if !situations.is_empty() {
                // A checked situation this estimate does not figure leaves out
                // the whole Form 1116 claim, one note for each (the loss
                // allocation and recapture tie the categories together).
                for code in situations.iter() {
                    if let Some(situation) = crate::ForeignTaxSituation::from_code(*code) {
                        notice(
                            notes,
                            "WARN_FOREIGN_SITUATION_LEFT_OUT",
                            Some(MessageArg::Resource(situation.label_key().into())),
                            &IssueContext::Section(TaxSection::MoreCredits),
                        );
                    }
                }
                credits.foreign_tax = F::No;
            }
        }
        if matches!(
            &credits.foreign_tax,
            F::Statements { eligible, .. } if *eligible != Some(true)
        ) {
            notice(
                notes,
                "WARN_FOREIGN_TAX_CREDIT_NOT_CLAIMED",
                None,
                &IssueContext::Section(TaxSection::MoreCredits),
            );
            credits.foreign_tax = F::No;
        }
    }
}

// Preserve factual plan ownership and independently known medical cost.
// Unconfirmed LTC qualification omits only that optional premium benefit.
fn health_plan(
    plan: &mut crate::BusinessHealthInsurance,
    _label: &str,
    people: &[crate::PersonChoice],
    health_people: &alloc::collections::BTreeMap<i64, crate::health::PersonEligibility>,
    notes: &mut ScopeNotes<'_>,
    context: &IssueContext,
) {
    let context = context.clone().corrective(TaxSection::SelfEmployedBenefits);
    if let crate::BusinessHealthInsurance::Plan { long_term_care, establishment, .. } = plan {
        if !establishment.included() {
            notice(
                notes,
                "WARN_SE_HEALTH_LEFT_OUT",
                Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                &context,
            );
        }
        retain_scoped(
            long_term_care,
            &context.clone().manual(),
            "long_term_care",
            |premium, context| {
                // Preserve invalid accepted references for normal graph rejection.
                // Omission of an optional benefit cannot repair a dangling ID.
                if !health_people.contains_key(&premium.person) {
                    return true;
                }
                if !premium.qualified.qualified()
                    || !health_people
                        .get(&premium.person)
                        .is_some_and(|v| v.medical || v.self_employed)
                {
                    notice(
                        notes,
                        "WARN_LTC_PREMIUM_LEFT_OUT",
                        Some(insured_arg(people, premium.person)),
                        context,
                    );
                }
                // Keep source costs for entity-payment reconciliation. The health
                // owner omits the unavailable benefit using these same facts.
                true
            },
        );
        // An otherwise empty plan can establish canonical Marketplace
        // assignments; source normalization must preserve its factual owner.
    }
}

// Fuel sales and claims whose answers do not qualify are left out; true when
// nothing is left to claim.
fn fuel_lines(
    fuels: &mut crate::FederalFuelLines,
    notes: &mut ScopeNotes<'_>,
    context: &IssueContext,
) -> bool {
    let (sales, claims) = fuels.drop_unsupported();
    if sales {
        notice(notes, "WARN_FUEL_SALE_LEFT_OUT", None, context);
    }
    if claims {
        notice(notes, "WARN_FUEL_CLAIM_LEFT_OUT", None, context);
    }
    (sales || claims) && !fuels.has_lines()
}

// Audit group 1, rows 8-101 (batch 5a): filers, community property, income
// statements with special treatment, Schedule C scope answers, fuel uses and
// recoveries, de minimis expenses and the estate or trust K-1 activities.
fn group1a(
    d: &mut FederalDraft,
    notes: &mut ScopeNotes<'_>,
    direct_participation: &alloc::sync::Arc<crate::direct_participation::DirectParticipationView>,
    macrs: &alloc::sync::Arc<crate::macrs_source::MacrsElectionView>,
) {
    if let Some(people) = d.people.as_mut() {
        for (owner, person) in [
            (crate::Owner::Taxpayer, Some(&mut people.taxpayer)),
            (crate::Owner::Spouse, people.spouse.as_mut()),
        ]
        .into_iter()
        .filter_map(|(owner, person)| person.map(|p| (owner, p)))
        {
            let context = IssueContext::owner(
                if owner == crate::Owner::Taxpayer {
                    TaxSection::People
                } else {
                    TaxSection::Spouse
                },
                owner,
            );
            let name = person.name();
            if let Some(died) = person.died.as_mut()
                && died.bonds.as_ref().is_some_and(|bonds| !bonds.eligible)
            {
                notice(
                    notes,
                    "WARN_FINAL_BONDS_LEFT_OUT",
                    Some(MessageArg::Encoded(name)),
                    &context,
                );
                died.bonds = None;
            }
        }
        // A deceased shareholder's suspended S corporation losses are one of the
        // death-related adjustments the answer "no other adjustment applies"
        // excludes: when the person says one applies, the K-1 is calculated
        // without its suspended losses (the adjustments notice says so).
        for entry in d.income.iter_mut() {
            if let FederalIncome::Scorp { data } = &mut entry.income
                && let Some(person) = people.person(entry.owner.owned())
                && person.died.as_ref().is_some_and(|died| !died.ordinary)
            {
                drop_scorp_suspended_history(data);
            }
        }
    }
    // At-risk limits apply only to a loss (Form 6198): a profitable business
    // not fully at risk is calculated normally.
    let at_risk_losses: Vec<i64> = d
        .income
        .iter()
        .filter_map(|entry| match &entry.income {
            FederalIncome::Business { data }
                if direct_participation.get(entry.id)
                    != crate::participation::Participation::Unavailable
                    && !data.current_risk_supported(direct_participation.get(entry.id))
                    && crate::section179::operating_in_context(
                        d,
                        entry.id,
                        data,
                        notes.source_claims,
                        macrs,
                    )
                    .and_then(|operating| {
                        business_loss_needs_risk_with_claims(
                            crate::related_forms::RelatedFormContext::bound(
                                d,
                                notes.source_claims,
                                direct_participation,
                            )
                            .with_macrs(macrs),
                            entry.id,
                            data,
                            operating,
                        )
                    })
                    .is_ok_and(|loss| loss) =>
            {
                Some(entry.id)
            }
            // An answered Form4835 No without supplied method facts keeps
            // the original named omission of an operating loss. A supplied
            // method belongs to the actual Form6198 owner instead.
            FederalIncome::FarmRental { data }
                if matches!(
                    data.at_risk,
                    Some(crate::RentalAtRisk::Limited { record: None, .. })
                ) && data
                    .net_in_context(
                        d,
                        usize::from(entry.owner == crate::DocumentOwner::Spouses),
                        entry.id,
                        crate::depreciation::mid_quarter(d),
                        false,
                        notes.source_claims,
                        Some(macrs.as_ref()),
                    )
                    .is_ok_and(|net| net < 0) =>
            {
                Some(entry.id)
            }
            _ => None,
        })
        .collect();
    let targets = routed_targets(d, notes.source_claims);
    let mut removed = Vec::new();
    d.income.retain(|entry| {
        let context = crate::interview::income_issue_context(entry);
        let key = match &entry.income {
            FederalIncome::UndistributedGain { data } if data.special => {
                "WARN_UNDISTRIBUTED_GAIN_LEFT_OUT"
            }
            // A special gain code or treatment the App cannot compute leaves an opportunity fund
            // investment out; a deferral whose investment came after the 180 days defers nothing
            // (decision 1, the no-benefit path); a special treatment leaves it out.
            FederalIncome::OpportunityFund { data } if data.left_out() => {
                "WARN_QOF_INVESTMENT_LEFT_OUT"
            }
            FederalIncome::OpportunityFundDeferral { data } if data.special => {
                "WARN_QOF_DEFERRAL_LEFT_OUT"
            }
            FederalIncome::OpportunityFundDeferral { data } if !data.within_180_days => {
                "WARN_QOF_DEFERRAL_NOT_ALLOWED"
            }
            FederalIncome::Interest { data } => {
                match crate::FederalInterest::from(data).special.notice() {
                    Some(key) => key,
                    None => return true,
                }
            }
            FederalIncome::Dividend { data } => match data.special.notice() {
                Some(key) => key,
                None => return true,
            },
            FederalIncome::Oid { data } => match crate::FederalOid::from(data).special.notice() {
                Some(key) => key,
                None => return true,
            },
            FederalIncome::Sale { data } => match data.special.notice() {
                Some(key) => key,
                None => return true,
            },
            FederalIncome::Section1256 { data } if data.special => "WARN_SECTION_1256_LEFT_OUT",
            FederalIncome::Gambling { data } if !crate::FederalGambling::from(data).ordinary => {
                "WARN_GAMBLING_LEFT_OUT"
            }
            FederalIncome::Business { data } => {
                let key = if matches!(
                    data.activity,
                    BusinessActivity::Gambling { eligible: false, .. }
                ) {
                    "WARN_GAMBLING_BUSINESS_LEFT_OUT"
                } else if (direct_participation.get(entry.id)
                    == crate::participation::Participation::No
                    && !data.current_passive_supported(direct_participation.get(entry.id)))
                    || data.special_treatment
                    || matches!(
                        data.source,
                        crate::BusinessSource::ForeignServices { confirmed: false, .. }
                    )
                {
                    "WARN_BUSINESS_LEFT_OUT"
                } else if at_risk_losses.contains(&entry.id) {
                    "WARN_BUSINESS_LOSS_LEFT_OUT"
                } else {
                    return true;
                };
                if matches!(data.activity, BusinessActivity::Farming { .. }) {
                    removed.push(entry.id);
                }
                key
            }
            FederalIncome::FarmRental { .. } if at_risk_losses.contains(&entry.id) => {
                removed.push(entry.id);
                "WARN_FARM_RENTAL_LOSS_LEFT_OUT"
            }
            _ => return true,
        };
        notice(
            notes,
            key,
            Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
            &context,
        );
        false
    });
    farm_documents(d, &targets, &removed, notes);
    for entry in &mut d.income {
        let context = crate::interview::income_issue_context(entry);
        let label = entry.label.as_str();
        match &mut entry.income {
            FederalIncome::Rental { data } => {
                let before = data.expenses.len();
                data.expenses.retain(|expense| {
                    !matches!(
                        expense.treatment,
                        crate::BusinessExpenseKind::DeMinimis { eligible: false, .. }
                    )
                });
                if data.expenses.len() != before {
                    notice(
                        notes,
                        "WARN_DE_MINIMIS_EXPENSE_LEFT_OUT",
                        Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                        &context,
                    );
                }
            }
            FederalIncome::Business { data } => {
                if data.de_minimis.as_ref().is_some_and(|v| !v.eligible) {
                    data.de_minimis = None;
                    notice(
                        notes,
                        "WARN_DE_MINIMIS_EXPENSE_LEFT_OUT",
                        Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                        &context.item("de_minimis", 0),
                    );
                }
                let before = data.expenses.len();
                data.expenses.retain(|expense| {
                    !matches!(
                        expense.treatment,
                        crate::BusinessExpenseKind::DeMinimis { eligible: false, .. }
                    )
                });
                if data.expenses.len() != before {
                    notice(
                        notes,
                        "WARN_DE_MINIMIS_EXPENSE_LEFT_OUT",
                        Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                        &context,
                    );
                }
                if data.tips.as_ref().is_some_and(|tips| !tips.eligible) {
                    notice(
                        notes,
                        "WARN_BUSINESS_TIPS_LEFT_OUT",
                        Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                        &context,
                    );
                    if let Some(tips) = &mut data.tips {
                        tips.amount = 0;
                    }
                }
                if data.overtime.as_ref().is_some_and(|overtime| !overtime.eligible) {
                    notice(
                        notes,
                        "WARN_BUSINESS_OVERTIME_LEFT_OUT",
                        Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                        &context,
                    );
                    if let Some(overtime) = &mut data.overtime {
                        overtime.amount = 0;
                    }
                }
                if data.fuel_tax_credit.as_ref().is_some_and(|credit| !credit.records_complete) {
                    notice(
                        notes,
                        "WARN_FUEL_CREDIT_LEFT_OUT",
                        Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                        &context,
                    );
                    data.fuel_tax_credit = None;
                }
                if let Some(credit) = &mut data.fuel_tax_credit
                    && fuel_uses(&mut credit.fuels, notes, &context)
                {
                    data.fuel_tax_credit = None;
                }
                fuel_recovery(&mut data.cash_fuel_tax_recovery, label, notes, &context);
            }
            FederalIncome::Partnership { data } => {
                if let Some(credit) = &mut data.fuel_tax_credit
                    && fuel_uses(&mut credit.fuels, notes, &context)
                {
                    data.fuel_tax_credit = None;
                }
            }
            _ => {}
        }
    }
}

// Ultimate-purchaser fuel uses whose answers do not qualify are left out;
// true when nothing is left to claim.
fn fuel_uses(
    fuels: &mut crate::FederalFuelLines,
    notes: &mut ScopeNotes<'_>,
    context: &IssueContext,
) -> bool {
    let dropped = fuels.drop_unconfirmed_uses();
    if dropped.left_out {
        notice(notes, "WARN_FUEL_USE_LEFT_OUT", None, context);
    }

    dropped.left_out && !fuels.has_lines()
}

// A Schedule C cash-method fuel-tax recovery: one not received in 2026 is
// left out, one already in receipts is not added again, and an unsupported
// or unknown partial benefit includes the whole recovery.
fn fuel_recovery(
    recovery: &mut Option<crate::FederalCashFuelRecovery>,
    _label: &str,
    notes: &mut ScopeNotes<'_>,
    context: &IssueContext,
) {
    let Some(value) = recovery.as_mut() else {
        return;
    };
    if !value.occurred_in_year {
        notice(
            notes,
            "WARN_FUEL_RECOVERY_LEFT_OUT",
            Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
            context,
        );
        *recovery = None;
        return;
    }
    if !value.not_reported_elsewhere {
        *recovery = None;
        return;
    }
    match value.full_tax_benefit {
        true => {}
        false
            if value
                .partial_prior_year_benefit
                .as_ref()
                .is_some_and(crate::FederalPartialCashFuelTaxBenefit::qualified) => {}
        false => {
            notice(
                notes,
                "WARN_PARTNER_FUEL_PARTIAL_BENEFIT_LEFT_OUT",
                Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                context,
            );
            value.full_tax_benefit = true;
            value.partial_prior_year_benefit = None;
        }
    }
    if !value.niit_recovery_excluded {
        notice(
            notes,
            "WARN_PARTNER_FUEL_NIIT_RECOVERY_LEFT_OUT",
            Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
            context,
        );
        value.niit_recovery_excluded = true;
    }
}

// Unit B9-1: the Schedule K-1 box items the estimate does not compute are left
// out with one notice each, and the K-1 stays (answers and unknowns, decision
// 1 and the inventor decision of 2026-10-10): the box 13 (12) codes that are a
// benefit the App does not figure, a section 108 or section 111 exclusion the
// App does not apply, the item-only situations of Something else applies and
// box 20 code AQ. Reported deductions still reduce basis. Unit B9-7 also
// removes uncomputed credits and oil/gas depletion from this working copy;
// no unclaimed depletion deduction reduces basis or SE earnings.
fn k1_items_left_out(d: &mut FederalDraft, notes: &mut ScopeNotes<'_>) {
    use crate::k1_items::K1ItemKind;
    for entry in &mut d.income {
        let context = crate::interview::income_issue_context(entry);
        let (items, specials, crp, information) = match &mut entry.income {
            FederalIncome::Partnership { data } => (
                data.k1_items(),
                data.special.iter().filter_map(|situation| situation.item()).collect::<Vec<_>>(),
                data.crp_payments != 0,
                {
                    let names =
                        data.unentered.iter().filter_map(|item| item.item()).collect::<Vec<_>>();
                    data.unentered.retain(|item| !item.left_out());
                    names
                },
            ),
            FederalIncome::Scorp { data } => {
                let native = crate::ScorpK1::from(&*data);
                let items = native.k1_items();
                let specials = native
                    .special
                    .iter()
                    .filter_map(|situation| situation.item())
                    .collect::<Vec<_>>();
                let names =
                    native.unentered.iter().filter_map(|item| item.item()).collect::<Vec<_>>();
                data.statements.situations.retain(|item| !item.left_out());
                data.paper.information.retain(|row| row.code != "R");
                data.paper.credits.retain(|row| {
                    matches!(row.code.as_str(), "AX" | "O" | "E")
                        || row.code == "M" && row.amount.is_none_or(|amount| amount >= 0)
                        || native.credits.iter().any(|credit| {
                            crate::partnership_source::credit_code_name(&credit.code) == row.code
                        })
                });
                (items, specials, false, names)
            }
            _ => continue,
        };
        let label = MessageArg::Nested(alloc::boxed::Box::new(context.label()));
        let mut names: Vec<&'static str> = items.left_out();
        if items.has(K1ItemKind::DebtCancellation) {
            names.push("K1_ITEM_DEBT_EXCLUSION");
        }
        if items.has(K1ItemKind::Recovery) {
            names.push("K1_ITEM_RECOVERY_EXCLUSION");
        }
        names.extend(specials);
        names.extend(information);
        if crp {
            names.push("K1_ITEM_CRP_PAYMENTS");
        }
        for name in names {
            notice_args(
                notes,
                "WARN_K1_ITEM_LEFT_OUT",
                alloc::vec![label.clone(), MessageArg::Resource(name.into())],
                &context,
            );
        }
    }
}

// Scope charity carryovers, child annuities, employee stock and the federal
// model's relief, bad-debt, credit, mortgage and charity confirmations.
fn group1b(d: &mut FederalDraft, notes: &mut ScopeNotes<'_>) {
    use crate::RetirementTaxable as T;
    d.income.retain(|entry| {
        let context = crate::interview::income_issue_context(entry);
        let key = match &entry.income {
            FederalIncome::EmployeeStockSale { data } if data.special => {
                "WARN_EMPLOYEE_STOCK_SALE_LEFT_OUT"
            }
            FederalIncome::NuaSale { data } if data.special => "WARN_NUA_SALE_LEFT_OUT",
            FederalIncome::Scorp { data } if !crate::ScorpK1::from(data).basis_complete() => {
                "WARN_SCORP_K1_LEFT_OUT"
            }
            FederalIncome::DisasterMitigation { data } if !data.qualified => {
                "WARN_DISASTER_MITIGATION_LEFT_OUT"
            }
            FederalIncome::Royalty { data } if !data.ordinary => {
                "WARN_NONBUSINESS_PROPERTY_LEFT_OUT"
            }
            FederalIncome::NonbusinessProperty { ordinary: false, .. } => {
                "WARN_NONBUSINESS_PROPERTY_LEFT_OUT"
            }
            FederalIncome::ReturnOfCapital { special: true, .. } => {
                "WARN_RETURN_OF_CAPITAL_LEFT_OUT"
            }
            FederalIncome::WorthlessSecurity { data } if !data.ordinary => {
                "WARN_WORTHLESS_SECURITY_LEFT_OUT"
            }
            // Worthlessness not established in 2026: no loss this year.
            FederalIncome::WorthlessSecurity { data } => return data.worthless,
            FederalIncome::NonbusinessBadDebt { data } if !data.source.supported() => {
                "WARN_BAD_DEBT_LEFT_OUT"
            }
            // A factual No (not genuine, not worthless in 2026, a political
            // debtor, a source fact that fails) means no loss. A pending
            // legal guarantee determination remains unavailable with its existing note.
            FederalIncome::NonbusinessBadDebt { data } => {
                if data.political_debtor || !data.bona_fide || !data.worthless {
                    return false;
                }
                match data.source.facts() {
                    Some(qualified) => return qualified,
                    None => "WARN_BAD_DEBT_NOT_DEDUCTED",
                }
            }
            _ => return true,
        };
        notice(
            notes,
            key,
            Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
            &context,
        );
        false
    });
    for entry in &mut d.income {
        let context = crate::interview::income_issue_context(entry);
        match &mut entry.income {
            FederalIncome::Business { data } => {
                // Omit only the uncomputed benefit, retaining this activity's
                // known income. IDs and accepted predecessor answers are unchanged;
                // association validity belongs to the complete source graph.
                retain_scoped(
                    &mut data.intangibles,
                    &context,
                    "data.intangibles",
                    |record, item| {
                        if record.mandatory_tax_boundary().is_some() {
                            return true;
                        }
                        let Some(reason) = record.boundary() else { return true };
                        notice_args(
                            notes,
                            "WARN_INTANGIBLE_BENEFIT_LEFT_OUT",
                            alloc::vec![
                                display_label(item, &record.label),
                                MessageArg::Nested(alloc::boxed::Box::new(Message::new(
                                    reason,
                                    alloc::vec![],
                                ))),
                            ],
                            &item.clone().named(&record.label),
                        );
                        false
                    },
                );
                let beauty_unsupported = |credit: &crate::EmployerTipFicaCredit| {
                    matches!(
                        credit.service,
                        crate::EmployerTipService::BeautyServices { eligible_tips_only: false, .. }
                    )
                };
                if data
                    .employer_tip_fica_credit
                    .as_ref()
                    .is_some_and(|credit| !credit.confirmed || beauty_unsupported(credit))
                {
                    notice(
                        notes,
                        "WARN_TIP_FICA_CREDIT_LEFT_OUT",
                        Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                        &context,
                    );
                    data.employer_tip_fica_credit = None;
                }
                if data.work_opportunity_credit.as_ref().is_some_and(|credit| !credit.confirmed) {
                    notice(
                        notes,
                        "WARN_WORK_OPPORTUNITY_CREDIT_LEFT_OUT",
                        Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                        &context,
                    );
                    data.work_opportunity_credit = None;
                }
                if data.retirement_plan_credit.as_ref().is_some_and(|credit| !credit.confirmed) {
                    notice(
                        notes,
                        "WARN_RETIREMENT_PLAN_CREDIT_LEFT_OUT",
                        Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                        &context,
                    );
                    data.retirement_plan_credit = None;
                }
                if let Some(credit) = &data.small_employer_health_credit {
                    let key = if credit.coverage == crate::HealthCreditCoverage::Other {
                        Some("WARN_SMALL_EMPLOYER_HEALTH_NOT_SHOP")
                    } else if !credit.confirmed {
                        Some("WARN_SMALL_EMPLOYER_HEALTH_CREDIT_LEFT_OUT")
                    } else {
                        None
                    };
                    if let Some(key) = key {
                        notice(
                            notes,
                            key,
                            Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                            &context,
                        );
                        data.small_employer_health_credit = None;
                    }
                }
                if data.paid_leave_credit.as_ref().is_some_and(|credit| !credit.confirmed) {
                    notice(
                        notes,
                        "WARN_PAID_LEAVE_CREDIT_LEFT_OUT",
                        Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                        &context,
                    );
                    data.paid_leave_credit = None;
                }
            }
            FederalIncome::EstateTrustPortfolio { data: stored } => {
                let mut record = crate::FederalEstateTrustPortfolioK1::from(&*stored);
                let data = &mut record;
                for activity in &mut data.activities.activities {
                    let unresolved = matches!(
                        activity.participation,
                        crate::EstateParticipation::Unresolved
                    ) || matches!(&activity.participation, crate::EstateParticipation::Material(p) | crate::EstateParticipation::Professional(p) if p.left_out());
                    if unresolved {
                        notice_args(
                            notes,
                            "WARN_ESTATE_ACTIVITY_LOSS_LEFT_OUT",
                            alloc::vec![MessageArg::Text(activity.name.clone())],
                            &context,
                        );
                        // Preserve known positive source income. An unclassified
                        // loss/deduction is an unavailable benefit, not permission
                        // to omit this K-1's ordinary portfolio income.
                        activity.income = phase::floor(activity.income, 0);
                        activity.depreciation = 0;
                        activity.depletion = 0;
                        activity.amortization = 0;
                        activity.carried.regular = 0;
                        activity.carried.alternative = 0;
                    }
                }
                for (kind, total) in [
                    (crate::EstateActivityKind::Business, &mut data.activities.box6),
                    (crate::EstateActivityKind::RentalRealEstate, &mut data.activities.box7),
                    (crate::EstateActivityKind::OtherRental, &mut data.activities.box8),
                ] {
                    *total = data
                        .activities
                        .activities
                        .iter()
                        .filter(|a| a.kind == kind)
                        .map(|a| a.income)
                        .sum();
                }
                // Keep source classification on the working copy: Form 4952
                // must still know when an unentered code E limits its benefit.
                for item in &data.unentered {
                    if let Some(key) = item.benefit_notice_key() {
                        notice(notes, key, None, &context);
                    }
                }
                if data.unentered.contains(&crate::EstateTrustK1Item::Credits) {
                    notice(notes, "WARN_ESTATE_UNENTERED_CREDIT_LEFT_OUT", None, &context);
                    data.unentered.retain(|item| *item != crate::EstateTrustK1Item::Credits);
                }
                let activity_classes = data.activities.activities.clone();
                data.reported_credits.retain(|credit| {
                    if crate::estate_trust::credit_identity(&credit.code).is_none() {
                        notice_args(
                            notes,
                            "WARN_ESTATE_CREDIT_LEFT_OUT",
                            alloc::vec![MessageArg::Text(
                                crate::estate_trust::credit_letters(&credit.code).into()
                            )],
                            &context,
                        );
                        return false;
                    }
                    if crate::estate_trust::credit_passive(
                        activity_classes.as_slice(),
                        &credit.activities,
                    )
                    .is_none()
                    {
                        notice_args(
                            notes,
                            "WARN_ESTATE_CREDIT_ACTIVITY_LEFT_OUT",
                            alloc::vec![MessageArg::Text(
                                crate::estate_trust::credit_letters(&credit.code).into()
                            )],
                            &context,
                        );
                        return false;
                    }
                    true
                });
                if let Some(foreign) = &data.foreign_statement
                    && foreign.treatment == crate::EstateForeignTreatment::RequiresAllocation
                {
                    notice(notes, "WARN_ESTATE_FOREIGN_ALLOCATION_LEFT_OUT", None, &context);
                    data.foreign_statement = None;
                }
                if let Some(finals) = &data.final_character {
                    if !data.final_successor() {
                        notice(notes, "WARN_ESTATE_FINAL_ENTITLEMENT_LEFT_OUT", None, &context);
                        data.final_67e_deduction = 0;
                        data.final_short_carry = 0;
                        data.final_long_carry = 0;
                        data.final_personal_property_tax = 0;
                        data.final_real_estate_tax = 0;
                        data.final_state_income_tax = 0;
                        data.final_state_sales_tax = 0;
                        data.final_nol = None;
                        data.final_character = None;
                    } else {
                        for item in &finals.additional {
                            if let crate::EstateFinalDeduction::Other { description } = item {
                                notice_args(
                                    notes,
                                    "WARN_ESTATE_FINAL_OTHER_LEFT_OUT",
                                    alloc::vec![MessageArg::Text(description.clone())],
                                    &context,
                                );
                            }
                        }
                    }
                }
                // Without a timely Form 1041-T the payment stays the
                // fiduciary's.
                if data.allocated_estimated_tax > 0 && !data.form_1041t_timely {
                    notice(
                        notes,
                        "WARN_ESTATE_TRUST_PAYMENT_LEFT_OUT",
                        Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                        &context,
                    );
                    data.allocated_estimated_tax = 0;
                }
                if let Some(nol) = &data.final_nol
                    && !nol.fiduciary_history_complete
                {
                    notice(
                        notes,
                        "WARN_ESTATE_TRUST_NOL_LEFT_OUT",
                        Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                        &context,
                    );
                    data.final_nol = None;
                }
                *stored = crate::EstateTrustSource::from_record(&record);
            }
            FederalIncome::Scorp { data } if !crate::ScorpK1::from(&*data).passive.complete => {
                notice(
                    notes,
                    "WARN_SCORP_PASSIVE_HISTORY_LEFT_OUT",
                    Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                    &context,
                );
                data.personal.exact.passive_complete = None;
                drop_scorp_passive_history(data);
            }
            FederalIncome::NuaSale { data } => {
                // The supported working copy omits the uncomputed deduction.
                if let Some(inheritance) = data.inheritance.as_mut() {
                    if inheritance.estate_tax {
                        notice(
                            notes,
                            "WARN_NUA_ESTATE_TAX_LEFT_OUT",
                            Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                            &context,
                        );
                    }
                    inheritance.estate_tax = false;
                }
            }

            FederalIncome::Retirement { data: source } => {
                // An unconfirmed temporary child annuity recovers no cost.
                if let Some(T::ChildAnnuity { data: child }) =
                    &crate::FederalRetirement::from(&*source).taxable
                    && !child.confirmed
                {
                    notice(
                        notes,
                        "WARN_RETIREMENT_BASIS_LEFT_OUT",
                        Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                        &context,
                    );
                    source.tax_payer_amount();
                }
            }
            _ => {}
        }
    }
    investment_credits(d, notes);
    if let Some(adjustments) = d.adjustments.as_mut()
        && let crate::SocialElection::Compare { complete, .. } = adjustments.social_election
        && !complete
    {
        // The regular method: the lump-sum election only lowers tax.
        notice(
            notes,
            "WARN_SOCIAL_LUMP_SUM_ELECTION_LEFT_OUT",
            None,
            &IssueContext::Section(TaxSection::Retirement),
        );
        adjustments.social_election = crate::SocialElection::Regular;
    }
    if let Some(iras) = d.adjustments.as_mut().map(|a| &mut a.iras) {
        for (owner, facts) in [
            (crate::Owner::Taxpayer, Some(&mut iras.taxpayer)),
            (crate::Owner::Spouse, iras.spouse.as_mut()),
        ]
        .into_iter()
        .filter_map(|(owner, facts)| facts.map(|facts| (owner, facts)))
        {
            let context = IssueContext::owner(TaxSection::IraContributions, owner);
            // A 2027 deposit not designated for 2026 belongs to 2027.
            if let Some(deposits) = &facts.next_year_deposits
                && !deposits.designation_confirmed
                && deposits.traditional <= facts.traditional
                && deposits.roth <= facts.roth
            {
                notice(notes, "WARN_IRA_NEXT_YEAR_DEPOSIT_LEFT_OUT", None, &context);
                facts.traditional -= deposits.traditional;
                facts.roth -= deposits.roth;
                facts.next_year_deposits = None;
            }
        }
    }
    if let Some(deductions) = d.deductions.as_ref() {
        for (i, loan) in deductions.mortgage_loans().unwrap_or_default().iter().enumerate() {
            if matches!(&loan.refund_of_interest, crate::MortgageInterestRefund::Yes { origin: Some(crate::MortgageRefundOrigin::Deducted(w)), .. } if !w.ordinary)
            {
                let context = IssueContext::Section(TaxSection::Mortgage).item("mortgage.loans", i);
                notice(
                    notes,
                    "WARN_RECOVERY_LEFT_OUT",
                    Some(display_label(&context, &loan.property)),
                    &context,
                );
            }
        }
    }
    if let Some(credits) = d.credits.as_mut()
        && credits.tip_fica_carryforward.as_ref().is_some_and(|carry| !carry.ordinary)
    {
        notice(
            notes,
            "WARN_TIP_FICA_CARRYFORWARD_LEFT_OUT",
            None,
            &IssueContext::Section(TaxSection::MoreCredits),
        );
        credits.tip_fica_carryforward = None;
    }
    if let Some(claim) = d.credits.as_mut().and_then(|c| c.mortgage_credit.as_mut())
        && let Some(certificate) = &claim.current
    {
        use crate::MortgageCertificateHistory as H;
        let reissue = match &certificate.history {
            H::Original | H::Unreviewed => true,
            H::ReissuedBefore2026 { facts }
            | H::ReissuedDuring2026SameRate { facts, .. }
            | H::ReissuedDuring2026DifferentRates { facts, .. } => facts.answered(),
        };
        if !certificate.eligible || !reissue {
            notice(
                notes,
                "WARN_MORTGAGE_CREDIT_LEFT_OUT",
                None,
                &IssueContext::Section(TaxSection::MoreCredits),
            );
            claim.current = None;
        }
    }
    if let Some(deductions) = d.deductions.as_mut() {
        // Qualification waits for every printed treatment; unresolved gifts stay
        // in the calculation so their named source boundary remains visible.
        if let Ok(all) = deductions.gifts() {
            let documents = &deductions.vehicle_gift_documents;
            retain_scoped(
                &mut deductions.property_gifts,
                &IssueContext::Section(TaxSection::GiftCarryovers),
                "deductions.property_gifts",
                |gift, context| {
                    let kept =
                        gift.qualified(&all, &deductions.appraisals, deductions.basis_election);
                    if !kept {
                        notice(
                            notes,
                            "WARN_GIFT_LEFT_OUT",
                            Some(display_label(context, &gift.description)),
                            context,
                        );
                    }
                    kept
                },
            );
            retain_scoped(
                &mut deductions.vehicle_gifts,
                &IssueContext::Section(TaxSection::GiftCarryovers),
                "deductions.vehicle_gifts",
                |vehicle, context| {
                    let Some(document) =
                        documents.iter().find(|document| document.id == vehicle.id)
                    else {
                        return true;
                    };
                    let Ok(gift) = crate::PropertyGift::try_from((&*vehicle, &document.form))
                    else {
                        return true;
                    };
                    let kept =
                        gift.qualified(&all, &deductions.appraisals, deductions.basis_election);
                    if !kept {
                        notice(
                            notes,
                            "WARN_GIFT_LEFT_OUT",
                            Some(display_label(context, &gift.description)),
                            context,
                        );
                    }
                    kept
                },
            );
            let vehicles = &deductions.vehicle_gifts;
            deductions
                .vehicle_gift_documents
                .retain(|document| vehicles.iter().any(|vehicle| vehicle.id == document.id));
        }
        retain_scoped(
            &mut deductions.charity_carryovers,
            &IssueContext::Section(TaxSection::GiftCarryovers),
            "deductions.charity_carryovers",
            |carry, context| {
                if !carry.same_filers {
                    notice(notes, "WARN_CHARITY_CARRY_LEFT_OUT", None, context);
                }
                carry.same_filers
            },
        );
    }
    section179_without_spouse_records(d, notes);
    farm_rental_section_179(d, notes);
}

// Section 179 needs the active conduct of a trade or business (Reg.
// 1.179-2(c)(6)); whether a landlord paid in crop shares meets it is a facts
// question the App does not decide. A farm rental's election is zeroed and
// its property depreciated instead; a carryforward waits unused.
fn farm_rental_section_179(d: &mut FederalDraft, notes: &mut ScopeNotes<'_>) {
    use crate::{EquipmentHistory as H, Section179 as S};
    for entry in &mut d.income {
        let context = crate::interview::income_issue_context(entry);
        let FederalIncome::FarmRental { data } = &mut entry.income else {
            continue;
        };
        let mut changed = false;
        for asset in &mut data.assets {
            match &mut asset.history {
                H::New { section179: S::Eligible { elected }, .. } if *elected > 0 => {
                    *elected = 0;
                    changed = true;
                }
                H::Continuing { carryforward, .. } if *carryforward > 0 => changed = true,
                _ => {}
            }
        }
        if changed {
            notice(
                notes,
                "WARN_FARM_RENTAL_SECTION_179_LEFT_OUT",
                Some(MessageArg::Nested(alloc::boxed::Box::new(context.label()))),
                &context,
            );
        }
    }
}

// A separate return without spouse section 179 records takes no
// section 179 this year (no share of the dollar limit, section179::facts):
// 2026 elections become ordinary depreciation of the same assets and
// carryforwards wait for 2027. A K-1 section 179 amount still refuses there.
fn section179_without_spouse_records(d: &mut FederalDraft, notes: &mut ScopeNotes<'_>) {
    let context = IssueContext::Business;
    use crate::{EquipmentHistory as H, Section179 as S};
    if !d.setup.as_ref().is_some_and(|setup| {
        matches!(
            setup.status,
            crate::FilingChoice::Separate { section179: crate::Section179Spouse::NotSupplied }
        )
    }) || d.income.iter().any(|entry| match &entry.income {
        FederalIncome::Partnership { data } => {
            data.section179 > 0 || data.regular.carried.section179 > 0
        }
        FederalIncome::Scorp { data } => {
            let data = crate::ScorpK1::from(data);
            data.section179 > 0 || data.carried.section179 > 0
        }
        _ => false,
    }) {
        return;
    }
    let mut changed = d.income.iter().any(|entry| match &entry.income {
        FederalIncome::Partnership { data } => data.section179_carryover > 0,
        FederalIncome::Scorp { data } => crate::ScorpK1::from(data).section179_carryover > 0,
        _ => false,
    });
    for asset in d
        .income
        .iter_mut()
        .filter_map(|entry| {
            if let FederalIncome::Business { data } = &mut entry.income {
                Some(&mut data.assets)
            } else {
                None
            }
        })
        .flatten()
        .chain(
            d.adjustments
                .iter_mut()
                .flat_map(|a| &mut a.employee_expenses)
                .flat_map(|claim| &mut claim.equipment_assets),
        )
    {
        match &mut asset.history {
            H::New { section179: S::Eligible { elected }, .. } if *elected > 0 => {
                *elected = 0;
                changed = true;
            }
            H::Continuing { carryforward, .. } if *carryforward > 0 => changed = true,
            _ => {}
        }
    }
    if changed {
        notice(notes, "WARN_SECTION_179_SPOUSE_RECORDS_LEFT_OUT", None, &context);
    }
}

// Named source gaps affect only the unestablished benefit. The accepted farm
// and its routed receipts remain in the working return. Required disposition
// basis is checked by the owning sale, rather than removed as a benefit.
fn farm_named_benefits(d: &mut FederalDraft, notes: &mut ScopeNotes<'_>) {
    let mut averaging_gap = false;
    for entry in &mut d.income {
        let context = crate::interview::income_issue_context(entry);
        let (farm, assets) = match &mut entry.income {
            FederalIncome::Business { data } => match &mut data.activity {
                BusinessActivity::Farming { data: farm } => (farm, &mut data.assets),
                _ => continue,
            },
            FederalIncome::FarmRental { data } => (&mut data.farm, &mut data.assets),
            _ => continue,
        };
        if farm.specials.limitations.contains(&5) && farm.conservation.is_some() {
            farm.conservation = None;
            notice(notes, "WARN_FARM_REVISED_ASSESSMENT_OMITTED", None, &context);
        }
        if farm.specials.limitations.contains(&7) {
            // Unidentified history cannot support a property deduction. An
            // actual disposal remains present so its mandatory gain is not
            // lost along with an unclaimed deduction.
            let deduction_only =
                assets.iter().all(|asset| matches!(asset.disposal, crate::AssetDisposal::No));
            if deduction_only {
                assets.clear();
            }
            notice(
                notes,
                if deduction_only {
                    "WARN_FARM_PROPERTY_HISTORY_OMITTED"
                } else {
                    "NOTE_FARM_PROPERTY_DISPOSITION_SOURCE"
                },
                None,
                &context,
            );
        }
        averaging_gap |= farm.specials.limitations.contains(&9);
    }
    if averaging_gap
        && let Some(screening) = &mut d.screening
        && screening.farm_averaging.take().is_some()
    {
        notice(
            notes,
            "WARN_FARM_AVERAGING_HISTORY_OMITTED",
            None,
            &IssueContext::Section(TaxSection::Farming),
        );
    }
}

fn estate_loss_sources(d: &mut FederalDraft, notes: &mut ScopeNotes<'_>) {
    for entry in &mut d.income {
        let context = crate::interview::income_issue_context(entry);
        let FederalIncome::EstateTrustPortfolio { data } = &mut entry.income else {
            continue;
        };
        let Some(nol) = &mut data.statements.final_nol else {
            continue;
        };
        if !nol.valid() {
            continue;
        }
        let periods = &nol.taxable_periods;
        nol.years.retain(|y| {
            if crate::FinalEstateTrustNol::post_2017(y, periods).is_none()
                || (crate::FinalEstateTrustNol::post_2017(y, periods) == Some(false)
                    && y.remaining_years.is_none())
            {
                notice_args(
                    notes,
                    "WARN_ESTATE_NOL_PERIOD_LEFT_OUT",
                    alloc::vec![MessageArg::Number(y.year as i64)],
                    &context,
                );
                false
            } else {
                true
            }
        });
        if nol.years.is_empty() {
            data.statements.final_nol = None;
        }
    }
    let conflicts = crate::estate_trust::loss_conflicts(d);
    for entry in &mut d.income {
        let context = crate::interview::income_issue_context(entry);
        let FederalIncome::EstateTrustPortfolio { data } = &mut entry.income else {
            continue;
        };
        let Some(nol) = &mut data.statements.final_nol else {
            continue;
        };
        if !nol.valid() {
            continue;
        }
        for y in &mut nol.years {
            if let Some(&(regular, alternative)) = conflicts.get(&(
                entry.id,
                y.year,
                y.original_period.as_ref().map_or(0, |p| p.began),
                y.original_period.as_ref().map_or(0, |p| p.ended),
            )) {
                notice_args(
                    notes,
                    "WARN_ESTATE_NOL_POOL_LEFT_OUT",
                    alloc::vec![MessageArg::Number(y.year as i64)],
                    &context,
                );
                if regular {
                    y.regular_balance = 0;
                }
                if alternative {
                    y.alternative_balance = 0;
                }
            }
        }
        nol.years.retain(|y| y.regular_balance != 0 || y.alternative_balance != 0);
        if nol.years.is_empty() {
            data.statements.final_nol = None;
        }
    }
}

// An unconfirmed passive history leaves out the passive loss carryover and the rental
// activities' carryovers (boxes 2 and 3).
fn drop_scorp_passive_history(source: &mut crate::ScorpSource) {
    if let crate::K1ScorpStart::Continuing { carried, .. } = &mut source.personal.basis.start {
        carried.passive_loss = 0;
        carried.rental_real_estate_passive_loss = 0;
        carried.other_rental_passive_loss = 0;
        carried.rental_real_estate_section1231_passive_loss = 0;
        carried.other_rental_section1231_passive_loss = 0;
    }
}

// Pub. 559: suspended S-corporation losses do not carry onto the shareholder's final return.
fn drop_scorp_suspended_history(source: &mut crate::ScorpSource) {
    if let crate::K1ScorpStart::Continuing { carried, .. } = &mut source.personal.basis.start {
        carried.basis_loss = 0;
        carried.risk_loss = 0;
        carried.passive_loss = 0;
        carried.basis_short = 0;
        carried.basis_long = 0;
        carried.basis_collectibles = 0;
        carried.risk_short = 0;
        carried.risk_long = 0;
        carried.risk_collectibles = 0;
        carried.section1231_basis_loss = 0;
        carried.section1231_risk_loss = 0;
        carried.section1231_passive_loss = 0;
        carried.rental_real_estate_passive_loss = 0;
        carried.other_rental_passive_loss = 0;
        carried.rental_real_estate_section1231_passive_loss = 0;
        carried.other_rental_section1231_passive_loss = 0;
    }
    if let Some(crate::ScorpAmtRecords::Separate { record }) = &mut source.personal.amt {
        record.rentals.drop_passive_history();
    }
}
