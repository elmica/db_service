//! Read-only ODBC access to Aldelo Jet/MDB. Windows only.

#[allow(unused_imports)]
use crate::order_data::{OrderBatch, OrderHeader, OrderPayment, OrderRefund, OrderTransaction};
use std::path::Path;

#[cfg(windows)]
use odbc_api::{ConnectionOptions, Cursor, CursorRow, Environment, Nullable};

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

        let headers = read_order_headers(&conn, last_order_id, batch_size)?;
        if headers.is_empty() {
            return Ok(OrderBatch {
                headers: vec![],
                transactions: vec![],
                payments: vec![],
                refunds: vec![],
            });
        }

        let max_oid = headers.iter().filter_map(|h| h.order_id).max().unwrap_or(last_order_id);

        let transactions = read_order_transactions(&conn, last_order_id, max_oid)?;
        let payments = read_order_payments(&conn, last_order_id, max_oid)?;
        let refunds = read_order_refunds(&conn, last_order_id, max_oid)?;

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
    format!("Driver={{Microsoft Access Driver (*.mdb)}};DBQ={};READONLY=TRUE;", s)
}

#[cfg(windows)]
fn get_opt_i64(row: &mut CursorRow<'_>, col: u16) -> Result<Option<i64>, MdbError> {
    let mut field = Nullable::<i64>::null();
    row.get_data(col, &mut field).map_err(MdbError::Odbc)?;
    Ok(field.into_opt())
}

#[cfg(windows)]
fn get_opt_i32(row: &mut CursorRow<'_>, col: u16) -> Result<Option<i32>, MdbError> {
    let mut field = Nullable::<i32>::null();
    row.get_data(col, &mut field).map_err(MdbError::Odbc)?;
    Ok(field.into_opt())
}

#[cfg(windows)]
fn get_opt_f32(row: &mut CursorRow<'_>, col: u16) -> Result<Option<f32>, MdbError> {
    let mut field = Nullable::<f32>::null();
    row.get_data(col, &mut field).map_err(MdbError::Odbc)?;
    Ok(field.into_opt())
}

#[cfg(windows)]
fn get_opt_i16(row: &mut CursorRow<'_>, col: u16) -> Result<Option<i16>, MdbError> {
    let mut field = Nullable::<i16>::null();
    row.get_data(col, &mut field).map_err(MdbError::Odbc)?;
    Ok(field.into_opt())
}

#[cfg(windows)]
fn get_string(row: &mut CursorRow<'_>, col: u16) -> Result<String, MdbError> {
    let mut buf = Vec::new();
    let present = row.get_text(col, &mut buf).map_err(MdbError::Odbc)?;
    Ok(if present {
        String::from_utf8(buf).unwrap_or_default()
    } else {
        String::new()
    })
}

#[cfg(windows)]
fn read_order_headers(
    conn: &odbc_api::Connection<'_>,
    last_order_id: i64,
    batch_size: usize,
) -> Result<Vec<OrderHeader>, MdbError> {
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
        .execute(&sql, &last_order_id, None)
        .map_err(MdbError::Odbc)?
    else {
        return Ok(vec![]);
    };

    let mut out = Vec::new();
    while let Some(mut row) = cursor.next_row().map_err(MdbError::Odbc)? {
        out.push(row_to_order_header(&mut row)?);
    }
    Ok(out)
}

#[cfg(windows)]
fn row_to_order_header(row: &mut CursorRow<'_>) -> Result<OrderHeader, MdbError> {
    let order_date_time = get_string(row, 2)?;
    let order_status = get_string(row, 19)?;
    let amount_due = get_opt_f32(row, 20)?.unwrap_or(0.0);
    let sub_total = get_opt_f32(row, 21)?.unwrap_or(0.0);
    let sales_tax_rate = get_opt_f32(row, 14)?.unwrap_or(0.0);
    let discount_taxable = get_opt_i16(row, 18).map(|v| v == Some(1)).unwrap_or(false);
    let table_ready = get_opt_i16(row, 33).map(|v| v == Some(1)).unwrap_or(false);
    let guest_check_printed = get_opt_i16(row, 36).map(|v| v == Some(1)).unwrap_or(false);
    let sales_tax_amount_used = get_opt_f32(row, 28)?.unwrap_or(0.0);
    let bar_tab_pre_auth = get_opt_i16(row, 44).map(|v| v == Some(1)).unwrap_or(false);
    let row_guid = get_string(row, 46)?;
    Ok(OrderHeader {
        order_id: get_opt_i64(row, 1)?,
        order_date_time,
        employee_id: get_opt_i64(row, 3)?.unwrap_or(0),
        station_id: get_opt_i64(row, 4)?.unwrap_or(0),
        order_type: get_string(row, 5)?,
        dine_in_table_id: get_opt_i64(row, 6)?,
        customer_id: get_opt_i64(row, 7)?,
        delivery_charge: get_opt_f32(row, 8)?,
        delivery_comp: get_opt_f32(row, 9)?,
        driver_employee_id: get_opt_i64(row, 10)?,
        driver_departure_time: Some(get_string(row, 11)?).filter(|s| !s.is_empty()),
        driver_arrival_time: Some(get_string(row, 12)?).filter(|s| !s.is_empty()),
        on_hold_until_time: Some(get_string(row, 13)?).filter(|s| !s.is_empty()),
        sales_tax_rate,
        discount_id: get_opt_i64(row, 15)?,
        discount_amount: get_opt_f32(row, 16)?,
        discount_basis: Some(get_string(row, 17)?).filter(|s| !s.is_empty()),
        discount_taxable,
        order_status,
        amount_due,
        sub_total,
        gratuity_percent: get_opt_i64(row, 22)?,
        cash_gratuity: get_opt_f32(row, 23)?,
        credit_id: get_opt_i64(row, 24)?,
        credit_amount_used: get_opt_f32(row, 25)?,
        discount_amount_used: get_opt_f32(row, 26)?,
        surcharge_amount_used: get_opt_f32(row, 27)?,
        sales_tax_amount_used,
        gst_rate: get_opt_f32(row, 29)?,
        gst_amount_used: get_opt_f32(row, 30)?,
        bar_tab_name: Some(get_string(row, 31)?).filter(|s| !s.is_empty()),
        server_bank_id: get_opt_i64(row, 32)?,
        table_ready,
        guest_number: get_opt_i64(row, 34)?,
        specific_customer_name: Some(get_string(row, 35)?).filter(|s| !s.is_empty()),
        guest_check_printed,
        server_bank_type: Some(get_string(row, 37)?).filter(|s| !s.is_empty()),
        server_bank_amount: get_opt_f32(row, 38)?,
        edit_timestamp: Some(get_string(row, 39)?).filter(|s| !s.is_empty()),
        remote_site_number: get_opt_i32(row, 40)?,
        remote_orig_row_id: get_opt_i64(row, 41)?,
        factura_number: get_opt_i64(row, 42)?,
        store_number: get_opt_i64(row, 43)?,
        bar_tab_pre_auth,
        parent_order_id: get_opt_i64(row, 45)?,
        row_guid,
    })
}

#[cfg(windows)]
fn read_order_transactions(
    conn: &odbc_api::Connection<'_>,
    min_order_id: i64,
    max_order_id: i64,
) -> Result<Vec<OrderTransaction>, MdbError> {
    let sql = "SELECT OrderTransactionID, OrderID, MenuItemID, MenuItemAutoPriceText, MenuItemUnitPrice, \
               Quantity, ExtendedPrice, DiscountID, DiscountAmount, DiscountBasis, DiscountTaxable, \
               TransactionStatus, NotificationStatus, ShortNote, EditTimestamp, RowGUID \
               FROM OrderTransactions WHERE OrderID > ? AND OrderID <= ? ORDER BY OrderID";
    let Some(mut cursor) = conn
        .execute(sql, (&min_order_id, &max_order_id), None)
        .map_err(MdbError::Odbc)?
    else {
        return Ok(vec![]);
    };
    let mut out = Vec::new();
    while let Some(mut row) = cursor.next_row().map_err(MdbError::Odbc)? {
        out.push(row_to_order_transaction(&mut row)?);
    }
    Ok(out)
}

#[cfg(windows)]
fn row_to_order_transaction(row: &mut CursorRow<'_>) -> Result<OrderTransaction, MdbError> {
    let discount_taxable = get_opt_i16(row, 11).map(|v| v == Some(1)).unwrap_or(false);
    Ok(OrderTransaction {
        order_transaction_id: get_opt_i64(row, 1)?,
        order_id: get_opt_i64(row, 2)?.unwrap_or(0),
        menu_item_id: get_opt_i64(row, 3)?.unwrap_or(0),
        menu_item_auto_price_text: Some(get_string(row, 4)?).filter(|s| !s.is_empty()),
        menu_item_unit_price: get_opt_f32(row, 5)?.unwrap_or(0.0),
        quantity: get_opt_f32(row, 6)?.unwrap_or(0.0),
        extended_price: get_opt_f32(row, 7)?.unwrap_or(0.0),
        discount_id: get_opt_i64(row, 8)?,
        discount_amount: get_opt_f32(row, 9)?,
        discount_basis: Some(get_string(row, 10)?).filter(|s| !s.is_empty()),
        discount_taxable,
        transaction_status: get_string(row, 12)?,
        notification_status: get_string(row, 13)?,
        short_note: Some(get_string(row, 14)?).filter(|s| !s.is_empty()),
        edit_timestamp: Some(get_string(row, 15)?).filter(|s| !s.is_empty()),
        row_guid: get_string(row, 16)?,
    })
}

#[cfg(windows)]
fn read_order_payments(
    conn: &odbc_api::Connection<'_>,
    min_order_id: i64,
    max_order_id: i64,
) -> Result<Vec<OrderPayment>, MdbError> {
    let sql = "SELECT OrderPaymentID, PaymentDateTime, CashierID, NonCashierEmployeeID, OrderID, \
               PaymentMethod, AmountTendered, AmountPaid, EditTimestamp, EmployeeComp, RowGUID \
               FROM OrderPayments WHERE OrderID > ? AND OrderID <= ? ORDER BY OrderID";
    let Some(mut cursor) = conn
        .execute(sql, (&min_order_id, &max_order_id), None)
        .map_err(MdbError::Odbc)?
    else {
        return Ok(vec![]);
    };
    let mut out = Vec::new();
    while let Some(mut row) = cursor.next_row().map_err(MdbError::Odbc)? {
        out.push(row_to_order_payment(&mut row)?);
    }
    Ok(out)
}

#[cfg(windows)]
fn row_to_order_payment(row: &mut CursorRow<'_>) -> Result<OrderPayment, MdbError> {
    Ok(OrderPayment {
        order_payment_id: get_opt_i64(row, 1)?,
        payment_date_time: get_string(row, 2)?,
        cashier_id: get_opt_i64(row, 3)?.unwrap_or(0),
        non_cashier_employee_id: get_opt_i64(row, 4)?,
        order_id: get_opt_i64(row, 5)?.unwrap_or(0),
        payment_method: get_string(row, 6)?,
        amount_tendered: get_opt_f32(row, 7)?.unwrap_or(0.0),
        amount_paid: get_opt_f32(row, 8)?.unwrap_or(0.0),
        edit_timestamp: Some(get_string(row, 9)?).filter(|s| !s.is_empty()),
        employee_comp: get_opt_f32(row, 10)?.unwrap_or(0.0),
        row_guid: get_string(row, 11)?,
    })
}

#[cfg(windows)]
fn read_order_refunds(
    conn: &odbc_api::Connection<'_>,
    min_order_id: i64,
    max_order_id: i64,
) -> Result<Vec<OrderRefund>, MdbError> {
    let sql = "SELECT RefundDateTime, OrderID, CashierID, NonCashierEmployeeID, AmountRefunded, \
               RefundMethod, AutoID, EditTimestamp, RefundReason, RowGUID \
               FROM OrderRefunds WHERE OrderID > ? AND OrderID <= ? ORDER BY OrderID";
    let Some(mut cursor) = conn
        .execute(sql, (&min_order_id, &max_order_id), None)
        .map_err(MdbError::Odbc)?
    else {
        return Ok(vec![]);
    };
    let mut out = Vec::new();
    while let Some(mut row) = cursor.next_row().map_err(MdbError::Odbc)? {
        out.push(row_to_order_refund(&mut row)?);
    }
    Ok(out)
}

#[cfg(windows)]
fn row_to_order_refund(row: &mut CursorRow<'_>) -> Result<OrderRefund, MdbError> {
    Ok(OrderRefund {
        refund_date_time: get_string(row, 1)?,
        order_id: get_opt_i64(row, 2)?,
        cashier_id: get_opt_i64(row, 3)?.unwrap_or(0),
        non_cashier_employee_id: get_opt_i64(row, 4)?,
        amount_refunded: get_opt_f32(row, 5)?.unwrap_or(0.0),
        refund_method: get_string(row, 6)?,
        auto_id: get_opt_i64(row, 7)?,
        edit_timestamp: Some(get_string(row, 8)?).filter(|s| !s.is_empty()),
        refund_reason: get_string(row, 9)?,
        row_guid: get_string(row, 10)?,
    })
}
