ALTER TABLE shopping_cart_items
    DROP CONSTRAINT shopping_cart_items_shopping_cart_id_fkey,
    ADD CONSTRAINT shopping_cart_items_shopping_cart_id_fkey
        FOREIGN KEY (shopping_cart_id) REFERENCES user_shopping_carts(id) ON DELETE CASCADE;

ALTER TABLE shopping_cart_item_data
    DROP CONSTRAINT shopping_cart_item_data_shopping_cart_item_id_fkey,
    ADD CONSTRAINT shopping_cart_item_data_shopping_cart_item_id_fkey
        FOREIGN KEY (shopping_cart_item_id) REFERENCES shopping_cart_items(id) ON DELETE CASCADE;
