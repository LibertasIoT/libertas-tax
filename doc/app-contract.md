# Tax App authoring contract

Read this App-specific contract with the canonical
[Libertas CodeBot guide](https://github.com/LibertasIoT/libertas-doc/blob/main/CODEBOT.md).
It states required App behavior for public contributors; private maintainer
instructions and investigation records are not prerequisites. The
[Rust workflow](codebot/rust-development.md) owns verification steps, and
[COVERAGE.md](../COVERAGE.md) alone owns supported tax scope and evidence limits.
Use synthetic taxpayer data. Tie changed tax rules to their tax year and
owning IRS sources; an on-screen estimate is not a filing-ready return.

## Prototype contract naming

Use one current data model and protocol, with unsuffixed type and variant names.
Backward compatibility is waived for this unreleased App; do not retain
retired payloads, migration branches, or historical ABI tests. Keep tax-year and
accepted-edit revision tracking, which serve tax rules and concurrency rather
than compatibility.

## Identity, storage and cache

Every Request, including entry and preview, carries the hidden
`#[libertas_fixed("current-user")]` `LibertasUser` as its first payload field.
The Hub verifies that identity; route only to that user's `Interview`, and
reject a different identity. Store one latest accepted return per verified user
and actual server-authorized tax year. The current endpoint serves its existing
year; storage isolation does not add a year selector or another year's tax rules.
Follow [protocol.rs](../src/core/protocol.rs),
[interview.rs](../src/federal/interview/mod.rs) and [storage.rs](../src/core/storage.rs).

Load the complete accepted return in one read transaction, with paged reads.
Only a missing header with no return records means fresh answers. Missing
required records, an unknown layout or corrupt data cannot become an empty or
successful return.
Save retains the existing candidate validation and error order. It compares the
persisted cookie and accepted revision in the write transaction. A competing
change discards the candidate, reloads current answers and returns the existing
stale-editor response; it never replays the edit. Changed answers, pruning and
the new revision commit atomically before the reply or cache update. An
unchanged accepted page keeps its revision and performs no write. Running estimates and warm navigation perform no writes.
Opening a fresh editor commits its complete empty return and server cookie
before replying, so an uncached editor can save on its next request. Only the latest accepted revision is kept;
revision numbers do not create accepted-answer history.

Use `LruCache` keyed by `(verified user, tax_year)` with the positive
`cache_bytes` task setting, default 67,108,864 bytes (64 MiB). The budget is the
exact sum of the cached returns' encoded record values. Access promotes a
return; eviction removes the least recently used persisted return, and a miss
reloads it. A return larger than the entire budget still works for its request
and is then dropped. If a Save grows a cached return beyond the budget, remove
that return after commit. The byte budget does not guarantee process memory or
database latency. Never enumerate other users' returns during a request.

The platform's value, transaction-byte and time limits remain binding. A
complete semantic source or section must fit the 4 MiB value limit, including
its stored wrapper. A Save must fit 16 MiB of mutation bytes and
the transaction expiry; exceeding a limit is a Data failure, never truncation,
a smaller tax-source cap or a successful partial Save. In particular an intact
payments section retains its ordered facts and optional presence; very large
waiver text can exceed the value limit. Filed-copy namespaces and their
separate counter are reserved, but this App has no filing or freeze action.

[model.rs](../src/core/model.rs) owns the stable storage roots and release
marker. Keep the original single-u32 marker first and append new stored cases.

All returns remain synthetic until the owner explicitly declares production;
the development reset policy remains available for incompatible synthetic
formats. Normal keyed loading never uses that policy to erase known corrupt
returns. Before production, designate the first published schema baseline and
preserve append-only key/value compatibility and supported older reads.
See [storage regressions](../src/tests/core/storage.rs).

This App uses ordinary Data APIs without the error-status opt-in. Keep read
values and input/stored-value validation, but discard status-only returns:
the Hub terminates the task on a data failure. Do not add read-after-write
verification, retry or separate recovery machinery; follow the shared
[Data API contract](https://github.com/LibertasIoT/libertas-doc/blob/main/reference/protocols.md#persistence-failure-and-the-no-status-api).

## Validation ownership

- Validation is server-only (inventor decision of 2026-10-13, replacing the earlier both-sides
  rule). The Tax App declares no `libertas_validation_rules`, and no `libertas_range_min`/
  `libertas_range_max` expression that reads another field.
  - Clients still enforce ordinary Libertas schema validity: non-nullable fields, valid typed
    values, exact monetary input, Size, Pattern, number bounds and literal ranges.
  - A relationship between fields (withholding above the wages it was taken from) is checked
    once, in the Rust save path. It replies `Problem` with a useful localized key through
    `libertas_formatted_text`.
  - A refusal caused by a submitted field carries its
    [ErrorFieldPath](https://github.com/LibertasIoT/libertas-doc/blob/main/reference/protocols.md#error-field-path),
    so clients highlight that field after the dialog. A refusal with no submitted field cause
    leaves the locator null.
  - Derived fields are recomputed when a page is built and ignored on save.
- Implement tax-specific validation and calculation once in Rust. Return typed
  protocol errors with useful input context on failure; do not duplicate tax
  rules in client-side ValidationRules.
- An unfinished form is not an encodable non-nullable Request. Any preview
  protocol must explicitly model partial answers and unknown computed results.
  Report a missing platform contract before changing the requested behavior
  or encoding.

## Answers and unknowns

1. No takes the no-benefit path: where a Yes/No gate would refuse on No,
   No leaves the benefit out (or applies the added tax) and the estimate
   proceeds. A kept Not sure takes the same conservative path and adds one
   `WARN_*` review note stating the assumption, for example "Assumed no
   prior home-sale exclusion; if you did exclude one, your tax may be higher."
2. A scope confirmation ("no unlisted special treatment", "records
   complete", "ordinary business") answered No shows one per-item
   `WARN_*_LEFT_OUT` notice in the item's own words and leaves that item out
   instead of refusing the whole estimate. `unknowns::supported` removes such
   items from a working copy before the rules run; a rule that keeps part of
   an item decides in place.
3. A two-sided question, where either answer can raise tax, is a required
   Yes/No with no Not sure; its description says how to find the answer and
   both answers compute. An unsupported side follows decision 2.
4. An unknown is assumed conservatively with a warning, never refused.

Types follow from them. A Yes/No-only question is `bool` (`Option<bool>`
where it was `Option<Answer>`) with no Default, so it is an explicit,
initially unselected two-choice; phrase it so true is Yes and add
`[TrueText]`/`[FalseText]` only where Yes/No does not fit. `Answer` remains
only where Not sure is kept and handled. A field no person is ever asked
(always written by its page) is a constant and is removed from the type.
Pages never write Not sure as a placeholder or "unsupported" marker; model
those states explicitly. In the Libertas data model an unanswered required
question cannot reach the App: the client cannot submit a Request until every
required question has an answer, including a Nullable without a Default.
Use the platform's [Nullable choices](https://github.com/LibertasIoT/libertas-doc/blob/main/reference/presentation.md#nullable-choices)
for questions whose positive answer carries details and for optional overrides.
Their field documentation supplies both answer labels; the decision to require
an answer or start on the null answer follows the
[authoring contract](https://github.com/LibertasIoT/libertas-doc/blob/main/reference/app-contract.md#nullable-choices).
A "not known" state reaches the App only as a
declared unknown variant (`NotSure`, `NotSupplied`, `Unresolved`, ...) or as
null in a field whose documented null answer specifically means unknown.
Those unknown answers follow decision 4. Other documented null answers retain
their own negative, not-applicable, no-claim or fallback meaning.
An unknown variant or unknown null means one documented thing: either remove it so the
question is a required choice (when the person can look the answer up), or
keep it as a handled unknown that takes the conservative path with its one
`WARN_*` note; never keep it only to refuse on it. "Not applicable" is its own
variant (or the field is omitted where it does not apply), never merged with
"not known". Refusals remain only for an added tax the App cannot compute or a
whole-return situation outside the App's scope (plus validity and consistency
checks on entered values with no conservative resolution); a warning or
notice never changes `ResultState`.

Use [unknowns.rs](../src/core/unknowns.rs), the shared note helpers in
[federal_rules.rs](../src/federal/federal_rules.rs), and their focused fixtures. Preserve
accepted answers when constructing the supported calculation working copy.

## Money precision

Every money calculation follows one rule, implemented once in `src/kernel/ratio.rs`.

- **Cents through stages.** Source amounts are exact cents. Intermediate stages (limits,
  allocations, suspensions, carryovers, ratios) carry exact cents and never round an operand.
- **Round once per reported amount.** A reported line or category amount rounds once, to whole
  dollars, with `kernel::ratio::dollars`: 50 cents and over round away from zero, under 50 cents
  drop. A total is the sum of its rounded lines, not a rounding of their cents.
- **Splitting a rounded amount.** A rounded joint amount divides between owners or source classes
  only with `project_rounded_pair` (or `project_fraction_pair` for a rational share). The declared
  first part rounds once and the rest takes the exact residual. The order is stated where the split
  is defined, and an amount that fails to foot is Unavailable, never assigned to the residual owner.
- **Several source classes in one share.** When one owner's rounded category amount combines
  sources of different legal classes (for example separate and community withholding), legally
  equivalent sources are combined first. Each class is weighted by its exact source cents as a
  rational weight, and the resulting share is rounded once. Never round per source or per document,
  and never hand the rounding residue to one source.
- **No invented precision.** No ratio, proportion or owner is invented to make figures foot.
  An unknown component stays Unavailable under [Answers and unknowns](#answers-and-unknowns).
- **Cent columns.** A form row that shows source cents (for example Form 8958) keeps them, and its
  documented difference from the rounded line is not moved to one party.

Modules call these helpers. They don't define their own rounding or inline `/ 100` arithmetic.

## Source facts and availability

A source fact is a value the return can know only from an actual document, record or other
return: an origin date, a vintage, an entity period, a prior-year carry, a counterpart's reported
amount. Every source fact follows one pattern.

- **Three states.** A fact is Reported (a value), NotReported (the source shows none, which is a
  real zero or absence), or Unavailable (unknown). Unavailable never becomes zero and is never
  copied from a related figure.
- **Bounds.** A Reported value is checked on Save against its actual bounds, for example a share
  between the owner's loss and zero, or a date inside the entity period.
- **Named boundary.** A calculation that needs an Unavailable fact stops at that point under one
  named boundary notice. The notice says which fact would let the calculation continue. Nothing
  around it is guessed.
- **Optional, never a gate.** A source fact is optional input. It is never a Yes/No gate that only
  unlocks the real question; ask the real question directly.

New source facts use the shared source-fact type and helpers in `src/kernel` rather than a new
ad-hoc `Option` or enum.

## Person attribution

Every amount that belongs to a person carries that ownership in one way.

- **Owner.** Ownership uses the existing choice for that kind of record. A document's owner is its
  physical recipient, taxpayer or spouse. A household ownership fact (a shared deduction, an
  expense or an account) may also be both. Unknown is a separate state, never treated as both.
  An owner is required wherever a per-person split (joint injured spouse, per-person NIIT, Form
  8958, a community-property separate return) needs it, and it is checked on Save. When only one
  owner is possible, the choice is supplied and hidden; it is never a default answer.
- **Shares.** A both-owned or community amount divides by an actual share. A community share
  follows the state's community rule. A split of a rounded amount follows
  [Money precision](#money-precision).
- **Capacities.** A person's share of a joint limit never exceeds that person's own established
  amount. For example, an excess-business-loss share never exceeds that owner's net business loss.
- **Counterparts.** A figure from the other spouse's separate return is a
  [source fact](#source-facts-and-availability) on the owning source.
- **One implementation.** Attribution logic is implemented once and reused by every consumer.

## Source identity and association

Every source record has one durable identity: a stable ID, never a list position or an address.
- **Links between sources** (a component to its physical building, a sale to its property, a K-1
  child to its parent, a credit to its origin facility) are stored as identity references through
  the one association provider. They are never inferred from labels, positions or amounts.
- **Unresolved links:** a missing or ambiguous link is Unavailable under its named boundary.
- **Consumers** read sources through their owner's published view and never reach into another
  owner's records.

## Required answers and page construction

- **A required answer starts unanswered.** No page builder supplies a minimum, a default or a
  placeholder for an answer the person hasn't given. Where a complete EditRequest can't represent
  "unanswered", the page is built through WorkflowRequest construction.
- **Validity lives in the save path.** A constraint the person reaches step by step, such as "at
  least two members", lives in the native save check and stored-record validation, not in the
  Page type. That lets the person build the answer up incrementally.

## Interview pages

Collect documents before interviewing. Income documents collects papers and assigns
unowned papers; topic pages ask only the facts those documents leave unresolved.
Ask each necessary question once, on the page that owns its answer, and name the
associated document by its kind, issuer and printed person. Derive new questions
and pending items from accepted changes so the person never has to repeat a walk.

Use ordinary language and one label for each question beside its typed control.
Do not show internal record numbers, field names such as “value” or “answers”,
position numbers or nested wrapper labels. Keep pages short and split by tax
topic, with each page doing one job. Do not ask a question whose enclosing list
already supplies its answer.

Order pages and choices by filer weight, common situations first and rare ones
last. A common situation gets its own direct question or page, rather than a row
buried in a catch-all list. Long lists retain only the rare tail and an obvious
“None of these” answer. Independent situations remain independently answerable.

Hide empty sections. Show only the current page's problems there, with a count
and a Review link for the others. Every link and button must perform its named
action without freezing. Presentation changes preserve the underlying source
facts, calculations, validation and ordinary Save/reopen lifecycle.

After document collection and any unresolved owner assignments, ask “Income
without a form” as three independent questions, each initially No: work for
yourself, tips not reported to an employer, and other income. Work and tips
open their existing topic pages for the facts their tax rules need. Other income
records one annual amount and a short description for Schedule 1 line 8z; do not
create a payer document or infer who received it. Keep the remaining rare
situations on Beyond your documents, without asking these common questions again.

## No count caps on records

The App never caps how many records a person can enter: income sources, documents, dependents,
businesses, farms, parcels, rentals, sales lots, K-1s, assets or any other repeated entry. The only
bound is the platform's size limit on a stored value and a message frame. A collection keeps a
fixed count only where tax structure itself fixes it (for example two spouses, twelve months, four
estimated-payment quarters).

The same applies to every other cap. A length, count or amount bound exists only when one of these
requires it, and the reason is written beside it:
- **Tax structure** fixes it (two spouses, twelve months).
- **A legal identifier format** fixes it (a 9-digit SSN or EIN, a routing number).
- **An external filing format** fixes it (an IRS e-file field length, cited).
- **Encoding or arithmetic safety** needs it, where checked arithmetic doesn't already cover
  overflow.

Any other cap is redundant and is removed.

## Derived carrier capacity

A field that carries values copied or derived from sources (a label, a count, a total) has no cap of
its own. It admits whatever its sources admit, and nothing is truncated or clamped. See
[No caps](#no-count-caps-on-records).

## Domain engines

Tax algorithms that recur across forms are implemented once, as engines under `src/kernel`, and every
form calls them with its own parameters. A form module keeps only what is specific to it.
- **Cost recovery.** One schedule engine covers every depreciation and amortization case: method,
  convention, recovery period, placed-in-service date, basis and prior allowed or allowable
  amounts. That includes MACRS, ADS, historical, production, timber, §179 interaction and §197,
  §195 and §248 amortization, each as a parameter set.
- **Loss limitations.** One stacking engine applies basis, at-risk, passive activity and excess
  business loss in statutory order, per activity, with suspended amounts carried through
  `kernel::carry`.
- **Recapture.** One owner computes recapture by class: §1245, §1250 and unrecaptured §1250, §179,
  and credit recapture.
- **Installment sales.** One engine covers the gross-profit ratio, recapture first, payments and
  interest by year, and dispositions of the obligation.

The cross-cutting owners (Money precision, Source facts, Person attribution, Source identity) apply
inside every engine. A new case that an engine doesn't yet cover extends the engine, never a local
copy.

## Kernel rules

`src/kernel` holds the shared owners and domain engines. It stays small, stable and trusted.
- **Use it.** A pattern the kernel covers is always written with it. No local wrapper, adapter or
  re-implementation reshapes a kernel API. When a kernel API is awkward for a new case, extend it.
- **Admission.** Code enters the kernel only when this contract names it as an owner or engine, or
  when it has at least three real uses in different modules. Rules specific to one form or topic
  never enter it.
- **Layering.** Kernel code depends only on other kernel modules and the core return data types,
  never on a form or topic module.
- **Compatible change.** A change to a kernel API migrates every caller in the same change. No
  deprecated twin is left behind.
- **Tests beside the module.** Each kernel module has its tests in `src/kernel/<name>_tests.rs`,
  one per behavior or boundary, built with the shared test support.

## Investment and personal installment sales

Enter a current-year installment sale of investment or personal-use property through
its own income source, with the actual seller and community-property classification.
The ordinary income-document record owns its identity, seller, source facts and
community classification. Correcting that source retains its identity through
Save, reopen and restart.
Business, rental and farm property sales remain on their property's existing page;
actual note interest remains a separate interest-income entry.

The shared installment engine supplies recognized profit to Schedule D, preserving
the holding period and collectible character. Electing out or a sale without a
positive installment gain uses the existing full-sale calculation; personal and
related-party losses remain nondeductible. Taxable personal-use gain can enter net
investment income even though it does not enter the investment-only Form 4952 pool.
Eligible loan proceeds secured by the note and a related buyer's second disposition
use the same engine, counting actual and deemed principal once. Personal-use
property retains its pledge-rule exemption. Special situations that cannot be
calculated retain their named unavailable result.
See [Publication 537](https://www.irs.gov/publications/p537) and the
[Form 8960 instructions](https://www.irs.gov/instructions/i8960).

## Mortgage records and home rentals

Enter foreign real property in the same rental records, choosing its actual
country and entering amounts in U.S. dollars. Translate receipts and payments
using the date's exchange rate or an appropriate yearly average. Original
service dates select required ADS: 30 years for residential property placed
after 2017, 40 years for earlier residential and nonresidential property.
Foreign real-property rental income does not enter QBI. Unsupported historical
recovery, incidental-asset ADS and territorial treatment remain explicit source
boundaries with the entered records retained. Country and passive-participation
answers do not establish a Form 1116 category, foreign branch or foreign-loss
allocation; the credit requires its own supported source allocation. See
[Publication 946](https://www.irs.gov/publications/p946) and the
[IRS currency guidance](https://www.irs.gov/individuals/international-taxpayers/foreign-currency-and-currency-exchange-rates).

Enter each Form 1098, or a loan without a Form 1098, on **Home mortgage
interest**. The first lender's entry owns the home, use of the proceeds and
balance facts for the loan. Save that entry before selecting it from another
record. If a lender transfer produces another Form 1098, enter that form's
boxes and select the saved first-lender entry. Each later lender links directly
to that entry; it does not supply a second balance ledger or repeat the loan's
personal-record questions. The App combines the lenders' interest and counts
the loan balance once.

For a dwelling rented out and also used as a home, add its saved original loan
records under Related forms on the rental page. When only one eligible rental
exists, unclaimed loans apply to it automatically without storing a relation.
Rentals with no personal use keep mortgage interest in their ordinary expenses
and do not receive these loan links. Enter the whole property's mortgage
interest for your ownership share on **Home mortgage interest**, before dividing it between
rental and personal use. The linked records supply the rental's mortgage
interest; do not enter that amount again as a rental expense. A loan selected
this way supplies one whole-property allocation, so it cannot also be claimed
in full by another rental record. Loans shared between separately entered
properties need an attributable allocation the current picker does not model.
Points and mortgage insurance also come from the loan record. The rental share
of points spread over the loan term, or of a main home's purchase points
deducted in the year paid, follows the loan record's own term and timing, with
one note stating the straight-line assumption. A second home that fails the
section 280A(d)(1) residence test on the rental's days has no personally
deductible interest; the App applies that test from the rental's days.

The App applies the common debt limit and the recorded rental use to the same
loan facts, keeping acquisition-related interest separate from borrowing for
unrelated personal expenses. Choosing Standard, Itemized or Automatic keeps
the mortgage records and links; the deduction choice changes how the amounts
are used. AI Import reads the document boxes and preserves these personal-record
links. Renaming or reordering an entry preserves its identity. Unlink dependent
lender or rental records before deleting or replacing their first-lender entry;
an unavailable selection must be corrected before saving.

A Form 8396 certificate names its saved first-lender loan; the credit then
reduces that loan's interest deduction, the rental share of it on Schedule E
and the rest on Schedule A. A certificate with no loan named reduces Schedule A
alone.

A linked rental row does not show a standalone profit. Estimate the return to see its coordinated rental result.

For a replaced or removed building component, enter its partial disposition
on the building row. If the rental remains a passive activity, its property
loss shares the activity's passive-loss limits and any available rental
allowance. The allowed loss remains on Form 4797; operating losses remain on
Schedule E. A partial disposition does not release all suspended losses as a
sale of the entire activity can. A materially participating real estate
professional's qualifying nonpassive rental keeps its nonpassive treatment.
See [IRS Publication 925](https://www.irs.gov/publications/p925).
## Cooperative source records

Form 1099-PATR keeps printed payer boxes separate from supplemental records.
Allocate distributions to their actual same-owner Schedule C, farm or farm-rental
activity; the sole eligible activity may default automatically. Personal purchases,
property refunds, notice ownership and basis, qualified-payment allocations and
issuer credit notices remain distinct source facts. Import and search never invent
accepted activity or property identities or tax entitlement.

The cooperative source adapter feeds the existing income, QBI and business-credit
owners once. Farm uses production-sale distributions, excluding deductible-purchase
rebates, in its shared conservation gross-income limit; missing economic character
leaves that dependent deduction unavailable with the Farm warning. Linked land
refunds use the accepted parcel's regular/AMT basis or its current direct sale;
missing required basis, taxable-excess character or sale-history facts prevent a
complete estimate. An unavailable optional deduction or credit has its own review
note while known taxable income remains included. See the owning
[cooperative adapter](../src/farm/cooperative.rs) and [Farm rules](../src/farm/farm.rs);
[COVERAGE.md](../COVERAGE.md) records the verified scope and remaining limits.

## Farm cost-sharing improvement sources

Section 126 keeps the improvement's supported fair market value and income-right
present values separate from the actual excluded cost-share funding applied to
it. Mixed funding needs that actual funding amount; it cannot exceed the
improvement cost or government payment. Native Save checks both
limits. The amount must be zero for an unapproved program, absent conservation
certification or an actual election not to exclude any payment. An unanswered
mixed-funding amount leaves the required source issue visible.

Mixed-improvement basis is actual cost less that recorded excluded funding.
The existing wholly qualified, fully funded worksheet retains its supported
basis calculation. Both feed the selected saved parcel's existing regular and
AMT basis owner once. The income-right appraisal still controls qualified
improvement income and section 1255 history; it does not stand in for the
funding allocation. See [IRS Publication 225](https://www.irs.gov/publications/p225).

## Farm supplemental sources and prior minimum-tax histories

A farm's supplemental source rows select documents already saved on the return
and offered by its source picker. Save checks those identities against the
current accepted catalog. Removing a selected document through its normal topic
or checklist removes the unavailable farm source row during normal pruning;
reopening the farm shows its remaining sources. Selecting a document does not
establish additional tax treatment from the supplemental character alone.

Prior minimum-tax histories keep their actual prior return units. Leaving a
joint return removes histories for the departed spouse and the same joint
couple. The taxpayer's separate history remains. A retained former-joint history
keeps its existing attribution limitation; changing status never converts it
into an individual history or invents a credit allocation.

## K-1 royalties

Enter partnership box 7 royalties under **Other boxes**, or S corporation
box 6 royalties beside dividends. Enter box 13 or box 12 code I once in the
K-1's **Other deductions** list. AI Import reads these document amounts;
basis, amounts at risk and retained loss records remain your own answers.
The estimate applies the basis and at-risk limits before showing the royalty
expenses on Schedule E line 19. Gross royalties appear on line 4 and the net
amount on line 21, with all direct and K-1 royalty columns combined in Part I.
These portfolio royalties do not release passive losses, earn QBI or create
self-employment income; their net amount remains investment income. Separate
AMT balances can change the allowed expense. Reopen the same K-1 to correct
its boxes or retained records and estimate again.

Oil and gas depletion information (partnership box 20 code T or S corporation
box 17 code R) leaves that deduction out with a note; the rest of the K-1
stays in the estimate. Other credits the App does not calculate receive their
own note. Code AX carbon oxide sequestration credit recapture remains an
unsupported added tax. Correct these entries by reopening the same K-1.

## K-1 depletion adjustments for AMT

Retain printed partnership box 17 code C or S corporation box 15 code C and its
statement references. A sole supported ordinary-business source is assigned
automatically. For multiple ordinary activities, the K-1 topic shows one signed
amount per actual activity; complete allocations must equal the printed total,
including offsetting amounts when that total is zero. Saving unavailable or
partial allocations retains every known amount, including zero. The owning
calculation stops at a named source boundary only where it needs missing facts.

Apply each supported preference through that activity's AMT basis, at-risk and
passive owners before reporting Form 6251 lines 2d, 2m or 2n. A separate supplied
AMT net already includes its adjustments; the regular-record baseline receives
the printed depletion adjustment once. Supported source-specific AMT earnings
feed the existing retirement, health-insurance and IRA limits while actual
Schedule SE tax and half-SE remain regular-tax amounts. Statement categories
D/E/F and property-distribution basis need their own supported source paths;
code C alone does not establish those facts.

## Partnership self-employment earnings

Enter box 14 code A with its printed sign, separately from box 1. These
amounts can differ because self-employment earnings exclude certain items
included in ordinary business income. The App applies the shared loss limits
and deducts the section 179 amount claimed before computing self-employment
tax. If an ordinary loss is suspended and the reported self-employment share
cannot be allocated within it, the estimate identifies the K-1 for correction.
Limited-partner and trading-business restrictions still apply.

Code B gross farming or fishing income identifies a farm partnership:
its allowed net self-employment earnings appear on Schedule SE line 1a.
Other partnership earnings appear on line 2. These earnings combine with
the same person's other businesses under one Social Security wage base;
spouses keep separate computations. Optional-method elections use their
normal lines. Gross income and its qualified share also supply the existing
half-self-employment deduction allocation for QBI; the gross amount itself
does not add taxable income. Reopen the same K-1 to correct these facts.

## Partnership business property

On the partnership K-1, **Other boxes → Business property** takes the box 10
section 1231 gain or loss and the box 9c unrecaptured section 1250 part that
the statement assigns to the box 1 business. Keep rental property amounts
with their rental activity. Enter the statement's qualified section 1231
amount in the section 199A statement, with the same sign and no larger than
the business amount. These property amounts are separate from box 1 and
portfolio capital gains.

Enter business-property losses carried from the prior year in their separate
basis, at-risk and passive tabs under **Losses carried from 2025**. The shared
loss limits preserve their character. Allowed property amounts join all
other business-property sources in the return's one five-year lookback;
ordinary amounts can enter QBI and the remaining capital gain cannot. An
allocation the statements do not determine is reported for review. Publicly
traded partnership property remains outside this implementation.
When these nonpassive businesses have only property losses, the excess-business-loss
limit defers the disallowed property loss and its qualified portion. Only the
deductible property loss enters next year's five-year lookback history. Mixed
operating/property losses and rental-property allocations that remain unresolved
are reported as unsupported.

For one nonpassive ordinary partnership, operating profit can offset a property
loss, or a property gain can offset an operating loss. The disallowed qualified
portion follows the gross loss's current and retained qualification, including
original-year qualified ordinary losses when no current statement is attached.
Current SSTB restrictions do not reclassify those retained qualified losses.
Losses carried in the gain character, two loss characters, and combinations with
other businesses, rental activity, sale statements or partner deductions still
need their separate allocation and are reported for review.

AMT can use its independent activity/property and lookback records. Reopen the
same K-1 to correct its values; corrections preserve the document identity.

## K-1 rental property

On either K-1 page, enter the statement's section 1231 and unrecaptured
section 1250 amounts with the box 2 or box 3 activity that owns them. Keep
them separate from the box 1 business and portfolio capital amounts. Each
rental activity has separate ordinary and section 1231 passive-loss
carryovers under **Losses carried from 2025**. AI Import reads the printed
amounts; these carryovers and the full-sale selection remain your own facts.

Select an activity as sold in full only when the entity disposed of all of
its property in a fully taxable transaction to an unrelated buyer. An
overall loss releases that activity's current and prior passive losses;
an overall gain can offset other passive losses. A real estate professional's
former passive activity keeps its nonpassive character. A partial sale
retains the normal passive limits. Installment sales, like-kind exchanges,
casualties, theft and related-buyer dispositions need separate worksheets.

Allowed property amounts join the return's Form 4797 calculation once.
QBI uses the resulting ordinary character only when the section 199A
statement lists that rental activity; its wages and property are counted
once. Investment-tax treatment also uses the recorded professional safe
harbor, so releasing a passive loss alone does not exclude it from investment
income. A rental-property loss held back by basis or at-risk limits is
reported as unsupported because its activity-specific carryover is not
modeled. Mixed property excess-business-loss allocations still need their
separate worksheets. Regular and AMT property records remain independent.
Reopen the same K-1 to correct the amounts or sale selection.

For a professional's former passive activity, prior losses released against
its own income follow that income's investment-tax treatment. Additional
prior losses released by a qualifying full sale can still reduce investment
income when the professional safe harbor excludes the current activity.

## K-1 Section 179 sale statements

On either K-1 page, **Other boxes** includes **Section 179 property the entity
sold** for the Form 1065 box 20 code L or Form 1120-S box 17 code K statement.
Enter each property's description, acquisition and sale dates, and your shares
of its sales price, cost including selling expenses, depreciation excluding
section 179, and section 179 deduction passed through. Use a separate AMT list
when its depreciation differs; **Same as regular** copies the regular list.
Installment sales, exchanges, casualties and theft need separate worksheets.

Enter the sold-property part of your own section 179 carryover separately from
the printed statement. It cannot exceed the total carryover or the section 179
amounts for the properties sold, in either tax system. Acquisition cannot follow
the sale, and depreciation plus section 179 cannot exceed cost. AI Import reads
the printed sale list and preserves your personal carryover amounts.

Save retains these facts. Reopen the same K-1 to correct them; corrections keep
the document identity, and accepted facts survive a task restart. Invalid edits
leave the accepted return unchanged. An imported empty list requires correcting
any nonzero sold-property carryover before Save.

Fully taxable sales use the same K-1 basis, at-risk and passive limits as the
business's other items. Form 4797 shows each property's gain or allowed loss
once. Sold-property carryover is restored to that property's basis and removed
from Form 4562's remaining carryover; it is not another deduction. Qualified
ordinary sale amounts join that business's QBI calculation. These sale amounts
do not add self-employment earnings. Investment-income treatment follows the
activity's participation and trading classification. Independent AMT amounts
enter the AMT calculation and Form 6251 line 2k; QBI stays the regular deduction.
Allowed ordinary sale amounts also enter the excess-business-loss and NOL
worksheets as business amounts; shareholder-loan gains retain their investment
character. The total NOL carryforward includes both the current-year NOL and the
separately disallowed excess business loss. A single nonpassive ordinary K-1
business can combine its ordinary sale amounts with its operating loss and
released historical ordinary losses. Its gross losses retain the qualified
portions determined by the earlier loss limits and original-year records;
sale profits do not reduce the loss pool used for that allocation. Current SSTB
eligibility does not replace historical qualification. Without a current QBI
statement, only independently qualified historical losses enter this benefit.
Other unresolved qualified-loss allocations still require review.

An unavailable qualified section 1250 source portion differs from a documented
zero. Retained qualified-loss records likewise differ from the underlying basis,
at-risk and passive balances: selecting no records does not confirm those
balances are zero. Missing qualified history retains its own QBI notice and
does not remove an otherwise allowed income-tax loss. Leaving a qualified loss
out can increase the QBI deduction. Gross self-employment source information
is also distinct from net profit and a confirmed zero; it keeps its separate
gross-allocation notice when unavailable.

A nonzero sold-property carryover with multiple properties needs a per-property
allocation that this App does not calculate. Missing or inconsistent activity
records, unresolved QBI character and other unsupported combinations still
produce their specific review issues. Saving a statement does not resolve them.

## Schedule C personal records

The Self-employment page keeps the business header, gross receipts before
returns, other business income, employee plan contributions, meals by treatment,
leases and the annual de minimis election as personal records. Enter each
Form 1099 separately and link it to the business; its amount joins gross receipts
once. Returns and allowances are bounded by manual receipts plus those linked
sources. Current business choices and edited source amounts decide the bound;
accepted rows outside the editor supply invariant context. The server derives
that context again on save. Do not import these personal facts from a Form 1099.
Owner retirement contributions belong on Self-employed health insurance and
retirement, apart from employees' Schedule C line 19 contributions.

The annual election, depletion and overtime controls each require an explicit
negative or a positive choice with its facts. A lease states whether it continues
or ended in 2026. Print-only header and vehicle answers may retain their approved
supplementary defaults. Original lease-year tables cover 2018–2026; older leases
are refused by name. Inclusion reduces the lease-payment deduction to no less
than zero and does not reduce separate operating costs. Defined-benefit amounts
come from the actuary, subject to the earnings limit and an explanatory note;
the App does not calculate the actuarial limit.

Use the existing business expense, inventory, vehicle and royalty-depletion
owners when extending these records. A lease uses its original lease-year IRS
inclusion table throughout its term. Depletion joins the return-wide oil/gas
limit before the resulting deduction reaches business profit, SE and QBI.
The original accepted election, inventory and depletion records survive any
omission from the calculation's supported working copy. Preserve ordinary schema
validity, native relationship checks, reopen/save and stored-return round trips.
The [coverage owner](../COVERAGE.md) records tax limitations and verification;
[source fixtures](../src/tests/business/schedule_c_specials.rs) and the
[Schedule C footing audit](../src/tests/cross/line_audit_c_se.rs) hold worked amounts.

## Labels

Direction of 2026-10-08: a label, caption or choice text that the App
produces is a FormattedText field (`#[libertas_formatted_text]` +
`#[libertas_read_only]`) built from a string-resource key and typed
arguments, so it localizes and can carry a name, a number or a date as an
argument. A plain `String` label is allowed only when its value is pure user
input: a name the person typed for their own record (an asset, a policy, a
building, a W-2, a child) that the App never composes or translates. "You",
"Your spouse", a form number, a document kind, or any text assembled by the
App is never a `String` label. Catalog members shown by an `EnumSource`
picker follow the same rule for their promoted label field.

## Estimate note presentation

Use **Estimate notes** for the shared result note list. Its heading and help
must apply to either a refund or a balance owed and explain assumptions,
items left out, and calculation limitations. Keep rule-specific warnings
with their relevant notes and help, and update every reference when changing
the heading.

Correction ownership and record references follow these rules:

- Give each actionable issue or assumption its typed source context and the
  page that actually owns its editable facts. Preserve source provenance when
  a correction belongs on another page; offer the actual alternatives when
  either side of a contradiction can be corrected. A related page without
  those fields is not a correction destination. Unsupported computations and
  raw-only facts stay readable without a fabricated link or Review self-link.
  Message text, translations and resource keys never determine navigation.
- Prefer a provided nonblank name. Use the same string-resource argument with
  literal text for the name or a typed integer for the actual record ID when
  no name is provided. Localize platform default document names through the
  existing document-kind resources, with the real record ID distinguishing
  unnamed documents. Never manufacture a persistent ID for display.
- Regenerate notes on every calculation from that calculation's answers;
  names need no stability across calculations. Do not persist separate note
  IDs or note history. Deduplicate within one result by source identity,
  corrective scope and message. Names are
  mutable display data: equal names must not merge different records. Use an
  income ID, actual owner and historical filing year, or a nested occurrence
  scoped to its parent within the accepted revision. Keep intentional global
  notes global. Preserve affected-record labels in both Review and completed
  estimates.
- Actionable estimate notes use Review's separate correction-note list and
  normal `ReviewItem`/`OpenSection` binding. They never become blocking
  remaining items. All blockers, including notices with no editor, prevent
  Finish; no correction-link count cap may hide a blocker. A completed estimate
  offers `ReviewAccepted` to return to these corrections.
- Every page that shows the estimate, hubs included, shows an "Estimate so far"
  line and only the `remaining_items` fixable on that page, each marked "Fix".
  Other items become one "N more items in later sections" row linking to Review;
  omit that row when none remain elsewhere. Review alone keeps the complete
  list from the same producer and adds its first unfinished page. Its rows
  retain coming up, fix or not calculated states and their actual answer-page
  destinations. The line never repeats a refusal as text and
  holds no figure while a walked topic with nothing entered would count as
  zero. Next and Resume always open the first unfinished page, even after a
  jump ahead.
- The last page before the review, Situations people often miss, is required:
  the review lists it as the first unfinished page and Finish waits for its
  save, where "None of these" answers it. Its income and taxes question plans
  the page of each option checked (and a form already on the return checks its
  option without storing it); unchecking withdraws the plan and removes no
  entry. Its list of situations the App does not prepare is the refusal list
  itself (`Unprepared`, built from `RareSituation`, `RareAdditionalTax` and the
  payload-less refusals), so a refusal added to its owner joins the page; a
  chosen one makes the estimate not calculated. The pages that only held
  those refusals are gone; More adjustments, More credits and Other taxes
  stay reachable for their supported items and are walked only when planned
  or needed.
- A document's **Amount on the return** preview uses the same supported
  working-copy projection as the estimate. A wholly excluded document previews
  zero; accepted answers are not changed by a preview.

Unknown historical earned-income-credit eligibility preserves the known filed
credit unchanged in a repayment comparison and counts no increase, with a
note. A known denial with a contradictory positive filed credit remains a
consistency error; follow [claim_repayment.rs](../src/deductions/claim_repayment.rs).
An unknown final FUTA credit-reduction rate claims no state unemployment credit
and uses the gross 6% liability, with an editable correction note; never
substitute a guessed or zero reduction rate. Follow
[household_employment.rs](../src/business/household_employment.rs).

Recompute current read-only IRA workplace coverage from actual W-2 and plan
facts with the page's shared resolver before worksheet selection. Update only
the derived fields in the supported working copy, preserving its omissions.
Historical reconstructed worksheets retain their source-year participation
facts; do not replace them with current-year coverage or a stale current flag.
Follow [retirement.rs](../src/retirement/retirement.rs) and its tests.

## Wage source facts and credit choices

Store each employer's printed W-2 amounts once. Railroad payroll uses its own box
14a compensation, tier 1, tier 2, Medicare and actual Additional Medicare withholding;
ordinary Social Security boxes stay blank. Form 8959 line 19 includes ordinary box 12
codes B/N with box 6, as its instructions require, and excludes RRTA B/N. Keep the
cent source amounts through line 22 and round the resulting ordinary credit before
combining it with RRTA withholding on line 24. A statutory employee's box 1 supplies one
Schedule C's gross receipts, separately from that job's typed expenses. Preserve the
employer's withholding and payroll facts when an unsupported linked business leaves
the estimate. Employee pay without a W-2 retains the actual payroll system and independent
Social Security and Medicare coverage. Only both-covered pay with a real A/C/G/H
reason uses Form 8919. A domestic state/local position uses its actual State
Social Security Administrator coverage determination; a section 218 agreement
alone establishes neither exemption nor a filing reason. Other known coverage
uses an unnumbered payroll worksheet and enters the estimated total tax.
Physical payroll bases stay with the actual employee even when community income
is shared. Unavailable facts preserve known income and independent payroll
components; a materially unknown added tax prevents a complete total. A valid
foreign-system totalization certificate must establish exemption for this pay
and period. Foreign employment alone is not an exemption. Foreign-government
and employee-representative special obligations remain explicit limitations.
Actual railroad compensation and withholding reuse the RRTA and Additional
Medicare owners, including their separate threshold; ordinary unpaid RRTA
collection remains outside the estimate. These worksheets do not assert a
filing-ready projection for a non-Form-8919 payroll tax.

Tips and overtime eligibility use explicit No amount / Ineligible / Eligible choices.
No amount is valid only without the corresponding source amount; it is never a negative
eligibility answer. The optional statutory business selection overrides the computed
zero-expense Schedule C; deleting or retyping that business restores the computed
default through normal pruning. Statutory pay for work abroad uses a saved Schedule C's
existing services-source owner, even with no expenses; it never also enters the
W-2 employee foreign-pay amount. The existing foreign exclusion owner allocates
gross exclusion and related expenses; excluded statutory gross is not IRA
compensation. Mixed foreign operations retain the business source owner's existing
limitation. Foreign housing with statutory employee pay is refused by name because
that owner's self-employed housing worksheet does not establish the employee funding
allocation; no housing figure is presented as supported for that combination.

Keep personal tip records, noncash tips, occupation answers, Medicaid amounts already
in box 1 and statutory business associations outside AI-selected W-2 fields. Import
may replace printed box 12 facts without replacing those answers. Household wages,
strike benefits and Medicaid payments without a W-2 belong to Other income; routed
1099 amounts use those same kinds.

Compare each filer's combat-pay and excluded Medicaid earned-income choices using
the actual combined EIC and additional child credit. Choose exclusion on an exact
tie; apply Form 2441 and IRA compensation rules independently. The person can decline
the EIC. Difficulty-of-care payments add nondeductible traditional IRA contribution
room and do not create a deduction or Roth compensation. These estimates use the
2026 W-2 instructions and draft 2026 return forms, with published prior-year
instructions where 2026 instructions are unavailable; they require final-year review
before filing use.

## Individual QBI aggregation facts

Income topics share one complete set of aggregation eligibility, election,
history and annual disclosure facts. Group labels on the actual sources own
current membership. Save a new source without a group before selecting it in
an election; its normal accepted identity then appears in the disclosure picker.
Active records offer their current members; inactive prior-election records
offer genuine live sources without claiming current membership. A rename
does not erase history. Correct history directly in the same normal Save,
including alongside a rename; unavailable or unfavorable answers remain
saved and keep a selected group's estimate explicitly incomplete. Existing
direct-only groups without these facts retain their source-attestation path. Single-business
ordinary S-corporation and non-PTP partnership sources use the existing wage
and property limit after their source qualification and loss limits.

## Questions, import and search

Follow the platform's [ask-once rule](https://github.com/LibertasIoT/libertas-doc/blob/main/reference/app-contract.md#ask-once-directly)
and [AI design obligations](https://github.com/LibertasIoT/libertas-doc/blob/main/reference/app-contract.md#ai-access-in-every-design-step).
On a tax-document page, name the form, boxes or lines and attached statements in
its import instruction. Classify each added field as document-sourced or supplied
from personal records; preserve non-imported histories and confirmations outside
any import-selected nullable value or Union payload. Review import mappings and
person-record preservation in tests when changing a page.

Selecting `NoStatement` must retain personal records while omitting current
statement amounts. Previously qualified suspended losses can still participate
from actual prior balances when allowed, using their original-year qualification.
Retained unrelated histories stay inactive and receive structural validation,
without reconciliation to a nonexistent current statement. Missing history for
an actual prior suspended balance adds one source-linked notice and omits only
the affected QBI loss character; a complete other character may participate.
Supplied amounts must reconcile even if another stage's history is missing.
Zero qualification is a real loss row with zero qualified amounts, not an empty
list. Follow the owning source and fixtures in
[partnership.rs](../src/passthrough/partnership.rs), [scorp.rs](../src/passthrough/scorp/mod.rs) and
[business_rules.rs](../src/activity/business_rules/mod.rs).

Estate and trust K-1 sources retain the fiduciary's actual taxable-period dates
and accounting election. On an approved change to or from a 52/53-week year,
a change stub of one through six days belongs to the following taxable-period
row, not a separate year. For the following ordinary year, retain its normal
month-first beginning separately; do not apply the elected-year nearest-month
convention to its actual beginning or reset an inherited loss's carry period.
Current source-year applicability and inherited loss-year character remain
separate. Follow [estate_trust.rs](../src/passthrough/estate_trust.rs) and the existing
period and loss owners.

Enter a community-property estate or trust K-1 as its whole source. Calculations
apply the return's income and property-gain shares separately, including the
statement's regular and AMT components; Save, reopen and restart preserve the
entered amounts. An aggregate AMT or net-investment-income adjustment without
the components needed for those different shares prevents a complete estimate.
Unverified optional investment-interest, QBI or capital-loss NIIT attribution
retains its named benefit omission while known income remains included.

Use the fiduciary's reported QBI statement once. A statement identifying several
uniquely linked, fully allowed material activities needs no passive-loss ratio
when the existing activity owner confirms no prior, released or suspended losses.
For limited or shared activities, retain the fiduciary's signed per-activity QBI
allocation and the beneficiary's original qualified-loss history. Current
allocations reconcile to the statement; shared qualified portions cannot repeat
one activity's loss. The existing passive and vintage owners determine current
allowance, historical release and carryover. Shared source prefixes preserve
one rounding result regardless of statement order. Normal Save, reopen and restart
retain these records. Unavailable attribution leaves only the affected QBI
benefit out with its named notice; preserve known income and the accepted source.
Never infer an allocation from taxable distributions or create another QBI engine.

A complete fiduciary investment statement replaces its eligible ordinary subtotal
for Form 4952. Reconciled regular and AMT preferential portions use the existing
return-wide election once; it changes the corresponding preferential tax bases
without adding income again. An allocated election must also fit the actual AMT
qualified-dividend and net-capital-gain pools. Unresolved source character retains
its named investment-interest benefit boundary.

For each new page or destination, document the relevant `TaxSection` member's
form identifiers, common names and actual supported scope, bind the destination
through `CopyFrom`, and update `TAX_NAVIGATION_PROMPT` where ranking needs change.
Search selects only offered links and eligible action choices under the public
[AI Search contract](https://github.com/LibertasIoT/libertas-doc/blob/main/reference/protocols.md#ai-navigation-and-ai-search).
Report the changed pages' import declarations, target fields and search
vocabulary with the implementation.

## Community-property manual amounts

Income statements and K-1 records retain the source amounts their owning pages
request; the App applies their community allocation once. Other pages that ask
for a deduction or payment attributable to this separate return need that return's
amount, rather than a second copy of the household total. Personal expenses paid
from community funds generally split equally; separate-funded personal expenses
belong to the spouse who paid them. Keep full costs where a source page explicitly
asks for them, including costs subsequently allocated to a home office. Adjustment
rules remain specific to the actual source: IRA deductions are individual, and
the recipient's existing share of community income is not deductible alimony.

Separately paid estimated taxes belong to the payer. Joint estimated payments may
be divided by agreement; without agreement, each separate return uses its tax as
a fraction of both spouses' separate taxes. The Payments page records that amount
and does not apply another community fraction. Withholding stays with its income
source. See [Publication 555](https://www.irs.gov/publications/p555), Deductions and
Estimated tax payments. Form 2441 uses actual individual earned income, including
the explicit spouse or selected-month earnings asked on the care page, rather
than the community allocation used for income tax; see the
[Form 2441 instructions](https://www.irs.gov/instructions/i2441), Lines 4 and 5 and
Lines 18 and 19. Bond credit and designated-income entries retain their existing
reported source and accepted distribution link; do not repeat income already
included through that link.

## Child and dependent care

Enter each person's care costs on **Children and people you support**, then
complete **Child and dependent care** for Form 2441. The care page shows those
qualifying-person costs as read-only facts, records providers and their
identification status, and asks the plan and earned-income facts the form uses.
A provider's identifying number itself is never collected. Refusal to supply a
number retains the expense with an attachment reminder; an unidentified or
prohibited related provider's expense is left out with an explanation.

Enter or import each W-2 on **Wages** first. Its actual box 10 supplies the
care-benefit row; enter only the part already included in box 1 and any forfeited
or carried-forward amount there. Supply one combined maximum allowed by the
actual dependent-care plans, counting each distinct plan once. A same-plan
grace balance does not increase that plan's maximum. Include grace-only and
own-business plans when they provided benefits; households with no such
benefits choose **No benefits**. Own-business or partnership benefits select
the actual saved source with the matching owner.

For a student or incapable joint filer, select the actual months and supply
the earned income in those months or explicitly choose no earned income. The aggregate
calculation is a lower bound when a detailed monthly breakdown could give
more; the estimate explains that limit. Shared months count for only one
filer. Separate filers explicitly supply their spouse's earned income or
choose **I do not know my spouse’s 2026 earned income**, which counts zero with a conservative-estimate note.
These factual Nullable choices have no default; the editor requires an explicit
answer. Unpaid 2026 care and prior-year care first paid in 2026 are optional
claims. Their default null answers state that no such expenses are claimed;
supply an amount or the prior-year worksheet facts to claim them. The unpaid
amount enters only Part III incurred expenses, never the current-year paid
expenses for the credit. Each W-2 benefit row likewise requires an explicit
answer for amounts also in box 1 and amounts forfeited or carried forward.

The running estimate checks the same candidate facts as Save without accepting
them. Correct a W-2 or business on its owning page; normal saving removes care
answers whose actual source disappears or no longer fits. Independent provider
and prior-year facts survive changes to the current qualifying-person list.
Reopening the care page uses the latest accepted facts. AI Import reads printed
W-2 boxes, while AI Search opens this same care page; neither invents a plan
maximum, provider relationship, selected-month earnings or a source identity.

## Marketplace and self-employed health calculations

When participating families do not all agree on a policy allocation, each
nonresidual count-based share rounds down to a whole percent; the enrolling
or remaining family takes the residual. This is the App's general source
method inferred from Publication 974's 33/33/34 example. Agreed fractions
and actual no-advance SLCSP ratios remain exact.

A filer who can be claimed as a dependent but is not actually claimed has no
premium tax credit or tax-family membership of their own. Marketplace uses
the actual enrollment attester for each covered-person period to determine
which return reconciles its advance credit. This return repays only its own
responsible advances; an identified other family keeps its responsibility.
Unknown responsibility with advance credit prevents a complete reconciliation.
With no advance credit and no tax-family membership, an unknown attester does
not create credit or repayment. Save and reopen retain these source facts.

QSEHRA affordability uses the employee's own household income and actual
self-only permitted benefit and benchmark. For a claimed dependent employee,
use the employer notice and the employee's own or joint-return income
components, adding actual dependents' required-return income. Reuse required
returns already entered on Marketplace; documentary components must agree
with those sources. Joint-return income includes both spouses. The employee's
actual spouse and dependents use existing People identities. An affordable
arrangement excludes only those people. The actual permitted benefit still
reduces the claimant's monthly credit when other eligible family coverage
remains. Unanswered context or an actual relative unavailable on People keeps
that employee's reconciliation unavailable; an explicitly empty household is
complete. Save and reopen preserve these source facts. See
[Notice 2017-67, section K](https://www.irs.gov/pub/irs-drop/n-17-67.pdf).

Each established business plan selects the saved Marketplace statement and actual coverage months. Allocate existing premium sources and the same credit among businesses once, preserving each trade's earnings limit and actual entity-funded portion. Changing calculation method retains these source facts. Canonical nonspecified premiums use the existing business earnings limit before specified premiums; their exact source fractions share the remaining limit. In Iterative work, ordinary manual and qualified long-term-care premiums retain cents in the existing owner and shared age pool too. Round the combined business deduction once, then preserve its source usage through medical-expense allocation. Actual Schedule C/F profit and the S-corporation limit retain their existing source cents during Iterative work; linked farm statement income comes from the same farm owner, and the corporation uses its selected W-2 Medicare wages. The partnership owner retains positive box 14 and guaranteed-payment source cents after its existing admitted loss and section 179 deductions. The optional-method owner attributes its fixed Schedule SE earnings total using actual source cents. Worksheet W retains the same annual Schedule 1 half-SE deduction and attributes that fixed total using source profit ratios, preserving its residual cents and source identities. These projections preserve the existing annual filed amounts; they do not recalculate SE tax or invent cents for deductions settled by another owner. Positive nonpassive rental income with no prior loss retains the existing rental owner’s source-cent residual during Iterative work. Passive-loss pools, prior-loss releases and negative rental net still retain their existing annual owner projection; this residual does not reconstruct their allocations. SEP inputs likewise come from their existing owner. The IRA owner retains contribution and chosen nondeductible source cents for Iterative work while preserving its annual basis ownership and prescribed phaseout limits. A prior traditional IRA excess absorbed by the current year's unused room also enters that deduction work once; it remains separate from fresh contributions for the Saver's Credit. A covered IRA enters only after its ordered deduction stage completes. That worksheet uses the selected Iterative health deduction before filing rounding; its prescribed $10-up phaseout can differ from one calculated using the rounded health deduction. Confirmed employee-funded pension contributions likewise retain the existing owner's admitted cents for this work calculation. The sole-plan credit belongs to the full policy enrollment source, independently of its SLCSP split into specified and nonspecified expenses. A continuing Step 3/5 specified-premium worksheet component may be negative; combine it with the selected nonspecified costs on that same source before claiming a nonnegative deduction or allocating medical expenses. Fully offset a credited source before allocating remaining positive costs. Attribute only the selected deduction when Simplified retains a more conservative worksheet amount than the final credit permits; the remaining costs become medical expenses only through their existing eligibility rules. Proportional attribution of remaining costs is the App's reasonable source method, rather than an additional IRS-prescribed formula. If nonspecified premiums already consume the trade's earnings limit, preserve the publication's separate Worksheet P/Form 7206 stop route.

The Iterative method uses cent work amounts, including admitted source projections and required dependent-return income before filing-row rounding. Reuse the owning annual tax-character and allocation results; cent work does not authorize another income engine. It stops only when both the credit and specified-premium deduction change by less than $1. Keep the selected Step 4 credit together with the Step 5 deduction; computing another credit from that deduction would replace the accepted pair. A cycle or exhausted calculation budget starts a fresh Simplified calculation of both amounts and adds a review note. Reuse the existing owners for the prescribed order of affected passive losses, IRA deductions, savings-bond exclusions and student-loan interest; simultaneous iteration does not establish that ordered procedure. Retain each completed owner’s actual amount while repeating the method for the next listed item. Temporarily omitting an IRA deduction from candidate household income must not create nondeductible contribution basis; final basis and QCD agreement still belong to their existing annual owners. See [Publication 974](https://www.irs.gov/publications/p974), Iterative Calculation Method and Special Instructions for Self-Employed Individuals Who Claim Certain Deductions/Exclusions.

When Form 8582 lines 1d and 3 are losses, initial candidate income omits the allowed real-estate rental loss components while retaining positive rental and other passive income. A prior loss absorbed by the rental's own profit is still a loss component. At its prescribed stage, retain the full passive owner's result, source suspensions and original regular-tax MAGI context for subsequent regular and AMT calculations. Competing non-rental losses, concurrent nonpassive professional rental losses, and disposed or former-passive loss releases whose initial treatment is unverified prevent the coupled estimate with a named source limitation; ordinary independent calculations remain available.

Cent work amounts and source expense allocations remain distinct from rounded form rows. Annual Form 8962 rows round once; monthly totals sum their displayed columns. Derive net credit and repayment from those same filing totals. Do not redistribute a filing-rounding difference into another policy or business expense. The marriage worksheets retain their specifically required half-income and contribution rounding.

Medical expenses retain their statement/month identity through the final retained subsidy and selected self-employed deduction. Aggregate the remaining eligible source fractions with other medical cents before rounding Schedule A's expense total. Medical eligibility comes from the People owner, independently of the premium-credit tax family. A policy covering someone whose actual medical cost attribution is unverified receives a named omitted-medical-benefit notice while its known mandatory credit reconciliation remains available. Each policy month can retain the actual premium increase for covered nonmedical children under 27, including a verified zero, and the actual child portion of its existing tax-free funding. Partial entity reimbursement can retain its actual child portion too. These facts come from actual costs and funding; a silver-plan ratio does not establish them. Apply each business's selected self-employed deduction to its actual funded child cost first; unused child cost never becomes a medical expense. Subtract the same policy month's retained subsidy and medical deduction usage from its remaining actual medical cost, stopping the medical benefit at zero. This combines Publication 502's child-allocation permission and credit exclusion as the App's conservative source method, rather than an IRS-published combined example. Funding never changes the policy's allocable credit, and these medical cost classes remain independent of its silver-plan expense fractions. Missing evidence and a verified zero remain distinct. If corrected People facts make the same insured child medically eligible, retain the actual cost facts and ignore their now-inapplicable nonmedical classification. See [Publication 502](https://www.irs.gov/publications/p502), Coverage for nondependents and Child under age 27.

Allocate subsidy among linked plans by their actual enrollment sources and consistent business parts, before reducing expense cost for actual tax-free funding. This is the App's selected reasonable source-allocation method under Publication 974; the publication does not prescribe that exact method for every funding combination. A source-authorized reference enrollment premium can differ from current cash paid. Keep those two facts distinct, and check the final selected health/subsidy pair against actual source costs before reporting a complete estimate. Interim iteration amounts are not final expense claims.

## Health-account source ownership

Enter each W-2 once on Wages. Health accounts displays its actual nonzero code W
and code R amounts and asks which part the employer designated for 2026 or
2025. A split retains the entered 2026 part and the remaining 2025 part; an
unanswered year withholds the complete estimate. Current-year amounts feed the
same owner's HSA or Archer contribution calculation once. Enter only employer
principal outside those W-2 sources in the account's other-employer amount,
including actual next-calendar-year deposits designated for this return.
Prior designations do not become current contributions or taxable excess merely
because they appear on a current W-2. Corrections, retained prior excess and
deposit-year wage reconciliation retain their existing separate source facts.
Changing a split's source amount requires reviewing its allocation first;
clearing the year answer permits normal source correction and re-answering.
Removing or reassigning a linked source likewise requires clearing its allocation.

Qualified IRA-to-HSA funding names the actual same-owner Form 1099-R payment by its accepted identity. An inherited payment resolves its existing beneficiary/deceased-owner group for annual basis, QCD and estate-tax calculations; that annual group is never a substitute payment. Each selected payment's gross bounds its total funding, and source exclusions aggregate into the group once. Reordering or renaming source rows preserves identity. Removing or changing a selected payment or its required group must first update or remove the funding link. Selecting a statement does not by itself establish dated subpayments or solve earlier-activity basis treatment.

Health accounts retain actual qualified IRA-to-HSA funding uses from 2007 through 2024 separately from the existing 2025 testing-period history. The two source histories jointly establish lifetime use; neither adds a current contribution or repeats current recapture income. A prior qualified transfer still consumes lifetime use after a testing-period failure. Basis funding generally requires funding before other current-year basis activity. An own Roth IRA also supports distinct dated nonqualified ordinary withdrawals before or between transfers when its confirmed opening history and accepted current sources establish no incoming Roth basis. Each funding uses its own contemporaneous aggregate Roth value, uses earnings first, and removes the exact regular/conversion-year taxable/nontaxable classes once from the existing annual ordinary-income and conversion-recapture calculation. Retained current-year regular contributions also qualify when every transfer’s recorded aggregate Roth value covers that transfer plus all opening regular/conversion basis and all retained annual regular contributions. The same bound includes the existing IRA owner's upper limit on prior Roth excess absorbed into current contribution room; an employer Roth SEP/SIMPLE contribution is current conversion basis, rather than an absent incoming source. This sufficient bound establishes zero funding basis without inferring deposit timing; next-year deposits designated for this tax year are already part of that annual total. Ordinary withdrawals keep their existing annual basis and conversion-recapture treatment. Falling below the bound leaves the funding exclusion unverified; it does not establish that basis was necessarily consumed. When funding shares a date with another payment in that same Roth pool, enter each source's actual within-day sequence from 1 through 100 with its existing Form 1099-R. Missing or duplicate sequences leave funding unverified; statement identities and entry order never establish chronology. A payment combining ordinary and funding amounts, qualified payments and other incoming basis remain outside this added path. A nonqualified inherited Roth also supports earlier and intervening ordinary withdrawals using its own beneficiary/deceased-owner opening regular and conversion basis. Each actual funding payment must be separate and dated, with its own contemporaneous group value; the annual inherited Roth owner removes exactly the classes consumed by funding. Ordinary inherited payments sharing a date commute when no funding payment shares it; otherwise the same actual source sequence is required. Personal contributions and other deceased-owner groups cannot replenish that inherited pool. Missing opening basis, an ambiguous funding date or a combined ordinary/funding source leaves this exclusion unverified. A funding-first answer cannot override an entered earlier ordinary withdrawal date in that same Roth group. Correct that answer in Health accounts or the actual date in Retirement before saving or estimating. Rejected edits leave the accepted source, answer and identifiers unchanged. General traditional earlier-activity coordination and other unentered inherited basis adjustments remain uncomputed; a chronological ordinary-IRA pro rata rule is not established. See [Notice 2008-51](https://www.irs.gov/pub/irs-drop/n-08-51.pdf), One-time qualified HSA funding distribution, and [Treasury regulation 1.408A-6](https://www.ecfr.gov/current/title-26/chapter-I/subchapter-A/part-1/section-1.408A-6), A-4/A-8/A-9 Roth ordering.

A separate personal traditional IRA funding payment can also coordinate with
ordinary withdrawals when its actual source-event aggregate value covers the
funding amount plus all opening IRA basis. This sufficient bound requires one
funding payment, a unique valid annual basis record and no current IRA
contributions, incoming basis, QCDs, conversions, rollovers, corrections,
repayments or other basis changes. It proves zero funding basis use without
choosing ordinary withdrawal order. The retirement owner then applies its
normal annual Form 8606 calculation. A lower value does not establish how much
basis funding consumed; general earlier-activity proper adjustments remain
uncomputed. The source-event value and the HSA receipt date remain distinct.

Keep each actual Form1099-SA or trustee statement as one Other income record, including its dated nonmedical payments. Own HSA distributions use that same source as Archer, Medicare Advantage and inherited-account distributions. Contributions, contribution corrections and eligibility testing histories belong to Health accounts. Do not recreate ordinary distribution fields on its account editor or fabricate multiple forms to apply age/disability exceptions to one statement. Unreported account events must identify their trustee-statement origin and actual terminated-account or pledged-assets facts.

Attempted indirect rollovers retain actual source receipt dates, separate deposit parts, the same owner's HSA or Archer destination, and actual ordinary cash contribution designation if exclusion fails. The estimate derives current successful rollover history across accepted sources; failed attempts and direct transfers do not consume that history. Eligible parts are excluded once. Failed 2026-designated cash enters the existing destination contribution/excess calculation once and must be excluded from manual personal and later-deposit totals. A 2027-designated deposit does not become a 2026 contribution, although its failed source distribution remains 2026 income. Source previews show known taxable source amounts using the complete candidate history, independently of destination deductions. Unresolved designation, destination facts, prior-year effects, failed property or competing same-day priority prevents the affected complete estimate. The Health accounts page displays linked cash deposits read-only and derives a missing destination-account topic. Each correction may link its original failed-deposit statement separately from its returned-excess statement. Source, owner, destination, deposit date and designated year must match; combined returned principal cannot exceed that original deposit. Manual corrections use only other manual contribution room. The estimate adds linked gross and subtracts its actual returns once, including net later deposits, while reopening preserves manual amounts and accepted source links. Changing or deleting an original source cannot strand a retained correction.

An entered HSA returned-excess statement links by its accepted source ID to the existing contribution correction. The correction supplies actual principal, signed attributable income, payer and contribution year; its rows reconcile to source gross and printed earnings when present. A blank printed earnings box does not imply a zero custodian adjustment. Count corrective income and removed contributions once. Source amount changes reopen correction review; removing both source and correction through normal checklist pruning is allowed, while retained corrections cannot name a deleted, retyped or differently owned source.

Archer corrections use the same accepted statement identities and source checks. A standalone Archer statement with actual printed earnings retains its source income; link it when entering the contribution correction so that income is consumed once by that owner. A genuinely blank earnings box needs actual signed correction detail. HSA and Archer pickers are filtered views of the same accepted income records, with a shared read-only HealthReturnDocumentChoice; they do not create another statement ledger.

Retained Archer excess first decreases by taxable distributions, then by the
current year's unused contribution capacity under Form 5329. That capacity
subtracts current personal and employer contributions. Absorbed prior excess
supplies the prior-year deduction even when current employer deposits bar a
deduction for current personal deposits; it never becomes a new deposit.
The remaining excess uses the existing year-end account-value limit and 6%
tax. See [Form 5329 instructions](https://www.irs.gov/instructions/i5329),
Part VI, line 35.

A qualified Form 1099-R code W cash-value charge, alone or with code 6, retains its actual box 8a investment reduction in the original retirement statement. Its gross is excluded from reported pension income and never becomes an additional medical or business premium deduction. Zero investment reduction is valid. Another payment uses its actual already-adjusted investment; this charge does not subtract it twice or establish an ordinary annuity-benefit source. A post-annuitization exclusion-ratio change requires the insurer's actual taxable result or an independently supported calculation. See the [2026 recipient instructions](https://www.irs.gov/pub/irs-pdf/f1099r.pdf), [box 8a and code W instructions](https://www.irs.gov/pub/irs-pdf/i1099r.pdf), and [Notice 2011-68](https://www.irs.gov/irb/2011-36_IRB).

A nonqualified long-term-care receipt with unresolved tax character remains an accepted income source and prevents a complete estimate for the affected return. It is not an optional deduction to discard. Omit only an unverified optional exclusion when an existing owner independently retains the actual includible income; otherwise establish the accident/health, life or annuity treatment before claiming complete tax.

LTC receipts retain separate actual insurance and accelerated-life or viatical origins for boxes 1 and 2. Contract qualification reuses the shared qualified/grandfathered contract facts. Nonqualified accident/health receipts use actual after-tax versus tax-free employer premiums, the prescribed individual/group-policy ratio, eligible medical reimbursement and permanent-injury facts. They are calculated outside Form 8853 Section C’s shared per-diem limit; additional actual interest remains taxable interest. Missing mandatory receipt character withholds the complete estimate. An actual same-owner annuity or prior-year recovery names its accepted source ID and reuses that owner’s calculation once. The readonly source choices show actual owner and gross; native candidate validation checks source identity and relevant gross bounds, while the server rebuilds source facts rather than trusting echoed catalogs. Source deletion or reassignment requires changing/removing the link, and both records can be removed together in the normal candidate lifecycle. A linked completed annuity retains independently established taxable income when a separate LTC exclusion is unavailable; a nominal source ID never establishes a new exclusion. Import reads the printed boxes and does not fabricate these legal/source facts.

Form 8379 keeps interest paid with LTC benefits with its actual recipient, separately from taxable care benefits. A qualified care-period record does not obscure another source’s investment-income allocation. A linked annuity retains its original taxable calculation once. Correct the payer statement, recipient or source link in Other income; normal Save, reopen and restart preserve those facts. Unresolved mandatory receipt character continues to prevent a complete estimate.

For each periodic-payment interval, the insured's filing unit answers whether
another policyholder received qualifying payments for that same insured and
period. A spouse on the joint return belongs to the same filing unit. A sole
payee retains the full raw limit; with other payees, the insured's filing unit
receives its priority allocation. Other policyholders share the remainder by
their actual payments. Keep that raw allocation separate from the exclusion,
which cannot exceed this return's payments, and from taxable receipts. An
unanswered payee question does not establish sole-payee status.


HSA monthly contribution eligibility follows actual first-day coverage and Medicare status, including retroactive enrollment. The coverage choices explain the 2026 bronze/catastrophic, telehealth and qualifying direct-primary-care exceptions. Direct-primary-care fees above the contribution-eligibility limit can still be qualified HSA medical expenses when the actual arrangement and payment meet those separate rules; employer-paid fees are excluded from the beneficiary's expense. These are eligibility and actual-use facts in the existing HSA and distribution editors, with no duplicate medical-expense ledger.

HSA contribution details use the existing owner calculation and retain its monthly and December limit branches, annual Archer reduction, separate married-family catch-up and prior excess deduction. Independently rounded cents detail can differ by a cent from its sum; the combined owner limit retains its original numerator and the estimate keeps its existing rounding. These details do not create another contribution ledger or calculation.

Historical last-month recapture keeps monthly fractions through subtraction and retains cents for Marketplace work income. Form 8889 Part III and estimate transfer rows round separately; original prior-year contributions remain historical facts and never become current deposits.

HSA distribution details retain actual cents and dated exception classes in the on-screen Form 8889 calculation. Separate rows identify the rounded owner income and additional tax used by the estimate; independent spouses remain separate. Pledged collateral enters income directly, while an account terminated by a prohibited transaction contributes its January 1 value to the distribution form. Actual linked returned-excess statements and unlinked completed corrections enter gross distributions and their exclusion once. These detailed calculations are not filing-ready rounded forms.

Foreign income taxes can be claimed as an explicit credit or itemized deduction
on More credits. The deduction reads each retained 1099-INT/DIV foreign-tax component,
qualified estate or trust payee statement and attached K-3 once, then adds
only taxes entered on no such source. Nominee and later-recipient amounts
stay with their existing owners; instrument records supply retained tax.
K-1 basis-suspended taxes stay out. Additional taxes distinguish the person’s
foreign earnings, portfolio income, passive activity income and other income;
taxes on excluded earnings use that person’s net Form 2555 amounts. The choice
survives income saves, estimates, restart and deduction-method changes.

The deduction goes to Schedule A line 6 outside SALT and is added back on Form
6251 line 2a. Known investment taxes also feed Form 8960 line 9b; portfolio taxes
reduce Form 4952 net investment income while passive activity taxes do not.
K-3 credit categories alone do not establish NIIT or Form 4952 attribution:
an uncertain NIIT benefit is omitted with a source notice, and an unresolved
allocation that can change Form 4952 prevents a complete estimate. Released
K-1 foreign-tax basis carries have no current K-3 country/tax source and are not
invented as current K-3 taxes. Sanctioned-country taxes in an ordinary credit
claim retain their source and may still be deducted under section 901(j)(3).
Direct source rows under an accrual credit election do not establish cash payment:
their potential sanctioned-country deduction is omitted with a named notice and
its unresolved portfolio-expense share remains in the Form 4952 completeness
check. Attached K-3 taxes instead retain the entity's paid/accrued treatment.
An estate statement requiring unentered allocation keeps its existing source
notice and omitted benefit; its qualified-payee treatment remains distinct.
A standard-deduction result keeps the chosen foreign-tax deduction and explains
that it gives no benefit; the App does not select the credit automatically.

The new deduction answers come from personal tax records and have no AI Import
selection. Existing document import continues to own 1099 and K-3 tax boxes;
Search routes “deduct foreign taxes,” “foreign tax deduction,” and “Schedule A
line 6 foreign income tax” to More credits.

Foreign portfolio taxes join the existing regular investment-interest expense
pool; AMT adds them back. Source-specific royalty borrowing keeps its separate
Schedule E and Schedule A destinations. Known itemizing expenses can be used
before AGI; an automatic choice or unresolved expense proceeds only when both
allowance/carry endpoints agree. Otherwise the existing named borrowing-benefit
notice leaves that deduction and election out, retaining source income and
ordinary expenses. See [Publication 514](https://www.irs.gov/publications/p514),
[Form 4952 instructions](https://www.irs.gov/pub/irs-pdf/f4952.pdf) and
[Form 8960 instructions](https://www.irs.gov/instructions/i8960).

## Injured-spouse rate elections

A Form 8379 source allocation rounds the selected injured spouse's column
first within each actual native joint amount; the other spouse receives the
exact residual. This also applies when the second spouse is injured and to
signed capital losses.

Qualified small-business stock allocations retain each source's actual
acquisition-date exclusion and admitted 28% rate amount. A fully excluded
source receives no taxable gain from another spouse's partial exclusion.
Changing a source date recalculates its amounts through the normal editor;
source identities and the joint totals remain reconciled.

Education, ABLE and medical-account distributions use their actual recipient,
which can differ from the student or insured person. Their taxable amounts,
exceptions, contribution corrections and additional taxes retain the owning
form's expense coordination and rounding. A correction without an income
document still belongs to its actual account owner; it does not create another
income document. Correcting a recipient uses normal Save, reopen and restart
and recalculates the allocation against the same ordinary joint return.

A Form 8379 allocation keeps the joint return's actual Form 4952 capital and
qualified-dividend elections. These elections change preferential tax treatment,
not income or Form 1040 line 3a. A sole eligible source owner determines the
allocation automatically. When both spouses have eligible sources, use the
existing shared amount rows to enter the injured spouse's elected part; the
other spouse receives the remainder. Bounds use the actual investment pool and
source capacities, including retained investment losses. Personal-use gains
cannot increase that pool. Do not infer election ownership from a loan borrower
or an investment-interest deduction's destination.

The selected parts survive normal Save, reopen and restart. Reject duplicate,
stale or out-of-bound component answers against regenerated native bounds;
a correction may reopen the current editor. An unresolved source leaves the
Form 8379 liability estimate unavailable while preserving the ordinary joint
estimate. Follow the [Form 4952 instructions](https://www.irs.gov/pub/irs-pdf/f4952.pdf)
for the election's classes and rate treatment; this App does not file the election.

Form 8379 keeps account additional taxes with the actual IRA owner or account
beneficiary after the native contribution limits, corrections, account-value
caps and form rounding. A Coverdell contributor is not necessarily its
beneficiary. Schedule H taxes follow the named household employer, including
employee income tax withheld through that employer's payroll. Those employee
amounts remain Schedule H tax; they do not become either spouse's personal
withholding credit. These allocations reuse the existing IRA and Other taxes
records without another account or payroll ledger.

Direct business and rental property retain their native Form 4255 recapture
tax with the actual source owner. Correcting that owner or the earlier credit
used through the existing property editor recalculates the same tax source;
it does not introduce another allocation question. Final property-income
attribution remains independently required for a complete injured-spouse
refund estimate.

Direct business and rental property keep the existing Form 4797 ordinary,
long-term and unrecaptured gain characters with the actual source owner after
activity limits. A shared lookback amount is allocated only when its source
capacities determine the split. An unresolved character or later loss-limit
allocation leaves the affected injured-spouse benefit unavailable.

Earlier installment sales and related-party exchanges of business or rental
property use their recorded seller for the same final property characters.
Correct the seller, payment or recognized deferred gain in Investment sales;
the joint recognition, lookback and rounding remain unchanged. A shared
lookback without a determined source split still leaves the affected
injured-spouse benefit unavailable.

Direct royalties keep the actual source owner's net after the existing
expense and depletion calculations and the borrowing interest charged to
that royalty. Form 8379 uses that same net for other income and net investment
income. Correct the owner, receipts or borrowing through their existing pages;
the joint return's calculation and rounding stay the same. A signed net
investment-income allocation or another unresolved investment deduction can
still leave the injured-spouse benefit unavailable.


Crop-share farm rentals keep each source's actual allowed Form 4835 amount
with its owner after depreciation, prior losses and passive limits.
Form 8379 uses the same final amount for line 13b and the net investment
income allocation behind line 18. Correct the owner, receipts or expenses
in Farming; normal Save, reopen and restart retain those source records.
A negative owner investment-income result or another unresolved investment
deduction can still leave the affected tax allocation unavailable. The
joint return continues to use its existing form calculations.

Government unemployment retains the actual taxable benefit after same-year
repayments. RTAA, personal taxable grants and family leave stay with that
recipient; payments routed to a business or farm enter that destination once.
Correct the form and repayments in Unemployment and state refunds, or jury
pay and its employer remittance in Other income. Normal Save, reopen and
restart retain the corrected recipient and amounts. These accepted sources,
taxable alimony, educational scholarships and casual gambling keep their
existing noninvestment treatment while Form 8379 allocates another source's
actual investment tax. The joint return's figures and ordinary source limits
remain unchanged.


Mortgage subsidy recapture on a sale follows its matching main-home sale
owner. A nonspouse gift records whose interest was given away: taxpayer,
spouse or both. On a joint return with Form 8828 recapture tax, this answer is
required and checked on Save. A missing answer keeps the source page in
Review, including after an income correction creates recapture tax. No owner
is assumed, and the joint Form 8828 calculation stays unchanged. With no joint
recapture tax the null answer applies; on a return that is not joint the empty
allowed set supplies that answer and withholds the question. Both spouses'
interests use the existing bounded Form 8379 amount row; any exclusive tax
stays with its actual owner and is outside the shared part.

## Retained property and farmer gross income

In Investment sales, an earlier business-property installment sale asks for
its original livestock classification. Use the sale-year records and Form 4797.
The current recognized gain enters farmer gross income only when those records
identify qualifying livestock; the unavailable answer leaves that test unknown.
A nonlivestock answer also leaves the test unknown when the original farm
use is not established: farm machinery gains can qualify too. Current machinery
already attached to a farm uses its recognized gain, including immediate
recapture and the current installment amount; farmland gain stays outside the
farming numerator. A positive farm real-property gain without established
farming character leaves qualification unknown. See [Publication 225, chapter 15](https://www.irs.gov/publications/p225)
and [Publication 505](https://www.irs.gov/publications/p505). This classification does not change the sale's gain or character.
Normal Save, reopen and restart retain the answer. Related 2024–2025 real-property
exchanges are known nonlivestock; a QOF's section 1231 code alone cannot establish
the original property's classification. Prior-year qualification remains independent.
