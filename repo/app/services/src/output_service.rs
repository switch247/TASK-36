use anyhow::{anyhow, Result};
use app_core::types::UserRole;
use base64::Engine;
use chrono::Utc;
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use tracing::info;

#[derive(Debug, Clone, Serialize)]
pub struct PrintOutput {
    pub output_type: String,
    pub mode: String,
    pub watermark: Option<String>,
    pub content: String,
    pub template_id: String,
    pub template_version_no: i32,
}

#[derive(Clone)]
pub struct OutputService {
    pool: MySqlPool,
}

impl OutputService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn generate_print_output(
        &self,
        session_id: &str,
        output_type: &str,
        mode: &str,
        user_id: &str,
        actor_role: &UserRole,
        actor_is_admin: bool,
    ) -> Result<PrintOutput> {
        let session = if actor_is_admin {
            sqlx::query_as::<_, SessionRow>(
                "SELECT id, template_name, duration_minutes, status, starts_at, ends_at, created_by FROM exam_sessions WHERE id = ?",
            )
            .bind(session_id)
            .fetch_optional(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, SessionRow>(
                "SELECT id, template_name, duration_minutes, status, starts_at, ends_at, created_by
                 FROM exam_sessions
                 WHERE id = ?
                   AND (
                        created_by = ?
                        OR EXISTS (
                            SELECT 1 FROM exam_session_assignments esa
                            WHERE esa.session_id = exam_sessions.id AND esa.user_id = ?
                        )
                   )",
            )
            .bind(session_id)
            .bind(user_id)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?
        }
        .ok_or_else(|| anyhow!("forbidden: session not found or not accessible"))?;

        let template = sqlx::query_as::<_, TemplateVersionRow>(
            "SELECT template_id, version_no, snapshot, locked_for_final_print FROM template_versions WHERE template_id = ? ORDER BY version_no DESC LIMIT 1",
        )
        .bind(&session.template_name)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow!("template version not found"))?;

        let watermark = if mode.eq_ignore_ascii_case("TestPrint") {
            Some(format!(
                "TEST PRINT {}",
                Utc::now().format("%Y-%m-%d %H:%M:%S")
            ))
        } else {
            None
        };

        if mode.eq_ignore_ascii_case("FinalPrint") {
            let role_allowed = matches!(
                actor_role,
                UserRole::Admin | UserRole::Coordinator | UserRole::Proctor
            );
            if !role_allowed {
                return Err(anyhow!("forbidden: role cannot final-print"));
            }

            if !template.locked_for_final_print {
                sqlx::query(
                    "UPDATE template_versions SET locked_for_final_print = TRUE WHERE template_id = ? AND version_no = ?",
                )
                .bind(&template.template_id)
                .bind(template.version_no)
                .execute(&self.pool)
                .await?;
            }

            sqlx::query("UPDATE exam_sessions SET locked_for_final_print = TRUE, status = 'FinalPrinted' WHERE id = ?")
                .bind(session_id)
                .execute(&self.pool)
                .await?;
        }

        let content = self
            .build_output_payload(output_type, &session, &template, mode)
            .await?;

        let print_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO print_outputs (id, session_id, output_type, mode, watermark, payload, created_by, template_id, template_version_no) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
            .bind(print_id)
            .bind(session_id)
            .bind(output_type)
            .bind(mode)
            .bind(watermark.clone())
            .bind(&content)
            .bind(user_id)
            .bind(&template.template_id)
            .bind(template.version_no)
            .execute(&self.pool)
            .await?;

        info!(
            action = "generate_print_output",
            output_type = %output_type,
            mode = %mode,
            session_id = %session_id,
            template_id = %template.template_id,
            template_version_no = template.version_no,
            "output generated"
        );

        Ok(PrintOutput {
            output_type: output_type.to_string(),
            mode: mode.to_string(),
            watermark,
            content,
            template_id: template.template_id,
            template_version_no: template.version_no,
        })
    }

    async fn build_output_payload(
        &self,
        output_type: &str,
        session: &SessionRow,
        template: &TemplateVersionRow,
        mode: &str,
    ) -> Result<String> {
        let candidates = sqlx::query_as::<_, CandidateRow>(
            "SELECT id, national_id, scanned_barcode, metadata FROM candidates WHERE created_by = ? ORDER BY created_at DESC LIMIT 500",
        )
        .bind(&session.created_by)
        .fetch_all(&self.pool)
        .await?;
        let rooms = sqlx::query_as::<_, RoomRow>(
            "SELECT id, location, capacity FROM rooms WHERE created_by = ? ORDER BY id ASC",
        )
        .bind(&session.created_by)
        .fetch_all(&self.pool)
        .await?;
        let assets = sqlx::query_as::<_, AssetRow>(
            "SELECT id, booklet_code, tracking_status, incident_count FROM assets WHERE session_id = ? ORDER BY id ASC",
        )
        .bind(&session.id)
        .fetch_all(&self.pool)
        .await?;
        let proctors = sqlx::query_as::<_, ProctorRow>(
            "SELECT id, username FROM users WHERE role = 'Proctor' ORDER BY username ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        let session_json = json!({
            "id": session.id,
            "template_id": session.template_name,
            "duration_minutes": session.duration_minutes,
            "status": session.status,
            "starts_at": session.starts_at,
            "ends_at": session.ends_at
        });

        let base = json!({
            "output_type": output_type,
            "mode": mode,
            "generated_at": Utc::now().to_rfc3339(),
            "template": {
                "template_id": template.template_id,
                "version_no": template.version_no,
                "snapshot": template.snapshot
            },
            "session": session_json
        });

        let payload = match output_type {
            "AdmitCard" => {
                let cards: Vec<Value> = candidates
                    .iter()
                    .map(|c| {
                        json!({
                            "candidate_id": c.id,
                            "national_id_masked": Self::mask_sensitive_id(&c.national_id),
                            "scanned_barcode": c.scanned_barcode,
                            "room_id": c.metadata.get("room_id").and_then(Value::as_str).unwrap_or("unassigned"),
                        })
                    })
                    .collect();
                json!({ "base": base, "admit_cards": cards })
            }
            "SeatingChart" => {
                let rows: Vec<Value> = rooms
                    .iter()
                    .map(|r| {
                        let allocated = candidates
                            .iter()
                            .filter(|c| {
                                c.metadata.get("room_id").and_then(Value::as_str)
                                    == Some(r.id.as_str())
                            })
                            .count();
                        json!({
                            "room_id": r.id,
                            "location": r.location,
                            "capacity": r.capacity,
                            "allocated_candidates": allocated
                        })
                    })
                    .collect();
                json!({ "base": base, "seating_chart": rows })
            }
            "DoorSign" => {
                let signs: Vec<Value> = rooms
                    .iter()
                    .map(|r| {
                        json!({
                            "room_id": r.id,
                            "location": r.location,
                            "session_id": session.id,
                            "template_id": session.template_name
                        })
                    })
                    .collect();
                json!({ "base": base, "door_signs": signs })
            }
            "ProctorPacket" => {
                let roster: Vec<Value> = proctors
                    .iter()
                    .map(|p| json!({ "proctor_id": p.id, "username": p.username }))
                    .collect();
                let checklist = template
                    .snapshot
                    .get("proctor_packet")
                    .and_then(|v| v.get("checklist"))
                    .cloned()
                    .unwrap_or_else(|| {
                        json!([
                            "Verify candidate identity",
                            "Verify room readiness",
                            "Record incidents",
                            "Confirm asset return"
                        ])
                    });
                json!({ "base": base, "proctor_roster": roster, "checklist": checklist })
            }
            "SummaryReport" => {
                let incidents: i64 = assets.iter().map(|a| a.incident_count as i64).sum();
                json!({
                    "base": base,
                    "summary": {
                        "candidate_count": candidates.len(),
                        "room_count": rooms.len(),
                        "asset_count": assets.len(),
                        "incident_count": incidents
                    },
                    "assets": assets.iter().map(|a| json!({
                        "asset_id": a.id,
                        "booklet_code": a.booklet_code,
                        "tracking_status": a.tracking_status,
                        "incident_count": a.incident_count
                    })).collect::<Vec<Value>>()
                })
            }
            _ => return Err(anyhow!("unsupported output_type")),
        };

        Ok(serde_json::to_string_pretty(&payload)?)
    }

    pub fn mask_sensitive_id(value: &str) -> String {
        if value.len() <= 4 {
            return "****".to_string();
        }
        let suffix = &value[value.len() - 4..];
        format!("****{}", suffix)
    }

    pub fn export_csv_whitelisted(rows: &[serde_json::Value], fields: &[&str]) -> Result<String> {
        if fields.is_empty() {
            return Err(anyhow!("at least one field is required"));
        }

        let mut out = String::new();
        out.push_str(&fields.join(","));
        out.push('\n');

        for row in rows {
            let mut cols = Vec::with_capacity(fields.len());
            for field in fields {
                let val = row
                    .get(*field)
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "\"\"".to_string());
                cols.push(val.replace(',', " "));
            }
            out.push_str(&cols.join(","));
            out.push('\n');
        }

        Ok(out)
    }

    pub fn export_excel_like_tsv(rows: &[serde_json::Value], fields: &[&str]) -> Result<String> {
        if fields.is_empty() {
            return Err(anyhow!("at least one field is required"));
        }
        fn esc_xml(value: &str) -> String {
            value
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('"', "&quot;")
                .replace('\'', "&apos;")
        }

        let mut xml = String::from(
            r#"<?xml version="1.0"?>
<Workbook xmlns="urn:schemas-microsoft-com:office:spreadsheet"
 xmlns:o="urn:schemas-microsoft-com:office:office"
 xmlns:x="urn:schemas-microsoft-com:office:excel"
 xmlns:ss="urn:schemas-microsoft-com:office:spreadsheet">
<Worksheet ss:Name="Export">
<Table>"#,
        );

        xml.push_str("<Row>");
        for field in fields {
            xml.push_str(&format!(
                "<Cell><Data ss:Type=\"String\">{}</Data></Cell>",
                esc_xml(field)
            ));
        }
        xml.push_str("</Row>");

        for row in rows {
            xml.push_str("<Row>");
            for field in fields {
                let raw = row
                    .get(*field)
                    .map(|v| {
                        if let Some(s) = v.as_str() {
                            s.to_string()
                        } else {
                            v.to_string()
                        }
                    })
                    .unwrap_or_default();
                xml.push_str(&format!(
                    "<Cell><Data ss:Type=\"String\">{}</Data></Cell>",
                    esc_xml(&raw)
                ));
            }
            xml.push_str("</Row>");
        }
        xml.push_str("</Table></Worksheet></Workbook>");

        Ok(format!(
            "data:application/vnd.ms-excel;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(xml.as_bytes())
        ))
    }

    pub fn export_pdf_placeholder(document_title: &str, body: &str) -> String {
        // Minimal real PDF generation (single page, Helvetica).
        fn esc(s: &str) -> String {
            s.replace('\\', "\\\\")
                .replace('(', "\\(")
                .replace(')', "\\)")
        }

        let text = format!(
            "{}\n{}\nGenerated: {}",
            document_title,
            body,
            Utc::now().to_rfc3339()
        );
        let stream = format!(
            "BT /F1 12 Tf 50 770 Td ({}) Tj ET",
            esc(&text.replace('\n', " | "))
        );

        let obj1 = "1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n".to_string();
        let obj2 = "2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n".to_string();
        let obj3 = "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>\nendobj\n".to_string();
        let obj4 =
            "4 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n".to_string();
        let obj5 = format!(
            "5 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n",
            stream.len(),
            stream
        );

        let mut pdf = Vec::<u8>::new();
        pdf.extend_from_slice(b"%PDF-1.4\n");
        let mut offsets = vec![0usize];
        for obj in [&obj1, &obj2, &obj3, &obj4, &obj5] {
            offsets.push(pdf.len());
            pdf.extend_from_slice(obj.as_bytes());
        }
        let xref_start = pdf.len();
        pdf.extend_from_slice(format!("xref\n0 {}\n", offsets.len()).as_bytes());
        pdf.extend_from_slice(b"0000000000 65535 f \n");
        for off in offsets.iter().skip(1) {
            pdf.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
        }
        pdf.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
                offsets.len(),
                xref_start
            )
            .as_bytes(),
        );

        format!(
            "data:application/pdf;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(pdf)
        )
    }
}

#[derive(sqlx::FromRow)]
struct SessionRow {
    id: String,
    template_name: String,
    duration_minutes: i32,
    status: String,
    starts_at: chrono::NaiveDateTime,
    ends_at: chrono::NaiveDateTime,
    created_by: String,
}

#[derive(sqlx::FromRow)]
struct TemplateVersionRow {
    template_id: String,
    version_no: i32,
    snapshot: Value,
    locked_for_final_print: bool,
}

#[derive(sqlx::FromRow)]
struct CandidateRow {
    id: String,
    national_id: String,
    scanned_barcode: String,
    metadata: Value,
}

#[derive(sqlx::FromRow)]
struct RoomRow {
    id: String,
    location: String,
    capacity: i32,
}

#[derive(sqlx::FromRow)]
struct AssetRow {
    id: String,
    booklet_code: String,
    tracking_status: String,
    incident_count: i32,
}

#[derive(sqlx::FromRow)]
struct ProctorRow {
    id: String,
    username: String,
}
