//! Proptest strategies for queries.

use crate::query::{Clause, Predicate, Query, Value};
use proptest::{
    arbitrary::{any, Arbitrary},
    collection::{vec, VecStrategy},
    prop_oneof,
    strategy::{LazyTupleUnion, Map, Strategy, WA},
};

const STRING_REGEX: &'static str = "[ -~]*";

impl Arbitrary for Value {
    type Parameters = ();
    type Strategy = LazyTupleUnion<(
        WA<Map<<i64 as Arbitrary>::Strategy, fn(i64) -> Value>>,
        WA<Map<&'static str, fn(String) -> Value>>,
        WA<Map<&'static str, fn(String) -> Value>>,
    )>;

    fn arbitrary_with((): ()) -> Self::Strategy {
        prop_oneof![
            any::<i64>().prop_map(Value::Int),
            STRING_REGEX.prop_map(Value::String),
            STRING_REGEX.prop_map(Value::Var),
        ]
    }
}

impl Arbitrary for Predicate {
    type Parameters = ();
    type Strategy = Map<
        (&'static str, VecStrategy<<Value as Arbitrary>::Strategy>),
        fn((String, Vec<Value>)) -> Predicate,
    >;

    fn arbitrary_with((): ()) -> Self::Strategy {
        (STRING_REGEX, vec(any::<Value>(), 0..5)).prop_map(|(name, args)| Predicate { name, args })
    }
}

impl Arbitrary for Clause {
    type Parameters = ();
    type Strategy = Map<
        (
            <Predicate as Arbitrary>::Strategy,
            VecStrategy<(
                <bool as Arbitrary>::Strategy,
                <Predicate as Arbitrary>::Strategy,
            )>,
        ),
        fn((Predicate, Vec<(bool, Predicate)>)) -> Clause,
    >;

    fn arbitrary_with((): ()) -> Self::Strategy {
        (any::<Predicate>(), vec(any::<(bool, Predicate)>(), 0..8))
            .prop_map(|(head, body)| Clause { head, body })
    }
}

impl Arbitrary for Query {
    type Parameters = ();
    type Strategy = Map<
        (
            VecStrategy<<Clause as Arbitrary>::Strategy>,
            <Predicate as Arbitrary>::Strategy,
        ),
        fn((Vec<Clause>, Predicate)) -> Query,
    >;

    fn arbitrary_with((): ()) -> Self::Strategy {
        (vec(any::<Clause>(), 0..10), any::<Predicate>())
            .prop_map(|(clauses, predicate)| Query { clauses, predicate })
    }
}
