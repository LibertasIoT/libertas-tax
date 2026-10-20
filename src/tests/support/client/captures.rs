use crate::TaxInterviewProtocol as P;
use alloc::{string::String, vec::Vec};

// Explicit historical fixture fields, never a decoder for current payloads.
// SDK leaf encodings are inserted at measured old-schema field boundaries;
// their exact inverse must recover every original capture byte.
pub(in crate::tests) fn captured_with_fields<T: libertas::AvroDecode + libertas::AvroEncode>(
    original: &[u8],
    fields: &[(usize, usize, Vec<u8>)],
) -> T {
    let mut current = Vec::new();
    let mut undo = Vec::new();
    let mut previous = 0;
    for (offset, old_length, value) in fields {
        assert!(*offset >= previous && *offset + *old_length <= original.len());
        current.extend_from_slice(&original[previous..*offset]);
        undo.push((current.len(), value.len(), *offset, *old_length));
        current.extend_from_slice(value);
        previous = *offset + *old_length;
    }
    current.extend_from_slice(&original[previous..]);
    let mut offset = 0;
    let value = T::avro_decode(&current, &mut offset).expect("historical fields in current schema");
    assert_eq!(offset, current.len());
    let mut encoded = Vec::new();
    value.avro_encode(&mut encoded);
    assert_eq!(encoded, current, "exact current captured value bytes");
    for (offset, length, old_offset, old_length) in undo.into_iter().rev() {
        drop(encoded.splice(
            offset..offset + length,
            original[old_offset..old_offset + old_length].iter().copied(),
        ));
    }
    assert_eq!(encoded, original, "every original capture byte survives");
    value
}

/// Read immutable requests from before the activity graph and benefit writer.
/// Preserve every original byte and fixture offset; current client payload
/// overrides use the current decoder directly.
pub(in crate::tests) fn captured_request_before_k1_activity_context(bytes: &[u8]) -> P {
    let fields = captured_request_fields(bytes);
    if fields.is_empty() {
        let request = P::from_avro(bytes).expect("unchanged captured request");
        assert_eq!(request.to_avro(), bytes);
        return request;
    }
    let request: P = captured_with_fields(bytes, &fields);
    match &request {
        P::SaveIncomes { empty_k1_activity_choices, .. } => {
            assert_eq!(empty_k1_activity_choices, &super::builders::empty_k1_activity_choices())
        }
        P::SaveSelfEmployedBenefits { value, .. } => {
            assert!(value.activity_partnerships.is_empty());
            assert!(value.activity_scorporations.is_empty());
        }
        P::SavePayments { .. }
        | P::EstimatePayments { .. }
        | P::SaveInjuredSpouse { .. }
        | P::EstimateInjuredSpouse { .. } => {}
        _ => unreachable!(),
    }
    request
}

// The pre-nullable Rental capture answered the old required All
// variant. Its offsets were measured by typed traversal of the original
// request schema. Preserve All, not the new nullable tag's None meaning.
pub(in crate::tests) fn captured_rental_request_before_nullable_risk(
    original: &[u8],
    risk_offset: usize,
) -> P {
    use libertas::AvroEncode;
    assert_eq!(original[risk_offset], 0, "original All variant tag");
    let mut risk = Vec::new();
    Some(crate::RentalAtRisk::All).avro_encode(&mut risk);
    let mut fields = captured_request_fields(original);
    fields.extend(captured_rental_history_fields(original, risk_offset));
    fields.push((risk_offset, 1, risk));
    fields.sort_by_key(|(offset, _, _)| *offset);
    let request: P = captured_with_fields(original, &fields);
    let P::SaveIncomes { documents, .. } = &request else {
        panic!("historical rental income request")
    };
    let rentals: Vec<_> = documents
        .documents
        .iter()
        .filter_map(|item| match &item.document {
            crate::IncomeDocument::Rental(page) => Some(page),
            _ => None,
        })
        .collect();
    assert_eq!(rentals.len(), 1, "the original single-rental capture");
    assert_eq!(rentals[0].more.at_risk, Some(crate::RentalAtRisk::All));
    assert_eq!(rentals[0].building.casualty, None);
    assert_eq!(rentals[0].building.sale_history, None);
    assert!(rentals[0].more.property_qbi_history.is_empty());
    request
}

// Traverse the original SaveIncomes / IncomeDocuments / Rental owners. The
// current fields prepend Building.casualty/sale_history and RentalDetails.property_qbi_history;
// this retained editor supplied neither. Use absence, never inferred records.
fn captured_rental_history_fields(
    original: &[u8],
    risk_offset: usize,
) -> Vec<(usize, usize, Vec<u8>)> {
    use libertas::{AvroDecode, AvroEncode};
    let mut offset = 0;
    macro_rules! skip {
        ($field:ident: $ty:ty) => {
            let _ = <$ty>::avro_decode(original, &mut offset).expect(stringify!($field));
        };
    }
    assert_eq!(
        i32::avro_decode(original, &mut offset).unwrap(),
        libertas_macros::variant_index!(P::SaveIncomes) as i32,
    );
    skip!(qbi_sources: Vec<crate::QbiAggregationMemberChoice>);
    skip!(allowed_injured_source_facts: Vec<i32>);
    skip!(allowed_injured_retained_owners: Vec<i32>);
    skip!(ltc_payee_sources: Vec<crate::RecordChoice>);
    skip!(wage_scorporations: Vec<crate::RecordChoice>);
    skip!(allowed_wage_scorporations: Vec<i32>);
    skip!(initial_k1_investment_choices: Vec<i32>);
    skip!(research_businesses: Vec<crate::RecordChoice>);
    skip!(retained_research_choices: Vec<i32>);
    skip!(k1_credit_lines: Vec<i32>);
    skip!(research_credit_lines: Vec<i32>);
    skip!(renewable_credit_lines: Vec<i32>);
    skip!(transferable_credit_lines: Vec<i32>);
    skip!(user: libertas::LibertasUser);
    skip!(allowed_continuing_risk: Vec<i32>);
    skip!(cookie: String);
    skip!(revision: i64);
    skip!(page: i32);
    skip!(previous: i32);
    skip!(review: bool);
    skip!(estimate: Option<Vec<u8>>);
    skip!(warnings: Vec<crate::SaveWarning>);
    skip!(progress: Option<Vec<u8>>);
    skip!(topic: crate::TaxSection);
    skip!(allowed_documents: Vec<i32>);
    skip!(allowed_owners: Vec<i32>);
    skip!(allowed_community: Vec<i32>);
    skip!(allowed_refund_management: Vec<i32>);
    skip!(foreign_countries: Vec<crate::RecordChoice>);
    skip!(interest_statements: Vec<crate::RecordChoice>);
    skip!(people: Vec<crate::PersonChoice>);
    skip!(farms: Vec<crate::RecordChoice>);
    skip!(cooperative_activities: Vec<crate::RecordChoice>);
    skip!(cooperative_properties: Vec<crate::RecordChoice>);
    skip!(businesses: Vec<crate::RecordChoice>);
    skip!(qbi_businesses: Vec<crate::RecordChoice>);
    skip!(rentals: Vec<crate::RecordChoice>);
    skip!(mortgage_loans: Vec<crate::RecordChoice>);
    skip!(royalty_properties: Vec<crate::RecordChoice>);
    skip!(equipment_rentals: Vec<crate::RecordChoice>);
    skip!(hobbies: Vec<crate::RecordChoice>);
    skip!(ccc_lots: Vec<crate::RecordChoice>);
    skip!(farm_codes: Vec<crate::FarmCodeChoice>);
    skip!(farm_limitations: Vec<crate::FarmCodeChoice>);
    skip!(farm_sources: Vec<crate::FarmChoice>);
    skip!(basis_assets: Vec<crate::FarmChoice>);
    skip!(farm_method_changes: Vec<crate::FarmChoice>);
    skip!(inherited_iras: Vec<crate::RecordChoice>);
    skip!(inherited_roth_iras: Vec<crate::RecordChoice>);
    skip!(ltc_insured: Vec<crate::RecordChoice>);
    skip!(annuity_sources: Vec<crate::LtcSourceChoice>);
    skip!(recovery_sources: Vec<crate::LtcSourceChoice>);
    skip!(allowed_business_source: Vec<i32>);
    skip!(allowed_farm_source: Vec<i32>);
    skip!(allowed_partner_source: Vec<i32>);
    skip!(allowed_home_spouse: Vec<i32>);
    skip!(allowed_other_parent: Vec<i32>);
    skip!(children: Vec<crate::ChildChoice>);
    skip!(document_facts: crate::DocumentFacts);
    skip!(receipt_context: crate::BusinessReceiptContext);
    assert_eq!(i64::avro_decode(original, &mut offset).unwrap(), 1, "one captured row");
    skip!(investment_caps: Option<crate::InvestmentCapitalCap>);
    skip!(id: i64);
    skip!(owner: crate::Owner);
    assert_eq!(
        i32::avro_decode(original, &mut offset).unwrap(),
        libertas_macros::variant_index!(crate::IncomeDocument::Rental) as i32,
    );
    skip!(address: String);
    skip!(kind: crate::RentalPropertyType);
    skip!(rent: i64);
    skip!(mortgage_loans: Vec<i64>);
    skip!(expenses: crate::RentalExpenses);
    let building = offset;
    // Original Building before sale history: exact field order and native widths.
    skip!(cost: i64);
    skip!(credit_reduction: i64);
    skip!(land: i64);
    skip!(in_service: u32);
    let transition = offset;
    skip!(acquired: crate::RentalAcquisition);
    skip!(partial: Option<crate::PartialDisposition>);
    skip!(assets: Vec<crate::CanonicalAssetPage>);
    skip!(sale: crate::RentalSale);
    skip!(statement: Option<crate::Rental1099S>);
    skip!(personal_use: crate::PersonalUse);
    let details = offset;
    // This capture has no improvement rows, whose own shape also changed.
    assert!(Vec::<crate::Improvement>::avro_decode(original, &mut offset).unwrap().is_empty());
    skip!(participation: crate::RentalParticipation);
    skip!(suspended: crate::SuspendedLoss);
    skip!(method_changes: Vec<crate::MethodChange>);
    skip!(home_operating_carryover: i64);
    let prior_home_carry = offset;
    assert_eq!(
        i64::avro_decode(original, &mut offset).unwrap(),
        0,
        "synthetic capture has no prior home-use carry"
    );
    let prior_home_carry_end = offset;
    skip!(qbi: crate::RentalBusiness);
    skip!(qbi_group: String);
    skip!(payroll: crate::BusinessPayroll);
    assert_eq!(offset, risk_offset, "original RentalDetails.at_risk boundary");
    let mut building_defaults = Vec::new();
    Option::<crate::RentalCasualty>::None.avro_encode(&mut building_defaults);
    0_i64.avro_encode(&mut building_defaults);
    Option::<crate::RentalEnergyClaimPage>::None.avro_encode(&mut building_defaults);
    Option::<crate::RentalBuildingSaleHistory>::None.avro_encode(&mut building_defaults);
    let mut no_qbi_history = Vec::new();
    Vec::<crate::RentalEnergyBuildingFacts>::new().avro_encode(&mut no_qbi_history);
    Vec::<crate::RentalPropertyLossYear>::new().avro_encode(&mut no_qbi_history);
    // The actual historical capture has no documentary transition answer.
    // Preserve that absence and the derived empty choices for its 2020 date.
    let mut no_transition = Vec::new();
    Option::<crate::Commercial1993Transition>::None.avro_encode(&mut no_transition);
    Vec::<i32>::new().avro_encode(&mut no_transition);
    let mut actual_no_prior_carry = Vec::new();
    //2026-10-19: this fixture's independent regular/AMT source records establish no carry.
    for _ in 0..4 {
        crate::SourceCents::NotReported.avro_encode(&mut actual_no_prior_carry);
    }
    alloc::vec![
        (prior_home_carry, prior_home_carry_end - prior_home_carry, actual_no_prior_carry),
        (building, 0, building_defaults),
        (transition, 0, no_transition),
        (details, 0, no_qbi_history)
    ]
}

// Original-schema offsets let request additions and the asset owner's fields
// compose in one exact inverse projection.
fn captured_request_fields(bytes: &[u8]) -> Vec<(usize, usize, Vec<u8>)> {
    use libertas::{AvroDecode, AvroEncode};
    let mut offset = 0;
    let variant = i32::avro_decode(bytes, &mut offset).expect("captured request variant");
    if variant == libertas_macros::variant_index!(P::SaveIncomes) as i32 {
        let rental_choices = offset;
        let mut initial_rental_choices = Vec::new();
        Vec::<i32>::new().avro_encode(&mut initial_rental_choices);
        Vec::<crate::QbiAggregationMemberChoice>::avro_decode(bytes, &mut offset)
            .expect("captured QBI source choices");
        let mut additions = Vec::new();
        Vec::<crate::RecordChoice>::new().avro_encode(&mut additions);
        super::builders::empty_k1_activity_choices().avro_encode(&mut additions);
        return alloc::vec![(rental_choices, 0, initial_rental_choices), (offset, 0, additions)];
    }
    let payments = variant == libertas_macros::variant_index!(P::SavePayments) as i32
        || variant == libertas_macros::variant_index!(P::EstimatePayments) as i32;
    let injured = variant == libertas_macros::variant_index!(P::SaveInjuredSpouse) as i32
        || variant == libertas_macros::variant_index!(P::EstimateInjuredSpouse) as i32;
    if payments || injured {
        libertas::LibertasUser::avro_decode(bytes, &mut offset).unwrap();
        if variant == libertas_macros::variant_index!(P::SavePayments) as i32
            || variant == libertas_macros::variant_index!(P::SaveInjuredSpouse) as i32
        {
            String::avro_decode(bytes, &mut offset).unwrap();
            i64::avro_decode(bytes, &mut offset).unwrap();
            i32::avro_decode(bytes, &mut offset).unwrap();
            i32::avro_decode(bytes, &mut offset).unwrap();
            bool::avro_decode(bytes, &mut offset).unwrap();
            Vec::<u8>::avro_decode(bytes, &mut offset).unwrap();
            Vec::<crate::SaveWarning>::avro_decode(bytes, &mut offset).unwrap();
            Vec::<u8>::avro_decode(bytes, &mut offset).unwrap();
        }
        if payments {
            for _ in 0..4 {
                Vec::<i32>::avro_decode(bytes, &mut offset).unwrap();
            }
            let start = offset;
            let withheld = crate::Withheld::avro_decode(bytes, &mut offset).unwrap();
            let mut value = Vec::new();
            0i64.avro_encode(&mut value);
            Option::<crate::OverpaymentRecipient>::None.avro_encode(&mut value);
            if variant == libertas_macros::variant_index!(P::SavePayments) as i32 {
                Option::<i64>::None.avro_encode(&mut value);
                Some(withheld).avro_encode(&mut value);
            }
            return alloc::vec![(start, offset - start, value)];
        }
        crate::Owner::avro_decode(bytes, &mut offset).unwrap();
        crate::OffsetDebt::avro_decode(bytes, &mut offset).unwrap();
        bool::avro_decode(bytes, &mut offset).unwrap();
        crate::InjuredCommunity::avro_decode(bytes, &mut offset).unwrap();
        crate::EstimatedSplit::avro_decode(bytes, &mut offset).unwrap();
        Option::<crate::InjuredLoanBorrower>::avro_decode(bytes, &mut offset).unwrap();
        Vec::<crate::InjuredAmountPage>::avro_decode(bytes, &mut offset).unwrap();
        Vec::<crate::InjuredAmountPage>::avro_decode(bytes, &mut offset).unwrap();
        Vec::<crate::InjuredClaimant>::avro_decode(bytes, &mut offset).unwrap();
        let start = offset;
        let joint = i64::avro_decode(bytes, &mut offset).unwrap();
        let mut value = Vec::new();
        Some(joint).avro_encode(&mut value);
        return alloc::vec![(start, offset - start, value)];
    }
    if variant != libertas_macros::variant_index!(P::SaveSelfEmployedBenefits) as i32 {
        return Vec::new();
    }
    libertas::LibertasUser::avro_decode(bytes, &mut offset).expect("captured benefit user");
    String::avro_decode(bytes, &mut offset).expect("captured benefit cookie");
    i64::avro_decode(bytes, &mut offset).expect("captured benefit revision");
    i32::avro_decode(bytes, &mut offset).expect("captured benefit page");
    i32::avro_decode(bytes, &mut offset).expect("captured benefit previous page");
    bool::avro_decode(bytes, &mut offset).expect("captured benefit review flag");
    let estimate_start = offset;
    let estimate = Vec::<u8>::avro_decode(bytes, &mut offset).expect("captured benefit estimate");
    let estimate_length = offset - estimate_start;
    Vec::<crate::SaveWarning>::avro_decode(bytes, &mut offset).expect("captured benefit warnings");
    let progress_start = offset;
    let progress = Vec::<u8>::avro_decode(bytes, &mut offset).expect("captured benefit progress");
    let progress_length = offset - progress_start;
    let mut estimate_field = Vec::new();
    Some(estimate).avro_encode(&mut estimate_field);
    let mut progress_field = Vec::new();
    Some(progress).avro_encode(&mut progress_field);
    // The old always-present byte fields remain present, even when empty.
    // SelfEmployedBenefits is the final request field; its two new lists append
    // after the six original fields, without replacing any source facts.
    let mut activities = Vec::new();
    Vec::<crate::K1ActivityPartnerBenefits>::new().avro_encode(&mut activities);
    Vec::<crate::K1ActivityScorpBenefits>::new().avro_encode(&mut activities);
    alloc::vec![
        (estimate_start, estimate_length, estimate_field),
        (progress_start, progress_length, progress_field),
        (bytes.len(), 0, activities),
    ]
}
