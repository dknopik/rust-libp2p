use std::collections::HashMap;

use crate::generated::wiretap::StringDef;

pub(crate) struct StringInterner {
    map: HashMap<String, u32>,
    next_id: u32,
}

impl StringInterner {
    pub(crate) fn new() -> Self {
        Self {
            map: HashMap::new(),
            next_id: 0,
        }
    }

    pub(crate) fn intern(&mut self, s: &str) -> (u32, Option<StringDef>) {
        if let Some(&id) = self.map.get(s) {
            return (id, None);
        }
        let id = self.next_id;
        self.next_id += 1;
        self.map.insert(s.to_owned(), id);
        let def = StringDef {
            id,
            value: s.to_owned(),
        };
        (id, Some(def))
    }

    pub(crate) fn snapshot(&self) -> Vec<StringDef> {
        let mut defs: Vec<_> = self
            .map
            .iter()
            .map(|(value, &id)| StringDef {
                id,
                value: value.clone(),
            })
            .collect();
        defs.sort_by_key(|d| d.id);
        defs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intern_returns_same_id() {
        let mut interner = StringInterner::new();
        let (id1, def1) = interner.intern("hello");
        let (id2, def2) = interner.intern("hello");
        assert_eq!(id1, id2);
        assert!(def1.is_some());
        assert!(def2.is_none());
    }

    #[test]
    fn intern_different_strings() {
        let mut interner = StringInterner::new();
        let (id1, _) = interner.intern("a");
        let (id2, _) = interner.intern("b");
        assert_ne!(id1, id2);
    }

    #[test]
    fn snapshot_ordered() {
        let mut interner = StringInterner::new();
        interner.intern("c");
        interner.intern("a");
        interner.intern("b");
        let snap = interner.snapshot();
        assert_eq!(snap.len(), 3);
        assert!(snap[0].id < snap[1].id && snap[1].id < snap[2].id);
    }
}
