use super::*;

// Each category has exactly one actual owner. Expected amounts below are sums
// of the native independently rounded categories, not a new shared-rest_rounding_source
// allocation convention. Stored inputs are integer cents:49/50/51c surround
// the whole-dollar boundary; no fraction of a stored cent is invented.

#[test]
fn unit13_rest_rounding_single_category_49_50_51_cent_controls() {
    for (index, kind) in [
        RestRoundingSource::Interest,
        RestRoundingSource::Dividend,
        RestRoundingSource::Jury,
        RestRoundingSource::Unemployment,
    ]
    .into_iter()
    .enumerate()
    {
        for (cents, rounded) in [(49, 0), (50, 1), (51, 1)] {
            for owner in [Owner::Taxpayer, Owner::Spouse] {
                let mut categories = [0; 4];
                categories[index] = rounded;
                let owners = if owner == Owner::Taxpayer { [rounded, 0] } else { [0, rounded] };
                assert_rest_rounding_sources(
                    &[(kind, owner, cents)],
                    categories,
                    owners,
                    Owner::Taxpayer,
                );
            }
        }
    }
}

#[test]
fn unit13_rest_rounding_two_exclusive_categories_keep_native_joint_footing() {
    assert_rest_rounding_sources(
        &[
            (RestRoundingSource::Interest, Owner::Taxpayer, 50),
            (RestRoundingSource::Jury, Owner::Spouse, 50),
        ],
        [1, 0, 1, 0],
        [1, 1],
        Owner::Taxpayer,
    );
}

#[test]
fn unit13_rest_rounding_three_exclusive_categories_keep_native_joint_footing() {
    assert_rest_rounding_sources(
        &[
            (RestRoundingSource::Interest, Owner::Taxpayer, 50),
            (RestRoundingSource::Dividend, Owner::Spouse, 50),
            (RestRoundingSource::Jury, Owner::Taxpayer, 50),
        ],
        [1, 1, 1, 0],
        [2, 1],
        Owner::Taxpayer,
    );
}

#[test]
fn unit13_rest_rounding_matching_grand_total_keeps_exclusive_category_owners() {
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        assert_rest_rounding_sources(
            &[
                (RestRoundingSource::Interest, Owner::Taxpayer, 50),
                (RestRoundingSource::Dividend, Owner::Taxpayer, 50),
                (RestRoundingSource::Jury, Owner::Spouse, 49),
                (RestRoundingSource::Unemployment, Owner::Spouse, 49),
            ],
            [1, 1, 0, 0],
            [2, 0],
            injured,
        );
    }
}

#[test]
fn unit13_rest_rounding_mixed_categories_matching_owner_totals_control() {
    assert_rest_rounding_sources(
        &[
            (RestRoundingSource::Interest, Owner::Taxpayer, 49),
            (RestRoundingSource::Dividend, Owner::Spouse, 50),
            (RestRoundingSource::Jury, Owner::Taxpayer, 51),
        ],
        [0, 1, 1, 0],
        [1, 1],
        Owner::Taxpayer,
    );
}
