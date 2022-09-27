use std::marker::PhantomData;

use diesel::query_builder::FromClause;
use diesel::{
    backend::Backend,
    expression::ValidGrouping,
    query_builder::{AsQuery, AstPass, QueryFragment, SelectStatement},
    Expression, QueryResult, QuerySource, SelectableExpression,
};

pub trait SubqueryProjection<QS> {
    /// The name of this alias in the query
    const RELATION_NAME: &'static str;

    type DefaultSelection: Expression;

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

pub trait AsSubquery<Query> {
    fn as_subquery<Selection>(self) -> Subquery<Selection, Query>
    where
        Selection: SubqueryProjection<Query>;
}

impl<Query> AsSubquery<Query> for Query {
    fn as_subquery<Selection>(self) -> Subquery<Selection, Query>
    where
        Selection: SubqueryProjection<Query>,
    {
        Subquery {
            query: self,
            _phantom: PhantomData,
        }
    }
}
