use std::marker::PhantomData;

use diesel::query_builder::FromClause;
use diesel::Table;
use diesel::{
    backend::Backend,
    expression::ValidGrouping,
    query_builder::{AsQuery, AstPass, QueryFragment, SelectStatement},
    Expression, QueryResult, QuerySource, SelectableExpression,
};

pub trait SubqueryProjection<QS> {
    /// The name of this alias in the query
    const RELATION_NAME: &'static str;

    type PrimaryKey: SelectableExpression<Subquery<Self, QS>> + ValidGrouping<()>;
    type DefaultSelection: Expression;

    fn primary_key() -> Self::PrimaryKey;
    fn default_selection() -> Self::DefaultSelection;
}

/// We have a query with fields selected, which need
/// to be mapped to the new namespace.
#[derive(Clone)]
pub struct Subquery<Selection: ?Sized, Query> {
    query: Query,
    _phantom: PhantomData<Selection>,
}

impl<Selection, Query, DB> QueryFragment<DB> for Subquery<Selection, Query>
where
    DB: Backend,
    Selection: SubqueryProjection<Query>,
    Query: QueryFragment<DB>,
{
    fn walk_ast<'b>(&'b self, mut pass: AstPass<'_, 'b, DB>) -> QueryResult<()> {
        pass.push_sql("(");
        self.query.walk_ast(pass.reborrow())?;
        pass.push_sql(")");
        pass.push_sql(" AS ");
        pass.push_identifier(Selection::RELATION_NAME)?;
        Ok(())
    }
}

impl<Selection, Query> QuerySource for Subquery<Selection, Query>
where
    Self: Clone,
    Selection: SubqueryProjection<Query>,
    <Selection as SubqueryProjection<Query>>::DefaultSelection: SelectableExpression<Self>,
{
    type FromClause = Self;
    type DefaultSelection = <Selection as SubqueryProjection<Query>>::DefaultSelection;

    fn from_clause(&self) -> Self::FromClause {
        self.clone()
    }

    fn default_selection(&self) -> Self::DefaultSelection {
        <Selection as SubqueryProjection<Query>>::default_selection()
    }
}

impl<Selection, Query> AsQuery for Subquery<Selection, Query>
where
    Self: QuerySource,
    <Self as QuerySource>::DefaultSelection: ValidGrouping<()>,
{
    type SqlType = <<Self as QuerySource>::DefaultSelection as Expression>::SqlType;
    type Query = SelectStatement<FromClause<Self>>;

    fn as_query(self) -> Self::Query {
        SelectStatement::simple(self)
    }
}

impl<Selection: Clone, Query: Clone> Table for Subquery<Selection, Query>
where
    Subquery<Selection, Query>: AsQuery,
    Selection: SubqueryProjection<Query>,
    <Selection as SubqueryProjection<Query>>::DefaultSelection: SelectableExpression<Self>,
    <Selection as SubqueryProjection<Query>>::DefaultSelection: ValidGrouping<()>,
    <Selection as SubqueryProjection<Query>>::PrimaryKey: SelectableExpression<Self>,
    <Selection as SubqueryProjection<Query>>::PrimaryKey: ValidGrouping<()>,
{
    type PrimaryKey = <Selection as SubqueryProjection<Query>>::PrimaryKey;
    type AllColumns = <Selection as SubqueryProjection<Query>>::DefaultSelection;

    fn primary_key(&self) -> Self::PrimaryKey {
        <Selection as SubqueryProjection<Query>>::primary_key()
    }

    fn all_columns() -> Self::AllColumns {
        <Selection as SubqueryProjection<Query>>::default_selection()
    }
}

trait AsSubquery<S2, Query> {
    fn as_subquery(self) -> Subquery<S2, Query>;
}

impl<S2, Query> AsSubquery<S2, Query> for Query {
    fn as_subquery(self) -> Subquery<S2, Query> {
        Subquery {
            query: self,
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use diesel::{debug_query, pg::Pg, ExpressionMethods, QueryDsl, sql_types::Text};

    use super::*;

    diesel::table! {
        foo (id) {
            id -> Int8,
            name -> Text,
            age -> Int8,
        }
    }

    macro_rules! subquery {
        ($name:ident, $query:ty, $(($col:ident, $type:ty)),+) => {
            mod $name {
                use diesel::backend::Backend;
                use diesel::dsl::Select;
                use diesel::expression::is_aggregate::No;
                use diesel::expression::ValidGrouping;
                use diesel::query_builder::QueryFragment;
                use diesel::sql_types::{Int8, Text};
                use diesel::{AppearsOnTable, Column, Expression, SelectableExpression};

                use super::foo;
                use crate::{Subquery, SubqueryProjection};

                #[derive(Clone)]
                pub struct table {}

                impl SubqueryProjection<$query> for table {
                    const RELATION_NAME: &'static str = stringify!($name);

                    type PrimaryKey = id;
                    type DefaultSelection = ($($col),+);

                    fn primary_key() -> Self::PrimaryKey {
                        id
                    }
                    fn default_selection() -> Self::DefaultSelection {
                        ($($col),+)
                    }
                }

                $(
                pub struct $col;

                impl Expression for $col {
                    type SqlType = $type;
                }

                impl Column for $col {
                    const NAME: &'static str = stringify!($col);

                    type Table = Subquery<table, $query>;
                }

                impl AppearsOnTable<Subquery<table, $query>> for $col {}
                impl SelectableExpression<Subquery<table, $query>> for $col {}
                impl ValidGrouping<()> for $col {
                    type IsAggregate = No;
                }

                impl<DB: Backend> QueryFragment<DB> for $col {
                    fn walk_ast<'b>(
                        &'b self,
                        mut pass: diesel::query_builder::AstPass<'_, 'b, DB>,
                    ) -> diesel::QueryResult<()> {
                        pass.push_identifier(stringify!($name))?;
                        pass.push_sql(".");
                        pass.push_identifier(<$col as Column>::NAME)
                    }
                }
                )+
            }
        };
    }

    subquery!(bar, Select<foo::table, (foo::id, foo::name)>, (id, Int8), (name, Text));

    #[test]
    fn it_works() {
        let sub_query = foo::table.select((foo::id, foo::name));
        //.filter(foo::age.eq(24));
        let sub_query: Subquery<bar::table, _> = sub_query.as_subquery();
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
}
