-- Add migration script here
CREATE TABLE invoices (
    primary_key BIGSERIAL PRIMARY KEY,
    id TEXT NOT NULL UNIQUE
);

CREATE TABLE invoice_data (
    primary_key BIGSERIAL PRIMARY KEY,

    invoice_id TEXT NOT NULL
        REFERENCES invoices(id)
        ON DELETE NO ACTION,

    name TEXT NOT NULL,
    value TEXT NOT NULL,

    CONSTRAINT invoice_data_invoice_id_name_key
        UNIQUE (invoice_id, name)
);
