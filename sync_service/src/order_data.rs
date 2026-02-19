//! Order DTOs for Aldelo Jet tables. All optional fields use Option; Convex receives these as JSON.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct OrderHeader {
    pub order_id: Option<i64>,
    pub order_date_time: String,
    pub employee_id: i64,
    pub station_id: i64,
    pub order_type: String,
    pub dine_in_table_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub delivery_charge: Option<f32>,
    pub delivery_comp: Option<f32>,
    pub driver_employee_id: Option<i64>,
    pub driver_departure_time: Option<String>,
    pub driver_arrival_time: Option<String>,
    pub on_hold_until_time: Option<String>,
    pub sales_tax_rate: f32,
    pub discount_id: Option<i64>,
    pub discount_amount: Option<f32>,
    pub discount_basis: Option<String>,
    pub discount_taxable: bool,
    pub order_status: String,
    pub amount_due: f32,
    pub sub_total: f32,
    pub gratuity_percent: Option<i64>,
    pub cash_gratuity: Option<f32>,
    pub credit_id: Option<i64>,
    pub credit_amount_used: Option<f32>,
    pub discount_amount_used: Option<f32>,
    pub surcharge_amount_used: Option<f32>,
    pub sales_tax_amount_used: f32,
    pub gst_rate: Option<f32>,
    pub gst_amount_used: Option<f32>,
    pub bar_tab_name: Option<String>,
    pub server_bank_id: Option<i64>,
    pub table_ready: bool,
    pub guest_number: Option<i64>,
    pub specific_customer_name: Option<String>,
    pub guest_check_printed: bool,
    pub server_bank_type: Option<String>,
    pub server_bank_amount: Option<f32>,
    pub edit_timestamp: Option<String>,
    pub remote_site_number: Option<i32>,
    pub remote_orig_row_id: Option<i64>,
    pub factura_number: Option<i64>,
    pub store_number: Option<i64>,
    pub bar_tab_pre_auth: bool,
    pub parent_order_id: Option<i64>,
    pub row_guid: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderTransaction {
    pub order_transaction_id: Option<i64>,
    pub order_id: i64,
    pub menu_item_id: i64,
    pub menu_item_auto_price_text: Option<String>,
    pub menu_item_unit_price: f32,
    pub quantity: f32,
    pub extended_price: f32,
    pub discount_id: Option<i64>,
    pub discount_amount: Option<f32>,
    pub discount_basis: Option<String>,
    pub discount_taxable: bool,
    pub transaction_status: String,
    pub notification_status: String,
    pub short_note: Option<String>,
    pub edit_timestamp: Option<String>,
    pub row_guid: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderPayment {
    pub order_payment_id: Option<i64>,
    pub payment_date_time: String,
    pub cashier_id: i64,
    pub non_cashier_employee_id: Option<i64>,
    pub order_id: i64,
    pub payment_method: String,
    pub amount_tendered: f32,
    pub amount_paid: f32,
    pub edit_timestamp: Option<String>,
    pub employee_comp: f32,
    pub row_guid: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderRefund {
    pub refund_date_time: String,
    pub order_id: Option<i64>,
    pub cashier_id: i64,
    pub non_cashier_employee_id: Option<i64>,
    pub amount_refunded: f32,
    pub refund_method: String,
    pub auto_id: Option<i64>,
    pub edit_timestamp: Option<String>,
    pub refund_reason: String,
    pub row_guid: String,
}

/// One batch of orders: headers and related transactions, payments, refunds.
#[derive(Debug, Clone, Serialize)]
pub struct OrderBatch {
    pub headers: Vec<OrderHeader>,
    pub transactions: Vec<OrderTransaction>,
    pub payments: Vec<OrderPayment>,
    pub refunds: Vec<OrderRefund>,
}

impl OrderBatch {
    pub fn max_order_id(&self) -> Option<i64> {
        self.headers
            .iter()
            .filter_map(|h| h.order_id)
            .max()
    }

    pub fn is_empty(&self) -> bool {
        self.headers.is_empty()
    }
}
