#[macro_export]
macro_rules! subquery {
    ($query_alias:ident, $query:ty, $(($col_alias:ident, $col_type:ty)),+) => {
        mod $query_alias {
            use diesel::backend::Backend;
            use diesel::dsl::Select;
            use diesel::expression::is_aggregate::No;
            use diesel::expression::ValidGrouping;
            use diesel::query_builder::QueryFragment;
            use diesel::{AppearsOnTable, Expression, SelectableExpression};

            use super::foo;
            use crate::{Subquery, SubqueryProjection};

            #[derive(Clone)]
            #[allow(non_camel_case_types)]
            pub struct subquery {}

            impl SubqueryProjection<$query> for subquery {
                const RELATION_NAME: &'static str = stringify!($query_alias);

                type DefaultSelection = ($($col_alias),+);

                fn default_selection() -> Self::DefaultSelection {
                    ($($col_alias),+)
                }
            }

            $(
            #[allow(non_camel_case_types)]
            pub struct $col_alias;

            impl Expression for $col_alias {
                type SqlType = <$col_type as Expression>::SqlType;
            }

            impl AppearsOnTable<Subquery<subquery, $query>> for $col_alias {}
            impl SelectableExpression<Subquery<subquery, $query>> for $col_alias {}
            impl ValidGrouping<()> for $col_alias {
                type IsAggregate = No;
            }

            impl<DB: Backend> QueryFragment<DB> for $col_alias {
                fn walk_ast<'b>(
                    &'b self,
                    mut pass: diesel::query_builder::AstPass<'_, 'b, DB>,
                ) -> diesel::QueryResult<()> {
                    pass.push_identifier(stringify!($query_alias))?;
                    pass.push_sql(".");
                    pass.push_identifier(stringify!($col_alias))
                }
            }
            )+
        }
    };
}
