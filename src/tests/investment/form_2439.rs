use super::*;

// Every printed box and topic answer of a stored Form 2439 lands once in the
// internal undistributed-gain model; the party blocks reach no federal figure.
#[test]
fn form_2439_converter_carries_every_box_into_the_internal_model() {
    let source = UndistributedGainSource {
        form: form2439_paper(Form2439 {
            gain: 100,
            unrecaptured: 200,
            small_business: 300,
            collectibles: 400,
            tax_paid: 500,
            ..Default::default()
        }),
        section_1202_exclusion: Some(FundQsbsExclusion::Fifty),
        special: true,
    };
    assert_eq!(
        UndistributedGain::from(&source),
        UndistributedGain {
            gain: 100,
            unrecaptured: 200,
            small_business: 300,
            section_1202_exclusion: Some(FundQsbsExclusion::Fifty),
            collectibles: 400,
            tax_paid: 500,
            special: true,
        }
    );
}
