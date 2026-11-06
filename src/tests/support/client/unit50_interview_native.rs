extern crate std;

use super::{restart, send};
use crate::interview::{Interview, Reply};
use crate::tests::support::assert::accepted;
use crate::{FederalDraft, TaxInterviewProtocol as P, TaxSection};
use alloc::format;
use libertas::AvroDecode;

// Ordinary native client operations shared by the native interview source scenarios.
// Each test still owns its typed answers and all figure/source assertions.
// Use the existing send owner so capture, reply encoding and atomic-refusal
// observations keep the actual named test thread and its native request.
pub(in crate::tests) struct NativeInterviewCase(pub &'static str);

impl NativeInterviewCase {
    pub(in crate::tests) fn write(&self, suffix: &str, value: &P) {
        if let Ok(path) = std::env::var(format!("{}_{suffix}", self.0)) {
            std::fs::write(path, value.to_avro()).unwrap();
        }
    }

    pub(in crate::tests) fn required_write(&self, suffix: &str, value: &P) {
        let path = std::env::var(format!("{}_{suffix}", self.0)).unwrap();
        std::fs::write(path, value.to_avro()).unwrap();
    }

    pub(in crate::tests) fn input(&self, suffix: &str) -> Option<P> {
        self.read(suffix, false)
    }

    pub(in crate::tests) fn checked_input(&self, suffix: &str) -> Option<P> {
        self.read(suffix, true)
    }

    fn read(&self, suffix: &str, check_bytes: bool) -> Option<P> {
        let path = std::env::var(format!("{}_{suffix}", self.0)).ok()?;
        let bytes = std::fs::read(path).unwrap();
        let mut offset = 0;
        let request = P::avro_decode(&bytes, &mut offset).unwrap();
        assert_eq!(offset, bytes.len());
        if check_bytes {
            assert_eq!(request.to_avro(), bytes);
        }
        Some(request)
    }

    pub(in crate::tests) fn calculation(&self, session: &mut Interview) -> bool {
        self.calculate(session, false, false)
    }

    pub(in crate::tests) fn required_calculation(&self, session: &mut Interview) -> bool {
        self.calculate(session, true, false)
    }

    pub(in crate::tests) fn required_checked_calculation(&self, session: &mut Interview) -> bool {
        self.calculate(session, true, true)
    }

    fn calculate(&self, session: &mut Interview, required_output: bool, check_bytes: bool) -> bool {
        let Some(request) = self.read("CALC_INPUT", check_bytes) else { return false };
        let Reply::Response(reply) = send(session, request) else {
            panic!("{}: native calculation response", self.0)
        };
        if required_output {
            self.required_write("CALC_OUTPUT", &reply);
        } else {
            self.write("CALC_OUTPUT", &reply);
        }
        true
    }

    pub(in crate::tests) fn files(
        &self,
        suffix: &str,
        files: &[(&str, alloc::vec::Vec<u8>)],
    ) -> bool {
        let Ok(directory) = std::env::var(format!("{}_{suffix}", self.0)) else { return false };
        let directory = std::path::Path::new(&directory);
        std::fs::create_dir_all(directory).unwrap();
        for (name, bytes) in files {
            std::fs::write(directory.join(name), bytes).unwrap();
        }
        true
    }

    pub(in crate::tests) fn output(&self, editor: &P, stored: &super::NativeDraftRecords) {
        let Ok(directory) = std::env::var(format!("{}_OUTPUT", self.0)) else { return };
        let directory = std::path::Path::new(&directory);
        std::fs::create_dir_all(directory).unwrap();
        std::fs::write(directory.join("reopened.avro"), editor.to_avro()).unwrap();
        let path = directory.join("persisted.avro");
        super::write_native_draft_records(&path, stored);
        assert_eq!(super::read_native_draft_records(path), *stored);
    }

    pub(in crate::tests) fn write_file(&self, suffix: &str, name: &str, value: &P) {
        self.files(suffix, &[(name, value.to_avro())]);
    }

    pub(in crate::tests) fn seed(&self, value: &P) -> bool {
        self.files("SEED_DIR", &[("seed.avro", value.to_avro())])
    }

    pub(in crate::tests) fn save(&self, session: &mut Interview, answers: P) {
        self.write("ANSWERS", &answers);
        self.submit(session, answers);
    }

    pub(in crate::tests) fn submit(&self, session: &mut Interview, answers: P) {
        let request = self.input("PAYLOAD").unwrap_or(answers);
        accepted(&if matches!(request, P::SaveDocuments { .. }) {
            super::income_pages::save_source_fixture_request(session, request)
        } else {
            send(session, request)
        });
    }

    pub(in crate::tests) fn editor(
        &self,
        session: &mut Interview,
        topic: TaxSection,
        output: &str,
    ) -> P {
        let request = crate::tests::cross::interview::navigate(&session.draft, topic);
        let editor = match send(session, request) {
            Reply::Edit(editor) => editor,
            other => panic!("{}: native editor, got {other:?}", self.0),
        };
        self.write(output, &editor);
        editor
    }

    pub(in crate::tests) fn restart(&self, session: &mut Interview) -> FederalDraft {
        let saved = session.draft.clone();
        *session = restart(saved.clone());
        assert_eq!(session.draft, saved);
        saved
    }
}
