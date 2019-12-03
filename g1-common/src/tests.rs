use crate::query::{Clause, Predicate, Query, Value};
use pretty_assertions::assert_eq;
use proptest::{arbitrary::any, proptest};

proptest! {
    #[test]
    fn idempotent_parse_tostring_value(v in any::<Value>()) {
        let s = v.to_string();
        let v2 = s.parse().unwrap();
        assert_eq!(v, v2);
    }

    #[test]
    fn idempotent_parse_tostring_predicate(p in any::<Predicate>()) {
        let s = p.to_string();
        let p2 = s.parse().unwrap();
        assert_eq!(p, p2);
    }

    #[test]
    fn idempotent_parse_tostring_clause(c in any::<Clause>()) {
        let s = c.to_string();
        let c2 = s.parse().unwrap();
        assert_eq!(c, c2);
    }

    #[test]
    fn idempotent_parse_tostring_query(q in any::<Query>()) {
        let s = q.to_string();
        let q2 = s.parse().unwrap();
        assert_eq!(q, q2);
    }
}

#[test]
fn idempotent_parse_tostring_query_example() {
    let q = Query {
        clauses: vec![
            Clause {
                head: Predicate {
                    name: "edge".to_string(),
                    args: vec![Value::Str("A".to_string()), Value::Str("B".to_string())],
                },
                body: Vec::new(),
            },
            Clause {
                head: Predicate {
                    name: "edge".to_string(),
                    args: vec![Value::Str("A".to_string()), Value::Str("C".to_string())],
                },
                body: Vec::new(),
            },
            Clause {
                head: Predicate {
                    name: "edge".to_string(),
                    args: vec![Value::Str("B".to_string()), Value::Str("C".to_string())],
                },
                body: Vec::new(),
            },
            Clause {
                head: Predicate {
                    name: "path".to_string(),
                    args: vec![Value::Var("X".to_string()), Value::Var("X".to_string())],
                },
                body: Vec::new(),
            },
            Clause {
                head: Predicate {
                    name: "path".to_string(),
                    args: vec![Value::Var("X".to_string()), Value::Var("Z".to_string())],
                },
                body: vec![
                    (
                        false,
                        Predicate {
                            name: "path".to_string(),
                            args: vec![Value::Var("X".to_string()), Value::Var("Y".to_string())],
                        },
                    ),
                    (
                        false,
                        Predicate {
                            name: "edge".to_string(),
                            args: vec![Value::Var("Y".to_string()), Value::Var("Z".to_string())],
                        },
                    ),
                ],
            },
        ],
        goal: Predicate {
            name: "path".to_string(),
            args: vec![Value::Str("A".to_string()), Value::Var("X".to_string())],
        },
    };

    let s = q.to_string();
    let q2 = s.parse().unwrap();
    assert_eq!(q, q2);

    let s2 = r#"edge("A", "B").
edge("A", "C").
edge("B", "C").
path(X, X).
path(X, Z) :-
    path(X, Y),
    edge(Y, Z).
?- path("A", X)."#;
    assert_eq!(s, s2);
}
