# Convex setup: single table `aldeloOrders` (flattened)

Use this in a **separate Convex project** (e.g. your KDS app). The Aldelo sync service POSTs batches to an HTTP endpoint; this doc describes the schema, mutation, and HTTP setup so you can implement it from scratch.

---

## 1. Payload from the sync service

The Rust service sends a **POST** with JSON:

```json
{
  "headers": [ { "order_id": 123, "order_date_time": "...", ... } ],
  "transactions": [ { "order_transaction_id": 456, "order_id": 123, ... } ],
  "payments": [ { "order_payment_id": 789, "order_id": 123, ... } ],
  "refunds": [ { "refund_date_time": "...", "order_id": 123, ... } ]
}
```

We **flatten** this into one table: each document = one order (from `headers`) with nested `transactions`, `payments`, and `refunds` arrays.

---

## 2. Single table: `aldeloOrders`

One table, one document per order. Header fields at top level; related rows as nested arrays.

### Schema (`convex/schema.ts`)

```ts
import { defineSchema, defineTable } from "convex/server";
import { v } from "convex/values";

export default defineSchema({
  aldeloOrders: defineTable({
    // ---- From OrderHeaders (flattened) ----
    order_id: v.optional(v.number()),
    order_date_time: v.string(),
    employee_id: v.number(),
    station_id: v.number(),
    order_type: v.string(),
    dine_in_table_id: v.optional(v.number()),
    customer_id: v.optional(v.number()),
    delivery_charge: v.optional(v.number()),
    delivery_comp: v.optional(v.number()),
    driver_employee_id: v.optional(v.number()),
    driver_departure_time: v.optional(v.string()),
    driver_arrival_time: v.optional(v.string()),
    on_hold_until_time: v.optional(v.string()),
    sales_tax_rate: v.number(),
    discount_id: v.optional(v.number()),
    discount_amount: v.optional(v.number()),
    discount_basis: v.optional(v.string()),
    discount_taxable: v.boolean(),
    order_status: v.string(),
    amount_due: v.number(),
    sub_total: v.number(),
    gratuity_percent: v.optional(v.number()),
    cash_gratuity: v.optional(v.number()),
    credit_id: v.optional(v.number()),
    credit_amount_used: v.optional(v.number()),
    discount_amount_used: v.optional(v.number()),
    surcharge_amount_used: v.optional(v.number()),
    sales_tax_amount_used: v.number(),
    gst_rate: v.optional(v.number()),
    gst_amount_used: v.optional(v.number()),
    bar_tab_name: v.optional(v.string()),
    server_bank_id: v.optional(v.number()),
    table_ready: v.boolean(),
    guest_number: v.optional(v.number()),
    specific_customer_name: v.optional(v.string()),
    guest_check_printed: v.boolean(),
    server_bank_type: v.optional(v.string()),
    server_bank_amount: v.optional(v.number()),
    edit_timestamp: v.optional(v.string()),
    remote_site_number: v.optional(v.number()),
    remote_orig_row_id: v.optional(v.number()),
    factura_number: v.optional(v.number()),
    store_number: v.optional(v.number()),
    bar_tab_pre_auth: v.boolean(),
    parent_order_id: v.optional(v.number()),
    row_guid: v.string(),

    // ---- Nested: transactions, payments, refunds ----
    transactions: v.array(v.object({
      order_transaction_id: v.optional(v.number()),
      order_id: v.number(),
      menu_item_id: v.number(),
      menu_item_auto_price_text: v.optional(v.string()),
      menu_item_unit_price: v.number(),
      quantity: v.number(),
      extended_price: v.number(),
      discount_id: v.optional(v.number()),
      discount_amount: v.optional(v.number()),
      discount_basis: v.optional(v.string()),
      discount_taxable: v.boolean(),
      transaction_status: v.string(),
      notification_status: v.string(),
      short_note: v.optional(v.string()),
      edit_timestamp: v.optional(v.string()),
      row_guid: v.string(),
    })),
    payments: v.array(v.object({
      order_payment_id: v.optional(v.number()),
      payment_date_time: v.string(),
      cashier_id: v.number(),
      non_cashier_employee_id: v.optional(v.number()),
      order_id: v.number(),
      payment_method: v.string(),
      amount_tendered: v.number(),
      amount_paid: v.number(),
      edit_timestamp: v.optional(v.string()),
      employee_comp: v.number(),
      row_guid: v.string(),
    })),
    refunds: v.array(v.object({
      refund_date_time: v.string(),
      order_id: v.optional(v.number()),
      cashier_id: v.number(),
      non_cashier_employee_id: v.optional(v.number()),
      amount_refunded: v.number(),
      refund_method: v.string(),
      auto_id: v.optional(v.number()),
      edit_timestamp: v.optional(v.string()),
      refund_reason: v.string(),
      row_guid: v.string(),
    })),
  }).index("by_order_id", ["order_id"]),
});
```

If your Convex version uses a different schema syntax (e.g. no `v.` in schema), adjust to match the Convex docs.

---

## 3. Validators for the HTTP body

Define validators that match the sync service payload (snake_case). Use these in the mutation `args` and optionally in the HTTP handler.

### Validators and mutation (`convex/aldeloOrders.ts`)

```ts
import { mutation } from "./_generated/server";
import { v } from "convex/values";

// Matches Rust OrderHeader
const orderHeaderValidator = v.object({
  order_id: v.optional(v.number()),
  order_date_time: v.string(),
  employee_id: v.number(),
  station_id: v.number(),
  order_type: v.string(),
  dine_in_table_id: v.optional(v.number()),
  customer_id: v.optional(v.number()),
  delivery_charge: v.optional(v.number()),
  delivery_comp: v.optional(v.number()),
  driver_employee_id: v.optional(v.number()),
  driver_departure_time: v.optional(v.string()),
  driver_arrival_time: v.optional(v.string()),
  on_hold_until_time: v.optional(v.string()),
  sales_tax_rate: v.number(),
  discount_id: v.optional(v.number()),
  discount_amount: v.optional(v.number()),
  discount_basis: v.optional(v.string()),
  discount_taxable: v.boolean(),
  order_status: v.string(),
  amount_due: v.number(),
  sub_total: v.number(),
  gratuity_percent: v.optional(v.number()),
  cash_gratuity: v.optional(v.number()),
  credit_id: v.optional(v.number()),
  credit_amount_used: v.optional(v.number()),
  discount_amount_used: v.optional(v.number()),
  surcharge_amount_used: v.optional(v.number()),
  sales_tax_amount_used: v.number(),
  gst_rate: v.optional(v.number()),
  gst_amount_used: v.optional(v.number()),
  bar_tab_name: v.optional(v.string()),
  server_bank_id: v.optional(v.number()),
  table_ready: v.boolean(),
  guest_number: v.optional(v.number()),
  specific_customer_name: v.optional(v.string()),
  guest_check_printed: v.boolean(),
  server_bank_type: v.optional(v.string()),
  server_bank_amount: v.optional(v.number()),
  edit_timestamp: v.optional(v.string()),
  remote_site_number: v.optional(v.number()),
  remote_orig_row_id: v.optional(v.number()),
  factura_number: v.optional(v.number()),
  store_number: v.optional(v.number()),
  bar_tab_pre_auth: v.boolean(),
  parent_order_id: v.optional(v.number()),
  row_guid: v.string(),
});

const orderTransactionValidator = v.object({
  order_transaction_id: v.optional(v.number()),
  order_id: v.number(),
  menu_item_id: v.number(),
  menu_item_auto_price_text: v.optional(v.string()),
  menu_item_unit_price: v.number(),
  quantity: v.number(),
  extended_price: v.number(),
  discount_id: v.optional(v.number()),
  discount_amount: v.optional(v.number()),
  discount_basis: v.optional(v.string()),
  discount_taxable: v.boolean(),
  transaction_status: v.string(),
  notification_status: v.string(),
  short_note: v.optional(v.string()),
  edit_timestamp: v.optional(v.string()),
  row_guid: v.string(),
});

const orderPaymentValidator = v.object({
  order_payment_id: v.optional(v.number()),
  payment_date_time: v.string(),
  cashier_id: v.number(),
  non_cashier_employee_id: v.optional(v.number()),
  order_id: v.number(),
  payment_method: v.string(),
  amount_tendered: v.number(),
  amount_paid: v.number(),
  edit_timestamp: v.optional(v.string()),
  employee_comp: v.number(),
  row_guid: v.string(),
});

const orderRefundValidator = v.object({
  refund_date_time: v.string(),
  order_id: v.optional(v.number()),
  cashier_id: v.number(),
  non_cashier_employee_id: v.optional(v.number()),
  amount_refunded: v.number(),
  refund_method: v.string(),
  auto_id: v.optional(v.number()),
  edit_timestamp: v.optional(v.string()),
  refund_reason: v.string(),
  row_guid: v.string(),
});

export const ingestBatch = mutation({
  args: {
    headers: v.array(orderHeaderValidator),
    transactions: v.array(orderTransactionValidator),
    payments: v.array(orderPaymentValidator),
    refunds: v.array(orderRefundValidator),
  },
  handler: async (ctx, args) => {
    for (const h of args.headers) {
      const orderId = h.order_id;
      const transactions = args.transactions.filter((t) => t.order_id === orderId);
      const payments = args.payments.filter((p) => p.order_id === orderId);
      const refunds = args.refunds.filter((r) => r.order_id === orderId);

      const doc = {
        ...h,
        transactions,
        payments,
        refunds,
      };

      const existing =
        orderId != null
          ? await ctx.db
              .query("aldeloOrders")
              .withIndex("by_order_id", (q) => q.eq("order_id", orderId))
              .unique()
          : null;

      if (existing) {
        await ctx.db.patch(existing._id, doc);
      } else {
        await ctx.db.insert("aldeloOrders", doc);
      }
    }
  },
});
```

---

## 4. HTTP endpoint for the sync service

The sync service needs a **POST** URL. Use Convex’s HTTP actions (or your Convex version’s equivalent).

- Create an HTTP route, e.g. **POST** `/ingest_orders`.
- In the handler:
  1. Parse body: `const body = await request.json();`
  2. Call the mutation: `await ctx.runMutation(api.aldeloOrders.ingestBatch, body);`
  3. Return `200` with a body like `{ "ok": true }`.

Set the sync service’s **convex_url** to:

`https://<your-deployment>.convex.site/ingest_orders`

(Replace `<your-deployment>` with your Convex deployment name.)

Optional: check `Authorization: Bearer <CONVEX_API_KEY>` in the handler and reject if invalid.

---

## 5. Setup from scratch (other Cursor project)

1. Create or open the Convex project (e.g. your KDS app): `npx convex init` or open existing Convex app.
2. Add the schema: put the `aldeloOrders` table in `convex/schema.ts`. If your Convex schema does not support `v.` inside `defineTable`, define the table with plain field names and types per Convex docs; the mutation still receives the nested structure from the sync service.
3. Add the mutation: create `convex/aldeloOrders.ts` with the validators and `ingestBatch` mutation above.
4. Add the HTTP route that calls `ingestBatch` (e.g. in `convex/http.ts` or wherever your project defines HTTP routes).
5. Deploy: `npx convex deploy` (or use Convex dashboard).
6. Copy the HTTP URL for `/ingest_orders` into the Aldelo sync service config as `convex_url`.

---

## 6. Sync service config reminder

In the machine running the Rust sync service, set:

- **convex_url**: `https://<deployment>.convex.site/ingest_orders`
- **CONVEX_API_KEY** (optional): same value you check in the HTTP handler.

---

## Summary

- **One table**: `aldeloOrders` — one document per order.
- **Flattened**: Header fields at top level; `transactions`, `payments`, and `refunds` are arrays on each order doc.
- **Idempotent**: Upsert by `order_id` so repeated POSTs from the sync service are safe.
