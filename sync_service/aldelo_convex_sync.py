#!/usr/bin/env python3
"""
Aldelo Jet → Convex sync service.
Read-only MDB access; incremental sync with cursor file; exponential backoff on errors.
Run as Windows service via NSSM on the POS machine.
"""

import json
import os
import time
from pathlib import Path

import pyodbc
import requests


# ---------- Config ----------

def load_config(config_path=None):
    """Load config from TOML file. Uses tomllib (Python 3.11+) or tomli."""
    path = config_path or os.environ.get("ALDELO_SYNC_CONFIG", "config.toml")
    if not Path(path).exists():
        raise FileNotFoundError(f"Config not found: {path}")

    try:
        import tomllib
        with open(path, "rb") as f:
            raw = tomllib.load(f)
    except ImportError:
        try:
            import tomli as tomllib
            with open(path, "rb") as f:
                raw = tomllib.load(f)
        except ImportError:
            raise ImportError("Need tomli for Python < 3.11: pip install tomli")

    # TOML may have top-level keys or a [sync] section
    cfg = raw.get("sync", raw) if isinstance(raw, dict) else raw

    return {
        "mdb_path": cfg.get("mdb_path", "C:\\Aldelo\\Data\\ChathamSandwich.mdb"),
        "convex_url": os.environ.get("CONVEX_URL") or cfg.get("convex_url", ""),
        "cursor_path": cfg.get("cursor_path", "C:\\AldeloSync\\cursor.json"),
        "poll_interval_secs": int(cfg.get("poll_interval_secs", 5)),
        "backoff_initial_secs": int(cfg.get("backoff_initial_secs", 10)),
        "backoff_multiplier": int(cfg.get("backoff_multiplier", 2)),
        "backoff_max_secs": int(cfg.get("backoff_max_secs", 60)),
        "quick_retries": int(cfg.get("quick_retries", 2)),
        "batch_size": int(cfg.get("batch_size", 200)),
    }


# ---------- Cursor ----------

def load_cursor(path):
    """Load cursor; if missing/invalid, return default (start from beginning)."""
    p = Path(path)
    if not p.exists():
        return {"last_order_id": 0}
    try:
        data = json.loads(p.read_text(encoding="utf-8"))
        return {"last_order_id": int(data.get("last_order_id", 0))}
    except Exception:
        return {"last_order_id": 0}


def save_cursor(path, last_order_id):
    p = Path(path)
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(json.dumps({"last_order_id": last_order_id}, indent=2), encoding="utf-8")


# ---------- DB: connection string ----------

def connection_string(mdb_path):
    return (
        f"DRIVER={{Microsoft Access Driver (*.mdb)}};"
        f"DBQ={mdb_path};"
        "READONLY=TRUE;"
    )


# ---------- DB: read batch ----------

def dict_from_row(cursor, row):
    return {cursor.description[i][0]: row[i] for i in range(len(row))}


def _to_iso(val):
    if val is None:
        return None
    if hasattr(val, "isoformat"):
        return val.isoformat()
    return str(val)


def read_order_headers(conn, last_order_id, batch_size):
    cur = conn.cursor()
    cur.execute("""
        SELECT TOP (?)
            OrderID, OrderDateTime, EmployeeID, StationID, OrderType, DineInTableID,
            CustomerID, DeliveryCharge, DeliveryComp, DriverEmployeeID, DriverDepartureTime,
            DriverArrivalTime, OnHoldUntilTime, SalesTaxRate, DiscountID, DiscountAmount,
            DiscountBasis, DiscountTaxable, OrderStatus, AmountDue, SubTotal, GratuityPercent,
            CashGratuity, CreditID, CreditAmountUsed, DiscountAmountUsed, SurchargeAmountUsed,
            SalesTaxAmountUsed, GSTRate, GSTAmountUsed, BarTabName, ServerBankID, TableReady,
            GuestNumber, SpecificCustomerName, GuestCheckPrinted, ServerBankType, ServerBankAmount,
            EditTimestamp, RemoteSiteNumber, RemoteOrigRowID, FacturaNumber, StoreNumber,
            BarTabPreAuth, ParentOrderID, RowGUID
        FROM OrderHeaders
        WHERE OrderID > ?
        ORDER BY OrderID
    """, (batch_size, last_order_id))

    headers = []
    for row in cur.fetchall():
        r = dict_from_row(cur, row)
        headers.append({
            "order_id": r.get("OrderID"),
            "order_date_time": _to_iso(r.get("OrderDateTime")) or "",
            "employee_id": int(r.get("EmployeeID") or 0),
            "station_id": int(r.get("StationID") or 0),
            "order_type": str(r.get("OrderType") or ""),
            "dine_in_table_id": r.get("DineInTableID"),
            "customer_id": r.get("CustomerID"),
            "delivery_charge": float(r["DeliveryCharge"]) if r.get("DeliveryCharge") is not None else None,
            "delivery_comp": float(r["DeliveryComp"]) if r.get("DeliveryComp") is not None else None,
            "driver_employee_id": r.get("DriverEmployeeID"),
            "driver_departure_time": str(r["DriverDepartureTime"]) if r.get("DriverDepartureTime") else None,
            "driver_arrival_time": str(r["DriverArrivalTime"]) if r.get("DriverArrivalTime") else None,
            "on_hold_until_time": str(r["OnHoldUntilTime"]) if r.get("OnHoldUntilTime") else None,
            "sales_tax_rate": float(r.get("SalesTaxRate") or 0),
            "discount_id": r.get("DiscountID"),
            "discount_amount": float(r["DiscountAmount"]) if r.get("DiscountAmount") is not None else None,
            "discount_basis": str(r["DiscountBasis"]) if r.get("DiscountBasis") else None,
            "discount_taxable": bool(r.get("DiscountTaxable")),
            "order_status": str(r.get("OrderStatus") or ""),
            "amount_due": float(r.get("AmountDue") or 0),
            "sub_total": float(r.get("SubTotal") or 0),
            "gratuity_percent": r.get("GratuityPercent"),
            "cash_gratuity": float(r["CashGratuity"]) if r.get("CashGratuity") is not None else None,
            "credit_id": r.get("CreditID"),
            "credit_amount_used": float(r["CreditAmountUsed"]) if r.get("CreditAmountUsed") is not None else None,
            "discount_amount_used": float(r["DiscountAmountUsed"]) if r.get("DiscountAmountUsed") is not None else None,
            "surcharge_amount_used": float(r["SurchargeAmountUsed"]) if r.get("SurchargeAmountUsed") is not None else None,
            "sales_tax_amount_used": float(r.get("SalesTaxAmountUsed") or 0),
            "gst_rate": float(r["GSTRate"]) if r.get("GSTRate") is not None else None,
            "gst_amount_used": float(r["GSTAmountUsed"]) if r.get("GSTAmountUsed") is not None else None,
            "bar_tab_name": str(r["BarTabName"]) if r.get("BarTabName") else None,
            "server_bank_id": r.get("ServerBankID"),
            "table_ready": bool(r.get("TableReady")),
            "guest_number": r.get("GuestNumber"),
            "specific_customer_name": str(r["SpecificCustomerName"]) if r.get("SpecificCustomerName") else None,
            "guest_check_printed": bool(r.get("GuestCheckPrinted")),
            "server_bank_type": str(r["ServerBankType"]) if r.get("ServerBankType") else None,
            "server_bank_amount": float(r["ServerBankAmount"]) if r.get("ServerBankAmount") is not None else None,
            "edit_timestamp": _to_iso(r.get("EditTimestamp")),
            "remote_site_number": r.get("RemoteSiteNumber"),
            "remote_orig_row_id": r.get("RemoteOrigRowID"),
            "factura_number": r.get("FacturaNumber"),
            "store_number": r.get("StoreNumber"),
            "bar_tab_pre_auth": bool(r.get("BarTabPreAuth")),
            "parent_order_id": r.get("ParentOrderID"),
            "row_guid": str(r.get("RowGUID") or ""),
        })
    return headers


def read_order_transactions(conn, min_order_id, max_order_id):
    cur = conn.cursor()
    cur.execute("""
        SELECT OrderTransactionID, OrderID, MenuItemID, MenuItemAutoPriceText, MenuItemUnitPrice,
               Quantity, ExtendedPrice, DiscountID, DiscountAmount, DiscountBasis, DiscountTaxable,
               TransactionStatus, NotificationStatus, ShortNote, EditTimestamp, RowGUID
        FROM OrderTransactions
        WHERE OrderID > ? AND OrderID <= ?
        ORDER BY OrderID
    """, (min_order_id, max_order_id))

    out = []
    for row in cur.fetchall():
        r = dict_from_row(cur, row)
        out.append({
            "order_transaction_id": r.get("OrderTransactionID"),
            "order_id": int(r.get("OrderID") or 0),
            "menu_item_id": int(r.get("MenuItemID") or 0),
            "menu_item_auto_price_text": str(r["MenuItemAutoPriceText"]) if r.get("MenuItemAutoPriceText") else None,
            "menu_item_unit_price": float(r.get("MenuItemUnitPrice") or 0),
            "quantity": float(r.get("Quantity") or 0),
            "extended_price": float(r.get("ExtendedPrice") or 0),
            "discount_id": r.get("DiscountID"),
            "discount_amount": float(r["DiscountAmount"]) if r.get("DiscountAmount") is not None else None,
            "discount_basis": str(r["DiscountBasis"]) if r.get("DiscountBasis") else None,
            "discount_taxable": bool(r.get("DiscountTaxable")),
            "transaction_status": str(r.get("TransactionStatus") or ""),
            "notification_status": str(r.get("NotificationStatus") or ""),
            "short_note": str(r["ShortNote"]) if r.get("ShortNote") else None,
            "edit_timestamp": _to_iso(r.get("EditTimestamp")),
            "row_guid": str(r.get("RowGUID") or ""),
        })
    return out


def read_order_payments(conn, min_order_id, max_order_id):
    cur = conn.cursor()
    cur.execute("""
        SELECT OrderPaymentID, PaymentDateTime, CashierID, NonCashierEmployeeID, OrderID,
               PaymentMethod, AmountTendered, AmountPaid, EditTimestamp, EmployeeComp, RowGUID
        FROM OrderPayments
        WHERE OrderID > ? AND OrderID <= ?
        ORDER BY OrderID
    """, (min_order_id, max_order_id))

    out = []
    for row in cur.fetchall():
        r = dict_from_row(cur, row)
        out.append({
            "order_payment_id": r.get("OrderPaymentID"),
            "payment_date_time": _to_iso(r.get("PaymentDateTime")) or "",
            "cashier_id": int(r.get("CashierID") or 0),
            "non_cashier_employee_id": r.get("NonCashierEmployeeID"),
            "order_id": int(r.get("OrderID") or 0),
            "payment_method": str(r.get("PaymentMethod") or ""),
            "amount_tendered": float(r.get("AmountTendered") or 0),
            "amount_paid": float(r.get("AmountPaid") or 0),
            "edit_timestamp": _to_iso(r.get("EditTimestamp")),
            "employee_comp": float(r.get("EmployeeComp") or 0),
            "row_guid": str(r.get("RowGUID") or ""),
        })
    return out


def read_order_refunds(conn, min_order_id, max_order_id):
    cur = conn.cursor()
    cur.execute("""
        SELECT RefundDateTime, OrderID, CashierID, NonCashierEmployeeID, AmountRefunded,
               RefundMethod, AutoID, EditTimestamp, RefundReason, RowGUID
        FROM OrderRefunds
        WHERE OrderID > ? AND OrderID <= ?
        ORDER BY OrderID
    """, (min_order_id, max_order_id))

    out = []
    for row in cur.fetchall():
        r = dict_from_row(cur, row)
        out.append({
            "refund_date_time": _to_iso(r.get("RefundDateTime")) or "",
            "order_id": r.get("OrderID"),
            "cashier_id": int(r.get("CashierID") or 0),
            "non_cashier_employee_id": r.get("NonCashierEmployeeID"),
            "amount_refunded": float(r.get("AmountRefunded") or 0),
            "refund_method": str(r.get("RefundMethod") or ""),
            "auto_id": r.get("AutoID"),
            "edit_timestamp": _to_iso(r.get("EditTimestamp")),
            "refund_reason": str(r.get("RefundReason") or ""),
            "row_guid": str(r.get("RowGUID") or ""),
        })
    return out


def read_next_batch(mdb_path, last_order_id, batch_size):
    conn_str = connection_string(mdb_path)
    conn = pyodbc.connect(conn_str)

    headers = read_order_headers(conn, last_order_id, batch_size)
    if not headers:
        conn.close()
        return {"headers": [], "transactions": [], "payments": [], "refunds": []}

    max_oid = max((h["order_id"] for h in headers if h["order_id"] is not None), default=last_order_id)

    transactions = read_order_transactions(conn, last_order_id, max_oid)
    payments = read_order_payments(conn, last_order_id, max_oid)
    refunds = read_order_refunds(conn, last_order_id, max_oid)

    conn.close()
    return {
        "headers": headers,
        "transactions": transactions,
        "payments": payments,
        "refunds": refunds,
    }


# ---------- Convex ----------

def send_batch(convex_url, batch, api_key=None):
    headers = {"Content-Type": "application/json"}
    if api_key:
        headers["Authorization"] = f"Bearer {api_key}"

    r = requests.post(convex_url, json=batch, headers=headers, timeout=30)
    if not r.ok:
        raise RuntimeError(f"Convex returned {r.status_code}: {r.text}")
    return r


# ---------- Backoff ----------

class BackoffState:
    def __init__(self, config):
        self.base_interval = config["poll_interval_secs"]
        self.current_interval = self.base_interval
        self.next_backoff = config["backoff_initial_secs"]
        self.backoff_initial = config["backoff_initial_secs"]
        self.multiplier = config["backoff_multiplier"]
        self.backoff_max = config["backoff_max_secs"]

    def on_success(self):
        self.current_interval = self.base_interval
        self.next_backoff = self.backoff_initial

    def on_error(self):
        self.current_interval = min(self.next_backoff, self.backoff_max)
        self.next_backoff = min(self.next_backoff * self.multiplier, self.backoff_max)


# ---------- Main ----------

def run_one_poll(config, cursor, cursor_path):
    batch = read_next_batch(
        config["mdb_path"],
        cursor["last_order_id"],
        config["batch_size"],
    )

    if not batch["headers"]:
        return

    api_key = os.environ.get("CONVEX_API_KEY")
    send_batch(config["convex_url"], batch, api_key)

    max_id = max((h["order_id"] for h in batch["headers"] if h["order_id"] is not None), default=None)
    if max_id is not None:
        cursor["last_order_id"] = max_id
        save_cursor(cursor_path, max_id)
        print(
            f"Synced {len(batch['headers'])} headers, {len(batch['transactions'])} transactions, "
            f"{len(batch['payments'])} payments, {len(batch['refunds'])} refunds; cursor -> {max_id}"
        )


def main():
    import sys
    config_path = sys.argv[1] if len(sys.argv) > 1 else None
    config = load_config(config_path)
    cursor_path = config["cursor_path"]
    cursor = load_cursor(cursor_path)
    backoff = BackoffState(config)
    api_key = os.environ.get("CONVEX_API_KEY")

    if not config["convex_url"]:
        print("ERROR: convex_url not set in config or CONVEX_URL env")
        return 1

    print(f"Starting sync: mdb={config['mdb_path']}, cursor at {cursor_path}")
    print(f"Base poll interval: {config['poll_interval_secs']}s")

    while True:
        time.sleep(backoff.current_interval)

        quick_retries = config["quick_retries"]
        backoff_applied = False

        while True:
            try:
                run_one_poll(config, cursor, cursor_path)
                backoff.on_success()
                break
            except Exception as e:
                if quick_retries > 0:
                    quick_retries -= 1
                    print(f"Poll failed ({quick_retries} quick retries left): {e}")
                    time.sleep(1)
                else:
                    print(f"Poll failed after quick retries: {e}")
                    backoff.on_error()
                    backoff_applied = True
                    break

        if backoff_applied:
            print(f"Next poll in {backoff.current_interval}s (backoff)")

    return 0


if __name__ == "__main__":
    exit(main())
