ALTER TABLE candidates
    ADD UNIQUE KEY uq_candidates_national_id (national_id),
    ADD UNIQUE KEY uq_candidates_scanned_barcode (scanned_barcode);
