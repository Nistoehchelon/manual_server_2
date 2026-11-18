ALTER TABLE users
    ADD CONSTRAINT uq_users_id UNIQUE (id);
    
CREATE TABLE user_shopping_carts (
    primary_key BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    id TEXT NOT NULL UNIQUE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE NO ACTION
);

CREATE TABLE shopping_cart_items (
    primary_key BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    id TEXT NOT NULL UNIQUE,
    shopping_cart_id TEXT NOT NULL REFERENCES user_shopping_carts(id) ON DELETE NO ACTION,
    name TEXT NOT NULL,
    UNIQUE (shopping_cart_id, name)
);

CREATE TABLE shopping_cart_item_data (
    primary_key BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    id TEXT NOT NULL UNIQUE,
    shopping_cart_item_id TEXT NOT NULL REFERENCES shopping_cart_items(id) ON DELETE NO ACTION,
    name TEXT NOT NULL UNIQUE,
    value TEXT NOT NULL
);

ALTER TABLE invoice_cart_items
    ADD CONSTRAINT uq_invoice_cart_items_invoice_id_name UNIQUE (invoice_id, name);

ALTER TABLE invoice_cart_item_data
    ADD CONSTRAINT uq_invoice_cart_item_data_item_id_name UNIQUE (invoice_cart_item_id, name);


ALTER TABLE invoice_cart_items
    DROP CONSTRAINT invoice_cart_items_invoice_id_fkey,
    ADD CONSTRAINT invoice_cart_items_invoice_id_fkey
        FOREIGN KEY (invoice_id) REFERENCES invoices(id) ON DELETE NO ACTION;

ALTER TABLE invoice_cart_item_data
    DROP CONSTRAINT invoice_cart_item_data_invoice_cart_item_id_fkey,
    ADD CONSTRAINT invoice_cart_item_data_invoice_cart_item_id_fkey
        FOREIGN KEY (invoice_cart_item_id) REFERENCES invoice_cart_items(id) ON DELETE NO ACTION;
