//! Hex map PDF generation for print-and-play output.
//!
//! Renders the current hex map as a flat grid with terrain coloring and hex
//! coordinates. Hex size is scaled to match the counter size so physical
//! counters fit on the printed hexes. Single-page output.

use printpdf::{
    BuiltinFont, Color, LinePoint, Mm, Op, PaintMode, PdfDocument, PdfFontHandle, PdfPage,
    PdfSaveOptions, Point, Polygon, PolygonRing, Pt, Rgb, TextItem, WindingOrder,
};

use crate::contracts::game_system::{EntityRole, EntityType};

use super::counter_sheet::CounterSize;
use super::{ExportData, ExportError, ExportFile, ExportOutput, ExportTarget};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Page margin in millimeters (0.5 inch).
const MARGIN_MM: f32 = 12.7;

/// US Letter page width in millimeters.
const LETTER_WIDTH_MM: f32 = 215.9;

/// US Letter page height in millimeters.
const LETTER_HEIGHT_MM: f32 = 279.4;

/// Default outline color for hex borders.
const HEX_BORDER_R: f32 = 0.3;
const HEX_BORDER_G: f32 = 0.3;
const HEX_BORDER_B: f32 = 0.3;

/// Light gray for empty (unassigned) hexes.
const EMPTY_HEX_R: f32 = 0.92;
const EMPTY_HEX_G: f32 = 0.92;
const EMPTY_HEX_B: f32 = 0.92;

// ---------------------------------------------------------------------------
// Exporter
// ---------------------------------------------------------------------------

/// Hex map PDF exporter for print-and-play output.
///
/// Renders the hex grid with terrain coloring and coordinates. Hex size
/// matches the counter size for physical compatibility.
#[derive(Debug)]
pub struct HexMapExporter {
    pub counter_size: CounterSize,
}

impl Default for HexMapExporter {
    fn default() -> Self {
        Self {
            counter_size: CounterSize::FiveEighths,
        }
    }
}

#[allow(clippy::unnecessary_literal_bound)]
impl ExportTarget for HexMapExporter {
    fn name(&self) -> &str {
        "Hex Map PDF"
    }

    fn extension(&self) -> &str {
        "pdf"
    }

    fn export(&self, data: &ExportData) -> Result<ExportOutput, ExportError> {
        if data.grid_config.map_radius == 0 && data.board_entities.is_empty() {
            return Err(ExportError::EmptyGameSystem);
        }

        let pdf_bytes = generate_hex_map(data, self.counter_size)?;

        Ok(ExportOutput {
            files: vec![ExportFile {
                name: "hex-map".to_string(),
                extension: "pdf".to_string(),
                data: pdf_bytes,
            }],
        })
    }
}

// ---------------------------------------------------------------------------
// Hex geometry
// ---------------------------------------------------------------------------

/// Compute hex size (center-to-vertex) from counter size.
///
/// For pointy-top: flat-to-flat = sqrt(3) * size, so size = counter / sqrt(3).
/// For flat-top: flat-to-flat = 2 * size * cos(30) = sqrt(3) * size, same formula.
/// We use the flat-to-flat distance to match the counter width.
fn hex_size_from_counter(counter_mm: f32) -> f32 {
    counter_mm / 3.0_f32.sqrt()
}

/// Compute pixel position of hex center from axial coordinates.
///
/// Pointy-top layout:
///   x = size * (sqrt(3) * q + sqrt(3)/2 * r)
///   y = size * (3/2 * r)
///
/// Flat-top layout:
///   x = size * (3/2 * q)
///   y = size * (sqrt(3)/2 * q + sqrt(3) * r)
#[allow(clippy::many_single_char_names)]
fn hex_center(q: i32, r: i32, size: f32, pointy_top: bool) -> (f32, f32) {
    let sqrt3 = 3.0_f32.sqrt();
    let qf = q as f32;
    let rf = r as f32;

    if pointy_top {
        let hx = size * (sqrt3 * qf + sqrt3 / 2.0 * rf);
        let hy = size * (1.5 * rf);
        (hx, hy)
    } else {
        let hx = size * (1.5 * qf);
        let hy = size * (sqrt3 / 2.0 * qf + sqrt3 * rf);
        (hx, hy)
    }
}

/// Generate the 6 vertices of a hex centered at (cx, cy).
///
/// Pointy-top: first vertex at 90 degrees (top).
/// Flat-top: first vertex at 0 degrees (right).
#[allow(clippy::many_single_char_names)]
fn hex_vertices(cx: f32, cy: f32, size: f32, pointy_top: bool) -> [(f32, f32); 6] {
    let start_angle: f32 = if pointy_top { 30.0 } else { 0.0 };
    let mut verts = [(0.0, 0.0); 6];
    for (idx, vert) in verts.iter_mut().enumerate() {
        let angle_deg = start_angle + 60.0 * idx as f32;
        let angle_rad = angle_deg.to_radians();
        *vert = (cx + size * angle_rad.cos(), cy + size * angle_rad.sin());
    }
    verts
}

// ---------------------------------------------------------------------------
// PDF Generation
// ---------------------------------------------------------------------------

/// Generate the hex map PDF bytes.
fn generate_hex_map(data: &ExportData, counter_size: CounterSize) -> Result<Vec<u8>, ExportError> {
    let hex_size = hex_size_from_counter(counter_size.mm());
    let map_radius = data.grid_config.map_radius;
    let pointy_top = data.grid_config.pointy_top;

    // Compute all hex positions in the grid.
    let radius = map_radius as i32;
    let all_hexes: Vec<(i32, i32)> = hexx::shapes::hexagon(hexx::Hex::ZERO, radius as u32)
        .map(|hex| (hex.x(), hex.y()))
        .collect();

    // Compute bounding box of the grid (in mm, centered at origin).
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;

    for &(q, r) in &all_hexes {
        let (cx, cy) = hex_center(q, r, hex_size, pointy_top);
        for &(vx, vy) in &hex_vertices(cx, cy, hex_size, pointy_top) {
            min_x = min_x.min(vx);
            max_x = max_x.max(vx);
            min_y = min_y.min(vy);
            max_y = max_y.max(vy);
        }
    }

    let grid_width = max_x - min_x;
    let grid_height = max_y - min_y;
    let usable_width = LETTER_WIDTH_MM - 2.0 * MARGIN_MM;
    let usable_height = LETTER_HEIGHT_MM - 2.0 * MARGIN_MM;

    // Check if the grid fits on a single page.
    if grid_width > usable_width || grid_height > usable_height {
        return Err(ExportError::GenerationFailed(format!(
            "Hex map ({grid_width:.1}mm x {grid_height:.1}mm) exceeds page area ({usable_width:.1}mm x {usable_height:.1}mm). \
             Reduce map radius or counter size."
        )));
    }

    // Offset to center the grid on the page.
    let offset_x = MARGIN_MM + (usable_width - grid_width) / 2.0 - min_x;
    let offset_y = MARGIN_MM + (usable_height - grid_height) / 2.0 - min_y;

    let font = PdfFontHandle::Builtin(BuiltinFont::Helvetica);
    let font_pt = hex_size * 0.45;

    let mut ops = Vec::new();

    // Build a lookup map for board entities.
    let board_type_map: std::collections::HashMap<(i32, i32), &EntityType> = data
        .board_entities
        .iter()
        .filter_map(|(pos, entity_data)| {
            let entity_type = data.entity_types.iter().find(|t| {
                t.id == entity_data.entity_type_id && t.role == EntityRole::BoardPosition
            })?;
            Some(((pos.q, pos.r), entity_type))
        })
        .collect();

    // Render each hex.
    #[allow(clippy::similar_names)]
    for &(q, r) in &all_hexes {
        let (cx, cy) = hex_center(q, r, hex_size, pointy_top);
        let draw_x = cx + offset_x;
        let draw_y = cy + offset_y;

        let verts = hex_vertices(draw_x, draw_y, hex_size, pointy_top);

        // Fill color: terrain color or light gray for empty.
        let fill = if let Some(entity_type) = board_type_map.get(&(q, r)) {
            let srgba = entity_type.color.to_srgba();
            (srgba.red, srgba.green, srgba.blue)
        } else {
            (EMPTY_HEX_R, EMPTY_HEX_G, EMPTY_HEX_B)
        };

        ops.push(Op::SaveGraphicsState);
        ops.push(Op::SetFillColor {
            col: Color::Rgb(Rgb::new(fill.0, fill.1, fill.2, None)),
        });
        ops.push(Op::SetOutlineColor {
            col: Color::Rgb(Rgb::new(HEX_BORDER_R, HEX_BORDER_G, HEX_BORDER_B, None)),
        });
        ops.push(Op::SetOutlineThickness { pt: Pt(0.3) });
        ops.push(Op::DrawPolygon {
            polygon: hex_polygon(&verts),
        });
        ops.push(Op::RestoreGraphicsState);

        // Coordinate label.
        let label = format!("{q},{r}");
        let label_width = estimate_text_width(&label, font_pt);
        let text_x = draw_x - label_width / 2.0;
        let text_y = draw_y - font_pt * 0.3528 / 2.0;

        // Text contrast.
        let luminance = 0.299 * fill.0 + 0.587 * fill.1 + 0.114 * fill.2;
        let text_color = if luminance > 0.5 {
            Color::Rgb(Rgb::new(0.2, 0.2, 0.2, None))
        } else {
            Color::Rgb(Rgb::new(0.9, 0.9, 0.9, None))
        };

        ops.push(Op::StartTextSection);
        ops.push(Op::SetFillColor { col: text_color });
        ops.push(Op::SetFont {
            font: font.clone(),
            size: Pt(font_pt),
        });
        ops.push(Op::SetTextCursor {
            pos: Point {
                x: Mm(text_x).into(),
                y: Mm(text_y).into(),
            },
        });
        ops.push(Op::ShowText {
            items: vec![TextItem::Text(label)],
        });
        ops.push(Op::EndTextSection);
    }

    let page = PdfPage::new(Mm(LETTER_WIDTH_MM), Mm(LETTER_HEIGHT_MM), ops);
    let mut doc = PdfDocument::new("Hexorder Hex Map");
    doc.with_pages(vec![page]);
    let mut warnings = Vec::new();
    let bytes = doc.save(&PdfSaveOptions::default(), &mut warnings);

    Ok(bytes)
}

/// Create a hexagon polygon from 6 vertices.
fn hex_polygon(verts: &[(f32, f32); 6]) -> Polygon {
    let points: Vec<LinePoint> = verts
        .iter()
        .map(|&(vx, vy)| LinePoint {
            p: Point {
                x: Mm(vx).into(),
                y: Mm(vy).into(),
            },
            bezier: false,
        })
        .collect();

    Polygon {
        rings: vec![PolygonRing { points }],
        mode: PaintMode::FillStroke,
        winding_order: WindingOrder::NonZero,
    }
}

/// Rough text width estimate using average character width for Helvetica.
fn estimate_text_width(text: &str, font_pt: f32) -> f32 {
    let char_width_mm = font_pt * 0.52 * 0.3528;
    text.len() as f32 * char_width_mm
}
