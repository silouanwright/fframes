//! A title card described as flexbox and solved with Taffy. The same tree is a lower third in
//! 16:9 and a centred card in 9:16; text leaves are measured with fframes' own line breaking.
use std::sync::{Arc, Mutex};

use fframes::{
    AudioMap, BreakLinesOpts, Color, Duration, FFramesContext, FontQuery, Frame, Svgr, TextAlign,
    Transform, Video, include_media_dir, svgr,
};
use taffy::prelude::*;

include_media_dir!(pub struct LayoutMedia, "examples/layout/media");

const FONT: &str = "DM Sans";
const LINE_HEIGHT: f32 = 1.25;
const PAD: f32 = 44.0;
const GAP: f32 = 18.0;
const LOGO: f32 = 120.0;
const MARGIN: f32 = 80.0;
const CARD_SECONDS: f32 = 3.0;

pub struct Card {
    pub name: &'static str,
    pub role: &'static str,
    pub place: &'static str,
}

pub const CARDS: [Card; 2] = [
    Card {
        name: "Lena Hart",
        role: "Director of Photography",
        place: "Northbridge Film School",
    },
    Card {
        name: "Alexandra Montgomery-Fitzgerald",
        role: "Professor Emerita of Medieval History and Director of the Centre for Manuscript Studies",
        place: "University of Northbridge, Faculty of Humanities",
    },
];

#[derive(Debug, Clone, Copy)]
struct TextStyle {
    size: usize,
    fill: &'static str,
}

const NAME: TextStyle = TextStyle {
    size: 60,
    fill: "#ffffff",
};
const ROLE: TextStyle = TextStyle {
    size: 34,
    fill: "#f1dca6",
};
const PLACE: TextStyle = TextStyle {
    size: 30,
    fill: "#c9d2df",
};

#[derive(Debug, Clone, Copy, Default)]
struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

#[derive(Debug)]
struct CardLayout {
    card: Rect,
    logo: Rect,
    divider: Rect,
    texts: [(Rect, &'static str, TextStyle); 3],
}

fn break_opts(
    style: TextStyle,
    width: usize,
    align: TextAlign,
    x: usize,
    y: usize,
) -> BreakLinesOpts<'static, usize, usize> {
    BreakLinesOpts {
        font: FontQuery {
            family: FONT,
            size: style.size,
            weight: 500,
            ..Default::default()
        },
        width,
        line_height: LINE_HEIGHT,
        x,
        y,
        align,
        fill: style.fill,
        ..Default::default()
    }
}

/// Size of `text` wrapped into `max_width`: the widest line and the height of all lines.
fn measure(
    frame: &mut Frame,
    ctx: &FFramesContext<'_, '_>,
    text: &str,
    style: TextStyle,
    max_width: f32,
) -> Option<Size<f32>> {
    let lines = frame.text_break_lines_structure(
        ctx,
        text,
        break_opts(style, max_width as usize, TextAlign::Left, 0, 0),
    )?;
    let width = lines.lines.iter().map(|line| line.width).max().unwrap_or(0);
    Some(Size {
        width: width as f32,
        height: lines.lines.len() as f32 * style.size as f32 * LINE_HEIGHT,
    })
}

/// A row (logo beside left-aligned text) in landscape, a centred column in portrait. Returns
/// `None` while a font is not resolved yet, so the caller retries on the next frame.
fn solve(
    card: &Card,
    width: f32,
    height: f32,
    frame: &mut Frame,
    ctx: &FFramesContext<'_, '_>,
) -> Option<CardLayout> {
    let landscape = width > height;
    let texts = [(card.name, NAME), (card.role, ROLE), (card.place, PLACE)];

    let mut tree: TaffyTree<usize> = TaffyTree::new();
    let leaves = [0, 1, 2].map(|i| tree.new_leaf_with_context(Style::default(), i).ok());
    let [Some(name), Some(role), Some(place)] = leaves else {
        return None;
    };
    let divider = tree
        .new_leaf(Style {
            size: Size {
                width: length(120.0),
                height: length(1.0),
            },
            ..Default::default()
        })
        .ok()?;
    let logo = tree
        .new_leaf(Style {
            size: Size {
                width: length(LOGO),
                height: length(LOGO),
            },
            flex_shrink: 0.0,
            ..Default::default()
        })
        .ok()?;
    let column = tree
        .new_with_children(
            Style {
                flex_direction: FlexDirection::Column,
                align_items: if landscape {
                    AlignItems::START
                } else {
                    AlignItems::CENTER
                },
                gap: Size {
                    width: length(0.0),
                    height: length(GAP),
                },
                max_size: Size {
                    width: length(if landscape {
                        900.0
                    } else {
                        (width - 2.0 * (MARGIN + PAD)).min(860.0)
                    }),
                    height: auto(),
                },
                ..Default::default()
            },
            &[name, role, divider, place],
        )
        .ok()?;
    let card_node = tree
        .new_with_children(
            Style {
                flex_direction: if landscape {
                    FlexDirection::Row
                } else {
                    FlexDirection::Column
                },
                align_items: AlignItems::CENTER,
                gap: Size {
                    width: length(2.0 * GAP),
                    height: length(2.0 * GAP),
                },
                padding: taffy::Rect {
                    left: length(PAD),
                    right: length(PAD),
                    top: length(PAD),
                    bottom: length(PAD),
                },
                ..Default::default()
            },
            &[logo, column],
        )
        .ok()?;
    let root = tree
        .new_with_children(
            Style {
                size: Size {
                    width: length(width),
                    height: length(height),
                },
                flex_direction: FlexDirection::Column,
                justify_content: if landscape {
                    JustifyContent::FLEX_END
                } else {
                    JustifyContent::CENTER
                },
                align_items: if landscape {
                    AlignItems::START
                } else {
                    AlignItems::CENTER
                },
                padding: taffy::Rect {
                    left: length(MARGIN),
                    right: length(MARGIN),
                    top: length(MARGIN),
                    bottom: length(MARGIN),
                },
                ..Default::default()
            },
            &[card_node],
        )
        .ok()?;

    let mut unresolved = false;
    tree.compute_layout_with_measure(root, Size::MAX_CONTENT, |inputs, _, leaf, style| {
        let leaf = leaf.copied();
        taffy::compute_leaf_layout(
            inputs,
            style,
            |_, _| 0.0,
            |known, available| {
                let Some(i) = leaf else { return Size::ZERO };
                let max_width = known.width.unwrap_or(match available.width {
                    AvailableSpace::Definite(w) => w,
                    AvailableSpace::MinContent => 0.0,
                    AvailableSpace::MaxContent => 1e6,
                });
                let (text, style) = texts[i];
                measure(frame, ctx, text, style, max_width).unwrap_or_else(|| {
                    unresolved = true;
                    Size::ZERO
                })
            },
        )
    })
    .ok()?;
    if unresolved {
        return None;
    }

    let absolute = |path: &[NodeId]| {
        let mut rect = Rect::default();
        for &node in path {
            let layout = tree.layout(node).ok()?;
            rect.x += layout.location.x;
            rect.y += layout.location.y;
            rect.w = layout.size.width;
            rect.h = layout.size.height;
        }
        Some(rect)
    };
    Some(CardLayout {
        card: absolute(&[root, card_node])?,
        logo: absolute(&[root, card_node, logo])?,
        divider: absolute(&[root, card_node, column, divider])?,
        texts: [
            (
                absolute(&[root, card_node, column, name])?,
                texts[0].0,
                texts[0].1,
            ),
            (
                absolute(&[root, card_node, column, role])?,
                texts[1].0,
                texts[1].1,
            ),
            (
                absolute(&[root, card_node, column, place])?,
                texts[2].0,
                texts[2].1,
            ),
        ],
    })
}

fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t.clamp(0.0, 1.0)).powi(3)
}

/// The cards in a `W`x`H` frame. Layout is solved on the first frame and reused by every
/// later frame on every render thread.
#[derive(Debug, Default)]
pub struct TitleCard<const W: usize, const H: usize> {
    layouts: Mutex<Option<Arc<Vec<CardLayout>>>>,
}

impl<const W: usize, const H: usize> TitleCard<W, H> {
    fn layouts(
        &self,
        frame: &mut Frame,
        ctx: &FFramesContext<'_, '_>,
    ) -> Option<Arc<Vec<CardLayout>>> {
        let mut cached = self.layouts.lock().ok()?;
        if let Some(layouts) = cached.as_ref() {
            return Some(layouts.clone());
        }
        let solved: Vec<CardLayout> = CARDS
            .iter()
            .map(|card| solve(card, W as f32, H as f32, frame, ctx))
            .collect::<Option<_>>()?;
        let solved = Arc::new(solved);
        *cached = Some(solved.clone());
        Some(solved)
    }
}

impl<const W: usize, const H: usize> Video for TitleCard<W, H> {
    const FPS: usize = 30;
    const WIDTH: usize = W;
    const HEIGHT: usize = H;
    const BACKGROUND_COLOR: Color = Color::BLACK;

    fn duration(&self) -> Duration<'_> {
        Duration::Seconds(CARDS.len() as f32 * CARD_SECONDS)
    }

    fn audio(&self) -> AudioMap<'_> {
        AudioMap::none()
    }

    fn render_frame<'a>(&'a self, mut frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let t = frame.seconds();
        let index = ((t / CARD_SECONDS) as usize).min(CARDS.len() - 1);
        let local = t - index as f32 * CARD_SECONDS;
        let shown =
            ease_out(local / 0.5) * (1.0 - ease_out((local - (CARD_SECONDS - 0.35)) / 0.35));

        let card = self.layouts(&mut frame, ctx).map(|layouts| {
            let l = &layouts[index];
            let align = if W > H { TextAlign::Left } else { TextAlign::Center };
            let texts: Svgr = l.texts.iter().filter_map(|(rect, text, style)| {
                let baseline = rect.y + style.size as f32 * 0.966;
                frame.text_break_lines(ctx, text, break_opts(*style, rect.w.ceil() as usize + 1, align, rect.x as usize, baseline as usize))
            }).collect();
            svgr!(
                <g opacity={shown} transform={Transform::translate(0.0, (1.0 - shown) * 24.0)}>
                    <rect x={l.card.x} y={l.card.y} width={l.card.w} height={l.card.h} rx="26" fill="rgba(6,8,12,0.75)" stroke="rgba(255,255,255,0.14)" stroke-width="1" />
                    <rect x={l.logo.x} y={l.logo.y} width={LOGO} height={LOGO} rx="28" fill="none" stroke="#f1dca6" stroke-width="3" />
                    <text x={l.logo.x + LOGO / 2.0} y={l.logo.y + LOGO * 0.68} text-anchor="middle" font-family={FONT} font-weight="500" font-size={LOGO * 0.5} fill="#f1dca6">"N"</text>
                    <rect x={l.divider.x} y={l.divider.y} width={l.divider.w} height="1" fill="rgba(255,255,255,0.22)" />
                    {texts}
                </g>
            )
        });

        svgr!(
            <svg xmlns="http://www.w3.org/2000/svg" viewBox={format!("0 0 {W} {H}")} width={ctx.current_video_size.width} height={ctx.current_video_size.height}>
                <defs>
                    <linearGradient id="bg" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0" stop-color="#1b2433" />
                        <stop offset="1" stop-color="#0a0d13" />
                    </linearGradient>
                </defs>
                <rect width={W} height={H} fill="url(#bg)" />
                {card}
            </svg>
        )
    }
}
