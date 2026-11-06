use crate::business_credit_source::BusinessCredit;
use crate::common::{CalculationLine, LineKind, Message, MessageArg, ResultState};
use crate::diagnostics::contextual_message;
use crate::form_lines::{FormLine, FormLineKind, TaxForm};
use crate::return_state::{FederalAmount, FederalEstimate, FederalResult, FederalReviewNoteView};
use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::String,
    vec::Vec,
};

impl From<FederalResult> for FederalEstimate {
    fn from(r: FederalResult) -> Self {
        Self {
            state: r.state,
            revision: r.revision,
            review_notes: r
                .review_notes
                .into_iter()
                .map(|note| FederalReviewNoteView {
                    message: note
                        .context
                        .as_ref()
                        .map_or_else(
                            || note.display.clone(),
                            |context| contextual_message(&note.display, context),
                        )
                        .encode(),
                })
                .collect(),
            method: Message::new("ESTIMATE_METHOD_AND_LIMITATIONS", alloc::vec![]).encode(),
            // Every calculation step, shown as its label, amount and
            // explanation; an unsupported result shows none.
            breakdown: if r.state == ResultState::Unsupported {
                Vec::new()
            } else {
                r.lines
                    .iter()
                    .map(|l| {
                        let count = l.kind != LineKind::Money;
                        FederalAmount {
                            label: counted_label(line_label(&l.id), l).encode(),
                            amount: if count { 0 } else { l.amount },
                            explanation: l.explanation.encode(),
                            form_line: l.form_line.clone(),
                            money: (!count).then_some(l.amount),
                        }
                    })
                    .collect()
            },
            return_lines: r.return_lines,
            tax: r.tax,
            farmland_installments: r.farmland_installments,
            refundable_credits: r.refundable_credits,
            payments: r.payments,
            refund: r.refund,
            balance: r.balance,
        }
    }
}

impl FormLine {
    // A calculated draft return row's label, "<form name> <line> — <label>"
    // ("Schedule C 31 — Business 2 net profit"): the form's name and printed
    // line first, then the id's own label (ROW_LABEL), or ROW_<pattern> of the
    // few ids whose row has its own wording (`ROW_OVERRIDES`).
    pub fn row_label(&self, id: &str) -> Message {
        let keys = IdKeys::new(id);
        #[cfg(test)]
        debug_assert!(keys.registered(), "no return row resource for calculation id {id}");
        let mut all = alloc::vec![
            MessageArg::Resource(self.form.name_key().into()),
            MessageArg::Text(self.line.clone()),
        ];
        if keys.row_override() {
            all.extend(keys.args);
            Message::new(keys.row, all)
        } else {
            all.push(MessageArg::Nested(alloc::boxed::Box::new(Message::new(
                keys.line, keys.args,
            ))));
            Message::new("ROW_LABEL", all)
        }
    }
}

// A calculation id's label (LINE_<pattern>): the id's words, each numbered
// or named part ("business_2", a child's name) an argument.
pub fn line_label(id: &str) -> Message {
    let keys = IdKeys::new(id);
    #[cfg(test)]
    debug_assert!(keys.registered(), "no label resource for calculation id {id}");
    Message::new(keys.line, keys.args)
}

/// The resource keys of one calculation id, built in one place: the label
/// (`LINE_`), the review row (`ROW_`, only for `ROW_OVERRIDES`) and the explanation (`EXPLAIN_`, which
/// only some ids have) of its pattern, and the arguments its numbered or named
/// parts pass. A new id adds the `LINE_` resource and the others named
/// here; the tests derive every one of them from the `LINE_` keys.
pub(crate) struct IdKeys {
    pub pattern: String,
    pub line: String,
    pub row: String,
    #[cfg_attr(not(test), allow(dead_code))]
    pub explain: String,
    pub args: Vec<MessageArg>,
}

impl IdKeys {
    pub(crate) fn new(id: &str) -> Self {
        let (pattern, args) = id_pattern(id);
        let upper = pattern.to_uppercase();
        Self {
            line: alloc::format!("LINE_{upper}"),
            row: alloc::format!("ROW_{upper}"),
            explain: alloc::format!("EXPLAIN_{upper}"),
            pattern,
            args,
        }
    }

    /// Whether the id's return row has its own wording (`ROW_<pattern>`).
    pub(crate) fn row_override(&self) -> bool {
        crate::line_labels::ROW_OVERRIDES.binary_search(&self.pattern.as_str()).is_ok()
    }

    /// Whether the id's pattern has its `LINE_` resource in the topic catalogs.
    #[cfg(test)]
    pub(crate) fn registered(&self) -> bool {
        crate::APP_STRINGS.iter().any(|(key, _)| *key == self.line)
    }
}
// An id's pattern: its '_' parts, a number or a part that is not a lower-case
// word replaced by N and passed as a text argument. The rows of a general
// business credit (`business_credit_<credit>_<year>_<kind>`) share one pattern
// per kind (`business_credit_credit_N_<kind>`): the credit is the first
// argument, as the resource naming its form, so a new credit needs no label.
pub fn id_pattern(id: &str) -> (String, Vec<MessageArg>) {
    // Rental equipment uses the same native asset report as C/F equipment.
    // Reuse its existing row resources for those reports; rental-specific
    // depreciation rows retain their own established resource patterns.
    if let Some((source, asset)) = id.strip_prefix("rental_").and_then(|v| v.split_once("_asset_"))
        && let Some((index, suffix)) = asset.split_once('_')
        && (suffix.starts_with("disposal_")
            || suffix.starts_with("section179_")
            || suffix.starts_with("partial_"))
    {
        return id_pattern(&alloc::format!("business_{source}_asset_{index}_{suffix}"));
    }
    if let Some((source, suffix)) = id.strip_prefix("rental_").and_then(|v| v.split_once("_sale_"))
        && matches!(suffix, "ordinary_activity_adjustment" | "section1231_activity_adjustment")
    {
        return id_pattern(&alloc::format!("rental_{source}_sale_at_risk_adjustment"));
    }
    if let Some((home, rest)) = id.strip_prefix("home_").and_then(|id| id.split_once("_owner_"))
        && let Some((owner, tail)) = rest.split_once('_')
        && matches!(owner, "0" | "1")
    {
        let (tail, mut tail_args) = id_pattern(tail);
        let mut args = alloc::vec![
            MessageArg::Text(home.into()),
            MessageArg::Resource(if owner == "0" { "PERSON_YOU" } else { "PERSON_SPOUSE" }.into())
        ];
        args.append(&mut tail_args);
        return (alloc::format!("home_N_owner_N_{tail}"), args);
    }
    if let Some((credit, rest)) =
        id.strip_prefix("business_credit_").and_then(BusinessCredit::split_id)
    {
        if let Some(rest) = rest.strip_prefix("source_")
            && let Some((source, rest)) = rest.split_once("_filer_")
            && let Some((filer, tail)) = rest.split_once('_')
        {
            let (tail, mut tail_args) = id_pattern(tail);
            let mut args = alloc::vec![
                MessageArg::Resource(credit.form_message_key().into()),
                MessageArg::Text(source.into()),
                MessageArg::Resource(
                    match filer {
                        "1" => "CARRY_FILER_YOU",
                        "2" => "CARRY_FILER_SPOUSE",
                        _ => "CARRY_FILER_SAME",
                    }
                    .into()
                ),
            ];
            args.append(&mut tail_args);
            return (alloc::format!("business_credit_credit_source_N_filer_N_{tail}"), args);
        }
        let (tail, mut tail_args) = id_pattern(rest);
        let mut args = alloc::vec![MessageArg::Resource(credit.form_message_key().into())];
        args.append(&mut tail_args);
        return (alloc::format!("business_credit_credit_{tail}"), args);
    }
    let mut pattern = String::new();
    let mut args = Vec::new();
    for part in id.split('_').filter(|p| !p.is_empty()) {
        if !pattern.is_empty() {
            pattern.push('_');
        }
        let word = part.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
            && !part.bytes().all(|b| b.is_ascii_digit());
        if word {
            pattern.push_str(part);
        } else {
            pattern.push('N');
            args.push(MessageArg::Text(part.into()));
        }
    }
    (pattern, args)
}

impl TaxForm {
    // The resource that names the form (FORM_<variant>).
    pub fn name_key(self) -> &'static str {
        match self {
            Self::Form1040 => "FORM_FORM1040",
            Self::Schedule1 => "FORM_SCHEDULE1",
            Self::Schedule1A => "FORM_SCHEDULE1A",
            Self::Schedule2 => "FORM_SCHEDULE2",
            Self::Schedule3 => "FORM_SCHEDULE3",
            Self::Schedule8812 => "FORM_SCHEDULE8812",
            Self::ScheduleA => "FORM_SCHEDULEA",
            Self::ScheduleB => "FORM_SCHEDULEB",
            Self::ScheduleC => "FORM_SCHEDULEC",
            Self::ScheduleD => "FORM_SCHEDULED",
            Self::ScheduleE => "FORM_SCHEDULEE",
            Self::ScheduleEic => "FORM_SCHEDULEEIC",
            Self::ScheduleF => "FORM_SCHEDULEF",
            Self::ScheduleH => "FORM_SCHEDULEH",
            Self::ScheduleJ => "FORM_SCHEDULEJ",
            Self::ScheduleR => "FORM_SCHEDULER",
            Self::ScheduleSe => "FORM_SCHEDULESE",
            Self::Form982 => "FORM_FORM982",
            Self::Form1040Es => "FORM_FORM1040ES",
            Self::Form1062 => "FORM_FORM1062",
            Self::Form1116 => "FORM_FORM1116",
            Self::Form1116ScheduleB => "FORM_FORM1116_SCHEDULE_B",
            Self::Form2439 => "FORM_FORM2439",
            Self::Form2106 => "FORM_FORM2106",
            Self::Form2441 => "FORM_FORM2441",
            Self::Form2555 => "FORM_FORM2555",
            Self::Form3468 => "FORM_FORM3468",
            Self::Form3800 => "FORM_FORM3800",
            Self::Form4136 => "FORM_FORM4136",
            Self::Form4137 => "FORM_FORM4137",
            Self::Form4255 => "FORM_FORM4255",
            Self::Form4562 => "FORM_FORM4562",
            Self::Form4562B => "FORM_FORM4562B",
            Self::Form4684 => "FORM_FORM4684",
            Self::Form4797 => "FORM_FORM4797",
            Self::Form4835 => "FORM_FORM4835",
            Self::Form4952 => "FORM_FORM4952",
            Self::Form4972 => "FORM_FORM4972",
            Self::Form5329 => "FORM_FORM5329",
            Self::Form5695 => "FORM_FORM5695",
            Self::Form5884 => "FORM_FORM5884",
            Self::Form6198 => "FORM_FORM6198",
            Self::Form6251 => "FORM_FORM6251",
            Self::Form6252 => "FORM_FORM6252",
            Self::Form6781 => "FORM_FORM6781",
            Self::Form7203 => "FORM_FORM7203",
            Self::Form7206 => "FORM_FORM7206",
            Self::Form8283 => "FORM_FORM8283",
            Self::Form8379 => "FORM_FORM8379",
            Self::Form8396 => "FORM_FORM8396",
            Self::Form8582 => "FORM_FORM8582",
            Self::Form8582Cr => "FORM_FORM8582CR",
            Self::Form8606 => "FORM_FORM8606",
            Self::Form8615 => "FORM_FORM8615",
            Self::Form8814 => "FORM_FORM8814",
            Self::Form8801 => "FORM_FORM8801",
            Self::Form8815 => "FORM_FORM8815",
            Self::Form8824 => "FORM_FORM8824",
            Self::Form8829 => "FORM_FORM8829",
            Self::Form8834 => "FORM_FORM8834",
            Self::Form8839 => "FORM_FORM8839",
            Self::Form8846 => "FORM_FORM8846",
            Self::Form8853 => "FORM_FORM8853",
            Self::Form8859 => "FORM_FORM8859",
            Self::Form8862 => "FORM_FORM8862",
            Self::Form8863 => "FORM_FORM8863",
            Self::Form8880 => "FORM_FORM8880",
            Self::Form8881 => "FORM_FORM8881",
            Self::Form8889 => "FORM_FORM8889",
            Self::Form8911 => "FORM_FORM8911",
            Self::Form8919 => "FORM_FORM8919",
            Self::Form8936 => "FORM_FORM8936",
            Self::Form8941 => "FORM_FORM8941",
            Self::Form8949 => "FORM_FORM8949",
            Self::Form8958 => "FORM_FORM8958",
            Self::Form8959 => "FORM_FORM8959",
            Self::Form8960 => "FORM_FORM8960",
            Self::Form8962 => "FORM_FORM8962",
            Self::Form8978 => "FORM_FORM8978",
            Self::Form8994 => "FORM_FORM8994",
            Self::Form8995 => "FORM_FORM8995",
            Self::Form8995A => "FORM_FORM8995A",
            Self::FormW2 => "FORM_FORMW2",
            Self::FormW2G => "FORM_FORMW2G",
            Self::Form1095A => "FORM_FORM1095A",
            Self::Form1098 => "FORM_FORM1098",
            Self::Form1098E => "FORM_FORM1098E",
            Self::Form1098T => "FORM_FORM1098T",
            Self::Form1099B => "FORM_FORM1099B",
            Self::Form1099C => "FORM_FORM1099C",
            Self::Form1099Div => "FORM_FORM1099DIV",
            Self::Form1099G => "FORM_FORM1099G",
            Self::Form1099Int => "FORM_FORM1099INT",
            Self::Form1099K => "FORM_FORM1099K",
            Self::Form1099Misc => "FORM_FORM1099MISC",
            Self::Form1099Nec => "FORM_FORM1099NEC",
            Self::Form1099Q => "FORM_FORM1099Q",
            Self::Form1099R => "FORM_FORM1099R",
            Self::Form1099Sa => "FORM_FORM1099SA",
            Self::Form5498 => "FORM_FORM5498",
            Self::ScheduleK1Form1041 => "FORM_SCHEDULEK1FORM1041",
            Self::ScheduleK1Form1065 => "FORM_SCHEDULEK1FORM1065",
            Self::ScheduleK1Form1120S => "FORM_SCHEDULEK1FORM1120S",
            Self::Form8912 => "FORM_8912",
            Self::Form8915F => "FORM_FORM8915F",
        }
    }
    // The review's form order: Form 1040, Schedules 1, 2, 3, A, B, C, D, E
    // and SE, then every other form in declaration order.
    fn review_group(self) -> u8 {
        match self {
            Self::Form1040 => 0,
            Self::Schedule1 => 1,
            Self::Schedule2 => 2,
            Self::Schedule3 => 3,
            Self::ScheduleA => 4,
            Self::ScheduleB => 5,
            Self::ScheduleC => 6,
            Self::ScheduleD => 7,
            Self::ScheduleE => 8,
            Self::ScheduleSe => 9,
            _ => 10,
        }
    }
}

// The lines each form carries in a computed return: every typed return line
// and calculation line, grouped by form. Coverage is derived from this rather
// than maintained as prose.
pub fn form_coverage(result: &FederalResult) -> BTreeMap<TaxForm, BTreeSet<String>> {
    let mut coverage: BTreeMap<TaxForm, BTreeSet<String>> = BTreeMap::new();
    let typed = result
        .return_lines
        .iter()
        .filter_map(|line| line.form_line.as_ref())
        .chain(result.lines.iter().filter_map(|line| line.form_line.as_ref()));
    for form_line in typed {
        coverage.entry(form_line.form).or_default().insert(form_line.line.clone());
    }
    coverage
}

// The review's calculated draft return: every line the computed return
// carries, grouped by form (TaxForm::review_group). Form 1040's typed lines
// keep their order and zero amounts; every other form lists its calculation
// lines, labelled with their calculation id, one copy at a time (a copy is
// the business, rental property or person the id names, "business_2_...",
// "rental_3_...", "spouse_...", in the order the return first reaches it;
// lines naming none form one copy) and in printed line order within a copy
// (FormLine::order), dropping zero amounts. A calculation line for a typed
// return line is not repeated, while two calculation lines on one form line
// (two businesses' Schedule C) are both listed.
// A review row's sort key: form group, form, copy rank, printed line order
// and the return's order.
type ReviewKey = (u8, TaxForm, usize, (u32, String), usize);
// A count line's label carries its number as a typed Count argument (a number
// of people, not dollars); a money line's label is unchanged.
fn counted_label(label: Message, line: &CalculationLine) -> Message {
    match line.kind {
        LineKind::Money => label,
        LineKind::Count => Message::new(
            "DRAFT_ROW_WITH_COUNT",
            alloc::vec![
                MessageArg::Nested(alloc::boxed::Box::new(label)),
                MessageArg::Count(line.amount / 100),
            ],
        ),
        // A ratio's number is a typed decimal argument of the label.
        LineKind::Ratio => Message::new(
            "DRAFT_ROW_WITH_RATIO",
            alloc::vec![
                MessageArg::Nested(alloc::boxed::Box::new(label)),
                MessageArg::Decimal { units: line.amount, scale: 4 },
            ],
        ),
        LineKind::Year => Message::new(
            "DRAFT_ROW_WITH_YEAR",
            alloc::vec![
                MessageArg::Nested(alloc::boxed::Box::new(label)),
                MessageArg::Decimal { units: line.amount / 100, scale: 0 },
            ],
        ),
        LineKind::Percentage => Message::new(
            "DRAFT_ROW_WITH_PERCENTAGE",
            alloc::vec![
                MessageArg::Nested(alloc::boxed::Box::new(label)),
                MessageArg::Decimal { units: line.amount, scale: 2 },
            ],
        ),
        LineKind::Text => Message::new(
            "DRAFT_ROW_WITH_TEXT",
            alloc::vec![
                MessageArg::Nested(alloc::boxed::Box::new(label)),
                MessageArg::Nested(alloc::boxed::Box::new(line.explanation.clone())),
            ],
        ),
    }
}
pub fn draft_return(result: &FederalResult) -> Vec<FederalAmount> {
    let mut rows: Vec<(ReviewKey, FederalAmount)> = result
        .return_lines
        .iter()
        .filter(|line| line.form_line.as_ref().is_some_and(|form| form.kind == FormLineKind::Line))
        .enumerate()
        .map(|(index, line)| {
            let form = line.form_line.as_ref().unwrap().form;
            ((form.review_group(), form, 0, (0, String::new()), index), line.clone())
        })
        .collect();
    let typed: BTreeSet<(TaxForm, String)> = rows
        .iter()
        .filter_map(|row| row.1.form_line.as_ref())
        .map(|f| (f.form, f.line.clone()))
        .collect();
    let mut copies: Vec<(TaxForm, &str)> = Vec::new();
    // Indexes of the nonzero count lines among `result.lines`.
    let mut counts: BTreeSet<(TaxForm, usize)> = BTreeSet::new();
    for (index, line) in result.lines.iter().enumerate() {
        let Some(form_line) = &line.form_line else {
            continue;
        };
        if form_line.kind == FormLineKind::Line
            && typed.contains(&(form_line.form, form_line.line.clone()))
        {
            continue;
        }
        let copy = (form_line.form, copy_of(&line.id));
        let rank = copies.iter().position(|c| *c == copy).unwrap_or_else(|| {
            copies.push(copy);
            copies.len() - 1
        });
        // A number of people, not dollars: the count is a typed number
        // argument of the label and the row's amount stays zero.
        if line.kind == LineKind::Text || line.kind != LineKind::Money && line.amount != 0 {
            counts.insert((form_line.form, index));
        }
        rows.push((
            (form_line.form.review_group(), form_line.form, rank, form_line.order(), index),
            FederalAmount {
                label: counted_label(form_line.row_label(&line.id), line).encode(),
                amount: if line.kind != LineKind::Money { 0 } else { line.amount },
                explanation: line.explanation.encode(),
                form_line: Some(form_line.clone()),
                money: (line.kind == LineKind::Money).then_some(line.amount),
            },
        ));
    }
    // These forms print whole dollars. Keep the calculation rows and their
    // cent-precision sums intact until display.
    for (_, row) in &mut rows {
        if row.form_line.as_ref().is_some_and(|line| {
            matches!(
                line.form,
                TaxForm::ScheduleC
                    | TaxForm::ScheduleE
                    | TaxForm::Form4562
                    | TaxForm::Form4562B
                    | TaxForm::Form4684
                    | TaxForm::Form4797
                    | TaxForm::Form4835
                    | TaxForm::Form2106
                    | TaxForm::Form3468
                    | TaxForm::Form5329
                    | TaxForm::Form6198
                    | TaxForm::Form6251
                    | TaxForm::Form8606
                    | TaxForm::Form8846
                    | TaxForm::Form8915F
                    | TaxForm::Form8958
                    | TaxForm::Form8829
            )
        }) && let Some(amount) = row.money
        {
            let amount = crate::kernel::ratio::dollars(amount) * 100;
            row.amount = amount;
            row.money = Some(amount);
        }
    }
    rows.retain(|(key, row)| {
        row.amount != 0
            || key.1 == TaxForm::Form1040
            || row.form_line.as_ref().is_some_and(|line| {
                line.form == TaxForm::Form4562 && line.line == "11"
                    || line.form == TaxForm::Form8919
                        && matches!(line.line.as_str(), "7" | "8" | "9")
            })
            || counts.contains(&(key.1, key.4))
    });
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    rows.into_iter().map(|row| row.1).collect()
}

// The copy of a form a calculation id belongs to: "business_2" for
// "business_2_net_profit", "rental_3" for a rental property's line ("farm_4",
// "royalty_5" and a child's "child_election_6" likewise), "spouse"
// for "spouse_self_employment_tax", and "" for an id naming none of them.
fn copy_of(id: &str) -> &str {
    // Each filer and disaster beginning year owns a separate Form8915-F.
    if let Some(rest) = id.strip_prefix("form_8915_") {
        for who in ["taxpayer", "spouse"] {
            if let Some(tail) = rest.strip_prefix(who).and_then(|s| s.strip_prefix('_')) {
                let digits = tail.bytes().take_while(u8::is_ascii_digit).count();
                if digits > 0 && tail[digits..].starts_with('_') {
                    return &id[.."form_8915_".len() + who.len() + 1 + digits];
                }
            }
        }
    }
    // Inherited basis never combines with personal or another decedent's
    // basis. Preserve those form-copy boundaries while sorting review rows.
    for prefix in ["inherited_ira_", "inherited_roth_"] {
        if let Some(rest) = id.strip_prefix(prefix) {
            let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
            if digits > 0 && rest[digits..].starts_with('_') {
                return &id[..prefix.len() + digits];
            }
        }
    }
    for prefix in ["hsa_", "archer_"] {
        if let Some(rest) = id.strip_prefix(prefix) {
            if rest.starts_with("1_") {
                return "taxpayer";
            }
            if rest.starts_with("2_") {
                return "spouse";
            }
        }
    }
    if let Some(rest) = id.strip_prefix("ira_excess_") {
        for who in ["taxpayer", "spouse"] {
            if rest.starts_with(who) && rest[who.len()..].starts_with('_') {
                return &rest[..who.len()];
            }
        }
    }
    // Each actual prior return has its own Form8801 exclusion worksheet;
    // the shared current-credit limit is projected afterward as its own copy.
    if let Some(rest) = id.strip_prefix("minimum_tax_history_") {
        let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
        if digits > 0 && rest[digits..].starts_with("_line_") {
            return &id[.."minimum_tax_history_".len() + digits];
        }
    }
    // One Form 8814 for each child's election ("child_election_4_tax").
    if let Some(rest) = id.strip_prefix("child_election_") {
        let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
        if digits > 0 && rest[digits..].starts_with('_') {
            return &id[.."child_election_".len() + digits];
        }
    }
    for person in ["taxpayer", "spouse"] {
        if id.starts_with(person) && id[person.len()..].starts_with('_') {
            return person;
        }
    }
    // One Form 1116 and one Schedule B (Form 1116) for each category and tax
    // system ("foreign_general_FR_gross_income", "foreign_amt_passive_8_taxes_total",
    // "foreign_general_regular_2024_carry_opening").
    if let Some(rest) = id.strip_prefix("foreign_") {
        let amt = rest.starts_with("amt_");
        let after = rest.strip_prefix("amt_").unwrap_or(rest);
        for category in ["general", "passive", "branch", "sanctioned"] {
            if let Some(tail) = after.strip_prefix(category)
                && tail.starts_with('_')
            {
                let end = "foreign_".len() + if amt { 4 } else { 0 } + category.len();
                // A Schedule B id names its tax system after the category.
                let system = ["regular", "amt"]
                    .into_iter()
                    .find(|s| tail[1..].starts_with(s) && tail[1 + s.len()..].starts_with('_'));
                return match system {
                    Some(system) => &id[..end + 1 + system.len()],
                    None => &id[..end],
                };
            }
        }
    }
    let mut parts = id.splitn(3, '_');
    match (parts.next(), parts.next()) {
        (Some(kind @ ("business" | "rental" | "farm" | "royalty")), Some(number))
            if !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit()) =>
        {
            &id[..kind.len() + 1 + number.len()]
        }
        _ => "",
    }
}
