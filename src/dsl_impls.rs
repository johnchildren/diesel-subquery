use diesel::query_dsl::methods::FilterDsl;
use diesel::query_dsl::select_dsl::SelectDsl;
use diesel::QueryDsl;
use diesel::{query_builder::AsQuery, Expression};

use crate::Subquery;

impl<Selection, Query> QueryDsl for Subquery<Selection, Query> {}

impl<Selection, Query, Predicate> FilterDsl<Predicate> for Subquery<Selection, Query>
where
    Self: AsQuery,
    <Self as AsQuery>::Query: FilterDsl<Predicate>,
{
    type Output = diesel::dsl::Filter<<Self as AsQuery>::Query, Predicate>;

    fn filter(self, predicate: Predicate) -> Self::Output {
        self.as_query().filter(predicate)
    }
}

impl<Selection, SubquerySource, Query> SelectDsl<Selection> for Subquery<SubquerySource, Query>
where
    Selection: Expression,
    Self: AsQuery,
    <Self as AsQuery>::Query: SelectDsl<Selection>,
{
    type Output = diesel::dsl::Select<<Self as AsQuery>::Query, Selection>;

    fn select(self, selection: Selection) -> Self::Output {
        self.as_query().select(selection)
    }
}
