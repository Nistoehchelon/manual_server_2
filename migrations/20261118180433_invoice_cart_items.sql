CREATE TABLE invoice_cart_items (
    primary_key BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    id TEXT NOT NULL UNIQUE,
    invoice_id TEXT NOT NULL REFERENCES invoices(id) ON DELETE RESTRICT,
    name TEXT NOT NULL
);

CREATE TABLE invoice_cart_item_data (
    primary_key BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    id TEXT NOT NULL UNIQUE,
    invoice_cart_item_id TEXT NOT NULL REFERENCES invoice_cart_items(id) ON DELETE RESTRICT,
    name TEXT NOT NULL,
    value TEXT NOT NULL
);

CREATE INDEX idx_invoice_cart_items_invoice_id 
    ON invoice_cart_items(invoice_id);

CREATE INDEX idx_invoice_cart_item_data_item_id 
    ON invoice_cart_item_data(invoice_cart_item_id);