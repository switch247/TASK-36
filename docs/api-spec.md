openapi: 3.0.3
info:
  title: Eagle Exam Ops API
  version: 0.4.0
servers:
  - url: http://localhost:8000/api/v1
components:
  securitySchemes:
    BearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT
    SessionHeader:
      type: apiKey
      in: header
      name: x-session-id
  schemas:
    ApiError:
      type: object
      properties:
        code:
          type: integer
        message:
          type: string
        details:
          nullable: true
paths:
  /auth/login:
    post:
      security: []
      summary: Login and obtain jwt + session id
  /reports/dashboard:
    get:
      summary: Dashboard counters for frontend
      security:
        - BearerAuth: []
          SessionHeader: []
  /candidates:
    get:
      summary: List candidates with pagination, sorting, and filtering
      parameters:
        - in: query
          name: page
          schema: { type: integer, minimum: 1, default: 1 }
        - in: query
          name: limit
          schema: { type: integer, minimum: 1, maximum: 100, default: 20 }
        - in: query
          name: sort_by
          schema: { type: string, enum: [id, scanned_barcode, national_id, created_at] }
        - in: query
          name: sort_order
          schema: { type: string, enum: [asc, desc], default: desc }
        - in: query
          name: filter
          schema: { type: string }
    post:
      summary: Create candidate
  /rooms:
    get:
      summary: List rooms with pagination, sorting, and filtering
      parameters:
        - in: query
          name: page
          schema: { type: integer }
        - in: query
          name: limit
          schema: { type: integer }
        - in: query
          name: sort_by
          schema: { type: string, enum: [id, capacity, location, created_at] }
        - in: query
          name: sort_order
          schema: { type: string, enum: [asc, desc] }
        - in: query
          name: filter
          schema: { type: string }
  /sessions:
    get:
      summary: List sessions with pagination, sorting, and filtering
      parameters:
        - in: query
          name: page
          schema: { type: integer }
        - in: query
          name: limit
          schema: { type: integer }
        - in: query
          name: sort_by
          schema: { type: string, enum: [id, status, template_name, starts_at, created_at] }
        - in: query
          name: sort_order
          schema: { type: string, enum: [asc, desc] }
        - in: query
          name: filter
          schema: { type: string }
    post:
      summary: Create session using MM/DD/YYYY hh:mm AM/PM
  /assets:
    get:
      summary: List assets with pagination, sorting, and filtering
      parameters:
        - in: query
          name: page
          schema: { type: integer }
        - in: query
          name: limit
          schema: { type: integer }
        - in: query
          name: sort_by
          schema: { type: string, enum: [id, booklet_code, tracking_status, expires_on, created_at] }
        - in: query
          name: sort_order
          schema: { type: string, enum: [asc, desc] }
        - in: query
          name: filter
          schema: { type: string }
  /operations/seat-utilization:
    get:
      summary: Seat utilization reporting with pagination, sorting, and filtering
  /operations/near-expiry-alerts:
    get:
      summary: Near expiry alerts with pagination, sorting, and filtering
  /operations/incident-rates:
    get:
      summary: Incident rates with pagination, sorting, and filtering
  /outputs:
    post:
      summary: Generic output generation endpoint used by frontend
  /attachments:
    post:
      summary: Upload attachment metadata with extension/count/fingerprint checks (max 10 files per record, SHA-256 dedupe)
  /templates/{id}/lock:
    post:
      summary: Persist locked template version snapshot
  /exports/csv:
    post:
      summary: Export CSV with field whitelisting and sensitive ID masking
  /exports/excel:
    post:
      summary: Export Excel-like TSV with field whitelisting and sensitive ID masking
  /exports/pdf:
    post:
      summary: Export PDF placeholder with field whitelisting and sensitive ID masking
