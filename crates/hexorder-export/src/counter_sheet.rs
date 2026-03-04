//! Counter sheet PDF generation for print-and-play output.
//!
//! Generates a grid of unit counters on letter-size paper. Each counter shows
//! the entity type name and up to three numeric property values. Counters are
//! grouped by entity type and colored with the type's designer-assigned color.

use printpdf::{
    BuiltinFont, Color, Mm, Op, PaintMode, PdfDocument, PdfFontHandle, PdfPage, PdfSaveOptions,
    Point, Pt, Rect, Rgb, TextItem, WindingOrder,
};

use hexorder_contracts::game_system::{EntityRole, EntityType, PropertyValue, TypeId};

use super::{ExportData, ExportError, ExportFile, ExportOutput, ExportTarget};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Counter size options for print-and-play counter sheets.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CounterSize {
    /// 1/2 inch (12.7 mm) — smallest standard wargame counter.
    Half,
    /// 5/8 inch (15.875 mm) — most common wargame counter size.
    FiveEighths,
    /// 3/4 inch (19.05 mm) — large counter for readability.
    ThreeQuarters,
}

impl CounterSize {
    /// Counter side length in millimeters.
    pub fn mm(self) -> f32 {
        match self {
            Self::Half => 12.7,
            Self::FiveEighths => 15.875,
            Self::ThreeQuarters => 19.05,
        }
    }
}

/// Page margin in millimeters (0.5 inch).
const MARGIN_MM: f32 = 12.7;

/// Gap between counters in millimeters (cutting guide).
const GAP_MM: f32 = 0.5;

/// US Letter page width in millimeters.
const LETTER_WIDTH_MM: f32 = 215.9;

/// US Letter page height in millimeters.
const LETTER_HEIGHT_MM: f32 = 279.4;

/// Points per millimeter (1 pt = 1/72 inch, 1 inch = 25.4 mm).
const PT_PER_MM: f32 = 72.0 / 25.4;

// ---------------------------------------------------------------------------
// Exporter
// ---------------------------------------------------------------------------

/// Print-and-play PDF exporter producing counter sheets.
///
/// Counters are square, colored with the entity type color, and show the
/// type name plus up to three numeric property values.
#[derive(Debug)]
pub struct PrintAndPlayExporter {
    pub counter_size: CounterSize,
}

impl Default for PrintAndPlayExporter {
    fn default() -> Self {
        Self {
            counter_size: CounterSize::FiveEighths,
        }
    }
}

#[allow(clippy::unnecessary_literal_bound)]
impl ExportTarget for PrintAndPlayExporter {
    fn name(&self) -> &str {
        "Print-and-Play PDF"
    }

    fn extension(&self) -> &str {
        "pdf"
    }

    fn export(&self, data: &ExportData) -> Result<ExportOutput, ExportError> {
        let token_types: Vec<&EntityType> = data
            .entity_types
            .iter()
            .filter(|t| t.role == EntityRole::Token)
            .collect();

        if token_types.is_empty() && data.token_entities.is_empty() {
            return Err(ExportError::EmptyGameSystem);
        }

        let pdf_bytes = generate_counter_sheet(data, &token_types, self.counter_size)?;

        Ok(ExportOutput {
            files: vec![ExportFile {
                name: "counter-sheet".to_string(),
                extension: "pdf".to_string(),
                data: pdf_bytes,
            }],
        })
    }
}

// ---------------------------------------------------------------------------
// PDF Generation
// ---------------------------------------------------------------------------

/// A single counter to be rendered on the sheet.
struct CounterInfo {
    /// Entity type name.
    name: String,
    /// Background color (r, g, b) in 0.0..1.0 range.
    color: (f32, f32, f32),
    /// Up to three numeric property values as (name, value) pairs.
    values: Vec<(String, String)>,
}

/// Generate the counter sheet PDF bytes.
fn generate_counter_sheet(
    data: &ExportData,
    token_types: &[&EntityType],
    counter_size: CounterSize,
) -> Result<Vec<u8>, ExportError> {
    let counters = collect_counters(data, token_types);
    if counters.is_empty() {
        return Err(ExportError::EmptyGameSystem);
    }

    let size_mm = counter_size.mm();
    let usable_width = LETTER_WIDTH_MM - 2.0 * MARGIN_MM;
    let usable_height = LETTER_HEIGHT_MM - 2.0 * MARGIN_MM;

    let cols = ((usable_width + GAP_MM) / (size_mm + GAP_MM)).floor() as usize;
    let rows = ((usable_height + GAP_MM) / (size_mm + GAP_MM)).floor() as usize;
    let per_page = cols * rows;

    if per_page == 0 {
        return Err(ExportError::GenerationFailed(
            "Counter size too large for page".to_string(),
        ));
    }

    let mut doc = PdfDocument::new("Hexorder Counter Sheet");
    let mut pages = Vec::new();

    for chunk in counters.chunks(per_page) {
        let ops = render_page(chunk, cols, size_mm);
        let page = PdfPage::new(Mm(LETTER_WIDTH_MM), Mm(LETTER_HEIGHT_MM), ops);
        pages.push(page);
    }

    doc.with_pages(pages);
    let mut warnings = Vec::new();
    let bytes = doc.save(&PdfSaveOptions::default(), &mut warnings);

    Ok(bytes)
}

/// Collect counter data from the export snapshot.
///
/// One counter per token entity instance placed on the board. If no instances
/// exist, falls back to one counter per token entity type definition.
fn collect_counters(data: &ExportData, token_types: &[&EntityType]) -> Vec<CounterInfo> {
    if data.token_entities.is_empty() {
        // No placed tokens — generate one counter per token type definition.
        return token_types.iter().map(|t| type_to_counter(t)).collect();
    }

    // One counter per placed token instance.
    data.token_entities
        .iter()
        .filter_map(|(_pos, entity_data)| {
            let entity_type = data
                .entity_types
                .iter()
                .find(|t| t.id == entity_data.entity_type_id)?;

            let values = extract_property_values(entity_type, &entity_data.properties);
            Some(CounterInfo {
                name: entity_type.name.clone(),
                color: bevy_color_to_rgb(entity_type.color),
                values,
            })
        })
        .collect()
}

/// Convert a token entity type definition to a counter using default property values.
fn type_to_counter(entity_type: &EntityType) -> CounterInfo {
    let values: Vec<(String, String)> = entity_type
        .properties
        .iter()
        .filter_map(|prop| {
            let display = format_property_value(&prop.default_value)?;
            Some((prop.name.clone(), display))
        })
        .take(3)
        .collect();

    CounterInfo {
        name: entity_type.name.clone(),
        color: bevy_color_to_rgb(entity_type.color),
        values,
    }
}

/// Extract up to 3 displayable property values from instance data.
fn extract_property_values(
    entity_type: &EntityType,
    properties: &std::collections::HashMap<TypeId, PropertyValue>,
) -> Vec<(String, String)> {
    entity_type
        .properties
        .iter()
        .filter_map(|prop_def| {
            let value = properties
                .get(&prop_def.id)
                .unwrap_or(&prop_def.default_value);
            let display = format_property_value(value)?;
            Some((prop_def.name.clone(), display))
        })
        .take(3)
        .collect()
}

/// Format a property value for display on a counter.
/// Returns `None` for non-displayable types (lists, maps, structs, etc.).
pub fn format_property_value(value: &PropertyValue) -> Option<String> {
    match value {
        PropertyValue::Int(n) | PropertyValue::IntRange(n) => Some(n.to_string()),
        PropertyValue::Float(n) | PropertyValue::FloatRange(n) => Some(format!("{n:.1}")),
        PropertyValue::String(s) | PropertyValue::Enum(s) if !s.is_empty() => Some(s.clone()),
        PropertyValue::Bool(b) => Some(if *b { "Y" } else { "N" }.to_string()),
        _ => None,
    }
}

/// Convert a Bevy Color to (r, g, b) floats in 0.0..1.0.
fn bevy_color_to_rgb(color: bevy::color::Color) -> (f32, f32, f32) {
    let srgba = color.to_srgba();
    (srgba.red, srgba.green, srgba.blue)
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

/// Convert millimeters to points for the PDF coordinate system.
fn mm_to_pt(mm: f32) -> Pt {
    Pt(mm * PT_PER_MM)
}

/// Render a page of counters as PDF operations.
#[allow(clippy::many_single_char_names)]
fn render_page(counters: &[CounterInfo], cols: usize, size_mm: f32) -> Vec<Op> {
    let mut ops = Vec::new();

    // Font size scales with counter size.
    let name_font_pt = size_mm * 0.28;
    let value_font_pt = size_mm * 0.35;

    let name_font = PdfFontHandle::Builtin(BuiltinFont::Helvetica);
    let value_font = PdfFontHandle::Builtin(BuiltinFont::HelveticaBold);

    for (i, counter) in counters.iter().enumerate() {
        let col = i % cols;
        let row = i / cols;

        // Counter position (bottom-left corner). PDF origin is bottom-left.
        let x = MARGIN_MM + (col as f32) * (size_mm + GAP_MM);
        let y = LETTER_HEIGHT_MM - MARGIN_MM - ((row + 1) as f32) * (size_mm + GAP_MM) + GAP_MM;

        // Background fill rectangle.
        let (r, g, b) = counter.color;
        ops.push(Op::SaveGraphicsState);
        ops.push(Op::SetFillColor {
            col: Color::Rgb(Rgb::new(r, g, b, None)),
        });
        ops.push(Op::SetOutlineColor {
            col: Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)),
        });
        ops.push(Op::SetOutlineThickness { pt: Pt(0.5) });
        ops.push(Op::DrawRectangle {
            rectangle: Rect {
                x: mm_to_pt(x),
                y: mm_to_pt(y),
                width: mm_to_pt(size_mm),
                height: mm_to_pt(size_mm),
                mode: Some(PaintMode::FillStroke),
                winding_order: Some(WindingOrder::NonZero),
            },
        });

        // Determine text brightness for contrast.
        let luminance = 0.299 * r + 0.587 * g + 0.114 * b;
        let text_color = if luminance > 0.5 {
            Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None))
        } else {
            Color::Rgb(Rgb::new(1.0, 1.0, 1.0, None))
        };

        // Unit name (top center).
        let name_display = truncate_name(&counter.name, size_mm);
        let name_x = x + size_mm * 0.5 - estimate_text_width(&name_display, name_font_pt) * 0.5;
        let name_y = y + size_mm * 0.7;

        ops.push(Op::StartTextSection);
        ops.push(Op::SetFillColor {
            col: text_color.clone(),
        });
        ops.push(Op::SetFont {
            font: name_font.clone(),
            size: Pt(name_font_pt),
        });
        ops.push(Op::SetTextCursor {
            pos: Point {
                x: Mm(name_x).into(),
                y: Mm(name_y).into(),
            },
        });
        ops.push(Op::ShowText {
            items: vec![TextItem::Text(name_display)],
        });
        ops.push(Op::EndTextSection);

        // Property values along the bottom of the counter.
        render_property_values(
            &mut ops,
            counter,
            x,
            y,
            size_mm,
            value_font_pt,
            &text_color,
            &value_font,
        );

        ops.push(Op::RestoreGraphicsState);
    }

    ops
}

/// Render up to 3 property values at the bottom of a counter.
#[allow(clippy::too_many_arguments)]
fn render_property_values(
    ops: &mut Vec<Op>,
    counter: &CounterInfo,
    x: f32,
    y: f32,
    size_mm: f32,
    font_pt: f32,
    text_color: &Color,
    font: &PdfFontHandle,
) {
    let value_y = y + size_mm * 0.15;

    let positions: &[(usize, f32)] = match counter.values.len() {
        0 => return,
        1 => &[(0, 0.5)],
        2 => &[(0, 0.12), (1, 0.88)],
        _ => &[(0, 0.12), (1, 0.5), (2, 0.88)],
    };

    for &(idx, frac) in positions {
        if idx >= counter.values.len() {
            break;
        }
        let display = &counter.values[idx].1;
        let base_x = x + size_mm * frac;

        // Align: left-align first, center middle, right-align last.
        let text_x = if frac < 0.3 {
            base_x
        } else if frac > 0.7 {
            base_x - estimate_text_width(display, font_pt)
        } else {
            base_x - estimate_text_width(display, font_pt) * 0.5
        };

        ops.push(Op::StartTextSection);
        ops.push(Op::SetFillColor {
            col: text_color.clone(),
        });
        ops.push(Op::SetFont {
            font: font.clone(),
            size: Pt(font_pt),
        });
        ops.push(Op::SetTextCursor {
            pos: Point {
                x: Mm(text_x).into(),
                y: Mm(value_y).into(),
            },
        });
        ops.push(Op::ShowText {
            items: vec![TextItem::Text(display.clone())],
        });
        ops.push(Op::EndTextSection);
    }
}

/// Rough text width estimate using average character width for Helvetica.
fn estimate_text_width(text: &str, font_pt: f32) -> f32 {
    // Helvetica average character width ≈ 0.52 × font size.
    // Convert from points to mm: 1pt = 0.3528mm.
    let char_width_mm = font_pt * 0.52 * 0.3528;
    text.len() as f32 * char_width_mm
}

/// Truncate a name to fit within the counter width.
fn truncate_name(name: &str, size_mm: f32) -> String {
    // Estimate max chars that fit (using small font).
    let font_pt = size_mm * 0.28;
    let max_width = size_mm * 0.85;
    let char_width = font_pt * 0.52 * 0.3528;
    let max_chars = (max_width / char_width).floor() as usize;

    if name.len() <= max_chars {
        name.to_string()
    } else if max_chars > 2 {
        format!("{}..", &name[..max_chars - 2])
    } else {
        name[..max_chars].to_string()
    }
}
