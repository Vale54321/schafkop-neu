use bevy::{
    asset::{AssetMetaCheck, AssetPlugin, RenderAssetUsages},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    window::{WindowPlugin, Window},
};
use bevy::ecs::relationship::Relationship;
use schafkopf_logic::{
    deck::{Card, Deck, Rank, Suit},
    gamemode::Gamemode,
    player::{HumanPlayer, InternalPlayer},
};



const CARD_TEXTURE_WIDTH: usize = 112;
const CARD_TEXTURE_HEIGHT: usize = 190;
const CARD_WORLD_SIZE: Vec2 = Vec2::new(CARD_TEXTURE_WIDTH as f32, CARD_TEXTURE_HEIGHT as f32);
const GLYPH_WIDTH: usize = 5;
const GLYPH_HEIGHT: usize = 7;
const GLYPH_STRIDE: usize = 6;
const SUIT_ICON_PX: usize = 32;
const ATLAS_COLS: usize = 2;
const ATLAS_ROWS: usize = 5;
const LABEL_MARGIN_X: usize = 14;
const LABEL_MARGIN_Y: usize = 8;
const LABEL_TEXT_GAP: usize = 4;

#[derive(Resource)]
struct CurrentGamemode(Gamemode);

// Resource to hold the currently clicked card label
#[derive(Resource, Default)]
struct ClickedLabel(pub Option<String>);

// Marker for the UI text that shows the clicked card name
#[derive(Component)]
struct ClickText;

#[derive(Resource)]
struct SuitAtlas {
    texture: Handle<Image>,
    layout:  Handle<TextureAtlasLayout>,
}

#[derive(Resource)]
struct SauImage {
    texture: Handle<Image>,
}

impl SuitAtlas {
    fn load(
        asset_server: &AssetServer,
        layouts: &mut Assets<TextureAtlasLayout>,
    ) -> Self {
        let texture: Handle<Image> = asset_server.load("symbole.png");
        let layout = TextureAtlasLayout::from_grid(
            UVec2::splat(32),
            ATLAS_COLS as u32,
            ATLAS_ROWS as u32,
            None,
            None,
        );
        let layout_handle = layouts.add(layout);

        Self { texture, layout: layout_handle }
    }

    fn index_for(&self, suit: Suit) -> usize {
        match suit {
            Suit::Eichel => 0,
            Suit::Gras   => 1,
            Suit::Herz   => 2,
            Suit::Schell => 3,
        }
    }
}

#[derive(Resource)]
struct PlayerHandResource {
    cards: Vec<Card>,
}

// Marker for the base (non-atlas) sprite child under a card parent
#[derive(Component)]
struct BaseCardSprite;

// Resource to track if cards have been saved
#[derive(Resource, Default)]
struct CardsSaved(bool);

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
        )
        .init_resource::<CardsSaved>()
        .add_systems(Startup, (setup_game, spawn_click_text))
        // Spawn the player hand once the atlas image is fully loaded
        .add_systems(Update, (spawn_player_hand, save_all_cards))
        .add_systems(Update, update_click_text)
        .run();
}

fn setup_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2d);

    let atlas = SuitAtlas::load(&asset_server, &mut texture_layouts);
    commands.insert_resource(atlas);

    let sau_image = SauImage {
        texture: asset_server.load("schell_sau.png"),
    };
    commands.insert_resource(sau_image);

    let mut deck = Deck::new();
    deck.shuffle();
    let [mut hand1, hand2, hand3, hand4] =
        deck.deal_4x8().expect("expected a full deck to deal four hands");

    sort_cards(&mut hand1);

    let mut p1 = HumanPlayer::new(1, "Alice");
    let mut p2 = HumanPlayer::new(2, "Bob");
    let mut p3 = HumanPlayer::new(3, "Clara");
    let mut p4 = HumanPlayer::new(4, "Max");

    p1.set_hand(hand1);
    p2.set_hand(hand2);
    p3.set_hand(hand3);
    p4.set_hand(hand4);

    let mode = Gamemode::Wenz(None);
    commands.insert_resource(CurrentGamemode(mode));

    commands.insert_resource(ClickedLabel::default());

    let mut demo_cards = vec![
        Card { suit: Suit::Eichel, rank: Rank::Sieben },
        Card { suit: Suit::Gras, rank: Rank::Sieben },
        Card { suit: Suit::Eichel, rank: Rank::Acht },
        Card { suit: Suit::Gras, rank: Rank::Acht },
        Card { suit: Suit::Eichel, rank: Rank::Neun },
        Card { suit: Suit::Gras, rank: Rank::Neun },
        Card { suit: Suit::Eichel, rank: Rank::Zehn },
        Card { suit: Suit::Gras, rank: Rank::Zehn },
        Card { suit: Suit::Schell, rank: Rank::Zehn },
        Card { suit: Suit::Herz, rank: Rank::Zehn },
    ];
    sort_cards(&mut demo_cards);

    //commands.insert_resource(PlayerHandResource { cards: demo_cards });

    commands.insert_resource(PlayerHandResource {
        cards: p1.hand().clone(),
    });    
}

fn spawn_click_text(mut commands: Commands, _asset_server: Res<AssetServer>) {
    commands.spawn((
        Text::new("click a card"),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextLayout::new_with_justify(Justify::Left),
        Node {
            position_type: PositionType::Absolute,
            top: px(5),
            left: px(5),
            ..default()
        },
        ClickText,
    ));
}

fn save_all_cards(
    mut images: ResMut<Assets<Image>>,
    atlas: Option<Res<SuitAtlas>>,
    sau_image: Option<Res<SauImage>>,
    mut cards_saved: ResMut<CardsSaved>,
) {
    use std::fs;
    
    // Skip if already saved
    if cards_saved.0 {
        return;
    }

    // Check if atlas resource exists
    let Some(atlas) = atlas else {
        return;
    };

    // Check if sau_image resource exists
    let Some(sau_image) = sau_image else {
        return;
    };
    
    // Wait for atlas to load
    if images.get(&atlas.texture).and_then(|img| img.data.as_ref()).is_none() {
        return;
    }

    // Wait for sau image to load
    if images.get(&sau_image.texture).and_then(|img| img.data.as_ref()).is_none() {
        return;
    }

    // Mark as saved to prevent running again
    cards_saved.0 = true;

    // Create output directory
    let _ = fs::create_dir_all("generated_cards");

    // Generate all 32 cards
    let suits = [Suit::Eichel, Suit::Gras, Suit::Herz, Suit::Schell];
    let ranks = [
        Rank::Ass, Rank::Zehn, Rank::Koenig, Rank::Ober,
        Rank::Unter, Rank::Neun, Rank::Acht, Rank::Sieben,
    ];

    for suit in &suits {
        for rank in &ranks {
            let card = Card { suit: *suit, rank: *rank };
            let image_handle = create_card_texture(&mut images, &atlas, &sau_image, &card);
            
            if let Some(image) = images.get(&image_handle) {
                let filename = format!("generated_cards/{}_{}.png", 
                    format!("{:?}", suit).to_lowercase(),
                    format!("{:?}", rank).to_lowercase()
                );
                
                // Save using Bevy's DynamicImage
                if let Ok(dynamic_image) = image.clone().try_into_dynamic() {
                    if let Err(e) = dynamic_image.save(&filename) {
                        eprintln!("Failed to save {}: {}", filename, e);
                    } else {
                        println!("Saved {}", filename);
                    }
                }
            }
        }
    }
    
    println!("All cards saved to generated_cards/ directory");
}

fn spawn_player_hand(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    atlas: Res<SuitAtlas>,
    sau_image: Res<SauImage>,
    hand: Res<PlayerHandResource>,
    q_existing: Query<(), With<BaseCardSprite>>, // guard to spawn once
) {
    // Only proceed once the atlas image is loaded and has CPU-side pixel data
    if images.get(&atlas.texture).and_then(|img| img.data.as_ref()).is_none() {
        return;
    }

    // Prevent spawning multiple times (system runs every frame)
    if !q_existing.is_empty() {
        return;
    }
    let spacing = CARD_WORLD_SIZE.x + 5.0;
    let start_x = -(spacing * (hand.cards.len() as f32 - 1.0) / 2.0);
    let y = -200.0;

    for (i, card) in hand.cards.iter().enumerate() {
        let base_handle = create_card_texture(&mut images, &atlas, &sau_image, card);

        let parent = commands
            .spawn(Transform::from_xyz(start_x + i as f32 * spacing, y, 0.0))
            .observe(on_hover())
            .observe(on_unhover())
            .id();

        commands.entity(parent).with_children(|c| {
            c.spawn((
                Sprite {
                    image: base_handle,
                    custom_size: Some(CARD_WORLD_SIZE),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.0),
                Pickable::default(),
                BaseCardSprite,
            ))
            .observe(on_click_select(*card));
        });
    }
}
fn sort_cards(cards: &mut Vec<Card>) {
    cards.sort_by(|a, b| a.suit.cmp(&b.suit).then(a.rank.cmp(&b.rank)));
}


fn create_card_texture(images: &mut Assets<Image>, atlas: &SuitAtlas, sau_image: &SauImage, card: &Card) -> Handle<Image> {
    let mut pixels = vec![0u8; CARD_TEXTURE_WIDTH * CARD_TEXTURE_HEIGHT * 4];
    let top_h = CARD_TEXTURE_HEIGHT / 2;
    let border_gap = 9;
    let card_radius = 10;

    // Initialize with transparent background
    let transparent = [0, 0, 0, 0];
    for chunk in pixels.chunks_exact_mut(4) {
        chunk.copy_from_slice(&transparent);
    }

    let background = [255, 255, 255, 255];
    draw_rounded_rect_filled(
        &mut pixels,
        0,
        0,
        CARD_TEXTURE_WIDTH,
        top_h,
        card_radius,
        background,
    );

    // Blit a suit/rank pattern (pips) from the atlas into the base texture
    if let Some(atlas_img) = images.get(&atlas.texture) {
        let dest_x = (CARD_TEXTURE_WIDTH.saturating_sub(SUIT_ICON_PX)) / 2;
        let dest_y = top_h;

        // Build the pip layout for this card and blit it in one go
        let pattern = pip_layout(card, dest_x as i32, dest_y as i32);
        blit_pattern(
            &mut pixels,
            CARD_TEXTURE_WIDTH,
            CARD_TEXTURE_HEIGHT,
            atlas_img,
            &pattern,
        );
    }

    // Blit Sau image at the bottom for Ass cards
    if card.rank == Rank::Ass && card.suit == Suit::Schell {
        if let Some(sau_img) = images.get(&sau_image.texture) {
            let sau_width = 94;
            let sau_height = 86;
            // Center horizontally, position at bottom of top half
            let sau_x = (CARD_TEXTURE_WIDTH - sau_width) / 2;
            let sau_y = top_h - sau_height;
            blit_image(&mut pixels, CARD_TEXTURE_WIDTH, CARD_TEXTURE_HEIGHT, sau_img, sau_x, sau_y);
        }
    }

    // Draw a rounded rectangle border in the TOP half with 9px gap from card edge and 6px radius.
    // This will be mirrored to the bottom half later.
    let border_color = [45, 45, 45, 255];

    let radius = card_radius - border_gap + 2;

    draw_rounded_rect_border(
        &mut pixels,
        border_gap,
        border_gap,
        CARD_TEXTURE_WIDTH - border_gap,
        top_h,
        radius,
        border_color,
    );

    let rank_text = rank_label(card.rank);

    let ink = [15, 15, 15, 255];
    let white_bg = [255, 255, 255, 255];
    let rank_text_len = rank_text.chars().count();

    // Draw rank labels centered on the vertical border with white background
    let text_width = rank_text_len * GLYPH_STRIDE - 1; // -1 because last char has no trailing space
    let bg_padding = 1; // padding used in draw_text_with_bg
    let total_box_width = text_width + 2 * bg_padding;
    let vertical_padding = 5; // padding from the corner vertically
    
    // Top-left corner label - centered on the left border line
    let left_border_x = border_gap;
    let label_x_left = left_border_x - (total_box_width / 2) + bg_padding;
    let label_y = border_gap + radius + vertical_padding;
    draw_text_with_bg(&mut pixels, label_x_left, label_y, rank_text, ink, white_bg);

    // Top-right corner label - centered on the right border line
    let right_border_x = CARD_TEXTURE_WIDTH - border_gap - 1;
    let label_x_right = right_border_x - (total_box_width / 2) + bg_padding;
    draw_text_with_bg(&mut pixels, label_x_right, label_y, rank_text, ink, white_bg);

    // Mirror the entire top half of the card to the bottom half (rotated 180°)
    mirror_top_half_to_bottom(&mut pixels, CARD_TEXTURE_WIDTH, CARD_TEXTURE_HEIGHT);

    // No outer border needed - the card has rounded corners with transparency outside

    let extent = Extent3d {
        width: CARD_TEXTURE_WIDTH as u32,
        height: CARD_TEXTURE_HEIGHT as u32,
        depth_or_array_layers: 1,
    };

    let image = Image::new_fill(
        extent,
        TextureDimension::D2,
        &pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );

    images.add(image)
}

// Describes a single blit operation from the atlas into the card texture.
// x,y are absolute pixel coordinates for the top-left of the 32x32 icon within the card texture.
// rotation: 0=0°, 1=90° CW, 2=180°, 3=270° CW
#[derive(Clone, Copy, Debug)]
struct AtlasBlit {
    index: usize,
    x: i32,
    y: i32,
    rotation: u8,  // 0, 1, 2, or 3 for 0°, 90°, 180°, 270°
}

// Apply a list of AtlasBlit operations in order.
fn blit_pattern(
    dest_pixels: &mut [u8],
    dest_w: usize,
    dest_h: usize,
    atlas_img: &Image,
    pattern: &[AtlasBlit],
) {
    for op in pattern {
        // Skip negative origins; cast to usize only when non-negative
        if op.x < 0 || op.y < 0 { continue; }
        blit_atlas_icon_top_left(
            dest_pixels,
            dest_w,
            dest_h,
            atlas_img,
            op.index,
            op.x as usize,
            op.y as usize,
            op.rotation,
        );
    }
}

fn suit_to_atlas_index(suit: Suit) -> usize {
    match suit {
        Suit::Eichel => 0,
        Suit::Gras   => 1,
        Suit::Herz   => 2,
        Suit::Schell => 3,
    }
}

fn pip_layout(card: &Card, dest_x: i32, dest_y: i32) -> Vec<AtlasBlit> {
    use Rank::*;
    use Suit::*;

    let HORIZONTAL_SPACING = 29;
    let CROWN_SPACING = 94;

    match (card.suit, card.rank) {
        (Herz, Neun) => vec![
            AtlasBlit { index: 4, x: dest_x,     y: dest_y - 46, rotation: 0 },
            AtlasBlit { index: 5, x: dest_x,     y: dest_y - 65, rotation: 0 },
            AtlasBlit { index: 4, x: dest_x,     y: dest_y - 84, rotation: 0 },

            AtlasBlit { index: 2, x: dest_x - HORIZONTAL_SPACING, y: dest_y - 34, rotation: 0 },
            AtlasBlit { index: 2, x: dest_x - HORIZONTAL_SPACING, y: dest_y - 53, rotation: 0 },
            AtlasBlit { index: 2, x: dest_x - HORIZONTAL_SPACING, y: dest_y - 72, rotation: 0 },

            AtlasBlit { index: 2, x: dest_x + HORIZONTAL_SPACING, y: dest_y - 34, rotation: 0 },
            AtlasBlit { index: 2, x: dest_x + HORIZONTAL_SPACING, y: dest_y - 53, rotation: 0 },
            AtlasBlit { index: 2, x: dest_x + HORIZONTAL_SPACING, y: dest_y - 72, rotation: 0 },
        ],
        (Herz, Acht) => vec![
            AtlasBlit { index: 2, x: dest_x - HORIZONTAL_SPACING + 3, y: dest_y - 34, rotation: 0 },
            AtlasBlit { index: 2, x: dest_x - HORIZONTAL_SPACING + 3, y: dest_y - 50, rotation: 0 },
            AtlasBlit { index: 2, x: dest_x - HORIZONTAL_SPACING + 3, y: dest_y - 66, rotation: 0 },
            AtlasBlit { index: 2, x: dest_x - HORIZONTAL_SPACING + 3, y: dest_y - 82, rotation: 0 },

            AtlasBlit { index: 2, x: dest_x + HORIZONTAL_SPACING - 3, y: dest_y - 34, rotation: 0 },
            AtlasBlit { index: 2, x: dest_x + HORIZONTAL_SPACING - 3, y: dest_y - 50, rotation: 0 },
            AtlasBlit { index: 2, x: dest_x + HORIZONTAL_SPACING - 3, y: dest_y - 66, rotation: 0 },
            AtlasBlit { index: 2, x: dest_x + HORIZONTAL_SPACING - 3, y: dest_y - 82, rotation: 0 },
        ],
        (Herz, Sieben) => vec![
            AtlasBlit { index: 2, x: dest_x,     y: dest_y - 84, rotation: 0 },

            AtlasBlit { index: 2, x: dest_x - HORIZONTAL_SPACING, y: dest_y - 34, rotation: 0 },
            AtlasBlit { index: 2, x: dest_x - HORIZONTAL_SPACING, y: dest_y - 53, rotation: 0 },
            AtlasBlit { index: 2, x: dest_x - HORIZONTAL_SPACING, y: dest_y - 72, rotation: 0 },

            AtlasBlit { index: 2, x: dest_x + HORIZONTAL_SPACING, y: dest_y - 34, rotation: 0 },
            AtlasBlit { index: 2, x: dest_x + HORIZONTAL_SPACING, y: dest_y - 53, rotation: 0 },
            AtlasBlit { index: 2, x: dest_x + HORIZONTAL_SPACING, y: dest_y - 72, rotation: 0 },
        ],
        (Schell, Neun) => vec![
            AtlasBlit { index: 7, x: dest_x,     y: dest_y - 46, rotation: 0 },
            AtlasBlit { index: 3, x: dest_x,     y: dest_y - 46, rotation: 0 },
            AtlasBlit { index: 3, x: dest_x,     y: dest_y - 65, rotation: 0 },
            AtlasBlit { index: 3, x: dest_x,     y: dest_y - 84, rotation: 0 },

            AtlasBlit { index: 7, x: dest_x - HORIZONTAL_SPACING, y: dest_y - 36, rotation: 0 },
            AtlasBlit { index: 3, x: dest_x - HORIZONTAL_SPACING, y: dest_y - 36, rotation: 0 },
            AtlasBlit { index: 3, x: dest_x - HORIZONTAL_SPACING, y: dest_y - 54, rotation: 0 },
            AtlasBlit { index: 3, x: dest_x - HORIZONTAL_SPACING, y: dest_y - 70, rotation: 0 },

            AtlasBlit { index: 7, x: dest_x + HORIZONTAL_SPACING, y: dest_y - 36, rotation: 0 },
            AtlasBlit { index: 3, x: dest_x + HORIZONTAL_SPACING, y: dest_y - 36, rotation: 0 },
            AtlasBlit { index: 3, x: dest_x + HORIZONTAL_SPACING, y: dest_y - 54, rotation: 0 },
            AtlasBlit { index: 3, x: dest_x + HORIZONTAL_SPACING, y: dest_y - 70, rotation: 0 },
        ],
        (Schell, Acht) => vec![
            AtlasBlit { index: 7, x: dest_x - HORIZONTAL_SPACING + 3, y: dest_y - 34, rotation: 0 },
            AtlasBlit { index: 3, x: dest_x - HORIZONTAL_SPACING + 3, y: dest_y - 34, rotation: 0 },
            AtlasBlit { index: 3, x: dest_x - HORIZONTAL_SPACING + 3, y: dest_y - 50, rotation: 0 },
            AtlasBlit { index: 3, x: dest_x - HORIZONTAL_SPACING + 3, y: dest_y - 66, rotation: 0 },
            AtlasBlit { index: 3, x: dest_x - HORIZONTAL_SPACING + 3, y: dest_y - 82, rotation: 0 },

            AtlasBlit { index: 7, x: dest_x + HORIZONTAL_SPACING - 3, y: dest_y - 34, rotation: 0 },
            AtlasBlit { index: 3, x: dest_x + HORIZONTAL_SPACING - 3, y: dest_y - 34, rotation: 0 },
            AtlasBlit { index: 3, x: dest_x + HORIZONTAL_SPACING - 3, y: dest_y - 50, rotation: 0 },
            AtlasBlit { index: 3, x: dest_x + HORIZONTAL_SPACING - 3, y: dest_y - 66, rotation: 0 },
            AtlasBlit { index: 3, x: dest_x + HORIZONTAL_SPACING - 3, y: dest_y - 82, rotation: 0 },
        ],
        (Schell, Sieben) => vec![
            AtlasBlit { index: 7, x: dest_x,      y: dest_y - 84, rotation: 0 },
            AtlasBlit { index: 3, x: dest_x,      y: dest_y - 84, rotation: 0 },

            AtlasBlit { index: 7, x: dest_x - HORIZONTAL_SPACING, y: dest_y - 36, rotation: 0 },
            AtlasBlit { index: 3, x: dest_x - HORIZONTAL_SPACING, y: dest_y - 36, rotation: 0 },
            AtlasBlit { index: 3, x: dest_x - HORIZONTAL_SPACING, y: dest_y - 54, rotation: 0 },
            AtlasBlit { index: 3, x: dest_x - HORIZONTAL_SPACING, y: dest_y - 70, rotation: 0 },

            AtlasBlit { index: 7, x: dest_x + HORIZONTAL_SPACING, y: dest_y - 36, rotation: 0 },
            AtlasBlit { index: 3, x: dest_x + HORIZONTAL_SPACING, y: dest_y - 36, rotation: 0 },
            AtlasBlit { index: 3, x: dest_x + HORIZONTAL_SPACING, y: dest_y - 54, rotation: 0 },
            AtlasBlit { index: 3, x: dest_x + HORIZONTAL_SPACING, y: dest_y - 70, rotation: 0 },
        ],
        (Schell, Zehn) | (Herz, Zehn) => {
            let suit = card.suit;
            let id = suit_to_atlas_index(suit);
            let mut blits = Vec::new();

            let mut spacing = 16;

            // Right column
            for i in 0..3 {
                blits.push(AtlasBlit {
                    index: id,
                    x: dest_x + HORIZONTAL_SPACING,
                    y: dest_y - 36 - (i * spacing),
                    rotation: 0,
                });

                // Schell decor
                if (i==0 && suit==Schell) {
                    blits.push(AtlasBlit {
                        index: 7,
                        x: dest_x + HORIZONTAL_SPACING,
                        y: dest_y - 36 - (i * spacing),
                        rotation: 0,
                    });
                }
            }

            // Left column
            for i in 0..3 {
                blits.push(AtlasBlit {
                    index: id,
                    x: dest_x - HORIZONTAL_SPACING,
                    y: dest_y - 36 - (i * spacing),
                    rotation: 0,
                });

                // Schell decor
                if (i==0 && suit==Schell) {
                    blits.push(AtlasBlit {
                        index: 7,
                        x: dest_x - HORIZONTAL_SPACING,
                        y: dest_y - 36 - (i * spacing),
                        rotation: 0,
                    });
                }
            }

            let center_setoff = 29;

            // Center column
            if suit == Schell {
                // Schell pattern with decorative overlay on first pip
                blits.push(AtlasBlit { index: 7, x: dest_x, y: dest_y - center_setoff, rotation: 0 });
                for i in 0..4 {
                    blits.push(AtlasBlit {
                        index: 3,
                        x: dest_x,
                        y: dest_y - center_setoff - (i * spacing),
                        rotation: 0,
                    });
                }
            } else {
                // Herz pattern (alternating heart types)
                let indices = [5, 4, 5, 4];
                for i in 0..4 {
                    blits.push(AtlasBlit {
                        index: indices[i],
                        x: dest_x,
                        y: dest_y - 28 - (i as i32 * spacing),
                        rotation: 0,
                    });
                }
            }


            // Center bottom crown
            blits.push(AtlasBlit { index: 6, x: dest_x, y: dest_y - CROWN_SPACING, rotation: 0 });

            blits
        },
        (Gras, Acht) | (Eichel, Acht) => {
            let suit = card.suit;
            let id = suit_to_atlas_index(suit);
            let mut blits = Vec::new();

            let spacing = if suit == Eichel { 17 } else { 16 };
    
            // Right column 
            for i in 0..4 {
                blits.push(AtlasBlit {
                    index: id,
                    x: dest_x + HORIZONTAL_SPACING,
                    y: dest_y - 32 - (i * spacing),
                    rotation: 3,
                });
            }

            // Left column
            for i in 0..4 {
                blits.push(AtlasBlit {
                    index: id,
                    x: dest_x - HORIZONTAL_SPACING,
                    y: dest_y - 32 - (i * spacing),
                    rotation: 1,
                });
            }
    
            blits
        },
        (Gras, Zehn) | (Eichel, Zehn) => {
            let suit = card.suit;
            let id = suit_to_atlas_index(suit);
            let mut blits = Vec::new();

            let mut spacing = 13;
            if suit == Eichel {
                spacing = 14;
            }
    
            // Right column 
            for i in 0..5 {
                blits.push(AtlasBlit {
                    index: id,
                    x: dest_x + HORIZONTAL_SPACING,
                    y: dest_y - 32 - (i * spacing),
                    rotation: 3,
                });
            }

            // Left column
            for i in 0..5 {
                blits.push(AtlasBlit {
                    index: id,
                    x: dest_x - HORIZONTAL_SPACING,
                    y: dest_y - 32 - (i * spacing),
                    rotation: 1,
                });
            }

            // Center bottom crown
            blits.push(AtlasBlit { index: 6, x: dest_x, y: dest_y - CROWN_SPACING, rotation: 0 });
    
            blits
        },
        (_, Koenig) | (_, Ober) => {
            let id = suit_to_atlas_index(card.suit);
            vec![
                AtlasBlit { index: id, x: dest_x - 28, y: dest_y - 82, rotation: 0 },
            ]
        },
        (_, Unter) => {
            let id = suit_to_atlas_index(card.suit);
            vec![
                AtlasBlit { index: id, x: dest_x - 28, y: dest_y - 34, rotation: 0 },
            ]
        },
        (Schell, Ass) | (Herz, Ass) => {
            let id = if card.suit == Schell { 8 } else { 9 };
            vec![
                AtlasBlit { index: id, x: dest_x + 26, y: dest_y - 82, rotation: 0 },
                AtlasBlit { index: id, x: dest_x - 26, y: dest_y - 82, rotation: 1 },
            ]
        },
        (Gras, Ass) | (Eichel, Ass) => {
            let id = suit_to_atlas_index(card.suit);
            vec![
                AtlasBlit { index: id, x: dest_x + 26, y: dest_y - 82, rotation: 3 },
                AtlasBlit { index: id, x: dest_x - 26, y: dest_y - 82, rotation: 1 },
            ]
        },
        (_, _) => {
            let id = suit_to_atlas_index(card.suit);
            vec![
                AtlasBlit { index: id, x: dest_x, y: dest_y - 32, rotation: 0 },
            ]
        },
    }
}

fn blit_atlas_icon_top_left(
    dest_pixels: &mut [u8],
    dest_w: usize,
    dest_h: usize,
    atlas_img: &Image,
    index: usize,
    dest_x: usize,
    dest_y: usize,
    rotation: u8,
) {
    // Compute source top-left in the atlas based on index and known grid
    let col = index % ATLAS_COLS;
    let row = index / ATLAS_COLS;
    let src_x = col * SUIT_ICON_PX;
    let src_y = row * SUIT_ICON_PX;

    let atlas_w = SUIT_ICON_PX * ATLAS_COLS;
    let atlas_h = SUIT_ICON_PX * ATLAS_ROWS;

    // Clip to destination bounds
    let max_w = SUIT_ICON_PX.min(dest_w.saturating_sub(dest_x));
    let max_h = SUIT_ICON_PX.min(dest_h.saturating_sub(dest_y));

    if let Some(ref data) = atlas_img.data {
        for y in 0..max_h {
            for x in 0..max_w {
                // Apply rotation transformation to get source coordinates
                let (sx_offset, sy_offset) = match rotation {
                    0 => (x, y),                                    // 0°: no rotation
                    1 => (SUIT_ICON_PX - 1 - y, x),                // 90° CW
                    2 => (SUIT_ICON_PX - 1 - x, SUIT_ICON_PX - 1 - y), // 180°
                    3 => (y, SUIT_ICON_PX - 1 - x),                // 270° CW (or 90° CCW)
                    _ => (x, y),                                    // fallback
                };

                let sx = src_x + sx_offset;
                let sy = src_y + sy_offset;
                
                if sx >= atlas_w || sy >= atlas_h {
                    continue;
                }

                let dx = dest_x + x;
                let dy = dest_y + y;

                let s_index = (sy * atlas_w + sx) * 4;
                let d_index = (dy * dest_w + dx) * 4;

                if s_index + 4 <= data.len() && d_index + 4 <= dest_pixels.len() {
                    let a = data[s_index + 3];
                    if a == 0 { continue; }
                    dest_pixels[d_index + 0] = data[s_index + 0];
                    dest_pixels[d_index + 1] = data[s_index + 1];
                    dest_pixels[d_index + 2] = data[s_index + 2];
                    dest_pixels[d_index + 3] = 255; // opaque result
                }
            }
        }
    }
}

// Blit a full image onto the card texture at the specified position
fn blit_image(
    dest_pixels: &mut [u8],
    dest_w: usize,
    dest_h: usize,
    src_img: &Image,
    dest_x: usize,
    dest_y: usize,
) {
    if let Some(ref data) = src_img.data {
        let src_w = src_img.width() as usize;
        let src_h = src_img.height() as usize;

        for y in 0..src_h {
            for x in 0..src_w {
                let dx = dest_x + x;
                let dy = dest_y + y;

                // Skip if out of bounds
                if dx >= dest_w || dy >= dest_h {
                    continue;
                }

                let s_index = (y * src_w + x) * 4;
                let d_index = (dy * dest_w + dx) * 4;

                if s_index + 4 <= data.len() && d_index + 4 <= dest_pixels.len() {
                    let a = data[s_index + 3];
                    if a == 0 { continue; } // Skip transparent pixels
                    
                    dest_pixels[d_index + 0] = data[s_index + 0];
                    dest_pixels[d_index + 1] = data[s_index + 1];
                    dest_pixels[d_index + 2] = data[s_index + 2];
                    dest_pixels[d_index + 3] = 255; // opaque result
                }
            }
        }
    }
}

fn draw_rounded_rect_filled(
    pixels: &mut [u8],
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
    radius: usize,
    color: [u8; 4],
) {
    let r = radius as i32;
    
    for y in y1..y2 {
        for x in x1..x2 {
            let mut inside = false;
            
            // Check if we're in the main rectangle body (not in TOP corner regions)
            if x >= x1 + radius && x < x2.saturating_sub(radius) {
                // Middle horizontal section - always inside
                inside = true;
            } else if y >= y1 + radius {
                // Below the top corners - always inside (full width)
                inside = true;
            } else {
                // We're in a TOP corner region - check distance from corner center
                let (cx, cy) = if x < x1 + radius {
                    // Top-left corner
                    (x1 + radius, y1 + radius)
                } else {
                    // Top-right corner
                    (x2.saturating_sub(radius + 1), y1 + radius)
                };
                
                let dx = x as i32 - cx as i32;
                let dy = y as i32 - cy as i32;
                let dist_sq = dx * dx + dy * dy;
                
                if dist_sq <= r * r {
                    inside = true;
                }
            }
            
            if inside {
                set_pixel(pixels, x, y, color);
            }
        }
    }
}

fn draw_rounded_rect_border(
    pixels: &mut [u8],
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
    radius: usize,
    color: [u8; 4],
) {
    // Draw top horizontal line (excluding corners)
    for x in (x1 + radius)..(x2.saturating_sub(radius)) {
        set_pixel(pixels, x, y1, color);
    }

    // NO bottom horizontal line - it will be mirrored from the top

    // Draw left vertical line (excluding top corner, extending to bottom edge)
    for y in (y1 + radius)..y2 {
        set_pixel(pixels, x1, y, color);
    }

    // Draw right vertical line (excluding top corner, extending to bottom edge)
    for y in (y1 + radius)..y2 {
        set_pixel(pixels, x2.saturating_sub(1), y, color);
    }

    // Draw only the two top rounded corners
    draw_rounded_corner(pixels, x1 + radius, y1 + radius, radius, color, 0); // top-left
    draw_rounded_corner(pixels, x2.saturating_sub(radius + 1), y1 + radius, radius, color, 1); // top-right
}

fn draw_rounded_corner(
    pixels: &mut [u8],
    cx: usize,
    cy: usize,
    radius: usize,
    color: [u8; 4],
    quadrant: u8,
) {
    let r = radius as i32;
    for dy in -r..=r {
        for dx in -r..=r {
            let dist_sq = dx * dx + dy * dy;
            let inner_r_sq = (r - 1) * (r - 1);
            let outer_r_sq = r * r;

            // Only draw pixels on the circle edge (border)
            if dist_sq >= inner_r_sq && dist_sq <= outer_r_sq {
                let draw = match quadrant {
                    0 => dx <= 0 && dy <= 0, // top-left
                    1 => dx >= 0 && dy <= 0, // top-right
                    2 => dx <= 0 && dy >= 0, // bottom-left
                    3 => dx >= 0 && dy >= 0, // bottom-right
                    _ => false,
                };

                if draw {
                    let px = (cx as i32 + dx) as usize;
                    let py = (cy as i32 + dy) as usize;
                    set_pixel(pixels, px, py, color);
                }
            }
        }
    }
}

fn draw_text(pixels: &mut [u8], start_x: usize, start_y: usize, text: &str, color: [u8; 4]) {
    let mut x = start_x;
    for ch in text.chars() {
        if let Some(bitmap) = glyph_bitmap(ch) {
            draw_glyph(pixels, x, start_y, bitmap, color);
        }
        x += GLYPH_STRIDE;
    }
}

fn draw_text_with_bg(
    pixels: &mut [u8],
    start_x: usize,
    start_y: usize,
    text: &str,
    fg_color: [u8; 4],
    bg_color: [u8; 4],
) {
    let text_len = text.chars().count();
    let text_width = if text_len > 0 { text_len * GLYPH_STRIDE - 1 } else { 0 };
    let padding = 1;

    // Draw white background rectangle with padding
    let bg_x = start_x.saturating_sub(padding);
    let bg_y = start_y.saturating_sub(padding);
    let bg_width = text_width + 2 * padding;
    let bg_height = GLYPH_HEIGHT + 2 * padding;

    for y in bg_y..(bg_y + bg_height) {
        for x in bg_x..(bg_x + bg_width) {
            set_pixel(pixels, x, y, bg_color);
        }
    }

    // Draw the text on top
    draw_text(pixels, start_x, start_y, text, fg_color);
}

fn draw_glyph(pixels: &mut [u8], start_x: usize, start_y: usize, glyph: [u8; 7], color: [u8; 4]) {
    for (row, pattern) in glyph.iter().enumerate() {
        for col in 0..5 {
            if (pattern >> (4 - col)) & 1 == 1 {
                // Draw the pixel and one to the right for bold effect
                set_pixel(pixels, start_x + col, start_y + row, color);
                set_pixel(pixels, start_x + col + 1, start_y + row, color);
            }
        }
    }
}

fn set_pixel(pixels: &mut [u8], x: usize, y: usize, color: [u8; 4]) {
    if x >= CARD_TEXTURE_WIDTH || y >= CARD_TEXTURE_HEIGHT {
        return;
    }
    let index = (y * CARD_TEXTURE_WIDTH + x) * 4;
    pixels[index..index + 4].copy_from_slice(&color);
}

// Copy the top half of the image to the bottom half with a 180° rotation.
// For each pixel (x, y) in the top half, write it to (W-1-x, H-1-y).
fn mirror_top_half_to_bottom(pixels: &mut [u8], width: usize, height: usize) {
    let half_h = height / 2;
    for y in 0..half_h {
        for x in 0..width {
            let src_index = (y * width + x) * 4;
            let mx = width - 1 - x;
            let my = height - 1 - y;
            let dst_index = (my * width + mx) * 4;

            // Read first to avoid overlapping mutable/immutable borrows
            let rgba = [
                pixels[src_index],
                pixels[src_index + 1],
                pixels[src_index + 2],
                pixels[src_index + 3],
            ];
            pixels[dst_index] = rgba[0];
            pixels[dst_index + 1] = rgba[1];
            pixels[dst_index + 2] = rgba[2];
            pixels[dst_index + 3] = rgba[3];
        }
    }
}

fn glyph_bitmap(ch: char) -> Option<[u8; 7]> {
    match ch {
        '0' => Some([0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110]),
        '1' => Some([0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110]),
        '7' => Some([0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000]),
        '8' => Some([0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110]),
        '9' => Some([0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b11100]),
        'A' => Some([0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001]),
        'K' => Some([0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001]),
        'O' => Some([0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110]),
        'U' => Some([0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110]),
        _ => None,
    }
}

fn rank_label(rank: Rank) -> &'static str {
    match rank {
        Rank::Ass => "A",
        Rank::Zehn => "10",
        Rank::Koenig => "K",
        Rank::Ober => "O",
        Rank::Unter => "U",
        Rank::Neun => "9",
        Rank::Acht => "8",
        Rank::Sieben => "7",
    }
}

fn on_hover(
) -> impl Fn(
    On<Pointer<Over>>,
    Query<&mut Transform>,
    Query<&Children>,
    Query<(&mut Sprite, Option<&BaseCardSprite>)>,
    Query<&ChildOf>,
)
{
    move |ev, mut q_transform, q_children, mut q_sprite, q_parent| {
        // Determine the card parent entity from the event target
        let mut parent_entity = ev.event_target();
        if let Ok(parent) = q_parent.get(parent_entity) {
            parent_entity = parent.get();
        }

        // Scale the parent
        if let Ok(mut transform) = q_transform.get_mut(parent_entity) {
            transform.scale = Vec3::splat(1.1);
        }

        // Tint only the base sprite child (marked with BaseCardSprite)
        if let Ok(children) = q_children.get(parent_entity) {
            for child in children.iter() {
                if let Ok((mut sprite, maybe_base)) = q_sprite.get_mut(child) {
                    if maybe_base.is_some() {
                        sprite.color = Color::srgb(0.6, 0.6, 0.6);
                    }
                }
            }
        }
    }
}

fn on_unhover(
) -> impl Fn(
    On<Pointer<Out>>,
    Query<&mut Transform>,
    Query<&Children>,
    Query<(&mut Sprite, Option<&BaseCardSprite>)>,
    Query<&ChildOf>,
)
{
    move |ev, mut q_transform, q_children, mut q_sprite, q_parent| {
        // Determine the card parent entity from the event target
        let mut parent_entity = ev.event_target();
        if let Ok(parent) = q_parent.get(parent_entity) {
            parent_entity = parent.get();
        }

        // Reset parent scale
        if let Ok(mut transform) = q_transform.get_mut(parent_entity) {
            transform.scale = Vec3::ONE;
        }

        // Reset tint on the base sprite child
        if let Ok(children) = q_children.get(parent_entity) {
            for child in children.iter() {
                if let Ok((mut sprite, maybe_base)) = q_sprite.get_mut(child) {
                    if maybe_base.is_some() {
                        sprite.color = Color::WHITE;
                    }
                }
            }
        }
    }
}

fn on_click_select(card: Card) -> impl Fn(On<Pointer<Press>>, ResMut<ClickedLabel>) {
    move |_, mut clicked| {
        println!("Clicked on card: {:?}", card);
        clicked.0 = Some(format!("{} {}", card.suit, card.rank));
    }
}

fn update_click_text(mut q: Query<&mut Text, With<ClickText>>, clicked: Res<ClickedLabel>) {
    if let Some(mut text) = q.iter_mut().next() {
        if let Some(label) = &clicked.0 {
            *text = Text::new(label.clone());
        } else {
            *text = Text::new("click a card");
        }
    }
}