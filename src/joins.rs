use diesel::query_builder::AsQuery;
use diesel::query_dsl::InternalJoinDsl;

use crate::Subquery;

impl<Rhs, Kind, On, Selection, Query> InternalJoinDsl<Rhs, Kind, On> for Subquery<Selection, Query> 
where
    Self: AsQuery,
    <Self as AsQuery>::Query: InternalJoinDsl<Rhs, Kind, On>,
{
    type Output = <<Subquery<Selection, Query> as AsQuery>::Query as InternalJoinDsl<Rhs, Kind, On>>::Output;

    fn join(self, rhs: Rhs, kind: Kind, on: On) -> Self::Output {
        self.as_query().join(rhs, kind, on)
    }
}

/*
impl <T, Rhs, Kind, On, Selection, Query> JoinTo<T> for Subquery<Selection, Query>
where
    T: Table,
    Query: InternalJoinDsl<Rhs, Kind, On>,
{
    type FromClause = <Query as JoinTo<T>>::FromClause;
    type OnClause = <Query as JoinTo<T>>::OnClause;

    fn join_target(rhs: T) -> (Self::FromClause, Self::OnClause) {
        let (from_clause, on_clause) = <Query as InternalJoinDsl<Rhs, Kind, On>>::join(rhs);
        (from_clause, on_clause)
    }
}
*/
