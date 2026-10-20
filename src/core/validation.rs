// Transient native validation failures. No accepted state or schema rule engine.
// Source owners name a local field; existing Save walkers prepend submitted positions.
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

#[derive(Clone, Debug, PartialEq, Eq)]
enum PathSegment {
    Field(&'static str),
    Index(usize),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FieldPath(Vec<PathSegment>);
impl FieldPath {
    fn field(name: &'static str) -> Self {
        Self(alloc::vec![PathSegment::Field(name)])
    }
    fn list_item(name: &'static str, index: usize) -> Self {
        Self(alloc::vec![PathSegment::Field(name), PathSegment::Index(index)])
    }
    fn map_field_prefix(mut self, from: &[&'static str], to: &[&'static str]) -> Self {
        let matches = self.0.get(..from.len()).is_some_and(|prefix| {
            prefix.iter().zip(from).all(
                |(segment, name)| matches!(segment, PathSegment::Field(field) if field == name),
            )
        });
        if matches {
            self.0.splice(0..from.len(), to.iter().map(|name| PathSegment::Field(name)));
        }
        self
    }
    fn in_field(mut self, name: &'static str) -> Self {
        self.0.insert(0, PathSegment::Field(name));
        self
    }
    fn in_list(mut self, name: &'static str, index: usize) -> Self {
        self.0.insert(0, PathSegment::Index(index));
        self.in_field(name)
    }
    pub(crate) fn square(self) -> String {
        let mut text = String::from("[");
        for (i, segment) in self.0.into_iter().enumerate() {
            if i != 0 {
                text.push('/');
            }
            match segment {
                PathSegment::Field(name) => text.push_str(name),
                PathSegment::Index(index) => text.push_str(&index.to_string()),
            }
        }
        text.push(']');
        text
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ValidationIssue {
    pub key: &'static str,
    pub field: Option<FieldPath>,
}
impl ValidationIssue {
    pub(crate) fn new(key: &'static str) -> Self {
        Self { key, field: None }
    }
    pub(crate) fn at_record(mut self) -> Self {
        self.field = Some(FieldPath(Vec::new()));
        self
    }
    pub(crate) fn has_field_prefix(&self, fields: &[&str]) -> bool {
        self.field.as_ref().is_some_and(|path| {
            path.0.get(..fields.len()).is_some_and(|prefix| {
                prefix.iter().zip(fields).all(
                    |(segment, name)| matches!(segment, PathSegment::Field(field) if field == name),
                )
            })
        })
    }
    pub(crate) fn at_field(mut self, name: &'static str) -> Self {
        self.field = Some(FieldPath::field(name));
        self
    }
    pub(crate) fn at_list_item(mut self, name: &'static str, index: usize) -> Self {
        self.field = Some(FieldPath::list_item(name, index));
        self
    }
    // Project an owning record's field prefix into its editor's field names.
    // Numeric indices and the remaining typed path are retained unchanged.
    pub(crate) fn map_field_prefix(mut self, from: &[&'static str], to: &[&'static str]) -> Self {
        self.field = self.field.map(|path| path.map_field_prefix(from, to));
        self
    }
    pub(crate) fn in_field(mut self, name: &'static str) -> Self {
        self.field = self.field.map(|path| path.in_field(name));
        self
    }
    pub(crate) fn in_list(mut self, name: &'static str, index: usize) -> Self {
        self.field = self.field.map(|path| path.in_list(name, index));
        self
    }
}
impl From<&'static str> for ValidationIssue {
    fn from(key: &'static str) -> Self {
        Self::new(key)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SaveFailure {
    pub message: Vec<u8>,
    pub field: Option<FieldPath>,
}
impl From<Vec<u8>> for SaveFailure {
    fn from(message: Vec<u8>) -> Self {
        // Legacy failures remain explicitly unlocated work, not certified fieldless refusals.
        Self { message, field: None }
    }
}
impl From<ValidationIssue> for SaveFailure {
    fn from(issue: ValidationIssue) -> Self {
        Self { message: libertas::libertas_formatted_text(issue.key, &[]), field: issue.field }
    }
}
