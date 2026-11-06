use alloc::string::String;

use crate::tests::*;

pub(in crate::tests) fn student_credit_paper(form: StudentCredit) -> Form1098TPaper {
    let StudentCredit {
        filer,
        student,
        tuition,
        scholarships,
        half_time,
        graduate,
        prior_tuition_adjustment,
        prior_scholarship_adjustment,
    } = form;
    Form1098TPaper {
        filer,
        student,
        box_1: tuition,
        box_5: scholarships,
        box_8: half_time,
        box_9: graduate,
        box_4: match prior_tuition_adjustment {
            SourceCents::Reported { amount } => Some(amount),
            SourceCents::NotReported => None,
            SourceCents::Unavailable => {
                panic!("Cannot encode an unavailable prior tuition adjustment as Form 1098-T box 4")
            }
        },
        box_6: match prior_scholarship_adjustment {
            SourceCents::Reported { amount } => Some(amount),
            SourceCents::NotReported => None,
            SourceCents::Unavailable => panic!(
                "Cannot encode an unavailable prior scholarship adjustment as Form 1098-T box 6"
            ),
        },
        calendar_year: 2026,
        corrected: false,
        filer_telephone: String::new(),
        account_number: String::new(),
        box_7: false,
        box_10: 0,
    }
}

pub(in crate::tests) fn education_source(data: FederalEducation) -> EducationSource {
    let FederalEducation {
        id,
        student,
        tuition,
        scholarships,
        half_time,
        graduate,
        prior_tuition_adjustment,
        prior_scholarship_adjustment,
    } = data;
    EducationSource {
        id,
        student,
        assigned: crate::DocumentRecipient::from_key(student)
            .unwrap_or(crate::DocumentRecipient::Unknown),
        owner_source: Some(crate::OwnerSource::Added),
        joint_share: None,
        form: student_credit_paper(StudentCredit {
            filer: Default::default(),
            student: Default::default(),
            tuition,
            scholarships,
            half_time,
            graduate,
            prior_tuition_adjustment,
            prior_scholarship_adjustment,
        }),
    }
}
