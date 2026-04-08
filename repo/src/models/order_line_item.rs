use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::schema::order_line_items;

#[derive(Debug, Queryable, Identifiable, Selectable, Clone)]
#[diesel(table_name = order_line_items)]
pub struct OrderLineItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub sku: String,
    pub description: String,
    pub quantity: i32,
    pub unit_price_cents: i64,
    pub tax_cents: i64,
    pub line_total_cents: i64,
    pub original_line_item_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = order_line_items)]
pub struct NewOrderLineItem {
    pub order_id: Uuid,
    pub sku: String,
    pub description: String,
    pub quantity: i32,
    pub unit_price_cents: i64,
    pub tax_cents: i64,
    pub line_total_cents: i64,
    pub original_line_item_id: Option<Uuid>,
}

#[derive(Serialize)]
pub struct OrderLineItemResponse {
    pub id: Uuid,
    pub order_id: Uuid,
    pub sku: String,
    pub description: String,
    pub quantity: i32,
    pub unit_price_cents: i64,
    pub tax_cents: i64,
    pub line_total_cents: i64,
    pub original_line_item_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl From<OrderLineItem> for OrderLineItemResponse {
    fn from(li: OrderLineItem) -> Self {
        Self {
            id: li.id,
            order_id: li.order_id,
            sku: li.sku,
            description: li.description,
            quantity: li.quantity,
            unit_price_cents: li.unit_price_cents,
            tax_cents: li.tax_cents,
            line_total_cents: li.line_total_cents,
            original_line_item_id: li.original_line_item_id,
            created_at: li.created_at,
        }
    }
}

#[derive(Deserialize, Validate)]
pub struct CreateLineItemInput {
    #[validate(length(min = 1, max = 128))]
    pub sku: String,
    #[validate(length(min = 1, max = 512))]
    pub description: String,
    #[validate(range(min = 1))]
    pub quantity: i32,
    pub unit_price_cents: i64,
    #[serde(default)]
    pub tax_cents: i64,
}
