use indexmap::IndexMap;
use std::fmt::Formatter;

pub struct TestCases<'node> {
    map: IndexMap<&'node syn::Ident, Vec<&'node syn::Expr>>,
    amount_of_test_cases: Option<usize>,
}

impl std::fmt::Debug for TestCases<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let ids = self
            .map
            .iter()
            .map(|(id, _)| format!("{}", id))
            .collect::<Vec<String>>();
        let joined = ids.join(", ");
        f.write_str(&format!("TestCases {{  identifiers = {} }}", joined))
    }
}

impl<'node> TestCases<'node> {
    pub fn empty() -> Self {
        Self {
            map: IndexMap::new(),
            amount_of_test_cases: None,
        }
    }

    pub fn insert(&mut self, id: &'node syn::Ident, exprs: Vec<&'node syn::Expr>) {
        let expressions = exprs.len();

        //
        match self.amount_of_test_cases {
            Some(amount) if amount != expressions => panic!(
                "[parameterized-macro] error: Each test-case parameter should have an equal amount of values passed to it.\n\
                    Expected {} arguments for '{}', but got: {}", amount, id, expressions,
            ),
            Some(_) => {}
            None => {
                self.amount_of_test_cases = Some(expressions)
            },
        };

        //
        if expressions != self.unwrap_amount_of_test_cases() {
            panic!(
                "[parameterized-macro] error: Each test-case parameter should have an equal amount of values passed to it.\n\
                    Expected {} arguments for '{}', but got: {}", self.unwrap_amount_of_test_cases(), id, expressions,
            );
        }

        // Only insert if the id does not yet exist
        if self.map.get(id).is_none() {
            self.map.insert(id, exprs);
        } else {
            panic!(
                "[parameterized-macro] error: found duplicate entry for '{}'",
                id
            );
        }
    }

    pub fn get(&self, id: &syn::Ident, ith: usize) -> &syn::Expr {
        if let Some(exprs) = self.map.get(id) {
            exprs[ith]
        } else {
            panic!(
                "[parameterized-macro] error: Unable to find value for parameter '{}' (case #{})",
                id, ith
            );
        }
    }

    pub fn amount_of_test_cases(&self) -> Option<usize> {
        self.amount_of_test_cases
    }

    // NB: Panics if amount of test cases is unknown, i.e. if we haven't used the first parameter
    //    to seed the amount of expressions required to be defined for each parameter.
    //    This should never happen to 'parameterized' crate users, and must be guarded against in the
    //    places where it's used.
    fn unwrap_amount_of_test_cases(&self) -> usize {
        if let Some(amount) = self.amount_of_test_cases {
            amount
        } else {
            unreachable!()
        }
    }
}
