use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Tax form
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
)]
pub enum TaxForm {
    /// Form 1040
    Form1040,
    /// Schedule 1 (Form 1040)
    Schedule1,
    /// Schedule 1-A (Form 1040)
    Schedule1A,
    /// Schedule 2 (Form 1040)
    Schedule2,
    /// Schedule 3 (Form 1040)
    Schedule3,
    /// Schedule 8812 (Form 1040)
    Schedule8812,
    /// Schedule A (Form 1040)
    ScheduleA,
    /// Schedule B (Form 1040)
    ScheduleB,
    /// Schedule C (Form 1040)
    ScheduleC,
    /// Schedule D (Form 1040)
    ScheduleD,
    /// Schedule E (Form 1040)
    ScheduleE,
    /// Schedule EIC (Form 1040)
    ScheduleEic,
    /// Schedule F (Form 1040)
    ScheduleF,
    /// Schedule H (Form 1040)
    ScheduleH,
    /// Schedule J (Form 1040)
    ScheduleJ,
    /// Schedule R (Form 1040)
    ScheduleR,
    /// Schedule SE (Form 1040)
    ScheduleSe,
    /// Form 982
    Form982,
    /// Form 1040-ES
    Form1040Es,
    /// Form 1062
    Form1062,
    /// Form 1116
    Form1116,
    /// Schedule B (Form 1116)
    Form1116ScheduleB,
    /// Form 2439
    Form2439,
    /// Form 2106
    Form2106,
    /// Form 2441
    Form2441,
    /// Form 2555
    Form2555,
    /// Form 3468
    Form3468,
    /// Form 3800
    Form3800,
    /// Form 4136
    Form4136,
    /// Form 4137
    Form4137,
    /// Form 4255
    Form4255,
    /// Form 4562
    Form4562,
    /// Form 4562-B
    Form4562B,
    /// Form 4684
    Form4684,
    /// Form 4797
    Form4797,
    /// Form 4835
    Form4835,
    /// Form 4952
    Form4952,
    /// Form 4972
    Form4972,
    /// Form 5329
    Form5329,
    /// Form 5695
    Form5695,
    /// Form 5884
    Form5884,
    /// Form 6198
    Form6198,
    /// Form 6251
    Form6251,
    /// Form 6252
    Form6252,
    /// Form 6781
    Form6781,
    /// Form 7203
    Form7203,
    /// Form 7206
    Form7206,
    /// Form 8283
    Form8283,
    /// Form 8379
    Form8379,
    /// Form 8396
    Form8396,
    /// Form 8582
    Form8582,
    /// Form 8582-CR
    Form8582Cr,
    /// Form 8606
    Form8606,
    /// Form 8615
    Form8615,
    /// Form 8814
    Form8814,
    /// Form 8801
    Form8801,
    /// Form 8815
    Form8815,
    /// Form 8824
    Form8824,
    /// Form 8829
    Form8829,
    /// Form 8834
    Form8834,
    /// Form 8839
    Form8839,
    /// Form 8846
    Form8846,
    /// Form 8853
    Form8853,
    /// Form 8859
    Form8859,
    /// Form 8862
    Form8862,
    /// Form 8863
    Form8863,
    /// Form 8880
    Form8880,
    /// Form 8881
    Form8881,
    /// Form 8889
    Form8889,
    /// Form 8911
    Form8911,
    /// Form 8919
    Form8919,
    /// Form 8936
    Form8936,
    /// Form 8941
    Form8941,
    /// Form 8949
    Form8949,
    /// Form 8958
    Form8958,
    /// Form 8959
    Form8959,
    /// Form 8960
    Form8960,
    /// Form 8962
    Form8962,
    /// Form 8978
    Form8978,
    /// Form 8994
    Form8994,
    /// Form 8995
    Form8995,
    /// Form 8995-A
    Form8995A,
    /// Form W-2
    FormW2,
    /// Form W-2G
    FormW2G,
    /// Form 1095-A
    Form1095A,
    /// Form 1098
    Form1098,
    /// Form 1098-E
    Form1098E,
    /// Form 1098-T
    Form1098T,
    /// Form 1099-B
    Form1099B,
    /// Form 1099-C
    Form1099C,
    /// Form 1099-DIV
    Form1099Div,
    /// Form 1099-G
    Form1099G,
    /// Form 1099-INT
    Form1099Int,
    /// Form 1099-K
    Form1099K,
    /// Form 1099-MISC
    Form1099Misc,
    /// Form 1099-NEC
    Form1099Nec,
    /// Form 1099-Q
    Form1099Q,
    /// Form 1099-R
    Form1099R,
    /// Form 1099-SA
    Form1099Sa,
    /// Form 5498
    Form5498,
    /// Schedule K-1 (Form 1041)
    ScheduleK1Form1041,
    /// Schedule K-1 (Form 1065)
    ScheduleK1Form1065,
    /// Schedule K-1 (Form 1120-S)
    ScheduleK1Form1120S,
    /// Form 8912
    Form8912,
    /// Form 8915-F
    Form8915F,
}

/// How this review row participates in its printed line
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum FormLineKind {
    /// Printed line amount
    Line,
    /// Source or column included in the printed line
    Subrow,
}

/// Form line
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FormLine {
    /// Form
    pub form: TaxForm,
    /// Line
    pub line: String,
    /// Printed line or supporting source
    pub kind: FormLineKind,
}

impl FormLine {
    pub fn new(form: TaxForm, line: &str) -> Self {
        Self { form, line: line.into(), kind: FormLineKind::Line }
    }
    pub fn subrow(form: TaxForm, line: &str) -> Self {
        Self { form, line: line.into(), kind: FormLineKind::Subrow }
    }
    // The printed order of a line: its leading number, then the rest
    // ("8" before "8a" before "10"; "19i" before "26(a)").
    pub fn order(&self) -> (u32, String) {
        if self.form == TaxForm::Form8915F && matches!(self.line.as_str(), "A" | "B") {
            return (0, self.line.clone());
        }
        let digits =
            self.line.len() - self.line.trim_start_matches(|c: char| c.is_ascii_digit()).len();
        (self.line[..digits].parse().unwrap_or(u32::MAX), self.line[digits..].into())
    }
}

// The printed line of a "line<ref>_<name>" expense suffix ("line24b_meals"
// is line 24b), shared by the Schedule C and Schedule E expense categories.
pub(crate) fn printed_line(suffix: &str) -> &str {
    let rest = suffix.strip_prefix("line").unwrap_or(suffix);
    rest.split_once('_').map_or(rest, |(line, _)| line)
}
