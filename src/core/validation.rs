// Transient native validation failures. No accepted state or schema rule engine.
// Source owners name a local field; Save walkers prepend submitted keys or positions.
use alloc::{string::String, vec::Vec};

pub(crate) const MAX: i64 = 100_000_000_000;

pub(crate) fn amounts(v: &[i64]) -> bool {
    v.iter().all(|n| (0..=MAX).contains(n))
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PathSegment {
    Field(&'static str),
    Index(usize),
    StringKey(String),
    SignedKey(i64),
    UnsignedKey(u64),
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
    fn map_list_item_field_prefix(
        mut self,
        list: &[&str],
        from: &[&'static str],
        to: &[&'static str],
        selected: &[bool],
    ) -> Self {
        let is_list = self.0.get(..list.len()).is_some_and(|prefix| {
            prefix.iter().zip(list).all(
                |(segment, name)| matches!(segment, PathSegment::Field(field) if field == name),
            )
        });
        if !is_list {
            return self;
        }
        let Some(PathSegment::Index(index)) = self.0.get(list.len()) else {
            return self;
        };
        if selected.get(*index) != Some(&true) {
            return self;
        }
        let offset = list.len() + 1;
        let matches = self.0.get(offset..offset + from.len()).is_some_and(|prefix| {
            prefix.iter().zip(from).all(
                |(segment, name)| matches!(segment, PathSegment::Field(field) if field == name),
            )
        });
        if matches {
            self.0.splice(
                offset..offset + from.len(),
                to.iter().map(|name| PathSegment::Field(name)),
            );
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
    fn in_key(mut self, name: &'static str, key: impl Into<libertas::SchemaPathKey>) -> Self {
        let segment = match key.into() {
            libertas::SchemaPathKey::String(value) => PathSegment::StringKey(value),
            libertas::SchemaPathKey::Signed(value) => PathSegment::SignedKey(value),
            libertas::SchemaPathKey::Unsigned(value) => PathSegment::UnsignedKey(value),
        };
        self.0.insert(0, segment);
        self.in_field(name)
    }
    pub(crate) fn square(self) -> String {
        let mut path = libertas::SchemaPath::root();
        for segment in self.0 {
            path = match segment {
                PathSegment::Field(name) => path.field(name),
                PathSegment::Index(index) => path.index(index),
                PathSegment::StringKey(key) => path.key(key),
                PathSegment::SignedKey(key) => path.key(key),
                PathSegment::UnsignedKey(key) => path.key(key),
            };
        }
        path.into()
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ValidationIssue {
    pub key: &'static str,
    pub field: Option<FieldPath>,
}
impl ValidationIssue {
    // The field the issue names, without its enclosing path.
    pub(crate) fn leaf(&self) -> Option<&'static str> {
        self.field.as_ref()?.0.iter().rev().find_map(|segment| match segment {
            PathSegment::Field(name) => Some(*name),
            _ => None,
        })
    }
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
    /// Prepend the submitted member's declared key; fieldless diagnostics stay fieldless.
    pub(crate) fn in_key(
        mut self,
        name: &'static str,
        key: impl Into<libertas::SchemaPathKey>,
    ) -> Self {
        self.field = self.field.map(|path| path.in_key(name, key));
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
impl SaveFailure {
    pub(crate) fn map_current_field_prefix(
        mut self,
        from: &[&'static str],
        to: &[&'static str],
    ) -> Self {
        self.field = self.field.filter(|path| {
            path.0.get(..from.len()).is_some_and(|prefix| {
                prefix.iter().zip(from).all(
                    |(segment, name)| matches!(segment, PathSegment::Field(field) if field == name),
                )
            })
        });
        self.map_field_prefix(from, to)
    }
    pub(crate) fn map_submitted_items(
        mut self,
        list: &[&str],
        targets: &[Option<FieldPath>],
    ) -> Self {
        self.field = self.field.and_then(|mut path| {
            let matches = path.0.get(..list.len()).is_some_and(|prefix| {
                prefix.iter().zip(list).all(
                    |(segment, name)| matches!(segment, PathSegment::Field(field) if field == name),
                )
            });
            if matches && let Some(PathSegment::Index(index)) = path.0.get(list.len()) {
                let target = targets.get(*index).and_then(Option::as_ref)?;
                path.0.splice(0..list.len() + 1, target.0.iter().cloned());
            }
            Some(path)
        });
        self
    }
    pub(crate) fn map_field_prefix(mut self, from: &[&'static str], to: &[&'static str]) -> Self {
        self.field = self.field.map(|path| path.map_field_prefix(from, to));
        self
    }
    pub(crate) fn map_list_item_field_prefix(
        mut self,
        list: &[&str],
        from: &[&'static str],
        to: &[&'static str],
        selected: &[bool],
    ) -> Self {
        self.field =
            self.field.map(|path| path.map_list_item_field_prefix(list, from, to, selected));
        self
    }
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
