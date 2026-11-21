CREATE TABLE receipts (
    primary_key BIGSERIAL PRIMARY KEY,
    id          TEXT UNIQUE NOT NULL,
    invoice_id  TEXT NOT NULL REFERENCES invoices(id) ON DELETE NO ACTION,
    html_content TEXT NOT NULL
);