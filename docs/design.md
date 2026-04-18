# Design Notes

## Deduplication Logic
Guided merge matching is implemented with a dual threshold strategy:
1. Exact identifier match (`ID`) immediately qualifies records for merge review.
2. If IDs are not exact, names are compared using normalized Levenshtein similarity.
   - threshold: `>= 0.90`
   - DOB must also match exactly.

This avoids false merges where similar names exist but dates diverge.

## Encryption Approach
Candidate date of birth is encrypted at application layer before persistence:
- Algorithm: AES-256-GCM
- Nonce: 12-byte random nonce per encryption
- Storage format: base64(nonce + ciphertext)

Decryption reconstructs nonce/ciphertext from payload and validates authenticity via GCM tag.

## Field Masking in Exports
Sensitive identifiers such as `national_id`, `id_number`, and `candidate_id` are masked to `****<last4>` before output serialization.

## Print Lock Compliance
When print mode is `FinalPrint`, session row is locked (`locked_for_final_print = true`) and status is updated to `FinalPrinted`.
