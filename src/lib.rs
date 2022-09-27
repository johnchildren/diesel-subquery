mod dsl_impls;
mod joins;
mod macros;
mod subquery;

pub use subquery::{AsSubquery, Subquery, SubqueryProjection};

#[cfg(test)]
mod tests {
    use diesel::{debug_query, pg::Pg, CombineDsl, ExpressionMethods, JoinOnDsl, QueryDsl};

    use super::*;

    diesel::table! {
        foo (id) {
            id -> Int8,
            name -> Text,
            age -> Int8,
        }
    }

    subquery!(bar, Select<foo::table, (foo::id, foo::name)>, (id, foo::id), (name, foo::name));

    #[test]
    fn select() {
        let sub_query = foo::table.select((foo::id, foo::name));
        let sub_query = sub_query.as_subquery::<bar::subquery>();
        let sub_query_string = debug_query::<Pg, _>(&sub_query).to_string();

        assert_eq!(
            sub_query_string,
            "(SELECT \"foo\".\"id\", \"foo\".\"name\" FROM \"foo\") AS \"bar\" -- binds: []"
        );

        let query = sub_query.clone().select((bar::id,));
        let query_string = debug_query::<Pg, _>(&query).to_string();

        assert_eq!(query_string, "SELECT \"bar\".\"id\" FROM (SELECT \"foo\".\"id\", \"foo\".\"name\" FROM \"foo\") AS \"bar\" -- binds: []");

        let query = sub_query.filter(bar::id.eq(24));
        let query_string = debug_query::<Pg, _>(&query).to_string();

        assert_eq!(query_string, "SELECT \"bar\".\"id\", \"bar\".\"name\" FROM (SELECT \"foo\".\"id\", \"foo\".\"name\" FROM \"foo\") AS \"bar\" WHERE (\"bar\".\"id\" = $1) -- binds: [24]");
    }

    subquery!(
        bap,
        diesel::dsl::Filter<
            Select<foo::table, (foo::id, foo::name)>,
            diesel::helper_types::Eq<foo::name, &str>,
        >,
        (id, foo::id),
        (name, foo::name)
    );

    #[test]
    fn filter() {
        let sub_query = foo::table
            .select((foo::id, foo::name))
            .filter(foo::name.eq("hi"));
        let sub_query = sub_query.as_subquery::<bap::subquery>();
        let sub_query_string = debug_query::<Pg, _>(&sub_query).to_string();

        assert_eq!(
            sub_query_string,
            "(SELECT \"foo\".\"id\", \"foo\".\"name\" FROM \"foo\" WHERE (\"foo\".\"name\" = $1)) AS \"bap\" -- binds: [\"hi\"]"
        );

        let query = sub_query.clone().select((bap::id,));
        let query_string = debug_query::<Pg, _>(&query).to_string();

        assert_eq!(query_string, "SELECT \"bap\".\"id\" FROM (SELECT \"foo\".\"id\", \"foo\".\"name\" FROM \"foo\" WHERE (\"foo\".\"name\" = $1)) AS \"bap\" -- binds: [\"hi\"]");

        let query = sub_query.filter(bar::id.eq(24));
        let query_string = debug_query::<Pg, _>(&query).to_string();

        assert_eq!(query_string, "SELECT \"bap\".\"id\", \"bap\".\"name\" FROM (SELECT \"foo\".\"id\", \"foo\".\"name\" FROM \"foo\" WHERE (\"foo\".\"name\" = $1)) AS \"bap\" WHERE (\"bar\".\"id\" = $2) -- binds: [\"hi\", 24]");
    }

    subquery!(
        baz,
        diesel::helper_types::Union<
            Select<foo::table, (foo::id, foo::name)>,
            Select<foo::table, (foo::id, foo::name)>,
        >,
        (id, foo::id),
        (name, foo::name)
    );

    #[test]
    fn union() {
        let sub_query = foo::table
            .select((foo::id, foo::name))
            .union(foo::table.select((foo::id, foo::name)));
        let sub_query = sub_query.as_subquery::<baz::subquery>();
        let sub_query_string = debug_query::<Pg, _>(&sub_query).to_string();

        assert_eq!(
            sub_query_string,
            "((SELECT \"foo\".\"id\", \"foo\".\"name\" FROM \"foo\") UNION (SELECT \"foo\".\"id\", \"foo\".\"name\" FROM \"foo\")) AS \"baz\" -- binds: []"
        );

        let query = sub_query.clone().select((baz::id,));
        let query_string = debug_query::<Pg, _>(&query).to_string();

        assert_eq!(query_string, "SELECT \"baz\".\"id\" FROM ((SELECT \"foo\".\"id\", \"foo\".\"name\" FROM \"foo\") UNION (SELECT \"foo\".\"id\", \"foo\".\"name\" FROM \"foo\")) AS \"baz\" -- binds: []");
    }

    /*
    #[test]
    fn joins() {
        let sub_query = foo::table.select((foo::id, foo::name));
        let sub_query = sub_query.as_subquery::<bar::subquery>();

        let query = sub_query.left_join(foo::table.on(foo::id.eq(bar::id)));
        let query_string = debug_query::<Pg, _>(&query).to_string();

        assert_eq!(query_string, "SELECT \"bar\".\"id\" FROM (SELECT \"foo\".\"id\", \"foo\".\"name\" FROM \"foo\") AS \"bar\" -- binds: []");
    }
    */
}
