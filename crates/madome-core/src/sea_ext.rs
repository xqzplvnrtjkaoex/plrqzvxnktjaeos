use sea_orm::{
    EntityTrait, Order, QueryOrder, Select,
    sea_query::{Func, SimpleExpr},
};

pub trait OrderByRandom {
    fn order_by_random(self) -> Self;
}

impl<E> OrderByRandom for Select<E>
where
    E: EntityTrait,
{
    fn order_by_random(mut self) -> Self {
        QueryOrder::query(&mut self)
            .order_by_expr(SimpleExpr::FunctionCall(Func::random()), Order::Desc);
        self
    }
}
