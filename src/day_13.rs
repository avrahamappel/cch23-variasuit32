use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::{get, post, routes, Route, State};
use sqlx::prelude::*;

use crate::common::{Error, DB};

#[get("/13/sql")]
async fn sql(db: &State<DB>) -> Result<String, Error> {
    let res = sqlx::query("SELECT 20231213").fetch_one(&db.pool).await?;
    let int: i32 = res.get(0);
    Ok(int.to_string())
}

#[post("/13/reset")]
async fn reset_db(db: &State<DB>) -> Result<(), Error> {
    db.pool.execute(include_str!("../db/schema_13.sql")).await?;
    Ok(())
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Order {
    id: i32,
    region_id: i32,
    gift_name: String,
    quantity: i32,
}

#[post("/13/orders", data = "<orders>")]
async fn place_orders(db: &State<DB>, orders: Json<Vec<Order>>) -> Result<(), Error> {
    for order in &*orders {
        sqlx::query(
            "INSERT INTO orders (id, region_id, gift_name, quantity) VALUES ($1, $2, $3, $4)",
        )
        .bind(order.id)
        .bind(order.region_id)
        .bind(&order.gift_name)
        .bind(order.quantity)
        .execute(&db.pool)
        .await?;
    }
    Ok(())
}

#[derive(Serialize, FromRow)]
#[serde(crate = "rocket::serde")]
struct OrderTotal {
    total: i64,
}

#[get("/13/orders/total")]
async fn orders_sum(db: &State<DB>) -> Result<Json<OrderTotal>, Error> {
    let res: OrderTotal = sqlx::query_as("SELECT SUM(quantity) AS total FROM orders")
        .fetch_one(&db.pool)
        .await?;

    Ok(Json(res))
}

#[derive(Serialize, FromRow, Default)]
#[serde(crate = "rocket::serde")]
struct OrdersPopular {
    popular: Option<String>,
}

#[get("/13/orders/popular")]
async fn orders_popular(db: &State<DB>) -> Result<Json<OrdersPopular>, Error> {
    let res: OrdersPopular = sqlx::query_as(
        "SELECT gift_name AS popular FROM (SELECT gift_name, SUM(quantity) AS total FROM orders GROUP BY gift_name) AS g ORDER BY total DESC LIMIT 1"
    )
        .fetch_one(&db.pool)
        .await
        .unwrap_or_default();
    Ok(Json(res))
}

pub fn routes() -> Vec<Route> {
    routes![sql, reset_db, place_orders, orders_sum, orders_popular]
}
