//! Read-only ODBC access to Aldelo Jet/MDB. Windows only.

#[allow(unused_imports)]
use crate::order_data::{OrderBatch, OrderHeader, OrderPayment, OrderRefund, OrderTransaction};
use std::path::Path;

#[cfg(windows)]
use odbc_api::{ConnectionOptions, Environment};

#[derive(Debug, thiserror::Error)]
pub enum MdbError {
    #[cfg(windows)]
    #[error("ODBC: {0}")]
    Odbc(#[from] odbc_api::Error),
    #[cfg(not(windows))]
    #[error("MDB access is only supported on Windows")]
    UnsupportedPlatform,
}

/// Open MDB, read next batch of orders after `last_order_id`, then close connection.
/// Returns empty batch if no new orders.
pub fn read_next_batch(
    mdb_path: &Path,
    last_order_id: i64,
    batch_size: usize,
) -> Result<OrderBatch, MdbError> {
    #[cfg(not(windows))]
    let _ = (mdb_path, last_order_id, batch_size);

    #[cfg(not(windows))]
    return Err(MdbError::UnsupportedPlatform);

    #[cfg(windows)]
    {
        let env = Environment::new()?;
        let conn_str = connection_string(mdb_path);
        let mut conn = env
            .connect_with_connection_string(&conn_str, ConnectionOptions::default())
            .map_err(MdbError::Odbc)?;

        let headers = read_order_headers(&mut conn, last_order_id, batch_size)?;
        if headers.is_empty() {
            return Ok(OrderBatch {
                headers: vec![],
                transactions: vec![],
                payments: vec![],
                refunds: vec![],
            });
        }

        let max_oid = headers.iter().filter_map(|h| h.order_id).max().unwrap_or(last_order_id);

        let transactions = read_order_transactions(&mut conn, last_order_id, max_oid)?;
        let payments = read_order_payments(&mut conn, last_order_id, max_oid)?;
        let refunds = read_order_refunds(&mut conn, last_order_id, max_oid)?;

        Ok(OrderBatch {
            headers,
            transactions,
            payments,
            refunds,
        })
    }
}

#[cfg(windows)]
fn connection_string(path: &Path) -> String {
    let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let s = abs.to_string_lossy().replace('\\', "\\\\");
    format!("Driver={{Microsoft Access Driver (*.mdb, *.accdb)}};DBQ={};", s)
}

#[cfg(windows)]
fn read_order_headers(
    conn: &mut odbc_api::Connection<'_>,
    last_order_id: i64,
    batch_size: usize,
) -> Result<Vec<OrderHeader>, MdbError> {
    // Jet/Access: TOP n, and ORDER ID can be null in schema so we use WHERE OrderID > ?
    let sql = format!(
        "SELECT TOP {} OrderID, OrderDateTime, EmployeeID, StationID, OrderType, DineInTableID, \
         CustomerID, DeliveryCharge, DeliveryComp, DriverEmployeeID, DriverDepartureTime, DriverArrivalTime, \
         OnHoldUntilTime, SalesTaxRate, DiscountID, DiscountAmount, DiscountBasis, DiscountTaxable, OrderStatus, \
         AmountDue, SubTotal, GratuityPercent, CashGratuity, CreditID, CreditAmountUsed, DiscountAmountUsed, \
         SurchargeAmountUsed, SalesTaxAmountUsed, GSTRate, GSTAmountUsed, BarTabName, ServerBankID, \
         TableReady, GuestNumber, SpecificCustomerName, GuestCheckPrinted, ServerBankType, ServerBankAmount, \
         EditTimestamp, RemoteSiteNumber, RemoteOrigRowID, FacturaNumber, StoreNumber, BarTabPreAuth, \
         ParentOrderID, RowGUID FROM OrderHeaders WHERE OrderID > ? ORDER BY OrderID",
        batch_size
    );
    let Some(mut cursor) = conn
        .execute(&sql, (last_order_id,))
        .map_err(MdbError::Odbc)?
    else {
        return Ok(vec![]);
    };

    let mut out = Vec::new();
    while let Some(row) = cursor.next_row().map_err(MdbError::Odbc)? {
        out.push(row_to_order_header(row)?);
    }
    Ok(out)
}

#[cfg(windows)]
fn row_to_order_header(row: odbc_api::Row<'_>) -> Result<OrderHeader, MdbError> {
    use odbc_api::Row as _;
    let order_date_time = row.get::<Option<String>>(1)?.unwrap_or_default();
    let order_status = row.get::<Option<String>>(18)?.unwrap_or_default();
    let amount_due = row.get::<Option<f32>>(19)?.unwrap_or(0.0);
    let sub_total = row.get::<Option<f32>>(20)?.unwrap_or(0.0);
    let sales_tax_rate = row.get::<Option<f32>>(13)?.unwrap_or(0.0);
    let discount_taxable = row.get::<Option<i16>>(17).map(|v| v == Some(1)).unwrap_or(false);
    let table_ready = row.get::<Option<i16>>(32).map(|v| v == Some(1)).unwrap_or(false);
    let guest_check_printed = row.get::<Option<i16>>(35).map(|v| v == Some(1)).unwrap_or(false);
    let sales_tax_amount_used = row.get::<Option<f32>>(27)?.unwrap_or(0.0);
    let bar_tab_pre_auth = row.get::<Option<i16>>(43).map(|v| v == Some(1)).unwrap_or(false);
    let row_guid = row.get::<Option<String>>(45)?.unwrap_or_default();
    Ok(OrderHeader {
        order_id: row.get(0)?,
        order_date_time,
        employee_id: row.get::<Option<i64>>(2)?.unwrap_or(0),
        station_id: row.get::<Option<i64>>(3)?.unwrap_or(0),
        order_type: row.get::<Option<String>>(4)?.unwrap_or_default(),
        dine_in_table_id: row.get(5)?,
        customer_id: row.get(6)?,
        delivery_charge: row.get(7)?,
        delivery_comp: row.get(8)?,
        driver_employee_id: row.get(9)?,
        driver_departure_time: row.get(10)?,
        driver_arrival_time: row.get(11)?,
        on_hold_until_time: row.get(12)?,
        sales_tax_rate,
        discount_id: row.get(14)?,
        discount_amount: row.get(15)?,
        discount_basis: row.get(16)?,
        discount_taxable,
        order_status,
        amount_due,
        sub_total,
        gratuity_percent: row.get(21)?,
        cash_gratuity: row.get(22)?,
        credit_id: row.get(23)?,
        credit_amount_used: row.get(24)?,
        discount_amount_used: row.get(25)?,
        surcharge_amount_used: row.get(26)?,
        sales_tax_amount_used,
        gst_rate: row.get(28)?,
        gst_amount_used: row.get(29)?,
        bar_tab_name: row.get(30)?,
        server_bank_id: row.get(31)?,
        table_ready,
        guest_number: row.get(33)?,
        specific_customer_name: row.get(34)?,
        guest_check_printed,
        server_bank_type: row.get(36)?,
        server_bank_amount: row.get(37)?,
        edit_timestamp: row.get(38)?,
        remote_site_number: row.get(39)?,
        remote_orig_row_id: row.get(40)?,
        factura_number: row.get(41)?,
        store_number: row.get(42)?,
        bar_tab_pre_auth,
        parent_order_id: row.get(44)?,
        row_guid,
    })
}

#[cfg(windows)]
fn read_order_transactions(
    conn: &mut odbc_api::Connection<'_>,
    min_order_id: i64,
    max_order_id: i64,
) -> Result<Vec<OrderTransaction>, MdbError> {
    let sql = "SELECT OrderTransactionID, OrderID, MenuItemID, MenuItemAutoPriceText, MenuItemUnitPrice, \
               Quantity, ExtendedPrice, DiscountID, DiscountAmount, DiscountBasis, DiscountTaxable, \
               TransactionStatus, NotificationStatus, ShortNote, EditTimestamp, RowGUID \
               FROM OrderTransactions WHERE OrderID > ? AND OrderID <= ? ORDER BY OrderID";
    let Some(mut cursor) = conn
        .execute(sql, (min_order_id, max_order_id))
        .map_err(MdbError::Odbc)?
    else {
        return Ok(vec![]);
    };
    let mut out = Vec::new();
    while let Some(row) = cursor.next_row().map_err(MdbError::Odbc)? {
        out.push(row_to_order_transaction(row)?);
    }
    Ok(out)
}

#[cfg(windows)]
fn row_to_order_transaction(row: odbc_api::Row<'_>) -> Result<OrderTransaction, MdbError> {
    use odbc_api::Row as _;
    let discount_taxable = row.get::<Option<i16>>(10).map(|v| v == Some(1)).unwrap_or(false);
    Ok(OrderTransaction {
        order_transaction_id: row.get(0)?,
        order_id: row.get::<Option<i64>>(1)?.unwrap_or(0),
        menu_item_id: row.get::<Option<i64>>(2)?.unwrap_or(0),
        menu_item_auto_price_text: row.get(3)?,
        menu_item_unit_price: row.get::<Option<f32>>(4)?.unwrap_or(0.0),
        quantity: row.get::<Option<f32>>(5)?.unwrap_or(0.0),
        extended_price: row.get::<Option<f32>>(6)?.unwrap_or(0.0),
        discount_id: row.get(7)?,
        discount_amount: row.get(8)?,
        discount_basis: row.get(9)?,
        discount_taxable,
        transaction_status: row.get::<Option<String>>(11)?.unwrap_or_default(),
        notification_status: row.get::<Option<String>>(12)?.unwrap_or_default(),
        short_note: row.get(13)?,
        edit_timestamp: row.get(14)?,
        row_guid: row.get::<Option<String>>(15)?.unwrap_or_default(),
    })
}

#[cfg(windows)]
fn read_order_payments(
    conn: &mut odbc_api::Connection<'_>,
    min_order_id: i64,
    max_order_id: i64,
) -> Result<Vec<OrderPayment>, MdbError> {
    let sql = "SELECT OrderPaymentID, PaymentDateTime, CashierID, NonCashierEmployeeID, OrderID, \
               PaymentMethod, AmountTendered, AmountPaid, EditTimestamp, EmployeeComp, RowGUID \
               FROM OrderPayments WHERE OrderID > ? AND OrderID <= ? ORDER BY OrderID";
    let Some(mut cursor) = conn
        .execute(sql, (min_order_id, max_order_id))
        .map_err(MdbError::Odbc)?
    else {
        return Ok(vec![]);
    };
    let mut out = Vec::new();
    while let Some(row) = cursor.next_row().map_err(MdbError::Odbc)? {
        out.push(row_to_order_payment(row)?);
    }
    Ok(out)
}

#[cfg(windows)]
fn row_to_order_payment(row: odbc_api::Row<'_>) -> Result<OrderPayment, MdbError> {
    use odbc_api::Row as _;
    Ok(OrderPayment {
        order_payment_id: row.get(0)?,
        payment_date_time: row.get::<Option<String>>(1)?.unwrap_or_default(),
        cashier_id: row.get::<Option<i64>>(2)?.unwrap_or(0),
        non_cashier_employee_id: row.get(3)?,
        order_id: row.get::<Option<i64>>(4)?.unwrap_or(0),
        payment_method: row.get::<Option<String>>(5)?.unwrap_or_default(),
        amount_tendered: row.get::<Option<f32>>(6)?.unwrap_or(0.0),
        amount_paid: row.get::<Option<f32>>(7)?.unwrap_or(0.0),
        edit_timestamp: row.get(8)?,
        employee_comp: row.get::<Option<f32>>(9)?.unwrap_or(0.0),
        row_guid: row.get::<Option<String>>(10)?.unwrap_or_default(),
    })
}

#[cfg(windows)]
fn read_order_refunds(
    conn: &mut odbc_api::Connection<'_>,
    min_order_id: i64,
    max_order_id: i64,
) -> Result<Vec<OrderRefund>, MdbError> {
    let sql = "SELECT RefundDateTime, OrderID, CashierID, NonCashierEmployeeID, AmountRefunded, \
               RefundMethod, AutoID, EditTimestamp, RefundReason, RowGUID \
               FROM OrderRefunds WHERE OrderID > ? AND OrderID <= ? ORDER BY OrderID";
    let Some(mut cursor) = conn
        .execute(sql, (min_order_id, max_order_id))
        .map_err(MdbError::Odbc)?
    else {
        return Ok(vec![]);
    };
    let mut out = Vec::new();
    while let Some(row) = cursor.next_row().map_err(MdbError::Odbc)? {
        out.push(row_to_order_refund(row)?);
    }
    Ok(out)
}

#[cfg(windows)]
fn row_to_order_refund(row: odbc_api::Row<'_>) -> Result<OrderRefund, MdbError> {
    use odbc_api::Row as _;
    Ok(OrderRefund {
        refund_date_time: row.get::<Option<String>>(0)?.unwrap_or_default(),
        order_id: row.get(1)?,
        cashier_id: row.get::<Option<i64>>(2)?.unwrap_or(0),
        non_cashier_employee_id: row.get(3)?,
        amount_refunded: row.get::<Option<f32>>(4)?.unwrap_or(0.0),
        refund_method: row.get::<Option<String>>(5)?.unwrap_or_default(),
        auto_id: row.get(6)?,
        edit_timestamp: row.get(7)?,
        refund_reason: row.get::<Option<String>>(8)?.unwrap_or_default(),
        row_guid: row.get::<Option<String>>(9)?.unwrap_or_default(),
    })
}
